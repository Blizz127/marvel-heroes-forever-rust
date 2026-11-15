//! Bounded AVM1 action scheduling, separated from action execution.
//!
//! Native evidence and limits: `docs/FRONTEND_ACTION_QUEUE.md`.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_QUEUE_ID: AtomicU64 = AtomicU64::new(1);

/// Native numeric bucket, independent of the record's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Priority(u8);

impl Priority {
    pub const COUNT: usize = 6;

    pub const fn from_raw(raw: u8) -> Option<Self> {
        if raw < Self::COUNT as u8 {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

/// Only the four recovered execution kinds are admitted here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RecordKind {
    ActionBuffer = 1,
    Event = 2,
    Function = 3,
    NativeCallback = 4,
}

impl RecordKind {
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::ActionBuffer),
            2 => Some(Self::Event),
            3 => Some(Self::Function),
            4 => Some(Self::NativeCallback),
            _ => None,
        }
    }
}

/// An opaque native u32 stamp; zero is valid, including after counter wrap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Token(pub u32);

#[derive(Debug, PartialEq, Eq)]
pub struct Record<T> {
    pub priority: Priority,
    pub kind: RecordKind,
    pub token: Token,
    pub payload: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    All,
    Token(Token),
}

/// Engineering bounds, not recovered native constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub pending_records: usize,
    pub nested_drains: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueError {
    PendingLimit,
    DrainDepthLimit,
    AllocationFailed,
    IdentityExhausted,
    WrongQueue,
    NoActiveDrain,
    DrainNotCurrent,
}

/// A failed enqueue returns ownership without changing the queue.
#[derive(Debug, PartialEq, Eq)]
pub struct EnqueueError<T> {
    pub reason: QueueError,
    pub payload: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrainHandle {
    queue: u64,
    serial: u64,
}

#[derive(Debug)]
struct DrainState {
    handle: DrainHandle,
    filter: Filter,
    priority: usize,
    observed_insertions: u32,
}

#[derive(Debug)]
struct Bucket<T> {
    records: VecDeque<Record<T>>,
    /// Index of the insertion cursor, not necessarily the tail.
    cursor: Option<usize>,
}

/// Single-root queue. No VM calls, frame decisions or object lifetime policy.
#[derive(Debug)]
pub struct ActionQueue<T> {
    identity: u64,
    limits: Limits,
    buckets: [Bucket<T>; Priority::COUNT],
    pending: usize,
    current_token: Token,
    token_counter: u32,
    insertion_counter: u32,
    drain_serial: u64,
    drains: Vec<DrainState>,
}

impl<T> ActionQueue<T> {
    pub fn new(limits: Limits) -> Result<Self, QueueError> {
        let identity = NEXT_QUEUE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| QueueError::IdentityExhausted)?;
        Ok(Self {
            identity,
            limits,
            buckets: std::array::from_fn(|_| Bucket {
                records: VecDeque::new(),
                cursor: None,
            }),
            pending: 0,
            // Concrete AS2 root constructor 0x1401d4490 initializes all three to 1.
            current_token: Token(1),
            token_counter: 1,
            insertion_counter: 1,
            drain_serial: 0,
            drains: Vec::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.pending
    }
    pub fn is_empty(&self) -> bool {
        self.pending == 0
    }
    pub fn current_token(&self) -> Token {
        self.current_token
    }
    pub fn insertion_counter(&self) -> u32 {
        self.insertion_counter
    }
    pub fn drain_depth(&self) -> usize {
        self.drains.len()
    }

    /// Allocate a stamp with the observed wrapping u32 increment. Does not install it.
    pub fn allocate_token(&mut self) -> Token {
        self.token_counter = self.token_counter.wrapping_add(1);
        Token(self.token_counter)
    }

    /// Install a stamp and return the previous stamp for explicit restoration.
    pub fn swap_token(&mut self, token: Token) -> Token {
        std::mem::replace(&mut self.current_token, token)
    }

    pub fn records(&self, priority: Priority) -> impl Iterator<Item = &Record<T>> {
        self.buckets[priority.0 as usize].records.iter()
    }

    /// Diagnostic index in the current stored order, not a native pointer.
    pub fn insertion_cursor(&self, priority: Priority) -> Option<usize> {
        self.buckets[priority.0 as usize].cursor
    }

    pub fn enqueue(
        &mut self,
        priority: Priority,
        kind: RecordKind,
        payload: T,
    ) -> Result<(), EnqueueError<T>> {
        if self.pending >= self.limits.pending_records {
            return Err(EnqueueError {
                reason: QueueError::PendingLimit,
                payload,
            });
        }
        let bucket = &mut self.buckets[priority.0 as usize];
        if bucket.records.try_reserve(1).is_err() {
            return Err(EnqueueError {
                reason: QueueError::AllocationFailed,
                payload,
            });
        }
        let at = bucket.cursor.map_or(0, |cursor| cursor + 1);
        bucket.records.insert(
            at,
            Record {
                priority,
                kind,
                token: self.current_token,
                payload,
            },
        );
        bucket.cursor = Some(at);
        self.pending += 1;
        self.insertion_counter = self.insertion_counter.wrapping_add(1);
        Ok(())
    }

    /// Start an independent scan, while retaining any outer scan's state.
    pub fn begin_drain(&mut self, filter: Filter) -> Result<DrainHandle, QueueError> {
        if self.drains.len() >= self.limits.nested_drains {
            return Err(QueueError::DrainDepthLimit);
        }
        let serial = self
            .drain_serial
            .checked_add(1)
            .ok_or(QueueError::IdentityExhausted)?;
        self.drains
            .try_reserve(1)
            .map_err(|_| QueueError::AllocationFailed)?;
        let handle = DrainHandle {
            queue: self.identity,
            serial,
        };
        self.drains.push(DrainState {
            handle,
            filter,
            priority: 0,
            observed_insertions: 0,
        });
        self.drain_serial = serial;
        Ok(handle)
    }

    fn check_current(&self, handle: DrainHandle) -> Result<(), QueueError> {
        if handle.queue != self.identity {
            return Err(QueueError::WrongQueue);
        }
        let top = self.drains.last().ok_or(QueueError::NoActiveDrain)?;
        if top.handle != handle {
            return Err(QueueError::DrainNotCurrent);
        }
        Ok(())
    }

    /// Detach before execution. The caller owns the returned record's lifetime.
    ///
    /// `None` ends this scan. Call `end_drain` before resuming its parent scan.
    /// Calling again without insertion safely returns `None`; native normal
    /// iteration does not establish that repeated-terminal-call contract.
    pub fn next(&mut self, handle: DrainHandle) -> Result<Option<Record<T>>, QueueError> {
        self.check_current(handle)?;
        let state = self.drains.last_mut().expect("validated active drain");
        if state.observed_insertions != self.insertion_counter {
            state.priority = 0;
            state.observed_insertions = self.insertion_counter;
        }
        while state.priority < Priority::COUNT {
            let bucket = &mut self.buckets[state.priority];
            let at = match state.filter {
                Filter::All => (!bucket.records.is_empty()).then_some(0),
                Filter::Token(token) => bucket.records.iter().position(|r| r.token == token),
            };
            if let Some(at) = at {
                let record = bucket.records.remove(at).expect("selected present record");
                bucket.cursor = match bucket.cursor {
                    Some(cursor) if cursor > at => Some(cursor - 1),
                    Some(cursor) if cursor == at => {
                        // Follow successor; only a removed filtered tail falls
                        // back to the predecessor. Head/tail are implicit in deque.
                        if at < bucket.records.len() {
                            Some(at)
                        } else {
                            at.checked_sub(1)
                        }
                    }
                    other => other,
                };
                self.pending -= 1;
                return Ok(Some(record));
            }
            state.priority += 1;
        }
        Ok(None)
    }

    /// Stop this scan, including an intentionally interrupted scan. Pending
    /// records, insertion cursors and the current token remain unchanged.
    pub fn end_drain(&mut self, handle: DrainHandle) -> Result<(), QueueError> {
        self.check_current(handle)?;
        self.drains.pop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(raw: u8) -> Priority {
        Priority::from_raw(raw).unwrap()
    }
    fn queue() -> ActionQueue<&'static str> {
        ActionQueue::new(Limits {
            pending_records: 32,
            nested_drains: 4,
        })
        .unwrap()
    }
    fn add(q: &mut ActionQueue<&'static str>, priority: u8, token: u32, name: &'static str) {
        let previous = q.swap_token(Token(token));
        q.enqueue(p(priority), RecordKind::ActionBuffer, name)
            .unwrap();
        q.swap_token(previous);
    }
    fn names(q: &ActionQueue<&str>, priority: u8) -> Vec<String> {
        q.records(p(priority))
            .map(|r| r.payload.to_owned())
            .collect()
    }
    fn take(q: &mut ActionQueue<&'static str>, h: DrainHandle) -> Option<&'static str> {
        q.next(h).unwrap().map(|r| r.payload)
    }

    #[test]
    fn fresh_root_and_numeric_boundaries() {
        let mut q = queue();
        assert_eq!(q.current_token(), Token(1));
        assert_eq!(q.insertion_counter(), 1);
        assert_eq!(q.allocate_token(), Token(2));
        assert_eq!(q.current_token(), Token(1));
        for n in 0..6 {
            assert_eq!(Priority::from_raw(n).unwrap().raw(), n);
        }
        assert!(Priority::from_raw(6).is_none());
        assert!(Priority::from_raw(255).is_none());
        assert!(RecordKind::from_raw(0).is_none());
        assert!(RecordKind::from_raw(5).is_none());
        for n in 1..=4 {
            assert_eq!(RecordKind::from_raw(n).unwrap() as u32, n);
        }
    }

    // Cursor seeds below are explicit native input states. The lifecycle writer
    // that repositions a cursor is not recovered and is not exposed as an API.
    #[test]
    fn filtered_middle_cursor_follows_successor_before_reentrant_insert() {
        let mut q = queue();
        for (t, s) in [(1, "A"), (2, "B"), (3, "C"), (1, "D")] {
            add(&mut q, 3, t, s);
        }
        q.buckets[3].cursor = Some(1); // A -> B(cursor) -> C -> D(tail)
        let h = q.begin_drain(Filter::Token(Token(2))).unwrap();
        assert_eq!(take(&mut q, h), Some("B"));
        assert_eq!(q.insertion_cursor(p(3)), Some(1)); // C, not A
        add(&mut q, 3, 2, "E");
        assert_eq!(names(&q, 3), ["A", "C", "E", "D"]);
        assert_eq!(q.records(p(3)).last().unwrap().payload, "D");
        assert_eq!(take(&mut q, h), Some("E"));
        assert_eq!(q.insertion_cursor(p(3)), Some(2)); // D
        assert_eq!(take(&mut q, h), None);
        q.end_drain(h).unwrap();
        assert_eq!(names(&q, 3), ["A", "C", "D"]);
        assert_eq!(q.current_token(), Token(1));
    }

    #[test]
    fn filtered_tail_cursor_falls_back_to_predecessor_and_empty_clears_it() {
        let mut q = queue();
        for (t, s) in [(1, "A"), (2, "B"), (2, "C")] {
            add(&mut q, 3, t, s);
        }
        let h = q.begin_drain(Filter::Token(Token(2))).unwrap();
        assert_eq!(take(&mut q, h), Some("B"));
        assert_eq!(q.insertion_cursor(p(3)), Some(1)); // still C
        assert_eq!(take(&mut q, h), Some("C"));
        assert_eq!(q.insertion_cursor(p(3)), Some(0)); // A is now tail/cursor
        add(&mut q, 3, 1, "D");
        assert_eq!(take(&mut q, h), None);
        q.end_drain(h).unwrap();
        let all = q.begin_drain(Filter::All).unwrap();
        assert_eq!(take(&mut q, all), Some("A"));
        assert_eq!(take(&mut q, all), Some("D"));
        assert_eq!(q.insertion_cursor(p(3)), None);
        assert!(q.records(p(3)).next().is_none());
        assert_eq!(take(&mut q, all), None);
        q.end_drain(all).unwrap();
    }

    #[test]
    fn normal_cursor_head_moves_to_successor_not_tail() {
        let mut q = queue();
        for s in ["A", "B", "C"] {
            add(&mut q, 4, 1, s);
        }
        q.buckets[4].cursor = Some(0);
        let h = q.begin_drain(Filter::All).unwrap();
        assert_eq!(take(&mut q, h), Some("A"));
        assert_eq!(q.insertion_cursor(p(4)), Some(0)); // B
        add(&mut q, 4, 1, "D");
        assert_eq!(names(&q, 4), ["B", "D", "C"]);
        assert_eq!(take(&mut q, h), Some("B"));
        assert_eq!(take(&mut q, h), Some("D"));
        add(&mut q, 4, 1, "E");
        assert_eq!(take(&mut q, h), Some("C"));
        assert_eq!(take(&mut q, h), Some("E"));
        assert_eq!(take(&mut q, h), None);
        q.end_drain(h).unwrap();
    }

    #[test]
    fn null_cursor_prepends_without_changing_old_tail() {
        let mut q = queue();
        add(&mut q, 2, 1, "A");
        add(&mut q, 2, 1, "B");
        q.buckets[2].cursor = None;
        add(&mut q, 2, 2, "X");
        assert_eq!(names(&q, 2), ["X", "A", "B"]);
        assert_eq!(q.records(p(2)).last().unwrap().payload, "B");
        let h = q.begin_drain(Filter::Token(Token(2))).unwrap();
        assert_eq!(take(&mut q, h), Some("X"));
        assert_eq!(q.insertion_cursor(p(2)), Some(0)); // A
        q.end_drain(h).unwrap();
    }

    #[test]
    fn nested_filtered_drain_preserves_outer_work_and_restarts_after_insertions() {
        let mut q = queue();
        add(&mut q, 4, 1, "frame");
        add(&mut q, 5, 1, "load");
        add(&mut q, 3, 2, "constructor");
        let outer = q.begin_drain(Filter::All).unwrap();
        let mut trace = vec![take(&mut q, outer).unwrap()];
        add(&mut q, 1, 3, "prototype");
        add(&mut q, 2, 3, "initializer");
        add(&mut q, 4, 2, "nested");
        let nested = q.begin_drain(Filter::Token(Token(2))).unwrap();
        assert_eq!(q.next(outer), Err(QueueError::DrainNotCurrent));
        trace.push(take(&mut q, nested).unwrap());
        add(&mut q, 0, 2, "urgent");
        trace.push(take(&mut q, nested).unwrap());
        assert_eq!(take(&mut q, nested), None);
        q.end_drain(nested).unwrap();
        while let Some(name) = take(&mut q, outer) {
            trace.push(name);
        }
        q.end_drain(outer).unwrap();
        assert_eq!(
            trace,
            [
                "constructor",
                "nested",
                "urgent",
                "prototype",
                "initializer",
                "frame",
                "load"
            ]
        );
        assert!(q.is_empty());
        assert_eq!(q.drain_depth(), 0);
    }

    #[test]
    fn native_u32_counter_wrap_and_zero_token_are_preserved() {
        let mut q = queue();
        q.token_counter = u32::MAX;
        let zero = q.allocate_token();
        assert_eq!(zero, Token(0));
        assert_eq!(q.swap_token(zero), Token(1));
        q.enqueue(p(4), RecordKind::Function, "A").unwrap();
        q.enqueue(p(5), RecordKind::Event, "B").unwrap();
        add(&mut q, 4, u32::MAX, "other-token");
        q.insertion_counter = u32::MAX;
        let h = q.begin_drain(Filter::Token(zero)).unwrap();
        let first = q.next(h).unwrap().unwrap();
        assert_eq!(
            (first.payload, first.kind, first.token),
            ("A", RecordKind::Function, zero)
        );
        q.enqueue(p(0), RecordKind::NativeCallback, "C").unwrap();
        assert_eq!(q.insertion_counter(), 0);
        assert_eq!(take(&mut q, h), Some("C"));
        assert_eq!(take(&mut q, h), Some("B"));
        assert_eq!(take(&mut q, h), None);
        q.end_drain(h).unwrap();
        assert_eq!(names(&q, 4), ["other-token"]);
        assert_eq!(q.records(p(4)).next().unwrap().token, Token(u32::MAX));
        assert_eq!(q.allocate_token(), Token(1));
    }

    #[test]
    fn independent_nested_hand_trace_preserves_record_retirement_order() {
        use std::{cell::RefCell, rc::Rc};
        struct Payload(&'static str, Rc<RefCell<Vec<&'static str>>>);
        impl Drop for Payload {
            fn drop(&mut self) {
                self.1.borrow_mut().push(self.0);
            }
        }
        fn select(
            q: &mut ActionQueue<Payload>,
            h: DrainHandle,
            pending: &mut Option<Record<Payload>>,
        ) -> Option<&'static str> {
            let next = q.next(h).unwrap();
            // Native sequence: detach successor, retire previous, return successor.
            drop(pending.take());
            let name = next.as_ref().map(|r| r.payload.0);
            *pending = next;
            name
        }
        let retired = Rc::new(RefCell::new(Vec::new()));
        let mut q = ActionQueue::new(Limits {
            pending_records: 16,
            nested_drains: 4,
        })
        .unwrap();
        for (priority, token, name) in [
            (2, 1, "A"),
            (2, 99, "U"),
            (2, 2, "B"),
            (2, 99, "C"),
            (3, 99, "D"),
        ] {
            q.swap_token(Token(token));
            q.enqueue(
                p(priority),
                RecordKind::Event,
                Payload(name, retired.clone()),
            )
            .unwrap_or_else(|_| panic!("fixture capacity"));
        }
        q.swap_token(Token(2));
        q.insertion_counter = 5; // explicit independent trace seed
        let outer = q.begin_drain(Filter::All).unwrap();
        let mut outer_pending = None;
        let mut nested_pending = None;
        let mut executed = vec![select(&mut q, outer, &mut outer_pending).unwrap()];
        let nested = q.begin_drain(Filter::Token(Token(2))).unwrap();
        executed.push(select(&mut q, nested, &mut nested_pending).unwrap());
        q.enqueue(p(1), RecordKind::Event, Payload("E", retired.clone()))
            .unwrap_or_else(|_| panic!("fixture capacity"));
        executed.push(select(&mut q, nested, &mut nested_pending).unwrap());
        assert_eq!(select(&mut q, nested, &mut nested_pending), None);
        q.end_drain(nested).unwrap();
        assert_eq!(*retired.borrow(), ["B", "E"]);
        assert_eq!(outer_pending.as_ref().unwrap().payload.0, "A");
        assert_eq!(
            q.records(p(2)).map(|r| r.payload.0).collect::<Vec<_>>(),
            ["U", "C"]
        );
        assert_eq!(
            q.records(p(3)).map(|r| r.payload.0).collect::<Vec<_>>(),
            ["D"]
        );
        while let Some(name) = select(&mut q, outer, &mut outer_pending) {
            executed.push(name);
        }
        q.end_drain(outer).unwrap();
        assert_eq!(executed, ["A", "B", "E", "U", "C", "D"]);
        assert_eq!(*retired.borrow(), ["B", "E", "A", "U", "C", "D"]);
        assert_eq!(q.insertion_counter(), 6);
        assert_eq!(q.current_token(), Token(2));
        assert!(q.is_empty());
    }

    #[test]
    fn limits_stale_handles_and_wrong_queue_fail_before_mutation() {
        let mut q = ActionQueue::new(Limits {
            pending_records: 1,
            nested_drains: 1,
        })
        .unwrap();
        add(&mut q, 3, 4, "A");
        let before = (
            q.len(),
            q.insertion_counter(),
            q.current_token(),
            q.insertion_cursor(p(3)),
        );
        assert_eq!(
            q.enqueue(p(2), RecordKind::Event, "returned"),
            Err(EnqueueError {
                reason: QueueError::PendingLimit,
                payload: "returned"
            })
        );
        assert_eq!(
            before,
            (
                q.len(),
                q.insertion_counter(),
                q.current_token(),
                q.insertion_cursor(p(3))
            )
        );
        let h = q.begin_drain(Filter::All).unwrap();
        assert_eq!(
            q.begin_drain(Filter::Token(Token(4))),
            Err(QueueError::DrainDepthLimit)
        );
        let mut other = queue();
        assert_eq!(other.next(h), Err(QueueError::WrongQueue));
        assert_eq!(q.drain_depth(), 1);
        assert_eq!(names(&q, 3), ["A"]);
        q.end_drain(h).unwrap();
        assert_eq!(q.next(h), Err(QueueError::NoActiveDrain));
        assert_eq!(names(&q, 3), ["A"]); // ending a scan does not clear the queue
        let newer = q.begin_drain(Filter::All).unwrap();
        assert_eq!(q.next(h), Err(QueueError::DrainNotCurrent));
        assert_eq!(take(&mut q, newer), Some("A"));
        q.end_drain(newer).unwrap();
        q.drain_serial = u64::MAX;
        assert_eq!(
            q.begin_drain(Filter::All),
            Err(QueueError::IdentityExhausted)
        );
        assert_eq!(q.drain_depth(), 0);
    }
}
