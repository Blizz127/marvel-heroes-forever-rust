//! Focused synthetic saved-execution checks. No native defaults or event-order
//! claims: every owner, profile, stack and register bank is supplied explicitly.
//! The generated inventory and mocked digest test routing only; no extracted
//! native name inventory or client bytecode is stored in this module.

use super::saved_runtime::{SavedEnvironmentId, SavedExecution};
use super::saved_state::{SavedActionId, SavedActionSource, SavedEnvironmentState};
use super::*;
use std::sync::Arc;

fn profile() -> Arc<NativeNameProfile> {
    let mut inventory = format!(
        "MHF-ATTACHED-NAMES\t1\nbuild\t1.52.0.1700\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
        "ab".repeat(32)
    );
    for id in 0..117 {
        let alias = id <= 21 || (110..=115).contains(&id);
        let name = if id == 17 {
            "_focusrect".to_owned()
        } else {
            format!("{}synthetic{id:03}", if alias { "_" } else { "" })
        };
        inventory.push_str(&format!("standard\t{id}\t{}\t{name}\n", u8::from(alias)));
    }
    let mut movieclip = vec!["constructor".to_owned(), "__proto__".to_owned()];
    movieclip.extend((0..42).map(|id| format!("syntheticClip{id:03}")));
    movieclip.sort();
    let mut object = vec!["constructor".to_owned()];
    object.extend((0..8).map(|id| format!("syntheticObject{id:03}")));
    object.sort();
    for (role, names) in [("movieclip", movieclip), ("object", object)] {
        for name in names {
            inventory.push_str(&format!("{role}\t{name}\n"));
        }
    }
    let expected = ExpectedProfile {
        build: "1.52.0.1700",
        source_sha256: [0xab; 32],
        inventory_sha256: [0x62; 32],
    };
    Arc::new(
        NativeNameProfile::parse_verified(inventory.as_bytes(), &expected, |bytes| {
            assert_eq!(bytes, inventory.as_bytes());
            Ok([0x62; 32]) // Explicit mock, never an artifact-authentication claim.
        })
        .unwrap(),
    )
}

fn fixture() -> (Vm, AttachedClipId) {
    let mut vm = Vm::new(15, Limits::default()).unwrap();
    vm.install_object_constructor().unwrap();
    vm.install_movie_clip_constructor().unwrap();
    vm.install_attached_name_profile(profile()).unwrap();
    let owner = vm
        .create_attached_movie_clip(AttachedClipSpec {
            target: DisplayTargetId(81),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(3),
                sprite_id: 12,
                header_version: 15,
            },
            parent: None,
            event_state: AttachedEventState {
                target_flags: 0x5200_0180,
                load_flags: 0,
                source_flags: 0,
            },
        })
        .unwrap();
    vm.adopt_attached_root(owner).unwrap();
    vm.set_attached_children(owner, vec![]).unwrap();
    (vm, owner)
}
fn supplied_registers() -> [Value; 4] {
    [
        Value::Null,
        Value::Bool(true),
        Value::Number(27.0),
        Value::string("retained register"),
    ]
}
fn environment(
    vm: &mut Vm,
    owner: AttachedClipId,
    stack: Vec<Value>,
    ceiling: usize,
) -> SavedEnvironmentId {
    let state = SavedEnvironmentState::from_supplied(
        owner,
        vm.attached_owner(owner).unwrap(),
        stack,
        supplied_registers(),
        ceiling,
    )
    .unwrap();
    vm.register_saved_environment(state).unwrap()
}
fn action(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend((payload.len() as u16).to_le_bytes());
    bytes.extend(payload);
    bytes
}
fn push(values: &[Value]) -> Vec<u8> {
    let mut payload = Vec::new();
    for value in values {
        match value {
            Value::String(value) => {
                payload.push(0);
                payload.extend(String::from_utf16(value).unwrap().as_bytes());
                payload.push(0);
            }
            Value::Bool(value) => payload.extend([5, u8::from(*value)]),
            Value::Number(value) => {
                assert_eq!(*value, f64::from(*value as i32));
                payload.push(7);
                payload.extend((*value as i32).to_le_bytes());
            }
            _ => panic!("unsupported synthetic Push literal"),
        }
    }
    action(0x96, &payload)
}
fn pool(bytes: &[u8]) -> Vec<u8> {
    let mut payload = vec![1, 0];
    payload.extend(bytes);
    payload.push(0);
    action(0x88, &payload)
}
fn install(vm: &mut Vm, owner: AttachedClipId, bytes: &[u8]) -> (SavedActionId, SavedActionSource) {
    assert!(!bytes.is_empty() && bytes[0] != 0);
    let program = Arc::new(Program::parse_at(bytes, 1000).unwrap());
    let span = program.span;
    let source = SavedActionSource {
        code: vm.register(program).unwrap(),
        span,
    };
    let id = vm.install_saved_action(owner, source, 1).unwrap();
    assert_eq!(vm.saved_action_info(id).unwrap().source, source);
    (id, source)
}
fn completed(result: SavedExecution) {
    assert!(matches!(
        result,
        SavedExecution::Completed { host_calls: 0, .. }
    ));
}

#[test]
fn saved_define_local_uses_target_normal_setter_and_preserves_global() {
    let (mut vm, owner) = fixture();
    vm.set(vm.global(), "localProbe", Value::Number(88.0))
        .unwrap();
    vm.set(vm.global(), "_focusrect", Value::Bool(true))
        .unwrap();
    vm.bind_attached_focus_rect(
        owner,
        AttachedFocusRectState {
            target_flags_b8: 0x2468_0180,
        },
    )
    .unwrap();
    let env = environment(&mut vm, owner, vec![Value::Number(9.0)], 8);
    let mut bytes = push(&[Value::string("localProbe"), Value::Number(12.0)]);
    bytes.push(0x3c);
    bytes.extend(push(&[Value::string("_focusrect"), Value::Bool(false)]));
    let focus_write_offset = 1000 + bytes.len();
    bytes.push(0x3c);
    let (id, source) = install(&mut vm, owner, &bytes);
    let objects_before = vm.object_count();
    assert!(vm.attached_info(owner).unwrap().storage.is_none());
    completed(vm.execute_saved_action(id, env, &mut RejectHost).unwrap());
    assert_eq!(
        vm.get(vm.global(), "localProbe").unwrap(),
        Value::Number(88.0)
    );
    assert_eq!(
        vm.get(vm.global(), "_focusrect").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        vm.read_attached_member(owner, "localProbe").unwrap(),
        AttachedMember::Present(Value::Number(12.0))
    );
    let target = vm.attached_owner(owner).unwrap();
    assert!(
        !vm.objects[target.0]
            .properties
            .contains_key(&text("localProbe"))
    );
    let focus = vm.attached_focus_rect(owner).unwrap().unwrap();
    assert_eq!(focus.state.target_flags_b8, 0x2468_0100);
    assert_eq!(
        focus.last_write,
        Some(CodeOrigin {
            code: source.code,
            source_offset: focus_write_offset
        })
    );
    // Exactly the ordinary write's lazy storage, with no activation allocation.
    assert_eq!(vm.object_count(), objects_before + 1);
    assert_eq!(
        vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Number(9.0)]
    );
    assert_eq!(
        vm.saved_environment_state(env).unwrap().registers(),
        &supplied_registers()
    );
}

#[test]
fn saved_define_local2_converts_and_pops_without_declaring_or_overwriting() {
    let (mut vm, owner) = fixture();
    vm.write_attached_member(owner, "localProbe", Value::Number(12.0))
        .unwrap();
    vm.set(vm.global(), "localProbe", Value::Number(88.0))
        .unwrap();
    vm.bind_attached_focus_rect(
        owner,
        AttachedFocusRectState {
            target_flags_b8: 0x180,
        },
    )
    .unwrap();
    let env = environment(
        &mut vm,
        owner,
        vec![
            Value::Number(9.0),
            Value::string("_focusrect"),
            Value::Number(42.0),
            Value::string("localProbe"),
        ],
        8,
    );
    let (id, _) = install(&mut vm, owner, &[0x41, 0x41, 0x41]);
    let before = vm.object_count();
    let string_units = vm.string_units;
    completed(vm.execute_saved_action(id, env, &mut RejectHost).unwrap());
    assert_eq!(
        vm.read_attached_member(owner, "localProbe").unwrap(),
        AttachedMember::Present(Value::Number(12.0))
    );
    assert_eq!(
        vm.get(vm.global(), "localProbe").unwrap(),
        Value::Number(88.0)
    );
    assert_eq!(
        vm.read_attached_member(owner, "42").unwrap(),
        AttachedMember::Missing
    );
    assert_eq!(vm.get(vm.global(), "42").unwrap(), Value::Undefined);
    assert_eq!(
        vm.attached_focus_rect(owner)
            .unwrap()
            .unwrap()
            .state
            .target_flags_b8,
        0x180
    );
    assert!(
        vm.attached_focus_rect(owner)
            .unwrap()
            .unwrap()
            .last_write
            .is_none()
    );
    assert_eq!(vm.object_count(), before);
    // Rust conversion of the supplied integer to "42" still ran; no claim
    // about native allocation counts or interning follows from this counter.
    assert_eq!(vm.string_units, string_units + 2);
    assert_eq!(
        vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Number(9.0)]
    );

    let (mut vm, owner) = fixture();
    let object_name = vm.object().unwrap();
    let env = environment(&mut vm, owner, vec![Value::Object(object_name)], 8);
    let (id, _) = install(&mut vm, owner, &[0x41]);
    let error = vm
        .execute_saved_action(id, env, &mut RejectHost)
        .unwrap_err();
    assert_eq!(
        error.kind,
        ErrorKind::Unsupported("object primitive coercion requires valueOf/toString")
    );
    assert!(vm.saved_environment_state(env).unwrap().stack().is_empty());
    assert!(vm.saved_environment_halted(env).unwrap());
    assert!(vm.attached_info(owner).unwrap().storage.is_none());
}

#[test]
fn saved_register_bank_persists_and_out_of_range_store_never_peeks_empty_stack() {
    let (mut vm, owner) = fixture();
    let env = environment(&mut vm, owner, vec![], 8);
    let mut bytes = action(0x87, &[4]);
    bytes.extend(action(0x87, &[255]));
    bytes.extend(push(&[Value::Number(55.0)]));
    bytes.extend(action(0x87, &[2]));
    bytes.push(0x17);
    bytes.extend(action(0x96, &[4, 4, 4, 255]));
    let (id, _) = install(&mut vm, owner, &bytes);
    completed(vm.execute_saved_action(id, env, &mut RejectHost).unwrap());
    let state = vm.saved_environment_state(env).unwrap();
    assert_eq!(state.stack(), &[Value::Undefined, Value::Undefined]);
    let mut expected = supplied_registers();
    expected[2] = Value::Number(55.0);
    assert_eq!(state.registers(), &expected);
    let (second, _) = install(&mut vm, owner, &action(0x96, &[4, 2]));
    completed(
        vm.execute_saved_action(second, env, &mut RejectHost)
            .unwrap(),
    );
    assert_eq!(
        vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Undefined, Value::Undefined, Value::Number(55.0)]
    );
    assert_eq!(
        vm.saved_environment_state(env).unwrap().registers(),
        &expected
    );
}

#[test]
fn saved_pool_uses_first_executed_record_and_invalid_indices_are_undefined() {
    let (mut vm, owner) = fixture();
    let env = environment(&mut vm, owner, vec![], 12);
    // The earlier serialized pool is skipped initially, then visited after the
    // chosen pool. Invalid UTF-8 would fail Rust decoding if it were consulted.
    let mut bytes = action(0x96, &[8, 200]); // No pool yet: Undefined.
    let first_jump = bytes.len();
    bytes.extend(action(0x99, &[0, 0]));
    let earlier_pool = bytes.len();
    bytes.extend(pool(&[0xff]));
    let tail_jump = bytes.len();
    bytes.extend(action(0x99, &[0, 0]));
    let chosen_pool = bytes.len();
    bytes.extend(pool(b"chosen"));
    let back_jump = bytes.len();
    bytes.extend(action(0x99, &[0, 0]));
    let tail = bytes.len();
    bytes.extend(action(0x96, &[8, 0, 8, 1, 9, 255, 255]));
    for (at, target) in [
        (first_jump, chosen_pool),
        (tail_jump, tail),
        (back_jump, earlier_pool),
    ] {
        let delta = i16::try_from(target as isize - (at + 5) as isize).unwrap();
        bytes[at + 3..at + 5].copy_from_slice(&delta.to_le_bytes());
    }
    let (id, _) = install(&mut vm, owner, &bytes);
    let expected = [
        Value::Undefined,
        Value::string("chosen"),
        Value::Undefined,
        Value::Undefined,
    ];
    let units_before = vm.string_units;
    completed(vm.execute_saved_action(id, env, &mut RejectHost).unwrap());
    assert_eq!(vm.saved_environment_state(env).unwrap().stack(), &expected);
    assert_eq!(
        vm.saved_action_info(id).unwrap().first_pool_offset,
        Some(chosen_pool as u32)
    );
    assert!(chosen_pool > earlier_pool);
    assert_eq!(vm.string_units, units_before + 6);
    completed(vm.execute_saved_action(id, env, &mut RejectHost).unwrap());
    assert_eq!(
        vm.saved_environment_state(env).unwrap().stack(),
        [expected.as_slice(), expected.as_slice()].concat()
    );
    assert_eq!(vm.string_units, units_before + 6);
}

#[test]
fn saved_supplied_stack_ceiling_preserves_partial_stack_bank_and_pool_on_error() {
    let (mut vm, owner) = fixture();
    assert!(vm.limits.stack_values > 2);
    let env = environment(&mut vm, owner, vec![Value::Number(9.0)], 2);
    let mut bytes = pool(b"held");
    bytes.extend(action(0x96, &[8, 0]));
    bytes.extend(action(0x87, &[1]));
    let failing_offset = 1000 + bytes.len();
    bytes.extend(push(&[Value::Number(77.0)]));
    let (id, source) = install(&mut vm, owner, &bytes);
    let error = vm
        .execute_saved_action(id, env, &mut RejectHost)
        .unwrap_err();
    assert!(matches!(error.kind, ErrorKind::Budget(_)));
    assert_eq!(
        error.origin,
        Some(CodeOrigin {
            code: source.code,
            source_offset: failing_offset
        })
    );
    let state = vm.saved_environment_state(env).unwrap();
    assert_eq!(state.stack(), &[Value::Number(9.0), Value::string("held")]);
    assert_eq!(state.registers()[1], Value::string("held"));
    assert_eq!(vm.saved_action_info(id).unwrap().first_pool_offset, Some(0));
    assert!(vm.saved_environment_halted(env).unwrap());
    let instructions = vm.instruction_count();
    assert_eq!(
        vm.execute_saved_action(id, env, &mut RejectHost)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported("saved Environment is halted")
    );
    assert_eq!(vm.instruction_count(), instructions);
}
