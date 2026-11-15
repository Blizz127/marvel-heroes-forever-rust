//! Bounded raw-state operations for the attached source MovieClip event path.
//!
//! These operations are based on the owned 1.52.0.1700 executable's retained
//! static evidence, summarized in `docs/ATTACHED_MOVIECLIP.md` and
//! `docs/FRONTEND_EVENT_DISPATCH.md`. They do not execute handlers, evaluate
//! member presence, drain queues, or model native resource/helper effects.
//! Callers supply the actual current fields and preserve the selected target's
//! identity across callbacks. No universal fresh-state constructor is provided:
//! construction defaults are changed by attachment and may change again during
//! callbacks. Admission alone does not establish successful event delivery.

/// Raw fields of one supported live attached source MovieClip target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedEventState {
    /// Native target DWORD at +0x6a.
    pub target_flags: u32,
    /// Native target WORD at +0x88.
    pub load_flags: u16,
    /// Native source-target WORD at +0x140.
    pub source_flags: u16,
}

/// Only the two event kinds admitted by this bounded wrapper profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedEventKind {
    /// Native event mask 0x40000; not a serialized SWF ClipEventFlags value.
    Construct,
    /// Native event mask 1; not a serialized SWF ClipEventFlags value.
    Load,
}

impl AttachedEventState {
    /// Apply only the established load-field transition in attachment caller
    /// 0x14020d8d0. Raw argument 7 bit 2 selects the alternate branch.
    ///
    /// This does not perform the rest of attachment, including resource/helper
    /// calls and the construction drain. The caller must account for their
    /// effects before evaluating subsequent state.
    pub fn apply_attachment_flag_transition(&mut self, raw_argument7: u32) {
        if raw_argument7 & 4 == 0 {
            self.load_flags &= !8;
        } else {
            self.load_flags |= 1;
        }
    }

    /// Capture the native load-check target-selection condition before the
    /// construction drain. Keep both this result and the selected target's
    /// identity; do not recompute selection after callbacks.
    pub fn capture_load_eligibility(&self) -> bool {
        self.target_flags & 0x400 != 0
    }

    /// Apply the source-field marking step to the same target selected before
    /// the drain, using its actual state after any callbacks.
    ///
    /// `predicate_present` is the actual saved-event/raw-member presence result,
    /// not a callable-value test. A present undefined or non-callable member can
    /// satisfy it. This method does not look up members or execute a predicate.
    /// The alternate raw-argument branch bypasses that predicate entirely.
    /// A false result never clears an already-set source bit 7.
    pub fn mark_load_presence(
        &mut self,
        captured_eligible: bool,
        raw_argument7: u32,
        predicate_present: bool,
    ) {
        if captured_eligible && (raw_argument7 & 4 != 0 || predicate_present) {
            self.source_flags |= 0x80;
        }
    }

    /// Evaluate the entry gates of native wrapper 0x1401aa850 in their proven
    /// order. A bit-4 rejection happens before the Load-only bit-5 clear.
    ///
    /// Retained canonical evidence `ev_d68111b3453d756458db0b3e5c41782de7710993a57d69fd26e97f2c897ace0b`
    /// in private `rea-display-lifecycle/complete.bundle.json` shows the bit-4
    /// branch at 0x1401aa872, the Load bit-5 clear at 0x1401aa88a, and the load
    /// field gates at 0x1401aa892..0x1401aa8a7. True means only that the common
    /// handler may be entered; its result and effects are outside this helper.
    pub fn admit(&mut self, event: AttachedEventKind) -> bool {
        if self.target_flags & 0x10 != 0 {
            return false;
        }
        match event {
            AttachedEventKind::Construct => true,
            AttachedEventKind::Load => {
                self.target_flags &= !0x20;
                self.load_flags & 8 != 0 || self.source_flags & 0x80 != 0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AttachedEventKind, AttachedEventState};

    #[test]
    fn attachment_transition_changes_only_its_selected_load_bit() {
        let initial = AttachedEventState {
            target_flags: 0x89ab_cdef,
            load_flags: 0xa55a,
            source_flags: 0x6789,
        };
        let mut ordinary = initial;
        ordinary.apply_attachment_flag_transition(0xffff_fffb);
        assert_eq!(ordinary.load_flags, 0xa552);
        assert_eq!(ordinary.target_flags, initial.target_flags);
        assert_eq!(ordinary.source_flags, initial.source_flags);

        let mut alternate = initial;
        alternate.apply_attachment_flag_transition(4);
        assert_eq!(alternate.load_flags, 0xa55b);
        assert_eq!(alternate.target_flags, initial.target_flags);
        assert_eq!(alternate.source_flags, initial.source_flags);
    }

    #[test]
    fn presence_uses_captured_selection_and_preserves_callback_state() {
        let mut selected = AttachedEventState {
            target_flags: 0x400,
            load_flags: 0x35,
            source_flags: 0x1234,
        };
        let captured = selected.capture_load_eligibility();
        // Stand-in for externally observed state after a callback, not a model
        // of callback execution. The selection must retain its earlier value.
        selected.target_flags = 0;
        selected.mark_load_presence(captured, 0, true);
        assert_eq!(selected.source_flags, 0x12b4);
        assert_eq!(selected.target_flags, 0);
        assert_eq!(selected.load_flags, 0x35);

        let mut not_selected = AttachedEventState {
            target_flags: 0,
            load_flags: 0,
            source_flags: 0,
        };
        let captured = not_selected.capture_load_eligibility();
        not_selected.target_flags = 0x400;
        not_selected.mark_load_presence(captured, 4, true);
        assert_eq!(not_selected.source_flags, 0);
    }

    #[test]
    fn presence_is_not_callability_and_false_does_not_clear() {
        for raw_argument7 in [0, 4] {
            for captured_eligible in [false, true] {
                for predicate_present in [false, true] {
                    let mut state = AttachedEventState {
                        target_flags: 0,
                        load_flags: 0,
                        source_flags: 0x42,
                    };
                    // The bool can represent a present non-callable member;
                    // no callable value is supplied to or needed by this API.
                    state.mark_load_presence(captured_eligible, raw_argument7, predicate_present);
                    let expected = if captured_eligible && (raw_argument7 == 4 || predicate_present)
                    {
                        0xc2
                    } else {
                        0x42
                    };
                    assert_eq!(state.source_flags, expected);

                    state.source_flags = 0xc2;
                    state.mark_load_presence(captured_eligible, raw_argument7, predicate_present);
                    assert_eq!(state.source_flags, 0xc2);
                }
            }
        }
    }

    #[test]
    fn wrapper_rejection_precedes_load_side_effect() {
        for event in [AttachedEventKind::Construct, AttachedEventKind::Load] {
            let initial = AttachedEventState {
                target_flags: 0xabcd_0030,
                load_flags: 8,
                source_flags: 0x80,
            };
            let mut state = initial;
            assert!(!state.admit(event));
            assert_eq!(state, initial);
        }
    }

    #[test]
    fn load_clears_bit_five_even_when_its_extra_gate_rejects() {
        for (load_flags, source_flags, admitted) in [
            (0x100, 0x200, false),
            (0x108, 0x200, true),
            (0x100, 0x280, true),
        ] {
            let mut state = AttachedEventState {
                target_flags: 0xabcd_0020,
                load_flags,
                source_flags,
            };
            assert_eq!(state.admit(AttachedEventKind::Load), admitted);
            assert_eq!(state.target_flags, 0xabcd_0000);
            assert_eq!(state.load_flags, load_flags);
            assert_eq!(state.source_flags, source_flags);
        }
    }

    #[test]
    fn construct_has_no_load_only_gate_or_bit_clear() {
        let initial = AttachedEventState {
            target_flags: 0x8020,
            load_flags: 0,
            source_flags: 0,
        };
        let mut state = initial;
        assert!(state.admit(AttachedEventKind::Construct));
        assert_eq!(state, initial);
    }
}
