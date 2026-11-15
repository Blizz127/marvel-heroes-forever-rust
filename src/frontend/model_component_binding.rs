//! Explicit saved Level/Model/ModelComponent composition. No references are
//! discovered, imported, initialized or rendered. See MODEL_COMPONENT_BINDING.md.

use super::{AssetLocation, AssetSet};
use crate::upk::{
    level::{LevelLink, LevelLinkSlot, LevelObject},
    model::{ModelArraySlot, ModelBulkArray, ModelObject},
    model_component::{ModelComponentNodes, ModelComponentObject},
    objects::Span,
};
use std::{io, sync::Arc};

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn charge(used: &mut usize, amount: usize, limit: usize, name: &str) -> io::Result<()> {
    *used = used
        .checked_add(amount)
        .filter(|n| *n <= limit)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("ModelComponent binding {name} limit exceeded ({limit})"),
            )
        })?;
    Ok(())
}

/// All locations are zero-based indices in one explicitly supplied AssetSet.
/// Components must be the complete saved Level array in its original order.
#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    pub level: AssetLocation,
    pub model: AssetLocation,
    pub components: &'a [AssetLocation],
}

/// Inspection limits, not asserted native acceptance limits.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub components: usize,
    pub serialized_bytes: usize,
    pub properties: usize,
    pub elements: usize,
    /// Aggregate component and element node-list entries, including duplicates.
    pub node_indices: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            components: 4096,
            serialized_bytes: 32 * 1024 * 1024,
            properties: 65_536,
            elements: 65_536,
            node_indices: 262_144,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub component_slots: usize,
    pub serialized_bytes: usize,
    pub properties: usize,
    pub elements: usize,
    pub node_indices: usize,
}

/// Complete original export bytes accompany every parsed span and raw field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedExport<T> {
    pub location: AssetLocation,
    pub serial_offset: usize,
    pub bytes: Vec<u8>,
    pub object: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeBinding {
    pub index: u16,
    /// Two-byte index span relative to the owning component export.
    pub source_span: Span,
    /// Corresponding 64-byte Nodes record span relative to the selected Model.
    pub model_record: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementBinding {
    /// Same ordinal as the retained ModelComponentObject::elements entry.
    pub nodes: Vec<NodeBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentBinding {
    /// Saved Level array link, including its original ordinal and span.
    pub level_link: LevelLink,
    pub export: RetainedExport<ModelComponentObject>,
    pub nodes: Vec<NodeBinding>,
    pub elements: Vec<ElementBinding>,
}

/// Read-only result, bound to the exact AssetSet namespace that supplied bytes.
/// Clone retains that namespace even if the AssetSet is moved or dropped.
#[derive(Clone, Debug)]
pub struct Binding {
    asset_identity: Arc<()>,
    level: RetainedExport<LevelObject>,
    model: RetainedExport<ModelObject>,
    model_link: LevelLink,
    nodes: ModelBulkArray,
    components: Vec<ComponentBinding>,
    usage: Usage,
}
impl Binding {
    pub fn matches_assets(&self, assets: &AssetSet) -> bool {
        Arc::ptr_eq(&self.asset_identity, &assets.identity)
    }
    pub fn level(&self) -> &RetainedExport<LevelObject> {
        &self.level
    }
    pub fn model(&self) -> &RetainedExport<ModelObject> {
        &self.model
    }
    pub fn model_link(&self) -> LevelLink {
        self.model_link
    }
    pub fn model_nodes(&self) -> ModelBulkArray {
        self.nodes
    }
    pub fn components(&self) -> &[ComponentBinding] {
        &self.components
    }
    pub fn usage(&self) -> &Usage {
        &self.usage
    }
}

/// Read only the caller-selected exports. Local exact-class readers retain all
/// their original restrictions; this composition never guesses a missing link.
pub fn prepare(assets: &AssetSet, request: Request<'_>, limits: Limits) -> io::Result<Binding> {
    let mut usage = Usage::default();
    charge(
        &mut usage.component_slots,
        request.components.len(),
        limits.components,
        "components",
    )?;
    let archive = assets.archive(request.level.package)?;
    let package = archive.package();
    // Preflight every location and the complete retained-byte budget before I/O.
    for location in [request.level, request.model]
        .into_iter()
        .chain(request.components.iter().copied())
    {
        if location.package != request.level.package {
            return Err(invalid(
                "selected Level, Model and components must use the same archive",
            ));
        }
        let export = package
            .exports
            .get(location.export)
            .ok_or_else(|| invalid("selected export is outside the archive"))?;
        charge(
            &mut usage.serialized_bytes,
            export.serial_size,
            limits.serialized_bytes,
            "serialized bytes",
        )?;
    }
    let level_reference = reference(request.level)?;
    let model_reference = reference(request.model)?;
    let level_bytes = archive.read_export(request.level.export)?;
    let level = LevelObject::parse_export(package, request.level.export, &level_bytes)?;
    charge(
        &mut usage.properties,
        level.object.properties.entries.len(),
        limits.properties,
        "properties",
    )?;
    let model_link = selected_links(&level, request)?;
    let model_bytes = archive.read_export(request.model.export)?;
    let model = ModelObject::parse_export(package, request.model.export, &model_bytes)?;
    charge(
        &mut usage.properties,
        model.object.properties.entries.len(),
        limits.properties,
        "properties",
    )?;
    let nodes = model
        .arrays
        .iter()
        .find(|a| a.slot == ModelArraySlot::Nodes)
        .copied()
        .ok_or_else(|| invalid("selected Model has no Nodes array"))?;
    let mut components = Vec::with_capacity(request.components.len());
    for (&location, &level_link) in request
        .components
        .iter()
        .zip(&level.links[level.model_components.links.clone()])
    {
        if package.exports[location.export].outer_index != level_reference {
            return Err(invalid(
                "selected component outer does not match the selected Level",
            ));
        }
        let bytes = archive.read_export(location.export)?;
        let component = ModelComponentObject::parse_export(package, location.export, &bytes)?;
        if component.model.reference != model_reference {
            return Err(invalid(
                "saved component Model link does not match the selected Model",
            ));
        }
        charge(
            &mut usage.properties,
            component.scene.properties.entries.len(),
            limits.properties,
            "properties",
        )?;
        charge(
            &mut usage.elements,
            component.elements.len(),
            limits.elements,
            "elements",
        )?;
        let bound_nodes = bind_nodes(&component.nodes, nodes, &mut usage, limits)?;
        let mut elements = Vec::with_capacity(component.elements.len());
        for element in &component.elements {
            elements.push(ElementBinding {
                nodes: bind_nodes(&element.nodes, nodes, &mut usage, limits)?,
            });
        }
        components.push(ComponentBinding {
            level_link,
            export: RetainedExport {
                location,
                serial_offset: package.exports[location.export].serial_offset,
                bytes,
                object: component,
            },
            nodes: bound_nodes,
            elements,
        });
    }
    Ok(Binding {
        asset_identity: Arc::clone(&assets.identity),
        level: RetainedExport {
            location: request.level,
            serial_offset: package.exports[request.level.export].serial_offset,
            bytes: level_bytes,
            object: level,
        },
        model: RetainedExport {
            location: request.model,
            serial_offset: package.exports[request.model.export].serial_offset,
            bytes: model_bytes,
            object: model,
        },
        model_link,
        nodes,
        components,
        usage,
    })
}

fn reference(location: AssetLocation) -> io::Result<i32> {
    location
        .export
        .checked_add(1)
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| invalid("selected export index exceeds signed reference range"))
}
fn selected_links(level: &LevelObject, request: Request<'_>) -> io::Result<LevelLink> {
    let model = level
        .links
        .iter()
        .find(|l| l.slot == LevelLinkSlot::Model)
        .copied()
        .ok_or_else(|| invalid("Level has no saved Model link"))?;
    if model.reference != reference(request.model)? {
        return Err(invalid(
            "saved Level Model link does not match the selected Model",
        ));
    }
    let links = level
        .links
        .get(level.model_components.links.clone())
        .ok_or_else(|| invalid("Level ModelComponents slice is invalid"))?;
    if links.len() != request.components.len() {
        return Err(invalid(
            "explicit component list must match the complete saved Level array",
        ));
    }
    for (ordinal, (link, &selected)) in links.iter().zip(request.components).enumerate() {
        if link.slot != LevelLinkSlot::ModelComponent(ordinal)
            || link.reference != reference(selected)?
        {
            return Err(invalid(
                "explicit component order disagrees with the saved Level array",
            ));
        }
    }
    Ok(model)
}

fn bind_nodes(
    source: &ModelComponentNodes,
    model: ModelBulkArray,
    usage: &mut Usage,
    limits: Limits,
) -> io::Result<Vec<NodeBinding>> {
    charge(
        &mut usage.node_indices,
        source.values.len(),
        limits.node_indices,
        "node indices",
    )?;
    let mut out = Vec::with_capacity(source.values.len());
    for (ordinal, &index) in source.values.iter().enumerate() {
        let offset = ordinal
            .checked_mul(2)
            .and_then(|n| source.values_span.offset.checked_add(n))
            .ok_or_else(|| invalid("component node source span overflow"))?;
        out.push(NodeBinding {
            index,
            source_span: Span { offset, length: 2 },
            model_record: model.record_span(usize::from(index))?,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::PackageArchive;

    fn word(out: &mut Vec<u8>, value: i32) {
        out.extend(value.to_le_bytes());
    }
    fn name(out: &mut Vec<u8>, index: i32) {
        word(out, index);
        word(out, 0);
    }
    fn text(out: &mut Vec<u8>, value: &str) {
        word(out, value.len() as i32 + 1);
        out.extend(value.as_bytes());
        out.push(0);
    }
    fn patch(out: &mut [u8], at: usize, value: usize) {
        out[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
    fn bulk(out: &mut Vec<u8>, stride: i32, count: i32) {
        word(out, stride);
        word(out, count);
        out.extend(vec![0xa5; (stride * count) as usize]);
    }
    fn indices(out: &mut Vec<u8>, values: &[u16]) {
        word(out, values.len() as i32);
        for value in values {
            out.extend(value.to_le_bytes());
        }
    }
    fn level(model: i32, components: &[i32]) -> Vec<u8> {
        let mut b = Vec::new();
        // NetIndex, None tag, actor-array owner and empty actors.
        for v in [-1, 0, 0, 1, 0] {
            word(&mut b, v);
        }
        // Four empty strings, empty string array and two raw FURL words.
        for _ in 0..7 {
            word(&mut b, 0);
        }
        word(&mut b, model);
        word(&mut b, components.len() as i32);
        for &reference in components {
            word(&mut b, reference);
        }
        word(&mut b, 0);
        b.push(0x77);
        b
    }
    fn model() -> Vec<u8> {
        let mut b = Vec::new();
        for v in [-1, 0, 0] {
            word(&mut b, v);
        }
        for _ in 0..7 {
            word(&mut b, 0);
        }
        bulk(&mut b, 12, 0);
        bulk(&mut b, 12, 0);
        bulk(&mut b, 64, 2);
        word(&mut b, 0);
        word(&mut b, 0);
        bulk(&mut b, 24, 0);
        word(&mut b, 0);
        word(&mut b, 0);
        word(&mut b, 0);
        bulk(&mut b, 4, 0);
        bulk(&mut b, 4, 0);
        word(&mut b, 0);
        word(&mut b, 0);
        bulk(&mut b, 4, 0);
        word(&mut b, 0);
        bulk(&mut b, 36, 0);
        for _ in 0..5 {
            word(&mut b, 0);
        }
        b.extend([0x91, 0x92]);
        b
    }
    fn component(model: i32, element_nodes: &[u16], nodes: &[u16]) -> Vec<u8> {
        let mut b = Vec::new();
        for v in [0, -1, 0, 0, model, 0x7fc0_0001, 1, 0, 0, -6] {
            word(&mut b, v);
        }
        indices(&mut b, element_nodes);
        for v in [2, 0, -6, 1, -1, 0x7fc0_0001, 2, 3] {
            word(&mut b, v);
        }
        b.extend(65535u16.to_le_bytes());
        indices(&mut b, nodes);
        b.push(0x95);
        b
    }

    struct Fixture {
        payloads: Vec<Vec<u8>>,
        classes: [i32; 6],
        outers: [i32; 6],
    }
    impl Fixture {
        fn new() -> Self {
            Self {
                payloads: vec![
                    level(2, &[3, 4, 3]),
                    model(),
                    component(2, &[0, 0], &[1, 0, 1]),
                    component(2, &[1], &[]),
                    level(6, &[]),
                    model(),
                ],
                classes: [-2, -3, -4, -4, -2, -3],
                outers: [0, 1, 1, 1, 0, 5],
            }
        }
        /// Original synthetic package with six local exports, no retail bytes.
        fn archive(self) -> PackageArchive {
            let names = [
                "None",
                "Core",
                "Class",
                "Package",
                "Engine",
                "Level",
                "Model",
                "ModelComponent",
                "Material",
                "MaterialX",
                "FirstLevel",
                "FirstModel",
                "ComponentA",
                "ComponentB",
                "OtherLevel",
                "OtherModel",
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
            word(&mut out, 6);
            let exports_at = out.len();
            word(&mut out, 0);
            word(&mut out, 6);
            let imports_at = out.len();
            word(&mut out, 0);
            let depends_at = out.len();
            word(&mut out, 0);
            for _ in 0..4 {
                word(&mut out, 0);
            }
            out.extend([0; 16]);
            word(&mut out, 1);
            for v in [6, names.len() as i32, 0, 10897, 136, 0, 0, 0, 0, 0] {
                word(&mut out, v);
            }
            let start = out.len();
            patch(&mut out, names_at, start);
            for value in names {
                text(&mut out, value);
                out.extend(0u64.to_le_bytes());
            }
            let start = out.len();
            patch(&mut out, imports_at, start);
            for (package, kind, outer, object) in [
                (1, 3, 0, 4),
                (1, 2, -1, 5),
                (1, 2, -1, 6),
                (1, 2, -1, 7),
                (1, 2, -1, 8),
                (4, 8, -1, 9),
            ] {
                name(&mut out, package);
                name(&mut out, kind);
                word(&mut out, outer);
                name(&mut out, object);
            }
            let start = out.len();
            patch(&mut out, exports_at, start);
            let mut payload_offsets = Vec::new();
            for i in 0..6 {
                word(&mut out, self.classes[i]);
                word(&mut out, 0);
                word(&mut out, self.outers[i]);
                name(&mut out, 10 + i as i32);
                word(&mut out, 0);
                out.extend(0u64.to_le_bytes());
                word(&mut out, self.payloads[i].len() as i32);
                payload_offsets.push(out.len());
                word(&mut out, 0);
                word(&mut out, 0);
                word(&mut out, 0);
                out.extend([0; 16]);
                word(&mut out, 0);
            }
            let start = out.len();
            patch(&mut out, depends_at, start);
            for _ in 0..6 {
                word(&mut out, 0);
            }
            let header = out.len();
            patch(&mut out, 8, header);
            for (offset, payload) in payload_offsets.into_iter().zip(self.payloads) {
                let start = out.len();
                patch(&mut out, offset, start);
                out.extend(payload);
            }
            PackageArchive::parse(out).unwrap()
        }
    }
    fn location(export: usize) -> AssetLocation {
        AssetLocation { package: 0, export }
    }
    fn selected() -> [AssetLocation; 3] {
        [location(2), location(3), location(2)]
    }
    fn request(components: &[AssetLocation]) -> Request<'_> {
        Request {
            level: location(0),
            model: location(1),
            components,
        }
    }
    fn prepare_fixture(f: Fixture) -> io::Result<Binding> {
        prepare(
            &AssetSet::new(vec![f.archive()]).unwrap(),
            request(&selected()),
            Limits::default(),
        )
    }

    #[test]
    fn retains_original_fields_duplicate_links_and_non_partitioned_nodes() {
        let assets = AssetSet::new(vec![Fixture::new().archive()]).unwrap();
        let result = prepare(&assets, request(&selected()), Limits::default()).unwrap();
        assert_eq!(result.model_nodes().count, 2);
        assert_eq!(
            result
                .components()
                .iter()
                .map(|c| c.export.location.export)
                .collect::<Vec<_>>(),
            vec![2, 3, 2]
        );
        let c = &result.components()[0];
        assert_eq!(
            c.nodes.iter().map(|n| n.index).collect::<Vec<_>>(),
            vec![1, 0, 1]
        );
        assert_eq!(
            c.elements[0]
                .nodes
                .iter()
                .map(|n| n.index)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );
        assert_eq!(c.export.object.component_index, 65535);
        assert_eq!(c.export.object.elements[0].references[0].reference, 0);
        assert_eq!(c.export.object.elements[0].references[1].reference, -6);
        assert_eq!(
            c.export.object.elements[0].references_20.links[1].reference,
            -6
        );
        assert_eq!(c.export.object.word_240.bits, 0x7fc0_0001);
        assert_eq!(
            c.export.object.elements[0].records_38.records[0].words,
            [u32::MAX, 0x7fc0_0001, 2, 3]
        );
        for c in result.components() {
            for node in c
                .nodes
                .iter()
                .chain(c.elements.iter().flat_map(|e| e.nodes.iter()))
            {
                assert_eq!(
                    node.source_span.bytes(&c.export.bytes).unwrap(),
                    node.index.to_le_bytes()
                );
                assert_eq!(
                    node.model_record.bytes(&result.model().bytes).unwrap(),
                    &[0xa5; 64]
                );
            }
            assert_eq!(
                c.export
                    .object
                    .unparsed_native_tail
                    .bytes(&c.export.bytes)
                    .unwrap(),
                &[0x95]
            );
        }
        assert_eq!(
            result
                .level()
                .object
                .unparsed_native_tail
                .bytes(&result.level().bytes)
                .unwrap(),
            &[0x77]
        );
        assert_eq!(
            result
                .model()
                .object
                .unparsed_native_tail
                .bytes(&result.model().bytes)
                .unwrap(),
            &[0x91, 0x92]
        );
        assert!(result.matches_assets(&assets));
        let moved = assets;
        assert!(result.matches_assets(&moved));
        let other = AssetSet::new(vec![Fixture::new().archive()]).unwrap();
        assert!(!result.matches_assets(&other));
        assert_eq!(result.usage().node_indices, 11);
    }

    #[test]
    fn rejects_selection_link_outer_and_both_node_range_mismatches() {
        for case in 0..9 {
            let mut f = Fixture::new();
            match case {
                0 => f.payloads[0] = level(6, &[3, 4, 3]),
                1 => f.payloads[0] = level(2, &[4, 3, 3]),
                2 => f.payloads[0] = level(2, &[3, 4]),
                3 => f.outers[2] = 5,
                4 => f.payloads[2] = component(6, &[0], &[1]),
                5 => f.payloads[2] = component(2, &[2], &[1]),
                6 => f.payloads[2] = component(2, &[0], &[2]),
                7 => f.payloads[0] = level(2, &[0, 4, 3]),
                _ => f.payloads[0] = level(2, &[-4, 4, 3]),
            }
            assert_eq!(
                prepare_fixture(f).unwrap_err().kind(),
                io::ErrorKind::InvalidData,
                "case {case}"
            );
        }
        let mut f = Fixture::new();
        f.classes[1] = -2;
        assert_eq!(
            prepare_fixture(f).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        let mut f = Fixture::new();
        f.classes[2] = -3;
        assert_eq!(
            prepare_fixture(f).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn rejects_cross_archive_and_missing_selection_before_reading() {
        let assets =
            AssetSet::new(vec![Fixture::new().archive(), Fixture::new().archive()]).unwrap();
        let selected = selected();
        for case in 0..4 {
            let mut selected = selected;
            let mut r = request(&selected);
            match case {
                0 => r.model.package = 1,
                1 => {
                    selected[0].package = 1;
                    r = request(&selected);
                }
                2 => r.model.export = 6,
                _ => r.level.package = 2,
            }
            assert_eq!(
                prepare(&assets, r, Limits::default()).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn enforces_aggregate_budgets_and_allows_empty_component_selection() {
        let assets = AssetSet::new(vec![Fixture::new().archive()]).unwrap();
        let selected = selected();
        let full = prepare(&assets, request(&selected), Limits::default()).unwrap();
        let exact = Limits {
            components: 3,
            serialized_bytes: full.usage().serialized_bytes,
            properties: 0,
            elements: 3,
            node_indices: 11,
        };
        assert!(prepare(&assets, request(&selected), exact).is_ok());
        for case in 0..4 {
            let mut low = exact;
            match case {
                0 => low.components -= 1,
                1 => low.serialized_bytes -= 1,
                2 => low.elements -= 1,
                _ => low.node_indices -= 1,
            }
            assert_eq!(
                prepare(&assets, request(&selected), low)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
        let mut f = Fixture::new();
        f.payloads[0] = level(2, &[]);
        let assets = AssetSet::new(vec![f.archive()]).unwrap();
        let result = prepare(&assets, request(&[]), Limits::default()).unwrap();
        assert!(result.components().is_empty());
        assert_eq!(result.usage().node_indices, 0);
    }
}
