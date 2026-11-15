//! Original bounded readers for the observed UE3 868/3 AnimSet/AnimSequence.
//!
//! Saved metadata, omissions and source spans are retained. Decoding requires
//! explicit values for omitted format properties; it does not evaluate a pose,
//! retarget skeletons, convert coordinates, or apply translation-bone policies.

use crate::upk::objects::{
    Property, PropertyList, Span, TagDetail, Value, parse_tagged_properties,
};
use crate::upk::{MAX_EXPORT_BYTES, NameRef, Package};
use std::collections::{HashMap, HashSet};
use std::io;

const MAX_TRACKS: usize = 4096;
const MAX_SEQUENCES: usize = 65_536;
const MAX_FRAMES: usize = 1_000_000;
const MAX_KEYS: usize = 2_000_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedName {
    pub name: NameRef,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub package_guid: [u8; 16],
    pub export_index: usize,
    pub serial_offset: usize,
    pub net_index: i32,
    pub properties: PropertyList,
    pub native: Span,
    /// Bytes after the established native layout, never silently consumed.
    pub unknown_tail: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimSet {
    pub source: Source,
    pub track_bone_names: Vec<SavedName>,
    /// Signed UE object references in saved order. Imports remain imports.
    pub sequences: Vec<i32>,
    pub use_translation_bone_names: Option<Vec<SavedName>>,
    pub force_mesh_translation_bone_names: Option<Vec<SavedName>>,
    pub preview_mesh_name: Option<SavedName>,
    pub animation_rotation_only: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoneBinding {
    /// Exact resolved-name equality only. Missing names are retained as None.
    pub mesh_bone_to_track: Vec<Option<usize>>,
    pub unused_track_indices: Vec<usize>,
}

impl AnimSet {
    pub fn from_export(package: &Package, index: usize, bytes: &[u8]) -> io::Result<Self> {
        let source = read_source(package, index, bytes, "AnimSet")?;
        let properties = &source.properties;
        let track_bone_names = name_array(
            package,
            bytes,
            required(package, properties, "TrackBoneNames")?,
            MAX_TRACKS,
        )?;
        if track_bone_names.is_empty() {
            return Err(invalid("AnimSet has no track bone names"));
        }
        let references = integer_array(
            bytes,
            required(package, properties, "Sequences")?,
            MAX_SEQUENCES,
        )?;
        for &reference in &references {
            valid_reference(package, reference)?;
        }
        let use_translation_bone_names =
            optional_name_array(package, properties, bytes, "UseTranslationBoneNames")?;
        let force_mesh_translation_bone_names =
            optional_name_array(package, properties, bytes, "ForceMeshTranslationBoneNames")?;
        let preview_mesh_name = optional_name(package, properties, "PreviewSkelMeshName")?;
        let animation_rotation_only = optional_bool(package, properties, "bAnimRotationOnly")?;
        Ok(Self {
            source,
            track_bone_names,
            sequences: references,
            use_translation_bone_names,
            force_mesh_translation_bone_names,
            preview_mesh_name,
            animation_rotation_only,
        })
    }

    /// This is a name correspondence, not proof of compatible reference poses
    /// or of the runtime's selected skeletal mesh. No fallback or case folding.
    pub fn bind_exact_names(&self, mesh_names: &[&str]) -> io::Result<BoneBinding> {
        if mesh_names.len() > MAX_TRACKS {
            return Err(invalid("mesh bone count exceeds limit"));
        }
        let mut lookup = HashMap::with_capacity(self.track_bone_names.len());
        for (index, name) in self.track_bone_names.iter().enumerate() {
            if lookup.insert(name.text.as_str(), index).is_some() {
                return Err(invalid("AnimSet contains ambiguous duplicate bone names"));
            }
        }
        let mut seen = HashSet::with_capacity(mesh_names.len());
        let mut used = vec![false; self.track_bone_names.len()];
        let mut mesh_bone_to_track = Vec::with_capacity(mesh_names.len());
        for name in mesh_names {
            if !seen.insert(*name) {
                return Err(invalid("mesh contains ambiguous duplicate bone names"));
            }
            let track = lookup.get(name).copied();
            if let Some(index) = track {
                used[index] = true;
            }
            mesh_bone_to_track.push(track);
        }
        Ok(BoneBinding {
            mesh_bone_to_track,
            unused_track_indices: used
                .iter()
                .enumerate()
                .filter_map(|(index, used)| (!used).then_some(index))
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimSequence {
    pub source: Source,
    pub outer_reference: i32,
    pub name: SavedName,
    pub duration_seconds: f32,
    pub num_frames: usize,
    pub rate_scale: Option<f32>,
    pub no_looping_interpolation: Option<bool>,
    pub is_additive: Option<bool>,
    pub additive_reference_name: Option<SavedName>,
    pub translation_format: Option<SavedName>,
    pub rotation_format: Option<SavedName>,
    pub key_encoding: Option<SavedName>,
    pub compressed_track_offsets: Vec<i32>,
    pub offsets_source: Span,
    pub raw_track_count: usize,
    pub compressed_source: Span,
    pub compressed_bytes: Vec<u8>,
}

impl AnimSequence {
    pub fn from_export(package: &Package, index: usize, bytes: &[u8]) -> io::Result<Self> {
        let mut source = read_source(package, index, bytes, "AnimSequence")?;
        let properties = &source.properties;
        let name = property_name(package, required(package, properties, "SequenceName")?)?;
        let duration_seconds = property_float(required(package, properties, "SequenceLength")?)?;
        if duration_seconds <= 0.0 {
            return Err(invalid("animation duration must be positive"));
        }
        let Value::Integer(frame_count) = required(package, properties, "NumFrames")?.value else {
            return Err(invalid("NumFrames is not an integer"));
        };
        let num_frames =
            usize::try_from(frame_count).map_err(|_| invalid("negative animation frame count"))?;
        if num_frames == 0 || num_frames > MAX_FRAMES {
            return Err(invalid("animation frame count exceeds limits"));
        }
        let rate_scale = find(package, properties, "RateScale")?
            .map(property_float)
            .transpose()?;
        let no_looping_interpolation =
            optional_bool(package, properties, "bNoLoopingInterpolation")?;
        let is_additive = optional_bool(package, properties, "bIsAdditive")?;
        let additive_reference_name = optional_name(package, properties, "AdditiveRefName")?;
        let translation_format = enum_name(
            package,
            properties,
            "TranslationCompressionFormat",
            "AnimationCompressionFormat",
        )?;
        let rotation_format = enum_name(
            package,
            properties,
            "RotationCompressionFormat",
            "AnimationCompressionFormat",
        )?;
        let key_encoding = enum_name(
            package,
            properties,
            "KeyEncodingFormat",
            "AnimationKeyFormat",
        )?;
        let offsets = required(package, properties, "CompressedTrackOffsets")?;
        let compressed_track_offsets = integer_array(bytes, offsets, MAX_TRACKS * 4)?;
        let Value::Array {
            elements: offsets_source,
            ..
        } = offsets.value
        else {
            unreachable!()
        };
        let mut input = Input {
            bytes,
            position: source.native.offset,
        };
        let raw_track_count = input.count(MAX_TRACKS)?;
        if raw_track_count != 0 {
            return Err(unsupported(
                "nonempty raw animation tracks are not established",
            ));
        }
        let compressed_length = input.count(MAX_EXPORT_BYTES)?;
        let compressed_source = Span {
            offset: input.position,
            length: compressed_length,
        };
        let compressed_bytes = input.take(compressed_length)?.to_vec();
        source.unknown_tail = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        Ok(Self {
            source,
            outer_reference: package.exports[index].outer_index,
            name,
            duration_seconds,
            num_frames,
            rate_scale,
            no_looping_interpolation,
            is_additive,
            additive_reference_name,
            translation_format,
            rotation_format,
            key_encoding,
            compressed_track_offsets,
            offsets_source,
            raw_track_count,
            compressed_source,
            compressed_bytes,
        })
    }

    /// Decode the observed Float96 XYZ storage and optional explicit frame times.
    /// Every absent format requires an explicit caller value; no UObject default
    /// or track timing is silently inferred. No pose interpolation is performed.
    pub fn decode(&self, set: &AnimSet, defaults: &DecodeDefaults) -> io::Result<DecodedSequence> {
        if self.num_frames == 0
            || self.num_frames > MAX_FRAMES
            || self.compressed_source.length != self.compressed_bytes.len()
            || self
                .compressed_source
                .offset
                .checked_add(self.compressed_source.length)
                .is_none_or(|end| end > MAX_EXPORT_BYTES)
        {
            return Err(invalid(
                "animation decode metadata is inconsistent or exceeds limits",
            ));
        }
        let reference = i32::try_from(
            self.source
                .export_index
                .checked_add(1)
                .ok_or_else(|| invalid("sequence reference overflow"))?,
        )
        .map_err(|_| invalid("sequence reference overflow"))?;
        let set_reference = i32::try_from(
            set.source
                .export_index
                .checked_add(1)
                .ok_or_else(|| invalid("AnimSet reference overflow"))?,
        )
        .map_err(|_| invalid("AnimSet reference overflow"))?;
        if self.source.package_guid != set.source.package_guid
            || self.outer_reference != set_reference
            || !set.sequences.contains(&reference)
        {
            return Err(invalid("sequence is not a saved member of this AnimSet"));
        }
        let translation = resolve_format(
            self.translation_format.as_ref(),
            defaults.translation,
            Compression::parse,
        )?;
        let rotation = resolve_format(
            self.rotation_format.as_ref(),
            defaults.rotation,
            Compression::parse,
        )?;
        let key_encoding = resolve_format(
            self.key_encoding.as_ref(),
            defaults.key_encoding,
            KeyEncoding::parse,
        )?;
        let count = set.track_bone_names.len();
        if count == 0 || count > MAX_TRACKS || self.compressed_track_offsets.len() != count * 4 {
            return Err(invalid(
                "animation offset array does not contain four words per named track",
            ));
        }
        let mut tracks = Vec::with_capacity(count);
        let mut key_budget = MAX_KEYS;
        let mut occupied = Vec::with_capacity(count * 2);
        for (index, offsets) in self.compressed_track_offsets.chunks_exact(4).enumerate() {
            let translations = decode_keys::<3>(
                &self.compressed_bytes,
                self.compressed_source.offset,
                offsets[0],
                offsets[1],
                self.num_frames,
                translation.value,
                key_encoding.value,
                &mut key_budget,
            )?;
            let rotations = decode_keys::<4>(
                &self.compressed_bytes,
                self.compressed_source.offset,
                offsets[2],
                offsets[3],
                self.num_frames,
                rotation.value,
                key_encoding.value,
                &mut key_budget,
            )?;
            for range in [translations.source, rotations.source] {
                if range.length > 0 {
                    occupied.push(range);
                }
            }
            tracks.push(BoneTrack {
                name: set.track_bone_names[index].clone(),
                translations,
                rotations,
            });
        }
        let unclaimed_spans = unclaimed_ranges(self.compressed_source, &mut occupied)?;
        Ok(DecodedSequence {
            translation,
            rotation,
            key_encoding,
            tracks,
            unclaimed_spans,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compression {
    None,
    Float96NoW,
}
impl Compression {
    fn parse(name: &str) -> io::Result<Self> {
        if name.eq_ignore_ascii_case("ACF_None") {
            Ok(Self::None)
        } else if name.eq_ignore_ascii_case("ACF_Float96NoW") {
            Ok(Self::Float96NoW)
        } else {
            Err(unsupported(
                "animation compression codec is not implemented",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEncoding {
    ConstantKeyLerp,
    VariableKeyLerp,
}
impl KeyEncoding {
    fn parse(name: &str) -> io::Result<Self> {
        if name.eq_ignore_ascii_case("AKF_ConstantKeyLerp") {
            Ok(Self::ConstantKeyLerp)
        } else if name.eq_ignore_ascii_case("AKF_VariableKeyLerp") {
            Ok(Self::VariableKeyLerp)
        } else {
            Err(unsupported("animation key encoding is not implemented"))
        }
    }
}

/// No Default implementation: these are explicit caller assumptions for omitted
/// properties, not claims that the retail class defaults have been recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeDefaults {
    pub translation: Option<Compression>,
    pub rotation: Option<Compression>,
    pub key_encoding: Option<KeyEncoding>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatOrigin {
    Serialized,
    CallerSupplied,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedFormat<T> {
    pub value: T,
    pub origin: FormatOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameTimes {
    Absent,
    Single,
    /// Keys are uniformly spaced; no endpoint/loop interpolation policy applied.
    Uniform {
        num_frames: usize,
    },
    Explicit {
        frames: Vec<u32>,
        source: Span,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keys<const N: usize> {
    /// Original signed offset and count are retained, including absent offsets.
    pub saved_offset: i32,
    pub saved_count: usize,
    pub values: Vec<[f32; N]>,
    pub frames: FrameTimes,
    /// Complete occupied byte span, including key/time alignment padding.
    pub source: Span,
    pub key_source: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoneTrack {
    pub name: SavedName,
    pub translations: Keys<3>,
    /// X,Y,Z unchanged; W is the nonnegative reconstruction for Float96NoW.
    /// No conjugation, normalization, coordinate conversion or parent transform.
    pub rotations: Keys<4>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedSequence {
    pub translation: ResolvedFormat<Compression>,
    pub rotation: ResolvedFormat<Compression>,
    pub key_encoding: ResolvedFormat<KeyEncoding>,
    pub tracks: Vec<BoneTrack>,
    /// Gaps/unused bytes remain explicit; zero gaps is not runtime parity.
    pub unclaimed_spans: Vec<Span>,
}

fn resolve_format<T: Copy>(
    saved: Option<&SavedName>,
    fallback: Option<T>,
    parse: impl FnOnce(&str) -> io::Result<T>,
) -> io::Result<ResolvedFormat<T>> {
    if let Some(name) = saved {
        Ok(ResolvedFormat {
            value: parse(&name.text)?,
            origin: FormatOrigin::Serialized,
        })
    } else {
        Ok(ResolvedFormat {
            value: fallback.ok_or_else(|| {
                unsupported("omitted animation format requires an explicit caller value")
            })?,
            origin: FormatOrigin::CallerSupplied,
        })
    }
}

fn decode_keys<const N: usize>(
    bytes: &[u8],
    base: usize,
    offset: i32,
    count: i32,
    num_frames: usize,
    compression: Compression,
    encoding: KeyEncoding,
    budget: &mut usize,
) -> io::Result<Keys<N>> {
    if N != 3 && N != 4 {
        return Err(invalid("invalid animation channel dimension"));
    }
    let count = usize::try_from(count).map_err(|_| invalid("negative animation key count"))?;
    if count > num_frames || count > *budget {
        return Err(invalid("animation key count exceeds frame count or budget"));
    }
    *budget -= count;
    if count == 0 {
        if offset < -1 || (offset >= 0 && offset as usize > bytes.len()) {
            return Err(invalid("absent animation offset outside stream"));
        }
        let source = Span {
            offset: base + offset.max(0) as usize,
            length: 0,
        };
        return Ok(Keys {
            saved_offset: offset,
            saved_count: count,
            values: Vec::new(),
            frames: FrameTimes::Absent,
            source,
            key_source: source,
        });
    }
    let start = usize::try_from(offset).map_err(|_| invalid("negative animation data offset"))?;
    if start % 4 != 0 || start > bytes.len() {
        return Err(invalid(
            "animation track offset unaligned or outside stream",
        ));
    }
    let components = if N == 4 && count > 1 && compression == Compression::None {
        4
    } else {
        3
    };
    let key_bytes = count
        .checked_mul(components * 4)
        .ok_or_else(|| invalid("animation key span overflow"))?;
    let mut input = Input {
        bytes,
        position: start,
    };
    // Check the complete key range before allocating, even for hostile counts.
    if key_bytes > bytes.len() - start {
        return Err(invalid("animation keys exceed compressed stream"));
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut key = [0.0; N];
        for component in key.iter_mut().take(components) {
            *component = input.float()?;
        }
        if N == 4 && components == 3 {
            let remaining = 1.0 - (key[0] * key[0] + key[1] * key[1] + key[2] * key[2]);
            key[3] = if remaining > 0.0 {
                remaining.sqrt()
            } else {
                0.0
            };
        }
        values.push(key);
    }
    input.align4()?;
    let frames = if count == 1 {
        FrameTimes::Single
    } else if encoding == KeyEncoding::ConstantKeyLerp {
        FrameTimes::Uniform { num_frames }
    } else {
        let time_start = input.position;
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            let frame = if num_frames < 256 {
                u32::from(input.take(1)?[0])
            } else {
                u32::from(u16::from_le_bytes(input.take(2)?.try_into().unwrap()))
            };
            if frame as usize >= num_frames
                || frames.last().is_some_and(|previous| *previous >= frame)
            {
                return Err(invalid(
                    "animation frame times are outside range or not strictly increasing",
                ));
            }
            frames.push(frame);
        }
        let source = Span {
            offset: base + time_start,
            length: input.position - time_start,
        };
        input.align4()?;
        FrameTimes::Explicit { frames, source }
    };
    Ok(Keys {
        saved_offset: offset,
        saved_count: count,
        values,
        frames,
        source: Span {
            offset: base + start,
            length: input.position - start,
        },
        key_source: Span {
            offset: base + start,
            length: key_bytes,
        },
    })
}

fn unclaimed_ranges(stream: Span, spans: &mut [Span]) -> io::Result<Vec<Span>> {
    spans.sort_by_key(|span| span.offset);
    let end = stream
        .offset
        .checked_add(stream.length)
        .ok_or_else(|| invalid("compressed source overflow"))?;
    let mut position = stream.offset;
    let mut gaps = Vec::new();
    let mut previous = None;
    for span in spans {
        // Exact shared tracks are possible; partial overlaps remain invalid.
        if previous == Some(*span) {
            continue;
        }
        let next = span
            .offset
            .checked_add(span.length)
            .ok_or_else(|| invalid("track source overflow"))?;
        if span.offset < position || next > end {
            return Err(invalid("animation tracks overlap or exceed stream"));
        }
        if span.offset > position {
            gaps.push(Span {
                offset: position,
                length: span.offset - position,
            });
        }
        position = next;
        previous = Some(*span);
    }
    if position < end {
        gaps.push(Span {
            offset: position,
            length: end - position,
        });
    }
    Ok(gaps)
}

fn read_source(package: &Package, index: usize, bytes: &[u8], class: &str) -> io::Result<Source> {
    if (
        package.summary.file_version,
        package.summary.licensee_version,
    ) != (868, 3)
    {
        return Err(unsupported(
            "animation requires observed PC package profile 868/3",
        ));
    }
    let export = package
        .exports
        .get(index)
        .ok_or_else(|| invalid("animation export index out of bounds"))?;
    if bytes.len() != export.serial_size || bytes.len() > MAX_EXPORT_BYTES {
        return Err(invalid("animation export size mismatch or limit exceeded"));
    }
    export
        .serial_offset
        .checked_add(bytes.len())
        .ok_or_else(|| invalid("animation export range overflow"))?;
    if !package
        .export_class_name(index)
        .is_some_and(|name| name.eq_ignore_ascii_case(class))
    {
        return Err(unsupported("animation export class mismatch"));
    }
    if export.object_flags & 0x0200_0000_0000_0000 != 0 {
        return Err(unsupported("unestablished animation UObject preamble"));
    }
    let net_index = Input { bytes, position: 0 }.i32()?;
    let properties = parse_tagged_properties(package, bytes, 4)?;
    let native = Span {
        offset: properties.end_offset,
        length: bytes.len() - properties.end_offset,
    };
    Ok(Source {
        package_guid: package.summary.guid,
        export_index: index,
        serial_offset: export.serial_offset,
        net_index,
        properties,
        native,
        unknown_tail: native,
    })
}

fn find<'a>(
    package: &Package,
    properties: &'a PropertyList,
    name: &str,
) -> io::Result<Option<&'a Property>> {
    let mut found = None;
    for property in &properties.entries {
        if package
            .resolve_name(property.name)
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
        {
            if found.is_some() || property.array_index != 0 {
                return Err(invalid("duplicate or indexed animation property"));
            }
            found = Some(property);
        }
    }
    Ok(found)
}
fn required<'a>(
    package: &Package,
    properties: &'a PropertyList,
    name: &str,
) -> io::Result<&'a Property> {
    find(package, properties, name)?.ok_or_else(|| invalid("required animation property absent"))
}
fn property_name(package: &Package, property: &Property) -> io::Result<SavedName> {
    let Value::Name(name) = property.value else {
        return Err(invalid("animation name property has wrong type"));
    };
    saved_name(package, name)
}
fn saved_name(package: &Package, name: NameRef) -> io::Result<SavedName> {
    let text = package
        .resolve_name(name)
        .ok_or_else(|| invalid("unresolved animation name"))?;
    if text.len() > 4096 {
        return Err(invalid("animation name too long"));
    }
    Ok(SavedName { name, text })
}
fn optional_name(
    package: &Package,
    properties: &PropertyList,
    name: &str,
) -> io::Result<Option<SavedName>> {
    find(package, properties, name)?
        .map(|property| property_name(package, property))
        .transpose()
}
fn enum_name(
    package: &Package,
    properties: &PropertyList,
    name: &str,
    enumeration: &str,
) -> io::Result<Option<SavedName>> {
    find(package, properties, name)?
        .map(|property| {
            let TagDetail::Enumeration(tag) = property.detail else {
                return Err(invalid("animation enum lacks enumeration tag"));
            };
            if !package
                .resolve_name(tag)
                .is_some_and(|name| name.eq_ignore_ascii_case(enumeration))
            {
                return Err(invalid("animation enum tag type mismatch"));
            }
            property_name(package, property)
        })
        .transpose()
}
fn property_float(property: &Property) -> io::Result<f32> {
    let Value::FloatBits(bits) = property.value else {
        return Err(invalid("animation float property has wrong type"));
    };
    let value = f32::from_bits(bits);
    if !value.is_finite() {
        return Err(invalid("nonfinite animation value"));
    }
    Ok(value)
}
fn optional_bool(
    package: &Package,
    properties: &PropertyList,
    name: &str,
) -> io::Result<Option<bool>> {
    find(package, properties, name)?
        .map(|property| match property.value {
            Value::Boolean(value) => Ok(value),
            _ => Err(invalid("animation bool property has wrong type")),
        })
        .transpose()
}
fn array_bytes<'a>(
    bytes: &'a [u8],
    property: &Property,
    limit: usize,
    width: usize,
) -> io::Result<(usize, &'a [u8])> {
    let Value::Array { count, elements } = property.value else {
        return Err(invalid("animation array property has wrong type"));
    };
    if count > limit || count.checked_mul(width) != Some(elements.length) {
        return Err(invalid("animation array length mismatch or limit exceeded"));
    }
    Ok((count, elements.bytes(bytes)?))
}
fn integer_array(bytes: &[u8], property: &Property, limit: usize) -> io::Result<Vec<i32>> {
    let (_, elements) = array_bytes(bytes, property, limit, 4)?;
    Ok(elements
        .chunks_exact(4)
        .map(|word| i32::from_le_bytes(word.try_into().unwrap()))
        .collect())
}
fn name_array(
    package: &Package,
    bytes: &[u8],
    property: &Property,
    limit: usize,
) -> io::Result<Vec<SavedName>> {
    let (count, elements) = array_bytes(bytes, property, limit, 8)?;
    let mut input = Input {
        bytes: elements,
        position: 0,
    };
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let index = input.count(package.names.len().saturating_sub(1))?;
        let number = input.u32()?;
        names.push(saved_name(package, NameRef { index, number })?);
    }
    Ok(names)
}
fn optional_name_array(
    package: &Package,
    properties: &PropertyList,
    bytes: &[u8],
    name: &str,
) -> io::Result<Option<Vec<SavedName>>> {
    find(package, properties, name)?
        .map(|property| name_array(package, bytes, property, MAX_TRACKS))
        .transpose()
}
fn valid_reference(package: &Package, reference: i32) -> io::Result<()> {
    let valid = if reference > 0 {
        reference as usize <= package.exports.len()
    } else if reference < 0 {
        -(i64::from(reference)) as usize <= package.imports.len()
    } else {
        true
    };
    if !valid {
        return Err(invalid("animation object reference out of bounds"));
    }
    Ok(())
}

struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Input<'a> {
    fn take(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("animation read overflow"))?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("truncated animation payload"))?;
        self.position = end;
        Ok(result)
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, maximum: usize) -> io::Result<usize> {
        let count =
            usize::try_from(self.i32()?).map_err(|_| invalid("negative animation count"))?;
        if count > maximum {
            return Err(invalid("animation count exceeds limit"));
        }
        Ok(count)
    }
    fn float(&mut self) -> io::Result<f32> {
        let value = f32::from_bits(self.u32()?);
        if !value.is_finite() {
            return Err(invalid("nonfinite animation key"));
        }
        Ok(value)
    }
    fn align4(&mut self) -> io::Result<()> {
        let padding = (4 - self.position % 4) % 4;
        self.take(padding)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    #[test]
    fn single_rotation_and_explicit_frames_preserve_source_coordinates() {
        let bytes = floats(&[0.0, 0.6, 0.0]);
        // Single keys use Float96 even when the sequence format is None.
        let key = decode_keys::<4>(
            &bytes,
            80,
            0,
            1,
            7,
            Compression::None,
            KeyEncoding::VariableKeyLerp,
            &mut MAX_KEYS,
        )
        .unwrap();
        assert_eq!(key.values[0][0..3], [0.0, 0.6, 0.0]);
        assert!((key.values[0][3] - 0.8).abs() < 1e-6);
        assert_eq!(key.frames, FrameTimes::Single);
        assert_eq!(
            key.source,
            Span {
                offset: 80,
                length: 12
            }
        );
        let mut bytes = floats(&[3.0, -2.0, 1.0, 6.0, 8.0, 4.0]);
        bytes.extend_from_slice(&[0, 6, 0xff, 0xff]);
        let keys = decode_keys::<3>(
            &bytes,
            80,
            0,
            2,
            7,
            Compression::None,
            KeyEncoding::VariableKeyLerp,
            &mut MAX_KEYS,
        )
        .unwrap();
        assert_eq!(keys.values, [[3.0, -2.0, 1.0], [6.0, 8.0, 4.0]]);
        assert_eq!(
            keys.frames,
            FrameTimes::Explicit {
                frames: vec![0, 6],
                source: Span {
                    offset: 104,
                    length: 2
                }
            }
        );
        assert_eq!(keys.source.length, 28);
        for end in 0..bytes.len() {
            assert!(
                decode_keys::<3>(
                    &bytes[..end],
                    0,
                    0,
                    2,
                    7,
                    Compression::None,
                    KeyEncoding::VariableKeyLerp,
                    &mut MAX_KEYS
                )
                .is_err()
            );
        }
    }

    #[test]
    fn frame_width_boundary_bad_times_and_hostile_counts() {
        let mut bytes = floats(&[0.0; 6]);
        bytes.extend_from_slice(&[0, 0, 255, 0]);
        let keys = decode_keys::<3>(
            &bytes,
            0,
            0,
            2,
            256,
            Compression::Float96NoW,
            KeyEncoding::VariableKeyLerp,
            &mut MAX_KEYS,
        )
        .unwrap();
        assert_eq!(
            keys.frames,
            FrameTimes::Explicit {
                frames: vec![0, 255],
                source: Span {
                    offset: 24,
                    length: 4
                }
            }
        );
        for times in [[0, 0, 0, 0], [0, 0, 0, 1], [3, 0, 2, 0]] {
            bytes[24..].copy_from_slice(&times);
            assert!(
                decode_keys::<3>(
                    &bytes,
                    0,
                    0,
                    2,
                    256,
                    Compression::None,
                    KeyEncoding::VariableKeyLerp,
                    &mut MAX_KEYS
                )
                .is_err()
            );
        }
        for (offset, count, budget) in [
            (-1, 1, MAX_KEYS),
            (1, 1, MAX_KEYS),
            (32, 1, MAX_KEYS),
            (0, -1, MAX_KEYS),
            (0, i32::MAX, MAX_KEYS),
            (0, 2, 1),
        ] {
            assert!(
                decode_keys::<3>(
                    &bytes,
                    0,
                    offset,
                    count,
                    256,
                    Compression::None,
                    KeyEncoding::ConstantKeyLerp,
                    &mut { budget }
                )
                .is_err()
            );
        }
        bytes[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(
            decode_keys::<3>(
                &bytes,
                0,
                0,
                1,
                256,
                Compression::None,
                KeyEncoding::ConstantKeyLerp,
                &mut MAX_KEYS
            )
            .is_err()
        );
    }

    #[test]
    fn omitted_and_unknown_formats_never_silently_fall_back() {
        assert!(resolve_format::<Compression>(None, None, Compression::parse).is_err());
        let actual = resolve_format(None, Some(Compression::None), Compression::parse).unwrap();
        assert_eq!(actual.origin, FormatOrigin::CallerSupplied);
        let saved = SavedName {
            name: NameRef {
                index: 0,
                number: 0,
            },
            text: "ACF_Float96NoW".into(),
        };
        let actual =
            resolve_format(Some(&saved), Some(Compression::None), Compression::parse).unwrap();
        assert_eq!(
            actual,
            ResolvedFormat {
                value: Compression::Float96NoW,
                origin: FormatOrigin::Serialized
            }
        );
        let unknown = SavedName {
            text: "ACF_NewCodec".into(),
            ..saved
        };
        assert!(
            resolve_format(Some(&unknown), Some(Compression::None), Compression::parse).is_err()
        );
    }

    #[test]
    fn overlap_and_unclaimed_bytes_are_explicit() {
        let stream = Span {
            offset: 100,
            length: 40,
        };
        let a = Span {
            offset: 104,
            length: 12,
        };
        let b = Span {
            offset: 120,
            length: 12,
        };
        assert_eq!(
            unclaimed_ranges(stream, &mut [b, a, a]).unwrap(),
            vec![
                Span {
                    offset: 100,
                    length: 4
                },
                Span {
                    offset: 116,
                    length: 4
                },
                Span {
                    offset: 132,
                    length: 8
                }
            ]
        );
        assert!(
            unclaimed_ranges(
                stream,
                &mut [
                    a,
                    Span {
                        offset: 112,
                        length: 12
                    }
                ]
            )
            .is_err()
        );
        assert!(
            unclaimed_ranges(
                stream,
                &mut [Span {
                    offset: 136,
                    length: 12
                }]
            )
            .is_err()
        );
    }
}
