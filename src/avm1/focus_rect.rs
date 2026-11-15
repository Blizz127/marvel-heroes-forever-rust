//! Bounded `_focusrect` state for the owned build's case-3 attached MovieClip.
//!
//! The caller must establish the concrete receiver class, native member ID 17,
//! and exact Boolean-false assignment before calling `clear`. This module
//! models only the proved target DWORD at +0xb8, described in
//! `docs/ATTACHED_FOCUSRECT.md`. It does not perform member admission, convert
//! other values, change display identity, or implement later rendering/focus
//! consumers. The existing event flags at target +0x6a are a different field.
//!
//! There is deliberately no default state. The caller supplies the actual
//! current +0xb8 word and keeps it associated with the same live target.
//! Neither fresh-target defaults nor a complete native runtime are inferred.

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
}
