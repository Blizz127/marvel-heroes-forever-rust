//! Read the current dynamic event member after a completed saved-vector phase.
//!
//! The receiver, caller Environment and explicit builtin-context association
//! remain separate from any Environment a future function call might select.
//! This module never invokes a function, getter or deferred builtin factory.

use super::saved_dispatch::{self, Progress};
use crate::avm1::vm::{
    self, AttachedClipId, AttachedClipSource, AttachedEventKind, AttachedMember, DisplayTargetId,
    LoadedMovieSourceId, ObjectId, SavedEnvironmentId, Value, ValueCallability, Vm,
};
use std::fmt;

/// An explicit caller assertion, not a recovered native default or VM token.
#[derive(Debug, Clone, Copy)]
pub struct Profile {
    /// The supplied outcome is the successful saved phase for this delivery in
    /// this VM. There has been no intervening event/script execution. Public
    /// arena IDs and the Copy outcome alone cannot prove this association.
    pub same_completed_phase: bool,
}

/// Actual live identities, with the source and caller Environment kept apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub owner: AttachedClipId,
    pub owner_object: ObjectId,
    pub display_target: DisplayTargetId,
    pub definition_source: AttachedClipSource,
    pub environment_movie: LoadedMovieSourceId,
    pub environment_header_version: u8,
    pub caller_environment: SavedEnvironmentId,
    pub event: AttachedEventKind,
    pub member_name: &'static str,
    pub saved_progress: Progress,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Missing,
    /// Includes Present Undefined, which must not fall through to a prototype.
    PresentNonCallable(Value),
    /// A function-valued VM object, preserving its exact identity. This does
    /// not promise support for its invocation, captures or selected Environment.
    CallablePending(Value),
    /// A retained native tag-11 name. No factory/cache is consulted and no
    /// namespace is changed; event-path conversion remains unsupported.
    DeferredConversionPending(Value),
}

/// A lookup result only. No variant represents complete event delivery.
#[derive(Debug, Clone, PartialEq)]
pub struct Lookup {
    pub context: Context,
    pub member: Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Admission,
    Receiver,
    Environment,
    BuiltinContext,
    MemberLookup,
    Classification,
}

#[derive(Debug)]
pub enum FailureKind {
    Vm(vm::Error),
    Admission(&'static str),
}

/// This read-only lookup has no new partial VM effects. The incoming saved
/// phase can already have wrapper/body effects; they remain in place on error.
/// Retaining that phase is diagnostic, never permission to replay it.
#[derive(Debug)]
pub struct Failure {
    pub phase: saved_dispatch::Outcome,
    pub stage: Stage,
    pub kind: FailureKind,
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "post-saved dynamic lookup at {:?}: ", self.stage)?;
        match &self.kind {
            FailureKind::Vm(error) => error.fmt(f),
            FailureKind::Admission(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            FailureKind::Vm(error) => Some(error),
            FailureKind::Admission(_) => None,
        }
    }
}

/// Look up `onLoad` or `onConstruct` in the current attached receiver.
///
/// The existing attached getter is authoritative for native-name admission,
/// own/child/prototype precedence, retained identities and lookup budgets.
/// Unsupported accessors/resolvers remain errors, not Missing. No cached value
/// from before the saved phase is used. This function takes neither mutable VM
/// access nor a Host; stack/register state and prior saved effects are retained.
/// It does not repeat wrapper eligibility, schedule work, call a member, select
/// a callable Environment or infer the unresolved native method-name hint.
pub fn lookup_after_saved(
    vm: &Vm,
    phase: saved_dispatch::Outcome,
    profile: Profile,
) -> Result<Lookup, Failure> {
    let fail = |stage, kind| Failure { phase, stage, kind };
    let admission = |stage, message| fail(stage, FailureKind::Admission(message));
    if !profile.same_completed_phase {
        return Err(admission(
            Stage::Admission,
            "same completed saved phase is unbound",
        ));
    }
    let saved_dispatch::Outcome::DynamicLookupPending {
        owner,
        environment,
        event,
        progress,
    } = phase
    else {
        return Err(admission(
            Stage::Admission,
            "wrapper skipped dynamic lookup",
        ));
    };
    let info = vm
        .attached_info(owner)
        .map_err(|error| fail(Stage::Receiver, FailureKind::Vm(error)))?;
    let owner_object = vm
        .attached_owner(owner)
        .map_err(|error| fail(Stage::Receiver, FailureKind::Vm(error)))?;
    let state = vm
        .saved_environment_state(environment)
        .map_err(|error| fail(Stage::Environment, FailureKind::Vm(error)))?;
    if state.owner() != owner || state.target() != owner_object {
        return Err(admission(
            Stage::Environment,
            "saved phase receiver and Environment differ",
        ));
    }
    if vm
        .saved_environment_halted(environment)
        .map_err(|error| fail(Stage::Environment, FailureKind::Vm(error)))?
    {
        return Err(admission(Stage::Environment, "saved Environment is halted"));
    }
    if !vm
        .is_bound_builtin_context_target(owner_object)
        .map_err(|error| fail(Stage::BuiltinContext, FailureKind::Vm(error)))?
    {
        return Err(admission(
            Stage::BuiltinContext,
            "receiver builtin context is unbound",
        ));
    }
    let member_name = match event {
        AttachedEventKind::Load => "onLoad",
        AttachedEventKind::Construct => "onConstruct",
    };
    let member = match vm
        .read_attached_member(owner, member_name)
        .map_err(|error| fail(Stage::MemberLookup, FailureKind::Vm(error)))?
    {
        AttachedMember::Missing => Member::Missing,
        AttachedMember::Present(value) => match vm
            .classify_callability(&value)
            .map_err(|error| fail(Stage::Classification, FailureKind::Vm(error)))?
        {
            ValueCallability::NonCallable => Member::PresentNonCallable(value),
            ValueCallability::Callable => Member::CallablePending(value),
            ValueCallability::DeferredBuiltin => Member::DeferredConversionPending(value),
        },
    };
    Ok(Lookup {
        context: Context {
            owner,
            owner_object,
            display_target: info.target,
            definition_source: info.source,
            environment_movie: info.environment_movie,
            environment_header_version: info.environment_header_version,
            caller_environment: environment,
            event,
            member_name,
            saved_progress: progress,
        },
        member,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::{
        Program,
        vm::{
            AttachedChild, AttachedClipSpec, AttachedEventState, AttachedMovieBinding,
            BuiltinFactory, ExpectedProfile, NativeNameProfile, RejectHost, SavedActionId,
            SavedActionSource, SavedEnvironmentState, text,
        },
    };
    use crate::frontend::saved_handlers::SavedHandlers;
    use std::sync::Arc;

    const PROFILE: Profile = Profile {
        same_completed_phase: true,
    };

    fn names() -> Arc<NativeNameProfile> {
        // Entirely invented inventory; no private native table enters source.
        let mut data = format!(
            "MHF-ATTACHED-NAMES\t1\nbuild\t1.52.0.1700\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
            "ab".repeat(32)
        );
        for id in 0..117 {
            let alias = id <= 21 || (110..=115).contains(&id);
            data.push_str(&format!(
                "standard\t{id}\t{}\t{}native{id:03}\n",
                u8::from(alias),
                if alias { "_" } else { "" },
            ));
        }
        for (role, count) in [("movieclip", 44), ("object", 9)] {
            let mut rows = vec!["constructor".to_owned()];
            if role == "movieclip" {
                rows.push("__proto__".to_owned());
            }
            rows.extend((rows.len()..count).map(|i| format!("{role}{i:03}")));
            rows.sort();
            for name in rows {
                data.push_str(&format!("{role}\t{name}\n"));
            }
        }
        Arc::new(
            NativeNameProfile::parse_verified(
                data.as_bytes(),
                &ExpectedProfile {
                    build: "1.52.0.1700",
                    source_sha256: [0xab; 32],
                    inventory_sha256: [0x51; 32],
                },
                |_| Ok([0x51; 32]),
            )
            .unwrap(),
        )
    }

    fn setup(with_names: bool, bind_context: bool) -> (Vm, AttachedClipId) {
        let mut vm = Vm::new(15, Default::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        if with_names {
            vm.install_attached_name_profile(names()).unwrap();
        }
        let owner = vm
            .create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(73),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(5),
                    sprite_id: 0,
                    header_version: 15,
                },
                parent: None,
                event_state: AttachedEventState {
                    target_flags: 0x5200_01a0,
                    load_flags: 8,
                    source_flags: 0,
                },
            })
            .unwrap();
        vm.adopt_attached_root(owner).unwrap();
        if bind_context {
            vm.bind_builtin_context_target(vm.attached_owner(owner).unwrap())
                .unwrap();
        }
        (vm, owner)
    }

    fn environment(vm: &mut Vm, owner: AttachedClipId) -> SavedEnvironmentId {
        let state = SavedEnvironmentState::from_supplied(
            owner,
            vm.attached_owner(owner).unwrap(),
            vec![Value::Number(17.0)],
            [
                Value::Null,
                Value::Bool(true),
                Value::Number(4.0),
                Value::Undefined,
            ],
            32,
        )
        .unwrap();
        vm.register_saved_environment(state).unwrap()
    }

    fn phase(
        vm: &mut Vm,
        owner: AttachedClipId,
        env: SavedEnvironmentId,
        event: AttachedEventKind,
    ) -> saved_dispatch::Outcome {
        saved_dispatch::execute_saved_phase(
            vm,
            &SavedHandlers::new(owner, Default::default()),
            event,
            env,
            &mut RejectHost,
        )
        .unwrap()
    }

    fn lookup(vm: &mut Vm, owner: AttachedClipId, env: SavedEnvironmentId) -> Lookup {
        let pending = phase(vm, owner, env, AttachedEventKind::Construct);
        lookup_after_saved(vm, pending, PROFILE).unwrap()
    }

    #[test]
    fn missing_is_distinct_and_identity_state_and_progress_survive_both_events() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let env = environment(&mut vm, owner);
        for (event, name) in [
            (AttachedEventKind::Load, "onLoad"),
            (AttachedEventKind::Construct, "onConstruct"),
        ] {
            let pending = phase(&mut vm, owner, env, event);
            let before = vm.attached_event_state(owner).unwrap();
            let count = vm.object_count();
            let result = lookup_after_saved(&vm, pending, PROFILE).unwrap();
            assert_eq!(result.member, Member::Missing);
            assert_eq!(result.context.member_name, name);
            assert_eq!(result.context.owner, owner);
            assert_eq!(
                result.context.owner_object,
                vm.attached_owner(owner).unwrap()
            );
            assert_eq!(result.context.display_target, DisplayTargetId(73));
            assert_eq!(
                result.context.definition_source.movie,
                LoadedMovieSourceId(5)
            );
            assert_eq!(result.context.environment_movie, LoadedMovieSourceId(5));
            assert_eq!(result.context.environment_header_version, 15);
            assert_eq!(result.context.caller_environment, env);
            assert_eq!(result.context.saved_progress, Progress::default());
            assert_eq!(vm.attached_event_state(owner).unwrap(), before);
            assert_eq!(vm.object_count(), count);
            assert_eq!(
                vm.saved_environment_state(env).unwrap().stack(),
                [Value::Number(17.0)]
            );
            assert_eq!(
                vm.saved_environment_state(env).unwrap().registers(),
                &[
                    Value::Null,
                    Value::Bool(true),
                    Value::Number(4.0),
                    Value::Undefined
                ]
            );
        }
    }

    #[test]
    fn present_undefined_shadows_deeper_callable_and_objects_are_not_truthiness_tests() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let deep = vm.object().unwrap();
        let near = vm.object().unwrap();
        let callback = Value::Object(vm.native(999).unwrap());
        vm.set(deep, "onConstruct", callback.clone()).unwrap();
        vm.set(near, "__proto__", Value::Object(deep)).unwrap();
        vm.set(near, "onConstruct", Value::Undefined).unwrap();
        vm.set_attached_prototype(owner, Some(near)).unwrap();
        let env = environment(&mut vm, owner);
        assert_eq!(
            lookup(&mut vm, owner, env).member,
            Member::PresentNonCallable(Value::Undefined)
        );
        let plain = Value::Object(vm.object().unwrap());
        vm.set(near, "onConstruct", plain.clone()).unwrap();
        assert_eq!(
            lookup(&mut vm, owner, env).member,
            Member::PresentNonCallable(plain)
        );
        vm.set(near, "onConstruct", callback.clone()).unwrap();
        assert_eq!(
            lookup(&mut vm, owner, env).member,
            Member::CallablePending(callback)
        );
    }

    #[test]
    fn named_child_wins_over_callable_prototype_and_unsupported_child_stops() {
        for supported in [true, false] {
            let (mut vm, owner) = setup(true, true);
            let prototype = vm.object().unwrap();
            let callback = vm.native(999).unwrap();
            vm.set(prototype, "onConstruct", Value::Object(callback))
                .unwrap();
            vm.set_attached_prototype(owner, Some(prototype)).unwrap();
            let child = if supported {
                let child = vm
                    .create_attached_movie_clip_with_binding(
                        AttachedClipSpec {
                            target: DisplayTargetId(74),
                            source: AttachedClipSource {
                                movie: LoadedMovieSourceId(6),
                                sprite_id: 11,
                                header_version: 11,
                            },
                            parent: Some(owner),
                            event_state: AttachedEventState {
                                target_flags: 0x5200_0180,
                                load_flags: 0,
                                source_flags: 0,
                            },
                        },
                        AttachedMovieBinding::LocalSourceOrImportedLocalExport,
                    )
                    .unwrap();
                AttachedChild::Clip {
                    name: text("onConstruct"),
                    clip: child,
                }
            } else {
                AttachedChild::Unsupported {
                    name: text("onConstruct"),
                    target: DisplayTargetId(74),
                }
            };
            let expected = match &child {
                AttachedChild::Clip { clip, .. } => Some(Value::DisplayClip(*clip)),
                _ => None,
            };
            vm.set_attached_children(owner, vec![child]).unwrap();
            let env = environment(&mut vm, owner);
            let pending = phase(&mut vm, owner, env, AttachedEventKind::Construct);
            let count = vm.object_count();
            let result = lookup_after_saved(&vm, pending, PROFILE);
            if let Some(value) = expected {
                assert_eq!(result.unwrap().member, Member::PresentNonCallable(value));
            } else {
                let failure = result.unwrap_err();
                assert_eq!(failure.stage, Stage::MemberLookup);
                assert_eq!(failure.phase, pending);
                assert!(matches!(
                    failure.kind,
                    FailureKind::Vm(vm::Error {
                        kind: vm::ErrorKind::Unsupported("attached named child receiver category"),
                        ..
                    })
                ));
            }
            assert_eq!(vm.object_count(), count);
            assert!(!vm.saved_environment_halted(env).unwrap());
        }
    }

    #[test]
    fn deferred_builtin_preserves_token_without_factory_or_publication() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let namespace = vm.object().unwrap();
        let token = vm
            .register_deferred_builtin(namespace, "ExampleFactory", BuiltinFactory::Unsupported, 0)
            .unwrap();
        let prototype = vm.object().unwrap();
        vm.set(prototype, "onConstruct", token.clone()).unwrap();
        vm.set_attached_prototype(owner, Some(prototype)).unwrap();
        let env = environment(&mut vm, owner);
        let count = vm.object_count();
        assert_eq!(
            lookup(&mut vm, owner, env).member,
            Member::DeferredConversionPending(token.clone())
        );
        assert_eq!(vm.get(namespace, "ExampleFactory").unwrap(), token);
        assert_eq!(vm.object_count(), count);
    }

    #[test]
    fn inherited_accessor_is_an_error_without_getter_invocation() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let prototype = vm.object().unwrap();
        let object = vm.get(vm.global(), "Object").unwrap();
        let Value::Object(object) = object else {
            panic!("Object constructor")
        };
        let Value::Object(object_prototype) = vm.get(object, "prototype").unwrap() else {
            panic!("Object prototype")
        };
        let add_property = vm.get(object_prototype, "addProperty").unwrap();
        let callback = vm.native(999).unwrap();
        // Only the supported Rust intrinsic is used to construct invented test
        // state, before a saved Environment exists. The getter is never called.
        vm.call(
            add_property,
            Value::Object(prototype),
            &[
                Value::string("onConstruct"),
                Value::Object(callback),
                Value::Null,
            ],
            &mut RejectHost,
        )
        .unwrap();
        vm.set_attached_prototype(owner, Some(prototype)).unwrap();
        let env = environment(&mut vm, owner);
        let pending = phase(&mut vm, owner, env, AttachedEventKind::Construct);
        let count = vm.object_count();
        let error = lookup_after_saved(&vm, pending, PROFILE).unwrap_err();
        assert_eq!(error.stage, Stage::MemberLookup);
        assert!(matches!(
            error.kind,
            FailureKind::Vm(vm::Error {
                kind: vm::ErrorKind::Unsupported("attached inherited accessor read"),
                ..
            })
        ));
        assert_eq!(vm.object_count(), count);
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            [Value::Number(17.0)]
        );
    }

    #[test]
    fn explicit_phase_context_name_and_child_snapshot_gates_fail_without_lookup_effects() {
        let (mut vm, owner) = setup(true, true);
        let env = environment(&mut vm, owner);
        let pending = phase(&mut vm, owner, env, AttachedEventKind::Construct);
        assert_eq!(
            lookup_after_saved(
                &vm,
                pending,
                Profile {
                    same_completed_phase: false
                }
            )
            .unwrap_err()
            .stage,
            Stage::Admission
        );
        assert_eq!(
            lookup_after_saved(
                &vm,
                saved_dispatch::Outcome::WrapperSkipped(
                    saved_dispatch::WrapperSkip::LoadEligibility
                ),
                PROFILE
            )
            .unwrap_err()
            .stage,
            Stage::Admission
        );
        assert_eq!(
            lookup_after_saved(&vm, pending, PROFILE).unwrap_err().stage,
            Stage::MemberLookup
        );
        for (with_names, bound, expected) in [
            (true, false, Stage::BuiltinContext),
            (false, true, Stage::MemberLookup),
        ] {
            let (mut vm, owner) = setup(with_names, bound);
            vm.set_attached_children(owner, vec![]).unwrap();
            let env = environment(&mut vm, owner);
            let pending = phase(&mut vm, owner, env, AttachedEventKind::Construct);
            assert_eq!(
                lookup_after_saved(&vm, pending, PROFILE).unwrap_err().stage,
                expected
            );
            assert_eq!(
                vm.saved_environment_state(env).unwrap().stack(),
                [Value::Number(17.0)]
            );
        }
        let invalid = saved_dispatch::Outcome::DynamicLookupPending {
            owner,
            environment: SavedEnvironmentId(usize::MAX),
            event: AttachedEventKind::Construct,
            progress: Progress::default(),
        };
        assert_eq!(
            lookup_after_saved(&vm, invalid, PROFILE).unwrap_err().stage,
            Stage::Environment
        );

        let other = vm
            .create_attached_movie_clip_with_binding(
                AttachedClipSpec {
                    target: DisplayTargetId(99),
                    source: AttachedClipSource {
                        movie: LoadedMovieSourceId(8),
                        sprite_id: 3,
                        header_version: 11,
                    },
                    parent: Some(owner),
                    event_state: AttachedEventState {
                        target_flags: 0x5200_0180,
                        load_flags: 8,
                        source_flags: 0,
                    },
                },
                AttachedMovieBinding::LocalSourceOrImportedLocalExport,
            )
            .unwrap();
        let other_env = environment(&mut vm, other);
        let mismatched = saved_dispatch::Outcome::DynamicLookupPending {
            owner,
            environment: other_env,
            event: AttachedEventKind::Construct,
            progress: Progress::default(),
        };
        assert_eq!(
            lookup_after_saved(&vm, mismatched, PROFILE)
                .unwrap_err()
                .stage,
            Stage::Environment
        );
    }

    #[test]
    fn halted_environment_rejects_stale_outcome_and_preserves_failure_state() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let env = environment(&mut vm, owner);
        let stale = phase(&mut vm, owner, env, AttachedEventKind::Construct);
        // Intentionally unsupported invented opcode, used to test error-state
        // admission. This is not an original body and is never retried.
        let program = Arc::new(Program::parse_at(&[0x2c, 0], 500).unwrap());
        let span = program.span;
        let code = vm.register(program).unwrap();
        let action = vm
            .install_saved_action(owner, SavedActionSource { code, span }, 0x40000)
            .unwrap();
        let mut table = SavedHandlers::<SavedActionId>::new(owner, Default::default());
        table.install_from_vm(&vm, action).unwrap();
        assert!(
            saved_dispatch::execute_saved_phase(
                &mut vm,
                &table,
                AttachedEventKind::Construct,
                env,
                &mut RejectHost
            )
            .is_err()
        );
        assert!(vm.saved_environment_halted(env).unwrap());
        let failure = lookup_after_saved(&vm, stale, PROFILE).unwrap_err();
        assert_eq!(failure.stage, Stage::Environment);
        assert_eq!(failure.phase, stale);
        assert!(vm.saved_environment_halted(env).unwrap());
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            [Value::Number(17.0)]
        );
    }

    #[test]
    fn live_lookup_observes_completed_saved_write_and_failure_preserves_saved_effects() {
        let (mut vm, owner) = setup(true, true);
        vm.set_attached_children(owner, vec![]).unwrap();
        let prototype = vm.object().unwrap();
        vm.set(prototype, "onConstruct", Value::Bool(false))
            .unwrap();
        vm.set_attached_prototype(owner, Some(prototype)).unwrap();
        vm.write_attached_member(owner, "holder", Value::Object(prototype))
            .unwrap();
        let env = environment(&mut vm, owner);
        // Invented body: GetVariable("holder").onConstruct = 77; End.
        let mut bytes = vec![0x96, 8, 0, 0];
        bytes.extend(b"holder\0");
        bytes.push(0x1c);
        let mut operands = vec![0];
        operands.extend(b"onConstruct\0");
        operands.extend([7, 77, 0, 0, 0]);
        bytes.extend([0x96, operands.len() as u8, 0]);
        bytes.extend(operands);
        bytes.extend([0x4f, 0]);
        let program = Arc::new(Program::parse_at(&bytes, 500).unwrap());
        let span = program.span;
        let code = vm.register(program).unwrap();
        let action = vm
            .install_saved_action(owner, SavedActionSource { code, span }, 0x40000)
            .unwrap();
        let mut table = SavedHandlers::<SavedActionId>::new(owner, Default::default());
        table.install_from_vm(&vm, action).unwrap();
        let pending = saved_dispatch::execute_saved_phase(
            &mut vm,
            &table,
            AttachedEventKind::Construct,
            env,
            &mut RejectHost,
        )
        .unwrap();
        let result = lookup_after_saved(&vm, pending, PROFILE).unwrap();
        assert_eq!(
            result.member,
            Member::PresentNonCallable(Value::Number(77.0))
        );
        assert_eq!(result.context.saved_progress.handlers_completed, 1);
        assert_eq!(result.context.saved_progress.instructions, 5);
        assert_eq!(result.context.saved_progress.host_calls, 0);
        // An explicit bad association is rejected, without rolling back the
        // prior phase or granting permission to run it again.
        let failure = lookup_after_saved(
            &vm,
            pending,
            Profile {
                same_completed_phase: false,
            },
        )
        .unwrap_err();
        assert_eq!(failure.phase, pending);
        assert_eq!(
            vm.get(prototype, "onConstruct").unwrap(),
            Value::Number(77.0)
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            [Value::Number(17.0)]
        );
        assert!(!vm.saved_environment_halted(env).unwrap());
    }
}
