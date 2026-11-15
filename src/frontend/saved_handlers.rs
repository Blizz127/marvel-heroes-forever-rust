//! Ordered saved Construct/onLoad bindings for one explicitly supplied owner.
//!
//! Native installation and dispatch contracts: `FRONTEND_EVENT_DISPATCH.md`
//! and `SAVED_EVENT_EXECUTION.md`. This table stores VM action handles, not
//! bytecode or freshly allocated per-delivery execution state. Split event
//! keys retain the same handle; distinct installations need distinct handles.
//! It neither invokes a handler nor resolves a dynamic event member.

use crate::avm1::vm::{AttachedClipId, AttachedEventKind};
use std::{collections::BTreeMap, fmt};

const LOAD: u32 = 1;
const CONSTRUCT: u32 = 0x40000;
const SUPPORTED_MASK: u32 = LOAD | CONSTRUCT;

/// Engineering limits, not recovered native allocation limits.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub installations: usize,
    pub references: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            installations: 4096,
            references: 8192,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    WrongOwner,
    UnsupportedNativeMask(u32),
    DuplicateInstallation,
    InstallationBudget,
    ReferenceBudget,
    AllocationFailed,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "saved handler table: {self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statistics {
    pub installations: usize,
    pub references: usize,
    pub populated_keys: usize,
}

/// `I` is the owning VM's action-arena handle, not a CodeId or source span.
///
/// The caller must establish each handle's owner and original native aggregate
/// mask from that VM. A numeric handle alone does not establish this identity.
/// No default owner, cross-VM association or automatic lifecycle is inferred.
/// Mutation during delivery is outside this bounded table/dispatcher profile.
#[derive(Debug)]
pub struct SavedHandlers<I> {
    owner: AttachedClipId,
    original_masks: BTreeMap<I, u32>,
    load: Vec<I>,
    construct: Vec<I>,
    limits: Limits,
}

impl<I: Copy + Ord> SavedHandlers<I> {
    pub fn new(owner: AttachedClipId, limits: Limits) -> Self {
        Self {
            owner,
            original_masks: BTreeMap::new(),
            load: Vec::new(),
            construct: Vec::new(),
            limits,
        }
    }
    pub fn owner(&self) -> AttachedClipId {
        self.owner
    }
    pub fn statistics(&self) -> Statistics {
        Statistics {
            installations: self.original_masks.len(),
            references: self.load.len() + self.construct.len(),
            populated_keys: usize::from(!self.load.is_empty())
                + usize::from(!self.construct.is_empty()),
        }
    }
    pub fn original_mask(&self, action: I) -> Option<u32> {
        self.original_masks.get(&action).copied()
    }

    /// Append one fresh installation to each supported native event key.
    ///
    /// The mask must come from the native loader/installed callable contract,
    /// not an unchecked serialized ClipEventFlags cast. This initial profile
    /// supports pure Load, pure Construct and their combined aggregate. The
    /// latter retains mode-3 source state in the VM even when selecting just
    /// one key; this table never derives a new execution mode from that key.
    ///
    /// A repeated action ID rejects because this API installs a fresh wrapper;
    /// installing identical source twice with two different IDs appends twice.
    /// No own/prototype lookup or callability test replaces static-map presence.
    /// Errors leave logical entries/order unchanged. Successful allocation is
    /// required; Rust allocation failures do not model native failure behavior.
    pub fn install(
        &mut self,
        action_owner: AttachedClipId,
        action: I,
        original_native_mask: u32,
    ) -> Result<(), Error> {
        if action_owner != self.owner {
            return Err(Error::WrongOwner);
        }
        if original_native_mask == 0 || original_native_mask & !SUPPORTED_MASK != 0 {
            return Err(Error::UnsupportedNativeMask(original_native_mask));
        }
        if self.original_masks.contains_key(&action) {
            return Err(Error::DuplicateInstallation);
        }
        if self.original_masks.len() >= self.limits.installations {
            return Err(Error::InstallationBudget);
        }
        let add_load = usize::from(original_native_mask & LOAD != 0);
        let add_construct = usize::from(original_native_mask & CONSTRUCT != 0);
        self.load
            .len()
            .checked_add(self.construct.len())
            .and_then(|n| n.checked_add(add_load + add_construct))
            .filter(|n| *n <= self.limits.references)
            .ok_or(Error::ReferenceBudget)?;
        // Reserve both vectors before changing semantic state. A later reserve
        // failure may change capacity, but never entries or installation data.
        self.load
            .try_reserve(add_load)
            .map_err(|_| Error::AllocationFailed)?;
        self.construct
            .try_reserve(add_construct)
            .map_err(|_| Error::AllocationFailed)?;
        self.original_masks.insert(action, original_native_mask);
        if add_load != 0 {
            self.load.push(action);
        }
        if add_construct != 0 {
            self.construct.push(action);
        }
        Ok(())
    }

    /// Presence and original insertion order only, not invocation success.
    ///
    /// Borrowing this slice excludes table mutation during the bounded delivery
    /// loop. Native reentrant map/vector mutation requires a distinct executor;
    /// do not describe this immutable borrow as the native general-case snapshot.
    /// Dynamic `onLoad`/`onConstruct` lookup must occur after saved calls return.
    pub fn handlers(&self, event: AttachedEventKind) -> Option<&[I]> {
        let entries = match event {
            AttachedEventKind::Load => &self.load,
            AttachedEventKind::Construct => &self.construct,
        };
        (!entries.is_empty()).then_some(entries.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keys_retain_shared_handles_and_original_per_key_order() {
        let owner = AttachedClipId(7);
        let mut table = SavedHandlers::new(owner, Limits::default());
        assert_eq!(table.handlers(AttachedEventKind::Load), None);
        for (id, mask) in [
            (11, LOAD),
            (12, LOAD | CONSTRUCT),
            (13, CONSTRUCT),
            (14, LOAD),
        ] {
            table.install(owner, id, mask).unwrap();
        }
        assert_eq!(
            table.handlers(AttachedEventKind::Load),
            Some([11, 12, 14].as_slice())
        );
        assert_eq!(
            table.handlers(AttachedEventKind::Construct),
            Some([12, 13].as_slice())
        );
        assert_eq!(table.original_mask(12), Some(LOAD | CONSTRUCT));
        assert_eq!(table.original_mask(99), None);
        assert_eq!(table.owner(), owner);
        assert_eq!(
            table.statistics(),
            Statistics {
                installations: 4,
                references: 5,
                populated_keys: 2
            }
        );
        // IDs 13 and 15 may refer to the same source code, but independent VM
        // installations retain separate source-wrapper pools and both append.
        table.install(owner, 15, CONSTRUCT).unwrap();
        assert_eq!(
            table.handlers(AttachedEventKind::Construct),
            Some([12, 13, 15].as_slice())
        );
    }

    #[test]
    fn rejected_installations_never_partially_publish_split_keys() {
        let owner = AttachedClipId(7);
        let mut table = SavedHandlers::new(
            owner,
            Limits {
                installations: 3,
                references: 2,
            },
        );
        table.install(owner, 1, LOAD).unwrap();
        for (other_owner, id, mask, expected) in [
            (AttachedClipId(8), 2, CONSTRUCT, Error::WrongOwner),
            (owner, 1, CONSTRUCT, Error::DuplicateInstallation),
            (owner, 2, 0, Error::UnsupportedNativeMask(0)),
            (owner, 2, 0x8, Error::UnsupportedNativeMask(0x8)),
            (
                owner,
                2,
                LOAD | 0x20000,
                Error::UnsupportedNativeMask(LOAD | 0x20000),
            ),
            (owner, 2, LOAD | CONSTRUCT, Error::ReferenceBudget),
        ] {
            assert_eq!(table.install(other_owner, id, mask), Err(expected));
            assert_eq!(
                table.handlers(AttachedEventKind::Load),
                Some([1].as_slice())
            );
            assert_eq!(table.handlers(AttachedEventKind::Construct), None);
            assert_eq!(table.original_mask(2), None);
            assert_eq!(
                table.statistics(),
                Statistics {
                    installations: 1,
                    references: 1,
                    populated_keys: 1
                }
            );
        }
        table.install(owner, 2, CONSTRUCT).unwrap();
        assert_eq!(
            table.handlers(AttachedEventKind::Construct),
            Some([2].as_slice())
        );
        let mut empty = SavedHandlers::new(
            owner,
            Limits {
                installations: 0,
                references: 4,
            },
        );
        assert_eq!(
            empty.install(owner, 1, LOAD),
            Err(Error::InstallationBudget)
        );
        assert_eq!(empty.handlers(AttachedEventKind::Load), None);
    }
}
