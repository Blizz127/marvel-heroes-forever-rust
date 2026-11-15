//! Explicit native GLOBAL/context association for bounded gfxExtensions values.
//!
//! Contract: docs/REA_GLOBAL_EXTENSIONS.md; retained verification.json SHA-256
//! 83fca8b24ea8b80af9457413898be5ca3db47bd2120f18df8a775cc184b1d202 and
//! setmember-review.json 2dab17ff8b0c8bad8f0388895e381294ce3d840d118c41b929e3c3425fac7e1c
//! under private evidence/rea-global-extensions/. Numeric conversion uses
//! rea-global-extensions-numeric/verification.json, SHA-256
//! 5b351c20fc1d488ea740098fbe540ebc35c7f30689e1e50371ce40de8776f188.
//! Native Integer payloads and already-correct IEEE64 payloads are supported;
//! nonzero f64 subnormals require a separate native FP-mode profile. This adds
//! no Float32 literal-origin/promotion parity claim. This opt-in subset does not
//! turn every Plain object into native GLOBAL or choose an initial context byte.
//! Resource preflight is an engineering bound, not native allocation parity.

use super::{
    CodeOrigin, ErrorKind, Kind, NativeMarker, ObjectId, Property, Result, Text, Value, Vm, text,
};

const EXTENSIONS: &str = "gfxExtensions";
const VERSION: &str = "gfxVersion";
const VERSION_TEXT: &str = "4.2.23";

/// Supplied native GLOBAL/context/no-watch witness, not an inferred default.
/// Every bound builtin target in this VM denotes the same explicitly adopted
/// context. `context_target` retains the target through which it was adopted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalExtensionsProfile {
    pub context_target: ObjectId,
    /// None records unknown adopted state; it never means native byte zero.
    pub current_byte: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalExtensionsInfo {
    pub global: ObjectId,
    pub context_target: ObjectId,
    pub current_byte: Option<u8>,
    pub last_write: Option<CodeOrigin>,
    /// Result of the most recent final marker attempt, not initial membership.
    /// ReadOnly can leave Some(false) after the context/version effects commit.
    pub marker_written: Option<bool>,
}

#[derive(Debug)]
pub(super) struct GlobalExtensionsState {
    info: GlobalExtensionsInfo,
}

/// Missing is the special getter's found=false result, not permission to fall
/// back to a stored gfxExtensions slot. Inactive alone permits ordinary lookup.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum GlobalExtensionsLookup {
    Inactive,
    Missing,
    Present(Value),
}

fn key_is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

impl Vm {
    /// Adopt an explicit native-global/same-context/no-watch profile once, while
    /// idle. Current selected data and known raw state are independent inputs.
    /// Unsupported resolver, parent, accessor or cleanup paths are not inferred
    /// safe from a matching object number or a previously stored Boolean.
    pub fn bind_global_extensions(&mut self, profile: GlobalExtensionsProfile) -> Result<()> {
        if self.global_extensions.is_some() {
            return self.fail(ErrorKind::Unsupported("global extensions already bound"));
        }
        if self.depth != 0 || self.saved_active || self.current_target.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "global extensions binding requires idle VM",
            ));
        }
        if !self.is_bound_builtin_context_target(profile.context_target)? {
            return self.fail(ErrorKind::Unsupported(
                "global extensions requires bound context target",
            ));
        }
        self.global_extensions_object_profile(self.global)?;
        self.global_extensions = Some(GlobalExtensionsState {
            info: GlobalExtensionsInfo {
                global: self.global,
                context_target: profile.context_target,
                current_byte: profile.current_byte,
                last_write: None,
                marker_written: None,
            },
        });
        Ok(())
    }

    pub fn global_extensions_info(&self) -> Option<GlobalExtensionsInfo> {
        self.global_extensions.as_ref().map(|state| state.info)
    }

    /// Exact native getter state precedes and ignores the stored property.
    /// The context byte zero result remains Missing even with an old own slot.
    pub(super) fn global_extensions_read(
        &self,
        id: ObjectId,
        key: &Text,
    ) -> Result<GlobalExtensionsLookup> {
        if !self.global_extensions_key(id, key) {
            return Ok(GlobalExtensionsLookup::Inactive);
        }
        let info = self.global_extensions_context()?;
        match info.current_byte {
            None => self.fail(ErrorKind::Unsupported("global extensions byte is unknown")),
            Some(0) => Ok(GlobalExtensionsLookup::Missing),
            Some(byte) => Ok(GlobalExtensionsLookup::Present(Value::Bool(byte == 1))),
        }
    }

    /// Returns true when this native route completed, including the normal
    /// setter's final ReadOnly false outcome. SetMember ignores that Boolean.
    /// False means only that this is not the bound exact receiver/name pair.
    pub(super) fn global_extensions_write(
        &mut self,
        id: ObjectId,
        key: &Text,
        value: &Value,
    ) -> Result<bool> {
        if !self.global_extensions_key(id, key) {
            return Ok(false);
        }
        let enabled = self.global_extensions_boolean(value)?;
        if self.current.is_some_and(|(_, opcode)| opcode != 0x4f) {
            return self.fail(ErrorKind::Unsupported(
                "global extensions requires SetMember caller flags",
            ));
        }
        let info = self.global_extensions_context()?;
        let extension_key = text(EXTENSIONS);
        let version_key = text(VERSION);
        let has_extension = self.objects[id.0].properties.contains_key(&extension_key);
        let has_version = self.objects[id.0].properties.contains_key(&version_key);
        let extension_flags = self.own_flags(id, &extension_key);
        let version_flags = self.own_flags(id, &version_key);
        let write_marker = !has_extension || extension_flags & 4 == 0;

        // Reserve enough budget for all future writes before the first native
        // logical effect. Deletion does not refund this VM's cumulative charges.
        let properties =
            usize::from(!has_extension && write_marker) + usize::from(enabled && !has_version);
        let units = usize::from(!has_extension && write_marker) * EXTENSIONS.len()
            + usize::from(enabled && !has_version) * VERSION.len()
            + usize::from(enabled) * VERSION_TEXT.len();
        if properties > self.limits.properties.saturating_sub(self.properties) {
            return self.fail(ErrorKind::Budget("properties"));
        }
        if units > self.limits.string_units.saturating_sub(self.string_units) {
            return self.fail(ErrorKind::Budget("global extensions string units"));
        }

        // 1e99a4: context byte changes BEFORE either auxiliary property effect.
        let origin = self.origin();
        let state = self.global_extensions.as_mut().unwrap();
        state.info.current_byte = Some(if enabled { 1 } else { 2 });
        state.info.last_write = origin;
        state.info.marker_written = None;

        if enabled {
            // 1e9a4a: raw setter bypasses ReadOnly but preserves existing flags.
            // Preflight above covers this value and any new property-name charge.
            let version = Value::String(self.allocate_text(VERSION_TEXT.encode_utf16().collect())?);
            self.raw_member(
                id,
                version_key,
                Property::Data(version),
                if has_version { version_flags } else { 0 },
            )?;
        } else if has_version && version_flags & 2 == 0 {
            // Same logical own removal as delete_member, without entering the
            // public protected-key guard or invoking accessors/watch callbacks.
            // Keep insertion history and cumulative property/string accounting.
            self.objects[id.0].properties.remove(&version_key);
            self.objects[id.0].flags.remove(&version_key);
            self.objects[id.0].native_hash_profile = false;
        }

        // 1e9ad7..1e9ae1: normal marker write is last. ReadOnly is a completed
        // native outcome; it must not undo the preceding byte/version effects.
        if write_marker {
            self.raw_member(
                id,
                extension_key,
                Property::NativeMarker(NativeMarker::GlobalExtensions),
                if has_extension { extension_flags } else { 0 },
            )?;
        }
        self.global_extensions.as_mut().unwrap().info.marker_written = Some(write_marker);
        debug_assert_eq!(
            self.global_extensions.as_ref().unwrap().info.global,
            info.global
        );
        Ok(true)
    }

    /// Narrow exception for the deferred registry's ordinary namespace guard.
    /// No other native marker or identically named ordinary object is admitted.
    pub(super) fn is_owned_global_extensions_marker(
        &self,
        id: ObjectId,
        key: &Text,
        property: &Property,
    ) -> bool {
        self.global_extensions_key(id, key)
            && matches!(
                property,
                Property::NativeMarker(NativeMarker::GlobalExtensions)
            )
            && matches!(
                self.objects[id.0].properties.get(key),
                Some(Property::NativeMarker(NativeMarker::GlobalExtensions))
            )
            && self.global_extensions_object_profile(id).is_ok()
    }

    // Native 17d730: tag4 tests its dword against zero. Signed32 integers are
    // exact in the VM's f64 representation. Tag3 excludes NaNs by integer bits
    // before its FP zero comparison. Classify represented bits without a
    // signalling-NaN comparison or broader String/Object/display conversion.
    fn global_extensions_boolean(&self, value: &Value) -> Result<bool> {
        match value {
            Value::Bool(boolean) => Ok(*boolean),
            Value::Number(number) => {
                let magnitude = number.to_bits() & 0x7fff_ffff_ffff_ffff;
                let exponent = magnitude & 0x7ff0_0000_0000_0000;
                let fraction = magnitude & 0x000f_ffff_ffff_ffff;
                if exponent == 0 && fraction != 0 {
                    // UCOMISD denormal handling depends on unproved MXCSR state.
                    return self.fail(ErrorKind::Unsupported(
                        "global extensions subnormal numeric profile",
                    ));
                }
                let nan = exponent == 0x7ff0_0000_0000_0000 && fraction != 0;
                Ok(magnitude != 0 && !nan)
            }
            _ => self.fail(ErrorKind::Unsupported(
                "global extensions requires proved Boolean or numeric payload",
            )),
        }
    }

    fn global_extensions_key(&self, id: ObjectId, key: &Text) -> bool {
        self.global_extensions.as_ref().is_some_and(|state| {
            id == self.global && id == state.info.global && key_is(key, EXTENSIONS)
        })
    }

    fn global_extensions_context(&self) -> Result<GlobalExtensionsInfo> {
        let Some(state) = self.global_extensions.as_ref() else {
            return self.fail(ErrorKind::Unsupported("global extensions is not bound"));
        };
        let info = state.info;
        if info.global != self.global {
            return self.fail(ErrorKind::Unsupported(
                "global extensions GLOBAL identity changed",
            ));
        }
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "global extensions requires current target",
            ));
        };
        // Both associations explicitly select this VM's one adopted context.
        // Different target identities do not imply different native contexts.
        if !self.is_bound_builtin_context_target(info.context_target)?
            || !self.is_bound_builtin_context_target(target)?
        {
            return self.fail(ErrorKind::Unsupported(
                "global extensions requires same bound context",
            ));
        }
        self.global_extensions_object_profile(info.global)?;
        Ok(info)
    }

    fn global_extensions_object_profile(&self, global: ObjectId) -> Result<()> {
        self.check_object(global)?;
        let object = &self.objects[global.0];
        if !matches!(object.kind, Kind::Plain)
            || object.prototype.is_some()
            || object
                .insertion_keys
                .iter()
                .any(|key| key_is(key, "__resolve"))
        {
            return self.fail(ErrorKind::Unsupported(
                "global extensions parent/resolver profile",
            ));
        }
        for name in [EXTENSIONS, VERSION] {
            let key = text(name);
            if self.own_flags(global, &key) > 7 {
                return self.fail(ErrorKind::Unsupported(
                    "global extensions descriptor flags profile",
                ));
            }
            match object.properties.get(&key) {
                None => {}
                Some(Property::Data(
                    Value::Undefined
                    | Value::Null
                    | Value::Bool(_)
                    | Value::Number(_)
                    | Value::String(_),
                )) => {}
                Some(Property::NativeMarker(NativeMarker::GlobalExtensions))
                    if name == EXTENSIONS => {}
                _ => {
                    return self.fail(ErrorKind::Unsupported(
                        "global extensions selected descriptor profile",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CodeId, Limits};
    use super::*;

    fn base() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.bind_builtin_context_target(vm.root).unwrap();
        vm
    }
    fn adopt(vm: &mut Vm, byte: Option<u8>) {
        vm.bind_global_extensions(GlobalExtensionsProfile {
            context_target: vm.root,
            current_byte: byte,
        })
        .unwrap();
        vm.current_target = Some(vm.root);
    }
    fn selected(vm: &mut Vm, name: &str, property: Property, flags: u8) {
        vm.raw_member(vm.global, text(name), property, flags)
            .unwrap();
    }
    fn write(vm: &mut Vm, enabled: bool) {
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(enabled))
                .unwrap()
        );
    }
    fn read(vm: &Vm) -> GlobalExtensionsLookup {
        vm.global_extensions_read(vm.global, &text(EXTENSIONS))
            .unwrap()
    }
    fn counts(vm: &Vm) -> (usize, usize, usize) {
        (vm.objects.len(), vm.properties, vm.string_units)
    }
    fn is_marker(vm: &Vm) -> bool {
        matches!(
            vm.objects[vm.global.0].properties.get(&text(EXTENSIONS)),
            Some(Property::NativeMarker(NativeMarker::GlobalExtensions))
        )
    }
    fn version(vm: &Vm) -> Option<Value> {
        match vm.objects[vm.global.0].properties.get(&text(VERSION)) {
            Some(Property::Data(value)) => Some(value.clone()),
            None => None,
            _ => panic!("expected ordinary version slot"),
        }
    }

    #[test]
    fn global_extensions_is_opt_in_and_exact_global_key_only() {
        let mut vm = base();
        selected(&mut vm, EXTENSIONS, Property::Data(Value::Bool(true)), 0);
        assert_eq!(read(&vm), GlobalExtensionsLookup::Inactive);
        assert!(
            !vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(false))
                .unwrap()
        );
        adopt(&mut vm, Some(0));
        assert_eq!(read(&vm), GlobalExtensionsLookup::Missing);
        assert_eq!(
            vm.global_extensions_read(vm.root, &text(EXTENSIONS))
                .unwrap(),
            GlobalExtensionsLookup::Inactive
        );
        assert_eq!(
            vm.global_extensions_read(vm.global, &text("gfxextensions"))
                .unwrap(),
            GlobalExtensionsLookup::Inactive
        );
        assert!(
            !vm.global_extensions_write(vm.root, &text(EXTENSIONS), &Value::Bool(true))
                .unwrap()
        );
        assert!(matches!(
            vm.objects[vm.global.0].properties.get(&text(EXTENSIONS)),
            Some(Property::Data(Value::Bool(true)))
        ));
    }

    #[test]
    fn global_extensions_raw_byte_controls_presence_not_old_slot_or_flags() {
        for (byte, expected) in [
            (0, GlobalExtensionsLookup::Missing),
            (1, GlobalExtensionsLookup::Present(Value::Bool(true))),
            (2, GlobalExtensionsLookup::Present(Value::Bool(false))),
            (255, GlobalExtensionsLookup::Present(Value::Bool(false))),
        ] {
            let mut vm = base();
            selected(
                &mut vm,
                EXTENSIONS,
                Property::Data(Value::Bool(byte != 1)),
                7,
            );
            adopt(&mut vm, Some(byte));
            assert_eq!(read(&vm), expected);
            assert_eq!(vm.global_extensions_info().unwrap().marker_written, None);
        }
        let mut vm = base();
        selected(&mut vm, EXTENSIONS, Property::Data(Value::Bool(true)), 0);
        adopt(&mut vm, None);
        assert!(
            vm.global_extensions_read(vm.global, &text(EXTENSIONS))
                .is_err()
        );
        assert_eq!(vm.global_extensions_info().unwrap().current_byte, None);
        write(&mut vm, false);
        assert_eq!(
            read(&vm),
            GlobalExtensionsLookup::Present(Value::Bool(false))
        );
    }

    #[test]
    fn global_extensions_true_then_false_tracks_real_marker_version_and_origin() {
        let mut vm = base();
        adopt(&mut vm, None);
        vm.current_code = Some(CodeId(0));
        vm.current = Some((6370, 0x4f));
        let before = counts(&vm);
        write(&mut vm, true);
        assert_eq!(counts(&vm), (before.0, before.1 + 2, before.2 + 29));
        assert!(is_marker(&vm));
        assert_eq!(version(&vm), Some(Value::String(text(VERSION_TEXT))));
        assert_eq!(vm.own_flags(vm.global, &text(VERSION)), 0);
        assert_eq!(vm.own_flags(vm.global, &text(EXTENSIONS)), 0);
        let info = vm.global_extensions_info().unwrap();
        assert_eq!(info.current_byte, Some(1));
        assert_eq!(info.last_write, vm.origin());
        assert_eq!(info.marker_written, Some(true));
        assert!(
            vm.is_owned_global_extensions_marker(
                vm.global,
                &text(EXTENSIONS),
                vm.objects[vm.global.0]
                    .properties
                    .get(&text(EXTENSIONS))
                    .unwrap()
            )
        );
        let after = counts(&vm);
        write(&mut vm, false);
        assert_eq!(counts(&vm), after);
        assert_eq!(version(&vm), None);
        assert!(is_marker(&vm));
        assert_eq!(vm.global_extensions_info().unwrap().current_byte, Some(2));
        assert!(
            vm.objects[vm.global.0]
                .insertion_keys
                .iter()
                .any(|key| key_is(key, VERSION))
        );
    }

    #[test]
    fn global_extensions_version_readonly_overwrites_and_dontdelete_retains() {
        let mut vm = base();
        selected(&mut vm, VERSION, Property::Data(Value::Number(99.0)), 7);
        selected(&mut vm, EXTENSIONS, Property::Data(Value::Null), 3);
        adopt(&mut vm, Some(0));
        write(&mut vm, true);
        assert_eq!(version(&vm), Some(Value::String(text(VERSION_TEXT))));
        assert_eq!(vm.own_flags(vm.global, &text(VERSION)), 7);
        assert_eq!(vm.own_flags(vm.global, &text(EXTENSIONS)), 3);
        write(&mut vm, false);
        assert_eq!(version(&vm), Some(Value::String(text(VERSION_TEXT))));
        assert_eq!(vm.own_flags(vm.global, &text(VERSION)), 7);
        assert_eq!(vm.global_extensions_info().unwrap().current_byte, Some(2));
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(true)
        );
    }

    #[test]
    fn global_extensions_final_readonly_preserves_earlier_true_and_false_effects() {
        let mut vm = base();
        selected(&mut vm, EXTENSIONS, Property::Data(Value::Number(17.0)), 5);
        adopt(&mut vm, None);
        write(&mut vm, true);
        assert_eq!(version(&vm), Some(Value::String(text(VERSION_TEXT))));
        assert_eq!(
            read(&vm),
            GlobalExtensionsLookup::Present(Value::Bool(true))
        );
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(false)
        );
        assert!(matches!(
            vm.objects[vm.global.0].properties.get(&text(EXTENSIONS)),
            Some(Property::Data(Value::Number(17.0)))
        ));
        assert_eq!(vm.own_flags(vm.global, &text(EXTENSIONS)), 5);
        write(&mut vm, false);
        assert_eq!(version(&vm), None);
        assert_eq!(
            read(&vm),
            GlobalExtensionsLookup::Present(Value::Bool(false))
        );
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(false)
        );
        assert!(matches!(
            vm.objects[vm.global.0].properties.get(&text(EXTENSIONS)),
            Some(Property::Data(Value::Number(17.0)))
        ));
    }

    #[test]
    fn global_extensions_binding_requires_idle_explicit_context_and_supported_state() {
        let mut vm = base();
        let profile = GlobalExtensionsProfile {
            context_target: vm.root,
            current_byte: None,
        };
        vm.current_target = Some(vm.root);
        assert!(vm.bind_global_extensions(profile).is_err());
        vm.current_target = None;
        vm.depth = 1;
        assert!(vm.bind_global_extensions(profile).is_err());
        vm.depth = 0;
        vm.saved_active = true;
        assert!(vm.bind_global_extensions(profile).is_err());
        vm.saved_active = false;
        assert!(
            vm.bind_global_extensions(GlobalExtensionsProfile {
                context_target: vm.global,
                ..profile
            })
            .is_err()
        );
        vm.objects[vm.global.0].prototype = Some(vm.root);
        assert!(vm.bind_global_extensions(profile).is_err());
        vm.objects[vm.global.0].prototype = None;
        vm.objects[vm.global.0]
            .insertion_keys
            .push(text("__resolve"));
        assert!(vm.bind_global_extensions(profile).is_err());
        vm.objects[vm.global.0].insertion_keys.pop();
        vm.bind_global_extensions(profile).unwrap();
        assert!(vm.bind_global_extensions(profile).is_err());
    }

    #[test]
    fn global_extensions_context_and_unproved_values_fail_before_any_effects() {
        let mut vm = base();
        adopt(&mut vm, Some(0));
        let before = counts(&vm);
        let info = vm.global_extensions_info();
        for value in [
            Value::Number(f64::from_bits(1)),
            Value::String(text("true")),
            Value::Null,
            Value::Undefined,
        ] {
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &value)
                    .is_err()
            );
        }
        for opcode in [0x1d, 0x3c, 0x41] {
            vm.current = Some((1, opcode));
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                    .is_err()
            );
        }
        vm.current = None;
        vm.current_target = None;
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                .is_err()
        );
        assert!(
            vm.global_extensions_read(vm.global, &text(EXTENSIONS))
                .is_err()
        );
        vm.current_target = Some(vm.global);
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                .is_err()
        );
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.global_extensions_info(), info);
        vm.bind_builtin_context_target(vm.global).unwrap();
        write(&mut vm, true);
        assert_eq!(
            read(&vm),
            GlobalExtensionsLookup::Present(Value::Bool(true))
        );
    }

    #[test]
    fn global_extensions_budget_rejection_is_preflighted_without_partial_native_effects() {
        for limit in 0..2 {
            let mut vm = base();
            adopt(&mut vm, None);
            let before = counts(&vm);
            let info = vm.global_extensions_info();
            if limit == 0 {
                vm.limits.properties = before.1 + 1;
            } else {
                vm.limits.string_units = before.2 + 28;
            }
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                    .is_err()
            );
            assert_eq!(counts(&vm), before);
            assert_eq!(vm.global_extensions_info(), info);
            assert_eq!(version(&vm), None);
            assert!(!is_marker(&vm));
        }
        let mut vm = base();
        adopt(&mut vm, None);
        vm.limits.properties = vm.properties + 1;
        vm.limits.string_units = vm.string_units + 13;
        write(&mut vm, false);
        assert!(is_marker(&vm));
        assert_eq!(version(&vm), None);
    }

    #[test]
    fn global_extensions_readonly_marker_and_version_reinsertion_keep_exact_charges() {
        let mut vm = base();
        selected(
            &mut vm,
            EXTENSIONS,
            Property::NativeMarker(NativeMarker::GlobalExtensions),
            4,
        );
        adopt(&mut vm, None);
        let before = counts(&vm);
        vm.limits.properties = before.1 + 1;
        vm.limits.string_units = before.2 + 16;
        write(&mut vm, true);
        assert_eq!(counts(&vm), (before.0, before.1 + 1, before.2 + 16));
        assert!(is_marker(&vm));
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(false)
        );
        write(&mut vm, false);
        assert_eq!(version(&vm), None);
        let deleted = counts(&vm);
        // A deleted own slot consumes fresh engineering capacity on reinsertion.
        // Its insertion history is deliberately retained, without a refund.
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                .is_err()
        );
        assert_eq!(counts(&vm), deleted);
        assert_eq!(vm.global_extensions_info().unwrap().current_byte, Some(2));
        vm.limits.properties += 1;
        vm.limits.string_units += 16;
        write(&mut vm, true);
        assert_eq!(counts(&vm), (deleted.0, deleted.1 + 1, deleted.2 + 16));
        assert_eq!(
            vm.objects[vm.global.0]
                .insertion_keys
                .iter()
                .filter(|key| key_is(key, VERSION))
                .count(),
            2
        );
    }

    #[test]
    fn global_extensions_unsupported_selected_descriptors_reject_before_byte_change() {
        for (name, property) in [
            (
                EXTENSIONS,
                Property::Accessor {
                    getter: Value::Undefined,
                    setter: None,
                },
            ),
            (VERSION, Property::Data(Value::Object(ObjectId(0)))),
            (
                EXTENSIONS,
                Property::NativeMarker(NativeMarker::ExternalAvailable),
            ),
            (
                VERSION,
                Property::NativeMarker(NativeMarker::GlobalExtensions),
            ),
        ] {
            let mut vm = base();
            adopt(&mut vm, Some(0));
            selected(&mut vm, name, property, 0);
            let before = counts(&vm);
            let info = vm.global_extensions_info();
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Bool(true))
                    .is_err()
            );
            assert_eq!(counts(&vm), before);
            assert_eq!(vm.global_extensions_info(), info);
        }
    }
    // These are supplied-value state checks, not execution of the original
    // Integer1 instruction, a native floating-point run, or source tag parity.
    #[test]
    fn global_extensions_numeric_matrix_preserves_input_and_sets_logical_state() {
        let cases = [
            (0.0, false),
            (-0.0, false),
            (f64::NAN, false),
            (f64::from_bits(0x7ff0_0000_0000_0001), false),
            (f64::from_bits(0xfff8_0000_0000_cafe), false),
            (1.0, true),
            (-1.0, true),
            (f64::from(i32::MIN), true),
            (f64::from(i32::MAX), true),
            (f64::MIN_POSITIVE, true),
            (-f64::MIN_POSITIVE, true),
            (f64::MAX, true),
            (-f64::MAX, true),
            (f64::INFINITY, true),
            (f64::NEG_INFINITY, true),
        ];
        for (number, enabled) in cases {
            let mut vm = base();
            adopt(&mut vm, None);
            vm.current_code = Some(CodeId(0));
            vm.current = Some((13566, 0x4f));
            let value = Value::Number(number);
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &value)
                    .unwrap()
            );
            assert_eq!(
                vm.global_extensions_info().unwrap().current_byte,
                Some(if enabled { 1 } else { 2 })
            );
            assert_eq!(
                read(&vm),
                GlobalExtensionsLookup::Present(Value::Bool(enabled))
            );
            assert_eq!(vm.global_extensions_info().unwrap().last_write, vm.origin());
            assert_eq!(
                vm.global_extensions_info().unwrap().marker_written,
                Some(true)
            );
            assert!(is_marker(&vm));
            assert_eq!(
                version(&vm),
                enabled.then(|| Value::String(text(VERSION_TEXT)))
            );
            let Value::Number(after) = value else {
                unreachable!()
            };
            assert_eq!(after.to_bits(), number.to_bits());
        }
    }

    #[test]
    fn global_extensions_numeric_false_and_true_choose_exact_budgeted_effects() {
        for number in [0.0, -0.0, f64::NAN] {
            let mut vm = base();
            adopt(&mut vm, None);
            let before = counts(&vm);
            vm.limits.properties = before.1 + 1;
            vm.limits.string_units = before.2 + 13;
            assert!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Number(number))
                    .unwrap()
            );
            assert_eq!(counts(&vm), (before.0, before.1 + 1, before.2 + 13));
            assert_eq!(version(&vm), None);
            assert_eq!(vm.global_extensions_info().unwrap().current_byte, Some(2));
        }
        let mut vm = base();
        adopt(&mut vm, None);
        let before = counts(&vm);
        let info = vm.global_extensions_info();
        vm.limits.properties = before.1 + 1;
        vm.limits.string_units = before.2 + 28;
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Number(1.0))
                .is_err()
        );
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.global_extensions_info(), info);
        assert_eq!(version(&vm), None);
        assert!(!is_marker(&vm));

        let mut vm = base();
        selected(&mut vm, EXTENSIONS, Property::Data(Value::Null), 4);
        selected(&mut vm, VERSION, Property::Data(Value::Number(17.0)), 4);
        adopt(&mut vm, None);
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Number(1.0))
                .unwrap()
        );
        assert_eq!(version(&vm), Some(Value::String(text(VERSION_TEXT))));
        assert_eq!(vm.own_flags(vm.global, &text(VERSION)), 4);
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(false)
        );
        assert!(
            vm.global_extensions_write(vm.global, &text(EXTENSIONS), &Value::Number(f64::NAN))
                .unwrap()
        );
        assert_eq!(version(&vm), None); // ReadOnly is not DontDelete.
        assert_eq!(vm.global_extensions_info().unwrap().current_byte, Some(2));
        assert_eq!(
            vm.global_extensions_info().unwrap().marker_written,
            Some(false)
        );
        assert!(matches!(
            vm.objects[vm.global.0].properties.get(&text(EXTENSIONS)),
            Some(Property::Data(Value::Null))
        ));
    }

    #[test]
    fn global_extensions_subnormal_and_unproved_categories_fail_without_effects() {
        let mut vm = base();
        adopt(&mut vm, Some(0));
        let before = counts(&vm);
        let info = vm.global_extensions_info();
        for value in [
            Value::Number(f64::from_bits(1)),
            Value::Number(-f64::from_bits(1)),
            Value::Number(f64::from_bits(0x000f_ffff_ffff_ffff)),
            Value::String(text("1")),
            Value::String(text("")),
            Value::Object(vm.global),
            Value::Null,
            Value::Undefined,
        ] {
            assert!(matches!(
                vm.global_extensions_write(vm.global, &text(EXTENSIONS), &value)
                    .unwrap_err()
                    .kind,
                ErrorKind::Unsupported(_)
            ));
            assert_eq!(counts(&vm), before);
            assert_eq!(vm.global_extensions_info(), info);
            assert_eq!(version(&vm), None);
            assert!(!is_marker(&vm));
        }
    }
}
