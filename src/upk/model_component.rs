//! Saved Engine.ModelComponent records for the selected ordinary UE3 868/3
//! map-instance profile. No dependencies are loaded or rendered.
//!
//! Nonzero polymorphic element headers remain unsupported. Unconsumed suffixes
//! are retained. Native evidence and admission limits: docs/UPK_MODEL_COMPONENT.md.

use crate::upk::Package;
use crate::upk::objects::Span;
use crate::world_scene::SavedSceneObject;
use std::{collections::HashSet, io};

pub const MAX_MODEL_COMPONENT_ELEMENTS: usize = 4096;
pub const MAX_MODEL_COMPONENT_ITEMS: usize = 65_536;
const STATE_FRAME: u64 = 0x0200_0000_0000_0000;
const NAMED_COMPONENT_OUTER_FLAG: u64 = 0x200;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelComponentLink {
    /// Relative to the containing native component, element or array.
    pub native_offset: usize,
    /// Zero=null, positive=one-based export, negative=one-based import.
    pub reference: i32,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelComponentWord {
    pub span: Span,
    pub bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelComponentNodes {
    pub native_offset: usize,
    pub count_span: Span,
    pub values_span: Span,
    /// Ordered raw u16 Model node indices; range requires the linked Model.
    pub values: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelComponentReferences {
    pub count_span: Span,
    pub values_span: Span,
    pub links: Vec<ModelComponentLink>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelComponentRawRecord {
    pub span: Span,
    pub words: [u32; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelComponentRawRecords {
    pub count_span: Span,
    pub records_span: Span,
    pub records: Vec<ModelComponentRawRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelComponentElement {
    pub span: Span,
    /// Native pointer +0x30 serializer discriminator; only zero is admitted.
    pub header_30: ModelComponentWord,
    /// Native element slots +0 and +8, in serialization order.
    pub references: [ModelComponentLink; 2],
    pub nodes: ModelComponentNodes,
    /// Native element array +0x20. Values retain nulls, duplicates and order.
    pub references_20: ModelComponentReferences,
    /// Native element array +0x38; four raw words per record, names unproved.
    pub records_38: ModelComponentRawRecords,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelComponentObject {
    /// Retains checked object/outer/archetype identities, original prefix,
    /// tagged properties and complete native payload span.
    pub scene: SavedSceneObject,
    pub prefix_reference: ModelComponentLink,
    pub net_index: i32,
    pub model: ModelComponentLink,
    /// Native +0x240; no member name, scalar type or range is inferred.
    pub word_240: ModelComponentWord,
    pub elements_count_span: Span,
    pub elements_span: Span,
    pub elements: Vec<ModelComponentElement>,
    pub component_index: u16,
    pub component_index_span: Span,
    pub nodes: ModelComponentNodes,
    pub unparsed_native_tail: Span,
}

impl ModelComponentObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported("ModelComponent requires UE3 profile 868/3"));
        }
        validate_instance(package, export_index)?;
        let scene = SavedSceneObject::from_export(package, export_index, bytes)?;
        let mut input = Reader {
            bytes,
            position: 0,
            remaining_items: MAX_MODEL_COMPONENT_ITEMS,
        };
        let prefix_reference = input.reference(package, 0x60)?;
        let net_index = input.word()?.bits as i32;
        input.position = scene.native_payload.offset;
        let model = input.reference(package, 0x238)?;
        let word_240 = input.word()?;
        let (count, elements_count_span) = input.count(MAX_MODEL_COMPONENT_ELEMENTS, false)?;
        // Every admitted element needs at least24 bytes; the component then
        // needs its u16 index and final array count. Preflight before reserve.
        let minimum = count
            .checked_mul(24)
            .and_then(|n| n.checked_add(6))
            .ok_or_else(|| invalid("ModelComponent element minimum overflow"))?;
        input.available(minimum)?;
        let start = input.position;
        let mut elements = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(input.element(package)?);
        }
        let elements_span = Span {
            offset: start,
            length: input.position - start,
        };
        let component_index_span = input.take(2)?;
        let component_index =
            u16::from_le_bytes(component_index_span.bytes(bytes)?.try_into().unwrap());
        let nodes = input.nodes(0x248)?;
        let unparsed_native_tail = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        Ok(Self {
            scene,
            prefix_reference,
            net_index,
            model,
            word_240,
            elements_count_span,
            elements_span,
            elements,
            component_index,
            component_index_span,
            nodes,
            unparsed_native_tail,
        })
    }
}

fn engine_class(package: &Package, index: usize, expected: &str) -> bool {
    let import = |reference: i32| {
        (reference < 0)
            .then(|| package.imports.get(reference.unsigned_abs() as usize - 1))
            .flatten()
    };
    let named = |name, expected: &str| {
        package
            .resolve_name(name)
            .is_some_and(|value| value.eq_ignore_ascii_case(expected))
    };
    let Some(export) = package.exports.get(index) else {
        return false;
    };
    let Some(class) = import(export.class_index) else {
        return false;
    };
    let Some(outer) = import(class.outer_index) else {
        return false;
    };
    named(class.object_name, expected)
        && named(class.class_package, "Core")
        && named(class.class_name, "Class")
        && named(outer.object_name, "Engine")
        && named(outer.class_package, "Core")
        && named(outer.class_name, "Package")
        && outer.outer_index == 0
}

fn validate_instance(package: &Package, index: usize) -> io::Result<()> {
    let export = package
        .exports
        .get(index)
        .ok_or_else(|| invalid("ModelComponent export is out of range"))?;
    if !engine_class(package, index, "ModelComponent") {
        return Err(unsupported(
            "native component reader requires imported Engine.ModelComponent",
        ));
    }
    if export.object_flags & STATE_FRAME != 0 {
        return Err(unsupported(
            "ModelComponent state-frame prefix is unsupported",
        ));
    }
    if export.outer_index <= 0 || !engine_class(package, export.outer_index as usize - 1, "Level") {
        return Err(unsupported(
            "ModelComponent profile requires a local Engine.Level outer",
        ));
    }
    let mut current = i32::try_from(
        index
            .checked_add(1)
            .ok_or_else(|| invalid("component index overflow"))?,
    )
    .map_err(|_| invalid("component index exceeds signed range"))?;
    let mut visited = HashSet::new();
    while current != 0 {
        if current < 0 {
            return Err(unsupported(
                "component prefix requires a known local outer chain",
            ));
        }
        if visited.len() == 64 || !visited.insert(current) {
            return Err(invalid("component outer chain is cyclic or too deep"));
        }
        let object = package
            .exports
            .get(current as usize - 1)
            .ok_or_else(|| invalid("component outer reference is out of range"))?;
        if object.object_flags & NAMED_COMPONENT_OUTER_FLAG != 0 {
            return Err(unsupported("component template/name prefix is unsupported"));
        }
        current = object.outer_index;
    }
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    remaining_items: usize,
}
impl Reader<'_> {
    fn available(&self, length: usize) -> io::Result<Span> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("component field offset overflow"))?;
        if end > self.bytes.len() {
            return Err(invalid("truncated ModelComponent payload"));
        }
        Ok(Span {
            offset: self.position,
            length,
        })
    }
    fn take(&mut self, length: usize) -> io::Result<Span> {
        let span = self.available(length)?;
        self.position += length;
        Ok(span)
    }
    fn word(&mut self) -> io::Result<ModelComponentWord> {
        let span = self.take(4)?;
        Ok(ModelComponentWord {
            span,
            bits: u32::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap()),
        })
    }
    fn reference(
        &mut self,
        package: &Package,
        native_offset: usize,
    ) -> io::Result<ModelComponentLink> {
        let word = self.word()?;
        let reference = word.bits as i32;
        if (reference > 0 && reference as usize > package.exports.len())
            || (reference < 0 && reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid(
                "component object reference is outside package tables",
            ));
        }
        Ok(ModelComponentLink {
            native_offset,
            reference,
            span: word.span,
        })
    }
    fn count(&mut self, maximum: usize, budgeted: bool) -> io::Result<(usize, Span)> {
        let word = self.word()?;
        let signed = word.bits as i32;
        if signed < 0 {
            return Err(invalid("negative ModelComponent array count"));
        }
        let count = signed as usize;
        if count > maximum || (budgeted && count > self.remaining_items) {
            return Err(unsupported(
                "ModelComponent array count exceeds inspection budget",
            ));
        }
        if budgeted {
            self.remaining_items -= count;
        }
        Ok((count, word.span))
    }
    fn nodes(&mut self, native_offset: usize) -> io::Result<ModelComponentNodes> {
        let (count, count_span) = self.count(MAX_MODEL_COMPONENT_ITEMS, true)?;
        let length = count
            .checked_mul(2)
            .ok_or_else(|| invalid("component u16 array size overflow"))?;
        let values_span = self.take(length)?;
        let values = values_span
            .bytes(self.bytes)?
            .chunks_exact(2)
            .map(|raw| u16::from_le_bytes(raw.try_into().unwrap()))
            .collect();
        Ok(ModelComponentNodes {
            native_offset,
            count_span,
            values_span,
            values,
        })
    }
    fn references(&mut self, package: &Package) -> io::Result<ModelComponentReferences> {
        let (count, count_span) = self.count(MAX_MODEL_COMPONENT_ITEMS, true)?;
        let values_span = self.available(
            count
                .checked_mul(4)
                .ok_or_else(|| invalid("component reference array size overflow"))?,
        )?;
        let mut links = Vec::with_capacity(count);
        for _ in 0..count {
            links.push(self.reference(package, 0x20)?);
        }
        Ok(ModelComponentReferences {
            count_span,
            values_span,
            links,
        })
    }
    fn records(&mut self) -> io::Result<ModelComponentRawRecords> {
        let (count, count_span) = self.count(MAX_MODEL_COMPONENT_ITEMS, true)?;
        let records_span = self.available(
            count
                .checked_mul(16)
                .ok_or_else(|| invalid("component raw record array size overflow"))?,
        )?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let start = self.position;
            let mut words = [0; 4];
            for word in &mut words {
                *word = self.word()?.bits;
            }
            records.push(ModelComponentRawRecord {
                span: Span {
                    offset: start,
                    length: 16,
                },
                words,
            });
        }
        Ok(ModelComponentRawRecords {
            count_span,
            records_span,
            records,
        })
    }
    fn element(&mut self, package: &Package) -> io::Result<ModelComponentElement> {
        let start = self.position;
        let header_30 = self.word()?;
        if header_30.bits != 0 {
            return Err(unsupported(
                "ModelComponent polymorphic element payload is unsupported",
            ));
        }
        let references = [self.reference(package, 0)?, self.reference(package, 8)?];
        let nodes = self.nodes(0x10)?;
        let references_20 = self.references(package)?;
        let records_38 = self.records()?;
        Ok(ModelComponentElement {
            span: Span {
                offset: start,
                length: self.position - start,
            },
            header_30,
            references,
            nodes,
            references_20,
            records_38,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};
    fn name(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn base_package(size: usize) -> Package {
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
                names: table,
                exports: table,
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
                "None", "Core", "Class", "Package", "Engine", "Model", "Instance",
            ]
            .into_iter()
            .map(|value| NameEntry {
                value: value.into(),
                flags: 0,
            })
            .collect(),
            imports: vec![
                Import {
                    class_package: name(1),
                    class_name: name(2),
                    outer_index: -2,
                    object_name: name(5),
                },
                Import {
                    class_package: name(1),
                    class_name: name(3),
                    outer_index: 0,
                    object_name: name(4),
                },
            ],
            exports: vec![Export {
                class_index: -1,
                super_index: 0,
                outer_index: 0,
                object_name: name(6),
                archetype_index: 0,
                object_flags: 0,
                serial_size: size,
                serial_offset: 4096,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }

    fn package(size: usize) -> Package {
        let mut p = base_package(size);
        p.names[5].value = "ModelComponent".into();
        for value in [
            "Level",
            "LevelInstance",
            "World",
            "WorldInstance",
            "Model",
            "ModelInstance",
        ] {
            p.names.push(NameEntry {
                value: value.into(),
                flags: 0,
            });
        }
        for index in [7, 9, 11] {
            p.imports.push(Import {
                class_package: name(1),
                class_name: name(2),
                outer_index: -2,
                object_name: name(index),
            });
        }
        p.exports[0].outer_index = 2;
        for (class_index, outer_index, object_name) in [(-3, 3, 8), (-4, 0, 10), (-5, 2, 12)] {
            let mut e = p.exports[0].clone();
            e.class_index = class_index;
            e.outer_index = outer_index;
            e.object_name = name(object_name);
            e.serial_size = 0;
            p.exports.push(e);
        }
        p
    }
    fn word(b: &mut Vec<u8>, value: i32) {
        b.extend(value.to_le_bytes());
    }
    fn nodes(b: &mut Vec<u8>, values: &[u16]) {
        word(b, values.len() as i32);
        for value in values {
            b.extend(value.to_le_bytes());
        }
    }
    fn fixture() -> Vec<u8> {
        let mut b = Vec::new();
        for value in [0, -1, 0, 0, 4, 0x7fc00001, 2] {
            word(&mut b, value);
        }
        for _ in 0..2 {
            for value in [0, 1, -5] {
                word(&mut b, value);
            }
            nodes(&mut b, &[0, 65535, 0]);
            for value in [3, 0, -2, 4, 1] {
                word(&mut b, value);
            }
            for value in [0x80000000u32, 0x7fc00001, 0xffffffff, 17] {
                b.extend(value.to_le_bytes());
            }
        }
        b.extend(65535u16.to_le_bytes());
        nodes(&mut b, &[0, 65535, 0]);
        b
    }
    #[test]
    fn retains_fields_order_raw_values_and_unknown_suffix() {
        let mut b = fixture();
        let end = b.len();
        b.extend([23, 59]);
        let c = ModelComponentObject::parse_export(&package(b.len()), 0, &b).unwrap();
        assert_eq!(c.prefix_reference.reference, 0);
        assert_eq!(c.net_index, -1);
        assert_eq!(c.scene.properties.start_offset, 8);
        assert_eq!(c.scene.native_payload.offset, 16);
        assert_eq!(c.model.reference, 4);
        assert_eq!(c.word_240.bits, 0x7fc00001);
        assert_eq!(c.component_index, 65535);
        assert_eq!(c.elements.len(), 2);
        assert_eq!(c.elements_span.length, 116);
        for e in &c.elements {
            assert_eq!(e.span.length, 58);
            assert_eq!(e.header_30.bits, 0);
            assert_eq!(e.references.map(|r| r.reference), [1, -5]);
            assert_eq!(e.nodes.values, [0, 65535, 0]);
            assert_eq!(
                e.references_20
                    .links
                    .iter()
                    .map(|r| r.reference)
                    .collect::<Vec<_>>(),
                [0, -2, 4]
            );
            assert_eq!(
                e.records_38.records[0].words,
                [0x80000000, 0x7fc00001, 0xffffffff, 17]
            );
            assert_eq!(e.records_38.records[0].span.bytes(&b).unwrap().len(), 16);
        }
        assert_eq!(c.nodes.values, [0, 65535, 0]);
        assert_eq!(
            c.unparsed_native_tail,
            Span {
                offset: end,
                length: 2
            }
        );
        assert_eq!(c.unparsed_native_tail.bytes(&b).unwrap(), &[23, 59]);
    }
    #[test]
    fn rejects_truncated_fields_bad_counts_references_and_unimplemented_variants() {
        let b = fixture();
        let c = ModelComponentObject::parse_export(&package(b.len()), 0, &b).unwrap();
        for n in 0..b.len() {
            assert!(
                ModelComponentObject::parse_export(&package(n), 0, &b[..n]).is_err(),
                "length {n}"
            );
        }
        let mut counts = vec![c.elements_count_span, c.nodes.count_span];
        let mut refs = vec![c.prefix_reference, c.model];
        for e in &c.elements {
            counts.extend([
                e.nodes.count_span,
                e.references_20.count_span,
                e.records_38.count_span,
            ]);
            refs.extend(e.references);
            refs.extend(&e.references_20.links);
        }
        for span in counts {
            for (bits, kind) in [
                (u32::MAX, io::ErrorKind::InvalidData),
                (65537, io::ErrorKind::Unsupported),
            ] {
                let mut changed = b.clone();
                changed[span.offset..span.offset + 4].copy_from_slice(&bits.to_le_bytes());
                assert_eq!(
                    ModelComponentObject::parse_export(&package(changed.len()), 0, &changed)
                        .unwrap_err()
                        .kind(),
                    kind
                );
            }
        }
        for link in refs {
            for value in [5i32, -6, i32::MIN] {
                let mut changed = b.clone();
                changed[link.span.offset..link.span.offset + 4]
                    .copy_from_slice(&value.to_le_bytes());
                assert_eq!(
                    ModelComponentObject::parse_export(&package(changed.len()), 0, &changed)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::InvalidData
                );
            }
        }
        for e in c.elements {
            for value in [1u32, 2, 3, u32::MAX] {
                let mut changed = b.clone();
                let at = e.header_30.span.offset;
                changed[at..at + 4].copy_from_slice(&value.to_le_bytes());
                assert_eq!(
                    ModelComponentObject::parse_export(&package(changed.len()), 0, &changed)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::Unsupported
                );
            }
        }
    }
    #[test]
    fn enforces_exact_class_outer_and_no_name_instance_prefix() {
        let b = fixture();
        for mutation in 0..11 {
            let mut p = package(b.len());
            match mutation {
                0 => p.summary.file_version = 867,
                1 => p.summary.licensee_version = 4,
                2 => p.names[5].value = "StaticMeshComponent".into(),
                3 => p.names[4].value = "Custom".into(),
                4 => p.exports[0].outer_index = 4,
                5 => p.exports[0].outer_index = -2,
                6 => p.exports[0].object_flags = STATE_FRAME,
                7 => p.exports[0].object_flags = NAMED_COMPONENT_OUTER_FLAG,
                8 => p.exports[1].object_flags = NAMED_COMPONENT_OUTER_FLAG,
                9 => p.exports[2].outer_index = 1,
                10 => p.imports[0].outer_index = 0,
                _ => unreachable!(),
            }
            assert!(
                ModelComponentObject::parse_export(&p, 0, &b).is_err(),
                "mutation {mutation}"
            );
        }
        assert!(ModelComponentObject::parse_export(&package(b.len()), 4, &b).is_err());
    }
    fn budget_fixture(first: usize, last: usize) -> Vec<u8> {
        let mut b = Vec::new();
        for value in [0, -1, 0, 0, 4, 0, 1, 0, 1, -5] {
            word(&mut b, value);
        }
        word(&mut b, first as i32);
        b.extend(std::iter::repeat_n(0, first * 2));
        word(&mut b, 0);
        word(&mut b, 0);
        b.extend(0u16.to_le_bytes());
        word(&mut b, last as i32);
        b.extend(std::iter::repeat_n(0, last * 2));
        b
    }
    #[test]
    fn checks_cumulative_item_budget_and_empty_element_array() {
        let b = budget_fixture(32768, 32768);
        assert!(ModelComponentObject::parse_export(&package(b.len()), 0, &b).is_ok());
        let b = budget_fixture(32768, 32769);
        assert_eq!(
            ModelComponentObject::parse_export(&package(b.len()), 0, &b)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        let mut b = Vec::new();
        for value in [0, -1, 0, 0, 4, 0, 0] {
            word(&mut b, value);
        }
        b.extend(0u16.to_le_bytes());
        word(&mut b, 0);
        b.extend([41, 67]);
        let c = ModelComponentObject::parse_export(&package(b.len()), 0, &b).unwrap();
        assert!(c.elements.is_empty());
        assert!(c.nodes.values.is_empty());
        assert_eq!(c.unparsed_native_tail.bytes(&b).unwrap(), &[41, 67]);
    }
}
