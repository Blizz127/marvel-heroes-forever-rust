//! Saved scene properties for the observed UE3 868/3 frontend map profile.
//!
//! These are serialized values, not evaluated world transforms. Missing fields
//! remain absent; archetypes, imported objects and native tails are not loaded.

use crate::upk::objects::{
    Property, PropertyList, Span, TagDetail, Value, parse_tagged_properties,
};
use crate::upk::{MAX_EXPORT_BYTES, Package};
use std::collections::HashSet;
use std::io;

const MAX_REFERENCES: usize = 4096;
const MAX_OUTER_DEPTH: usize = 64;
const MAX_REFERENCE_NAME_BYTES: usize = 4096;
const OBSERVED_ACTOR_PREFIX_FLAG: u64 = 0x0200_0000_0000_0000;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneObjectKind {
    Actor,
    Component,
}

/// Package-local identity. An import stays an import; no filename is inferred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectReference {
    /// Zero is null, negative is import, positive is one-based export.
    pub index: i32,
    /// Outer names first, local name last. Components remain separate strings,
    /// avoiding ambiguity when a name itself contains a period.
    pub names: Vec<String>,
    pub class_name: Option<String>,
}

impl ObjectReference {
    pub fn resolve(package: &Package, index: i32) -> io::Result<Self> {
        if index == 0 {
            return Ok(Self {
                index,
                names: Vec::new(),
                class_name: None,
            });
        }
        let class_name = if index > 0 {
            package.export_class_name(index as usize - 1)
        } else {
            package
                .imports
                .get(index.unsigned_abs() as usize - 1)
                .and_then(|entry| package.resolve_name(entry.class_name))
        }
        .ok_or_else(|| invalid("scene reference class is unresolved"))?;
        if class_name.len() > MAX_REFERENCE_NAME_BYTES {
            return Err(invalid("scene reference class name exceeds limit"));
        }
        let mut names = Vec::new();
        let mut visited = HashSet::new();
        let mut current = index;
        let mut name_bytes = 0usize;
        while current != 0 {
            if names.len() == MAX_OUTER_DEPTH || !visited.insert(current) {
                return Err(invalid("scene reference outer chain is cyclic or too deep"));
            }
            let (name, outer) = if current > 0 {
                let entry = package
                    .exports
                    .get(current as usize - 1)
                    .ok_or_else(|| invalid("scene export reference is out of bounds"))?;
                (entry.object_name, entry.outer_index)
            } else {
                let entry = package
                    .imports
                    .get(current.unsigned_abs() as usize - 1)
                    .ok_or_else(|| invalid("scene import reference is out of bounds"))?;
                (entry.object_name, entry.outer_index)
            };
            let name = package
                .resolve_name(name)
                .ok_or_else(|| invalid("scene reference name is unresolved"))?;
            name_bytes = name_bytes
                .checked_add(name.len())
                .ok_or_else(|| invalid("scene reference name length overflow"))?;
            if name_bytes > MAX_REFERENCE_NAME_BYTES {
                return Err(invalid("scene reference outer names exceed limit"));
            }
            names.push(name);
            current = outer;
        }
        names.reverse();
        Ok(Self {
            index,
            names,
            class_name: Some(class_name),
        })
    }
}

/// Index into `SavedSceneObject::properties.entries` plus the decoded value.
/// The original tag/payload spans remain available through that property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedField<T> {
    pub property_index: usize,
    pub value: T,
}

/// Raw float bits preserve signed zero and nonfinite values. Rotation retains
/// three signed serialized integers; this reader does not choose angle units.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SavedPlacement {
    pub location: Option<SavedField<[u32; 3]>>,
    pub relative_location: Option<SavedField<[u32; 3]>>,
    pub translation: Option<SavedField<[u32; 3]>>,
    pub rotation: Option<SavedField<[i32; 3]>>,
    pub scale: Option<SavedField<u32>>,
    pub scale3d: Option<SavedField<[u32; 3]>>,
    pub draw_scale: Option<SavedField<u32>>,
    pub draw_scale3d: Option<SavedField<[u32; 3]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedSceneObject {
    pub export_index: usize,
    pub serial_offset: usize,
    pub kind: SceneObjectKind,
    pub object: ObjectReference,
    pub outer: ObjectReference,
    pub archetype: ObjectReference,
    /// Preserved observed preamble, without assigning unverified word meanings.
    pub prefix: Vec<u8>,
    pub properties: PropertyList,
    pub native_payload: Span,
    pub placement: SavedPlacement,
    pub static_mesh: Option<SavedField<ObjectReference>>,
    pub base: Option<SavedField<ObjectReference>>,
    pub materials: Option<SavedField<Vec<ObjectReference>>>,
    pub static_mesh_components: Option<SavedField<Vec<ObjectReference>>>,
    pub light_components: Option<SavedField<Vec<ObjectReference>>>,
}

/// Only classes with established prefix offsets in the two inspected maps.
pub fn supported_class(name: &str) -> Option<SceneObjectKind> {
    match name.to_ascii_lowercase().as_str() {
        "brushcomponent"
        | "cylindercomponent"
        | "directionallightcomponent"
        | "fogvolumesphericaldensitycomponent"
        | "particlesystemcomponent"
        | "pointlightcomponent"
        | "skeletalmeshcomponent"
        | "staticmeshcomponent"
        | "modelcomponent" => Some(SceneObjectKind::Component),
        "cameraactor"
        | "emitter"
        | "fogvolumesphericaldensityinfo"
        | "lightmassimportancevolume"
        | "playerstart"
        | "postprocessvolume"
        | "skeletalmeshactor"
        | "staticlightcollectionactor"
        | "staticmeshcollectionactor"
        | "worldinfo" => Some(SceneObjectKind::Actor),
        _ => None,
    }
}

impl SavedSceneObject {
    pub fn from_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported("scene properties require UE3 profile 868/3"));
        }
        let export = package
            .exports
            .get(export_index)
            .ok_or_else(|| invalid("scene export index is out of bounds"))?;
        if bytes.len() != export.serial_size || bytes.len() > MAX_EXPORT_BYTES {
            return Err(invalid("scene export size mismatch or limit exceeded"));
        }
        export
            .serial_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("scene logical range overflow"))?;
        let class = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("scene export class is unresolved"))?;
        let kind = supported_class(&class)
            .ok_or_else(|| unsupported(format!("unestablished scene prefix for {class}")))?;
        let prefix_size = match kind {
            SceneObjectKind::Component => {
                if export.object_flags & OBSERVED_ACTOR_PREFIX_FLAG != 0 {
                    return Err(unsupported("component has an unobserved actor prefix flag"));
                }
                8
            }
            SceneObjectKind::Actor => {
                let collection = class.eq_ignore_ascii_case("staticmeshcollectionactor")
                    || class.eq_ignore_ascii_case("staticlightcollectionactor");
                match (
                    collection,
                    export.object_flags & OBSERVED_ACTOR_PREFIX_FLAG != 0,
                ) {
                    (true, false) => 4,
                    (false, true) => 26,
                    _ => {
                        return Err(unsupported(
                            "unobserved actor class/prefix flag combination",
                        ));
                    }
                }
            }
        };
        let prefix = bytes
            .get(..prefix_size)
            .ok_or_else(|| invalid("truncated scene preamble"))?
            .to_vec();
        let properties = parse_tagged_properties(package, bytes, prefix_size)?;
        let native_payload = Span {
            offset: properties.end_offset,
            length: bytes.len() - properties.end_offset,
        };
        native_payload.logical_offset(export.serial_offset)?;
        let object_index = i32::try_from(
            export_index
                .checked_add(1)
                .ok_or_else(|| invalid("scene object index overflow"))?,
        )
        .map_err(|_| invalid("scene object index exceeds signed range"))?;
        let mut result = Self {
            export_index,
            serial_offset: export.serial_offset,
            kind,
            object: ObjectReference::resolve(package, object_index)?,
            outer: ObjectReference::resolve(package, export.outer_index)?,
            archetype: ObjectReference::resolve(package, export.archetype_index)?,
            prefix,
            properties,
            native_payload,
            placement: SavedPlacement::default(),
            static_mesh: None,
            base: None,
            materials: None,
            static_mesh_components: None,
            light_components: None,
        };
        let mut reference_count = 0usize;
        for (index, property) in result.properties.entries.iter().enumerate() {
            let name = package
                .resolve_name(property.name)
                .ok_or_else(|| invalid("scene property name is unresolved"))?;
            match name.to_ascii_lowercase().as_str() {
                "location" => assign(
                    &mut result.placement.location,
                    index,
                    property,
                    vector(package, property, bytes)?,
                )?,
                "relativelocation" => assign(
                    &mut result.placement.relative_location,
                    index,
                    property,
                    vector(package, property, bytes)?,
                )?,
                "translation" => assign(
                    &mut result.placement.translation,
                    index,
                    property,
                    vector(package, property, bytes)?,
                )?,
                "rotation" => assign(
                    &mut result.placement.rotation,
                    index,
                    property,
                    rotator(package, property, bytes)?,
                )?,
                "scale3d" => assign(
                    &mut result.placement.scale3d,
                    index,
                    property,
                    vector(package, property, bytes)?,
                )?,
                "drawscale3d" => assign(
                    &mut result.placement.draw_scale3d,
                    index,
                    property,
                    vector(package, property, bytes)?,
                )?,
                "scale" => assign(
                    &mut result.placement.scale,
                    index,
                    property,
                    float(property)?,
                )?,
                "drawscale" => assign(
                    &mut result.placement.draw_scale,
                    index,
                    property,
                    float(property)?,
                )?,
                "staticmesh" | "base" => {
                    let Value::Object(reference) = property.value else {
                        return Err(invalid("scene object field is not an object reference"));
                    };
                    let field = if name.eq_ignore_ascii_case("base") {
                        &mut result.base
                    } else {
                        &mut result.static_mesh
                    };
                    assign(
                        field,
                        index,
                        property,
                        ObjectReference::resolve(package, reference)?,
                    )?;
                }
                "materials" | "staticmeshcomponents" | "lightcomponents" => {
                    let refs = reference_array(package, property, bytes, &mut reference_count)?;
                    let field = match name.to_ascii_lowercase().as_str() {
                        "materials" => &mut result.materials,
                        "staticmeshcomponents" => &mut result.static_mesh_components,
                        _ => &mut result.light_components,
                    };
                    assign(field, index, property, refs)?;
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

fn assign<T>(
    target: &mut Option<SavedField<T>>,
    index: usize,
    property: &Property,
    value: T,
) -> io::Result<()> {
    if property.array_index != 0 || target.is_some() {
        return Err(invalid("duplicate or indexed singleton scene property"));
    }
    *target = Some(SavedField {
        property_index: index,
        value,
    });
    Ok(())
}

fn structure<'a>(
    package: &Package,
    property: &Property,
    bytes: &'a [u8],
    expected: &str,
) -> io::Result<&'a [u8]> {
    let TagDetail::Structure(name) = property.detail else {
        return Err(invalid("scene placement field is not a structure"));
    };
    if !package
        .resolve_name(name)
        .is_some_and(|name| name.eq_ignore_ascii_case(expected))
    {
        return Err(invalid("scene placement structure type mismatch"));
    }
    let raw = property.payload.bytes(bytes)?;
    if raw.len() != 12 {
        return Err(invalid(
            "scene placement structure must contain three words",
        ));
    }
    Ok(raw)
}
fn vector(package: &Package, property: &Property, bytes: &[u8]) -> io::Result<[u32; 3]> {
    let raw = structure(package, property, bytes, "Vector")?;
    Ok(std::array::from_fn(|i| {
        u32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap())
    }))
}
fn rotator(package: &Package, property: &Property, bytes: &[u8]) -> io::Result<[i32; 3]> {
    let raw = structure(package, property, bytes, "Rotator")?;
    Ok(std::array::from_fn(|i| {
        i32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap())
    }))
}
fn float(property: &Property) -> io::Result<u32> {
    match property.value {
        Value::FloatBits(bits) => Ok(bits),
        _ => Err(invalid("scene placement scalar is not a float")),
    }
}
fn reference_array(
    package: &Package,
    property: &Property,
    bytes: &[u8],
    total: &mut usize,
) -> io::Result<Vec<ObjectReference>> {
    let Value::Array { count, elements } = property.value else {
        return Err(invalid(
            "scene reference array has a different property type",
        ));
    };
    *total = total
        .checked_add(count)
        .ok_or_else(|| invalid("scene reference count overflow"))?;
    if *total > MAX_REFERENCES
        || elements.length
            != count
                .checked_mul(4)
                .ok_or_else(|| invalid("scene reference array size overflow"))?
    {
        return Err(invalid(
            "scene reference array size or count limit mismatch",
        ));
    }
    elements
        .bytes(bytes)?
        .chunks_exact(4)
        .map(|word| ObjectReference::resolve(package, i32::from_le_bytes(word.try_into().unwrap())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};

    fn name(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn package(size: usize) -> Package {
        let table = TableLocation {
            count: 0,
            offset: 0,
        };
        Package {
            summary: Summary {
                file_version: 868,
                licensee_version: 3,
                header_size: 0,
                group: String::new(),
                package_flags: 0,
                names: table.clone(),
                exports: table.clone(),
                imports: table,
                depends_offset: 0,
                import_export_guids_offset: 0,
                import_guids_count: 0,
                export_guids_count: 0,
                thumbnail_table_offset: 0,
                guid: [0; 16],
                generations: Vec::new(),
                engine_version: 0,
                cooker_version: 0,
                compression_flags: 0,
                chunks: Vec::new(),
                package_source: 0,
                parsed_bytes: 0,
            },
            names: [
                "None",
                "StaticMeshComponent",
                "Example",
                "Materials",
                "ArrayProperty",
                "StaticMesh",
                "ObjectProperty",
                "Translation",
                "StructProperty",
                "Vector",
                "Material",
                "External",
                "Class",
            ]
            .into_iter()
            .map(|value| NameEntry {
                value: value.into(),
                flags: 0,
            })
            .collect(),
            imports: vec![
                Import {
                    class_package: name(0),
                    class_name: name(12),
                    outer_index: 0,
                    object_name: name(1),
                },
                Import {
                    class_package: name(0),
                    class_name: name(10),
                    outer_index: 0,
                    object_name: name(11),
                },
            ],
            exports: vec![Export {
                class_index: -1,
                super_index: 0,
                outer_index: 0,
                object_name: name(2),
                archetype_index: 0,
                object_flags: 0,
                serial_size: size,
                serial_offset: 100,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }
    fn word(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn fname(bytes: &mut Vec<u8>, index: i32) {
        word(bytes, index);
        word(bytes, 0);
    }
    fn tag(bytes: &mut Vec<u8>, field: i32, kind: i32, size: i32) {
        fname(bytes, field);
        fname(bytes, kind);
        word(bytes, size);
        word(bytes, 0);
    }
    fn sample() -> Vec<u8> {
        // Entirely constructed; no retail bytes or strings.
        let mut bytes = vec![0; 8];
        tag(&mut bytes, 3, 4, 12);
        for value in [2, 0, -2] {
            word(&mut bytes, value);
        }
        tag(&mut bytes, 7, 8, 12);
        fname(&mut bytes, 9);
        for bits in [0x3f80_0000u32, 0x8000_0000, 0x7fc0_0123] {
            word(&mut bytes, bits as i32);
        }
        fname(&mut bytes, 0);
        bytes
    }

    #[test]
    fn retains_missing_null_import_and_float_bits() {
        let mut bytes = sample();
        let saved = SavedSceneObject::from_export(&package(bytes.len()), 0, &bytes).unwrap();
        assert!(saved.static_mesh.is_none());
        assert!(saved.placement.rotation.is_none());
        let refs = &saved.materials.unwrap().value;
        assert_eq!(refs[0].index, 0);
        assert!(refs[0].names.is_empty());
        assert_eq!(refs[1].index, -2);
        assert_eq!(refs[1].names, ["External"]);
        assert_eq!(
            saved.placement.translation.unwrap().value,
            [0x3f80_0000, 0x8000_0000, 0x7fc0_0123]
        );
        bytes.truncate(bytes.len() - 8);
        tag(&mut bytes, 5, 6, 4);
        word(&mut bytes, 0);
        fname(&mut bytes, 0);
        let saved = SavedSceneObject::from_export(&package(bytes.len()), 0, &bytes).unwrap();
        assert_eq!(saved.static_mesh.unwrap().value.index, 0);
    }

    #[test]
    fn rejects_bad_array_count_duplicate_and_unobserved_prefix() {
        let mut bytes = sample();
        bytes[32..36].copy_from_slice(&3i32.to_le_bytes());
        assert!(SavedSceneObject::from_export(&package(bytes.len()), 0, &bytes).is_err());
        bytes = sample();
        let duplicate = bytes[8..44].to_vec();
        bytes.splice(8..8, duplicate);
        assert!(SavedSceneObject::from_export(&package(bytes.len()), 0, &bytes).is_err());
        bytes = sample();
        let mut metadata = package(bytes.len());
        metadata.exports[0].object_flags = OBSERVED_ACTOR_PREFIX_FLAG;
        assert_eq!(
            SavedSceneObject::from_export(&metadata, 0, &bytes)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        for size in 0..8 {
            assert!(SavedSceneObject::from_export(&package(size), 0, &bytes[..size]).is_err());
        }
    }

    #[test]
    fn bounds_reference_indices_and_outer_cycles() {
        let mut metadata = package(0);
        for index in [i32::MIN, i32::MAX, 2, -3] {
            assert!(ObjectReference::resolve(&metadata, index).is_err());
        }
        metadata.exports[0].outer_index = 1;
        assert!(ObjectReference::resolve(&metadata, 1).is_err());
        metadata.imports[1].outer_index = -2;
        assert!(ObjectReference::resolve(&metadata, -2).is_err());
    }
}
