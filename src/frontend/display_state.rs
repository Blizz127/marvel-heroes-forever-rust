//! Explicit display geometry for an existing structural Scene and VM host bindings.
//! No MovieClip construction, path lookup, event dispatch, or startup execution.

use crate::avm1::vm::{
    AttachedClipId, AttachedClipSource, AttachedVisibilityRequest, DisplayTargetId, Host, HostCall,
    HostCommand, LoadedMovieSourceId, MovieProperty, ObjectId, PropertyRequest, Value, Vm,
};
use crate::gfx::{
    CharacterKind, Movie,
    scene::{ActionNode, Affine, MovieId, NodeId, NodeKind, OpaqueNode, Scene, SceneNode, TagRef},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

pub const TWIPS_PER_PIXEL: f64 = 20.0;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_source_items: usize,
    pub max_source_string_bytes: usize,
    /// Total immutable movie bytes hashed for later current-scene projection.
    pub max_source_movie_bytes: usize,
    pub max_bindings: usize,
    pub max_property_calls: usize,
    pub max_recomputed_nodes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_depth: 256,
            max_source_items: 2_000_000,
            max_source_string_bytes: 16 * 1024 * 1024,
            max_source_movie_bytes: 256 * 1024 * 1024,
            max_bindings: 100_000,
            max_property_calls: 100_000,
            max_recomputed_nodes: 2_000_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayError {
    InvalidScene {
        node: NodeId,
        reason: &'static str,
    },
    InvalidNode(NodeId),
    InvalidMovie(MovieId),
    SourceMovieSet,
    SourceMovieChanged(MovieId),
    UnboundSource(LoadedMovieSourceId),
    SourceBindingConflict {
        source: LoadedMovieSourceId,
        movie: MovieId,
    },
    InvalidAttached(String),
    UnboundAttached(AttachedClipId),
    AttachedMismatch {
        clip: AttachedClipId,
        reason: &'static str,
    },
    UnboundObject(ObjectId),
    BindingConflict {
        object: ObjectId,
        node: NodeId,
    },
    Budget(&'static str),
    UnsupportedTarget,
    UnsupportedProperty {
        property: MovieProperty,
        write: bool,
    },
    UnsupportedValue(MovieProperty),
    PointerUnavailable,
    SingularTransform(NodeId),
    NonFinite(&'static str),
}
impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "display-state {self:?}")
    }
}
impl std::error::Error for DisplayError {}
pub type Result<T> = std::result::Result<T, DisplayError>;

#[derive(Clone, Debug)]
pub struct DisplayNode {
    /// Immutable original metadata. Its raw placement and world values are source data.
    pub source: SceneNode,
    pub local_matrix: Affine,
    pub world_matrix: Affine,
    pub local_visible: bool,
    pub effective_visible: bool,
    subtree_end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyOperation {
    Read(Value),
    Write { before: Value, after: Value },
}
#[derive(Clone, Debug)]
pub struct PropertyTrace {
    pub request: PropertyRequest,
    pub node: NodeId,
    pub pointer_stage_twips: Option<[f64; 2]>,
    pub operation: PropertyOperation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// Includes rejected requests, excluding requests denied by this budget itself.
    pub property_calls: usize,
    /// Includes work reserved by a write whose finite-result check subsequently fails.
    pub recomputed_nodes: usize,
    pub revision: u64,
}

/// Facts retained from the existing VM binding, not derived from instance names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttachedIdentity {
    root: ObjectId,
    owner: ObjectId,
    target: DisplayTargetId,
    source: AttachedClipSource,
    parent: Option<AttachedClipId>,
}

/// One explicit VM/Scene association domain. ObjectId is VM-local; the caller
/// must not reuse these bindings with another VM or a rebuilt Scene.
pub struct DisplayState {
    root: NodeId,
    nodes: Vec<DisplayNode>,
    bindings: BTreeMap<ObjectId, NodeId>,
    node_bindings: BTreeMap<NodeId, ObjectId>,
    movie_versions: BTreeMap<MovieId, u8>,
    movie_tag_codes: BTreeMap<MovieId, Vec<u16>>,
    source_movies: BTreeMap<MovieId, (usize, [u8; 32])>,
    source_actions: Vec<ActionNode>,
    source_opaque_nodes: Vec<OpaqueNode>,
    source_unbound_action_tags: Vec<TagRef>,
    source_bindings: BTreeMap<LoadedMovieSourceId, MovieId>,
    movie_sources: BTreeMap<MovieId, LoadedMovieSourceId>,
    attached_bindings: BTreeMap<AttachedClipId, NodeId>,
    attached_nodes: BTreeMap<NodeId, AttachedClipId>,
    attached_identities: BTreeMap<AttachedClipId, AttachedIdentity>,
    pointer_stage_twips: Option<[f64; 2]>,
    traces: Vec<PropertyTrace>,
    limits: Limits,
    stats: Statistics,
}
impl DisplayState {
    pub fn from_scene(scene: &Scene<'_>, limits: Limits) -> Result<Self> {
        let count = scene.nodes.len();
        if count == 0 || count > limits.max_nodes || limits.max_depth > 256 {
            return Err(DisplayError::Budget("scene nodes/depth"));
        }
        if scene.root != 0 {
            return Err(malformed(scene.root, "scene arena must start at its root"));
        }
        let mut items = 0usize;
        let mut bytes = 0usize;
        charge(
            &mut items,
            scene.movies.len(),
            limits.max_source_items,
            "source movies",
        )?;
        for movie in scene.movies.values() {
            charge(
                &mut items,
                movie.tags.len(),
                limits.max_source_items,
                "source tag codes",
            )?;
        }
        let mut movie_bytes = 0;
        for movie in scene.movies.values() {
            charge(
                &mut movie_bytes,
                movie.bytes().len(),
                limits.max_source_movie_bytes,
                "source movie bytes",
            )?;
        }
        for count in [
            scene.actions.len(),
            scene.opaque_nodes.len(),
            scene.unbound_action_tags.len(),
        ] {
            charge(
                &mut items,
                count,
                limits.max_source_items,
                "scene metadata items",
            )?;
        }
        for (id, source) in scene.nodes.iter().enumerate() {
            if source.id != id {
                return Err(malformed(id, "node identity differs from arena index"));
            }
            for amount in [
                source.children.len(),
                source.masks.len(),
                source.actions.len(),
                source.scaling_grids.len(),
                source.unsupported.len(),
            ] {
                charge(
                    &mut items,
                    amount,
                    limits.max_source_items,
                    "source metadata items",
                )?;
            }
            if let Some(segment) = &source.segment {
                charge(
                    &mut bytes,
                    segment.name.as_ref().map_or(0, String::len),
                    limits.max_source_string_bytes,
                    "source strings",
                )?;
            }
            if let Some(raw) = &source.raw {
                charge(
                    &mut items,
                    raw.filters.len(),
                    limits.max_source_items,
                    "source metadata items",
                )?;
                charge(
                    &mut items,
                    raw.clip_actions.len(),
                    limits.max_source_items,
                    "source metadata items",
                )?;
                for value in [&raw.name, &raw.class_name] {
                    charge(
                        &mut bytes,
                        value.as_ref().map_or(0, String::len),
                        limits.max_source_string_bytes,
                        "source strings",
                    )?;
                }
            }
            if let NodeKind::Import { chain, .. } = &source.kind {
                charge(
                    &mut items,
                    chain.len(),
                    limits.max_source_items,
                    "source metadata items",
                )?;
                for link in chain {
                    charge(
                        &mut bytes,
                        link.url.len(),
                        limits.max_source_string_bytes,
                        "source strings",
                    )?;
                    charge(
                        &mut bytes,
                        link.symbol.len(),
                        limits.max_source_string_bytes,
                        "source strings",
                    )?;
                }
            }
            if source.masks.iter().any(|&mask| mask >= count) {
                return Err(malformed(id, "mask identity is outside arena"));
            }
            if id == 0 {
                if source.parent.is_some()
                    || source.raw.is_some()
                    || !matches!(source.kind, NodeKind::Root)
                {
                    return Err(malformed(id, "root source is not an unplaced scene root"));
                }
            } else if source.parent.is_none()
                || source.raw.is_none()
                || matches!(source.kind, NodeKind::Root)
            {
                return Err(malformed(
                    id,
                    "nonroot lacks parent/placement or is another root",
                ));
            }
        }
        // Validate complete preorder, parent/child agreement, cycles and depth
        // before cloning any source metadata. Every subtree is then a range.
        let mut ends = vec![0usize; count];
        let mut stack = vec![(0usize, 0usize, false)];
        let mut next = 0usize;
        while let Some((id, depth, leaving)) = stack.pop() {
            if leaving {
                ends[id] = next;
                continue;
            }
            if depth > limits.max_depth {
                return Err(DisplayError::Budget("scene depth"));
            }
            if id != next || id >= count {
                return Err(malformed(id, "not a complete preorder tree"));
            }
            next += 1;
            stack.push((id, depth, true));
            for &child in scene.nodes[id].children.iter().rev() {
                let source = scene
                    .nodes
                    .get(child)
                    .ok_or(DisplayError::InvalidNode(child))?;
                if source.parent != Some(id) {
                    return Err(malformed(child, "parent/child disagreement"));
                }
                stack.push((child, depth + 1, false));
            }
        }
        if next != count {
            return Err(malformed(next, "unreachable node"));
        }
        let mut nodes: Vec<DisplayNode> = Vec::with_capacity(count);
        for (id, source) in scene.nodes.iter().enumerate() {
            let local_matrix = source
                .raw
                .as_ref()
                .map_or(Affine::default(), |raw| raw.matrix.into());
            let local_visible = source.raw.as_ref().is_none_or(|raw| raw.visible);
            let (world_matrix, effective_visible) = if let Some(parent) = source.parent {
                let parent = nodes
                    .get(parent)
                    .ok_or_else(|| malformed(id, "parent must precede child"))?;
                (
                    parent.world_matrix.compose(local_matrix),
                    parent.effective_visible && local_visible,
                )
            } else {
                (local_matrix, local_visible)
            };
            finite_affine(world_matrix)?;
            if world_matrix != source.world_matrix || effective_visible != source.effective_visible
            {
                return Err(malformed(
                    id,
                    "source derived state disagrees with raw placement",
                ));
            }
            nodes.push(DisplayNode {
                source: source.clone(),
                local_matrix,
                world_matrix,
                local_visible,
                effective_visible,
                subtree_end: ends[id],
            });
        }
        Ok(Self {
            root: scene.root,
            nodes,
            bindings: BTreeMap::new(),
            node_bindings: BTreeMap::new(),
            movie_versions: scene
                .movies
                .iter()
                .map(|(&id, movie)| (id, movie.header.version))
                .collect(),
            movie_tag_codes: scene
                .movies
                .iter()
                .map(|(&id, movie)| (id, movie.tags.iter().map(|tag| tag.code).collect()))
                .collect(),
            source_movies: scene
                .movies
                .iter()
                .map(|(&id, movie)| {
                    (
                        id,
                        (movie.bytes().len(), Sha256::digest(movie.bytes()).into()),
                    )
                })
                .collect(),
            source_actions: scene.actions.clone(),
            source_opaque_nodes: scene.opaque_nodes.clone(),
            source_unbound_action_tags: scene.unbound_action_tags.clone(),
            source_bindings: BTreeMap::new(),
            movie_sources: BTreeMap::new(),
            attached_bindings: BTreeMap::new(),
            attached_nodes: BTreeMap::new(),
            attached_identities: BTreeMap::new(),
            pointer_stage_twips: None,
            traces: Vec::new(),
            limits,
            stats: Statistics::default(),
        })
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn nodes(&self) -> &[DisplayNode] {
        &self.nodes
    }
    pub fn node(&self, id: NodeId) -> Result<&DisplayNode> {
        self.nodes.get(id).ok_or(DisplayError::InvalidNode(id))
    }
    pub fn stats(&self) -> Statistics {
        self.stats
    }
    pub fn traces(&self) -> &[PropertyTrace] {
        &self.traces
    }
    pub fn bindings(&self) -> &BTreeMap<ObjectId, NodeId> {
        &self.bindings
    }
    pub fn node_for_attached(&self, clip: AttachedClipId) -> Option<NodeId> {
        self.attached_bindings.get(&clip).copied()
    }
    pub fn movie_for_source(&self, source: LoadedMovieSourceId) -> Option<MovieId> {
        self.source_bindings.get(&source).copied()
    }
    pub fn movie_version(&self, movie: MovieId) -> Option<u8> {
        self.movie_versions.get(&movie).copied()
    }
    pub fn tag_code(&self, movie: MovieId, tag: usize) -> Option<u16> {
        self.movie_tag_codes.get(&movie)?.get(tag).copied()
    }
    pub fn pointer_stage_twips(&self) -> Option<[f64; 2]> {
        self.pointer_stage_twips
    }

    /// The host explicitly associates an already-created VM object. This method
    /// does not create objects or assert that any class constructor has executed.
    pub fn bind_object(&mut self, object: ObjectId, node: NodeId) -> Result<()> {
        self.node(node)?;
        if self.bindings.get(&object) == Some(&node) {
            return Ok(());
        }
        if self.bindings.contains_key(&object) || self.node_bindings.contains_key(&node) {
            return Err(DisplayError::BindingConflict { object, node });
        }
        if self.bindings.len() >= self.limits.max_bindings {
            return Err(DisplayError::Budget("bindings"));
        }
        self.bindings.insert(object, node);
        self.node_bindings.insert(node, object);
        Ok(())
    }

    /// Explicit association of a loaded source identity with this Scene's movie.
    /// Neither numeric identity is a file name or an implicit cross-VM alias.
    pub fn bind_movie_source(&mut self, source: LoadedMovieSourceId, movie: MovieId) -> Result<()> {
        if !self.movie_versions.contains_key(&movie) {
            return Err(DisplayError::InvalidMovie(movie));
        }
        if self.source_bindings.get(&source) == Some(&movie) {
            return Ok(());
        }
        if self.source_bindings.contains_key(&source) || self.movie_sources.contains_key(&movie) {
            return Err(DisplayError::SourceBindingConflict { source, movie });
        }
        self.source_bindings.insert(source, movie);
        self.movie_sources.insert(movie, source);
        Ok(())
    }

    /// Bind an already-created live attached MovieClip after checking its source,
    /// definition, version and parent against this Scene. Parent bindings must
    /// precede child bindings. This creates no clip or native lifecycle state.
    pub fn bind_attached(&mut self, vm: &Vm, clip: AttachedClipId, node: NodeId) -> Result<()> {
        let info = vm
            .attached_info(clip)
            .map_err(|error| DisplayError::InvalidAttached(error.to_string()))?;
        let source = &self.node(node)?.source;
        let mismatch = |reason| DisplayError::AttachedMismatch { clip, reason };
        let (movie, sprite) = match &source.kind {
            NodeKind::Root => (source.placement_movie, 0),
            kind => {
                let definition = kind
                    .definition()
                    .ok_or_else(|| mismatch("unresolved scene definition"))?;
                if definition.kind != CharacterKind::Sprite {
                    return Err(mismatch("scene definition is not a MovieClip sprite"));
                }
                (definition.movie_id, definition.character_id)
            }
        };
        let mapped = self
            .source_bindings
            .get(&info.source.movie)
            .ok_or(DisplayError::UnboundSource(info.source.movie))?;
        if *mapped != movie
            || info.source.sprite_id != sprite
            || self.movie_versions.get(&movie) != Some(&info.source.header_version)
        {
            return Err(mismatch("loaded source, sprite or header version differs"));
        }
        match (source.parent, info.parent) {
            (None, None) => {}
            (Some(parent), Some(parent_clip))
                if self.attached_bindings.get(&parent_clip) == Some(&parent) => {}
            _ => {
                return Err(mismatch(
                    "attached parent does not match the bound scene parent",
                ));
            }
        }
        if self
            .attached_bindings
            .get(&clip)
            .is_some_and(|&bound| bound != node)
        {
            return Err(mismatch("attached clip is already bound to another node"));
        }
        if self
            .attached_nodes
            .get(&node)
            .is_some_and(|old| *old != clip)
        {
            return Err(mismatch(
                "scene node is already bound to another attached clip",
            ));
        }
        let identity = AttachedIdentity {
            root: vm.root(),
            owner: info.owner,
            target: info.target,
            source: info.source,
            parent: info.parent,
        };
        if self
            .attached_identities
            .get(&clip)
            .is_some_and(|old| *old != identity)
        {
            return Err(mismatch("attached identity differs from retained binding"));
        }
        self.bind_object(info.owner, node)?;
        self.attached_bindings.insert(clip, node);
        self.attached_nodes.insert(node, clip);
        self.attached_identities.insert(clip, identity);
        Ok(())
    }

    /// Stage coordinates must already be in the Scene's world-twip space.
    /// None explicitly clears the sample; no (0,0) pointer is invented.
    pub fn set_pointer_stage_twips(&mut self, pointer: Option<[f64; 2]>) -> Result<()> {
        if pointer.is_some_and(|p| p.iter().any(|v| !v.is_finite())) {
            return Err(DisplayError::NonFinite("pointer"));
        }
        self.pointer_stage_twips = pointer;
        Ok(())
    }

    pub fn local_mouse_twips(&self, node: NodeId) -> Result<[f64; 2]> {
        let matrix = self.node(node)?.world_matrix;
        let pointer = self
            .pointer_stage_twips
            .ok_or(DisplayError::PointerUnavailable)?;
        inverse_point(matrix, pointer, node)
    }

    /// Owned renderer-facing instances with updated world transform/visibility.
    /// Every other field, especially raw source placement and source IDs, is kept.
    pub fn snapshot_nodes(&self) -> Vec<SceneNode> {
        self.nodes
            .iter()
            .map(|state| {
                let mut node = state.source.clone();
                node.world_matrix = state.world_matrix;
                node.effective_visible = state.effective_visible;
                node
            })
            .collect()
    }

    /// Snapshot the current render geometry with this state's retained source
    /// metadata. Supply the exact immutable movies from the original Scene.
    /// Hashes verify content, not native instance lifetime or VM completion.
    ///
    /// This does not execute actions, acknowledge text invalidation or rebuild
    /// DisplayState from changed derived geometry. Raw placement stays original;
    /// world transforms/effective visibility come from current host state.
    pub fn snapshot_scene<'movie>(
        &self,
        movies: BTreeMap<MovieId, &'movie Movie>,
    ) -> Result<Scene<'movie>> {
        if movies.len() != self.source_movies.len() {
            return Err(DisplayError::SourceMovieSet);
        }
        for (&id, &(length, digest)) in &self.source_movies {
            let movie = movies.get(&id).ok_or(DisplayError::SourceMovieSet)?;
            if movie.bytes().len() != length
                || self.movie_versions.get(&id) != Some(&movie.header.version)
                || <[u8; 32]>::from(Sha256::digest(movie.bytes())) != digest
            {
                return Err(DisplayError::SourceMovieChanged(id));
            }
        }
        Ok(Scene {
            root: self.root,
            movies,
            nodes: self.snapshot_nodes(),
            actions: self.source_actions.clone(),
            opaque_nodes: self.source_opaque_nodes.clone(),
            unbound_action_tags: self.source_unbound_action_tags.clone(),
        })
    }

    fn target(&self, request: &PropertyRequest) -> Result<NodeId> {
        // A context is always required, including explicit-object targets.
        let current = *self
            .bindings
            .get(&request.context)
            .ok_or(DisplayError::UnboundObject(request.context))?;
        match &request.target {
            Value::String(path) if path.is_empty() => Ok(current),
            Value::Object(object) => self
                .bindings
                .get(object)
                .copied()
                .ok_or(DisplayError::UnboundObject(*object)),
            Value::DisplayClip(clip) => self
                .attached_bindings
                .get(clip)
                .copied()
                .ok_or(DisplayError::UnboundAttached(*clip)),
            _ => Err(DisplayError::UnsupportedTarget),
        }
    }
    fn request_budget(&mut self) -> Result<()> {
        charge(
            &mut self.stats.property_calls,
            1,
            self.limits.max_property_calls,
            "property calls",
        )
    }
    fn property(&self, node: NodeId, property: MovieProperty) -> Result<Value> {
        let state = self.node(node)?;
        Ok(match property {
            MovieProperty::X => Value::Number(state.local_matrix.tx / TWIPS_PER_PIXEL),
            MovieProperty::Y => Value::Number(state.local_matrix.ty / TWIPS_PER_PIXEL),
            MovieProperty::Visible => Value::Bool(state.local_visible),
            MovieProperty::MouseX => {
                Value::Number(self.local_mouse_twips(node)?[0] / TWIPS_PER_PIXEL)
            }
            MovieProperty::MouseY => {
                Value::Number(self.local_mouse_twips(node)?[1] / TWIPS_PER_PIXEL)
            }
            _ => {
                return Err(DisplayError::UnsupportedProperty {
                    property,
                    write: false,
                });
            }
        })
    }
    pub fn read_property(&mut self, request: PropertyRequest) -> Result<Value> {
        self.request_budget()?;
        let node = self.target(&request)?;
        let value = self.property(node, request.property)?;
        self.traces.push(PropertyTrace {
            request,
            node,
            pointer_stage_twips: self.pointer_stage_twips,
            operation: PropertyOperation::Read(value.clone()),
        });
        Ok(value)
    }
    pub fn write_property(&mut self, request: PropertyRequest, value: Value) -> Result<()> {
        self.request_budget()?;
        let node = self.target(&request)?;
        if request.property == MovieProperty::Visible && self.attached_nodes.contains_key(&node) {
            return Err(DisplayError::UnsupportedProperty {
                property: MovieProperty::Visible,
                write: true,
            });
        }
        let mut matrix = self.nodes[node].local_matrix;
        let mut visible = self.nodes[node].local_visible;
        match request.property {
            MovieProperty::X | MovieProperty::Y => {
                let Value::Number(number) = value else {
                    return Err(DisplayError::UnsupportedValue(request.property));
                };
                let twips = number * TWIPS_PER_PIXEL;
                if !number.is_finite() || !twips.is_finite() {
                    return Err(DisplayError::NonFinite("position"));
                }
                if request.property == MovieProperty::X {
                    matrix.tx = twips;
                } else {
                    matrix.ty = twips;
                }
            }
            MovieProperty::Visible => {
                let Value::Bool(value) = value else {
                    return Err(DisplayError::UnsupportedValue(request.property));
                };
                visible = value;
            }
            _ => {
                return Err(DisplayError::UnsupportedProperty {
                    property: request.property,
                    write: true,
                });
            }
        }
        let before = self.property(node, request.property)?;
        self.update_subtree(node, matrix, visible)?;
        let after = self.property(node, request.property)?;
        self.traces.push(PropertyTrace {
            request,
            node,
            pointer_stage_twips: self.pointer_stage_twips,
            operation: PropertyOperation::Write { before, after },
        });
        Ok(())
    }

    /// Validate/stage the host projection before the VM's raw local-bit write.
    /// The returned borrow excludes intervening host mutation. Invoking it after
    /// that VM write publishes prepared visibility/trace/revision infallibly.
    /// Dropping it cancels semantic changes; attempted-work counters and vector
    /// capacity can still reflect preparation. No native node layout is claimed.
    pub fn prepare_attached_visibility(
        &mut self,
        request: AttachedVisibilityRequest,
    ) -> Result<Box<dyn FnOnce() + '_>> {
        self.request_budget()?;
        let mismatch = |reason| DisplayError::AttachedMismatch {
            clip: request.receiver,
            reason,
        };
        let identity = self
            .attached_identities
            .get(&request.receiver)
            .ok_or(DisplayError::UnboundAttached(request.receiver))?;
        let node = *self
            .attached_bindings
            .get(&request.receiver)
            .ok_or(DisplayError::UnboundAttached(request.receiver))?;
        if identity.root != request.root
            || identity.owner != request.owner
            || identity.target != request.target
            || self.bindings.get(&request.root) != Some(&self.root)
            || self.node_bindings.get(&self.root) != Some(&request.root)
            || self.bindings.get(&request.owner) != Some(&node)
            || self.node_bindings.get(&node) != Some(&request.owner)
            || self.attached_nodes.get(&node) != Some(&request.receiver)
        {
            return Err(mismatch(
                "visibility receiver/root/owner/target association differs",
            ));
        }
        if !self.bindings.contains_key(&request.context) {
            return Err(DisplayError::UnboundObject(request.context));
        }
        let state = &self.nodes[node];
        let (movie, sprite) = match state.source.kind {
            NodeKind::Root => (state.source.placement_movie, 0),
            _ => {
                let definition = state
                    .source
                    .kind
                    .definition()
                    .ok_or_else(|| mismatch("visibility source definition absent"))?;
                if definition.kind != CharacterKind::Sprite {
                    return Err(mismatch("visibility receiver is not a Sprite"));
                }
                (definition.movie_id, definition.character_id)
            }
        };
        if self.source_bindings.get(&identity.source.movie) != Some(&movie)
            || identity.source.sprite_id != sprite
            || self.movie_versions.get(&movie) != Some(&identity.source.header_version)
            || state.source.parent
                != identity
                    .parent
                    .and_then(|parent| self.attached_bindings.get(&parent).copied())
        {
            return Err(mismatch("visibility source/parent association differs"));
        }
        if state.local_visible != request.expected_local {
            return Err(mismatch(
                "host local visibility differs from current native bit",
            ));
        }
        let end = state.subtree_end;
        charge(
            &mut self.stats.recomputed_nodes,
            end - node,
            self.limits.max_recomputed_nodes,
            "descendant visibility work",
        )?;
        let revision = self
            .stats
            .revision
            .checked_add(1)
            .ok_or(DisplayError::Budget("revision"))?;
        let mut visibility = Vec::new();
        visibility
            .try_reserve(end - node)
            .map_err(|_| DisplayError::Budget("visibility update allocation"))?;
        for id in node..end {
            let state = &self.nodes[id];
            let local = if id == node {
                request.visible
            } else {
                state.local_visible
            };
            let parent = state.source.parent.map_or(true, |parent| {
                if parent >= node {
                    visibility[parent - node]
                } else {
                    self.nodes[parent].effective_visible
                }
            });
            visibility.push(local && parent);
        }
        self.traces
            .try_reserve(1)
            .map_err(|_| DisplayError::Budget("visibility trace allocation"))?;
        let trace = PropertyTrace {
            request: PropertyRequest {
                context: request.context,
                target: Value::DisplayClip(request.receiver),
                property: MovieProperty::Visible,
                origin: request.origin,
                source_offset: request.origin.map(|origin| origin.source_offset),
            },
            node,
            pointer_stage_twips: self.pointer_stage_twips,
            operation: PropertyOperation::Write {
                before: Value::Bool(request.expected_local),
                after: Value::Bool(request.visible),
            },
        };
        Ok(Box::new(move || {
            self.nodes[node].local_visible = request.visible;
            for (state, visible) in self.nodes[node..end].iter_mut().zip(visibility) {
                state.effective_visible = visible;
            }
            self.stats.revision = revision;
            self.traces.push(trace);
        }))
    }
    fn update_subtree(&mut self, node: NodeId, local: Affine, visible: bool) -> Result<()> {
        let end = self.nodes[node].subtree_end;
        charge(
            &mut self.stats.recomputed_nodes,
            end - node,
            self.limits.max_recomputed_nodes,
            "descendant update work",
        )?;
        let revision = self
            .stats
            .revision
            .checked_add(1)
            .ok_or(DisplayError::Budget("revision"))?;
        let mut updates: Vec<(Affine, bool)> = Vec::with_capacity(end - node);
        for id in node..end {
            let state = &self.nodes[id];
            let matrix = if id == node {
                local
            } else {
                state.local_matrix
            };
            let local_visible = if id == node {
                visible
            } else {
                state.local_visible
            };
            let (world, effective_visible) = if let Some(parent) = state.source.parent {
                let (parent_world, parent_visible) = if parent >= node {
                    updates[parent - node]
                } else {
                    (
                        self.nodes[parent].world_matrix,
                        self.nodes[parent].effective_visible,
                    )
                };
                (
                    parent_world.compose(matrix),
                    parent_visible && local_visible,
                )
            } else {
                (matrix, local_visible)
            };
            finite_affine(world)?;
            updates.push((world, effective_visible));
        }
        // All results are finite and all budgets admitted before geometry changes.
        self.nodes[node].local_matrix = local;
        self.nodes[node].local_visible = visible;
        for (id, (world, effective_visible)) in (node..end).zip(updates) {
            self.nodes[id].world_matrix = world;
            self.nodes[id].effective_visible = effective_visible;
        }
        self.stats.revision = revision;
        Ok(())
    }
}
impl Host for DisplayState {
    fn prepare_attached_visibility(
        &mut self,
        request: AttachedVisibilityRequest,
    ) -> std::result::Result<Box<dyn FnOnce() + '_>, String> {
        DisplayState::prepare_attached_visibility(self, request).map_err(|error| error.to_string())
    }
    fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
        Err("display state has no native function bindings".into())
    }
    fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
        Err("display state has no timeline, URL or other command bindings".into())
    }
    fn get_property(&mut self, request: PropertyRequest) -> std::result::Result<Value, String> {
        self.read_property(request)
            .map_err(|error| error.to_string())
    }
    fn set_property(
        &mut self,
        request: PropertyRequest,
        value: Value,
    ) -> std::result::Result<(), String> {
        self.write_property(request, value)
            .map_err(|error| error.to_string())
    }
}

/// Full affine inverse applied to a point. No zero-scale fallback, unrotation
/// shortcut, epsilon collapse, viewport conversion or integer rounding occurs.
fn inverse_point(matrix: Affine, point: [f64; 2], node: NodeId) -> Result<[f64; 2]> {
    finite_affine(matrix)?;
    let determinant = matrix.a * matrix.d - matrix.b * matrix.c;
    if determinant == 0.0 {
        return Err(DisplayError::SingularTransform(node));
    }
    if !determinant.is_finite() {
        return Err(DisplayError::NonFinite("inverse determinant"));
    }
    let x = point[0] - matrix.tx;
    let y = point[1] - matrix.ty;
    let result = [
        (matrix.d * x - matrix.c * y) / determinant,
        (-matrix.b * x + matrix.a * y) / determinant,
    ];
    if result.iter().any(|v| !v.is_finite()) {
        return Err(DisplayError::NonFinite("inverse point"));
    }
    Ok(result)
}
fn finite_affine(matrix: Affine) -> Result<()> {
    if [matrix.a, matrix.b, matrix.c, matrix.d, matrix.tx, matrix.ty]
        .iter()
        .all(|v| v.is_finite())
    {
        Ok(())
    } else {
        Err(DisplayError::NonFinite("transform"))
    }
}
fn charge(total: &mut usize, amount: usize, maximum: usize, label: &'static str) -> Result<()> {
    let next = total
        .checked_add(amount)
        .ok_or(DisplayError::Budget(label))?;
    if next > maximum {
        return Err(DisplayError::Budget(label));
    }
    *total = next;
    Ok(())
}
fn malformed(node: NodeId, reason: &'static str) -> DisplayError {
    DisplayError::InvalidScene { node, reason }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::scene::{ColorTransformF64, PathSegment};
    use crate::gfx::{ColorTransform, DisplayObject, Matrix};

    fn fixture(singular_child: bool) -> Scene<'static> {
        let source =
            |id, parent, children, raw: Option<DisplayObject>, kind, world_matrix| SceneNode {
                id,
                parent,
                children,
                segment: if id == 0 {
                    None
                } else {
                    Some(PathSegment {
                        depth: id as u16,
                        name: Some(format!("n{id}")),
                    })
                },
                placement_movie: 37,
                raw,
                kind,
                world_matrix,
                world_color: ColorTransformF64::from(ColorTransform::default()),
                effective_visible: true,
                masks: Vec::new(),
                is_mask: false,
                scaling_grids: Vec::new(),
                unsupported: Vec::new(),
                actions: Vec::new(),
                background: None,
            };
        let parent = Matrix {
            scale_x: 0,
            scale_y: 0,
            rotate_skew_0: 2 * 65536,
            rotate_skew_1: -3 * 65536,
            translate_x: 100,
            translate_y: 200,
        };
        let child = Matrix {
            scale_x: if singular_child { 0 } else { 65536 },
            scale_y: 65536,
            translate_x: 10,
            translate_y: 20,
            ..Matrix::default()
        };
        let parent_world: Affine = parent.into();
        let child_world = parent_world.compose(child.into());
        Scene {
            root: 0,
            movies: BTreeMap::new(),
            nodes: vec![
                source(0, None, vec![1, 3], None, NodeKind::Root, Affine::default()),
                source(
                    1,
                    Some(0),
                    vec![2],
                    Some(DisplayObject {
                        matrix: parent,
                        ..Default::default()
                    }),
                    NodeKind::ClassOnly,
                    parent_world,
                ),
                source(
                    2,
                    Some(1),
                    vec![],
                    Some(DisplayObject {
                        matrix: child,
                        ..Default::default()
                    }),
                    NodeKind::ClassOnly,
                    child_world,
                ),
                source(
                    3,
                    Some(0),
                    vec![],
                    Some(DisplayObject::default()),
                    NodeKind::ClassOnly,
                    Affine::default(),
                ),
            ],
            actions: Vec::new(),
            opaque_nodes: Vec::new(),
            unbound_action_tags: Vec::new(),
        }
    }
    fn request(context: usize, property: MovieProperty) -> PropertyRequest {
        PropertyRequest {
            origin: None,
            context: ObjectId(context),
            target: Value::string(""),
            property,
            source_offset: None,
        }
    }
    fn state() -> DisplayState {
        let mut state = DisplayState::from_scene(&fixture(false), Limits::default()).unwrap();
        state.bind_object(ObjectId(10), 1).unwrap();
        state.bind_object(ObjectId(11), 2).unwrap();
        state
    }

    fn visibility_fixture() -> (Vm, DisplayState, Vec<AttachedClipId>) {
        use crate::avm1::vm::{AttachedClipSpec, AttachedEventState};
        use crate::gfx::{
            Movie,
            scene::{MovieRef, NoImports},
        };
        fn tag(code: u16, body: &[u8]) -> Vec<u8> {
            assert!(body.len() < 63);
            let mut bytes = ((code << 6) | body.len() as u16).to_le_bytes().to_vec();
            bytes.extend(body);
            bytes
        }
        fn place(id: u16, depth: u16) -> Vec<u8> {
            let mut body = vec![6];
            body.extend(depth.to_le_bytes());
            body.extend(id.to_le_bytes());
            body.push(0);
            tag(26, &body)
        }
        fn sprite(id: u16, child: bool) -> Vec<u8> {
            let mut body = id.to_le_bytes().to_vec();
            body.extend(1u16.to_le_bytes());
            if child {
                body.extend(place(2, 1));
            }
            body.extend(tag(1, &[]));
            body.extend(tag(0, &[]));
            tag(39, &body)
        }
        let mut bytes = vec![b'G', b'F', b'X', 15, 0, 0, 0, 0, 8, 0, 0, 24, 1, 0];
        bytes.extend(sprite(2, false));
        bytes.extend(sprite(1, true));
        bytes.extend(place(1, 1));
        bytes.extend(place(2, 2));
        bytes.extend(tag(1, &[]));
        bytes.extend(tag(0, &[]));
        let len = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&len.to_le_bytes());
        let movie = Movie::parse(&bytes).unwrap();
        let scene =
            Scene::first_frame(MovieRef::new(37, &movie), &NoImports, Default::default()).unwrap();
        assert_eq!(scene.nodes.len(), 4);
        let mut state = DisplayState::from_scene(&scene, Default::default()).unwrap();
        let mut vm = Vm::new(15, Default::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        state.bind_movie_source(LoadedMovieSourceId(5), 37).unwrap();
        let mut clips = Vec::new();
        for node in &scene.nodes {
            let clip = vm
                .create_attached_movie_clip(AttachedClipSpec {
                    target: DisplayTargetId(700 + node.id as u64),
                    source: AttachedClipSource {
                        movie: LoadedMovieSourceId(5),
                        sprite_id: node
                            .kind
                            .definition()
                            .map_or(0, |definition| definition.character_id),
                        header_version: 15,
                    },
                    parent: node.parent.map(|parent| clips[parent]),
                    event_state: AttachedEventState {
                        target_flags: 0x5200_4180,
                        load_flags: 8,
                        source_flags: 0,
                    },
                })
                .unwrap();
            if node.id == 0 {
                vm.adopt_attached_root(clip).unwrap();
            }
            state.bind_attached(&vm, clip, node.id).unwrap();
            clips.push(clip);
        }
        (vm, state, clips)
    }
    fn visibility_request(
        vm: &Vm,
        clip: AttachedClipId,
        visible: bool,
    ) -> AttachedVisibilityRequest {
        let info = vm.attached_info(clip).unwrap();
        AttachedVisibilityRequest {
            root: vm.root(),
            context: vm.root(),
            receiver: clip,
            owner: info.owner,
            target: info.target,
            expected_local: info.event_state.target_flags & 0x4000 != 0,
            visible,
            origin: None,
        }
    }
    // Simulate only the documented VM/host transaction boundary. The native
    // named setter/profile has its own VM tests; these tests create no defaults.
    fn visibility_commit(
        vm: &mut Vm,
        state: &mut DisplayState,
        request: AttachedVisibilityRequest,
    ) {
        let mut flags = vm.attached_event_state(request.receiver).unwrap();
        flags.target_flags =
            (flags.target_flags & !0x4000) | if request.visible { 0x4000 } else { 0 };
        let clip = request.receiver;
        let commit = state.prepare_attached_visibility(request).unwrap();
        vm.update_attached_event_state(clip, flags).unwrap();
        commit();
    }

    #[test]
    fn staged_visibility_cancels_or_commits_without_changing_matrices_or_child_local_bits() {
        let (mut vm, mut state, clips) = visibility_fixture();
        let owner = vm.attached_owner(clips[1]).unwrap();
        state
            .write_property(request(owner.0, MovieProperty::X), Value::Number(10.25))
            .unwrap();
        let matrices: Vec<_> = state
            .nodes
            .iter()
            .map(|n| (n.local_matrix, n.world_matrix))
            .collect();
        let before = state.stats();
        let traces = state.traces().len();
        let req = visibility_request(&vm, clips[1], false);
        drop(state.prepare_attached_visibility(req.clone()).unwrap());
        assert!(
            state
                .nodes
                .iter()
                .all(|n| n.local_visible && n.effective_visible)
        );
        assert_eq!(state.stats().revision, before.revision);
        assert_eq!(state.traces().len(), traces);
        visibility_commit(&mut vm, &mut state, req);
        assert!(!state.nodes[1].local_visible);
        assert!(!state.nodes[1].effective_visible);
        assert!(state.nodes[2].local_visible);
        assert!(!state.nodes[2].effective_visible);
        assert!(state.nodes[0].effective_visible && state.nodes[3].effective_visible);
        assert_eq!(state.stats().revision, before.revision + 1);
        assert_eq!(state.traces().len(), traces + 1);
        assert_eq!(
            state
                .nodes
                .iter()
                .map(|n| (n.local_matrix, n.world_matrix))
                .collect::<Vec<_>>(),
            matrices
        );
        // A hidden ancestor does not change the child's expected LOCAL bit.
        let req = visibility_request(&vm, clips[2], true);
        visibility_commit(&mut vm, &mut state, req);
        assert!(state.nodes[2].local_visible && !state.nodes[2].effective_visible);
        let req = visibility_request(&vm, clips[1], true);
        visibility_commit(&mut vm, &mut state, req);
        assert!(
            state
                .nodes
                .iter()
                .all(|n| n.local_visible && n.effective_visible)
        );
        assert_eq!(
            state
                .nodes
                .iter()
                .map(|n| (n.local_matrix, n.world_matrix))
                .collect::<Vec<_>>(),
            matrices
        );
        assert!(
            state
                .nodes
                .iter()
                .all(|n| n.source.raw.as_ref().is_none_or(|raw| raw.visible))
        );
    }

    #[test]
    fn visibility_preflight_rejects_identity_coherence_and_budget_errors_before_publish() {
        let (vm, mut state, clips) = visibility_fixture();
        let req = visibility_request(&vm, clips[1], false);
        let bad = vec![
            AttachedVisibilityRequest {
                root: ObjectId(999),
                ..req.clone()
            },
            AttachedVisibilityRequest {
                context: ObjectId(999),
                ..req.clone()
            },
            AttachedVisibilityRequest {
                owner: vm.root(),
                ..req.clone()
            },
            AttachedVisibilityRequest {
                target: DisplayTargetId(999),
                ..req.clone()
            },
            AttachedVisibilityRequest {
                receiver: clips[2],
                ..req.clone()
            },
            AttachedVisibilityRequest {
                expected_local: false,
                ..req.clone()
            },
        ];
        for bad in bad {
            assert!(state.prepare_attached_visibility(bad).is_err());
            assert!(
                state
                    .nodes
                    .iter()
                    .all(|n| n.local_visible && n.effective_visible)
            );
            assert_eq!(state.stats().revision, 0);
            assert!(state.traces().is_empty());
        }
        state.limits.max_recomputed_nodes = 1;
        assert!(matches!(
            state.prepare_attached_visibility(req.clone()),
            Err(DisplayError::Budget("descendant visibility work"))
        ));
        state.limits.max_recomputed_nodes = 100;
        state.stats.revision = u64::MAX;
        assert!(matches!(
            state.prepare_attached_visibility(req),
            Err(DisplayError::Budget("revision"))
        ));
        assert!(
            state
                .nodes
                .iter()
                .all(|n| n.local_visible && n.effective_visible)
        );
        assert!(state.traces().is_empty());
    }

    #[test]
    fn generic_visible_writes_cannot_bypass_bound_attached_state() {
        let (vm, mut state, clips) = visibility_fixture();
        let owner = vm.attached_owner(clips[1]).unwrap();
        for target in [
            Value::DisplayClip(clips[1]),
            Value::Object(owner),
            Value::string(""),
        ] {
            let mut req = request(owner.0, MovieProperty::Visible);
            req.target = target;
            assert!(matches!(
                state.write_property(req, Value::Bool(false)),
                Err(DisplayError::UnsupportedProperty {
                    property: MovieProperty::Visible,
                    write: true
                })
            ));
        }
        assert!(
            state
                .nodes
                .iter()
                .all(|n| n.local_visible && n.effective_visible)
        );
        assert_eq!(state.stats().revision, 0);
        assert!(state.traces().is_empty());
    }

    #[test]
    fn named_script_write_commits_real_frontend_host_and_vm_visibility_together() {
        use crate::{
            attached_name_profile::{ExpectedProfile, NativeNameProfile},
            avm1::{Program, vm::AttachedVisibilityBaseProfile},
            frontend::host::FrontendHost,
        };
        use std::sync::Arc;
        let (mut vm, state, clips) = visibility_fixture();
        // Synthetic authenticated-profile plumbing; no private native inventory
        // is embedded. The native ID7 mapping has separate owned-binary evidence.
        let mut inventory = format!(
            "MHF-ATTACHED-NAMES\t1\nbuild\t1.52.0.1700\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
            "ab".repeat(32)
        );
        for id in 0..117 {
            let alias = id <= 21 || (110..=115).contains(&id);
            let key = if id == 7 {
                "_visible".to_owned()
            } else {
                format!("{}test{id:03}", if alias { "_" } else { "" })
            };
            inventory.push_str(&format!("standard\t{id}\t{}\t{key}\n", u8::from(alias)));
        }
        for (role, count, prefix) in [("movieclip", 42, true), ("object", 8, false)] {
            let mut keys = vec!["constructor".to_owned()];
            if prefix {
                keys.push("__proto__".to_owned());
            }
            keys.extend((0..count).map(|id| format!("{role}{id:03}")));
            keys.sort();
            for key in keys {
                inventory.push_str(&format!("{role}\t{key}\n"));
            }
        }
        let profile = NativeNameProfile::parse_verified(
            inventory.as_bytes(),
            &ExpectedProfile {
                build: "1.52.0.1700",
                source_sha256: [0xab; 32],
                inventory_sha256: [0x51; 32],
            },
            |_| Ok([0x51; 32]),
        )
        .unwrap();
        vm.install_attached_name_profile(Arc::new(profile)).unwrap();
        let receiver = clips[1];
        let owner = vm.attached_owner(receiver).unwrap();
        vm.bind_attached_target_status(receiver, 0x5a5a_0140)
            .unwrap();
        vm.bind_attached_visibility_policy(clips[0], false).unwrap();
        vm.bind_attached_visibility_profile(
            receiver,
            AttachedVisibilityBaseProfile::OrdinarySuccessfulAllocationProjection,
        )
        .unwrap();
        let mut host = FrontendHost::install(&mut vm, 15).unwrap();
        host.attach_display(state).unwrap();
        // This is an artificial script that exercises the real SetMember seam,
        // not an original constructor, panel hide workaround or rendered probe.
        let mut bytes = vec![0x96, 6, 0, 0, b't', b'h', b'i', b's', 0, 0x1c];
        let mut payload = vec![0];
        payload.extend(b"_visible\0");
        payload.extend([5, 0]);
        bytes.push(0x96);
        bytes.extend((payload.len() as u16).to_le_bytes());
        bytes.extend(payload);
        let setter_offset = 500 + bytes.len();
        bytes.extend([0x4f, 0]);
        let code = vm
            .register(Arc::new(Program::parse_at(&bytes, 500).unwrap()))
            .unwrap();
        let before = vm.attached_event_state(receiver).unwrap();
        vm.execute(code, owner, &mut host).unwrap();
        assert_eq!(
            vm.attached_event_state(receiver).unwrap().target_flags,
            before.target_flags & !0x4000
        );
        assert_eq!(
            vm.attached_target_status(receiver).unwrap(),
            Some(0x5a5a_0140)
        );
        assert!(vm.attached_info(receiver).unwrap().storage.is_none());
        let display = host.display().unwrap();
        assert!(!display.nodes[1].local_visible && !display.nodes[1].effective_visible);
        assert!(display.nodes[2].local_visible && !display.nodes[2].effective_visible);
        assert!(display.nodes[3].effective_visible);
        assert_eq!(display.stats().revision, 1);
        assert_eq!(display.traces().len(), 1);
        assert_eq!(
            display.traces()[0].request.origin.unwrap().source_offset,
            setter_offset
        );
        assert_eq!(
            vm.attached_visibility_info(receiver)
                .unwrap()
                .unwrap()
                .last_write,
            display.traces()[0].request.origin
        );
    }

    #[test]
    fn current_scene_projection_preserves_source_metadata_after_host_and_input_changes() {
        use crate::gfx::{Span, scene::ActionKind};
        let mut scene = fixture(false);
        scene.actions.push(ActionNode {
            owner: 1,
            movie_id: 37,
            source_tag: None,
            bytes: Span {
                offset: 0,
                length: 0,
            },
            kind: ActionKind::Clip {
                event_flags: 8,
                key_code: None,
            },
        });
        let tag = TagRef {
            movie_id: 37,
            tag_index: 0,
        };
        scene.opaque_nodes.push(OpaqueNode {
            owner: 2,
            source: tag,
            code: 999,
            body: Span {
                offset: 0,
                length: 0,
            },
        });
        scene.unbound_action_tags.push(tag);
        let mut state = DisplayState::from_scene(&scene, Limits::default()).unwrap();
        state.bind_object(ObjectId(11), 1).unwrap();
        state
            .write_property(request(11, MovieProperty::X), Value::Number(7.0))
            .unwrap();
        state
            .write_property(request(11, MovieProperty::Visible), Value::Bool(false))
            .unwrap();
        let before = (state.stats(), state.traces().len());
        let current = state.snapshot_scene(scene.movies.clone()).unwrap();
        assert_eq!(current.actions, scene.actions);
        assert_eq!(current.opaque_nodes, scene.opaque_nodes);
        assert_eq!(current.unbound_action_tags, scene.unbound_action_tags);
        for (source, node) in scene.nodes.iter().zip(&current.nodes) {
            assert_eq!(source.raw, node.raw);
            assert_eq!(source.kind, node.kind);
            assert_eq!(source.children, node.children);
            assert_eq!(source.actions, node.actions);
            assert_eq!(source.parent, node.parent);
            assert_eq!(source.scaling_grids, node.scaling_grids);
            assert_eq!(source.world_color, node.world_color);
        }
        assert_ne!(current.nodes[2].world_matrix, scene.nodes[2].world_matrix);
        assert_eq!(
            current.nodes[2].world_matrix,
            state.node(2).unwrap().world_matrix
        );
        assert!(!current.nodes[1].effective_visible && !current.nodes[2].effective_visible);
        assert!(current.nodes[3].effective_visible);
        assert!(scene.nodes[1].raw.as_ref().unwrap().visible);
        scene.actions.clear();
        scene.opaque_nodes.clear();
        scene.unbound_action_tags.clear();
        let later = state.snapshot_scene(scene.movies.clone()).unwrap();
        assert_eq!(later.actions, current.actions);
        assert_eq!(later.opaque_nodes, current.opaque_nodes);
        assert_eq!(later.unbound_action_tags, current.unbound_action_tags);
        assert_eq!((state.stats(), state.traces().len()), before);
    }

    fn projection_movie(red: u8) -> Movie {
        let mut bytes = vec![b'G', b'F', b'X', 15, 0, 0, 0, 0, 8, 0, 0, 24, 1, 0];
        bytes.extend(((9u16 << 6) | 3).to_le_bytes());
        bytes.extend([red, 0, 0]);
        bytes.extend((1u16 << 6).to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        let len = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&len.to_le_bytes());
        Movie::parse(&bytes).unwrap()
    }

    #[test]
    fn current_scene_projection_requires_exact_movie_set_content_and_version() {
        use crate::gfx::scene::{MovieRef, NoImports};
        let movie = projection_movie(7);
        let changed = projection_movie(8);
        assert_eq!(movie.bytes().len(), changed.bytes().len());
        let scene =
            Scene::first_frame(MovieRef::new(37, &movie), &NoImports, Default::default()).unwrap();
        let state = DisplayState::from_scene(&scene, Limits::default()).unwrap();
        assert!(matches!(
            state.snapshot_scene(BTreeMap::new()),
            Err(DisplayError::SourceMovieSet)
        ));
        assert!(matches!(
            state.snapshot_scene(BTreeMap::from([(38, &movie)])),
            Err(DisplayError::SourceMovieSet)
        ));
        assert!(matches!(
            state.snapshot_scene(BTreeMap::from([(37, &movie), (38, &movie)])),
            Err(DisplayError::SourceMovieSet)
        ));
        assert!(matches!(
            state.snapshot_scene(BTreeMap::from([(37, &changed)])),
            Err(DisplayError::SourceMovieChanged(37))
        ));
        let mut copy = Movie::parse(movie.bytes()).unwrap();
        assert!(state.snapshot_scene(BTreeMap::from([(37, &copy)])).is_ok());
        copy.header.version = 14;
        assert!(matches!(
            state.snapshot_scene(BTreeMap::from([(37, &copy)])),
            Err(DisplayError::SourceMovieChanged(37))
        ));
        assert_eq!(state.stats().revision, 0);
        assert!(state.traces().is_empty());
    }

    #[test]
    fn current_scene_projection_source_hashing_is_bounded_before_adoption() {
        use crate::gfx::scene::{MovieRef, NoImports};
        let movie = projection_movie(7);
        let scene =
            Scene::first_frame(MovieRef::new(37, &movie), &NoImports, Default::default()).unwrap();
        let mut limits = Limits {
            max_source_movie_bytes: movie.bytes().len(),
            ..Limits::default()
        };
        assert!(DisplayState::from_scene(&scene, limits).is_ok());
        limits.max_source_movie_bytes -= 1;
        assert!(matches!(
            DisplayState::from_scene(&scene, limits),
            Err(DisplayError::Budget("source movie bytes"))
        ));
    }

    #[test]
    fn inverse_respects_rotated_nonuniform_ancestors_and_pixel_boundary() {
        let mut state = state();
        // Child-local (60,-40) -> parent-local (70,-20) -> stage (160,340).
        state.set_pointer_stage_twips(Some([160.0, 340.0])).unwrap();
        assert_eq!(state.local_mouse_twips(2).unwrap(), [60.0, -40.0]);
        assert_eq!(
            state
                .read_property(request(11, MovieProperty::MouseX))
                .unwrap(),
            Value::Number(3.0)
        );
        assert_eq!(
            state
                .read_property(request(11, MovieProperty::MouseY))
                .unwrap(),
            Value::Number(-2.0)
        );
        assert_eq!(
            state.read_property(request(11, MovieProperty::X)).unwrap(),
            Value::Number(0.5)
        );
    }

    #[test]
    fn writes_update_only_subtree_and_preserve_source_and_local_visibility() {
        let scene = fixture(false);
        let mut state = state();
        state
            .write_property(request(10, MovieProperty::X), Value::Number(10.25))
            .unwrap();
        assert_eq!(state.node(1).unwrap().local_matrix.tx, 205.0);
        assert_eq!(
            state.node(2).unwrap().world_matrix.transform(0.0, 0.0),
            [145.0, 220.0]
        );
        assert_eq!(state.node(3).unwrap().world_matrix, Affine::default());
        state
            .write_property(request(10, MovieProperty::Visible), Value::Bool(false))
            .unwrap();
        assert!(!state.node(1).unwrap().effective_visible);
        assert!(!state.node(2).unwrap().effective_visible);
        assert_eq!(
            state
                .read_property(request(11, MovieProperty::Visible))
                .unwrap(),
            Value::Bool(true)
        );
        assert!(state.node(3).unwrap().effective_visible);
        let snapshot = state.snapshot_nodes();
        for (original, current) in scene.nodes.iter().zip(&snapshot) {
            assert_eq!(original.raw, current.raw);
            assert_eq!(original.kind, current.kind);
            assert_eq!(original.id, current.id);
            assert_eq!(original.parent, current.parent);
            assert_eq!(original.children, current.children);
            assert_eq!(original.placement_movie, current.placement_movie);
        }
        assert_eq!(
            snapshot[2].world_matrix,
            state.node(2).unwrap().world_matrix
        );
        assert_eq!(
            state.node(1).unwrap().source.world_matrix,
            scene.nodes[1].world_matrix
        );
        assert_eq!(state.stats().recomputed_nodes, 4);
    }

    #[test]
    fn rejects_missing_pointer_singular_inverse_unknown_targets_and_coercions() {
        let mut state = DisplayState::from_scene(&fixture(true), Limits::default()).unwrap();
        state.bind_object(ObjectId(11), 2).unwrap();
        assert_eq!(
            state
                .read_property(request(11, MovieProperty::MouseX))
                .unwrap_err(),
            DisplayError::PointerUnavailable
        );
        state.set_pointer_stage_twips(Some([1.0, 2.0])).unwrap();
        assert_eq!(
            state
                .read_property(request(11, MovieProperty::MouseX))
                .unwrap_err(),
            DisplayError::SingularTransform(2)
        );
        assert!(matches!(
            state.read_property(request(99, MovieProperty::X)),
            Err(DisplayError::UnboundObject(_))
        ));
        let mut path = request(11, MovieProperty::X);
        path.target = Value::string("_root.n1");
        assert_eq!(
            state.read_property(path).unwrap_err(),
            DisplayError::UnsupportedTarget
        );
        assert!(matches!(
            state.read_property(request(11, MovieProperty::Width)),
            Err(DisplayError::UnsupportedProperty { .. })
        ));
        assert!(matches!(
            state.write_property(request(11, MovieProperty::MouseX), Value::Number(1.0)),
            Err(DisplayError::UnsupportedProperty { write: true, .. })
        ));
        assert!(matches!(
            state.write_property(request(11, MovieProperty::X), Value::string("1")),
            Err(DisplayError::UnsupportedValue(_))
        ));
        assert!(matches!(
            state.write_property(request(11, MovieProperty::Visible), Value::Number(1.0)),
            Err(DisplayError::UnsupportedValue(_))
        ));
        assert!(
            state
                .set_pointer_stage_twips(Some([f64::NAN, 0.0]))
                .is_err()
        );
        assert_eq!(state.pointer_stage_twips(), Some([1.0, 2.0]));
    }

    #[test]
    fn budget_and_overflow_rejections_leave_geometry_unchanged() {
        let scene = fixture(false);
        let mut limited = DisplayState::from_scene(
            &scene,
            Limits {
                max_recomputed_nodes: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        limited.bind_object(ObjectId(10), 1).unwrap();
        let before = limited.node(1).unwrap().local_matrix;
        assert!(matches!(
            limited.write_property(request(10, MovieProperty::X), Value::Number(10.0)),
            Err(DisplayError::Budget(_))
        ));
        assert_eq!(limited.node(1).unwrap().local_matrix, before);
        assert_eq!(limited.stats().revision, 0);
        let mut state = state();
        let before = state.node(2).unwrap().local_matrix;
        assert!(matches!(
            state.write_property(
                request(11, MovieProperty::X),
                Value::Number(f64::MAX / 30.0)
            ),
            Err(DisplayError::NonFinite(_))
        ));
        assert_eq!(state.node(2).unwrap().local_matrix, before);
        assert_eq!(state.stats().revision, 0);
        assert!(state.bind_object(ObjectId(12), 2).is_err());
        let mut malformed_scene = fixture(false);
        malformed_scene.nodes[0].children.push(1);
        assert!(DisplayState::from_scene(&malformed_scene, Limits::default()).is_err());
    }
}
