//! Saved class/default provenance, without effective-default evaluation.
//!
//! Archives and import bindings are explicit. Only selected classes and their
//! bounded parent chains are read. No lookup scans, filesystem imports, scripts,
//! default merging, configuration application or object construction occur.

use super::{AssetLocation, AssetSet};
use crate::upk::{
    NameRef, Package,
    class::{ClassObject, Limits as ClassLimits},
    objects::{self, PropertyList, Span, TagDetail},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

/// Caller-declared logical package namespace, not an inferred file basename.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageName {
    pub package: usize,
    pub name: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportLocation {
    pub package: usize,
    pub import: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportBinding {
    pub source: ImportLocation,
    pub target: AssetLocation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub classes: usize,
    pub parent_depth: usize,
    pub import_bindings: usize,
    pub package_names: usize,
    pub outer_depth: usize,
    pub identity_bytes: usize,
    pub serialized_bytes: usize,
    pub default_bytes: usize,
    pub properties: usize,
    pub class: ClassLimits,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            classes: 128,
            parent_depth: 32,
            import_bindings: 4096,
            package_names: 256,
            outer_depth: 128,
            identity_bytes: 4 * 1024 * 1024,
            serialized_bytes: 8 * 1024 * 1024,
            default_bytes: 1024 * 1024,
            properties: 65_536,
            class: ClassLimits::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportIdentity {
    pub location: ImportLocation,
    pub object_path: String,
    pub class_package: String,
    pub class_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    Null,
    Local(AssetLocation),
    Imported {
        source: ImportIdentity,
        target: Option<AssetLocation>,
    },
}
impl Resolution {
    pub fn target(&self) -> Option<AssetLocation> {
        match self {
            Self::Local(value) => Some(*value),
            Self::Imported { target, .. } => *target,
            Self::Null => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkOrigin {
    /// Field span within the class export, with its original logical base.
    ClassField {
        class: AssetLocation,
        serial_offset: usize,
        span: Span,
    },
    /// Export-table metadata; no fictitious payload byte offset is supplied.
    ExportArchetype { default: AssetLocation },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub reference: i32,
    pub origin: LinkOrigin,
    pub resolution: Resolution,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagOrigin {
    pub class: AssetLocation,
    pub default: AssetLocation,
    pub serial_offset: usize,
    pub tag: Span,
    pub payload: Span,
    /// BoolProperty's inline byte, or the ordinary property payload.
    pub encoded_value: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedDefault {
    pub class: AssetLocation,
    pub location: AssetLocation,
    pub serial_offset: usize,
    pub preamble: DefaultPreamble,
    pub net_index: i32,
    pub properties: PropertyList,
    pub unknown_tail: Span,
    pub bytes: Vec<u8>,
}
/// An asset file grammar, not a claim about a live archive or initialized object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultPreamble {
    /// Four selected GFx classes, profile868/3, exact observed CDO flags and -1
    /// NetIndex. Native routing and fresh loader checkpoints corroborate it.
    ObservedGfx8683File,
}
impl SavedDefault {
    pub fn tag_origin(&self, index: usize) -> Option<TagOrigin> {
        let property = self.properties.entries.get(index)?;
        let encoded_value = if matches!(property.detail, TagDetail::Boolean(_)) {
            Span {
                offset: property
                    .tag
                    .offset
                    .checked_add(property.tag.length)?
                    .checked_sub(1)?,
                length: 1,
            }
        } else {
            property.payload
        };
        Some(TagOrigin {
            class: self.class,
            default: self.location,
            serial_offset: self.serial_offset,
            tag: property.tag,
            payload: property.payload,
            encoded_value,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultIssue {
    NullReference,
    MissingImport,
    ClassIdentityUnresolved,
    UnsupportedSchema,
    UnsupportedArchetype,
    StateFrame,
    MissingClassDefaultFlag,
    UnsupportedObjectFlags,
    UnexpectedNetIndex,
    UnsupportedTagLayout(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultBody {
    Saved(SavedDefault),
    Unresolved {
        location: Option<AssetLocation>,
        issue: DefaultIssue,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultRecord {
    pub link: Link,
    pub archetype: Option<Link>,
    pub body: DefaultBody,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassBody {
    Parsed {
        definition: ClassObject,
        superclass: Link,
        default: DefaultRecord,
    },
    Unsupported {
        reason: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassRecord {
    pub location: AssetLocation,
    pub qualified_name: String,
    pub bytes: Vec<u8>,
    pub body: ClassBody,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainEnd {
    Null,
    MissingImport(ImportIdentity),
    UnsupportedClass(AssetLocation),
    Cycle(AssetLocation),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentChain {
    pub root: AssetLocation,
    pub classes: Vec<AssetLocation>,
    pub end: ChainEnd,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub class_exports: usize,
    pub default_exports: usize,
    pub serialized_bytes: usize,
    pub explicit_properties: usize,
    pub identity_bytes: usize,
}
#[derive(Debug)]
pub struct PreparedDefaults {
    asset_identity: Arc<()>,
    pub package_names: Vec<PackageName>,
    pub import_bindings: Vec<ImportBinding>,
    pub records: BTreeMap<AssetLocation, ClassRecord>,
    pub chains: Vec<ParentChain>,
    pub statistics: Statistics,
}
impl PreparedDefaults {
    pub fn matches_assets(&self, assets: &AssetSet) -> bool {
        Arc::ptr_eq(&self.asset_identity, &assets.identity)
    }
}

pub fn prepare(
    assets: &AssetSet,
    selected: &[AssetLocation],
    package_names: &[PackageName],
    import_bindings: &[ImportBinding],
    limits: Limits,
) -> io::Result<PreparedDefaults> {
    if selected.len() > limits.classes
        || package_names.len() > limits.package_names
        || import_bindings.len() > limits.import_bindings
    {
        return Err(unsupported("class-default intake budget exceeded"));
    }
    let mut reader = Reader {
        assets,
        names: BTreeMap::new(),
        imports: BTreeMap::new(),
        limits,
        statistics: Statistics::default(),
    };
    let mut unique_names = BTreeSet::new();
    for name in package_names {
        assets.archive(name.package)?;
        if name.name.is_empty() || name.name.contains('.') {
            return Err(invalid(
                "class-default logical package name must be one nonempty component",
            ));
        }
        reader.charge_identity(name.name.len())?;
        if reader
            .names
            .insert(name.package, name.name.clone())
            .is_some()
            || !unique_names.insert(name.name.to_ascii_lowercase())
        {
            return Err(invalid("duplicate class-default package namespace"));
        }
    }
    for binding in import_bindings {
        if reader
            .imports
            .insert(binding.source, binding.target)
            .is_some()
        {
            return Err(invalid("duplicate class-default import binding"));
        }
        let source = reader.import_identity(binding.source)?;
        let path = reader.qualified_path(binding.target)?;
        let kind = reader.export_kind(binding.target)?;
        let expected_kind = format!("{}.{}", source.class_package, source.class_name);
        if !source.object_path.eq_ignore_ascii_case(&path)
            || !expected_kind.eq_ignore_ascii_case(&kind)
        {
            return Err(invalid(
                "class-default import binding disagrees with full object/class identity",
            ));
        }
    }
    let mut records = BTreeMap::new();
    let mut chains = Vec::with_capacity(selected.len());
    for &root in selected {
        let mut current = root;
        let mut seen = BTreeSet::new();
        let mut classes = Vec::new();
        let end = loop {
            if !seen.insert(current) {
                break ChainEnd::Cycle(current);
            }
            if classes.len() == limits.parent_depth {
                return Err(unsupported("class-default parent depth budget exceeded"));
            }
            if !records.contains_key(&current) {
                if records.len() == limits.classes {
                    return Err(unsupported("class-default class count budget exceeded"));
                }
                records.insert(current, reader.class(current)?);
            }
            classes.push(current);
            let ClassBody::Parsed { superclass, .. } = &records[&current].body else {
                break ChainEnd::UnsupportedClass(current);
            };
            match &superclass.resolution {
                Resolution::Null => break ChainEnd::Null,
                Resolution::Local(target) => current = *target,
                Resolution::Imported {
                    target: Some(target),
                    ..
                } => current = *target,
                Resolution::Imported {
                    source,
                    target: None,
                } => break ChainEnd::MissingImport(source.clone()),
            }
        };
        chains.push(ParentChain { root, classes, end });
    }
    Ok(PreparedDefaults {
        asset_identity: Arc::clone(&assets.identity),
        package_names: package_names.to_vec(),
        import_bindings: import_bindings.to_vec(),
        records,
        chains,
        statistics: reader.statistics,
    })
}

struct Reader<'a> {
    assets: &'a AssetSet,
    names: BTreeMap<usize, String>,
    imports: BTreeMap<ImportLocation, AssetLocation>,
    limits: Limits,
    statistics: Statistics,
}
impl Reader<'_> {
    fn charge_identity(&mut self, bytes: usize) -> io::Result<()> {
        charge(
            &mut self.statistics.identity_bytes,
            bytes,
            self.limits.identity_bytes,
            "identity bytes",
        )
    }
    fn name(&mut self, package: &Package, reference: NameRef) -> io::Result<String> {
        let base = &package
            .names
            .get(reference.index)
            .ok_or_else(|| invalid("name index out of bounds"))?
            .value;
        // Charge before allocating the resolved name, including a bounded suffix.
        self.charge_identity(
            base.len()
                .checked_add(if reference.number == 0 { 0 } else { 11 })
                .ok_or_else(|| invalid("name size overflow"))?,
        )?;
        package
            .resolve_name(reference)
            .ok_or_else(|| invalid("name index out of bounds"))
    }
    fn path(&mut self, package: &Package, mut reference: i32) -> io::Result<String> {
        if reference == 0 {
            return Err(invalid("null object has no identity"));
        }
        let mut seen = BTreeSet::new();
        let mut parts = Vec::new();
        while reference != 0 {
            if parts.len() == self.limits.outer_depth || !seen.insert(reference) {
                return Err(invalid("cyclic or excessive object outer chain"));
            }
            let (name, outer) = if reference > 0 {
                let item = package
                    .exports
                    .get(reference as usize - 1)
                    .ok_or_else(|| invalid("export reference out of bounds"))?;
                (item.object_name, item.outer_index)
            } else {
                let item = package
                    .imports
                    .get(reference.unsigned_abs() as usize - 1)
                    .ok_or_else(|| invalid("import reference out of bounds"))?;
                (item.object_name, item.outer_index)
            };
            let name = self.name(package, name)?;
            if name.is_empty() || name.contains('.') {
                return Err(invalid("invalid object path component"));
            }
            parts.push(name);
            reference = outer;
        }
        parts.reverse();
        self.charge_identity(parts.len().saturating_sub(1))?;
        Ok(parts.join("."))
    }
    fn qualified_path(&mut self, location: AssetLocation) -> io::Result<String> {
        let assets = self.assets;
        let package = assets.archive(location.package)?.package();
        let name = self
            .names
            .get(&location.package)
            .ok_or_else(|| invalid("missing explicit package namespace"))?
            .clone();
        self.charge_identity(
            name.len()
                .checked_add(1)
                .ok_or_else(|| invalid("identity overflow"))?,
        )?;
        let path = self.path(package, export_reference(location.export)?)?;
        Ok(format!("{name}.{path}"))
    }
    fn export_kind(&mut self, location: AssetLocation) -> io::Result<String> {
        let assets = self.assets;
        let package = assets.archive(location.package)?.package();
        let export = package
            .exports
            .get(location.export)
            .ok_or_else(|| invalid("export location out of bounds"))?;
        match export.class_index {
            0 => {
                self.charge_identity(10)?;
                Ok("Core.Class".into())
            }
            index if index > 0 => self.qualified_path(AssetLocation {
                package: location.package,
                export: index as usize - 1,
            }),
            index => self.path(package, index),
        }
    }
    fn import_identity(&mut self, location: ImportLocation) -> io::Result<ImportIdentity> {
        let assets = self.assets;
        let package = assets.archive(location.package)?.package();
        let import = package
            .imports
            .get(location.import)
            .ok_or_else(|| invalid("import location out of bounds"))?;
        let reference = export_reference(location.import)?
            .checked_neg()
            .ok_or_else(|| invalid("import index overflow"))?;
        Ok(ImportIdentity {
            location,
            object_path: self.path(package, reference)?,
            class_package: self.name(package, import.class_package)?,
            class_name: self.name(package, import.class_name)?,
        })
    }
    fn resolve(&mut self, package_index: usize, reference: i32) -> io::Result<Resolution> {
        if reference == 0 {
            return Ok(Resolution::Null);
        }
        if reference > 0 {
            let location = AssetLocation {
                package: package_index,
                export: reference as usize - 1,
            };
            self.assets
                .archive(package_index)?
                .package()
                .exports
                .get(location.export)
                .ok_or_else(|| invalid("local link is outside export table"))?;
            Ok(Resolution::Local(location))
        } else {
            let source = self.import_identity(ImportLocation {
                package: package_index,
                import: reference.unsigned_abs() as usize - 1,
            })?;
            let target = self.imports.get(&source.location).copied();
            Ok(Resolution::Imported { source, target })
        }
    }
    fn read(&mut self, location: AssetLocation, maximum: usize) -> io::Result<Vec<u8>> {
        let archive = self.assets.archive(location.package)?;
        let export = archive
            .package()
            .exports
            .get(location.export)
            .ok_or_else(|| invalid("export location out of bounds"))?;
        if export.serial_size > maximum {
            return Err(unsupported("class-default per-export byte budget exceeded"));
        }
        charge(
            &mut self.statistics.serialized_bytes,
            export.serial_size,
            self.limits.serialized_bytes,
            "serialized bytes",
        )?;
        archive.read_export(location.export)
    }
    fn class(&mut self, location: AssetLocation) -> io::Result<ClassRecord> {
        let qualified_name = self.qualified_path(location)?;
        let bytes = self.read(location, self.limits.class.max_export_bytes)?;
        self.statistics.class_exports += 1;
        let assets = self.assets;
        let package = assets.archive(location.package)?.package();
        let definition = match ClassObject::parse_export_with_limits(
            package,
            location.export,
            &bytes,
            self.limits.class,
        ) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::Unsupported => {
                return Ok(ClassRecord {
                    location,
                    qualified_name,
                    bytes,
                    body: ClassBody::Unsupported {
                        reason: error.to_string(),
                    },
                });
            }
            Err(error) => return Err(error),
        };
        let superclass = Link {
            reference: definition.structure.super_class.value,
            origin: LinkOrigin::ClassField {
                class: location,
                serial_offset: definition.serial_offset,
                span: definition.structure.super_class.span,
            },
            resolution: self.resolve(location.package, definition.structure.super_class.value)?,
        };
        let default_link = Link {
            reference: definition.default_object.value,
            origin: LinkOrigin::ClassField {
                class: location,
                serial_offset: definition.serial_offset,
                span: definition.default_object.span,
            },
            resolution: self.resolve(location.package, definition.default_object.value)?,
        };
        let default = self.default(location, &qualified_name, &superclass, default_link)?;
        Ok(ClassRecord {
            location,
            qualified_name,
            bytes,
            body: ClassBody::Parsed {
                definition,
                superclass,
                default,
            },
        })
    }
    fn default(
        &mut self,
        class: AssetLocation,
        class_name: &str,
        superclass: &Link,
        link: Link,
    ) -> io::Result<DefaultRecord> {
        let Some(location) = link.resolution.target() else {
            let issue = if matches!(link.resolution, Resolution::Null) {
                DefaultIssue::NullReference
            } else {
                DefaultIssue::MissingImport
            };
            return Ok(DefaultRecord {
                link,
                archetype: None,
                body: DefaultBody::Unresolved {
                    location: None,
                    issue,
                    bytes: Vec::new(),
                },
            });
        };
        let assets = self.assets;
        let package = assets.archive(location.package)?.package();
        let export = &package.exports[location.export];
        let archetype = Some(Link {
            reference: export.archetype_index,
            origin: LinkOrigin::ExportArchetype { default: location },
            resolution: self.resolve(location.package, export.archetype_index)?,
        });
        let actual_class = self.resolve(location.package, export.class_index)?;
        let bytes = self.read(location, self.limits.default_bytes)?;
        self.statistics.default_exports += 1;
        let unresolved = |issue, bytes| DefaultBody::Unresolved {
            location: Some(location),
            issue,
            bytes,
        };
        let body = if actual_class.target().is_none() {
            unresolved(DefaultIssue::ClassIdentityUnresolved, bytes)
        } else if actual_class.target() != Some(class) {
            return Err(invalid(
                "bound default export does not have the exact selected class",
            ));
        } else if !self.supported_schema(class, class_name, superclass)? {
            unresolved(DefaultIssue::UnsupportedSchema, bytes)
        } else if !self.supported_archetype(class_name, archetype.as_ref().unwrap())? {
            unresolved(DefaultIssue::UnsupportedArchetype, bytes)
        } else if export.object_flags & 0x0200_0000_0000_0000 != 0 {
            unresolved(DefaultIssue::StateFrame, bytes)
        } else if export.object_flags & 0x200 == 0 {
            unresolved(DefaultIssue::MissingClassDefaultFlag, bytes)
        } else if export.object_flags != 0x0007_0004_0000_0200 {
            unresolved(DefaultIssue::UnsupportedObjectFlags, bytes)
        } else {
            let net_index = i32::from_le_bytes(
                bytes
                    .get(..4)
                    .ok_or_else(|| invalid("truncated class-default NetIndex"))?
                    .try_into()
                    .unwrap(),
            );
            if net_index != -1 {
                unresolved(DefaultIssue::UnexpectedNetIndex, bytes)
            } else {
                match objects::parse_tagged_properties(package, &bytes, 4) {
                    Ok(properties) => {
                        charge(
                            &mut self.statistics.explicit_properties,
                            properties.entries.len(),
                            self.limits.properties,
                            "explicit properties",
                        )?;
                        let unknown_tail = Span {
                            offset: properties.end_offset,
                            length: bytes.len() - properties.end_offset,
                        };
                        DefaultBody::Saved(SavedDefault {
                            class,
                            location,
                            serial_offset: export.serial_offset,
                            preamble: DefaultPreamble::ObservedGfx8683File,
                            net_index,
                            properties,
                            unknown_tail,
                            bytes,
                        })
                    }
                    Err(error) if error.kind() == io::ErrorKind::Unsupported => {
                        unresolved(DefaultIssue::UnsupportedTagLayout(error.to_string()), bytes)
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        Ok(DefaultRecord {
            link,
            archetype,
            body,
        })
    }
    fn supported_schema(
        &mut self,
        class: AssetLocation,
        name: &str,
        superclass: &Link,
    ) -> io::Result<bool> {
        let expected = match name.to_ascii_lowercase().as_str() {
            "gfxui.gfxaction_openmovie" => "Engine.SequenceAction",
            "gfxui.gfxmovieplayer" => "Core.Object",
            "marvelgame.marvelgfxmovieplayer" => "GFxUI.GFxMoviePlayer",
            "marvelgame.marvelgfxfrontend" => "MarvelGame.MarvelGFxMoviePlayer",
            _ => return Ok(false),
        };
        let actual = match &superclass.resolution {
            Resolution::Null => return Ok(false),
            Resolution::Local(target) => self.qualified_path(*target)?,
            Resolution::Imported { source, .. } => source.object_path.clone(),
        };
        let package = self.assets.archive(class.package)?.package();
        Ok(package.exports[class.export].outer_index == 0 && actual.eq_ignore_ascii_case(expected))
    }
    fn supported_archetype(&mut self, class_name: &str, link: &Link) -> io::Result<bool> {
        let expected = match class_name.to_ascii_lowercase().as_str() {
            "gfxui.gfxaction_openmovie" => "Engine.Default__SequenceAction",
            "gfxui.gfxmovieplayer" => "Core.Default__Object",
            "marvelgame.marvelgfxmovieplayer" => "GFxUI.Default__GFxMoviePlayer",
            "marvelgame.marvelgfxfrontend" => "MarvelGame.Default__MarvelGFxMoviePlayer",
            _ => return Ok(false),
        };
        let actual = match &link.resolution {
            Resolution::Null => return Ok(false),
            Resolution::Local(target) => self.qualified_path(*target)?,
            Resolution::Imported { source, .. } => source.object_path.clone(),
        };
        Ok(actual.eq_ignore_ascii_case(expected))
    }
}
fn charge(current: &mut usize, amount: usize, limit: usize, what: &str) -> io::Result<()> {
    *current = current
        .checked_add(amount)
        .filter(|value| *value <= limit)
        .ok_or_else(|| unsupported(format!("class-default {what} budget exceeded")))?;
    Ok(())
}
fn export_reference(index: usize) -> io::Result<i32> {
    index
        .checked_add(1)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| invalid("package index exceeds signed reference range"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::PackageArchive;

    fn word(out: &mut Vec<u8>, value: i32) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn name(out: &mut Vec<u8>, index: i32) {
        word(out, index);
        word(out, 0);
    }
    fn text(out: &mut Vec<u8>, value: &str) {
        word(out, value.len() as i32 + 1);
        out.extend_from_slice(value.as_bytes());
        out.push(0);
    }
    fn patch(out: &mut [u8], at: usize, value: usize) {
        out[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
    /// Original synthetic package; no retail bytes. Two exports, three imports.
    fn archive(super_ref: i32, class_name: i32, default_flags: u64) -> PackageArchive {
        let mut class = Vec::new();
        for value in [-1, 0, super_ref, 0, 0, 0, 0, 0, 0, 0] {
            word(&mut class, value);
        }
        word(&mut class, 0);
        class.extend_from_slice(&0xffffu16.to_le_bytes());
        word(&mut class, 0);
        word(&mut class, 0);
        word(&mut class, 0x08000096);
        word(&mut class, -1);
        name(&mut class, 0);
        for _ in 0..9 {
            word(&mut class, 0);
        }
        name(&mut class, 0);
        word(&mut class, 2);
        assert_eq!(class.len(), 118);
        let mut default = Vec::new();
        word(&mut default, -1);
        name(&mut default, 8);
        name(&mut default, 7);
        word(&mut default, 0);
        word(&mut default, 0);
        default.push(1);
        name(&mut default, 0);
        let names = [
            "None",
            "Core",
            "Class",
            "Package",
            "Object",
            "GFxMoviePlayer",
            "Default__GFxMoviePlayer",
            "BoolProperty",
            "bAllowInput",
            "Default__Object",
        ];
        let mut out = Vec::new();
        word(&mut out, 0x9e2a83c1u32 as i32);
        word(&mut out, 868 | (3 << 16));
        word(&mut out, 0);
        text(&mut out, "None");
        word(&mut out, 0);
        word(&mut out, names.len() as i32);
        let names_at = out.len();
        word(&mut out, 0);
        word(&mut out, 2);
        let exports_at = out.len();
        word(&mut out, 0);
        word(&mut out, 3);
        let imports_at = out.len();
        word(&mut out, 0);
        let depends_at = out.len();
        word(&mut out, 0);
        for _ in 0..4 {
            word(&mut out, 0);
        }
        out.extend_from_slice(&[0; 16]);
        word(&mut out, 1);
        for value in [2, names.len() as i32, 0, 10897, 136, 0, 0, 0, 0, 0] {
            word(&mut out, value);
        }
        let start = out.len();
        patch(&mut out, names_at, start);
        for value in names {
            text(&mut out, value);
            out.extend_from_slice(&0u64.to_le_bytes());
        }
        let start = out.len();
        patch(&mut out, imports_at, start);
        // Core.Object class, Core package, Core.Default__Object instance.
        for (kind, outer, object) in [(2, -2, 4), (3, 0, 1), (4, -2, 9)] {
            name(&mut out, 1);
            name(&mut out, kind);
            word(&mut out, outer);
            name(&mut out, object);
        }
        let start = out.len();
        patch(&mut out, exports_at, start);
        let mut payload_offsets = Vec::new();
        for (kind, super_index, object_name, archetype, flags, size) in [
            (0, super_ref, class_name, 0, 0u64, class.len()),
            (1, 0, 6, -3, default_flags, default.len()),
        ] {
            word(&mut out, kind);
            word(&mut out, super_index);
            word(&mut out, 0);
            name(&mut out, object_name);
            word(&mut out, archetype);
            out.extend_from_slice(&flags.to_le_bytes());
            word(&mut out, size as i32);
            payload_offsets.push(out.len());
            word(&mut out, 0);
            word(&mut out, 0);
            word(&mut out, 0);
            out.extend_from_slice(&[0; 16]);
            word(&mut out, 0);
        }
        let start = out.len();
        patch(&mut out, depends_at, start);
        word(&mut out, 0);
        word(&mut out, 0);
        let header = out.len();
        patch(&mut out, 8, header);
        patch(&mut out, payload_offsets[0], header);
        out.extend_from_slice(&class);
        let start = out.len();
        patch(&mut out, payload_offsets[1], start);
        out.extend_from_slice(&default);
        PackageArchive::parse(out).unwrap()
    }
    fn packages() -> Vec<PackageName> {
        vec![PackageName {
            package: 0,
            name: "GFxUI".into(),
        }]
    }
    fn root() -> AssetLocation {
        AssetLocation {
            package: 0,
            export: 0,
        }
    }
    fn assets() -> AssetSet {
        AssetSet::new(vec![archive(-1, 5, 0x0007_0004_0000_0200)]).unwrap()
    }

    #[test]
    fn preserves_explicit_tag_origins_and_missing_parent_without_merging() {
        let assets = assets();
        let result = prepare(&assets, &[root()], &packages(), &[], Limits::default()).unwrap();
        assert_eq!(result.statistics.class_exports, 1);
        assert_eq!(result.statistics.default_exports, 1);
        assert_eq!(result.statistics.explicit_properties, 1);
        assert!(
            matches!(&result.chains[0].end,ChainEnd::MissingImport(identity) if identity.object_path=="Core.Object")
        );
        let ClassBody::Parsed { default, .. } = &result.records[&root()].body else {
            panic!()
        };
        assert!(matches!(
            &default.archetype.as_ref().unwrap().resolution,
            Resolution::Imported { target: None, .. }
        ));
        let DefaultBody::Saved(saved) = &default.body else {
            panic!()
        };
        let origin = saved.tag_origin(0).unwrap();
        assert_eq!(origin.class, root());
        assert_eq!(
            origin.default,
            AssetLocation {
                package: 0,
                export: 1
            }
        );
        assert_eq!(
            origin.encoded_value,
            Span {
                offset: 28,
                length: 1
            }
        );
        assert_eq!(origin.payload.length, 0);
        assert_eq!(saved.unknown_tail.length, 0);
        assert!(saved.tag_origin(1).is_none());
        assert!(result.matches_assets(&assets));
        let moved = assets;
        assert!(result.matches_assets(&moved));
        assert!(!result.matches_assets(&self::assets()));
    }
    #[test]
    fn binding_requires_full_identity_and_is_never_inferred_from_available_archive() {
        let assets = AssetSet::new(vec![
            archive(-1, 5, 0x0007_0004_0000_0200),
            archive(0, 4, 0x0007_0004_0000_0200),
        ])
        .unwrap();
        let names = vec![
            PackageName {
                package: 0,
                name: "GFxUI".into(),
            },
            PackageName {
                package: 1,
                name: "Core".into(),
            },
        ];
        let missing = prepare(&assets, &[root()], &names, &[], Limits::default()).unwrap();
        assert_eq!(missing.records.len(), 1);
        let binding = ImportBinding {
            source: ImportLocation {
                package: 0,
                import: 0,
            },
            target: AssetLocation {
                package: 1,
                export: 0,
            },
        };
        let resolved = prepare(&assets, &[root()], &names, &[binding], Limits::default()).unwrap();
        assert_eq!(resolved.records.len(), 2);
        assert_eq!(resolved.chains[0].classes.len(), 2);
        assert_eq!(resolved.chains[0].end, ChainEnd::Null);
        assert!(
            prepare(
                &assets,
                &[root()],
                &names,
                &[binding, binding],
                Limits::default()
            )
            .is_err()
        );
        let wrong = ImportBinding {
            target: AssetLocation {
                package: 0,
                export: 0,
            },
            ..binding
        };
        assert!(prepare(&assets, &[root()], &names, &[wrong], Limits::default()).is_err());
    }
    #[test]
    fn retains_unsupported_defaults_and_detects_parent_cycles() {
        let assets = AssetSet::new(vec![archive(1, 5, 0x0007_0004_0000_0200)]).unwrap();
        let result = prepare(&assets, &[root()], &packages(), &[], Limits::default()).unwrap();
        assert_eq!(result.chains[0].end, ChainEnd::Cycle(root()));
        let ClassBody::Parsed { default, .. } = &result.records[&root()].body else {
            panic!()
        };
        assert!(matches!(
            default.body,
            DefaultBody::Unresolved {
                issue: DefaultIssue::UnsupportedSchema,
                ..
            }
        ));
        let assets = AssetSet::new(vec![archive(-1, 5, 0x0207_0004_0000_0200)]).unwrap();
        let result = prepare(&assets, &[root()], &packages(), &[], Limits::default()).unwrap();
        let ClassBody::Parsed { default, .. } = &result.records[&root()].body else {
            panic!()
        };
        assert!(
            matches!(&default.body,DefaultBody::Unresolved{issue:DefaultIssue::StateFrame,bytes,..} if !bytes.is_empty())
        );
        assert_eq!(result.statistics.explicit_properties, 0);
    }
    #[test]
    fn enforces_intake_payload_property_and_identity_limits() {
        let assets = assets();
        for limits in [
            Limits {
                classes: 0,
                ..Limits::default()
            },
            Limits {
                parent_depth: 0,
                ..Limits::default()
            },
            Limits {
                serialized_bytes: 117,
                ..Limits::default()
            },
            Limits {
                default_bytes: 1,
                ..Limits::default()
            },
            Limits {
                properties: 0,
                ..Limits::default()
            },
            Limits {
                identity_bytes: 1,
                ..Limits::default()
            },
        ] {
            assert!(prepare(&assets, &[root()], &packages(), &[], limits).is_err());
        }
        assert!(prepare(&assets, &[root()], &[], &[], Limits::default()).is_err());
        assert!(
            prepare(
                &assets,
                &[root()],
                &[packages()[0].clone(), packages()[0].clone()],
                &[],
                Limits::default()
            )
            .is_err()
        );
        assert!(
            prepare(
                &assets,
                &[AssetLocation {
                    package: 0,
                    export: 2
                }],
                &packages(),
                &[],
                Limits::default()
            )
            .is_err()
        );
    }
}
