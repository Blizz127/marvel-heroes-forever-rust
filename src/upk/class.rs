//! Saved UClass metadata for the bounded, observed UE3 868/3 file profile.
//!
//! This reads serialized declarations and the terminal default-object reference.
//! It does not construct classes, apply defaults, link native fields or run scripts.
//! See docs/UPK_CLASS.md for the profile's evidence and unsupported cases.

use crate::upk::objects::Span;
use crate::upk::{MAX_EXPORT_BYTES, NameRef, Package};
use std::io;

const HAS_STATE_FRAME: u64 = 0x0200_0000_0000_0000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

/// Every span is relative to the complete serialized export supplied by caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Array<T> {
    pub count: Field<u32>,
    /// Count and all entries, including an empty array's four-byte count.
    pub span: Span,
    pub entries: Vec<T>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameReference {
    pub name: Field<NameRef>,
    /// Null=0, positive=one-based export, negative=one-based import.
    pub reference: Field<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interface {
    /// Source-corroborated pair of raw indices. Their target tables and runtime
    /// meanings are not established here, so neither is resolved as an object.
    pub interface_index: Field<i32>,
    pub type_index: Field<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructData {
    pub super_class: Field<i32>,
    pub script_text: Field<i32>,
    pub children: Field<i32>,
    pub cpp_text: Field<i32>,
    pub line: Field<i32>,
    pub text_position: Field<i32>,
    /// Loaded and serialized byte counts are distinct; neither is recomputed.
    pub loaded_script_size: Field<u32>,
    pub serialized_script_size: Field<u32>,
    /// Opaque serialized script only. No bytecode decoding or execution.
    pub script: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateData {
    pub probe_mask: Field<u32>,
    pub label_table_offset: Field<u16>,
    pub flags: Field<u32>,
    pub functions: Array<NameReference>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeClassName {
    /// Observed profile admits zero or positive narrow-string counts only.
    pub count: Field<u32>,
    /// Includes the serialized trailing NUL, if count is nonzero.
    pub payload: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_export_bytes: usize,
    /// Applies separately to both script-size fields; no loaded buffer allocated.
    pub max_script_bytes: usize,
    pub max_array_entries: usize,
    /// Aggregate entries across every map and category array.
    pub max_total_entries: usize,
    pub max_native_name_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_export_bytes: MAX_EXPORT_BYTES,
            max_script_bytes: 1_048_576,
            max_array_entries: 10_000,
            max_total_entries: 20_000,
            max_native_name_bytes: 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassObject {
    pub export_index: usize,
    pub serial_offset: usize,
    pub span: Span,
    pub net_index: Field<i32>,
    pub next: Field<i32>,
    pub structure: StructData,
    pub state: StateData,
    /// Exact saved bits. No effective runtime flags are inferred.
    pub flags: Field<u32>,
    pub within: Field<i32>,
    pub config_name: Field<NameRef>,
    pub component_defaults: Array<NameReference>,
    pub interfaces: Array<Interface>,
    pub dont_sort_categories: Array<Field<NameRef>>,
    pub hide_categories: Array<Field<NameRef>>,
    pub auto_expand_categories: Array<Field<NameRef>>,
    pub auto_collapse_categories: Array<Field<NameRef>>,
    /// Raw four-byte value, restricted to the corroborated 0/1 profile.
    pub force_script_order: Field<u32>,
    pub class_groups: Array<Field<NameRef>>,
    pub native_class_name: NativeClassName,
    pub dll_bind_name: Field<NameRef>,
    /// Terminal serialized reference, never selected by a Default__ name search.
    /// A positive target is checked to have this exact class_index.
    /// Null and imports remain explicit; this reader never follows either.
    pub default_object: Field<i32>,
}

impl ClassObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        Self::parse_export_with_limits(package, export_index, bytes, Limits::default())
    }

    pub fn parse_export_with_limits(
        package: &Package,
        export_index: usize,
        bytes: &[u8],
        limits: Limits,
    ) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "UClass requires the observed UE3 868/3 file profile",
            ));
        }
        let export = package
            .exports
            .get(export_index)
            .ok_or_else(|| invalid("UClass export index is out of bounds"))?;
        if export.class_index != 0 {
            return Err(unsupported(
                "UClass requires the package's intrinsic Class identity",
            ));
        }
        if export.object_flags & HAS_STATE_FRAME != 0 {
            return Err(unsupported("UClass state-frame preamble is unsupported"));
        }
        if bytes.len() != export.serial_size {
            return Err(invalid("UClass byte length differs from export metadata"));
        }
        if bytes.len() > limits.max_export_bytes.min(MAX_EXPORT_BYTES) {
            return Err(unsupported("UClass exceeds export inspection budget"));
        }
        let mut input = Reader {
            package,
            bytes,
            position: 0,
            limits,
            total_entries: 0,
        };
        let net_index = input.signed()?;
        if net_index.value != -1 {
            return Err(unsupported(
                "UClass NetIndex differs from the observed -1 profile",
            ));
        }
        let next = input.reference()?;
        let super_class = input.reference()?;
        if super_class.value != export.super_index {
            return Err(invalid(
                "UClass embedded super differs from export-table super",
            ));
        }
        let script_text = input.reference()?;
        let children = input.reference()?;
        let cpp_text = input.reference()?;
        let line = input.signed()?;
        let text_position = input.signed()?;
        let loaded_script_size = input.count(limits.max_script_bytes)?;
        let serialized_script_size = input.count(limits.max_script_bytes)?;
        let script = input.take(serialized_script_size.value as usize)?;
        let structure = StructData {
            super_class,
            script_text,
            children,
            cpp_text,
            line,
            text_position,
            loaded_script_size,
            serialized_script_size,
            script,
        };
        let state = StateData {
            probe_mask: input.word()?,
            label_table_offset: input.short()?,
            flags: input.word()?,
            functions: input.array(12, Reader::name_reference)?,
        };
        let flags = input.word()?;
        let within = input.reference()?;
        let config_name = input.name()?;
        let component_defaults = input.array(12, Reader::name_reference)?;
        let interfaces = input.array(8, |reader| {
            Ok(Interface {
                interface_index: reader.signed()?,
                type_index: reader.signed()?,
            })
        })?;
        let dont_sort_categories = input.array(8, Reader::name)?;
        let hide_categories = input.array(8, Reader::name)?;
        let auto_expand_categories = input.array(8, Reader::name)?;
        let auto_collapse_categories = input.array(8, Reader::name)?;
        let force_script_order = input.word()?;
        if force_script_order.value > 1 {
            return Err(unsupported(
                "UClass ForceScriptOrder is outside the 0/1 profile",
            ));
        }
        let class_groups = input.array(8, Reader::name)?;
        let signed_count = input.signed()?;
        if signed_count.value < 0 {
            return Err(unsupported(
                "UClass wide native name is outside the observed profile",
            ));
        }
        if signed_count.value as usize > limits.max_native_name_bytes {
            return Err(unsupported("UClass native name exceeds inspection budget"));
        }
        let count = Field {
            span: signed_count.span,
            value: signed_count.value as u32,
        };
        let payload = input.take(count.value as usize)?;
        let native_bytes = payload.bytes(bytes)?;
        if let Some((&terminator, content)) = native_bytes.split_last() {
            if terminator != 0 {
                return Err(invalid("UClass native name lacks its trailing NUL"));
            }
            if !content.is_ascii() {
                return Err(unsupported(
                    "UClass native name is outside the narrow ASCII profile",
                ));
            }
        }
        let native_class_name = NativeClassName { count, payload };
        let dll_bind_name = input.name()?;
        let default_object = input.reference()?;
        if input.position != bytes.len() {
            return Err(unsupported(
                "UClass has trailing bytes outside the supported profile",
            ));
        }
        if default_object.value > 0 {
            let class_reference = i32::try_from(
                export_index
                    .checked_add(1)
                    .ok_or_else(|| invalid("UClass reference overflow"))?,
            )
            .map_err(|_| invalid("UClass reference exceeds signed object range"))?;
            if package.exports[default_object.value as usize - 1].class_index != class_reference {
                return Err(invalid(
                    "UClass default target has a different class identity",
                ));
            }
        }
        Ok(Self {
            export_index,
            serial_offset: export.serial_offset,
            span: Span {
                offset: 0,
                length: bytes.len(),
            },
            net_index,
            next,
            structure,
            state,
            flags,
            within,
            config_name,
            component_defaults,
            interfaces,
            dont_sort_categories,
            hide_categories,
            auto_expand_categories,
            auto_collapse_categories,
            force_script_order,
            class_groups,
            native_class_name,
            dll_bind_name,
            default_object,
        })
    }
}

struct Reader<'a> {
    package: &'a Package,
    bytes: &'a [u8],
    position: usize,
    limits: Limits,
    total_entries: usize,
}

impl Reader<'_> {
    fn take(&mut self, length: usize) -> io::Result<Span> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("UClass span overflow"))?;
        if end > self.bytes.len() {
            return Err(invalid("truncated UClass field"));
        }
        let span = Span {
            offset: self.position,
            length,
        };
        self.position = end;
        Ok(span)
    }
    fn word(&mut self) -> io::Result<Field<u32>> {
        let span = self.take(4)?;
        Ok(Field {
            span,
            value: u32::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap()),
        })
    }
    fn signed(&mut self) -> io::Result<Field<i32>> {
        let value = self.word()?;
        Ok(Field {
            span: value.span,
            value: value.value as i32,
        })
    }
    fn short(&mut self) -> io::Result<Field<u16>> {
        let span = self.take(2)?;
        Ok(Field {
            span,
            value: u16::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap()),
        })
    }
    fn reference(&mut self) -> io::Result<Field<i32>> {
        let field = self.signed()?;
        if (field.value > 0 && field.value as usize > self.package.exports.len())
            || (field.value < 0 && field.value.unsigned_abs() as usize > self.package.imports.len())
        {
            return Err(invalid("UClass object reference is outside package tables"));
        }
        Ok(field)
    }
    fn name(&mut self) -> io::Result<Field<NameRef>> {
        let index = self.signed()?;
        let number = self.signed()?;
        if index.value < 0 || index.value as usize >= self.package.names.len() || number.value < 0 {
            return Err(invalid("UClass FName is outside the supported name table"));
        }
        Ok(Field {
            span: Span {
                offset: index.span.offset,
                length: 8,
            },
            value: NameRef {
                index: index.value as usize,
                number: number.value as u32,
            },
        })
    }
    fn count(&mut self, maximum: usize) -> io::Result<Field<u32>> {
        let field = self.signed()?;
        if field.value < 0 {
            return Err(invalid("negative UClass count"));
        }
        if field.value as usize > maximum {
            return Err(unsupported("UClass count exceeds inspection budget"));
        }
        Ok(Field {
            span: field.span,
            value: field.value as u32,
        })
    }
    fn name_reference(&mut self) -> io::Result<NameReference> {
        Ok(NameReference {
            name: self.name()?,
            reference: self.reference()?,
        })
    }
    fn array<T>(
        &mut self,
        stride: usize,
        mut read: impl FnMut(&mut Self) -> io::Result<T>,
    ) -> io::Result<Array<T>> {
        let count = self.count(self.limits.max_array_entries)?;
        let length = count.value as usize;
        self.total_entries = self
            .total_entries
            .checked_add(length)
            .ok_or_else(|| invalid("UClass aggregate count overflow"))?;
        if self.total_entries > self.limits.max_total_entries {
            return Err(unsupported("UClass aggregate entry budget exceeded"));
        }
        let required = length
            .checked_mul(stride)
            .ok_or_else(|| invalid("UClass array byte count overflow"))?;
        if required > self.bytes.len() - self.position {
            return Err(invalid("truncated UClass array"));
        }
        let mut entries = Vec::with_capacity(length);
        for _ in 0..length {
            entries.push(read(self)?);
        }
        Ok(Array {
            span: Span {
                offset: count.span.offset,
                length: self.position - count.span.offset,
            },
            count,
            entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn name(bytes: &mut Vec<u8>, number: i32) {
        word(bytes, 0);
        word(bytes, number);
    }
    fn fixture() -> (Package, Vec<u8>) {
        let mut bytes = Vec::new();
        for value in [-1, 0, -1, 0, 0, 0, 8, 9, 5, 3] {
            word(&mut bytes, value);
        }
        bytes.extend_from_slice(&[0xaa, 0xbb, 0xcc]);
        word(&mut bytes, 0x12345678);
        bytes.extend_from_slice(&0xffffu16.to_le_bytes());
        word(&mut bytes, 0x10203040);
        word(&mut bytes, 2);
        // Duplicate map keys are retained, not silently deduplicated.
        for reference in [1, 2] {
            name(&mut bytes, 0);
            word(&mut bytes, reference);
        }
        word(&mut bytes, 0x08000096);
        word(&mut bytes, -1);
        name(&mut bytes, 0);
        word(&mut bytes, 1);
        name(&mut bytes, 0);
        word(&mut bytes, 2);
        word(&mut bytes, 1);
        word(&mut bytes, i32::MIN);
        word(&mut bytes, 0x7fffffff);
        for count in [2, 0, 1, 0] {
            word(&mut bytes, count);
            for number in 0..count {
                name(&mut bytes, number);
            }
        }
        word(&mut bytes, 1);
        word(&mut bytes, 1);
        name(&mut bytes, 0);
        word(&mut bytes, 11);
        bytes.extend_from_slice(b"UISequence\0");
        name(&mut bytes, 0);
        word(&mut bytes, 2);
        let mut package = Package::parse(&crate::upk::tests::fixture()).unwrap();
        package.exports[0].class_index = 0;
        package.exports[0].super_index = -1;
        package.exports[0].object_flags = 0;
        package.exports[0].serial_size = bytes.len();
        let mut default = package.exports[0].clone();
        default.class_index = 1;
        default.serial_size = 0;
        package.exports.push(default);
        (package, bytes)
    }
    fn replace(bytes: &mut [u8], span: Span, value: i32) {
        assert_eq!(span.length, 4);
        bytes[span.offset..span.offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn error(package: &Package, bytes: &[u8], kind: io::ErrorKind) {
        assert_eq!(
            ClassObject::parse_export(package, 0, bytes)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    #[test]
    fn retains_saved_fields_separate_script_lengths_and_raw_interface_words() {
        let (package, bytes) = fixture();
        let class = ClassObject::parse_export(&package, 0, &bytes).unwrap();
        assert_eq!(class.span.length, bytes.len());
        assert_eq!(class.structure.loaded_script_size.value, 5);
        assert_eq!(class.structure.serialized_script_size.value, 3);
        assert_eq!(
            class.structure.script.bytes(&bytes).unwrap(),
            [0xaa, 0xbb, 0xcc]
        );
        assert_eq!(class.state.label_table_offset.value, 0xffff);
        assert_eq!(
            class.state.flags.span.offset,
            class.state.label_table_offset.span.offset + 2
        );
        assert_eq!(class.state.functions.entries.len(), 2);
        assert_eq!(
            class.state.functions.entries[0].name.value,
            class.state.functions.entries[1].name.value
        );
        assert_eq!(class.state.functions.entries[0].reference.value, 1);
        assert_eq!(class.state.functions.entries[1].reference.value, 2);
        assert_eq!(class.interfaces.entries[0].interface_index.value, i32::MIN);
        assert_eq!(class.interfaces.entries[0].type_index.value, i32::MAX);
        assert_eq!(class.component_defaults.entries[0].reference.value, 2);
        assert_eq!(class.dont_sort_categories.entries[1].value.number, 1);
        assert_eq!(
            class.native_class_name.payload.bytes(&bytes).unwrap(),
            b"UISequence\0"
        );
        assert_eq!(class.default_object.value, 2);
        assert_eq!(class.default_object.span.offset + 4, bytes.len());
        assert_eq!(
            class
                .default_object
                .span
                .logical_offset(class.serial_offset)
                .unwrap(),
            class.serial_offset + bytes.len() - 4
        );
    }

    #[test]
    fn rejects_every_truncation_and_trailing_layout() {
        let (mut package, bytes) = fixture();
        for length in 0..bytes.len() {
            package.exports[0].serial_size = length;
            assert!(
                ClassObject::parse_export(&package, 0, &bytes[..length]).is_err(),
                "length {length}"
            );
        }
        package.exports[0].serial_size = bytes.len();
        error(
            &package,
            &bytes[..bytes.len() - 1],
            io::ErrorKind::InvalidData,
        );
        let mut trailing = bytes.clone();
        trailing.push(0);
        package.exports[0].serial_size = trailing.len();
        error(&package, &trailing, io::ErrorKind::Unsupported);
    }

    #[test]
    fn rejects_bad_indices_and_exact_class_default_mismatch() {
        let (package, bytes) = fixture();
        let class = ClassObject::parse_export(&package, 0, &bytes).unwrap();
        for (span, value) in [
            (class.next.span, i32::MIN),
            (class.structure.script_text.span, 3),
            (class.structure.super_class.span, 0),
            (class.default_object.span, 3),
            (
                Span {
                    offset: class.config_name.span.offset,
                    length: 4,
                },
                -1,
            ),
            (
                Span {
                    offset: class.config_name.span.offset,
                    length: 4,
                },
                package.names.len() as i32,
            ),
            (
                Span {
                    offset: class.config_name.span.offset + 4,
                    length: 4,
                },
                -1,
            ),
        ] {
            let mut changed = bytes.clone();
            replace(&mut changed, span, value);
            error(&package, &changed, io::ErrorKind::InvalidData);
        }
        let mut wrong_class = package.clone();
        // Same object_name remains; name agreement cannot establish identity.
        wrong_class.exports[1].class_index = -1;
        error(&wrong_class, &bytes, io::ErrorKind::InvalidData);
        for reference in [0, -1] {
            let mut changed = bytes.clone();
            replace(&mut changed, class.default_object.span, reference);
            assert_eq!(
                ClassObject::parse_export(&package, 0, &changed)
                    .unwrap()
                    .default_object
                    .value,
                reference
            );
        }
    }

    #[test]
    fn rejects_negative_counts_and_preflights_per_array_and_aggregate_budgets() {
        let (package, bytes) = fixture();
        let class = ClassObject::parse_export(&package, 0, &bytes).unwrap();
        for span in [
            class.state.functions.count.span,
            class.interfaces.count.span,
            class.class_groups.count.span,
            class.structure.loaded_script_size.span,
            class.structure.serialized_script_size.span,
        ] {
            let mut changed = bytes.clone();
            replace(&mut changed, span, -1);
            error(&package, &changed, io::ErrorKind::InvalidData);
        }
        let mut oversized = bytes.clone();
        replace(&mut oversized, class.state.functions.count.span, 10_001);
        error(&package, &oversized, io::ErrorKind::Unsupported);
        replace(&mut oversized, class.state.functions.count.span, 10_000);
        error(&package, &oversized, io::ErrorKind::InvalidData);
        for limits in [
            Limits {
                max_array_entries: 1,
                ..Limits::default()
            },
            Limits {
                max_total_entries: 3,
                ..Limits::default()
            },
            Limits {
                max_script_bytes: 4,
                ..Limits::default()
            },
            Limits {
                max_native_name_bytes: 10,
                ..Limits::default()
            },
            Limits {
                max_export_bytes: bytes.len() - 1,
                ..Limits::default()
            },
        ] {
            assert_eq!(
                ClassObject::parse_export_with_limits(&package, 0, &bytes, limits)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
    }

    #[test]
    fn rejects_unsupported_profiles_and_native_name_encodings() {
        let (package, bytes) = fixture();
        let class = ClassObject::parse_export(&package, 0, &bytes).unwrap();
        for variant in 0..4 {
            let mut changed = package.clone();
            match variant {
                0 => changed.summary.file_version = 867,
                1 => changed.summary.licensee_version = 4,
                2 => changed.exports[0].class_index = -1,
                _ => changed.exports[0].object_flags |= HAS_STATE_FRAME,
            }
            error(&changed, &bytes, io::ErrorKind::Unsupported);
        }
        for (span, value) in [
            (class.net_index.span, 0),
            (class.force_script_order.span, 2),
            (class.native_class_name.count.span, -1),
        ] {
            let mut changed = bytes.clone();
            replace(&mut changed, span, value);
            error(&package, &changed, io::ErrorKind::Unsupported);
        }
        let mut changed = bytes.clone();
        changed[class.native_class_name.payload.offset] = 0xff;
        error(&package, &changed, io::ErrorKind::Unsupported);
        let mut changed = bytes;
        let end = class.native_class_name.payload.offset + class.native_class_name.payload.length;
        changed[end - 1] = 1;
        error(&package, &changed, io::ErrorKind::InvalidData);
    }
}
