//! Persistent, bounded execution of a movie's saved frame initializer phase.
//!
//! A stopped phase retains partial VM state for diagnosis and cannot be retried
//! on that state. This is not a display-object factory or a complete movie player.

use crate::{
    avm1::{
        Program,
        vm::{self, CodeId, Host, ObjectId, Value, Vm},
    },
    frontend::{MovieAsset, library::MovieId},
    gfx::{CharacterKind, TagData},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(message: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{frontend::AssetLocation, gfx::Movie};

    fn tag(code: u16, body: &[u8]) -> Vec<u8> {
        assert!(body.len() < 63);
        let mut bytes = ((code << 6) | body.len() as u16).to_le_bytes().to_vec();
        bytes.extend_from_slice(body);
        bytes
    }

    fn fixture(actions: &[&[u8]]) -> MovieAsset {
        let mut bytes = b"GFX\x0f\0\0\0\0\x08\0\0\x18\x01\0".to_vec();
        for (index, action) in actions.iter().enumerate() {
            let id = (index as u16 + 1).to_le_bytes();
            let mut sprite = id.to_vec();
            sprite.extend_from_slice(&[1, 0]);
            sprite.extend(tag(1, &[]));
            sprite.extend(tag(0, &[]));
            bytes.extend(tag(39, &sprite));
            let mut init = id.to_vec();
            init.extend_from_slice(action);
            bytes.extend(tag(59, &init));
        }
        // Ordinary Stop must not execute during the initializer phase.
        bytes.extend(tag(12, &[0x07, 0]));
        bytes.extend(tag(1, &[]));
        bytes.extend(tag(0, &[]));
        let length = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&length.to_le_bytes());
        MovieAsset {
            location: AssetLocation {
                package: 0,
                export: 0,
            },
            object_path: "Test.movie".into(),
            movie: Movie::parse(&bytes).unwrap(),
            references: None,
        }
    }

    #[test]
    fn source_plan_cannot_drop_reorder_or_repeat_initializers() {
        let asset = fixture(&[&[0x17, 0], &[0x17, 0]]);
        let mut session = InitSession::new(15, vm::Limits::default(), 100).unwrap();
        let plan = session.compile_frame(0, &asset, 0).unwrap();
        assert_eq!(plan.initializers.len(), 2);
        assert_eq!(plan.frame_actions.len(), 1);
        let mut changed = plan.clone();
        changed.initializers.clear();
        assert!(session.initialize(&changed, &mut vm::RejectHost).is_err());
        changed = plan.clone();
        changed.initializers.reverse();
        assert!(session.initialize(&changed, &mut vm::RejectHost).is_err());
        assert!(matches!(
            session.initialize(&plan, &mut vm::RejectHost).unwrap(),
            InitOutcome::Complete {
                initializers: 2,
                ..
            }
        ));
        assert!(matches!(
            session.initialize(&plan, &mut vm::RejectHost).unwrap(),
            InitOutcome::AlreadyInitialized
        ));
        assert!(session.setup_vm().is_err());
    }

    #[test]
    fn vm_failure_and_cumulative_budget_both_forbid_retry() {
        for (actions, budget, vm_failure) in [
            (vec![&[0x17, 0][..], &[0x07, 0][..]], 100, true),
            (vec![&[0x17, 0x17, 0][..]], 1, false),
        ] {
            let asset = fixture(&actions);
            let mut session = InitSession::new(15, vm::Limits::default(), budget).unwrap();
            let plan = session.compile_frame(0, &asset, 0).unwrap();
            let result = session.initialize(&plan, &mut vm::RejectHost);
            if vm_failure {
                let InitOutcome::Stopped(failure) = result.unwrap() else {
                    panic!("expected host rejection")
                };
                assert_eq!(failure.completed_initializers, 1);
                assert_eq!(failure.action.key, plan.initializers[1].key);
            } else {
                assert!(result.is_err());
            }
            assert!(session.is_halted());
            assert!(session.initialize(&plan, &mut vm::RejectHost).is_err());
            assert!(session.compile_frame(0, &asset, 0).is_err());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionKey {
    pub movie: MovieId,
    pub tag: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledAction {
    pub key: ActionKey,
    pub code: CodeId,
    pub source_offset: usize,
    pub sprite_id: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramePlan {
    pub movie: MovieId,
    pub object_path: String,
    pub frame: u16,
    /// Serialized order among this root frame's DoInitAction records.
    pub initializers: Vec<ScheduledAction>,
    /// Kept separate. This driver does not execute ordinary frame actions.
    pub frame_actions: Vec<ScheduledAction>,
}

#[derive(Clone, Debug)]
pub struct PhaseFailure {
    pub action: ScheduledAction,
    pub error: vm::Error,
    pub completed_initializers: usize,
    pub instructions: usize,
}

#[derive(Clone, Debug)]
pub enum InitOutcome {
    Complete {
        initializers: usize,
        instructions: usize,
        host_calls: usize,
    },
    AlreadyInitialized,
    Stopped(PhaseFailure),
}

struct RegisteredAction {
    code: CodeId,
    raw: Vec<u8>,
    source_offset: usize,
}

/// Movie IDs must retain identity within this session. Programs own their code,
/// so additional movies can be registered without borrowing a mutable library.
pub struct InitSession {
    vm: Vm,
    version: u8,
    sources: BTreeMap<MovieId, String>,
    actions: BTreeMap<ActionKey, RegisteredAction>,
    plans: BTreeMap<(MovieId, u16), FramePlan>,
    visited: BTreeSet<(MovieId, u16)>,
    failed: Option<PhaseFailure>,
    halted: bool,
    started: bool,
    max_phase_instructions: usize,
}

impl InitSession {
    pub fn new(version: u8, limits: vm::Limits, max_phase_instructions: usize) -> Result<Self> {
        if max_phase_instructions == 0 {
            return Err(invalid("initializer phase budget must be positive"));
        }
        Ok(Self {
            vm: Vm::new(version, limits)?,
            version,
            sources: BTreeMap::new(),
            actions: BTreeMap::new(),
            plans: BTreeMap::new(),
            visited: BTreeSet::new(),
            failed: None,
            halted: false,
            started: false,
            max_phase_instructions,
        })
    }

    pub fn vm(&self) -> &Vm {
        &self.vm
    }
    pub fn failure(&self) -> Option<&PhaseFailure> {
        self.failed.as_ref()
    }
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Explicit bootstrap bindings only. No script execution may have begun.
    pub fn setup_vm(&mut self) -> Result<&mut Vm> {
        if self.started {
            return Err(invalid(
                "bootstrap binding setup is closed after execution begins",
            ));
        }
        Ok(&mut self.vm)
    }

    /// Supply a non-executed MovieClip class/prototype identity for inheritance
    /// study. Calling this native constructor still goes through the given Host.
    /// This does not manufacture display objects or fake MovieClip methods.
    pub fn install_movie_clip_metadata(&mut self, native_binding: u32) -> Result<ObjectId> {
        let vm = self.setup_vm()?;
        if !matches!(vm.get(vm.global(), "MovieClip")?, Value::Undefined) {
            return Err(invalid("MovieClip global is already bound"));
        }
        let constructor = vm.install_object_constructor()?;
        let object_prototype = vm.get(constructor, "prototype")?;
        let native = vm.native(native_binding)?;
        let prototype = vm.object()?;
        vm.set(prototype, "__proto__", object_prototype)?;
        vm.set(prototype, "constructor", Value::Object(native))?;
        vm.set(native, "prototype", Value::Object(prototype))?;
        vm.set(vm.global(), "MovieClip", Value::Object(native))?;
        Ok(native)
    }

    pub fn compile_frame(
        &mut self,
        movie_id: MovieId,
        asset: &MovieAsset,
        frame: u16,
    ) -> Result<FramePlan> {
        if self.halted {
            return Err(invalid("initializer session is halted"));
        }
        let movie = &asset.movie;
        if movie.header.version != self.version {
            return Err(invalid(
                "mixed movie-version execution is outside this session profile",
            ));
        }
        if let Some(path) = self.sources.get(&movie_id) {
            if path != &asset.object_path {
                return Err(invalid("movie ID changed its object identity"));
            }
        }
        if movie
            .tags
            .iter()
            .any(|tag| matches!(tag.data, TagData::FileAttributes(flags) if flags & 8 != 0))
        {
            return Err(invalid(
                "AVM2 movie is outside the AVM1 initializer profile",
            ));
        }
        let root = movie
            .timelines
            .get(&0)
            .ok_or_else(|| invalid("movie has no root timeline"))?;
        if frame >= root.frame_count {
            return Err(invalid("initializer frame is outside the root timeline"));
        }
        let mut sprite_ids = BTreeSet::new();
        for tag in &movie.tags {
            if let TagData::Actions {
                sprite_id: Some(id),
                ..
            } = tag.data
            {
                if !sprite_ids.insert(id) {
                    return Err(invalid("multiple DoInitAction tags name the same sprite"));
                }
                if !movie
                    .dictionary
                    .get(&id)
                    .is_some_and(|entry| entry.kind == CharacterKind::Sprite)
                {
                    return Err(invalid("initializer names a non-sprite dictionary entry"));
                }
            }
        }
        let mut plan = FramePlan {
            movie: movie_id,
            object_path: asset.object_path.clone(),
            frame,
            initializers: Vec::new(),
            frame_actions: Vec::new(),
        };
        for &tag_index in &root.tags {
            let tag = &movie.tags[tag_index];
            if tag.frame_index != frame {
                continue;
            }
            let TagData::Actions { sprite_id, bytes } = tag.data else {
                continue;
            };
            let raw = bytes.bytes(movie.bytes())?;
            let key = ActionKey {
                movie: movie_id,
                tag: tag_index,
            };
            let code = if let Some(existing) = self.actions.get(&key) {
                if existing.raw != raw || existing.source_offset != bytes.offset {
                    return Err(invalid("registered action bytes or offset changed"));
                }
                existing.code
            } else {
                let code = self
                    .vm
                    .register(Arc::new(Program::parse_at(raw, bytes.offset)?))?;
                self.actions.insert(
                    key,
                    RegisteredAction {
                        code,
                        raw: raw.to_vec(),
                        source_offset: bytes.offset,
                    },
                );
                code
            };
            let action = ScheduledAction {
                key,
                code,
                source_offset: bytes.offset,
                sprite_id,
            };
            if sprite_id.is_some() {
                plan.initializers.push(action);
            } else {
                plan.frame_actions.push(action);
            }
        }
        self.sources.insert(movie_id, asset.object_path.clone());
        if let Some(existing) = self.plans.get(&(movie_id, frame)) {
            if existing != &plan {
                return Err(invalid("compiled frame plan changed"));
            }
        } else {
            self.plans.insert((movie_id, frame), plan.clone());
        }
        Ok(plan)
    }

    /// Run only the first-encounter initializer phase, in saved order. Ordinary
    /// frame actions, sprite construction, imported-movie ordering and streaming
    /// lifecycle remain outside this method. Failure requires a fresh session.
    pub fn initialize(&mut self, plan: &FramePlan, host: &mut impl Host) -> Result<InitOutcome> {
        if self.halted {
            return Err(invalid(
                "initializer session stopped with partial state; create a fresh session",
            ));
        }
        if self.plans.get(&(plan.movie, plan.frame)) != Some(plan) {
            return Err(invalid("frame plan differs from its compiled source order"));
        }
        if self.sources.get(&plan.movie) != Some(&plan.object_path) {
            return Err(invalid("frame plan belongs to another session source"));
        }
        for action in &plan.initializers {
            let Some(registered) = self.actions.get(&action.key) else {
                return Err(invalid("frame plan contains an unregistered action"));
            };
            if action.key.movie != plan.movie
                || registered.code != action.code
                || registered.source_offset != action.source_offset
            {
                return Err(invalid("frame plan action identity mismatch"));
            }
        }
        if self.visited.contains(&(plan.movie, plan.frame)) {
            return Ok(InitOutcome::AlreadyInitialized);
        }
        self.started = true;
        let mut instructions = 0usize;
        let mut host_calls = 0usize;
        let target = self.vm.root();
        for (completed, action) in plan.initializers.iter().enumerate() {
            match self.vm.execute(action.code, target, host) {
                Ok(execution) => {
                    instructions = instructions.saturating_add(execution.instructions);
                    host_calls = host_calls.saturating_add(execution.host_calls);
                    if instructions > self.max_phase_instructions {
                        // A per-action VM budget also applies. This cumulative
                        // admission check can exceed its threshold by one action.
                        self.halted = true;
                        return Err(invalid(
                            "cumulative initializer instruction budget exceeded; discard session",
                        ));
                    }
                }
                Err(error) => {
                    instructions = instructions.saturating_add(self.vm.instruction_count());
                    let failure = PhaseFailure {
                        action: action.clone(),
                        error,
                        completed_initializers: completed,
                        instructions,
                    };
                    self.failed = Some(failure.clone());
                    self.halted = true;
                    return Ok(InitOutcome::Stopped(failure));
                }
            }
        }
        self.visited.insert((plan.movie, plan.frame));
        Ok(InitOutcome::Complete {
            initializers: plan.initializers.len(),
            instructions,
            host_calls,
        })
    }
}
