//! Persistent execution of one explicitly selected saved MouseMove clip handler.
//!
//! This is a diagnostic bridge over original code and geometry, not initialized
//! MovieClip construction, native event ordering, or a complete movie player.

use super::display_state::{self, DisplayState};
use crate::{
    avm1::{
        Program,
        vm::{self, CodeId, ObjectId, Vm},
    },
    gfx::scene::{ActionKind, ActionNode, Scene, SceneNode},
};
use std::{io, sync::Arc};

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[derive(Clone, Debug)]
pub struct Limits {
    pub vm: vm::Limits,
    pub display: display_state::Limits,
    /// This bounded diagnostic has no heap collector or unbounded trace store.
    pub max_dispatches: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            vm: vm::Limits::default(),
            display: display_state::Limits::default(),
            max_dispatches: 10_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub dispatch_attempts: u64,
    pub completed_dispatches: u64,
    /// Counts only completed invocations. A failed invocation may mutate state.
    pub completed_instructions: u64,
    pub completed_host_calls: u64,
}

pub struct PointerController {
    vm: Vm,
    target: ObjectId,
    code: CodeId,
    binding: ActionNode,
    display: DisplayState,
    /// Last successful state; never publish a partially executed action.
    nodes: Vec<SceneNode>,
    max_dispatches: u64,
    stats: Statistics,
    halted: bool,
}
impl PointerController {
    pub fn from_scene(scene: &Scene<'_>, limits: Limits) -> io::Result<Self> {
        if limits.max_dispatches == 0 {
            return Err(invalid("pointer dispatch limit must be positive"));
        }
        let mut bindings = scene.actions.iter().filter(|action| {
            matches!(action.kind, ActionKind::Clip { event_flags, .. } if event_flags & 0x08 != 0)
        });
        let binding = bindings
            .next()
            .ok_or_else(|| invalid("scene has no saved MouseMove clip handler"))?
            .clone();
        if bindings.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "multiple MouseMove handlers require an established event dispatch order",
            ));
        }
        let movie = scene
            .movies
            .get(&binding.movie_id)
            .ok_or_else(|| invalid("clip action movie is missing"))?;
        let mut vm = Vm::new(movie.header.version, limits.vm).map_err(io::Error::other)?;
        // Explicit diagnostic association: this does not allocate native
        // MovieClip state, run a registered constructor or claim its category.
        let target = vm.object().map_err(io::Error::other)?;
        let mut display =
            DisplayState::from_scene(scene, limits.display).map_err(io::Error::other)?;
        display
            .bind_object(target, binding.owner)
            .map_err(io::Error::other)?;
        let code = vm
            .register(Arc::new(Program::parse_at(
                binding.bytes.bytes(movie.bytes())?,
                binding.bytes.offset,
            )?))
            .map_err(io::Error::other)?;
        let nodes = display.snapshot_nodes();
        Ok(Self {
            vm,
            target,
            code,
            binding,
            display,
            nodes,
            max_dispatches: limits.max_dispatches,
            stats: Statistics::default(),
            halted: false,
        })
    }

    pub fn binding(&self) -> &ActionNode {
        &self.binding
    }
    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }
    pub fn display(&self) -> &DisplayState {
        &self.display
    }
    pub fn stats(&self) -> Statistics {
        self.stats
    }
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Supply one actual movement in absolute stage twips. The native host owns
    /// surface/DPI/viewport conversion. None invalidates the sample without
    /// executing the handler, moving the cursor or changing visibility.
    ///
    /// Invalid numeric input leaves the prior sample intact. VM/host failures
    /// halt this controller permanently: actions are not transactional and must
    /// not be replayed against partially mutated state. The renderer-facing nodes
    /// remain the last completed snapshot. True means an invocation completed.
    pub fn pointer_stage_twips(&mut self, point: Option<[f64; 2]>) -> io::Result<bool> {
        if self.halted {
            return Err(io::Error::other(
                "pointer controller halted after failed dispatch",
            ));
        }
        if point.is_some_and(|p| p.iter().any(|n| !n.is_finite())) {
            return Err(invalid("pointer coordinates must be finite stage twips"));
        }
        if point.is_none() {
            self.display
                .set_pointer_stage_twips(None)
                .map_err(io::Error::other)?;
            return Ok(false);
        }
        if self.stats.dispatch_attempts >= self.max_dispatches {
            self.halted = true;
            return Err(io::Error::other(
                "pointer diagnostic dispatch budget exhausted",
            ));
        }
        self.display
            .set_pointer_stage_twips(point)
            .map_err(io::Error::other)?;
        self.stats.dispatch_attempts += 1;
        self.halted = true;
        let result = self
            .vm
            .execute(self.code, self.target, &mut self.display)
            .map_err(io::Error::other)?;
        self.stats.completed_instructions = self
            .stats
            .completed_instructions
            .checked_add(result.instructions as u64)
            .ok_or_else(|| io::Error::other("pointer instruction counter overflow"))?;
        self.stats.completed_host_calls = self
            .stats
            .completed_host_calls
            .checked_add(result.host_calls as u64)
            .ok_or_else(|| io::Error::other("pointer host counter overflow"))?;
        self.stats.completed_dispatches += 1;
        // DisplayState owns immutable source identity and only overlays current
        // transform/visibility. No action binding or raw placement is replaced.
        self.nodes = self.display.snapshot_nodes();
        self.halted = false;
        Ok(true)
    }
}
