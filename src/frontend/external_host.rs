//! Explicit current-movie routing for the verified frontend callback profile.
//! Native player creation, handler installation and script lifecycle are inputs.

use super::localized_callback::LocalizationCallback;
use crate::avm1::vm::{AttachedClipId, HostCall, HostResult, ObjectId, Value, Vm};
use std::collections::{BTreeMap, BTreeSet};

pub const CALL_BINDING: u32 = 0x4d48_0010;

/// Identity in the caller's explicit native-object domain, not an AVM ObjectId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativeObjectId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerAssociation {
    pub owner: NativeObjectId,
    pub receiver: Option<NativeObjectId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::vm::{
        self, AttachedClipSource, AttachedClipSpec, AttachedEventState, DisplayTargetId,
        LoadedMovieSourceId,
    };

    fn clip(vm: &mut Vm, target: u64, parent: Option<AttachedClipId>) -> AttachedClipId {
        vm.create_attached_movie_clip(AttachedClipSpec {
            target: DisplayTargetId(target),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(100),
                sprite_id: 0,
                header_version: 15,
            },
            parent,
            // Controlled supplied state; not a native fresh-instance assertion.
            event_state: AttachedEventState {
                target_flags: 0x5200_0000,
                load_flags: 0,
                source_flags: 0,
            },
        })
        .unwrap()
    }
    fn fixture() -> (Vm, AttachedClipId, AttachedClipId, ExternalHost) {
        let mut vm = Vm::new(15, vm::Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let root = clip(&mut vm, 1000, None);
        let child = clip(&mut vm, 2000, Some(root));
        vm.adopt_attached_root(root).unwrap();
        let host = ExternalHost::new(
            &vm,
            root,
            PlayerAssociation {
                owner: NativeObjectId(1),
                receiver: None,
            },
            HandlerState::Absent,
            Limits::default(),
        )
        .unwrap();
        (vm, root, child, host)
    }
    fn call(context: Option<ObjectId>, this: Value, args: &[Value]) -> HostCall<'_> {
        HostCall {
            origin: None,
            context,
            binding: CALL_BINDING,
            this,
            args,
            source_offset: None,
        }
    }
    fn is_undefined(result: Result<HostResult, String>) -> bool {
        matches!(result, Ok(HostResult::Value(Value::Undefined)))
    }

    #[test]
    fn current_target_is_required_even_with_valid_this_and_absent_handler() {
        let (vm, _, child, mut host) = fixture();
        let root = vm.root();
        let child_owner = vm.attached_owner(child).unwrap();
        assert!(
            host.handle_call(&call(None, Value::Object(root), &[]))
                .unwrap()
                .is_err()
        );
        assert!(
            host.handle_call(&call(Some(child_owner), Value::Object(root), &[]))
                .unwrap()
                .is_err()
        );
        host.bind_context(&vm, child).unwrap();
        assert!(is_undefined(
            host.handle_call(&call(Some(child_owner), Value::Null, &[]))
                .unwrap()
        ));
        assert!(!host.available(Some(child_owner)).unwrap());
        host.set_handler(HandlerState::Frontend);
        assert!(host.available(Some(child_owner)).unwrap());
        assert!(
            host.handle_call(&call(Some(child_owner), Value::Null, &[]))
                .unwrap()
                .is_err()
        );
        let args = [
            Value::string("GetLocalizedObjectDataByLabel"),
            Value::string("Frontend/UIButtonLogIn"),
        ];
        assert!(is_undefined(
            host.handle_call(&call(Some(child_owner), Value::Null, &args))
                .unwrap()
        ));
        host.set_receiver(Some(NativeObjectId(77)));
        assert!(
            host.handle_call(&call(Some(root), Value::Null, &args))
                .unwrap()
                .is_err()
        );
    }

    #[test]
    fn rejects_other_roots_and_budget_overflow_without_publishing_context() {
        let (mut vm, root, child, mut host) = fixture();
        let other_root = clip(&mut vm, 3000, None);
        assert!(host.bind_context(&vm, other_root).is_err());
        assert!(
            host.available(Some(vm.attached_owner(other_root).unwrap()))
                .is_err()
        );
        assert!(
            ExternalHost::new(
                &vm,
                child,
                host.association(),
                HandlerState::Absent,
                Limits::default()
            )
            .is_err()
        );
        let mut limited = ExternalHost::new(
            &vm,
            root,
            host.association(),
            HandlerState::Absent,
            Limits {
                max_contexts: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        assert!(limited.bind_context(&vm, child).is_err());
        assert!(
            limited
                .available(Some(vm.attached_owner(child).unwrap()))
                .is_err()
        );
        limited.bind_context(&vm, root).unwrap();
    }

    #[test]
    fn new_load_preserves_receiver_override_and_existing_snapshot_stays_explicit() {
        let owner = NativeObjectId(1);
        let override_receiver = NativeObjectId(2);
        assert_eq!(
            PlayerAssociation::after_successful_new_load(owner, None).receiver,
            Some(owner)
        );
        assert_eq!(
            PlayerAssociation::after_successful_new_load(owner, Some(override_receiver)).receiver,
            Some(override_receiver)
        );
        let (_, _, _, mut host) = fixture();
        assert_eq!(host.association().receiver, None);
        host.set_receiver(Some(override_receiver));
        host.set_handler(HandlerState::Frontend);
        assert_eq!(
            host.association(),
            PlayerAssociation {
                owner,
                receiver: Some(override_receiver)
            }
        );
        host.set_receiver(None);
        assert!(host.available(Some(host.root())).unwrap());
        assert_eq!(host.association().receiver, None);
    }
}
impl PlayerAssociation {
    /// Selected successful NEW-load transition only. Existing-wrapper Start
    /// does not perform this assignment; callers retain its supplied snapshot.
    pub fn after_successful_new_load(
        owner: NativeObjectId,
        previous_receiver: Option<NativeObjectId>,
    ) -> Self {
        Self {
            owner,
            receiver: Some(previous_receiver.unwrap_or(owner)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlerState {
    Absent,
    /// Explicit installed engine callback and its checked frontend profile.
    Frontend,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_contexts: usize,
    pub max_receivers: usize,
    pub max_parent_depth: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_contexts: 4096,
            max_receivers: 16,
            max_parent_depth: 256,
        }
    }
}

pub struct ExternalHost {
    root_clip: AttachedClipId,
    root: ObjectId,
    contexts: BTreeSet<ObjectId>,
    association: PlayerAssociation,
    handler: HandlerState,
    receivers: BTreeMap<NativeObjectId, LocalizationCallback>,
    limits: Limits,
}

impl ExternalHost {
    /// Bind one explicitly supplied attached root in this VM. Numeric IDs must
    /// never be reused across VMs. No source filename or `this` selects a movie.
    pub fn new(
        vm: &Vm,
        root_clip: AttachedClipId,
        association: PlayerAssociation,
        handler: HandlerState,
        limits: Limits,
    ) -> Result<Self, String> {
        if limits.max_contexts == 0
            || limits.max_contexts > 100_000
            || limits.max_receivers == 0
            || limits.max_receivers > 4096
            || limits.max_parent_depth == 0
            || limits.max_parent_depth > 4096
        {
            return Err("invalid external-interface routing budgets".into());
        }
        let info = vm.attached_info(root_clip).map_err(|e| e.to_string())?;
        if info.parent.is_some() || info.owner != vm.root() {
            return Err("external-interface root must be the adopted attached VM root".into());
        }
        Ok(Self {
            root_clip,
            root: info.owner,
            contexts: BTreeSet::from([info.owner]),
            association,
            handler,
            receivers: BTreeMap::new(),
            limits,
        })
    }
    pub fn root(&self) -> ObjectId {
        self.root
    }
    pub fn association(&self) -> PlayerAssociation {
        self.association
    }
    pub fn handler(&self) -> HandlerState {
        self.handler
    }
    pub fn set_handler(&mut self, handler: HandlerState) {
        self.handler = handler;
    }
    pub fn set_receiver(&mut self, receiver: Option<NativeObjectId>) {
        self.association.receiver = receiver;
    }

    /// The supported case-3 attachment tree has immutable parents. Validate
    /// that this explicit context reaches the exact root before publishing it.
    /// Definition-movie provenance does not replace this relationship.
    pub fn bind_context(&mut self, vm: &Vm, clip: AttachedClipId) -> Result<(), String> {
        if vm.root() != self.root {
            return Err("external-interface VM root changed".into());
        }
        let owner = vm.attached_owner(clip).map_err(|e| e.to_string())?;
        let mut current = clip;
        for _ in 0..self.limits.max_parent_depth {
            let info = vm.attached_info(current).map_err(|e| e.to_string())?;
            if current == self.root_clip {
                if info.parent.is_some() || info.owner != self.root {
                    return Err("external-interface attached root changed".into());
                }
                if !self.contexts.contains(&owner)
                    && self.contexts.len() >= self.limits.max_contexts
                {
                    return Err("external-interface context budget exceeded".into());
                }
                self.contexts.insert(owner);
                return Ok(());
            }
            current = info
                .parent
                .ok_or("external context belongs to another attached root")?;
        }
        Err("external-interface parent depth exceeded".into())
    }

    /// Register only an explicitly established Marvel frontend-player receiver
    /// with the checked reflected localization method. Unknown receiver classes
    /// remain unsupported; no engine-global or default-instance fallback exists.
    pub fn register_frontend_receiver(
        &mut self,
        receiver: NativeObjectId,
        backend: LocalizationCallback,
    ) -> Result<(), String> {
        if self.receivers.contains_key(&receiver) {
            return Err("external-interface receiver is already registered".into());
        }
        if self.receivers.len() >= self.limits.max_receivers {
            return Err("external-interface receiver budget exceeded".into());
        }
        self.receivers.insert(receiver, backend);
        Ok(())
    }
    pub fn receiver(&self, receiver: NativeObjectId) -> Option<&LocalizationCallback> {
        self.receivers.get(&receiver)
    }
    fn check_context(&self, context: Option<ObjectId>) -> Result<(), String> {
        let context = context.ok_or("external-interface call has no active script target")?;
        if !self.contexts.contains(&context) {
            return Err(
                "external-interface script target has no explicit movie association".into(),
            );
        }
        Ok(())
    }
    /// Supports a future native-marker hook; this does not install `available`.
    pub fn available(&self, context: Option<ObjectId>) -> Result<bool, String> {
        self.check_context(context)?;
        Ok(self.handler != HandlerState::Absent)
    }
    pub fn handle_call(&mut self, call: &HostCall<'_>) -> Option<Result<HostResult, String>> {
        if call.binding != CALL_BINDING {
            return None;
        }
        Some(self.dispatch(call))
    }
    fn dispatch(&mut self, call: &HostCall<'_>) -> Result<HostResult, String> {
        self.check_context(call.context)?;
        // Native no-handler path returns before argument conversion.
        if self.handler == HandlerState::Absent {
            return Ok(HostResult::Value(Value::Undefined));
        }
        // With a handler, arguments reach conversion before the engine rejects
        // a null receiver. Admit only our established string profile here;
        // arbitrary coercion and its side effects remain unsupported.
        if self.association.receiver.is_none() {
            LocalizationCallback::validated_label(call.args, 1024).map_err(|e| e.to_string())?;
            return Ok(HostResult::Value(Value::Undefined));
        }
        let receiver = self.association.receiver.unwrap();
        self.receivers
            .get_mut(&receiver)
            .ok_or("external-interface receiver profile is not registered")?
            .resolve_call(call.args)
            .map_err(|e| e.to_string())
    }
}
