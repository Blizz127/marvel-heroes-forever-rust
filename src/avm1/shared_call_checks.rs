//! Synthetic checks for the explicitly admitted saved/shared script-call path.
//! Every Environment, operand, fixed register, callable and binding witness is
//! supplied. These tests do not establish native defaults, original handler
//! execution, captured-Environment discovery, or native allocator parity.

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
        inventory.push_str(&format!(
            "standard\t{id}\t{}\t{}synthetic{id:03}\n",
            u8::from(alias),
            if alias { "_" } else { "" }
        ));
    }
    let mut clips = vec!["constructor".to_owned(), "__proto__".to_owned()];
    clips.extend((0..42).map(|id| format!("syntheticClip{id:03}")));
    clips.sort();
    let mut objects = vec!["constructor".to_owned()];
    objects.extend((0..8).map(|id| format!("syntheticObject{id:03}")));
    objects.sort();
    for (role, names) in [("movieclip", clips), ("object", objects)] {
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
            Ok([0x62; 32]) // Mock authentication for synthetic names only.
        })
        .unwrap(),
    )
}

fn setup() -> (Vm, AttachedClipId) {
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

fn record(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend(u16::try_from(payload.len()).unwrap().to_le_bytes());
    bytes.extend(payload);
    bytes
}

fn push(values: &[Value]) -> Vec<u8> {
    let mut payload = Vec::new();
    for value in values {
        match value {
            Value::Number(value) => {
                assert_eq!(*value, f64::from(*value as i32));
                payload.push(7);
                payload.extend((*value as i32).to_le_bytes());
            }
            Value::String(value) => {
                payload.push(0);
                payload.extend(String::from_utf16(value).unwrap().as_bytes());
                payload.push(0);
            }
            Value::Bool(value) => payload.extend([5, u8::from(*value)]),
            Value::Null => payload.push(2),
            Value::Undefined => payload.push(3),
            _ => panic!("nonliteral values belong in the supplied Environment"),
        }
    }
    record(0x96, &payload)
}

fn return_number(value: f64) -> Vec<u8> {
    let mut body = push(&[Value::Number(value)]);
    body.push(0x3e);
    body
}

struct Fixture {
    vm: Vm,
    owner: AttachedClipId,
    receiver: ObjectId,
    callable: ObjectId,
    callee_code: CodeId,
    body_offset: usize,
}

fn define(
    vm: &mut Vm,
    target: ObjectId,
    flags: u16,
    count: u8,
    parameters: &[(u8, &str)],
    body: &[u8],
) -> (ObjectId, CodeId, usize) {
    let mut payload = vec![0]; // Anonymous function leaves its reference on top.
    payload.extend(u16::try_from(parameters.len()).unwrap().to_le_bytes());
    payload.push(count);
    payload.extend(flags.to_le_bytes());
    for (register, name) in parameters {
        payload.push(*register);
        payload.extend(name.as_bytes());
        payload.push(0);
    }
    payload.extend(u16::try_from(body.len()).unwrap().to_le_bytes());
    let mut bytes = record(0x8e, &payload);
    bytes.extend(body); // Function body is outside the ActionLength payload.
    let program = Arc::new(Program::parse_at(&bytes, 40_000).unwrap());
    let Operands::Function(definition) = &program.instructions[0].operands else {
        panic!("synthetic definition must decode as Function2")
    };
    let body_offset = definition.body.span.offset;
    let code = vm.register(program).unwrap();
    let execution = vm.execute(code, target, &mut RejectHost).unwrap();
    let [Value::Object(callable)] = execution.stack.as_slice() else {
        panic!("anonymous definition must produce exactly one callable")
    };
    (*callable, code, body_offset)
}

fn fixture(flags: u16, count: u8, parameters: &[(u8, &str)], body: &[u8]) -> Fixture {
    let (mut vm, owner) = setup();
    let target = vm.attached_owner(owner).unwrap();
    let (callable, callee_code, body_offset) =
        define(&mut vm, target, flags, count, parameters, body);
    let receiver = vm.object().unwrap(); // Plain receiver with no prototype.
    vm.set(receiver, "m", Value::Object(callable)).unwrap();
    Fixture {
        vm,
        owner,
        receiver,
        callable,
        callee_code,
        body_offset,
    }
}

fn fixed() -> [Value; 4] {
    [
        Value::Null,
        Value::Bool(true),
        Value::Number(27.0),
        Value::string("supplied fixed register"),
    ]
}

fn operands(receiver: ObjectId, args: &[Value]) -> Vec<Value> {
    let mut stack = vec![Value::Number(17.0)];
    stack.extend(args.iter().rev().cloned());
    stack.extend([
        Value::Number(args.len() as f64),
        Value::Object(receiver),
        Value::string("m"),
    ]);
    stack
}

fn adopt(f: &mut Fixture, stack: Vec<Value>, ceiling: usize) -> SavedEnvironmentId {
    let state = SavedEnvironmentState::from_supplied(
        f.owner,
        f.vm.attached_owner(f.owner).unwrap(),
        stack,
        fixed(),
        ceiling,
    )
    .unwrap();
    f.vm.register_saved_environment(state).unwrap()
}

fn admit(f: &mut Fixture, env: SavedEnvironmentId) {
    f.vm.enable_saved_script_calls(env, SavedScriptCallProfile::EmptyDynamicNoPendingException)
        .unwrap();
    f.vm.bind_saved_script_environment(env, f.callable).unwrap();
}

fn install(f: &mut Fixture, bytes: &[u8]) -> SavedActionId {
    let program = Arc::new(Program::parse_at(bytes, 70_000).unwrap());
    let span = program.span;
    let source = SavedActionSource {
        code: f.vm.register(program).unwrap(),
        span,
    };
    f.vm.install_saved_action(f.owner, source, 1).unwrap()
}

fn complete(f: &mut Fixture, action: SavedActionId, env: SavedEnvironmentId) {
    let previous = (
        f.vm.current,
        f.vm.current_code,
        f.vm.current_target,
        f.vm.depth,
    );
    let result =
        f.vm.execute_saved_action(action, env, &mut RejectHost)
            .unwrap();
    assert!(matches!(
        result,
        SavedExecution::Completed { host_calls: 0, .. }
    ));
    assert!(!f.vm.saved_environment_halted(env).unwrap());
    assert!(
        f.vm.saved_environment_dynamic_registers(env)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        (
            f.vm.current,
            f.vm.current_code,
            f.vm.current_target,
            f.vm.depth
        ),
        previous
    );
}

#[test]
fn shared_call_cleanup_uses_current_top_for_balanced_residual_and_consumed_operands() {
    let baseline = return_number(42.0);
    let mut residual = push(&[Value::Number(99.0)]);
    residual.extend(return_number(42.0));
    let mut consumed = vec![0x17]; // Pop the caller's method-name operand.
    consumed.extend(return_number(42.0));
    let fallthrough = push(&[Value::Number(99.0)]);
    for (body, expected) in [
        (baseline, vec![Value::Number(17.0), Value::Number(42.0)]),
        (
            residual,
            vec![Value::Number(17.0), Value::Number(7.0), Value::Number(42.0)],
        ),
        (consumed, vec![Value::Number(42.0)]),
        (
            fallthrough,
            vec![Value::Number(17.0), Value::Number(7.0), Value::Undefined],
        ),
    ] {
        let mut f = fixture(0x2a, 2, &[], &body);
        let stack = operands(f.receiver, &[Value::Number(7.0)]);
        let env = adopt(&mut f, stack, 16);
        admit(&mut f, env);
        let action = install(&mut f, &[0x52]); // Caller falls through, not Return.
        complete(&mut f, action, env);
        assert_eq!(f.vm.saved_environment_state(env).unwrap().stack(), expected);
        assert_eq!(
            f.vm.saved_environment_state(env).unwrap().registers(),
            &fixed()
        );
    }
}

#[test]
fn shared_parameters_skip_missing_duplicate_register_and_preload_this_last() {
    let mut body = record(0x96, &[4, 1]);
    body.push(0x3e);
    for flags in [0x2a, 0x29] {
        for args in [
            vec![],
            vec![Value::Number(7.0)],
            vec![Value::Number(7.0), Value::Number(9.0)],
        ] {
            let mut f = fixture(flags, 2, &[(1, "first"), (1, "second")], &body);
            let expected = if flags == 0x29 {
                Value::Object(f.receiver)
            } else {
                args.last().cloned().unwrap_or(Value::Undefined)
            };
            let stack = operands(f.receiver, &args);
            let env = adopt(&mut f, stack, 16);
            admit(&mut f, env);
            let action = install(&mut f, &[0x52]);
            complete(&mut f, action, env);
            assert_eq!(
                f.vm.saved_environment_state(env).unwrap().stack(),
                &[Value::Number(17.0), expected]
            );
            assert_eq!(
                f.vm.saved_environment_state(env).unwrap().registers(),
                &fixed()
            );
        }
    }
}

#[test]
fn shared_missing_named_parameter_shadows_global_with_undefined() {
    let mut body = push(&[Value::string("missing")]);
    body.extend([0x1c, 0x3e]);
    let mut f = fixture(0x2a, 2, &[(1, "first"), (0, "missing")], &body);
    f.vm.set(f.vm.global(), "missing", Value::Number(88.0))
        .unwrap();
    let stack = operands(f.receiver, &[Value::Number(7.0)]);
    let env = adopt(&mut f, stack, 16);
    admit(&mut f, env);
    let action = install(&mut f, &[0x52]);
    complete(&mut f, action, env);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Number(17.0), Value::Undefined]
    );
    assert_eq!(
        f.vm.get(f.vm.global(), "missing").unwrap(),
        Value::Number(88.0)
    );
}

#[test]
fn shared_activation_locals_and_restored_saved_fence_use_distinct_targets() {
    let mut body = push(&[Value::string("probeLocal"), Value::Number(21.0)]);
    body.push(0x3c);
    body.extend(push(&[Value::string("probeLocal")]));
    body.extend([0x1c, 0x3e]);
    let mut f = fixture(0x2a, 0, &[], &body);
    f.vm.set(f.vm.global(), "probeLocal", Value::Number(88.0))
        .unwrap();
    let stack = operands(f.receiver, &[]);
    let env = adopt(&mut f, stack, 16);
    admit(&mut f, env);
    let mut bytes = vec![0x52];
    bytes.extend(push(&[Value::string("afterCall"), Value::Number(19.0)]));
    bytes.push(0x3c);
    let action = install(&mut f, &bytes);
    complete(&mut f, action, env);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Number(17.0), Value::Number(21.0)]
    );
    assert_eq!(
        f.vm.get(f.vm.global(), "probeLocal").unwrap(),
        Value::Number(88.0)
    );
    assert_eq!(
        f.vm.get(f.receiver, "probeLocal").unwrap(),
        Value::Undefined
    );
    assert_eq!(
        f.vm.read_attached_member(f.owner, "probeLocal").unwrap(),
        AttachedMember::Missing
    );
    assert_eq!(
        f.vm.read_attached_member(f.owner, "afterCall").unwrap(),
        AttachedMember::Present(Value::Number(19.0))
    );
}

#[test]
fn shared_callee_error_keeps_partial_stack_fixed_and_dynamic_then_halts() {
    let mut body = push(&[Value::Number(55.0)]);
    body.extend(record(0x87, &[1]));
    body.push(0x17);
    body.extend(push(&[Value::Number(66.0)]));
    let failed_relative = body.len();
    body.extend(push(&[Value::Number(77.0)]));
    let mut f = fixture(0x2a, 2, &[], &body);
    let original_stack = operands(f.receiver, &[Value::Number(7.0)]);
    let env = adopt(&mut f, original_stack.clone(), 6);
    admit(&mut f, env);
    let mut bytes = push(&[Value::Number(91.0)]);
    bytes.extend(record(0x87, &[2]));
    bytes.extend([0x17, 0x52]);
    let action = install(&mut f, &bytes);
    let previous = (
        f.vm.current,
        f.vm.current_code,
        f.vm.current_target,
        f.vm.depth,
    );
    let error =
        f.vm.execute_saved_action(action, env, &mut RejectHost)
            .unwrap_err();
    assert!(matches!(error.kind, ErrorKind::Budget(_)), "{error:?}");
    assert_eq!(error.opcode, Some(0x96));
    assert_eq!(
        error.origin,
        Some(CodeOrigin {
            code: f.callee_code,
            source_offset: f.body_offset + failed_relative,
        })
    );
    let mut expected_stack = original_stack;
    expected_stack.push(Value::Number(66.0));
    let mut expected_fixed = fixed();
    expected_fixed[2] = Value::Number(91.0);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().stack(),
        expected_stack
    );
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().registers(),
        &expected_fixed
    );
    assert_eq!(
        f.vm.saved_environment_dynamic_registers(env).unwrap(),
        &[Value::Number(55.0), Value::Undefined]
    );
    assert!(f.vm.saved_environment_halted(env).unwrap());
    assert_eq!(
        (
            f.vm.current,
            f.vm.current_code,
            f.vm.current_target,
            f.vm.depth
        ),
        previous
    );
    assert_eq!(f.vm.host_calls, 0);
    assert!(
        f.vm.execute_saved_action(action, env, &mut RejectHost)
            .is_err()
    );
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().stack(),
        expected_stack
    );
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().registers(),
        &expected_fixed
    );
    assert_eq!(
        f.vm.saved_environment_dynamic_registers(env).unwrap(),
        &[Value::Number(55.0), Value::Undefined]
    );
}

#[test]
fn shared_call_remains_gated_without_profile_or_callable_binding() {
    for enabled in [false, true] {
        let mut f = fixture(0x2a, 2, &[], &return_number(42.0));
        let stack = operands(f.receiver, &[]);
        let env = adopt(&mut f, stack.clone(), 16);
        if enabled {
            f.vm.enable_saved_script_calls(
                env,
                SavedScriptCallProfile::EmptyDynamicNoPendingException,
            )
            .unwrap();
        }
        let action = install(&mut f, &[0x52]);
        let error =
            f.vm.execute_saved_action(action, env, &mut RejectHost)
                .unwrap_err();
        assert!(matches!(error.kind, ErrorKind::Unsupported(_)), "{error:?}");
        assert_eq!(f.vm.saved_environment_state(env).unwrap().stack(), stack);
        assert_eq!(
            f.vm.saved_environment_state(env).unwrap().registers(),
            &fixed()
        );
        assert!(
            f.vm.saved_environment_dynamic_registers(env)
                .unwrap()
                .is_empty()
        );
        assert!(f.vm.saved_environment_halted(env).unwrap());
        assert_eq!(f.vm.host_calls, 0);
    }
}

#[test]
fn shared_admission_does_not_open_generic_call_or_poison_environment() {
    let mut f = fixture(0x2a, 2, &[], &return_number(42.0));
    let stack = operands(f.receiver, &[]);
    let env = adopt(&mut f, stack.clone(), 16);
    admit(&mut f, env);
    let error =
        f.vm.call(
            Value::Object(f.callable),
            Value::Object(f.receiver),
            &[],
            &mut RejectHost,
        )
        .unwrap_err();
    assert!(matches!(error.kind, ErrorKind::Unsupported(_)), "{error:?}");
    assert!(!f.vm.saved_environment_halted(env).unwrap());
    assert_eq!(f.vm.saved_environment_state(env).unwrap().stack(), stack);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().registers(),
        &fixed()
    );
    assert!(
        f.vm.saved_environment_dynamic_registers(env)
            .unwrap()
            .is_empty()
    );
    let action = install(&mut f, &[0x52]);
    complete(&mut f, action, env);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().stack(),
        &[Value::Number(17.0), Value::Number(42.0)]
    );
}

#[test]
fn shared_binding_rejects_different_captured_target_without_halting() {
    let mut f = fixture(0x2a, 2, &[], &return_number(42.0));
    let other_target = f.vm.object().unwrap();
    let (other, _, _) = define(&mut f.vm, other_target, 0x2a, 2, &[], &return_number(9.0));
    let stack = operands(f.receiver, &[]);
    let env = adopt(&mut f, stack.clone(), 16);
    f.vm.enable_saved_script_calls(env, SavedScriptCallProfile::EmptyDynamicNoPendingException)
        .unwrap();
    assert!(f.vm.bind_saved_script_environment(env, other).is_err());
    assert!(!f.vm.saved_environment_halted(env).unwrap());
    assert_eq!(f.vm.saved_environment_state(env).unwrap().stack(), stack);
    assert_eq!(
        f.vm.saved_environment_state(env).unwrap().registers(),
        &fixed()
    );
    assert!(
        f.vm.saved_environment_dynamic_registers(env)
            .unwrap()
            .is_empty()
    );
    f.vm.bind_saved_script_environment(env, f.callable).unwrap();
    let action = install(&mut f, &[0x52]);
    complete(&mut f, action, env);
}
