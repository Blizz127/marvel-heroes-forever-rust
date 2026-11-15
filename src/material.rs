//! Stored UE3 material properties and bounded local dependency inspection.
//! No shader evaluation, inherited parameter selection, or guessed texture roles.

use std::{collections::BTreeMap, io};

use crate::upk::{
    MAX_EXPORT_BYTES, NameRef, Package, PackageArchive,
    objects::{
        ObjectProperties, Property, PropertyList, Span, TagDetail, Value, parse_tagged_properties,
    },
};

const MAX_ARRAY: usize = 16_384;
const MAX_NESTED_PROPERTIES: usize = 65_536;
const MAX_OBJECT_DEPTH: usize = 128;
const MAX_OBJECT_NAME_BYTES: usize = 16_384;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialKind {
    Material,
    ConstantInstance,
}
impl MaterialKind {
    pub fn from_class(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("material") {
            Some(Self::Material)
        } else if name.eq_ignore_ascii_case("materialinstanceconstant") {
            Some(Self::ConstantInstance)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEnum {
    pub enumeration: NameRef,
    pub value: EnumValue,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumValue {
    Name(NameRef),
    Byte(u8),
}

/// Only values explicitly saved on this object. Absence does not supply defaults.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StoredSurfaceProperties {
    pub two_sided: Option<bool>,
    pub disable_depth_test: Option<bool>,
    pub is_masked: Option<bool>,
    pub blend_mode: Option<StoredEnum>,
    pub lighting_model: Option<StoredEnum>,
    pub opacity_mask_clip_bits: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterKind {
    Scalar,
    Vector,
    Texture,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterValue {
    ScalarBits(u32),
    /// Native LinearColor component order R, G, B, A; exact IEEE-754 bits.
    VectorBits([u32; 4]),
    TextureReference(i32),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub kind: ParameterKind,
    pub name: Option<NameRef>,
    pub value: Option<ParameterValue>,
    pub expression_guid: Option<[u8; 16]>,
    /// Retains every nested tag and its export-relative byte spans.
    pub properties: PropertyList,
    pub uninterpreted_properties: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Material {
    pub kind: MaterialKind,
    pub object: ObjectProperties,
    pub parent: Option<i32>,
    pub referenced_textures: Option<Vec<i32>>,
    pub expressions: Option<Vec<i32>>,
    pub scalar_parameters: Option<Vec<Parameter>>,
    pub vector_parameters: Option<Vec<Parameter>>,
    pub texture_parameters: Option<Vec<Parameter>>,
    pub surface: StoredSurfaceProperties,
    /// Top-level tags not interpreted by this module; their bytes remain opaque.
    pub uninterpreted_properties: Vec<usize>,
}

impl Material {
    pub fn from_export(package: &Package, index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported("material inspection requires UE3 868/3"));
        }
        let object = ObjectProperties::parse_export(package, index, bytes)?;
        let kind = MaterialKind::from_class(&object.class_name).ok_or_else(|| {
            unsupported("only Material and MaterialInstanceConstant are established")
        })?;
        let mut result = Self {
            kind,
            object,
            parent: None,
            referenced_textures: None,
            expressions: None,
            scalar_parameters: None,
            vector_parameters: None,
            texture_parameters: None,
            surface: StoredSurfaceProperties::default(),
            uninterpreted_properties: Vec::new(),
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut nested_count = 0;
        for (index, property) in result.object.properties.entries.iter().enumerate() {
            let name = package
                .resolve_name(property.name)
                .ok_or_else(|| invalid("unresolved material property name"))?
                .to_ascii_lowercase();
            let known = matches!(name.as_str(), "referencedtextures")
                || kind == MaterialKind::Material
                    && matches!(
                        name.as_str(),
                        "expressions"
                            | "twosided"
                            | "bdisabledepthtest"
                            | "bismasked"
                            | "blendmode"
                            | "lightingmodel"
                            | "opacitymaskclipvalue"
                    )
                || kind == MaterialKind::ConstantInstance
                    && matches!(
                        name.as_str(),
                        "parent"
                            | "scalarparametervalues"
                            | "vectorparametervalues"
                            | "textureparametervalues"
                    );
            if !known {
                result.uninterpreted_properties.push(index);
                continue;
            }
            if property.array_index != 0 || !seen.insert(name.clone()) {
                return Err(invalid(format!(
                    "duplicate or indexed material field {name}"
                )));
            }
            match name.as_str() {
                "referencedtextures" => {
                    result.referenced_textures = Some(reference_array(package, property, bytes)?)
                }
                "expressions" => {
                    result.expressions = Some(reference_array(package, property, bytes)?)
                }
                "parent" => result.parent = Some(object_value(property)?),
                "scalarparametervalues" => {
                    result.scalar_parameters = Some(parameter_array(
                        package,
                        property,
                        bytes,
                        ParameterKind::Scalar,
                        &mut nested_count,
                    )?)
                }
                "vectorparametervalues" => {
                    result.vector_parameters = Some(parameter_array(
                        package,
                        property,
                        bytes,
                        ParameterKind::Vector,
                        &mut nested_count,
                    )?)
                }
                "textureparametervalues" => {
                    result.texture_parameters = Some(parameter_array(
                        package,
                        property,
                        bytes,
                        ParameterKind::Texture,
                        &mut nested_count,
                    )?)
                }
                "twosided" => result.surface.two_sided = Some(bool_value(property)?),
                "bdisabledepthtest" => {
                    result.surface.disable_depth_test = Some(bool_value(property)?)
                }
                "bismasked" => result.surface.is_masked = Some(bool_value(property)?),
                "blendmode" => result.surface.blend_mode = Some(enum_value(property)?),
                "lightingmodel" => result.surface.lighting_model = Some(enum_value(property)?),
                "opacitymaskclipvalue" => {
                    result.surface.opacity_mask_clip_bits = Some(float_value(property)?)
                }
                _ => unreachable!(),
            }
        }
        Ok(result)
    }

    /// Enumerate saved references with their origin; no inherited override selection.
    pub fn dependencies(&self) -> Vec<Dependency> {
        let mut edges = Vec::new();
        if let Some(reference) = self.parent {
            edges.push(Dependency {
                reference,
                origin: DependencyOrigin::Parent,
            });
        }
        for (values, expression) in [
            (&self.referenced_textures, false),
            (&self.expressions, true),
        ] {
            if let Some(values) = values {
                for (item, &reference) in values.iter().enumerate() {
                    edges.push(Dependency {
                        reference,
                        origin: if expression {
                            DependencyOrigin::Expression { item }
                        } else {
                            DependencyOrigin::ReferencedTexture { item }
                        },
                    });
                }
            }
        }
        if let Some(parameters) = &self.texture_parameters {
            for (item, parameter) in parameters.iter().enumerate() {
                if let Some(ParameterValue::TextureReference(reference)) = parameter.value {
                    edges.push(Dependency {
                        reference,
                        origin: DependencyOrigin::TextureParameter { item },
                    });
                }
            }
        }
        for &property in &self.uninterpreted_properties {
            if let Value::Object(reference) = self.object.properties.entries[property].value {
                edges.push(Dependency {
                    reference,
                    origin: DependencyOrigin::OtherObjectProperty { property },
                });
            }
        }
        edges
    }
}

fn object_value(property: &Property) -> io::Result<i32> {
    if let Value::Object(value) = property.value {
        Ok(value)
    } else {
        Err(invalid("expected material object reference"))
    }
}
fn bool_value(property: &Property) -> io::Result<bool> {
    if let Value::Boolean(value) = property.value {
        Ok(value)
    } else {
        Err(invalid("expected material boolean"))
    }
}
fn float_value(property: &Property) -> io::Result<u32> {
    if let Value::FloatBits(value) = property.value {
        Ok(value)
    } else {
        Err(invalid("expected material float"))
    }
}
fn enum_value(property: &Property) -> io::Result<StoredEnum> {
    let TagDetail::Enumeration(enumeration) = property.detail else {
        return Err(invalid("expected material enumeration tag"));
    };
    let value = match property.value {
        Value::Name(value) => EnumValue::Name(value),
        Value::Byte(value) => EnumValue::Byte(value),
        _ => return Err(invalid("expected material enumeration value")),
    };
    Ok(StoredEnum { enumeration, value })
}
fn array(property: &Property) -> io::Result<(usize, Span)> {
    let Value::Array { count, elements } = property.value else {
        return Err(invalid("expected material array"));
    };
    if count > MAX_ARRAY {
        return Err(invalid("material array count exceeds limit"));
    }
    Ok((count, elements))
}
fn reference_array(package: &Package, property: &Property, bytes: &[u8]) -> io::Result<Vec<i32>> {
    let (count, span) = array(property)?;
    let bytes = span.bytes(bytes)?;
    if count.checked_mul(4) != Some(bytes.len()) {
        return Err(invalid("material reference array width/count mismatch"));
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| invalid("material allocation failed"))?;
    for word in bytes.chunks_exact(4) {
        let reference = i32::from_le_bytes(word.try_into().unwrap());
        check_reference(package, reference)?;
        values.push(reference);
    }
    Ok(values)
}
fn check_reference(package: &Package, reference: i32) -> io::Result<()> {
    if reference > 0 && reference as usize > package.exports.len()
        || reference < 0 && reference.unsigned_abs() as usize > package.imports.len()
    {
        Err(invalid("material reference outside package tables"))
    } else {
        Ok(())
    }
}

fn parameter_array(
    package: &Package,
    property: &Property,
    bytes: &[u8],
    kind: ParameterKind,
    nested_count: &mut usize,
) -> io::Result<Vec<Parameter>> {
    let (count, span) = array(property)?;
    if count > span.length / 8 {
        return Err(invalid("parameter count cannot fit terminators"));
    }
    let end = span
        .offset
        .checked_add(span.length)
        .ok_or_else(|| invalid("parameter array overflow"))?;
    let bounded = bytes
        .get(..end)
        .ok_or_else(|| invalid("parameter array outside export"))?;
    let mut position = span.offset;
    let mut parameters = Vec::new();
    parameters
        .try_reserve_exact(count)
        .map_err(|_| invalid("parameter allocation failed"))?;
    for _ in 0..count {
        // Restrict the existing tagged reader to this array's declared end.
        let properties = parse_tagged_properties(package, bounded, position)?;
        *nested_count = nested_count
            .checked_add(properties.entries.len())
            .ok_or_else(|| invalid("parameter property count overflow"))?;
        if *nested_count > MAX_NESTED_PROPERTIES {
            return Err(invalid("nested material property budget exceeded"));
        }
        position = properties.end_offset;
        let mut parameter = Parameter {
            kind,
            name: None,
            value: None,
            expression_guid: None,
            properties,
            uninterpreted_properties: Vec::new(),
        };
        let mut seen = std::collections::BTreeSet::new();
        for (index, field) in parameter.properties.entries.iter().enumerate() {
            let name = package
                .resolve_name(field.name)
                .ok_or_else(|| invalid("unresolved parameter field"))?
                .to_ascii_lowercase();
            if !matches!(
                name.as_str(),
                "parametername" | "parametervalue" | "expressionguid"
            ) {
                parameter.uninterpreted_properties.push(index);
                continue;
            }
            if field.array_index != 0 || !seen.insert(name.clone()) {
                return Err(invalid("duplicate or indexed parameter field"));
            }
            match name.as_str() {
                "parametername" => {
                    let Value::Name(value) = field.value else {
                        return Err(invalid("parameter name is not FName"));
                    };
                    parameter.name = Some(value);
                }
                "parametervalue" => {
                    parameter.value = Some(match kind {
                        ParameterKind::Scalar => ParameterValue::ScalarBits(float_value(field)?),
                        ParameterKind::Texture => {
                            ParameterValue::TextureReference(object_value(field)?)
                        }
                        ParameterKind::Vector => {
                            structure(package, field, "linearcolor")?;
                            let raw: [u8; 16] =
                                field.payload.bytes(bytes)?.try_into().map_err(|_| {
                                    invalid("LinearColor is not four f32 components")
                                })?;
                            ParameterValue::VectorBits(std::array::from_fn(|i| {
                                u32::from_le_bytes(raw[4 * i..4 * i + 4].try_into().unwrap())
                            }))
                        }
                    })
                }
                "expressionguid" => {
                    structure(package, field, "guid")?;
                    parameter.expression_guid = Some(
                        field
                            .payload
                            .bytes(bytes)?
                            .try_into()
                            .map_err(|_| invalid("parameter GUID width is not 16"))?,
                    );
                }
                _ => unreachable!(),
            }
        }
        parameters.push(parameter);
    }
    if position != end {
        return Err(invalid(
            "parameter arrays must exactly fill their declared span",
        ));
    }
    Ok(parameters)
}
fn structure(package: &Package, property: &Property, expected: &str) -> io::Result<()> {
    let Value::Structure { name } = property.value else {
        return Err(invalid("expected native material structure"));
    };
    if package
        .resolve_name(name)
        .is_some_and(|name| name.eq_ignore_ascii_case(expected))
    {
        Ok(())
    } else {
        Err(unsupported(format!(
            "unestablished material structure; expected {expected}"
        )))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencyOrigin {
    Parent,
    ReferencedTexture { item: usize },
    Expression { item: usize },
    TextureParameter { item: usize },
    OtherObjectProperty { property: usize },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dependency {
    pub reference: i32,
    pub origin: DependencyOrigin,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEdge {
    pub source: i32,
    pub dependency: Dependency,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectLocation {
    Null,
    Export(usize),
    Import(usize),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectIdentity {
    pub reference: i32,
    pub location: ObjectLocation,
    pub class_name: Option<String>,
    /// Package outer chain only. This is an object label, never a filesystem path.
    pub qualified_name: Option<String>,
}
impl ObjectIdentity {
    pub fn resolve(package: &Package, reference: i32) -> io::Result<Self> {
        check_reference(package, reference)?;
        if reference == 0 {
            return Ok(Self {
                reference,
                location: ObjectLocation::Null,
                class_name: None,
                qualified_name: None,
            });
        }
        let (location, class_name) = if reference > 0 {
            let index = reference as usize - 1;
            (
                ObjectLocation::Export(index),
                package.export_class_name(index),
            )
        } else {
            let index = reference.unsigned_abs() as usize - 1;
            (
                ObjectLocation::Import(index),
                package.resolve_name(package.imports[index].class_name),
            )
        };
        let class_name =
            class_name.ok_or_else(|| invalid("material dependency class is unresolved"))?;
        if class_name.len() > MAX_OBJECT_NAME_BYTES {
            return Err(invalid("material class name exceeds limit"));
        }
        let mut current = reference;
        let mut seen = std::collections::BTreeSet::new();
        let mut parts = Vec::new();
        let mut name_bytes = 0_usize;
        while current != 0 {
            check_reference(package, current)?;
            if parts.len() == MAX_OBJECT_DEPTH || !seen.insert(current) {
                return Err(invalid("material object outer cycle/depth exceeded"));
            }
            let name = package
                .resolve_object_name(current)
                .ok_or_else(|| invalid("material dependency name is unresolved"))?;
            name_bytes = name_bytes
                .checked_add(name.len() + 1)
                .ok_or_else(|| invalid("material object name size overflow"))?;
            if name_bytes > MAX_OBJECT_NAME_BYTES {
                return Err(invalid("material object name exceeds limit"));
            }
            parts.push(name);
            current = if current > 0 {
                package.exports[current as usize - 1].outer_index
            } else {
                package.imports[current.unsigned_abs() as usize - 1].outer_index
            };
        }
        parts.reverse();
        Ok(Self {
            reference,
            location,
            class_name: Some(class_name),
            qualified_name: Some(parts.join(".")),
        })
    }
}

pub trait MaterialSource {
    fn package(&self) -> &Package;
    fn read_export(&self, index: usize) -> io::Result<Vec<u8>>;
}
impl MaterialSource for PackageArchive {
    fn package(&self) -> &Package {
        self.package()
    }
    fn read_export(&self, index: usize) -> io::Result<Vec<u8>> {
        self.read_export(index)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GraphLimits {
    pub materials: usize,
    pub identities: usize,
    pub edges: usize,
    pub export_bytes: usize,
    pub name_bytes: usize,
    pub parent_depth: usize,
}
impl Default for GraphLimits {
    fn default() -> Self {
        Self {
            materials: 256,
            identities: 8192,
            edges: 32768,
            export_bytes: 32 * 1024 * 1024,
            name_bytes: 1024 * 1024,
            parent_depth: 64,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryReason {
    Null,
    ExternalImport,
    UnsupportedClass,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphBoundary {
    pub reference: i32,
    pub reason: BoundaryReason,
}
#[derive(Debug)]
pub struct MaterialNode {
    pub reference: i32,
    /// Owns the exact source bytes needed to inspect all opaque spans.
    pub bytes: Vec<u8>,
    pub material: Material,
}
#[derive(Debug)]
pub struct MaterialGraph {
    pub roots: Vec<i32>,
    pub materials: BTreeMap<i32, MaterialNode>,
    pub identities: BTreeMap<i32, ObjectIdentity>,
    pub edges: Vec<GraphEdge>,
    /// Root/parent targets where local Material/MIC traversal stops.
    pub boundaries: Vec<GraphBoundary>,
    pub loaded_bytes: usize,
}
impl MaterialGraph {
    pub fn inspect(source: &impl MaterialSource, roots: &[i32]) -> io::Result<Self> {
        Self::inspect_with_limits(source, roots, GraphLimits::default())
    }
    pub fn inspect_with_limits(
        source: &impl MaterialSource,
        roots: &[i32],
        limits: GraphLimits,
    ) -> io::Result<Self> {
        if (
            source.package().summary.file_version,
            source.package().summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported("material graph requires UE3 868/3"));
        }
        // Caller-supplied limits may tighten budgets, never remove hard ceilings.
        let defaults = GraphLimits::default();
        if limits.materials > defaults.materials
            || limits.identities > defaults.identities
            || limits.edges > defaults.edges
            || limits.export_bytes > defaults.export_bytes
            || limits.name_bytes > defaults.name_bytes
            || limits.parent_depth > defaults.parent_depth
            || roots.len() > limits.identities
        {
            return Err(invalid("material graph limits/roots exceed hard ceilings"));
        }
        let graph = Self {
            roots: roots.to_vec(),
            materials: BTreeMap::new(),
            identities: BTreeMap::new(),
            edges: Vec::new(),
            boundaries: Vec::new(),
            loaded_bytes: 0,
        };
        let mut build = GraphBuilder {
            source,
            limits,
            graph,
            name_bytes: 0,
            active: Vec::new(),
        };
        for &root in roots {
            build.visit(root)?;
        }
        Ok(build.graph)
    }
}
struct GraphBuilder<'a, S> {
    source: &'a S,
    limits: GraphLimits,
    graph: MaterialGraph,
    name_bytes: usize,
    active: Vec<i32>,
}
impl<S: MaterialSource> GraphBuilder<'_, S> {
    fn identify(&mut self, reference: i32) -> io::Result<()> {
        if self.graph.identities.contains_key(&reference) {
            return Ok(());
        }
        if self.graph.identities.len() == self.limits.identities {
            return Err(invalid("material identity budget exceeded"));
        }
        let identity = ObjectIdentity::resolve(self.source.package(), reference)?;
        let size = identity.qualified_name.as_ref().map_or(0, String::len)
            + identity.class_name.as_ref().map_or(0, String::len);
        self.name_bytes = self
            .name_bytes
            .checked_add(size)
            .ok_or_else(|| invalid("material name budget overflow"))?;
        if self.name_bytes > self.limits.name_bytes {
            return Err(invalid("material name budget exceeded"));
        }
        self.graph.identities.insert(reference, identity);
        Ok(())
    }
    fn visit(&mut self, reference: i32) -> io::Result<()> {
        self.identify(reference)?;
        if self.active.contains(&reference) {
            return Err(invalid("material parent cycle"));
        }
        if self.graph.materials.contains_key(&reference)
            || self
                .graph
                .boundaries
                .iter()
                .any(|v| v.reference == reference)
        {
            return Ok(());
        }
        let identity = &self.graph.identities[&reference];
        let reason = match identity.location {
            ObjectLocation::Null => Some(BoundaryReason::Null),
            ObjectLocation::Import(_) => Some(BoundaryReason::ExternalImport),
            ObjectLocation::Export(_)
                if identity
                    .class_name
                    .as_deref()
                    .and_then(MaterialKind::from_class)
                    .is_none() =>
            {
                Some(BoundaryReason::UnsupportedClass)
            }
            ObjectLocation::Export(_) => None,
        };
        if let Some(reason) = reason {
            self.graph
                .boundaries
                .push(GraphBoundary { reference, reason });
            return Ok(());
        }
        if self.active.len() == self.limits.parent_depth
            || self.graph.materials.len() == self.limits.materials
        {
            return Err(invalid("material parent depth/node budget exceeded"));
        }
        let ObjectLocation::Export(index) = identity.location else {
            unreachable!()
        };
        let size = self.source.package().exports[index].serial_size;
        self.graph.loaded_bytes = self
            .graph
            .loaded_bytes
            .checked_add(size)
            .ok_or_else(|| invalid("material byte budget overflow"))?;
        if size > MAX_EXPORT_BYTES || self.graph.loaded_bytes > self.limits.export_bytes {
            return Err(invalid("material byte budget exceeded before load"));
        }
        let bytes = self.source.read_export(index)?;
        let material = Material::from_export(self.source.package(), index, &bytes)?;
        let dependencies = material.dependencies();
        if dependencies.len() > self.limits.edges - self.graph.edges.len() {
            return Err(invalid("material edge budget exceeded"));
        }
        for dependency in dependencies {
            self.identify(dependency.reference)?;
            self.graph.edges.push(GraphEdge {
                source: reference,
                dependency,
            });
        }
        let parent = material.parent;
        self.graph.materials.insert(
            reference,
            MaterialNode {
                reference,
                bytes,
                material,
            },
        );
        self.active.push(reference);
        if let Some(parent) = parent {
            self.visit(parent)?;
        }
        self.active.pop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, Summary, TableLocation};
    use std::cell::Cell;

    const NAMES: &[&str] = &[
        "None",
        "Material",
        "MaterialInstanceConstant",
        "MaterialInstanceTimeVarying",
        "Class",
        "Package",
        "Texture2D",
        "External",
        "Tex",
        "ParentExternal",
        "A",
        "B",
        "C",
        "D",
        "Parent",
        "ObjectProperty",
        "ScalarParameterValues",
        "VectorParameterValues",
        "TextureParameterValues",
        "ArrayProperty",
        "ParameterName",
        "NameProperty",
        "ParameterValue",
        "FloatProperty",
        "StructProperty",
        "LinearColor",
        "ExpressionGUID",
        "Guid",
        "ExactParameter",
        "ReferencedTextures",
        "Expressions",
        "TwoSided",
        "BoolProperty",
        "BlendMode",
        "ByteProperty",
        "EBlendMode",
        "BLEND_Additive",
        "Unrecognized",
        "UnknownArray",
        "FutureField",
        "IntProperty",
    ];
    fn nr(name: &str) -> NameRef {
        NameRef {
            index: NAMES.iter().position(|&s| s == name).unwrap(),
            number: 0,
        }
    }
    fn fname(bytes: &mut Vec<u8>, name: &str) {
        let r = nr(name);
        bytes.extend((r.index as u32).to_le_bytes());
        bytes.extend(r.number.to_le_bytes());
    }
    fn tag(bytes: &mut Vec<u8>, name: &str, kind: &str, payload: &[u8], detail: Option<&str>) {
        fname(bytes, name);
        fname(bytes, kind);
        bytes.extend((payload.len() as i32).to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        if let Some(detail) = detail {
            fname(bytes, detail);
        }
        bytes.extend(payload);
    }
    fn array_tag(bytes: &mut Vec<u8>, name: &str, count: i32, elements: &[u8]) {
        let mut payload = count.to_le_bytes().to_vec();
        payload.extend(elements);
        tag(bytes, name, "ArrayProperty", &payload, None);
    }
    fn record(kind: ParameterKind) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut parameter_name = Vec::new();
        fname(&mut parameter_name, "ExactParameter");
        tag(
            &mut bytes,
            "ParameterName",
            "NameProperty",
            &parameter_name,
            None,
        );
        match kind {
            ParameterKind::Scalar => tag(
                &mut bytes,
                "ParameterValue",
                "FloatProperty",
                &0x7fc0_0012_u32.to_le_bytes(),
                None,
            ),
            ParameterKind::Vector => {
                let values = [-4.0_f32, 0.3, 8.0, 10.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>();
                tag(
                    &mut bytes,
                    "ParameterValue",
                    "StructProperty",
                    &values,
                    Some("LinearColor"),
                );
            }
            ParameterKind::Texture => tag(
                &mut bytes,
                "ParameterValue",
                "ObjectProperty",
                &(-4_i32).to_le_bytes(),
                None,
            ),
        }
        tag(
            &mut bytes,
            "ExpressionGUID",
            "StructProperty",
            &[0x5a; 16],
            Some("Guid"),
        );
        fname(&mut bytes, "None");
        bytes
    }
    fn instance(parent: i32) -> Vec<u8> {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        tag(
            &mut bytes,
            "Parent",
            "ObjectProperty",
            &parent.to_le_bytes(),
            None,
        );
        for (name, kind) in [
            ("ScalarParameterValues", ParameterKind::Scalar),
            ("VectorParameterValues", ParameterKind::Vector),
            ("TextureParameterValues", ParameterKind::Texture),
        ] {
            array_tag(&mut bytes, name, 1, &record(kind));
        }
        tag(
            &mut bytes,
            "Unrecognized",
            "ObjectProperty",
            &3_i32.to_le_bytes(),
            None,
        );
        array_tag(&mut bytes, "UnknownArray", 7, &[4, 3, 2, 1]);
        fname(&mut bytes, "None");
        bytes.extend([41, 43, 47]);
        bytes
    }
    fn base() -> Vec<u8> {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        array_tag(
            &mut bytes,
            "ReferencedTextures",
            3,
            &[3_i32, -4, 0]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        array_tag(
            &mut bytes,
            "Expressions",
            2,
            &[0_i32, 3]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        tag(&mut bytes, "TwoSided", "BoolProperty", &[], None);
        bytes.push(1);
        let mut blend = Vec::new();
        fname(&mut blend, "BLEND_Additive");
        tag(
            &mut bytes,
            "BlendMode",
            "ByteProperty",
            &blend,
            Some("EBlendMode"),
        );
        fname(&mut bytes, "None");
        bytes.extend([53, 59]);
        bytes
    }
    struct Source {
        package: Package,
        data: Vec<Vec<u8>>,
        loads: Cell<usize>,
    }
    impl MaterialSource for Source {
        fn package(&self) -> &Package {
            &self.package
        }
        fn read_export(&self, index: usize) -> io::Result<Vec<u8>> {
            self.loads.set(self.loads.get() + 1);
            Ok(self.data[index].clone())
        }
    }
    fn source(data: Vec<Vec<u8>>) -> Source {
        let table = |count| TableLocation { count, offset: 0 };
        let imports = [
            ("Material", "Class", 0),
            ("MaterialInstanceConstant", "Class", 0),
            ("External", "Package", 0),
            ("Tex", "Texture2D", -3),
            ("ParentExternal", "Material", -3),
            ("MaterialInstanceTimeVarying", "Class", 0),
            ("Texture2D", "Class", 0),
        ]
        .into_iter()
        .map(|(object, class, outer)| Import {
            class_package: nr("Package"),
            class_name: nr(class),
            outer_index: outer,
            object_name: nr(object),
        })
        .collect::<Vec<_>>();
        let exports = data
            .iter()
            .enumerate()
            .map(|(i, bytes)| Export {
                class_index: [-2, -1, -7, -6][i],
                super_index: 0,
                outer_index: 0,
                object_name: nr(["A", "B", "C", "D"][i]),
                archetype_index: 0,
                object_flags: 0,
                serial_size: bytes.len(),
                serial_offset: i * 1000,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            })
            .collect::<Vec<_>>();
        let summary = Summary {
            file_version: 868,
            licensee_version: 3,
            header_size: 0,
            group: String::new(),
            package_flags: 0,
            names: table(NAMES.len()),
            exports: table(exports.len()),
            imports: table(imports.len()),
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
        };
        Source {
            package: Package {
                summary,
                names: NAMES
                    .iter()
                    .map(|name| NameEntry {
                        value: (*name).into(),
                        flags: 0,
                    })
                    .collect(),
                imports,
                exports,
            },
            data,
            loads: Cell::new(0),
        }
    }
    fn usual() -> Source {
        source(vec![instance(2), base(), Vec::new(), Vec::new()])
    }

    #[test]
    fn exact_parameters_keep_bits_names_and_unknown_bytes() {
        let source = usual();
        let m = Material::from_export(&source.package, 0, &source.data[0]).unwrap();
        assert_eq!(m.parent, Some(2));
        assert_eq!(
            m.scalar_parameters.as_ref().unwrap()[0].value,
            Some(ParameterValue::ScalarBits(0x7fc0_0012))
        );
        assert_eq!(
            m.vector_parameters.as_ref().unwrap()[0].value,
            Some(ParameterValue::VectorBits(
                [-4.0_f32, 0.3, 8.0, 10.0].map(f32::to_bits)
            ))
        );
        let texture = &m.texture_parameters.as_ref().unwrap()[0];
        assert_eq!(texture.name, Some(nr("ExactParameter")));
        assert_eq!(texture.value, Some(ParameterValue::TextureReference(-4)));
        assert_eq!(texture.expression_guid, Some([0x5a; 16]));
        assert_eq!(m.uninterpreted_properties.len(), 2);
        assert_eq!(
            m.object.native_payload.bytes(&source.data[0]).unwrap(),
            [41, 43, 47]
        );
        let b = Material::from_export(&source.package, 1, &source.data[1]).unwrap();
        assert_eq!(b.surface.two_sided, Some(true));
        assert_eq!(b.surface.disable_depth_test, None);
        assert_eq!(
            b.surface.blend_mode.unwrap().value,
            EnumValue::Name(nr("BLEND_Additive"))
        );
    }

    #[test]
    fn missing_parameter_values_remain_absent_and_duplicate_names_keep_order() {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        let mut empty = Vec::new();
        tag(
            &mut empty,
            "FutureField",
            "IntProperty",
            &7_i32.to_le_bytes(),
            None,
        );
        fname(&mut empty, "None");
        let mut elements = empty;
        elements.extend(record(ParameterKind::Scalar));
        elements.extend(record(ParameterKind::Scalar));
        array_tag(&mut bytes, "ScalarParameterValues", 3, &elements);
        array_tag(&mut bytes, "ReferencedTextures", 0, &[]);
        fname(&mut bytes, "None");
        let s = source(vec![bytes]);
        let m = Material::from_export(&s.package, 0, &s.data[0]).unwrap();
        let p = m.scalar_parameters.unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p[0].name, None);
        assert_eq!(p[0].value, None);
        assert_eq!(p[0].uninterpreted_properties, [0]);
        assert_eq!(p[1].name, p[2].name);
        assert_eq!(m.parent, None);
        assert_eq!(m.referenced_textures, Some(Vec::new()));
    }

    #[test]
    fn parameter_array_cannot_consume_outside_declared_payload() {
        let mut record = record(ParameterKind::Scalar);
        record.truncate(record.len() - 8);
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        array_tag(&mut bytes, "ScalarParameterValues", 1, &record);
        fname(&mut bytes, "None");
        let s = source(vec![bytes]);
        assert!(Material::from_export(&s.package, 0, &s.data[0]).is_err());
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        array_tag(&mut bytes, "ScalarParameterValues", 16_385, &[]);
        fname(&mut bytes, "None");
        let s = source(vec![bytes]);
        assert!(Material::from_export(&s.package, 0, &s.data[0]).is_err());
    }

    #[test]
    fn incorrect_reference_width_and_duplicate_known_fields_fail() {
        for raw in [vec![1, 2, 3], i32::MIN.to_le_bytes().to_vec()] {
            let mut bytes = (-1_i32).to_le_bytes().to_vec();
            array_tag(&mut bytes, "ReferencedTextures", 1, &raw);
            fname(&mut bytes, "None");
            let s = source(vec![bytes]);
            assert!(Material::from_export(&s.package, 0, &s.data[0]).is_err());
        }
        let mut bytes = instance(0);
        bytes.truncate(bytes.len() - 11);
        tag(
            &mut bytes,
            "Parent",
            "ObjectProperty",
            &0_i32.to_le_bytes(),
            None,
        );
        fname(&mut bytes, "None");
        let s = source(vec![bytes, base(), Vec::new()]);
        assert!(Material::from_export(&s.package, 0, &s.data[0]).is_err());
    }

    #[test]
    fn graph_loads_local_parents_once_and_keeps_all_reference_origins() {
        let s = usual();
        let graph = MaterialGraph::inspect(&s, &[1, 2, 1]).unwrap();
        assert_eq!(s.loads.get(), 2);
        assert_eq!(graph.materials.len(), 2);
        assert_eq!(graph.edges.len(), 8);
        assert_eq!(
            graph.identities[&-4].qualified_name.as_deref(),
            Some("External.Tex")
        );
        assert_eq!(graph.identities[&-4].location, ObjectLocation::Import(3));
        assert!(graph.edges.iter().any(|edge| edge.source == 2
            && edge.dependency.reference == 0
            && matches!(
                edge.dependency.origin,
                DependencyOrigin::Expression { item: 0 }
            )));
        assert_eq!(graph.materials[&1].bytes, s.data[0]);
    }

    #[test]
    fn external_null_and_unsupported_parent_roots_are_explicit_boundaries() {
        let s = source(vec![instance(-5), base(), Vec::new(), Vec::new()]);
        let graph = MaterialGraph::inspect(&s, &[1, 0, 4]).unwrap();
        assert_eq!(s.loads.get(), 1);
        assert_eq!(
            graph.boundaries,
            [
                GraphBoundary {
                    reference: -5,
                    reason: BoundaryReason::ExternalImport
                },
                GraphBoundary {
                    reference: 0,
                    reason: BoundaryReason::Null
                },
                GraphBoundary {
                    reference: 4,
                    reason: BoundaryReason::UnsupportedClass
                }
            ]
        );
        assert!(!graph.materials.contains_key(&-5));
    }

    #[test]
    fn parent_cycles_and_depth_limits_fail() {
        let s = source(vec![instance(1), base(), Vec::new()]);
        assert!(MaterialGraph::inspect(&s, &[1]).is_err());
        let s = usual();
        let limits = GraphLimits {
            parent_depth: 1,
            ..GraphLimits::default()
        };
        assert!(MaterialGraph::inspect_with_limits(&s, &[1], limits).is_err());
        assert_eq!(s.loads.get(), 1);
    }

    #[test]
    fn graph_byte_budget_is_charged_before_loading_and_other_budgets_fail() {
        let s = usual();
        let limits = GraphLimits {
            export_bytes: s.data[0].len() - 1,
            ..GraphLimits::default()
        };
        assert!(MaterialGraph::inspect_with_limits(&s, &[1], limits).is_err());
        assert_eq!(s.loads.get(), 0);
        for limits in [
            GraphLimits {
                materials: 0,
                ..GraphLimits::default()
            },
            GraphLimits {
                identities: 1,
                ..GraphLimits::default()
            },
            GraphLimits {
                edges: 0,
                ..GraphLimits::default()
            },
            GraphLimits {
                name_bytes: 0,
                ..GraphLimits::default()
            },
        ] {
            assert!(MaterialGraph::inspect_with_limits(&usual(), &[1], limits).is_err());
        }
    }

    #[test]
    fn outer_cycles_bad_indices_and_wrong_profile_fail() {
        let mut s = usual();
        s.package.exports[0].outer_index = 1;
        assert!(ObjectIdentity::resolve(&s.package, 1).is_err());
        assert!(ObjectIdentity::resolve(&s.package, i32::MIN).is_err());
        s.package.summary.file_version = 867;
        assert!(MaterialGraph::inspect(&s, &[0]).is_err());
    }
}
