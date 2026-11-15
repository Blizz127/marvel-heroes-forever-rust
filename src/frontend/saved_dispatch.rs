//! The saved-vector phase of one explicitly requested Construct/Load delivery.
//!
//! Native wrapper and ordering: `docs/FRONTEND_EVENT_DISPATCH.md`. This does not
//! schedule events or invoke the dynamic member. A normal phase ends with that
//! lookup still pending. The table and VM must belong to the same session.

use super::saved_handlers::SavedHandlers;
use crate::avm1::vm::{
    self, AttachedClipId, AttachedEventKind, Host, SavedActionId, SavedEnvironmentId,
    SavedExecution, Vm,
};
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    pub static_map_present: bool,
    pub handlers_completed: usize,
    pub handlers_target_skipped: usize,
    /// Counts returned by completed bodies only. An error can have further
    /// partial effects retained by the VM; it is not included in these totals.
    pub instructions: usize,
    pub host_calls: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapperSkip {
    TargetBit4,
    LoadEligibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// No saved body or dynamic member is called. LoadEligibility may still
    /// clear target bit 5, as the native wrapper does before its extra gate.
    WrapperSkipped(WrapperSkip),
    /// Saved calls returned. Resolve the current dynamic member only now;
    /// its receiver is this owner and its caller is this retained Environment.
    DynamicLookupPending {
        owner: AttachedClipId,
        environment: SavedEnvironmentId,
        event: AttachedEventKind,
        progress: Progress,
    },
}

#[derive(Debug)]
pub enum FailureKind {
    Vm(vm::Error),
    Admission(&'static str),
    CounterOverflow,
}

#[derive(Debug)]
pub struct Failure {
    pub kind: FailureKind,
    pub handler_index: Option<usize>,
    pub action: Option<SavedActionId>,
    pub progress: Progress,
    /// False means this phase made no VM mutation. Once the wrapper has run,
    /// an error is a diagnostic stop, never authorization to replay the event.
    pub effects_may_be_partial: bool,
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "saved dispatch at {:?}: ", self.handler_index)?;
        match &self.kind {
            FailureKind::Vm(error) => error.fmt(f),
            FailureKind::Admission(message) => f.write_str(message),
            FailureKind::CounterOverflow => f.write_str("progress counter overflow"),
        }
    }
}
impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            FailureKind::Vm(error) => Some(error),
            _ => None,
        }
    }
}

/// Execute the complete immutable saved vector for one selected key.
///
/// The caller owns an idle, adopted Environment with explicit current state.
/// No stack/register defaults are supplied. Immutable borrowing excludes handler
/// table mutation; this is not the native general reentrant-vector profile.
/// Unsupported installation/Environment admission fails before wrapper effects.
/// A body error preserves VM state and stops before all later handlers and the
/// dynamic member. This routine never retries, rolls back or reports complete
/// event delivery. Queue eligibility and event production remain caller work.
pub fn execute_saved_phase(
    vm: &mut Vm,
    table: &SavedHandlers<SavedActionId>,
    event: AttachedEventKind,
    environment: SavedEnvironmentId,
    host: &mut impl Host,
) -> Result<Outcome, Failure> {
    let entries = table.handlers(event);
    let mut progress = Progress {
        static_map_present: entries.is_some(),
        ..Progress::default()
    };
    let fail = |kind, handler_index, action, progress, effects_may_be_partial| Failure {
        kind,
        handler_index,
        action,
        progress,
        effects_may_be_partial,
    };
    let pre = |error| fail(FailureKind::Vm(error), None, None, progress, false);
    let owner = table.owner();
    let target = vm.attached_owner(owner).map_err(pre)?;
    let state = vm.saved_environment_state(environment).map_err(pre)?;
    if state.owner() != owner || state.target() != target {
        return Err(fail(
            FailureKind::Admission("table and Environment owner/target differ"),
            None,
            None,
            progress,
            false,
        ));
    }
    if vm.saved_environment_halted(environment).map_err(pre)? {
        return Err(fail(
            FailureKind::Admission("saved Environment is halted"),
            None,
            None,
            progress,
            false,
        ));
    }
    let required_mask = match event {
        AttachedEventKind::Load => 1,
        AttachedEventKind::Construct => 0x40000,
    };
    // Public generic tables can contain supplied IDs/masks. Check every entry
    // against the actual arena, not just the first selected wrapper.
    for (index, &action) in entries.unwrap_or_default().iter().enumerate() {
        let info = vm.saved_action_info(action).map_err(|error| {
            fail(
                FailureKind::Vm(error),
                Some(index),
                Some(action),
                progress,
                false,
            )
        })?;
        if info.owner != owner
            || table.original_mask(action) != Some(info.original_mask)
            || info.original_mask != required_mask
        {
            return Err(fail(
                FailureKind::Admission(
                    "saved phase requires matching pure-mask owner installations",
                ),
                Some(index),
                Some(action),
                progress,
                false,
            ));
        }
    }

    let mut flags = vm.attached_event_state(owner).map_err(pre)?;
    let target_skipped = flags.target_flags & 0x10 != 0;
    let admitted = flags.admit(event);
    vm.update_attached_event_state(owner, flags).map_err(pre)?;
    if !admitted {
        return Ok(Outcome::WrapperSkipped(if target_skipped {
            WrapperSkip::TargetBit4
        } else {
            WrapperSkip::LoadEligibility
        }));
    }

    for (index, &action) in entries.unwrap_or_default().iter().enumerate() {
        let result = vm
            .execute_saved_action(action, environment, host)
            .map_err(|error| {
                fail(
                    FailureKind::Vm(error),
                    Some(index),
                    Some(action),
                    progress,
                    true,
                )
            })?;
        match result {
            SavedExecution::TargetBit4Skipped => progress.handlers_target_skipped += 1,
            SavedExecution::Completed {
                instructions,
                host_calls,
            } => {
                progress.handlers_completed += 1;
                let Some(instructions) = progress.instructions.checked_add(instructions) else {
                    return Err(fail(
                        FailureKind::CounterOverflow,
                        Some(index),
                        Some(action),
                        progress,
                        true,
                    ));
                };
                progress.instructions = instructions;
                let Some(host_calls) = progress.host_calls.checked_add(host_calls) else {
                    return Err(fail(
                        FailureKind::CounterOverflow,
                        Some(index),
                        Some(action),
                        progress,
                        true,
                    ));
                };
                progress.host_calls = host_calls;
            }
        }
    }
    Ok(Outcome::DynamicLookupPending {
        owner,
        environment,
        event,
        progress,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::{
        Program,
        vm::{
            AttachedClipSource, AttachedClipSpec, AttachedEventState, DisplayTargetId,
            LoadedMovieSourceId, RejectHost, SavedActionSource, SavedEnvironmentState, Value,
        },
    };
    use std::sync::Arc;

    fn fixture() -> (Vm, SavedHandlers<SavedActionId>, SavedEnvironmentId) {
        let mut vm = Vm::new(15, Default::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
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
        let env = vm.register_saved_environment(state).unwrap();
        (vm, SavedHandlers::new(owner, Default::default()), env)
    }
    fn install(
        vm: &mut Vm,
        table: &mut SavedHandlers<SavedActionId>,
        bytes: &[u8],
        mask: u32,
    ) -> SavedActionId {
        let program = Arc::new(Program::parse_at(bytes, 500).unwrap());
        let span = program.span;
        let code = vm.register(program).unwrap();
        let action = vm
            .install_saved_action(table.owner(), SavedActionSource { code, span }, mask)
            .unwrap();
        table.install_from_vm(vm, action).unwrap();
        action
    }
    #[test]
    fn saved_vector_preserves_order_and_environment_then_leaves_dynamic_lookup_pending() {
        let (mut vm, mut table, env) = fixture();
        // Push 40; StoreRegister 1. Leave the value on the shared stack.
        install(
            &mut vm,
            &mut table,
            &[0x96, 5, 0, 7, 40, 0, 0, 0, 0x87, 1, 0, 1, 0],
            0x40000,
        );
        // Push 2; Add2; StoreRegister 2. Consumes the preceding handler's 40.
        install(
            &mut vm,
            &mut table,
            &[0x96, 5, 0, 7, 2, 0, 0, 0, 0x47, 0x87, 1, 0, 2, 0],
            0x40000,
        );
        let outcome = execute_saved_phase(
            &mut vm,
            &table,
            AttachedEventKind::Construct,
            env,
            &mut RejectHost,
        )
        .unwrap();
        let Outcome::DynamicLookupPending {
            owner,
            environment,
            progress,
            ..
        } = outcome
        else {
            panic!("dynamic phase required")
        };
        assert_eq!((owner, environment), (table.owner(), env));
        assert!(progress.static_map_present);
        assert_eq!(
            (
                progress.handlers_completed,
                progress.handlers_target_skipped,
                progress.host_calls
            ),
            (2, 0, 0)
        );
        let state = vm.saved_environment_state(env).unwrap();
        assert_eq!(state.stack(), [Value::Number(17.0), Value::Number(42.0)]);
        assert_eq!(state.registers()[1], Value::Number(40.0));
        assert_eq!(state.registers()[2], Value::Number(42.0));
        assert!(!vm.saved_environment_halted(env).unwrap());
    }
    #[test]
    fn wrapper_gates_preserve_their_native_mutation_order_and_do_not_run_bodies() {
        let (mut vm, mut table, env) = fixture();
        let action = install(&mut vm, &mut table, &[0x88, 4, 0, 1, 0, b'x', 0, 0], 1);
        let owner = table.owner();
        let mut flags = vm.attached_event_state(owner).unwrap();
        flags.target_flags |= 0x10;
        vm.update_attached_event_state(owner, flags).unwrap();
        assert_eq!(
            execute_saved_phase(
                &mut vm,
                &table,
                AttachedEventKind::Load,
                env,
                &mut RejectHost
            )
            .unwrap(),
            Outcome::WrapperSkipped(WrapperSkip::TargetBit4)
        );
        assert_eq!(vm.attached_event_state(owner).unwrap(), flags);
        flags.target_flags &= !0x10;
        flags.load_flags = 0;
        vm.update_attached_event_state(owner, flags).unwrap();
        assert_eq!(
            execute_saved_phase(
                &mut vm,
                &table,
                AttachedEventKind::Load,
                env,
                &mut RejectHost
            )
            .unwrap(),
            Outcome::WrapperSkipped(WrapperSkip::LoadEligibility)
        );
        assert_eq!(
            vm.attached_event_state(owner).unwrap().target_flags,
            flags.target_flags & !0x20
        );
        assert_eq!(
            vm.saved_action_info(action).unwrap().first_pool_offset,
            None
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            [Value::Number(17.0)]
        );
        // No static key still requires later dynamic lookup after admission.
        let empty = SavedHandlers::new(owner, Default::default());
        let Outcome::DynamicLookupPending { progress, .. } = execute_saved_phase(
            &mut vm,
            &empty,
            AttachedEventKind::Construct,
            env,
            &mut RejectHost,
        )
        .unwrap() else {
            panic!("dynamic lookup is independent of static presence")
        };
        assert!(!progress.static_map_present);
        assert_eq!(progress.handlers_completed, 0);
    }
    #[test]
    fn body_failure_retains_prior_effects_and_does_not_visit_later_wrappers_or_retry() {
        let (mut vm, mut table, env) = fixture();
        install(&mut vm, &mut table, &[0x96, 5, 0, 7, 40, 0, 0, 0, 0], 1);
        let failed = install(&mut vm, &mut table, &[0x2c, 0], 1);
        let later = install(&mut vm, &mut table, &[0x88, 4, 0, 1, 0, b'x', 0, 0], 1);
        let error = execute_saved_phase(
            &mut vm,
            &table,
            AttachedEventKind::Load,
            env,
            &mut RejectHost,
        )
        .unwrap_err();
        assert_eq!(error.action, Some(failed));
        assert_eq!(error.handler_index, Some(1));
        assert_eq!(error.progress.handlers_completed, 1);
        assert!(error.effects_may_be_partial);
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            [Value::Number(17.0), Value::Number(40.0)]
        );
        assert!(vm.saved_environment_halted(env).unwrap());
        assert_eq!(vm.saved_action_info(later).unwrap().first_pool_offset, None);
        let retry = execute_saved_phase(
            &mut vm,
            &table,
            AttachedEventKind::Load,
            env,
            &mut RejectHost,
        )
        .unwrap_err();
        assert!(!retry.effects_may_be_partial);
        assert_eq!(retry.progress.handlers_completed, 0);
    }
    #[test]
    fn preflight_rejects_actual_mask_mismatch_and_combined_mode_before_load_mutation() {
        let (mut vm, mut actual, env) = fixture();
        let action = install(&mut vm, &mut actual, &[0x17, 0], 1 | 0x40000);
        let owner = actual.owner();
        let before = vm.attached_event_state(owner).unwrap();
        let mut supplied = SavedHandlers::new(owner, Default::default());
        supplied.install(owner, action, 1).unwrap();
        for table in [&actual, &supplied] {
            let error = execute_saved_phase(
                &mut vm,
                table,
                AttachedEventKind::Load,
                env,
                &mut RejectHost,
            )
            .unwrap_err();
            assert!(!error.effects_may_be_partial);
            assert!(matches!(error.kind, FailureKind::Admission(_)));
            assert_eq!(vm.attached_event_state(owner).unwrap(), before);
            assert!(!vm.saved_environment_halted(env).unwrap());
        }
    }
}
