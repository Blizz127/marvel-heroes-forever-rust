//! Original bridge between saved SwfMovie objects, GFX image IDs and Texture2D.
//!
//! This resolves serialized dependencies only. It does not execute ActionScript,
//! instantiate a frontend controller, or update saved render-target textures.

use crate::{
    gfx::{ExternalImage, Movie, TagData},
    texture::Texture2D,
    upk::{
        Package, PackageArchive,
        objects::{ObjectProperties, Property, Value},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

#[path = "frontend/action_queue.rs"]
pub mod action_queue;
#[path = "frontend/bootstrap.rs"]
pub mod bootstrap;
#[path = "frontend/class_registry.rs"]
pub mod class_registry;
#[path = "frontend/display_state.rs"]
pub mod display_state;
#[path = "frontend/external_host.rs"]
pub mod external_host;
#[path = "frontend/host.rs"]
pub mod host;
#[path = "frontend/init_ledger.rs"]
pub mod init_ledger;
#[path = "frontend/library.rs"]
pub mod library;
#[path = "frontend/localized_callback.rs"]
pub mod localized_callback;
#[path = "frontend/localized_data.rs"]
pub mod localized_data;
#[path = "frontend/localized_label.rs"]
pub mod localized_label;
#[path = "frontend/localized_prototype.rs"]
pub mod localized_prototype;
#[path = "frontend/pointer.rs"]
pub mod pointer;
#[cfg(feature = "renderer")]
#[path = "frontend/preview.rs"]
pub mod preview;
#[path = "frontend/scene_content.rs"]
pub mod scene_content;
#[path = "frontend/selection.rs"]
pub mod selection;
#[path = "frontend/stage_state.rs"]
pub mod stage_state;
#[path = "frontend/textfield_state.rs"]
pub mod textfield_state;
#[path = "frontend/textfield_host.rs"]
pub mod textfield_host;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// Indices into explicitly supplied archives, never inferred filesystem paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetLocation {
    pub package: usize,
    pub export: usize,
}

pub struct AssetSet {
    archives: Vec<PackageArchive>,
    textures: BTreeMap<String, Vec<AssetLocation>>,
}

impl AssetSet {
    pub fn new(archives: Vec<PackageArchive>) -> io::Result<Self> {
        if archives.len() > 256 {
            return Err(invalid("frontend archive count exceeds 256"));
        }
        let mut textures: BTreeMap<String, Vec<AssetLocation>> = BTreeMap::new();
        for (package_index, archive) in archives.iter().enumerate() {
            let package = archive.package();
            for index in 0..package.exports.len() {
                if package
                    .export_class_name(index)
                    .is_some_and(|s| s.eq_ignore_ascii_case("Texture2D"))
                {
                    let path = object_path(package, export_reference(index)?)?;
                    textures
                        .entry(path.to_ascii_lowercase())
                        .or_default()
                        .push(AssetLocation {
                            package: package_index,
                            export: index,
                        });
                }
            }
        }
        Ok(Self { archives, textures })
    }

    pub fn archive(&self, package: usize) -> io::Result<&PackageArchive> {
        self.archives
            .get(package)
            .ok_or_else(|| invalid("archive index out of bounds"))
    }

    pub fn read_movie(&self, location: AssetLocation) -> io::Result<MovieAsset> {
        let archive = self.archive(location.package)?;
        let package = archive.package();
        let bytes = archive.read_export(location.export)?;
        let properties = ObjectProperties::parse_export(package, location.export, &bytes)?;
        if !properties.class_name.eq_ignore_ascii_case("SwfMovie") {
            return Err(invalid("requested export is not a SwfMovie"));
        }
        let raw = singleton(package, &properties.properties.entries, "RawData")?
            .ok_or_else(|| invalid("SwfMovie has no serialized RawData"))?;
        let (count, movie_bytes) = array(raw, &bytes)?;
        if count != movie_bytes.len() {
            return Err(invalid("SwfMovie RawData byte count mismatch"));
        }
        let references = singleton(package, &properties.properties.entries, "References")?
            .map(|property| {
                let (count, data) = array(property, &bytes)?;
                let refs = object_array(count, data)?;
                for &reference in &refs {
                    if reference != 0 {
                        object_path(package, reference)?;
                    }
                }
                Ok::<_, io::Error>(refs)
            })
            .transpose()?;
        Ok(MovieAsset {
            location,
            object_path: object_path(package, export_reference(location.export)?)?,
            movie: Movie::parse(movie_bytes)?,
            references,
        })
    }

    pub fn read_texture(&self, location: AssetLocation) -> io::Result<Texture2D> {
        let archive = self.archive(location.package)?;
        let bytes = archive.read_export(location.export)?;
        Texture2D::from_export(archive.package(), location.export, &bytes)
    }

    /// Only declared References participate. An absent property can mean class
    /// defaults, so it produces an explicit unresolved result instead of a guess.
    pub fn bind_images(&self, asset: &MovieAsset) -> io::Result<Vec<ImageBinding>> {
        let package = self.archive(asset.location.package)?.package();
        let mut references = BTreeMap::<String, BTreeSet<i32>>::new();
        if let Some(list) = &asset.references {
            for &reference in list {
                if reference == 0 {
                    continue;
                }
                if !object_class(package, reference)?.eq_ignore_ascii_case("Texture2D") {
                    continue;
                }
                let name = package
                    .resolve_object_name(reference)
                    .ok_or_else(|| invalid("unresolved texture reference name"))?;
                references
                    .entry(name.to_ascii_lowercase())
                    .or_default()
                    .insert(reference);
            }
        }
        let mut bindings = Vec::new();
        for tag in &asset.movie.tags {
            let TagData::ExternalImage(image) = &tag.data else {
                continue;
            };
            let (lookup_name, mut basis) = image_lookup(image)?;
            let exact = references.get(&lookup_name);
            let normalized = lookup_name.replace('.', "_");
            let candidates = if exact.is_some() {
                exact
            } else if normalized != lookup_name {
                let found = references.get(&normalized);
                if found.is_some() {
                    basis = NameBasis::ObservedDotToUnderscore;
                }
                found
            } else {
                None
            };
            let resolution = if asset.references.is_none() {
                ImageResolution::MissingReferenceProperty
            } else if let Some(candidates) = candidates {
                if candidates.len() != 1 {
                    ImageResolution::AmbiguousReferences(candidates.iter().copied().collect())
                } else {
                    let reference = *candidates.first().unwrap();
                    let path = object_path(package, reference)?;
                    if reference > 0 {
                        ImageResolution::Resolved {
                            source_reference: reference,
                            object_path: path,
                            location: AssetLocation {
                                package: asset.location.package,
                                export: reference as usize - 1,
                            },
                        }
                    } else {
                        match self
                            .textures
                            .get(&path.to_ascii_lowercase())
                            .map(Vec::as_slice)
                        {
                            Some([location]) => ImageResolution::Resolved {
                                source_reference: reference,
                                object_path: path,
                                location: *location,
                            },
                            Some(locations) => ImageResolution::AmbiguousImport {
                                object_path: path,
                                locations: locations.to_vec(),
                            },
                            None => ImageResolution::MissingImport { object_path: path },
                        }
                    }
                }
            } else {
                ImageResolution::MissingNamedReference
            };
            bindings.push(ImageBinding {
                image: image.clone(),
                basis,
                resolution,
            });
        }
        Ok(bindings)
    }
}

pub struct MovieAsset {
    pub location: AssetLocation,
    pub object_path: String,
    pub movie: Movie,
    /// None preserves absence; Some(empty) is an explicitly empty array.
    pub references: Option<Vec<i32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameBasis {
    ExportName,
    FileStem,
    /// Empirical cooked-name conversion, not proof of Scaleform's general rule.
    ObservedDotToUnderscore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageResolution {
    Resolved {
        source_reference: i32,
        object_path: String,
        location: AssetLocation,
    },
    MissingReferenceProperty,
    MissingNamedReference,
    AmbiguousReferences(Vec<i32>),
    MissingImport {
        object_path: String,
    },
    AmbiguousImport {
        object_path: String,
        locations: Vec<AssetLocation>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageBinding {
    /// Logical image dimensions remain distinct from Texture2D texel dimensions.
    pub image: ExternalImage,
    pub basis: NameBasis,
    pub resolution: ImageResolution,
}

fn export_reference(index: usize) -> io::Result<i32> {
    index
        .checked_add(1)
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| invalid("export reference overflow"))
}

/// Full serialized object identity. Cooked package filename is not an outer name.
pub fn object_path(package: &Package, index: i32) -> io::Result<String> {
    if index == 0 {
        return Err(invalid("null reference has no object path"));
    }
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = index;
    while current != 0 {
        if names.len() >= 128 || !seen.insert(current) {
            return Err(invalid("cyclic or excessive object outer chain"));
        }
        let (name, outer) = if current > 0 {
            let export = package
                .exports
                .get(current as usize - 1)
                .ok_or_else(|| invalid("export reference out of bounds"))?;
            (export.object_name, export.outer_index)
        } else {
            let import = package
                .imports
                .get(current.unsigned_abs() as usize - 1)
                .ok_or_else(|| invalid("import reference out of bounds"))?;
            (import.object_name, import.outer_index)
        };
        let name = package
            .resolve_name(name)
            .ok_or_else(|| invalid("object name out of bounds"))?;
        if name.is_empty() || name.contains('.') {
            return Err(invalid("empty or dotted object path component"));
        }
        names.push(name);
        current = outer;
    }
    names.reverse();
    Ok(names.join("."))
}

fn object_class(package: &Package, reference: i32) -> io::Result<String> {
    if reference > 0 {
        package.export_class_name(reference as usize - 1)
    } else {
        package
            .imports
            .get(reference.unsigned_abs() as usize - 1)
            .and_then(|import| package.resolve_name(import.class_name))
    }
    .ok_or_else(|| invalid("unresolved referenced object class"))
}

fn singleton<'a>(
    package: &Package,
    properties: &'a [Property],
    name: &str,
) -> io::Result<Option<&'a Property>> {
    let mut found = None;
    for property in properties {
        if package
            .resolve_name(property.name)
            .is_some_and(|s| s.eq_ignore_ascii_case(name))
        {
            if found.is_some() || property.array_index != 0 {
                return Err(invalid(format!(
                    "duplicate or indexed SwfMovie {name} property"
                )));
            }
            found = Some(property);
        }
    }
    Ok(found)
}

fn array<'a>(property: &Property, bytes: &'a [u8]) -> io::Result<(usize, &'a [u8])> {
    match property.value {
        Value::Array { count, elements } => Ok((count, elements.bytes(bytes)?)),
        _ => Err(invalid("SwfMovie property must be an array")),
    }
}

fn object_array(count: usize, bytes: &[u8]) -> io::Result<Vec<i32>> {
    if count.checked_mul(4) != Some(bytes.len()) {
        return Err(invalid("SwfMovie References count/byte length mismatch"));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|v| i32::from_le_bytes(v.try_into().unwrap()))
        .collect())
}

fn image_lookup(image: &ExternalImage) -> io::Result<(String, NameBasis)> {
    let (name, basis) = if image.export_name.is_empty() {
        let leaf = image.file_name.rsplit(['/', '\\']).next().unwrap_or("");
        (
            leaf.rsplit_once('.').map_or(leaf, |(stem, _)| stem),
            NameBasis::FileStem,
        )
    } else {
        (image.export_name.as_str(), NameBasis::ExportName)
    };
    if name.is_empty() {
        return Err(invalid("external image has no usable asset name"));
    }
    Ok((name.to_ascii_lowercase(), basis))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_preserve_signed_indices_and_reject_truncation() {
        let bytes: Vec<_> = [169_i32, -20, 0]
            .iter()
            .flat_map(|n| n.to_le_bytes())
            .collect();
        assert_eq!(object_array(3, &bytes).unwrap(), [169, -20, 0]);
        assert!(object_array(2, &bytes).is_err());
        assert!(object_array(usize::MAX, &[]).is_err());
    }
    #[test]
    fn image_names_are_identifiers_not_opened_paths() {
        let mut image = ExternalImage {
            id: 1,
            format_code: 0,
            width: 4,
            height: 4,
            export_name: "".into(),
            file_name: "../MarvelUI/Example.png.tga".into(),
        };
        assert_eq!(
            image_lookup(&image).unwrap(),
            ("example.png".into(), NameBasis::FileStem)
        );
        image.export_name = "Explicit_Name".into();
        assert_eq!(
            image_lookup(&image).unwrap(),
            ("explicit_name".into(), NameBasis::ExportName)
        );
        image.export_name.clear();
        image.file_name.clear();
        assert!(image_lookup(&image).is_err());
    }
}
