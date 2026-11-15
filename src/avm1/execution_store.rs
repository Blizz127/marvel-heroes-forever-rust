//! Shared execution values, separate from a frame's control and scope state.
//!
//! The owner must keep one physical store across participating calls. A new
//! store explicitly starts with no dynamic registers; admission of that initial
//! state, native call semantics, and Environment identity belong to the VM.
//! Dynamic extents model checked Rust storage only, not native allocator parity.
//! RegisterBank is a structural cursor, not a lifetime token: the VM must own
//! descriptors and leave them in lexical LIFO order, including zero-sized frames.
use super::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StoreError {
    Underflow,
    StackLimit,
    InvalidRegister(u8),
    InvalidRegisterFrame,
    RegisterLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RegisterBank {
    Fixed,
    Function2 { base: usize, count: u8 },
}

/// Deliberately not Clone: callers share the store instead of copying operands.
#[derive(Debug)]
pub(super) struct ExecutionStore {
    pub(super) stack: Vec<Value>,
    pub(super) fixed: [Value; 4],
    pub(super) dynamic: Vec<Value>,
    pub(super) max_stack_values: usize,
}

impl ExecutionStore {
    pub(super) fn new(
        stack: Vec<Value>,
        fixed: [Value; 4],
        max_stack_values: usize,
    ) -> Result<Self, StoreError> {
        if stack.len() > max_stack_values {
            return Err(StoreError::StackLimit);
        }
        Ok(Self {
            stack,
            fixed,
            dynamic: Vec::new(),
            max_stack_values,
        })
    }

    pub(super) fn push(&mut self, value: Value) -> Result<(), StoreError> {
        if self.stack.len() >= self.max_stack_values {
            return Err(StoreError::StackLimit);
        }
        self.stack.push(value);
        Ok(())
    }

    pub(super) fn pop(&mut self) -> Result<Value, StoreError> {
        self.stack.pop().ok_or(StoreError::Underflow)
    }

    pub(super) fn peek_from_top(&self, depth: usize) -> Result<&Value, StoreError> {
        let index = self
            .stack
            .len()
            .checked_sub(depth.checked_add(1).ok_or(StoreError::Underflow)?)
            .ok_or(StoreError::Underflow)?;
        Ok(&self.stack[index])
    }

    pub(super) fn len(&self) -> usize {
        self.stack.len()
    }

    pub(super) fn read_register(&self, bank: RegisterBank, index: u8) -> Result<Value, StoreError> {
        match bank {
            RegisterBank::Fixed => self
                .fixed
                .get(usize::from(index))
                .cloned()
                .ok_or(StoreError::InvalidRegister(index)),
            RegisterBank::Function2 { base, count } => {
                let offset = self.dynamic_offset(base, count, index)?;
                Ok(self.dynamic[offset].clone())
            }
        }
    }

    pub(super) fn write_register(
        &mut self,
        bank: RegisterBank,
        index: u8,
        value: Value,
    ) -> Result<(), StoreError> {
        match bank {
            RegisterBank::Fixed => {
                *self
                    .fixed
                    .get_mut(usize::from(index))
                    .ok_or(StoreError::InvalidRegister(index))? = value;
            }
            RegisterBank::Function2 { base, count } => {
                let offset = self.dynamic_offset(base, count, index)?;
                self.dynamic[offset] = value;
            }
        }
        Ok(())
    }

    fn dynamic_end(&self, base: usize, count: u8) -> Result<usize, StoreError> {
        let end = base
            .checked_add(usize::from(count))
            .ok_or(StoreError::InvalidRegisterFrame)?;
        if end != self.dynamic.len() {
            return Err(StoreError::InvalidRegisterFrame);
        }
        Ok(end)
    }

    fn dynamic_offset(&self, base: usize, count: u8, index: u8) -> Result<usize, StoreError> {
        let end = self.dynamic_end(base, count)?;
        if index >= count {
            return Err(StoreError::InvalidRegister(index));
        }
        Ok(end - usize::from(index) - 1)
    }

    pub(super) fn enter_function2(&mut self, count: u8) -> Result<RegisterBank, StoreError> {
        let base = self.dynamic.len();
        let end = base
            .checked_add(usize::from(count))
            .filter(|end| *end <= self.max_stack_values)
            .ok_or(StoreError::RegisterLimit)?;
        self.dynamic
            .try_reserve(usize::from(count))
            .map_err(|_| StoreError::RegisterLimit)?;
        self.dynamic.resize(end, Value::Undefined);
        Ok(RegisterBank::Function2 { base, count })
    }

    pub(super) fn leave_function2(&mut self, bank: RegisterBank) -> Result<(), StoreError> {
        let RegisterBank::Function2 { base, count } = bank else {
            return Err(StoreError::InvalidRegisterFrame);
        };
        self.dynamic_end(base, count)?;
        self.dynamic.truncate(base);
        Ok(())
    }

    /// Normal caller cleanup uses the current top, never the entry stack depth.
    /// The caller invokes this only after a successful callee return.
    pub(super) fn finish_call(
        &mut self,
        drop_count: usize,
        result: Value,
    ) -> Result<(), StoreError> {
        let remaining = self
            .stack
            .len()
            .checked_sub(drop_count)
            .filter(|remaining| *remaining != 0)
            .ok_or(StoreError::Underflow)?;
        self.stack.truncate(remaining);
        self.stack[remaining - 1] = result;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(numbers: &[i32]) -> Vec<Value> {
        numbers
            .iter()
            .map(|n| Value::Number(f64::from(*n)))
            .collect()
    }

    fn store(stack: &[i32], max: usize) -> ExecutionStore {
        ExecutionStore::new(
            values(stack),
            std::array::from_fn(|_| Value::Undefined),
            max,
        )
        .unwrap()
    }

    #[test]
    fn construction_and_stack_edges_preserve_supplied_values() {
        let fixed = [
            Value::Null,
            Value::Bool(false),
            Value::Number(7.0),
            Value::Undefined,
        ];
        let mut state = ExecutionStore::new(values(&[1, 2]), fixed.clone(), 2).unwrap();
        assert!(state.dynamic.is_empty());
        assert_eq!(state.fixed, fixed);
        assert_eq!(state.len(), 2);
        assert_eq!(state.peek_from_top(0), Ok(&Value::Number(2.0)));
        assert_eq!(state.peek_from_top(1), Ok(&Value::Number(1.0)));
        assert_eq!(state.peek_from_top(2), Err(StoreError::Underflow));
        assert_eq!(state.peek_from_top(usize::MAX), Err(StoreError::Underflow));
        assert_eq!(state.push(Value::Null), Err(StoreError::StackLimit));
        assert_eq!(state.stack, values(&[1, 2]));
        assert_eq!(state.pop(), Ok(Value::Number(2.0)));
        assert_eq!(state.pop(), Ok(Value::Number(1.0)));
        assert_eq!(state.pop(), Err(StoreError::Underflow));
        assert!(state.stack.is_empty());
        state.push(Value::Bool(true)).unwrap();
        assert_eq!(state.peek_from_top(0), Ok(&Value::Bool(true)));
        assert!(matches!(
            ExecutionStore::new(values(&[1]), fixed, 0),
            Err(StoreError::StackLimit)
        ));
        assert_eq!(store(&[], 0).push(Value::Null), Err(StoreError::StackLimit));
    }

    #[test]
    fn call_cleanup_retains_callee_residue_below_current_top() {
        let mut state = store(&[10, 20, 30, 40], 8);
        state.push(Value::Number(50.0)).unwrap();
        state.finish_call(2, Value::Number(99.0)).unwrap();
        assert_eq!(state.stack, values(&[10, 20, 99]));
    }

    #[test]
    fn call_cleanup_uses_diminished_stack_without_entry_floor() {
        let mut state = store(&[10, 20, 30, 40], 8);
        state.pop().unwrap();
        state.finish_call(2, Value::Number(99.0)).unwrap();
        assert_eq!(state.stack, values(&[99]));
    }

    #[test]
    fn call_cleanup_failure_is_atomic_and_zero_drop_replaces_top() {
        let mut state = store(&[10, 20], 4);
        for drop_count in [2, 3, usize::MAX] {
            assert_eq!(
                state.finish_call(drop_count, Value::Null),
                Err(StoreError::Underflow)
            );
            assert_eq!(state.stack, values(&[10, 20]));
        }
        state.finish_call(0, Value::Null).unwrap();
        assert_eq!(state.stack, vec![Value::Number(10.0), Value::Null]);
        assert_eq!(
            store(&[], 4).finish_call(0, Value::Null),
            Err(StoreError::Underflow)
        );
    }

    #[test]
    fn fixed_bank_survives_function2_entry_and_exit() {
        let mut state = store(&[], 8);
        state
            .write_register(RegisterBank::Fixed, 0, Value::Number(10.0))
            .unwrap();
        state
            .write_register(RegisterBank::Fixed, 3, Value::Number(30.0))
            .unwrap();
        let bank = state.enter_function2(4).unwrap();
        state.write_register(bank, 0, Value::Number(100.0)).unwrap();
        state
            .write_register(RegisterBank::Fixed, 3, Value::Number(31.0))
            .unwrap();
        state.leave_function2(bank).unwrap();
        assert_eq!(
            state.read_register(RegisterBank::Fixed, 0),
            Ok(Value::Number(10.0))
        );
        assert_eq!(
            state.read_register(RegisterBank::Fixed, 3),
            Ok(Value::Number(31.0))
        );
        let before = state.fixed.clone();
        for index in [4, u8::MAX] {
            assert_eq!(
                state.read_register(RegisterBank::Fixed, index),
                Err(StoreError::InvalidRegister(index))
            );
            assert_eq!(
                state.write_register(RegisterBank::Fixed, index, Value::Null),
                Err(StoreError::InvalidRegister(index))
            );
            assert_eq!(state.fixed, before);
        }
    }

    #[test]
    fn function2_exact_extent_uses_reverse_indices() {
        let mut state = store(&[], 4);
        let bank = state.enter_function2(3).unwrap();
        assert_eq!(bank, RegisterBank::Function2 { base: 0, count: 3 });
        assert_eq!(state.dynamic, vec![Value::Undefined; 3]);
        state.write_register(bank, 0, Value::Number(10.0)).unwrap();
        state.write_register(bank, 2, Value::Number(30.0)).unwrap();
        assert_eq!(
            state.dynamic,
            vec![Value::Number(30.0), Value::Undefined, Value::Number(10.0)]
        );
        assert_eq!(state.read_register(bank, 0), Ok(Value::Number(10.0)));
        assert_eq!(state.read_register(bank, 1), Ok(Value::Undefined));
        assert_eq!(state.read_register(bank, 2), Ok(Value::Number(30.0)));
        let before = state.dynamic.clone();
        for index in [3, u8::MAX] {
            assert_eq!(
                state.read_register(bank, index),
                Err(StoreError::InvalidRegister(index))
            );
            assert_eq!(
                state.write_register(bank, index, Value::Null),
                Err(StoreError::InvalidRegister(index))
            );
            assert_eq!(state.dynamic, before);
        }
        state.leave_function2(bank).unwrap();
        assert!(state.dynamic.is_empty());
    }

    #[test]
    fn nested_extents_preserve_rust_prefix_with_lifo_ownership() {
        let mut state = store(&[], 6);
        let outer = state.enter_function2(2).unwrap();
        state.write_register(outer, 0, Value::Number(10.0)).unwrap();
        state.write_register(outer, 1, Value::Number(20.0)).unwrap();
        let prefix = state.dynamic.clone();
        let inner = state.enter_function2(3).unwrap();
        assert_eq!(inner, RegisterBank::Function2 { base: 2, count: 3 });
        assert_eq!(&state.dynamic[..2], prefix.as_slice());
        assert_eq!(
            state.read_register(outer, 0),
            Err(StoreError::InvalidRegisterFrame)
        );
        let before = state.dynamic.clone();
        assert_eq!(
            state.leave_function2(outer),
            Err(StoreError::InvalidRegisterFrame)
        );
        assert_eq!(state.dynamic, before);
        state
            .write_register(inner, 0, Value::Number(100.0))
            .unwrap();
        state
            .write_register(inner, 2, Value::Number(300.0))
            .unwrap();
        assert_eq!(
            state.dynamic,
            values(&[20, 10, 300])
                .into_iter()
                .chain([Value::Undefined, Value::Number(100.0)])
                .collect::<Vec<_>>()
        );
        state.leave_function2(inner).unwrap();
        assert_eq!(state.dynamic, prefix);
        assert_eq!(state.read_register(outer, 0), Ok(Value::Number(10.0)));
        state.leave_function2(outer).unwrap();
        assert!(state.dynamic.is_empty());
    }

    #[test]
    fn zero_count_has_no_registers_and_no_physical_extent() {
        let mut state = store(&[], 2);
        let outer = state.enter_function2(2).unwrap();
        state.write_register(outer, 1, Value::Number(7.0)).unwrap();
        let before = state.dynamic.clone();
        let empty = state.enter_function2(0).unwrap();
        assert_eq!(empty, RegisterBank::Function2 { base: 2, count: 0 });
        assert_eq!(
            state.read_register(empty, 0),
            Err(StoreError::InvalidRegister(0))
        );
        assert_eq!(
            state.write_register(empty, 0, Value::Null),
            Err(StoreError::InvalidRegister(0))
        );
        state.leave_function2(empty).unwrap();
        assert_eq!(state.dynamic, before);
        state.leave_function2(outer).unwrap();
        let mut zero = store(&[], 0);
        let bank = zero.enter_function2(0).unwrap();
        zero.leave_function2(bank).unwrap();
        assert_eq!(zero.enter_function2(1), Err(StoreError::RegisterLimit));
    }

    #[test]
    fn invalid_register_frames_and_limits_do_not_mutate_values() {
        let mut state = store(&[1], 3);
        let bank = state.enter_function2(2).unwrap();
        state.write_register(bank, 1, Value::Number(7.0)).unwrap();
        let before = state.dynamic.clone();
        for invalid in [
            RegisterBank::Function2 { base: 0, count: 1 },
            RegisterBank::Function2 { base: 3, count: 0 },
            RegisterBank::Function2 {
                base: usize::MAX,
                count: 1,
            },
        ] {
            assert_eq!(
                state.read_register(invalid, 0),
                Err(StoreError::InvalidRegisterFrame)
            );
            assert_eq!(
                state.write_register(invalid, 0, Value::Null),
                Err(StoreError::InvalidRegisterFrame)
            );
            assert_eq!(
                state.leave_function2(invalid),
                Err(StoreError::InvalidRegisterFrame)
            );
            assert_eq!(state.dynamic, before);
        }
        assert_eq!(
            state.leave_function2(RegisterBank::Fixed),
            Err(StoreError::InvalidRegisterFrame)
        );
        assert_eq!(state.enter_function2(2), Err(StoreError::RegisterLimit));
        assert_eq!(state.dynamic, before);
        assert_eq!(state.stack, values(&[1]));
        let tail = state.enter_function2(1).unwrap();
        assert_eq!(state.dynamic.len(), 3);
        assert_eq!(state.enter_function2(1), Err(StoreError::RegisterLimit));
        state.leave_function2(tail).unwrap();
        state.leave_function2(bank).unwrap();
        assert_eq!(
            state.read_register(bank, 0),
            Err(StoreError::InvalidRegisterFrame)
        );
    }
}
