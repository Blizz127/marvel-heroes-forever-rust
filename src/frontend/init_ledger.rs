//! Per-AVM-instance initializer tag submission, not bytecode execution.
//!
//! Native observations and bounded failure policy: `docs/FRONTEND_INIT_LEDGER.md`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub frames: u32,
    pub tags_per_frame: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateError {
    FrameLimit { requested: u32, limit: u32 },
    AllocationFailed,
}

/// Lookup failure/unpublished frames differ from an available empty list.
#[derive(Clone, Copy, Debug)]
pub enum TagList<'a, T> {
    Unavailable,
    Available(&'a [T]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub tag_count: u32,
    /// Dispatch callbacks that returned Ok, including valid no-op dispatches.
    pub completed: u32,
    /// Includes a callback that returned Err or did not return normally.
    pub attempted: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantineReason {
    DispatchFailed,
    /// Installed before calling external dispatch code; retained if it unwinds.
    InterruptedDispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quarantine {
    pub frame: u32,
    pub progress: Progress,
    pub reason: QuarantineReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameState {
    Eligible,
    Submitted { tag_count: u32 },
    Quarantined(Quarantine),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    AlreadySubmitted { tag_count: u32 },
    Unavailable,
    Empty,
    Submitted { tag_count: u32 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubmitError<LookupError, DispatchError> {
    FrameOutOfRange {
        frame: u32,
        capacity: u32,
    },
    Quarantined(Quarantine),
    Lookup(LookupError),
    TagLimit {
        count: usize,
        limit: u32,
    },
    Dispatch {
        error: DispatchError,
        quarantine: Quarantine,
    },
}

/// Own one ledger per concrete AVM display instance. No global symbol/code keys.
#[derive(Debug)]
pub struct InitLedger {
    submitted: Vec<Option<u32>>,
    capacity: u32,
    tag_limit: u32,
    quarantine: Option<Quarantine>,
}

impl InitLedger {
    pub fn new(capacity: u32, limits: Limits) -> Result<Self, CreateError> {
        if capacity > limits.frames {
            return Err(CreateError::FrameLimit {
                requested: capacity,
                limit: limits.frames,
            });
        }
        let size = usize::try_from(capacity).map_err(|_| CreateError::AllocationFailed)?;
        let mut submitted = Vec::new();
        submitted
            .try_reserve_exact(size)
            .map_err(|_| CreateError::AllocationFailed)?;
        submitted.resize(size, None);
        Ok(Self {
            submitted,
            capacity,
            tag_limit: limits.tags_per_frame,
            quarantine: None,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Engineering execution/recovery gate; partial effects are not rolled back.
    pub fn quarantine(&self) -> Option<Quarantine> {
        self.quarantine
    }

    /// Frame-local state. Any quarantine also blocks submission to other frames.
    pub fn state(&self, frame: u32) -> Option<FrameState> {
        let index = usize::try_from(frame).ok()?;
        let entry = self.submitted.get(index)?;
        if let Some(q) = self.quarantine.filter(|q| q.frame == frame) {
            return Some(FrameState::Quarantined(q));
        }
        Some(match entry {
            Some(tag_count) => FrameState::Submitted {
                tag_count: *tag_count,
            },
            None => FrameState::Eligible,
        })
    }

    /// Look up and dispatch an initializer list in stored order.
    ///
    /// Lookup is read-only and is skipped for an already submitted frame.
    /// Dispatch receives a zero-based tag index and must submit/accept that tag,
    /// without draining bytecode. A no-op tag can legitimately return Ok.
    ///
    /// A dispatch error may follow partial external effects, even on the first
    /// callback. Such an error quarantines this instance. There is no automatic
    /// retry or reset operation. Unwinding dispatch also leaves it quarantined.
    pub fn submit_with<'a, T: 'a, L, D>(
        &mut self,
        frame: u32,
        lookup: impl FnOnce(u32) -> Result<TagList<'a, T>, L>,
        mut dispatch: impl FnMut(u32, &'a T) -> Result<(), D>,
    ) -> Result<Outcome, SubmitError<L, D>> {
        if frame >= self.capacity {
            return Err(SubmitError::FrameOutOfRange {
                frame,
                capacity: self.capacity,
            });
        }
        if let Some(q) = self.quarantine {
            return Err(SubmitError::Quarantined(q));
        }
        let index = frame as usize; // constructor admitted this u32 capacity as usize
        if let Some(tag_count) = self.submitted[index] {
            return Ok(Outcome::AlreadySubmitted { tag_count });
        }
        let tags = match lookup(frame).map_err(SubmitError::Lookup)? {
            TagList::Unavailable => return Ok(Outcome::Unavailable),
            TagList::Available(tags) if tags.is_empty() => return Ok(Outcome::Empty),
            TagList::Available(tags) => tags,
        };
        let tag_count = u32::try_from(tags.len()).map_err(|_| SubmitError::TagLimit {
            count: tags.len(),
            limit: self.tag_limit,
        })?;
        if tag_count > self.tag_limit {
            return Err(SubmitError::TagLimit {
                count: tags.len(),
                limit: self.tag_limit,
            });
        }
        let mut progress = Progress {
            tag_count,
            completed: 0,
            attempted: 0,
        };
        for (tag_index, tag) in tags.iter().enumerate() {
            progress.attempted = tag_index as u32 + 1;
            self.quarantine = Some(Quarantine {
                frame,
                progress,
                reason: QuarantineReason::InterruptedDispatch,
            });
            if let Err(error) = dispatch(tag_index as u32, tag) {
                let quarantine = Quarantine {
                    frame,
                    progress,
                    reason: QuarantineReason::DispatchFailed,
                };
                self.quarantine = Some(quarantine);
                return Err(SubmitError::Dispatch { error, quarantine });
            }
            progress.completed += 1;
        }
        // The mark follows every tag dispatch and precedes any external executor.
        self.submitted[index] = Some(tag_count);
        self.quarantine = None;
        Ok(Outcome::Submitted { tag_count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    type TestResult = Result<Outcome, SubmitError<&'static str, &'static str>>;

    fn ledger(frames: u32) -> InitLedger {
        InitLedger::new(
            frames,
            Limits {
                frames: 8,
                tags_per_frame: 4,
            },
        )
        .unwrap()
    }
    fn submit(
        ledger: &mut InitLedger,
        frame: u32,
        input: TagList<'_, u8>,
        out: &mut Vec<u8>,
    ) -> TestResult {
        ledger.submit_with(
            frame,
            |_| Ok(input),
            |_, tag| {
                out.push(*tag);
                Ok(())
            },
        )
    }

    #[test]
    fn unavailable_then_nonempty_then_repeat_skips_lookup() {
        let mut ledger = ledger(1);
        let getter_calls = Cell::new(0);
        let mut submitted = Vec::new();
        let unavailable: TestResult = ledger.submit_with(
            0,
            |frame| {
                assert_eq!(frame, 0);
                getter_calls.set(getter_calls.get() + 1);
                Ok(TagList::<u8>::Unavailable)
            },
            |_, tag| {
                submitted.push(*tag);
                Ok(())
            },
        );
        assert_eq!(unavailable, Ok(Outcome::Unavailable));
        assert_eq!(ledger.state(0), Some(FrameState::Eligible));
        let ready: TestResult = ledger.submit_with(
            0,
            |_| {
                getter_calls.set(getter_calls.get() + 1);
                Ok(TagList::Available(&[9, 4]))
            },
            |index, tag| {
                assert_eq!(index as usize, submitted.len());
                submitted.push(*tag);
                Ok(())
            },
        );
        assert_eq!(ready, Ok(Outcome::Submitted { tag_count: 2 }));
        let repeat: TestResult = ledger.submit_with::<u8, _, _>(
            0,
            |_| panic!("submitted frame must skip lookup"),
            |_, _| panic!("submitted frame must skip dispatch"),
        );
        assert_eq!(repeat, Ok(Outcome::AlreadySubmitted { tag_count: 2 }));
        assert_eq!(getter_calls.get(), 2);
        assert_eq!(submitted, [9, 4]);
        assert_eq!(ledger.quarantine(), None);
    }

    #[test]
    fn empty_is_retryable_but_nonempty_noop_dispatch_marks() {
        let mut ledger = ledger(1);
        let mut submitted = Vec::new();
        assert_eq!(
            submit(&mut ledger, 0, TagList::Available(&[]), &mut submitted),
            Ok(Outcome::Empty)
        );
        assert_eq!(ledger.state(0), Some(FrameState::Eligible));
        let calls = Cell::new(0);
        let result: TestResult = ledger.submit_with(
            0,
            |_| Ok(TagList::Available(&[0, 0])),
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        );
        assert_eq!(result, Ok(Outcome::Submitted { tag_count: 2 }));
        assert_eq!(calls.get(), 2);
        assert!(submitted.is_empty()); // accepting a tag need not enqueue a buffer
        assert_eq!(
            ledger.state(0),
            Some(FrameState::Submitted { tag_count: 2 })
        );
    }

    #[test]
    fn ordered_tag_submission_is_marked_before_later_bytecode_execution() {
        let mut ledger = ledger(1);
        let mut pending = Vec::new();
        let mut executed = Vec::new();
        assert_eq!(
            submit(&mut ledger, 0, TagList::Available(&[5, 1, 8]), &mut pending),
            Ok(Outcome::Submitted { tag_count: 3 })
        );
        assert_eq!(pending, [5, 1, 8]);
        assert!(executed.is_empty());
        // Independent executor stand-in: the ledger never runs these actions.
        for action in pending {
            assert_eq!(
                ledger.state(0),
                Some(FrameState::Submitted { tag_count: 3 })
            );
            executed.push(action);
        }
        assert_eq!(executed, [5, 1, 8]);
    }

    #[test]
    fn each_instance_and_each_frame_has_its_own_ledger_entry() {
        let mut first = ledger(2);
        let mut second = ledger(2);
        let tags = [7];
        let mut effects = Vec::new();
        for (owner, instance, frame) in [(11, &mut first, 0), (22, &mut second, 0)] {
            let result: TestResult = instance.submit_with(
                frame,
                |_| Ok(TagList::Available(&tags)),
                |_, tag| {
                    effects.push((owner, frame, *tag));
                    Ok(())
                },
            );
            assert_eq!(result, Ok(Outcome::Submitted { tag_count: 1 }));
        }
        let next_frame: TestResult = first.submit_with(
            1,
            |_| Ok(TagList::Available(&tags)),
            |_, tag| {
                effects.push((11, 1, *tag));
                Ok(())
            },
        );
        assert_eq!(next_frame, Ok(Outcome::Submitted { tag_count: 1 }));
        assert_eq!(effects, [(11, 0, 7), (22, 0, 7), (11, 1, 7)]);
        assert_eq!(second.state(1), Some(FrameState::Eligible));
    }

    #[test]
    fn preflight_failures_leave_the_entry_eligible_without_tag_effects() {
        let mut ledger = ledger(1);
        let missing: TestResult = ledger.submit_with::<u8, _, _>(
            0,
            |_| Err("lookup failed"),
            |_, _| panic!("lookup failed before dispatch"),
        );
        assert_eq!(missing, Err(SubmitError::Lookup("lookup failed")));
        let mut effects = Vec::new();
        assert_eq!(
            submit(
                &mut ledger,
                0,
                TagList::Available(&[1, 2, 3, 4, 5]),
                &mut effects
            ),
            Err(SubmitError::TagLimit { count: 5, limit: 4 })
        );
        let outside: TestResult = ledger.submit_with::<u8, _, _>(
            1,
            |_| panic!("invalid frame must not reach lookup"),
            |_, _| panic!("invalid frame must not reach dispatch"),
        );
        assert_eq!(
            outside,
            Err(SubmitError::FrameOutOfRange {
                frame: 1,
                capacity: 1
            })
        );
        assert_eq!(ledger.state(0), Some(FrameState::Eligible));
        assert_eq!(ledger.state(1), None);
        assert_eq!(ledger.quarantine(), None);
        assert!(effects.is_empty());
        assert_eq!(
            submit(&mut ledger, 0, TagList::Available(&[4]), &mut effects),
            Ok(Outcome::Submitted { tag_count: 1 })
        );
    }

    #[test]
    fn partial_failure_retains_effects_and_quarantines_without_mark_or_retry() {
        let mut ledger = ledger(2);
        let mut effects = Vec::new();
        let result: TestResult = ledger.submit_with(
            0,
            |_| Ok(TagList::Available(&[1, 2, 3])),
            |index, tag| {
                effects.push(*tag); // failing callback may already have side effects
                if index == 1 {
                    Err("queue limit")
                } else {
                    Ok(())
                }
            },
        );
        let q = Quarantine {
            frame: 0,
            progress: Progress {
                tag_count: 3,
                completed: 1,
                attempted: 2,
            },
            reason: QuarantineReason::DispatchFailed,
        };
        assert_eq!(
            result,
            Err(SubmitError::Dispatch {
                error: "queue limit",
                quarantine: q
            })
        );
        assert_eq!(effects, [1, 2]);
        assert_eq!(ledger.state(0), Some(FrameState::Quarantined(q)));
        assert_eq!(ledger.submitted[0], None);
        for frame in [0, 1] {
            let blocked: TestResult = ledger.submit_with::<u8, _, _>(
                frame,
                |_| panic!("quarantine must stop lookup"),
                |_, _| panic!("must not retry"),
            );
            assert_eq!(blocked, Err(SubmitError::Quarantined(q)));
        }
        assert_eq!(effects, [1, 2]);
    }

    #[test]
    fn an_unwinding_dispatch_leaves_an_inspectable_quarantine() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let mut ledger = ledger(1);
        let mut effects = Vec::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _: TestResult = ledger.submit_with(
                0,
                |_| Ok(TagList::Available(&[1, 2, 3])),
                |index, tag| {
                    effects.push(*tag);
                    assert_ne!(index, 1, "chosen interrupted dispatch");
                    Ok(())
                },
            );
        }));
        assert!(result.is_err());
        let q = Quarantine {
            frame: 0,
            progress: Progress {
                tag_count: 3,
                completed: 1,
                attempted: 2,
            },
            reason: QuarantineReason::InterruptedDispatch,
        };
        assert_eq!(effects, [1, 2]);
        assert_eq!(ledger.quarantine(), Some(q));
        assert_eq!(ledger.submitted[0], None);
        let retry: TestResult = ledger.submit_with::<u8, _, _>(
            0,
            |_| panic!("interrupted submission cannot retry"),
            |_, _| Ok(()),
        );
        assert_eq!(retry, Err(SubmitError::Quarantined(q)));
    }

    #[test]
    fn capacity_is_fixed_and_checked_before_allocation_or_lookup() {
        let limits = Limits {
            frames: 2,
            tags_per_frame: 1,
        };
        assert!(matches!(
            InitLedger::new(u32::MAX, limits),
            Err(CreateError::FrameLimit {
                requested: u32::MAX,
                limit: 2
            })
        ));
        let mut empty = InitLedger::new(0, limits).unwrap();
        assert_eq!(empty.capacity(), 0);
        let result: TestResult = empty.submit_with::<u8, _, _>(
            0,
            |_| panic!("empty capacity cannot look up a frame"),
            |_, _| Ok(()),
        );
        assert_eq!(
            result,
            Err(SubmitError::FrameOutOfRange {
                frame: 0,
                capacity: 0
            })
        );
    }
}
