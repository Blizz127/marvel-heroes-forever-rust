//! Read-only subset of fresh source MovieClip prototype selection.
//!
//! Native contract: `docs/FRESH_MOVIECLIP_PROTOTYPE.md`, selector 0x14019b570
//! and conversion 0x140199640. Private evidence receipt:
//! b50ac7935fd9cc0f9ee272494d874e1840b75c64419816256fc5049fb9fc29bc.
//! The cache must already exist: this module never resolves a builtin, invokes
//! an accessor, creates storage or applies the later attached prototype setter.
//! This is a controlled current-state selector, not a recovered startup default.

use super::{ErrorKind, Kind, ObjectId, Property, Result, Value, Vm};

const MOVIE_CLIP: &[u16] = &[77, 111, 118, 105, 101, 67, 108, 105, 112];
const PROTOTYPE: &[u16] = &[112, 114, 111, 116, 111, 116, 121, 112, 101];
const RESOLVE: &[u16] = &[95, 95, 114, 101, 115, 111, 108, 118, 101];

/// Explicit association with this VM's single builtin context. The allocation
/// caller separately validates source, binding, parent and target association.
/// This does not certify another VM's object with an equal numeric ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialMovieClipPrototypeProfile {
    pub context_target: ObjectId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialMovieClipPrototypeReason {
    CachedGlobalMissing,
    CachedGlobalNonObject,
    CachedPrototypeMissing,
    CurrentPrototypeObject,
    CurrentPrototypeNull,
}

/// Diagnostic result produced by the VM during synchronous allocation.
/// Public getters do not make this a reusable allocation certificate: no public
/// API accepts it to install a prototype or validate a later object ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialMovieClipPrototypeSelection {
    pub(crate) fallback: Option<ObjectId>,
    pub(crate) selected: Option<ObjectId>,
    pub(crate) reason: InitialMovieClipPrototypeReason,
    pub(crate) context_target: ObjectId,
}

impl InitialMovieClipPrototypeSelection {
    pub const fn fallback(&self) -> Option<ObjectId> {
        self.fallback
    }
    pub const fn selected(&self) -> Option<ObjectId> {
        self.selected
    }
    pub const fn reason(&self) -> InitialMovieClipPrototypeReason {
        self.reason
    }
    pub const fn context_target(&self) -> ObjectId {
        self.context_target
    }
}

impl Vm {
    /// Select immediately before fresh allocation's direct initial store.
    /// The private cache is authoritative even if current C.prototype changed.
    /// None in today's cache field cannot distinguish uncached from cached-null,
    /// so that state remains unsupported instead of inventing a native result.
    pub(super) fn select_initial_movie_clip_prototype(
        &self,
        profile: InitialMovieClipPrototypeProfile,
    ) -> Result<InitialMovieClipPrototypeSelection> {
        if !self.is_bound_builtin_context_target(profile.context_target)? {
            return self.fail(ErrorKind::Unsupported(
                "initial MovieClip prototype requires bound builtin context",
            ));
        }
        // Native fallback acquisition precedes BOTH current raw reads. Requiring
        // an already available cache closes its possible factory/reentry effects.
        let Some(fallback) = self.movie_clip_prototype else {
            return self.fail(ErrorKind::Unsupported(
                "initial MovieClip prototype requires available builtin cache",
            ));
        };
        self.check_object(fallback)?;
        let mut visits = self.limits.member_visits.saturating_sub(self.member_visits);
        let result = |selected, reason| InitialMovieClipPrototypeSelection {
            fallback: Some(fallback),
            selected,
            reason,
            context_target: profile.context_target,
        };
        let Some(global_value) =
            self.initial_prototype_raw_data(self.global, MOVIE_CLIP, &mut visits)?
        else {
            return Ok(result(
                Some(fallback),
                InitialMovieClipPrototypeReason::CachedGlobalMissing,
            ));
        };
        let Some(holder) = self.initial_prototype_object(global_value)? else {
            return Ok(result(
                Some(fallback),
                InitialMovieClipPrototypeReason::CachedGlobalNonObject,
            ));
        };
        let Some(prototype) = self.initial_prototype_raw_data(holder, PROTOTYPE, &mut visits)?
        else {
            return Ok(result(
                Some(fallback),
                InitialMovieClipPrototypeReason::CachedPrototypeMissing,
            ));
        };
        let selected = self.initial_prototype_object(prototype)?;
        Ok(result(
            selected,
            if selected.is_some() {
                InitialMovieClipPrototypeReason::CurrentPrototypeObject
            } else {
                InitialMovieClipPrototypeReason::CurrentPrototypeNull
            },
        ))
    }

    /// The two fixed keys take ordinary raw storage routes on Plain objects and
    /// generic callable objects. Presence is preserved; no language getter runs.
    /// Missing through native graphs is deliberately narrower than own Data.
    fn initial_prototype_raw_data<'a>(
        &'a self,
        mut id: ObjectId,
        key: &[u16],
        visits: &mut usize,
    ) -> Result<Option<&'a Value>> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if !matches!(
                object.kind,
                Kind::Plain | Kind::Script(_) | Kind::Native(_) | Kind::MovieClipConstructor
            ) {
                return self.fail(ErrorKind::Unsupported(
                    "initial MovieClip prototype raw native getter profile",
                ));
            }
            let count = object.insertion_keys.len().saturating_add(1);
            if count > *visits {
                return self.fail(ErrorKind::Budget("initial prototype member visits"));
            }
            *visits -= count;
            if object.properties.contains_key(RESOLVE)
                || object
                    .insertion_keys
                    .iter()
                    .any(|name| name.as_ref() == RESOLVE)
            {
                return self.fail(ErrorKind::Unsupported(
                    "initial MovieClip prototype resolver history",
                ));
            }
            if let Some(property) = object.properties.get(key) {
                return match property {
                    Property::Data(value) => Ok(Some(value)),
                    Property::Accessor { .. } => self.fail(ErrorKind::Unsupported(
                        "initial MovieClip prototype accessor conversion",
                    )),
                    Property::NativeMarker(_) => self.fail(ErrorKind::Unsupported(
                        "initial MovieClip prototype native marker conversion",
                    )),
                };
            }
            // A partial native object is not a negative membership witness.
            // Even Object.prototype's Plain representation needs separate
            // complete inventory/admission proof before an absence can be used.
            if !matches!(object.kind, Kind::Plain)
                || Some(id) == self.object_prototype
                || self.is_geom_namespace(id)
            {
                return self.fail(ErrorKind::Unsupported(
                    "initial MovieClip prototype incomplete native absence",
                ));
            }
            match object.prototype {
                Some(parent) => id = parent,
                None => return Ok(None),
            }
        }
        self.fail(ErrorKind::Budget("initial prototype lookup depth"))
    }

    /// Native tag6/tag8 return their object identity without boxing or invocation.
    /// Display references (tag7), ordinary scalars, Null and Undefined convert to
    /// null here; they are not member_owner's display-interface conversion.
    fn initial_prototype_object(&self, value: &Value) -> Result<Option<ObjectId>> {
        match value {
            Value::Object(id) => {
                self.check_value(value)?;
                if matches!(
                    self.objects[id.0].kind,
                    Kind::ConstructorSuper { .. } | Kind::AttachedSuper { .. }
                ) {
                    return self.fail(ErrorKind::Unsupported(
                        "initial MovieClip prototype opaque super conversion",
                    ));
                }
                Ok(Some(*id))
            }
            Value::DeferredBuiltin(_) => self.fail(ErrorKind::Unsupported(
                "initial MovieClip prototype deferred factory conversion",
            )),
            Value::Undefined
            | Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::DisplayClip(_)
            | Value::DisplayTextField(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Limits, NativeMarker, text};
    use super::*;

    fn setup() -> (Vm, InitialMovieClipPrototypeProfile, ObjectId, ObjectId) {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        let constructor = vm.install_movie_clip_constructor().unwrap();
        let fallback = vm.movie_clip_prototype.unwrap();
        let context_target = vm.root;
        vm.bind_builtin_context_target(context_target).unwrap();
        (
            vm,
            InitialMovieClipPrototypeProfile { context_target },
            constructor,
            fallback,
        )
    }

    fn put(vm: &mut Vm, object: ObjectId, name: &str, value: Value) {
        vm.raw_member(object, text(name), Property::Data(value), 0)
            .unwrap();
    }

    fn select(
        vm: &Vm,
        profile: InitialMovieClipPrototypeProfile,
    ) -> InitialMovieClipPrototypeSelection {
        vm.select_initial_movie_clip_prototype(profile).unwrap()
    }

    #[test]
    fn decision_table_preserves_missing_and_present_null_conversion() {
        let (mut vm, profile, constructor, fallback) = setup();
        let original = select(&vm, profile);
        assert_eq!(original.selected(), Some(fallback));
        assert_eq!(
            original.reason(),
            InitialMovieClipPrototypeReason::CurrentPrototypeObject
        );
        assert_eq!(original.context_target(), profile.context_target);

        vm.objects[vm.global.0].properties.remove(MOVIE_CLIP);
        assert_eq!(
            select(&vm, profile).reason(),
            InitialMovieClipPrototypeReason::CachedGlobalMissing
        );
        for value in [
            Value::Undefined,
            Value::Null,
            Value::Bool(false),
            Value::Number(4.0),
            Value::string("not an object"),
        ] {
            let global = vm.global;
            put(&mut vm, global, "MovieClip", value.clone());
            let result = select(&vm, profile);
            assert_eq!(result.selected(), Some(fallback));
            assert_eq!(
                result.reason(),
                InitialMovieClipPrototypeReason::CachedGlobalNonObject
            );

            put(&mut vm, constructor, "prototype", value);
            put(&mut vm, global, "MovieClip", Value::Object(constructor));
            let result = select(&vm, profile);
            assert_eq!(result.fallback(), Some(fallback));
            assert_eq!(result.selected(), None);
            assert_eq!(
                result.reason(),
                InitialMovieClipPrototypeReason::CurrentPrototypeNull
            );
        }
        let holder = vm.object().unwrap();
        let global = vm.global;
        put(&mut vm, global, "MovieClip", Value::Object(holder));
        assert_eq!(
            select(&vm, profile).reason(),
            InitialMovieClipPrototypeReason::CachedPrototypeMissing
        );
        let selected = vm.native(31).unwrap();
        put(&mut vm, holder, "prototype", Value::Object(selected));
        let result = select(&vm, profile);
        assert_eq!(result.selected(), Some(selected));
        assert_eq!(result.fallback(), Some(fallback));
        assert_eq!(
            result.reason(),
            InitialMovieClipPrototypeReason::CurrentPrototypeObject
        );
    }

    #[test]
    fn modeled_plain_chain_and_own_undefined_shadowing() {
        let (mut vm, profile, _, fallback) = setup();
        let ancestor = vm.object().unwrap();
        let holder = vm.alloc(Kind::Plain, Some(ancestor)).unwrap();
        let selected = vm.object().unwrap();
        put(&mut vm, ancestor, "prototype", Value::Object(selected));
        let global = vm.global;
        put(&mut vm, global, "MovieClip", Value::Object(holder));
        assert_eq!(select(&vm, profile).selected(), Some(selected));
        put(&mut vm, holder, "prototype", Value::Undefined);
        assert_eq!(select(&vm, profile).selected(), None);
        vm.objects[holder.0].properties.remove(PROTOTYPE);
        vm.objects[ancestor.0].properties.remove(PROTOTYPE);
        assert_eq!(select(&vm, profile).selected(), Some(fallback));
        assert_eq!(
            select(&vm, profile).reason(),
            InitialMovieClipPrototypeReason::CachedPrototypeMissing
        );
    }

    #[test]
    fn unknown_native_ancestors_are_not_missing_but_own_data_wins() {
        let (mut vm, profile, constructor, _) = setup();
        vm.objects[constructor.0].properties.remove(PROTOTYPE);
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        let parent = vm.object_prototype.unwrap();
        let holder = vm.alloc(Kind::Plain, Some(parent)).unwrap();
        let global = vm.global;
        put(&mut vm, global, "MovieClip", Value::Object(holder));
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        put(&mut vm, holder, "prototype", Value::Null);
        assert_eq!(select(&vm, profile).selected(), None);
        vm.objects[holder.0].properties.remove(PROTOTYPE);
        put(&mut vm, parent, "prototype", Value::Undefined);
        assert_eq!(select(&vm, profile).selected(), None);
    }

    #[test]
    fn selected_accessors_markers_and_deferred_values_do_not_execute() {
        let (mut vm, profile, constructor, _) = setup();
        for descriptor in [
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            Property::NativeMarker(NativeMarker::ExternalAvailable),
        ] {
            vm.raw_member(constructor, text("prototype"), descriptor, 0)
                .unwrap();
            assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        }
        let namespace = vm.object().unwrap();
        let token = vm
            .register_deferred_builtin(
                namespace,
                "NeverRun",
                super::super::BuiltinFactory::Unsupported,
                0,
            )
            .unwrap();
        put(&mut vm, constructor, "prototype", token);
        let before = (
            vm.objects.len(),
            vm.properties,
            vm.string_units,
            vm.host_calls,
            vm.instructions,
            vm.member_visits,
        );
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        assert_eq!(
            before,
            (
                vm.objects.len(),
                vm.properties,
                vm.string_units,
                vm.host_calls,
                vm.instructions,
                vm.member_visits
            )
        );
    }

    #[test]
    fn cache_precedes_raw_reads_and_requires_explicit_context() {
        let (mut vm, profile, _, _) = setup();
        let other = vm.object().unwrap();
        assert!(
            vm.select_initial_movie_clip_prototype(InitialMovieClipPrototypeProfile {
                context_target: other
            })
            .is_err()
        );
        vm.movie_clip_prototype = None;
        vm.raw_member(
            vm.global,
            text("MovieClip"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        let error = vm.select_initial_movie_clip_prototype(profile).unwrap_err();
        assert!(matches!(
            error.kind,
            ErrorKind::Unsupported("initial MovieClip prototype requires available builtin cache")
        ));
    }

    #[test]
    fn resolver_history_cycles_and_visit_budget_are_explicit_gates() {
        let (mut vm, profile, constructor, fallback) = setup();
        let holder = vm.object().unwrap();
        let global = vm.global;
        put(&mut vm, global, "MovieClip", Value::Object(holder));
        put(&mut vm, holder, "__resolve", Value::Undefined);
        vm.objects[holder.0].properties.remove(RESOLVE);
        put(&mut vm, holder, "prototype", Value::Object(fallback));
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        let cycle = vm.object().unwrap();
        vm.objects[cycle.0].prototype = Some(cycle);
        put(&mut vm, global, "MovieClip", Value::Object(cycle));
        assert!(matches!(
            vm.select_initial_movie_clip_prototype(profile)
                .unwrap_err()
                .kind,
            ErrorKind::Budget(_)
        ));
        put(&mut vm, global, "MovieClip", Value::Object(constructor));
        vm.limits.member_visits = 0;
        assert!(matches!(
            vm.select_initial_movie_clip_prototype(profile)
                .unwrap_err()
                .kind,
            ErrorKind::Budget("initial prototype member visits")
        ));
    }

    #[test]
    fn unrelated_properties_and_unvisited_ancestors_do_not_run() {
        let (mut vm, profile, _, fallback) = setup();
        let holder = vm.object().unwrap();
        vm.objects[holder.0].prototype = Some(holder);
        put(&mut vm, holder, "prototype", Value::Undefined);
        vm.raw_member(
            holder,
            text("unrelated"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        let ancestor = vm.object().unwrap();
        put(&mut vm, ancestor, "MovieClip", Value::Object(holder));
        vm.objects[vm.global.0].properties.remove(MOVIE_CLIP);
        vm.objects[vm.global.0].prototype = Some(ancestor);
        assert_eq!(select(&vm, profile).selected(), None);
        let global = vm.global;
        put(&mut vm, global, "MovieClip", Value::Undefined);
        assert_eq!(select(&vm, profile).selected(), Some(fallback));
    }

    #[test]
    fn direct_conversion_preserves_tag_boundaries_and_validates_object_ids() {
        let (mut vm, profile, constructor, _) = setup();
        // Native tag7 is not generic object conversion; no display-owner lookup
        // occurs, so this controlled handle is not dereferenced or adopted.
        put(
            &mut vm,
            constructor,
            "prototype",
            Value::DisplayClip(super::super::AttachedClipId(usize::MAX)),
        );
        assert_eq!(select(&vm, profile).selected(), None);
        put(
            &mut vm,
            constructor,
            "prototype",
            Value::Object(ObjectId(usize::MAX)),
        );
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        let disguised = vm
            .alloc(Kind::AttachedOwner(super::super::AttachedClipId(0)), None)
            .unwrap();
        put(&mut vm, constructor, "prototype", Value::Object(disguised));
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
        let opaque = vm
            .alloc(
                Kind::ConstructorSuper {
                    constructor,
                    receiver: constructor,
                },
                None,
            )
            .unwrap();
        put(&mut vm, constructor, "prototype", Value::Object(opaque));
        assert!(vm.select_initial_movie_clip_prototype(profile).is_err());
    }
}
