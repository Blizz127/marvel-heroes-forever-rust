//! Explicit named visibility binding for one adopted attached root.
//!
//! This is the bounded equal-status branch in ATTACHED_VISIBILITY_BINDING.md.
//! Policy, raw status and base-operation profile are supplied facts, never
//! defaults inferred from placement, a source movie, or desired screenshots.
//! The host prepares all fallible work before the VM commits local bit14 and
//! the prepared host closure publishes its result. Native allocator/dirty-list
//! equivalence is outside the explicitly named projection profile.

use super::super::{
    AttachedNameClass, AttachedVisibilityRequest, CodeOrigin, ErrorKind, Host, Kind, ObjectId,
    Result, Text, Value, Vm, text,
};
use super::{AttachedClipId, AttachedTarget};

/// Supplied current MovieImpl policy for the adopted root instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedVisibilityPolicy {
    pub root: AttachedClipId,
    /// Current MovieImpl+0x4310 bit11, independent of imported MovieDef identity.
    pub no_invisible_advance: bool,
}

/// Explicit normal-completion profile for the base render-node operation.
/// Every case requires stable policy/status, valid disjoint state and no reentry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedVisibilityBaseProfile {
    /// Native target has an existing node whose backing bit equals the request.
    /// The supplied backing value is not the target's local visibility bit.
    ExistingNodeBackingEqual { backing_visible: bool },
    /// Existing node and existing writable change record; the retained native
    /// branch preserves the compared status/policy fields without allocation.
    ExistingNodeChangeRecord,
    /// Caller-admitted successful allocation/render projection. This does not
    /// assert that the native allocator or dirty bookkeeping has been modeled.
    OrdinarySuccessfulAllocationProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedVisibilityInfo {
    pub profile: AttachedVisibilityBaseProfile,
    /// Last admitted write, separate from shared focus/tab status provenance.
    pub last_write: Option<CodeOrigin>,
}

impl Vm {
    fn attached_visibility_binding_phase(&self) -> Result<()> {
        if self.depth != 0 || self.saved_active || self.current_target.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility binding during execution",
            ));
        }
        Ok(())
    }

    /// Validate typed, live identities and loaded versions along the complete
    /// parent chain. This also works before execution, when there is no caller.
    fn attached_visibility_root_member(
        &self,
        id: AttachedClipId,
        root: AttachedClipId,
    ) -> Result<()> {
        self.check_attached(root)?;
        self.check_object(self.root)?;
        if !matches!(self.objects[self.root.0].kind, Kind::AttachedOwner(actual) if actual == root)
            || self.attached[root.0].owner != self.root
            || self.attached[root.0].spec.parent.is_some()
        {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility requires the adopted root",
            ));
        }
        let mut next = Some(id);
        for _ in 0..self.limits.prototype_depth {
            let Some(current) = next else {
                return self.fail(ErrorKind::Unsupported(
                    "attached visibility crosses adopted root",
                ));
            };
            self.check_attached(current)?;
            let clip = &self.attached[current.0];
            self.check_object(clip.owner)?;
            if !matches!(self.objects[clip.owner.0].kind, Kind::AttachedOwner(actual) if actual == current)
                || !matches!(self.attached_targets.get(&clip.spec.target), Some(AttachedTarget::Clip(actual)) if *actual == current)
                || clip.spec.event_state.target_flags >> 24 != 0x52
                || clip.spec.event_state.target_flags & 0x80 == 0
                // Conservative live-context profile, not a claim that the
                // native visibility setter itself tests event-wrapper bit4.
                || clip.spec.event_state.target_flags & 0x10 != 0
            {
                return self.fail(ErrorKind::Unsupported(
                    "attached visibility requires live typed target identity",
                ));
            }
            if !(7..=19).contains(&clip.environment_header_version)
                || self.loaded_movie_targets.get(&clip.owner)
                    != Some(&clip.environment_header_version)
            {
                return self.fail(ErrorKind::Unsupported(
                    "attached visibility requires known loaded target version",
                ));
            }
            if current == root {
                return Ok(());
            }
            next = clip.spec.parent;
        }
        self.fail(ErrorKind::Budget("attached visibility parent depth"))
    }

    /// Bind one explicit current movie-policy snapshot outside execution.
    /// Unsupported policy-changing routes must remain blocked while installed.
    pub fn bind_attached_visibility_policy(
        &mut self,
        root: AttachedClipId,
        no_invisible_advance: bool,
    ) -> Result<()> {
        self.attached_visibility_binding_phase()?;
        if self.attached_visibility_policy.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility policy already bound",
            ));
        }
        self.attached_visibility_root_member(root, root)?;
        self.attached_visibility_policy = Some(AttachedVisibilityPolicy {
            root,
            no_invisible_advance,
        });
        Ok(())
    }

    /// Bind this receiver's explicit base-operation profile once. Binding does
    /// not create script storage, establish host coherence, or infer raw status.
    pub fn bind_attached_visibility_profile(
        &mut self,
        id: AttachedClipId,
        profile: AttachedVisibilityBaseProfile,
    ) -> Result<()> {
        self.attached_visibility_binding_phase()?;
        self.check_attached(id)?;
        if self.attached[id.0].visibility_profile.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility profile already bound",
            ));
        }
        let Some(policy) = self.attached_visibility_policy else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility policy is unbound",
            ));
        };
        self.attached_visibility_root_member(id, policy.root)?;
        self.attached[id.0].visibility_profile = Some(profile);
        Ok(())
    }

    pub fn attached_visibility_policy(&self) -> Option<AttachedVisibilityPolicy> {
        self.attached_visibility_policy
    }

    pub fn attached_visibility_info(
        &self,
        id: AttachedClipId,
    ) -> Result<Option<AttachedVisibilityInfo>> {
        self.check_attached(id)?;
        let clip = &self.attached[id.0];
        Ok(clip
            .visibility_profile
            .map(|profile| AttachedVisibilityInfo {
                profile,
                last_write: clip.visibility_last_write,
            }))
    }

    /// Return false only for a different exact key. A visibility request either
    /// commits both admitted states or returns an explicit pre-commit boundary.
    pub(in super::super) fn attached_visibility_write(
        &mut self,
        id: AttachedClipId,
        key: &Text,
        value: Value,
        host: &mut impl Host,
    ) -> Result<bool> {
        if key.as_ref() != text("_visible").as_ref() {
            return Ok(false);
        }
        self.check_attached(id)?;
        if !self.attached_name_profile.as_ref().is_some_and(|profile| {
            profile.classify_attached_name(key) == AttachedNameClass::Standard(7)
        }) {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility requires authenticated native ID7",
            ));
        }
        let Value::Bool(visible) = value else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility requires exact Boolean",
            ));
        };
        let Some(context) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility requires active attached Environment",
            ));
        };
        let Some(policy) = self.attached_visibility_policy else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility policy is unbound",
            ));
        };
        self.attached_visibility_root_member(id, policy.root)?;
        self.check_object(context)?;
        let Kind::AttachedOwner(caller) = self.objects[context.0].kind else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility requires typed caller Environment",
            ));
        };
        self.attached_visibility_root_member(caller, policy.root)?;
        if self.attached[caller.0].owner != context {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility caller owner mismatch",
            ));
        }
        let clip = &self.attached[id.0];
        let Some(profile) = clip.visibility_profile else {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility base profile is unbound",
            ));
        };
        if matches!(profile,
            AttachedVisibilityBaseProfile::ExistingNodeBackingEqual { backing_visible }
                if backing_visible != visible)
        {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility backing equality profile mismatch",
            ));
        }
        let Some(status) = clip.focus_rect else {
            return self.fail(ErrorKind::Unsupported("attached target status is unbound"));
        };
        let desired_status = !visible && policy.no_invisible_advance;
        if (status.state.target_flags_b8 & 8 != 0) != desired_status {
            return self.fail(ErrorKind::Unsupported(
                "attached visibility changed-status branch",
            ));
        }
        let before = clip.spec.event_state.target_flags;
        let after = (before & !0x4000) | if visible { 0x4000 } else { 0 };
        let origin = self.origin();
        let request = AttachedVisibilityRequest {
            root: self.root,
            context,
            receiver: id,
            owner: clip.owner,
            target: clip.spec.target,
            expected_local: before & 0x4000 != 0,
            visible,
            origin,
        };
        self.host_budget()?;
        let commit = host
            .prepare_attached_visibility(request)
            .map_err(|message| self.error(ErrorKind::Host(message)))?;
        // No fallible operation, callback, policy change or yield may intervene.
        // Host preparation owns its independent mutable borrow until commit.
        self.attached[id.0].spec.event_state.target_flags = after;
        self.attached[id.0].visibility_last_write = origin;
        commit();
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        AttachedEventState, CodeId, ExpectedProfile, HostCall, HostCommand, Limits,
        NativeNameProfile, Property, RejectHost,
    };
    use super::super::{
        AttachedChild, AttachedClipSource, AttachedClipSpec, AttachedMember, AttachedMovieBinding,
        DisplayTargetId, LoadedMovieSourceId,
    };
    use super::*;
    use std::sync::Arc;

    fn name_profile(correct: bool) -> Arc<NativeNameProfile> {
        // Artificial names and digest callback: no retail inventory in tests.
        let mut bytes = format!(
            "MHF-ATTACHED-NAMES\t1\nbuild\t1.52.0.1700\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
            "ab".repeat(32)
        );
        for id in 0..117 {
            let alias = id <= 21 || (110..=115).contains(&id);
            let key = if id == 7 && correct {
                "_visible".to_owned()
            } else {
                format!("{}synthetic{id:03}", if alias { "_" } else { "" })
            };
            bytes.push_str(&format!("standard\t{id}\t{}\t{key}\n", u8::from(alias)));
        }
        let mut movie_clip = vec!["constructor".to_owned(), "__proto__".to_owned()];
        movie_clip.extend((0..42).map(|id| format!("movie{id:03}")));
        let mut object = vec!["constructor".to_owned()];
        object.extend((0..8).map(|id| format!("object{id:03}")));
        for (role, mut keys) in [("movieclip", movie_clip), ("object", object)] {
            keys.sort();
            for key in keys {
                bytes.push_str(&format!("{role}\t{key}\n"));
            }
        }
        Arc::new(
            NativeNameProfile::parse_verified(
                bytes.as_bytes(),
                &ExpectedProfile {
                    build: "1.52.0.1700",
                    source_sha256: [0xab; 32],
                    inventory_sha256: [0x51; 32],
                },
                |_| Ok([0x51; 32]),
            )
            .unwrap(),
        )
    }

    fn spec(target: u64, parent: Option<AttachedClipId>, local: bool) -> AttachedClipSpec {
        AttachedClipSpec {
            target: DisplayTargetId(target),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(if parent.is_some() { 11 } else { 15 }),
                sprite_id: 27,
                header_version: if parent.is_some() { 11 } else { 15 },
            },
            parent,
            event_state: AttachedEventState {
                target_flags: 0x5200_8180 | if local { 0x4000 } else { 0 },
                load_flags: 0x1357,
                source_flags: 0x2468,
            },
        }
    }

    struct Fixture {
        vm: Vm,
        root: AttachedClipId,
        caller: AttachedClipId,
        receiver: AttachedClipId,
        descendant: AttachedClipId,
        other_root: AttachedClipId,
    }

    fn fixture(local: bool, status: u32) -> Fixture {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_attached_name_profile(name_profile(true))
            .unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let root = vm.create_attached_movie_clip(spec(1, None, false)).unwrap();
        let binding = AttachedMovieBinding::LocalSourceOrImportedLocalExport;
        let caller = vm
            .create_attached_movie_clip_with_binding(spec(2, Some(root), true), binding)
            .unwrap();
        let receiver = vm
            .create_attached_movie_clip_with_binding(spec(3, Some(root), local), binding)
            .unwrap();
        let descendant = vm
            .create_attached_movie_clip_with_binding(spec(4, Some(receiver), true), binding)
            .unwrap();
        let other_root = vm.create_attached_movie_clip(spec(5, None, true)).unwrap();
        vm.set_attached_children(
            receiver,
            vec![AttachedChild::Clip {
                name: text("_visible"),
                clip: descendant,
            }],
        )
        .unwrap();
        vm.adopt_attached_root(root).unwrap();
        vm.bind_attached_target_status(receiver, status).unwrap();
        Fixture {
            vm,
            root,
            caller,
            receiver,
            descendant,
            other_root,
        }
    }

    fn bind(f: &mut Fixture, policy: bool, profile: AttachedVisibilityBaseProfile) {
        f.vm.bind_attached_visibility_policy(f.root, policy)
            .unwrap();
        f.vm.bind_attached_visibility_profile(f.receiver, profile)
            .unwrap();
    }

    fn activate(f: &mut Fixture, offset: usize) {
        f.vm.current_target = Some(f.vm.attached_owner(f.caller).unwrap());
        f.vm.current = Some((offset, 0x4f));
        f.vm.current_code = Some(CodeId(9));
    }

    struct StagedHost {
        identity: (
            ObjectId,
            ObjectId,
            AttachedClipId,
            ObjectId,
            DisplayTargetId,
        ),
        local: bool,
        parent_effective: bool,
        effective: bool,
        descendant_local: bool,
        descendant_effective: bool,
        revision: u64,
        prepares: usize,
        commits: Vec<AttachedVisibilityRequest>,
        reject: bool,
    }

    impl StagedHost {
        fn for_fixture(f: &Fixture) -> Self {
            let clip = &f.vm.attached[f.receiver.0];
            let local = clip.spec.event_state.target_flags & 0x4000 != 0;
            Self {
                identity: (
                    f.vm.root,
                    f.vm.attached[f.caller.0].owner,
                    f.receiver,
                    clip.owner,
                    clip.spec.target,
                ),
                local,
                parent_effective: false, // Hidden parent, independently supplied.
                effective: false,
                descendant_local: true,
                descendant_effective: false,
                revision: 7,
                prepares: 0,
                commits: Vec::new(),
                reject: false,
            }
        }
    }

    impl Host for StagedHost {
        fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
            panic!("visibility must not call the generic host dispatcher")
        }

        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            panic!("visibility must not call the generic host command dispatcher")
        }

        fn prepare_attached_visibility(
            &mut self,
            request: AttachedVisibilityRequest,
        ) -> std::result::Result<Box<dyn FnOnce() + '_>, String> {
            self.prepares += 1; // Attempted-work diagnostic only.
            if self.reject {
                return Err("controlled host rejection".into());
            }
            if (
                request.root,
                request.context,
                request.receiver,
                request.owner,
                request.target,
            ) != self.identity
                || request.expected_local != self.local
            {
                return Err("controlled host identity/local mismatch".into());
            }
            let next_revision = self
                .revision
                .checked_add(1)
                .ok_or("controlled revision overflow")?;
            self.commits
                .try_reserve(1)
                .map_err(|_| "controlled trace reservation failed")?;
            let next_effective = self.parent_effective && request.visible;
            let descendant_effective = next_effective && self.descendant_local;
            Ok(Box::new(move || {
                self.local = request.visible;
                self.effective = next_effective;
                self.descendant_effective = descendant_effective;
                self.revision = next_revision;
                self.commits.push(request);
            }))
        }
    }

    #[test]
    fn bindings_are_explicit_once_and_reject_execution_or_other_roots() {
        let mut f = fixture(true, 0);
        let profile = AttachedVisibilityBaseProfile::ExistingNodeChangeRecord;
        assert_eq!(f.vm.attached_visibility_policy(), None);
        assert_eq!(f.vm.attached_visibility_info(f.receiver).unwrap(), None);
        assert!(
            f.vm.bind_attached_visibility_profile(f.receiver, profile)
                .is_err()
        );
        assert!(
            f.vm.bind_attached_visibility_policy(f.other_root, false)
                .is_err()
        );
        for active in 0..3 {
            f.vm.depth = usize::from(active == 0);
            f.vm.saved_active = active == 1;
            f.vm.current_target = if active == 2 { Some(f.vm.root) } else { None };
            assert!(f.vm.bind_attached_visibility_policy(f.root, false).is_err());
            assert_eq!(f.vm.attached_visibility_policy(), None);
        }
        f.vm.depth = 0;
        f.vm.saved_active = false;
        f.vm.current_target = None;
        f.vm.bind_attached_visibility_policy(f.root, true).unwrap();
        assert!(f.vm.bind_attached_visibility_policy(f.root, false).is_err());
        assert!(
            f.vm.bind_attached_visibility_profile(f.other_root, profile)
                .is_err()
        );
        f.vm.bind_attached_visibility_profile(f.receiver, profile)
            .unwrap();
        assert!(
            f.vm.bind_attached_visibility_profile(f.receiver, profile)
                .is_err()
        );
        assert_eq!(
            f.vm.attached_visibility_policy()
                .unwrap()
                .no_invisible_advance,
            true
        );
        assert_eq!(
            f.vm.attached_visibility_info(f.receiver).unwrap(),
            Some(AttachedVisibilityInfo {
                profile,
                last_write: None,
            })
        );
        assert!(f.vm.attached_visibility_info(AttachedClipId(999)).is_err());
    }

    #[test]
    fn exact_status_matrix_preserves_other_state_and_local_descendants() {
        for profile in [
            AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
            AttachedVisibilityBaseProfile::OrdinarySuccessfulAllocationProjection,
        ] {
            for policy in [false, true] {
                for status in [false, true] {
                    for old_local in [false, true] {
                        for requested in [false, true] {
                            let word = 0x1357_91e0 | if status { 8 } else { 0 };
                            let mut f = fixture(old_local, word);
                            bind(&mut f, policy, profile);
                            let before = f.vm.attached_event_state(f.receiver).unwrap();
                            let root_before = f.vm.attached_event_state(f.root).unwrap();
                            let descendant_before =
                                f.vm.attached_event_state(f.descendant).unwrap();
                            let focus = f.vm.attached_focus_rect(f.receiver).unwrap();
                            let tab = f.vm.attached_tab_enabled(f.receiver).unwrap();
                            let counts = (f.vm.objects.len(), f.vm.properties);
                            activate(&mut f, 123);
                            let mut host = StagedHost::for_fixture(&f);
                            let result = f.vm.attached_visibility_write(
                                f.receiver,
                                &text("_visible"),
                                Value::Bool(requested),
                                &mut host,
                            );
                            let admitted = status == (!requested && policy);
                            assert_eq!(result.is_ok(), admitted);
                            assert_eq!(
                                f.vm.attached_target_status(f.receiver).unwrap(),
                                Some(word)
                            );
                            assert_eq!(f.vm.attached_focus_rect(f.receiver).unwrap(), focus);
                            assert_eq!(f.vm.attached_tab_enabled(f.receiver).unwrap(), tab);
                            assert_eq!(f.vm.attached_event_state(f.root).unwrap(), root_before);
                            assert_eq!(
                                f.vm.attached_event_state(f.descendant).unwrap(),
                                descendant_before
                            );
                            assert_eq!((f.vm.objects.len(), f.vm.properties), counts);
                            assert!(f.vm.attached_info(f.receiver).unwrap().storage.is_none());
                            assert!(
                                f.vm.attached_info(f.receiver)
                                    .unwrap()
                                    .child_snapshot_installed
                            );
                            let after = f.vm.attached_event_state(f.receiver).unwrap();
                            if admitted {
                                assert_eq!(result.unwrap(), true);
                                assert_eq!(
                                    after.target_flags & !0x4000,
                                    before.target_flags & !0x4000
                                );
                                assert_eq!(after.target_flags & 0x4000 != 0, requested);
                                assert_eq!(
                                    (after.load_flags, after.source_flags),
                                    (before.load_flags, before.source_flags)
                                );
                                assert_eq!(host.prepares, 1);
                                assert_eq!(host.commits.len(), 1);
                                assert_eq!(host.commits[0].expected_local, old_local);
                                assert_eq!(host.local, requested);
                                assert_eq!(
                                    (
                                        host.effective,
                                        host.descendant_local,
                                        host.descendant_effective
                                    ),
                                    (false, true, false)
                                );
                                assert_eq!(
                                    f.vm.attached_visibility_info(f.receiver)
                                        .unwrap()
                                        .unwrap()
                                        .last_write,
                                    Some(CodeOrigin {
                                        code: CodeId(9),
                                        source_offset: 123
                                    })
                                );
                            } else {
                                assert_eq!(after, before);
                                assert_eq!(host.prepares, 0);
                                assert!(host.commits.is_empty());
                                assert_eq!(
                                    f.vm.attached_visibility_info(f.receiver)
                                        .unwrap()
                                        .unwrap()
                                        .last_write,
                                    None
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn backing_equality_is_distinct_from_local_equality_and_repeat_is_admitted() {
        let mut f = fixture(false, 0);
        bind(
            &mut f,
            false,
            AttachedVisibilityBaseProfile::ExistingNodeBackingEqual {
                backing_visible: true,
            },
        );
        activate(&mut f, 10);
        let mut host = StagedHost::for_fixture(&f);
        assert!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut host
            )
            .is_err()
        );
        assert_eq!(host.prepares, 0);
        for offset in [11, 12] {
            activate(&mut f, offset);
            assert!(
                f.vm.attached_visibility_write(
                    f.receiver,
                    &text("_visible"),
                    Value::Bool(true),
                    &mut host
                )
                .unwrap()
            );
        }
        assert_eq!(host.commits.len(), 2);
        assert!(!host.commits[0].expected_local);
        assert!(host.commits[1].expected_local);
        assert_eq!(host.revision, 9); // Model accounting, not native dirty count.
        assert_eq!(
            f.vm.attached_visibility_info(f.receiver)
                .unwrap()
                .unwrap()
                .last_write,
            Some(CodeOrigin {
                code: CodeId(9),
                source_offset: 12
            })
        );
        let mut f = fixture(true, 0);
        bind(
            &mut f,
            false,
            AttachedVisibilityBaseProfile::ExistingNodeBackingEqual {
                backing_visible: false,
            },
        );
        activate(&mut f, 13);
        let mut host = StagedHost::for_fixture(&f);
        assert!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut host
            )
            .unwrap()
        );
        assert!(host.commits[0].expected_local);
        assert!(!host.local);
    }

    #[test]
    fn missing_wrong_or_stale_admission_never_reaches_host() {
        for fault in 0..15 {
            let mut f = fixture(true, 0);
            bind(
                &mut f,
                false,
                AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
            );
            activate(&mut f, 20);
            match fault {
                0 => f.vm.attached_name_profile = None,
                1 => f.vm.attached_name_profile = Some(name_profile(false)),
                2 => f.vm.current_target = None,
                3 => f.vm.current_target = Some(f.vm.global),
                4 => f.vm.current_target = Some(f.vm.attached[f.other_root.0].owner),
                5 => f.vm.attached_visibility_policy = None,
                6 => f.vm.attached[f.receiver.0].visibility_profile = None,
                7 => f.vm.attached[f.receiver.0].focus_rect = None,
                8 => f.vm.attached[f.receiver.0].spec.event_state.target_flags &= !0x80,
                9 => {
                    f.vm.loaded_movie_targets
                        .remove(&f.vm.attached[f.receiver.0].owner);
                }
                10 => {
                    f.vm.attached_targets
                        .remove(&f.vm.attached[f.receiver.0].spec.target);
                }
                11 => f.vm.current_target = Some(ObjectId(usize::MAX)),
                12 => f.vm.attached[f.receiver.0].spec.event_state.target_flags |= 0x10,
                13 => f.vm.attached[f.caller.0].spec.event_state.target_flags |= 0x10,
                14 => f.vm.attached[f.root.0].spec.event_state.target_flags |= 0x10,
                _ => unreachable!(),
            }
            let before = f.vm.attached_event_state(f.receiver).unwrap();
            let mut host = StagedHost::for_fixture(&f);
            assert!(
                f.vm.attached_visibility_write(
                    f.receiver,
                    &text("_visible"),
                    Value::Bool(false),
                    &mut host
                )
                .is_err(),
                "fault {fault}"
            );
            assert_eq!(f.vm.attached_event_state(f.receiver).unwrap(), before);
            assert_eq!(host.prepares, 0);
            assert!(host.commits.is_empty());
            assert_eq!(f.vm.host_calls, 0);
        }
    }

    #[test]
    fn exact_key_boolean_and_native_precedence_do_not_create_or_replace_members() {
        let mut f = fixture(true, 0);
        bind(
            &mut f,
            false,
            AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
        );
        let storage = f.vm.ensure_attached_storage(f.receiver).unwrap();
        f.vm.raw_member(
            storage,
            text("_visible"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: Some(Value::Undefined),
            },
            4,
        )
        .unwrap();
        activate(&mut f, 30);
        let mut host = StagedHost::for_fixture(&f);
        assert!(
            !f.vm
                .attached_visibility_write(
                    AttachedClipId(999),
                    &text("_Visible"),
                    Value::Null,
                    &mut host
                )
                .unwrap()
        );
        for value in [
            Value::Undefined,
            Value::Null,
            Value::Number(0.0),
            Value::string("false"),
            Value::Object(f.vm.global),
            Value::DisplayClip(f.receiver),
        ] {
            assert!(
                f.vm.attached_visibility_write(f.receiver, &text("_visible"), value, &mut host)
                    .is_err()
            );
        }
        assert_eq!(host.prepares, 0);
        assert!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut host
            )
            .unwrap()
        );
        assert!(matches!(
            f.vm.objects[storage.0].properties.get(&text("_visible")),
            Some(Property::Accessor { .. })
        ));
        assert_eq!(f.vm.own_flags(storage, &text("_visible")), 4);
        assert_eq!(
            f.vm.read_attached_member(f.receiver, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        assert!(
            f.vm.write_attached_member(f.receiver, "_visible", Value::Bool(true))
                .is_err()
        );
    }

    #[test]
    fn host_admission_budget_and_default_host_errors_preserve_raw_state_and_provenance() {
        for fault in 0..5 {
            let mut f = fixture(true, 0);
            bind(
                &mut f,
                false,
                AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
            );
            activate(&mut f, 40);
            let before = f.vm.attached_event_state(f.receiver).unwrap();
            let mut host = StagedHost::for_fixture(&f);
            match fault {
                0 => host.reject = true,
                1 => host.local = false,
                2 => host.identity.4 = DisplayTargetId(999),
                3 => host.revision = u64::MAX,
                4 => f.vm.limits.host_calls = 0,
                _ => unreachable!(),
            }
            let host_before = (
                host.local,
                host.effective,
                host.descendant_local,
                host.descendant_effective,
                host.revision,
            );
            assert!(
                f.vm.attached_visibility_write(
                    f.receiver,
                    &text("_visible"),
                    Value::Bool(false),
                    &mut host
                )
                .is_err()
            );
            assert_eq!(f.vm.attached_event_state(f.receiver).unwrap(), before);
            assert_eq!(
                f.vm.attached_visibility_info(f.receiver)
                    .unwrap()
                    .unwrap()
                    .last_write,
                None
            );
            assert_eq!(
                (
                    host.local,
                    host.effective,
                    host.descendant_local,
                    host.descendant_effective,
                    host.revision
                ),
                host_before
            );
            assert!(host.commits.is_empty());
            assert_eq!(host.prepares, usize::from(fault != 4));
        }
        let mut f = fixture(true, 0);
        bind(
            &mut f,
            false,
            AttachedVisibilityBaseProfile::OrdinarySuccessfulAllocationProjection,
        );
        activate(&mut f, 41);
        let before = f.vm.attached_event_state(f.receiver).unwrap();
        let error =
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut RejectHost,
            )
            .unwrap_err();
        assert_eq!(
            error.kind,
            ErrorKind::Host("attached visibility host is unbound".into())
        );
        assert_eq!(f.vm.attached_event_state(f.receiver).unwrap(), before);
        assert_eq!(
            f.vm.attached_visibility_info(f.receiver)
                .unwrap()
                .unwrap()
                .last_write,
            None
        );
    }

    #[test]
    fn fixed_status_blocks_hide_then_show_and_preserves_successful_provenance() {
        let mut f = fixture(true, 8);
        bind(
            &mut f,
            true,
            AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
        );
        activate(&mut f, 50);
        let mut host = StagedHost::for_fixture(&f);
        assert!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut host
            )
            .unwrap()
        );
        let before = f.vm.attached_event_state(f.receiver).unwrap();
        let provenance = f.vm.attached_visibility_info(f.receiver).unwrap();
        activate(&mut f, 51);
        assert_eq!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(true),
                &mut host
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported("attached visibility changed-status branch")
        );
        assert_eq!(host.prepares, 1);
        assert_eq!(host.commits.len(), 1);
        assert_eq!(f.vm.attached_target_status(f.receiver).unwrap(), Some(8));
        assert_eq!(f.vm.attached_event_state(f.receiver).unwrap(), before);
        assert_eq!(
            f.vm.attached_visibility_info(f.receiver).unwrap(),
            provenance
        );
        host.reject = true;
        activate(&mut f, 52);
        assert!(
            f.vm.attached_visibility_write(
                f.receiver,
                &text("_visible"),
                Value::Bool(false),
                &mut host
            )
            .is_err()
        );
        assert_eq!(host.prepares, 2);
        assert_eq!(host.commits.len(), 1);
        assert_eq!(f.vm.attached_event_state(f.receiver).unwrap(), before);
        assert_eq!(
            f.vm.attached_visibility_info(f.receiver).unwrap(),
            provenance
        );
    }

    #[test]
    fn central_bypass_guards_preserve_visibility_but_allow_independent_load_bit() {
        let mut f = fixture(true, 0);
        bind(
            &mut f,
            false,
            AttachedVisibilityBaseProfile::ExistingNodeChangeRecord,
        );
        for id in [f.root, f.receiver, f.descendant] {
            let before = f.vm.attached_event_state(id).unwrap();
            let mut changed = before;
            changed.target_flags ^= 0x4000;
            assert!(f.vm.update_attached_event_state(id, changed).is_err());
            assert_eq!(f.vm.attached_event_state(id).unwrap(), before);
            changed = before;
            changed.target_flags ^= 0x20;
            f.vm.update_attached_event_state(id, changed).unwrap();
            assert_eq!(f.vm.attached_event_state(id).unwrap(), changed);
        }
        let global = f.vm.global;
        assert!(
            f.vm.set(global, "noInvisibleAdvance", Value::Bool(true))
                .is_err()
        );
        assert!(f.vm.get(global, "noInvisibleAdvance").is_err());
        assert!(
            f.vm.delete_member(global, &text("noInvisibleAdvance"))
                .is_err()
        );
        assert_eq!(
            f.vm.attached_visibility_policy()
                .unwrap()
                .no_invisible_advance,
            false
        );
        assert!(
            f.vm.write_attached_member(f.receiver, "_visible", Value::Bool(false))
                .is_err()
        );
        assert!(f.vm.attached_info(f.receiver).unwrap().storage.is_none());
    }
}
