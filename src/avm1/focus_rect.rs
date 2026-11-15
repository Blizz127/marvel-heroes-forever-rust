//! Bounded focus/tab status for the owned build's case-3 attached MovieClip.
//!
//! Both supported members use the same target DWORD at +0xb8. The caller must
//! establish the concrete receiver, native member ID (_focusrect 17 or
//! tabEnabled 29), and exact Boolean-false assignment before calling its setter.
//! This module performs only the proved shared-word operations. Name/profile,
//! watch, conversion, dispatch and trace admission belong to the caller, as do
//! display identity and later rendering/focus consumers. Event flags at target
//! +0x6a are a different field.
//!
//! There is deliberately no default or duplicated tab state. The caller supplies
//! the actual current +0xb8 word for one live target. A focus getter None means
//! native Null; a tabEnabled getter None means native Undefined. Neither is an
//! unbound-host-state default. See `docs/ATTACHED_FOCUSRECT.md` and
//! `docs/REA_ATTACHED_TAB_ENABLED.md` for the owned-build native evidence.

/// Supplied raw +0xb8 state of one supported attached source MovieClip target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedFocusRectState {
    /// Native target DWORD at +0xb8; distinct from event flags at +0x6a.
    pub target_flags_b8: u32,
}

impl AttachedFocusRectState {
    /// Read the proved native getter result from this supplied raw state.
    ///
    /// `None` means native Null, not absent/unknown host backing state.
    /// `Some` contains the native Boolean result. An unbound target must be
    /// represented separately by the caller; it cannot be guessed as Null.
    pub fn read(&self) -> Option<bool> {
        match self.target_flags_b8 & 0x180 {
            0 => None,
            value => Some(value == 0x180),
        }
    }

    /// Apply only the admitted exact Boolean-false native assignment.
    ///
    /// Native writes at 0x1401a9408 and 0x1401a9412 clear 0x80, then set
    /// 0x100, preserving every other +0xb8 bit. This is not a setter for
    /// Boolean true or any coerced input, and does not touch event state.
    pub fn clear(&mut self) {
        self.target_flags_b8 = (self.target_flags_b8 & !0x80) | 0x100;
    }

    /// Read tabEnabled from the same supplied raw word.
    ///
    /// `None` means native Undefined, unlike the focus getter's native Null.
    /// An unbound target must be represented separately by the caller.
    pub fn read_tab_enabled(&self) -> Option<bool> {
        match self.target_flags_b8 & 0x60 {
            0 => None,
            value => Some(value == 0x60),
        }
    }

    /// Apply only an admitted exact Boolean-false tabEnabled assignment.
    /// Clears 0x20 and sets 0x40; focus bits and every other bit are preserved.
    /// This method does not perform native member admission or post-write calls.
    pub fn disable_tab(&mut self) {
        self.target_flags_b8 = (self.target_flags_b8 & !0x20) | 0x40;
    }
}

#[cfg(test)]
mod tests {
    use super::AttachedFocusRectState;

    #[test]
    fn reads_all_focus_masks_without_changing_other_bits() {
        let profiles = [
            (0, None),
            (0x80, Some(false)),
            (0x100, Some(false)),
            (0x180, Some(true)),
        ];
        for unrelated in [0, u32::MAX, 0x1357_9bdf, 0xaaaa_aaaa, 0x5555_5555] {
            for (focus_bits, expected) in profiles {
                let supplied = (unrelated & !0x180) | focus_bits;
                let state = AttachedFocusRectState {
                    target_flags_b8: supplied,
                };
                assert_eq!(state.read(), expected);
                assert_eq!(state.target_flags_b8, supplied);
            }
        }
    }

    #[test]
    fn clear_preserves_unrelated_bits_and_is_idempotent() {
        let unrelated_words = [0, u32::MAX, 0x1357_9bdf, 0xaaaa_aaaa, 0x5555_5555]
            .into_iter()
            .chain((0..32).map(|bit| 1u32 << bit));
        for unrelated in unrelated_words {
            let unrelated = unrelated & !0x180;
            for focus_bits in [0, 0x80, 0x100, 0x180] {
                let mut state = AttachedFocusRectState {
                    target_flags_b8: unrelated | focus_bits,
                };
                state.clear();
                assert_eq!(state.target_flags_b8 & !0x180, unrelated);
                assert_eq!(state.target_flags_b8 & 0x180, 0x100);
                assert_eq!(state.read(), Some(false));
                let once = state;
                state.clear();
                assert_eq!(state, once);
            }
        }
    }

    #[test]
    fn reads_all_tab_masks_without_changing_supplied_state() {
        for unrelated in [0, u32::MAX, 0x1357_9bdf, 0xaaaa_aaaa, 0x5555_5555]
            .into_iter()
            .chain((0..32).map(|bit| 1u32 << bit))
        {
            for (tab_bits, expected) in [
                (0, None),
                (0x20, Some(false)),
                (0x40, Some(false)),
                (0x60, Some(true)),
            ] {
                let supplied = (unrelated & !0x60) | tab_bits;
                let state = AttachedFocusRectState {
                    target_flags_b8: supplied,
                };
                assert_eq!(state.read_tab_enabled(), expected);
                assert_eq!(state.target_flags_b8, supplied);
            }
        }
    }

    #[test]
    fn disable_tab_preserves_unrelated_bits_and_is_idempotent() {
        for unrelated in [0, u32::MAX, 0x1357_9bdf, 0xaaaa_aaaa, 0x5555_5555]
            .into_iter()
            .chain((0..32).map(|bit| 1u32 << bit))
        {
            let unrelated = unrelated & !0x60;
            for tab_bits in [0, 0x20, 0x40, 0x60] {
                let mut state = AttachedFocusRectState {
                    target_flags_b8: unrelated | tab_bits,
                };
                let focus = state.read();
                state.disable_tab();
                assert_eq!(state.target_flags_b8 & !0x60, unrelated);
                assert_eq!(state.target_flags_b8 & 0x60, 0x40);
                assert_eq!(state.read_tab_enabled(), Some(false));
                assert_eq!(state.read(), focus);
                let once = state;
                state.disable_tab();
                assert_eq!(state, once);
            }
        }
    }

    #[test]
    fn interleaved_focus_and_tab_updates_preserve_each_other_and_all_other_bits() {
        for unrelated in [0, u32::MAX, 0x1357_9bdf, 0xaaaa_aaaa, 0x5555_5555]
            .into_iter()
            .chain((0..32).map(|bit| 1u32 << bit))
        {
            let unrelated = unrelated & !(0x180 | 0x60);
            for focus_bits in [0, 0x80, 0x100, 0x180] {
                for tab_bits in [0, 0x20, 0x40, 0x60] {
                    let original = AttachedFocusRectState {
                        target_flags_b8: unrelated | focus_bits | tab_bits,
                    };
                    let mut focus_first = original;
                    focus_first.clear();
                    assert_eq!(focus_first.read_tab_enabled(), original.read_tab_enabled());
                    assert_eq!(focus_first.target_flags_b8 & 0x60, tab_bits);
                    focus_first.disable_tab();

                    let mut tab_first = original;
                    tab_first.disable_tab();
                    assert_eq!(tab_first.read(), original.read());
                    assert_eq!(tab_first.target_flags_b8 & 0x180, focus_bits);
                    tab_first.clear();

                    assert_eq!(focus_first, tab_first);
                    assert_eq!(focus_first.target_flags_b8, unrelated | 0x100 | 0x40);
                    assert_eq!(focus_first.read(), Some(false));
                    assert_eq!(focus_first.read_tab_enabled(), Some(false));
                    tab_first.disable_tab();
                    tab_first.clear();
                    assert_eq!(tab_first, focus_first);
                }
            }
        }
    }
}
