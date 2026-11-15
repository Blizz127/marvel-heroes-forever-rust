//! Persistent state for a bounded saved clip-action execution path.
//!
//! Native contracts are in `docs/SAVED_EVENT_EXECUTION.md`. These containers
//! do not install handlers or execute bytecode. The VM must validate retained
//! source bytes, object identities, context, target gates and activation rules.
//! In particular, creating this state does not make `Vm::execute` an event call.
//!
//! Action state belongs to one installation's retained source wrapper. Split
//! event keys refer to the same action ID; another installation gets another
//! state even when its source CodeId is equal. Environment state belongs to the
//! owner, not to each handler. Neither state implements Default or Clone.

use super::{AttachedClipId, CodeId, CodeOrigin, ObjectId, Text, Value};
use crate::avm1::Span;
use std::{fmt, sync::Arc};

/// VM arena handle, allocated once per installed source wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SavedActionId(pub usize);

/// File-absolute span corresponding to offset zero in one retained wrapper.
/// Code identity is independent of the receiving Environment's target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedActionSource {
    pub code: CodeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SavedMode {
    Mode3 = 3,
    Mode4 = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolUpdate {
    Installed,
    Reused,
    IgnoredDifferentOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateError {
    InvalidSourceSpan,
    EmptyEventMask,
    WrongCode,
    OffsetOutsideSource,
    PoolTooLarge,
    StackBudget,
}
impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "saved action state: {self:?}")
    }
}
impl std::error::Error for StateError {}

#[derive(Debug, PartialEq, Eq)]
pub enum PoolError<E> {
    State(StateError),
    Decode(E),
}
impl<E: fmt::Display> fmt::Display for PoolError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(error) => error.fmt(f),
            Self::Decode(error) => write!(f, "saved action pool decode: {error}"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for PoolError<E> {}

#[derive(Debug)]
pub struct SavedActionState {
    source: SavedActionSource,
    original_mask: u32,
    mode: SavedMode,
    first_pool_offset: Option<u32>,
    pool: Arc<Vec<Value>>,
}

impl SavedActionState {
    /// Create a fresh installed-wrapper state, never a fresh invocation state.
    ///
    /// The caller must have admitted the exact nonempty original action bytes
    /// and installer route. This checks span arithmetic, not source contents.
    /// A zero mask is outside this bounded installation API; it would produce
    /// no split event key. Other nonzero masks preserve their native raw mode.
    pub fn new(source: SavedActionSource, original_mask: u32) -> Result<Self, StateError> {
        if source.span.len == 0
            || source.span.len > u32::MAX as usize
            || source.span.end().is_none()
        {
            return Err(StateError::InvalidSourceSpan);
        }
        if original_mask == 0 {
            return Err(StateError::EmptyEventMask);
        }
        Ok(Self {
            source,
            original_mask,
            mode: if matches!(original_mask, 1 | 4 | 0x200 | 0x40000) {
                SavedMode::Mode4
            } else {
                SavedMode::Mode3
            },
            first_pool_offset: None,
            pool: Arc::new(Vec::new()),
        })
    }

    pub fn source(&self) -> SavedActionSource {
        self.source
    }
    pub fn original_mask(&self) -> u32 {
        self.original_mask
    }
    pub fn mode(&self) -> SavedMode {
        self.mode
    }
    /// Relative to the installed source span, not to the complete movie file.
    pub fn first_pool_offset(&self) -> Option<u32> {
        self.first_pool_offset
    }
    pub fn pool(&self) -> &Arc<Vec<Value>> {
        &self.pool
    }
    /// Native invalid pool indices produce Undefined in this saved profile.
    pub fn constant(&self, index: u16) -> Value {
        self.pool
            .get(usize::from(index))
            .cloned()
            .unwrap_or(Value::Undefined)
    }

    fn relative_offset(&self, origin: CodeOrigin) -> Result<u32, StateError> {
        if origin.code != self.source.code {
            return Err(StateError::WrongCode);
        }
        let offset = origin
            .source_offset
            .checked_sub(self.source.span.offset)
            .filter(|offset| *offset < self.source.span.len)
            .ok_or(StateError::OffsetOutsideSource)?;
        Ok(offset as u32)
    }

    /// Visit an admitted ConstantPool opcode. Only the first executed record
    /// is decoded/interned; revisits and different later offsets do no work.
    ///
    /// The VM validates the opcode boundary and original immutable bytes. Its
    /// decoder enforces string/allocation budgets before returning interned
    /// Text values. An empty pool still locks its first executed offset.
    ///
    /// This models successful native allocation only. A Rust decode/budget
    /// error leaves this state unchanged and must terminate delivery. It does
    /// not emulate the native marker's state after an allocation/parse failure,
    /// nor authorize retrying the failed delivery as a native continuation.
    pub fn visit_constant_pool<E>(
        &mut self,
        origin: CodeOrigin,
        decode: impl FnOnce() -> Result<Vec<Text>, E>,
    ) -> Result<PoolUpdate, PoolError<E>> {
        let offset = self.relative_offset(origin).map_err(PoolError::State)?;
        if let Some(first) = self.first_pool_offset {
            return Ok(if first == offset {
                PoolUpdate::Reused
            } else {
                PoolUpdate::IgnoredDifferentOffset
            });
        }
        let strings = decode().map_err(PoolError::Decode)?;
        if strings.len() > u16::MAX as usize {
            return Err(PoolError::State(StateError::PoolTooLarge));
        }
        self.pool = Arc::new(strings.into_iter().map(Value::String).collect());
        self.first_pool_offset = Some(offset);
        Ok(PoolUpdate::Installed)
    }
}

#[derive(Debug)]
pub struct SavedEnvironmentState {
    owner: AttachedClipId,
    target: ObjectId,
    stack: Vec<Value>,
    registers: [Value; 4],
    max_stack_values: usize,
}

impl SavedEnvironmentState {
    /// Adopt explicitly supplied current state for this one owner/Environment.
    /// The VM validates the coherent live owner/target/context and every Value.
    /// No fresh-Environment initialization or previous execution is inferred.
    pub fn from_supplied(
        owner: AttachedClipId,
        target: ObjectId,
        stack: Vec<Value>,
        registers: [Value; 4],
        max_stack_values: usize,
    ) -> Result<Self, StateError> {
        if stack.len() > max_stack_values {
            return Err(StateError::StackBudget);
        }
        Ok(Self {
            owner,
            target,
            stack,
            registers,
            max_stack_values,
        })
    }
    pub fn owner(&self) -> AttachedClipId {
        self.owner
    }
    /// Stable target for the bounded unchanged-target execution profile.
    pub fn target(&self) -> ObjectId {
        self.target
    }
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
    pub fn registers(&self) -> &[Value; 4] {
        &self.registers
    }
    pub fn push(&mut self, value: Value) -> Result<(), StateError> {
        if self.stack.len() >= self.max_stack_values {
            return Err(StateError::StackBudget);
        }
        self.stack.push(value);
        Ok(())
    }
    /// Individual opcodes retain responsibility for underflow/sentinel policy.
    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
    /// Reads above the fixed bank return Undefined, without allocating a slot.
    pub fn read_register(&self, index: u8) -> Value {
        self.registers
            .get(usize::from(index))
            .cloned()
            .unwrap_or(Value::Undefined)
    }
    /// Stores a supplied stack-top copy without consuming any operand.
    /// Returns false for the native ignored-index path above register three.
    pub fn store_register(&mut self, index: u8, value: Value) -> bool {
        let Some(slot) = self.registers.get_mut(usize::from(index)) else {
            return false;
        };
        *slot = value;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::vm::text;
    use std::{cell::Cell, convert::Infallible};

    fn source() -> SavedActionSource {
        SavedActionSource {
            code: CodeId(7),
            span: Span {
                offset: 1200,
                len: 100,
            },
        }
    }
    fn origin(offset: usize) -> CodeOrigin {
        CodeOrigin {
            code: CodeId(7),
            source_offset: offset,
        }
    }

    #[test]
    fn split_keys_share_first_executed_pool_but_separate_installations_do_not() {
        // Installation order and source equality are not pool identity. The
        // branch-selected first pool is deliberately later in file order.
        let mut actions = [
            SavedActionState::new(source(), 0x40001).unwrap(),
            SavedActionState::new(source(), 0x40000).unwrap(),
        ];
        let keys = [(1, SavedActionId(0)), (0x40000, SavedActionId(0))];
        assert_eq!(actions[0].source(), source());
        assert_eq!(actions[0].original_mask(), 0x40001);
        assert_eq!(actions[0].mode(), SavedMode::Mode3);
        assert_eq!(actions[1].mode(), SavedMode::Mode4);
        for mask in [1, 4, 0x200, 0x40000] {
            assert_eq!(
                SavedActionState::new(source(), mask).unwrap().mode(),
                SavedMode::Mode4
            );
        }
        let decodes = Cell::new(0);
        let action = &mut actions[keys[0].1.0];
        assert_eq!(action.constant(0), Value::Undefined);
        assert_eq!(
            action.visit_constant_pool(origin(1260), || {
                decodes.set(decodes.get() + 1);
                Ok::<_, Infallible>(vec![text("first executed")])
            }),
            Ok(PoolUpdate::Installed)
        );
        let retained = Arc::clone(action.pool());
        let action = &mut actions[keys[1].1.0];
        for (offset, expected) in [
            (1220, PoolUpdate::IgnoredDifferentOffset),
            (1260, PoolUpdate::Reused),
        ] {
            assert_eq!(
                action.visit_constant_pool(origin(offset), || {
                    decodes.set(decodes.get() + 1);
                    Err::<Vec<Text>, _>("must not decode")
                }),
                Ok(expected)
            );
        }
        assert_eq!(decodes.get(), 1);
        assert!(Arc::ptr_eq(&retained, action.pool()));
        assert_eq!(action.first_pool_offset(), Some(60));
        assert_eq!(action.constant(0), Value::string("first executed"));
        assert_eq!(action.constant(u16::MAX), Value::Undefined);
        assert_eq!(actions[1].first_pool_offset(), None);
        actions[1]
            .visit_constant_pool(origin(1220), || Ok::<_, Infallible>(Vec::new()))
            .unwrap();
        assert_eq!(actions[1].first_pool_offset(), Some(20));
        assert_eq!(actions[1].constant(0), Value::Undefined);
    }

    #[test]
    fn state_rejects_bad_provenance_and_failed_first_pool_without_publishing_it() {
        let mut action = SavedActionState::new(source(), 1).unwrap();
        for (bad, expected) in [
            (
                CodeOrigin {
                    code: CodeId(8),
                    source_offset: 1220,
                },
                StateError::WrongCode,
            ),
            (origin(1199), StateError::OffsetOutsideSource),
            (origin(1300), StateError::OffsetOutsideSource),
        ] {
            assert_eq!(
                action.visit_constant_pool(bad, || -> Result<Vec<Text>, Infallible> {
                    panic!("invalid provenance must not call decoder")
                }),
                Err(PoolError::State(expected))
            );
        }
        assert_eq!(
            action.visit_constant_pool(origin(1220), || Err::<Vec<Text>, _>("budget")),
            Err(PoolError::Decode("budget"))
        );
        assert_eq!(action.first_pool_offset(), None);
        assert!(action.pool().is_empty());
        assert!(matches!(
            SavedActionState::new(source(), 0),
            Err(StateError::EmptyEventMask)
        ));
        for span in [
            Span { offset: 0, len: 0 },
            Span {
                offset: usize::MAX,
                len: 1,
            },
        ] {
            assert!(matches!(
                SavedActionState::new(SavedActionSource { span, ..source() }, 1),
                Err(StateError::InvalidSourceSpan)
            ));
        }
    }

    #[test]
    fn successive_handlers_share_supplied_environment_operands_and_registers() {
        let original_registers = [
            Value::Number(9.0),
            Value::Null,
            Value::Undefined,
            Value::Bool(true),
        ];
        let mut env = SavedEnvironmentState::from_supplied(
            AttachedClipId(5),
            ObjectId(19),
            vec![Value::string("prior operand")],
            original_registers.clone(),
            2,
        )
        .unwrap();
        // Stand-ins for two successive handlers; this test does not execute
        // their bytecode or assert any event-delivery/activation semantics.
        env.push(Value::string("handler one")).unwrap();
        assert!(env.store_register(3, env.stack().last().unwrap().clone()));
        assert_eq!(env.stack().len(), 2);
        assert_eq!(env.push(Value::Null), Err(StateError::StackBudget));
        assert_eq!(env.pop(), Some(Value::string("handler one")));
        env.push(env.read_register(3)).unwrap();
        assert_eq!(env.pop(), Some(Value::string("handler one")));
        assert_eq!(env.pop(), Some(Value::string("prior operand")));
        assert_eq!(env.pop(), None);
        for index in 4..=u8::MAX {
            assert!(!env.store_register(index, Value::Bool(false)));
            assert_eq!(env.read_register(index), Value::Undefined);
        }
        assert_eq!(&env.registers()[..3], &original_registers[..3]);
        assert_eq!(env.read_register(3), Value::string("handler one"));
        assert_eq!(env.owner(), AttachedClipId(5));
        assert_eq!(env.target(), ObjectId(19));
        assert!(matches!(
            SavedEnvironmentState::from_supplied(
                AttachedClipId(5),
                ObjectId(19),
                vec![Value::Null],
                original_registers,
                0
            ),
            Err(StateError::StackBudget)
        ));
    }
}
