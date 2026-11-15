//! Bounded structural scene snapshots. No script, filter or geometry execution.

use super::{CharacterKind, ColorTransform, DisplayObject, Matrix, Movie, Rect, Span, TagData};
use std::collections::{BTreeMap, BTreeSet};
use std::io;

pub type MovieId = usize;
pub type NodeId = usize;

#[derive(Clone, Copy, Debug)]
pub struct MovieRef<'a> {
    pub id: MovieId,
    pub movie: &'a Movie,
}
impl<'a> MovieRef<'a> {
    pub const fn new(id: MovieId, movie: &'a Movie) -> Self {
        Self { id, movie }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ImportRequest<'a> {
    pub from_movie: MovieId,
    pub local_character_id: u16,
    pub url: &'a str,
    pub symbol: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedCharacter<'a> {
    pub movie: MovieRef<'a>,
    pub character_id: u16,
}

/// Resolution is supplied by the host, using its explicit asset/library bindings.
pub trait ImportResolver {
    fn resolve(&self, request: ImportRequest<'_>) -> io::Result<Option<ResolvedCharacter<'_>>>;
}

pub struct NoImports;
impl ImportResolver for NoImports {
    fn resolve(&self, _: ImportRequest<'_>) -> io::Result<Option<ResolvedCharacter<'_>>> {
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_actions: usize,
    pub max_tag_visits: usize,
    pub max_import_hops: usize,
    pub max_metadata_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_depth: 64,
            max_actions: 100_000,
            max_tag_visits: 2_000_000,
            max_import_hops: 32,
            max_metadata_bytes: 64 * 1024 * 1024,
        }
    }
}

/// x' = a*x + c*y + tx; y' = b*x + d*y + ty. Translation is in twips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}
impl Default for Affine {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}
impl From<Matrix> for Affine {
    fn from(raw: Matrix) -> Self {
        Self {
            a: f64::from(raw.scale_x) / 65_536.0,
            b: f64::from(raw.rotate_skew_0) / 65_536.0,
            c: f64::from(raw.rotate_skew_1) / 65_536.0,
            d: f64::from(raw.scale_y) / 65_536.0,
            tx: f64::from(raw.translate_x),
            ty: f64::from(raw.translate_y),
        }
    }
}
impl Affine {
    pub fn transform(self, x: f64, y: f64) -> [f64; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }
    /// Apply child first, then self (the parent).
    pub fn compose(self, child: Self) -> Self {
        let [tx, ty] = self.transform(child.tx, child.ty);
        Self {
            a: self.a * child.a + self.c * child.b,
            b: self.b * child.a + self.d * child.b,
            c: self.a * child.c + self.c * child.d,
            d: self.b * child.c + self.d * child.d,
            tx,
            ty,
        }
    }
    fn finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.tx, self.ty]
            .iter()
            .all(|v| v.is_finite())
    }
}

/// Unclamped affine RGBA transform. Addends retain 0..255 channel units.
/// Raw local transforms remain on nodes; native intermediate clamping is unproved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTransformF64 {
    pub multiply: [f64; 4],
    pub add: [f64; 4],
}
impl Default for ColorTransformF64 {
    fn default() -> Self {
        Self {
            multiply: [1.0; 4],
            add: [0.0; 4],
        }
    }
}
impl From<ColorTransform> for ColorTransformF64 {
    fn from(raw: ColorTransform) -> Self {
        Self {
            multiply: raw.multiply.map(|v| f64::from(v) / 256.0),
            add: raw.add.map(f64::from),
        }
    }
}
impl ColorTransformF64 {
    pub fn compose(self, child: Self) -> Self {
        let mut result = self;
        for channel in 0..4 {
            result.multiply[channel] *= child.multiply[channel];
            result.add[channel] += self.multiply[channel] * child.add[channel];
        }
        result
    }
    pub fn transform_unclamped(self, rgba: [f64; 4]) -> [f64; 4] {
        std::array::from_fn(|i| rgba[i] * self.multiply[i] + self.add[i])
    }
    fn finite(self) -> bool {
        self.multiply.iter().chain(&self.add).all(|v| v.is_finite())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagRef {
    pub movie_id: MovieId,
    pub tag_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionRef {
    pub movie_id: MovieId,
    pub character_id: u16,
    pub tag_index: usize,
    pub kind: CharacterKind,
    pub bounds: Option<Rect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportLink {
    pub source: TagRef,
    pub local_character_id: u16,
    pub url: String,
    pub symbol: String,
    pub target: Option<(MovieId, u16)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedImport {
    NoBinding,
    MissingTarget {
        movie_id: MovieId,
        character_id: u16,
    },
    Cycle {
        movie_id: MovieId,
        character_id: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Root,
    Definition(DefinitionRef),
    /// The import boundary remains explicit even when its final definition resolves.
    Import {
        chain: Vec<ImportLink>,
        resolved: Option<DefinitionRef>,
        unresolved: Option<UnresolvedImport>,
    },
    MissingCharacter {
        movie_id: MovieId,
        character_id: u16,
    },
    ClassOnly,
}
impl NodeKind {
    pub fn definition(&self) -> Option<DefinitionRef> {
        match self {
            Self::Definition(value) => Some(*value),
            Self::Import { resolved, .. } => *resolved,
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unsupported {
    RecursiveSprite(DefinitionRef),
    EmptyTimeline { movie_id: MovieId, timeline_id: u16 },
    ButtonStateNotInstantiated,
    NonDisplayDefinition(CharacterKind),
    InvalidClipDepth { depth: u16, clip_depth: u16 },
    ClassBindingNotExecuted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalingGrid {
    pub source: TagRef,
    pub splitter: Rect,
}

/// One path segment; following parent links preserves full instance paths without
/// repeatedly copying ancestor names. Duplicate/unnamed instances remain distinct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathSegment {
    pub depth: u16,
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SceneNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub segment: Option<PathSegment>,
    /// Movie containing this placement, even when the character is imported.
    pub placement_movie: MovieId,
    pub raw: Option<DisplayObject>,
    pub kind: NodeKind,
    pub world_matrix: Affine,
    pub world_color: ColorTransformF64,
    pub effective_visible: bool,
    /// Ancestor and active sibling mask sources; geometry is not rasterized here.
    pub masks: Vec<NodeId>,
    /// True for a valid nonzero clipping layer, whose own geometry is mask-only.
    pub is_mask: bool,
    pub scaling_grids: Vec<ScalingGrid>,
    pub unsupported: Vec<Unsupported>,
    pub actions: Vec<usize>,
    pub background: Option<[u8; 3]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Frame {
        timeline_id: u16,
        frame_index: u16,
    },
    InitDeclaration {
        sprite_id: u16,
    },
    SpriteInitializer {
        sprite_id: u16,
    },
    Clip {
        event_flags: u32,
        key_code: Option<u8>,
    },
}

/// Explicit unexecuted script binding. Repeated instances can bind the same bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionNode {
    pub owner: NodeId,
    pub movie_id: MovieId,
    pub source_tag: Option<TagRef>,
    pub bytes: Span,
    pub kind: ActionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueNode {
    pub owner: NodeId,
    pub source: TagRef,
    pub code: u16,
    pub body: Span,
}

#[derive(Debug)]
pub struct Scene<'a> {
    pub root: NodeId,
    pub movies: BTreeMap<MovieId, &'a Movie>,
    /// Preorder arena. Each node's children are in ascending local display depth.
    pub nodes: Vec<SceneNode>,
    pub actions: Vec<ActionNode>,
    pub opaque_nodes: Vec<OpaqueNode>,
    /// Action tags in registered movies that have no binding in this snapshot.
    pub unbound_action_tags: Vec<TagRef>,
}
impl<'a> Scene<'a> {
    pub fn first_frame(
        root: MovieRef<'a>,
        resolver: &'a impl ImportResolver,
        limits: Limits,
    ) -> io::Result<Self> {
        if limits.max_depth > 256 || limits.max_import_hops > 256 || limits.max_nodes == 0 {
            return Err(invalid("invalid scene depth/import/node limits"));
        }
        let mut builder = Builder::new(resolver, limits);
        builder.register(root)?;
        builder.metadata(std::mem::size_of::<SceneNode>())?;
        builder.scene.nodes.push(SceneNode {
            id: 0,
            parent: None,
            children: Vec::new(),
            segment: None,
            placement_movie: root.id,
            raw: None,
            kind: NodeKind::Root,
            world_matrix: Affine::default(),
            world_color: ColorTransformF64::default(),
            effective_visible: true,
            masks: Vec::new(),
            is_mask: false,
            scaling_grids: Vec::new(),
            unsupported: Vec::new(),
            actions: Vec::new(),
            background: None,
        });
        builder.expand_timeline(0, root, 0, &mut vec![(root.id, 0)], 0)?;
        builder.scene.unbound_action_tags = builder.unbound_actions.into_iter().collect();
        Ok(builder.scene)
    }

    /// Structured path from root to instance; names are not interpreted as paths.
    pub fn path(&self, id: NodeId) -> io::Result<Vec<&PathSegment>> {
        let mut path = Vec::new();
        let mut current = Some(id);
        let mut steps = 0;
        while let Some(index) = current {
            let node = self
                .nodes
                .get(index)
                .ok_or_else(|| invalid("unknown scene node"))?;
            steps += 1;
            if steps > self.nodes.len() {
                return Err(invalid("scene parent cycle"));
            }
            if let Some(segment) = &node.segment {
                path.push(segment);
            }
            current = node.parent;
        }
        path.reverse();
        Ok(path)
    }
}

#[derive(Default)]
struct MovieIndex {
    grids: BTreeMap<u16, Vec<ScalingGrid>>,
    initializers: BTreeMap<u16, Vec<TagRef>>,
}

struct Builder<'a, R> {
    resolver: &'a R,
    limits: Limits,
    tag_visits: usize,
    metadata_bytes: usize,
    scene: Scene<'a>,
    indexes: BTreeMap<MovieId, MovieIndex>,
    unbound_actions: BTreeSet<TagRef>,
}
impl<'a, R: ImportResolver> Builder<'a, R> {
    fn new(resolver: &'a R, limits: Limits) -> Self {
        Self {
            resolver,
            limits,
            tag_visits: 0,
            metadata_bytes: 0,
            scene: Scene {
                root: 0,
                movies: BTreeMap::new(),
                nodes: Vec::new(),
                actions: Vec::new(),
                opaque_nodes: Vec::new(),
                unbound_action_tags: Vec::new(),
            },
            indexes: BTreeMap::new(),
            unbound_actions: BTreeSet::new(),
        }
    }
    fn metadata(&mut self, bytes: usize) -> io::Result<()> {
        self.metadata_bytes = self
            .metadata_bytes
            .checked_add(bytes)
            .ok_or_else(|| invalid("scene metadata size overflow"))?;
        if self.metadata_bytes > self.limits.max_metadata_bytes {
            return Err(invalid("scene metadata budget exceeded"));
        }
        Ok(())
    }
    fn visit_tags(&mut self, count: usize) -> io::Result<()> {
        self.tag_visits = self
            .tag_visits
            .checked_add(count)
            .ok_or_else(|| invalid("scene tag budget overflow"))?;
        if self.tag_visits > self.limits.max_tag_visits {
            return Err(invalid("scene tag visit budget exceeded"));
        }
        Ok(())
    }
    fn register(&mut self, source: MovieRef<'a>) -> io::Result<()> {
        if let Some(old) = self.scene.movies.get(&source.id) {
            if !std::ptr::eq(*old, source.movie) {
                return Err(invalid("MovieId names different movie objects"));
            }
            return Ok(());
        }
        self.visit_tags(source.movie.tags.len())?;
        let mut index = MovieIndex::default();
        for (tag_index, tag) in source.movie.tags.iter().enumerate() {
            let reference = TagRef {
                movie_id: source.id,
                tag_index,
            };
            match &tag.data {
                TagData::ScalingGrid { id, splitter } => {
                    self.metadata(std::mem::size_of::<ScalingGrid>())?;
                    index.grids.entry(*id).or_default().push(ScalingGrid {
                        source: reference,
                        splitter: *splitter,
                    });
                }
                TagData::Actions { sprite_id, .. } => {
                    self.metadata(std::mem::size_of::<TagRef>() * 2)?;
                    self.unbound_actions.insert(reference);
                    if let Some(id) = sprite_id {
                        index.initializers.entry(*id).or_default().push(reference);
                    }
                }
                _ => {}
            }
        }
        self.scene.movies.insert(source.id, source.movie);
        self.indexes.insert(source.id, index);
        Ok(())
    }
    fn action(&mut self, action: ActionNode) -> io::Result<()> {
        if self.scene.actions.len() >= self.limits.max_actions {
            return Err(invalid("scene action budget exceeded"));
        }
        action
            .bytes
            .bytes(self.scene.movies[&action.movie_id].bytes())?;
        self.metadata(std::mem::size_of::<ActionNode>() + std::mem::size_of::<usize>())?;
        if let Some(source) = action.source_tag {
            self.unbound_actions.remove(&source);
        }
        let index = self.scene.actions.len();
        self.scene.nodes[action.owner].actions.push(index);
        self.scene.actions.push(action);
        Ok(())
    }
    fn bind_action_tag(
        &mut self,
        owner: NodeId,
        reference: TagRef,
        initializer: bool,
    ) -> io::Result<()> {
        let movie = self.scene.movies[&reference.movie_id];
        let tag = movie
            .tags
            .get(reference.tag_index)
            .ok_or_else(|| invalid("bad action tag index"))?;
        let TagData::Actions { sprite_id, bytes } = tag.data else {
            return Err(invalid("expected action tag"));
        };
        let kind = match sprite_id {
            Some(sprite_id) if initializer => ActionKind::SpriteInitializer { sprite_id },
            Some(sprite_id) => ActionKind::InitDeclaration { sprite_id },
            None => ActionKind::Frame {
                timeline_id: tag.timeline_id,
                frame_index: tag.frame_index,
            },
        };
        self.action(ActionNode {
            owner,
            movie_id: reference.movie_id,
            source_tag: Some(reference),
            bytes,
            kind,
        })
    }
    fn resolve(&mut self, source: MovieRef<'a>, id: u16) -> io::Result<NodeKind> {
        let mut current = source;
        let mut character_id = id;
        let mut chain = Vec::new();
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert((current.id, character_id)) {
                return Ok(NodeKind::Import {
                    chain,
                    resolved: None,
                    unresolved: Some(UnresolvedImport::Cycle {
                        movie_id: current.id,
                        character_id,
                    }),
                });
            }
            let Some(character) = current.movie.dictionary.get(&character_id) else {
                return Ok(if chain.is_empty() {
                    NodeKind::MissingCharacter {
                        movie_id: current.id,
                        character_id,
                    }
                } else {
                    NodeKind::Import {
                        chain,
                        resolved: None,
                        unresolved: Some(UnresolvedImport::MissingTarget {
                            movie_id: current.id,
                            character_id,
                        }),
                    }
                });
            };
            let tag = current
                .movie
                .tags
                .get(character.tag_index)
                .ok_or_else(|| invalid("definition tag outside movie"))?;
            if character.kind != CharacterKind::Imported {
                let definition = DefinitionRef {
                    movie_id: current.id,
                    character_id,
                    tag_index: character.tag_index,
                    kind: character.kind,
                    bounds: character.bounds,
                };
                return Ok(if chain.is_empty() {
                    NodeKind::Definition(definition)
                } else {
                    NodeKind::Import {
                        chain,
                        resolved: Some(definition),
                        unresolved: None,
                    }
                });
            }
            if chain.len() >= self.limits.max_import_hops {
                return Err(invalid("scene import hop budget exceeded"));
            }
            let TagData::ImportAssets { url, symbols } = &tag.data else {
                return Err(invalid("import dictionary entry has no ImportAssets tag"));
            };
            let symbol = symbols
                .iter()
                .find(|symbol| symbol.id == character_id)
                .ok_or_else(|| invalid("imported character has no symbol"))?;
            self.metadata(
                url.len()
                    .checked_add(symbol.name.len())
                    .and_then(|n| n.checked_add(std::mem::size_of::<ImportLink>()))
                    .ok_or_else(|| invalid("import metadata overflow"))?,
            )?;
            let target = self.resolver.resolve(ImportRequest {
                from_movie: current.id,
                local_character_id: character_id,
                url,
                symbol: &symbol.name,
            })?;
            chain.push(ImportLink {
                source: TagRef {
                    movie_id: current.id,
                    tag_index: character.tag_index,
                },
                local_character_id: character_id,
                url: url.clone(),
                symbol: symbol.name.clone(),
                target: target.map(|target| (target.movie.id, target.character_id)),
            });
            let Some(target) = target else {
                return Ok(NodeKind::Import {
                    chain,
                    resolved: None,
                    unresolved: Some(UnresolvedImport::NoBinding),
                });
            };
            self.register(target.movie)?;
            current = target.movie;
            character_id = target.character_id;
        }
    }
    fn expand_timeline(
        &mut self,
        owner: NodeId,
        source: MovieRef<'a>,
        timeline_id: u16,
        ancestry: &mut Vec<(MovieId, u16)>,
        depth: usize,
    ) -> io::Result<()> {
        let timeline = source
            .movie
            .timelines
            .get(&timeline_id)
            .ok_or_else(|| invalid("sprite definition has no timeline"))?;
        if timeline.frame_count == 0 {
            self.scene.nodes[owner]
                .unsupported
                .push(Unsupported::EmptyTimeline {
                    movie_id: source.id,
                    timeline_id,
                });
            return Ok(());
        }
        let mut prefix = Vec::new();
        for &index in &timeline.tags {
            self.visit_tags(2)?; // This check plus the existing static_frame walk.
            let tag = source
                .movie
                .tags
                .get(index)
                .ok_or_else(|| invalid("timeline tag outside movie"))?;
            if tag.timeline_id != timeline_id {
                return Err(invalid("timeline references another timeline's tag"));
            }
            if tag.frame_index != 0 {
                return Err(invalid("first-frame prefix has a nonzero frame index"));
            }
            prefix.push(index);
            if matches!(tag.data, TagData::ShowFrame) {
                break;
            }
        }
        let frame = source.movie.static_frame(timeline_id, 0)?;
        self.scene.nodes[owner].background = frame.background;
        for index in frame.ignored_action_tags {
            self.bind_action_tag(
                owner,
                TagRef {
                    movie_id: source.id,
                    tag_index: index,
                },
                false,
            )?;
        }
        for index in prefix {
            let tag = &source.movie.tags[index];
            if matches!(tag.data, TagData::Opaque) {
                self.metadata(std::mem::size_of::<OpaqueNode>())?;
                self.scene.opaque_nodes.push(OpaqueNode {
                    owner,
                    source: TagRef {
                        movie_id: source.id,
                        tag_index: index,
                    },
                    code: tag.code,
                    body: tag.body,
                });
            }
        }
        let inherited_masks = self.scene.nodes[owner].masks.clone();
        let mut sibling_masks: Vec<(NodeId, u16)> = Vec::new();
        for (placement_depth, raw) in frame.objects {
            sibling_masks.retain(|(_, end)| placement_depth <= *end);
            if depth >= self.limits.max_depth {
                return Err(invalid("scene nesting budget exceeded"));
            }
            if self.scene.nodes.len() >= self.limits.max_nodes {
                return Err(invalid("scene node budget exceeded"));
            }
            let mut metadata = std::mem::size_of::<SceneNode>();
            for value in [&raw.name, &raw.class_name].into_iter().flatten() {
                metadata = metadata
                    .checked_add(value.len() * 2)
                    .ok_or_else(|| invalid("node metadata overflow"))?;
            }
            metadata = metadata
                .checked_add(raw.filters.len() * std::mem::size_of::<super::Filter>())
                .and_then(|n| {
                    n.checked_add(raw.clip_actions.len() * std::mem::size_of::<super::ClipAction>())
                })
                .and_then(|n| {
                    n.checked_add(
                        (inherited_masks.len() + sibling_masks.len())
                            * std::mem::size_of::<NodeId>(),
                    )
                })
                .ok_or_else(|| invalid("node metadata overflow"))?;
            self.metadata(metadata)?;
            let kind = match raw.character_id {
                Some(id) => self.resolve(source, id)?,
                None => NodeKind::ClassOnly,
            };
            let definition = kind.definition();
            let parent = &self.scene.nodes[owner];
            let world_matrix = parent.world_matrix.compose(raw.matrix.into());
            let world_color = parent.world_color.compose(raw.color.into());
            if !world_matrix.finite() || !world_color.finite() {
                return Err(invalid("non-finite composed scene transform"));
            }
            let mut masks = inherited_masks.clone();
            masks.extend(sibling_masks.iter().map(|(node, _)| *node));
            let mut unsupported = Vec::new();
            if raw.class_name.is_some() {
                unsupported.push(Unsupported::ClassBindingNotExecuted);
            }
            let is_mask = match raw.clip_depth {
                Some(end) if end > placement_depth => true,
                Some(end) if end != 0 => {
                    unsupported.push(Unsupported::InvalidClipDepth {
                        depth: placement_depth,
                        clip_depth: end,
                    });
                    false
                }
                _ => false,
            };
            // Keep grids attached to import aliases as well as the final target.
            // Choosing their runtime precedence is deliberately left unresolved.
            let mut grid_keys = Vec::new();
            if let NodeKind::Import { chain, .. } = &kind {
                grid_keys.extend(
                    chain
                        .iter()
                        .map(|link| (link.source.movie_id, link.local_character_id)),
                );
            } else if let Some(id) = raw.character_id {
                grid_keys.push((source.id, id));
            }
            if let Some(definition) = definition {
                let key = (definition.movie_id, definition.character_id);
                if !grid_keys.contains(&key) {
                    grid_keys.push(key);
                }
            }
            let mut grids = Vec::new();
            for (movie_id, character_id) in grid_keys {
                if let Some(values) = self.indexes[&movie_id].grids.get(&character_id) {
                    grids.extend(values.iter().copied());
                }
            }
            self.metadata(grids.len() * std::mem::size_of::<ScalingGrid>())?;
            let id = self.scene.nodes.len();
            let visible = self.scene.nodes[owner].effective_visible && raw.visible;
            let segment = PathSegment {
                depth: placement_depth,
                name: raw.name.clone(),
            };
            self.scene.nodes.push(SceneNode {
                id,
                parent: Some(owner),
                children: Vec::new(),
                segment: Some(segment),
                placement_movie: source.id,
                raw: Some(raw),
                kind,
                world_matrix,
                world_color,
                effective_visible: visible,
                masks,
                is_mask,
                scaling_grids: grids,
                unsupported,
                actions: Vec::new(),
                background: None,
            });
            self.scene.nodes[owner].children.push(id);
            let clips = self.scene.nodes[id]
                .raw
                .as_ref()
                .unwrap()
                .clip_actions
                .clone();
            for clip in clips {
                self.action(ActionNode {
                    owner: id,
                    movie_id: source.id,
                    source_tag: None,
                    bytes: clip.actions,
                    kind: ActionKind::Clip {
                        event_flags: clip.event_flags,
                        key_code: clip.key_code,
                    },
                })?;
            }
            if let Some(definition) = definition {
                self.expand_definition(id, definition, ancestry, depth + 1)?;
            }
            if is_mask {
                sibling_masks.push((
                    id,
                    self.scene.nodes[id]
                        .raw
                        .as_ref()
                        .unwrap()
                        .clip_depth
                        .unwrap(),
                ));
            }
        }
        Ok(())
    }
    fn expand_definition(
        &mut self,
        owner: NodeId,
        definition: DefinitionRef,
        ancestry: &mut Vec<(MovieId, u16)>,
        depth: usize,
    ) -> io::Result<()> {
        let target = (definition.movie_id, definition.character_id);
        match definition.kind {
            CharacterKind::Sprite => {
                if ancestry.contains(&target) {
                    self.scene.nodes[owner]
                        .unsupported
                        .push(Unsupported::RecursiveSprite(definition));
                    return Ok(());
                }
                let initializers = self.indexes[&definition.movie_id]
                    .initializers
                    .get(&definition.character_id)
                    .cloned()
                    .unwrap_or_default();
                for source in initializers {
                    self.bind_action_tag(owner, source, true)?;
                }
                ancestry.push(target);
                let source =
                    MovieRef::new(definition.movie_id, self.scene.movies[&definition.movie_id]);
                self.expand_timeline(owner, source, definition.character_id, ancestry, depth)?;
                ancestry.pop();
            }
            CharacterKind::Button => self.scene.nodes[owner]
                .unsupported
                .push(Unsupported::ButtonStateNotInstantiated),
            CharacterKind::Shape | CharacterKind::Bitmap | CharacterKind::Text => {}
            other => self.scene.nodes[owner]
                .unsupported
                .push(Unsupported::NonDisplayDefinition(other)),
        }
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::super::{Character, ClipAction, Filter, Placement, Symbol, Tag, Timeline};
    use super::*;

    fn empty() -> Movie {
        Movie::parse(&[
            b'F', b'W', b'S', 10, 18, 0, 0, 0, 8, 0, 0, 12, 1, 0, 64, 0, 0, 0,
        ])
        .unwrap()
    }
    fn tag(movie: &mut Movie, timeline: u16, code: u16, data: TagData) -> usize {
        let index = movie.tags.len();
        movie.tags.push(Tag {
            code,
            record_offset: 0,
            body: Span { offset: 0, len: 0 },
            timeline_id: timeline,
            frame_index: 0,
            data,
        });
        index
    }
    fn definition(movie: &mut Movie, id: u16, kind: CharacterKind) {
        let code = match kind {
            CharacterKind::Sprite => 39,
            CharacterKind::Button => 34,
            _ => 2,
        };
        let index = tag(movie, 0, code, TagData::Definition { id, bounds: None });
        movie.dictionary.insert(
            id,
            Character {
                id,
                kind,
                tag_index: index,
                bounds: None,
            },
        );
    }
    fn object(id: u16) -> DisplayObject {
        DisplayObject {
            character_id: Some(id),
            ..Default::default()
        }
    }
    fn timeline(
        movie: &mut Movie,
        id: u16,
        objects: Vec<(u16, DisplayObject)>,
        extra: Vec<(u16, TagData)>,
    ) {
        let mut indexes = Vec::new();
        for (depth, raw) in objects {
            let mut p = Placement::new(depth);
            p.character_id = raw.character_id;
            p.class_name = raw.class_name;
            p.matrix = Some(raw.matrix);
            p.color = Some(raw.color);
            p.ratio = Some(raw.ratio);
            p.name = raw.name;
            p.clip_depth = raw.clip_depth;
            p.filters = Some(raw.filters);
            p.blend_mode = Some(raw.blend_mode);
            p.bitmap_cache = Some(raw.bitmap_cache);
            p.visible = Some(raw.visible);
            p.opaque_background = raw.opaque_background;
            p.has_clip_actions = raw.has_clip_actions;
            p.clip_actions = raw.clip_actions;
            indexes.push(tag(movie, id, 70, TagData::Place(p)));
        }
        for (code, data) in extra {
            indexes.push(tag(movie, id, code, data));
        }
        indexes.push(tag(movie, id, 1, TagData::ShowFrame));
        indexes.push(tag(movie, id, 0, TagData::End));
        movie.timelines.insert(
            id,
            Timeline {
                id,
                frame_count: 1,
                tags: indexes,
            },
        );
    }
    fn child(scene: &Scene<'_>, parent: NodeId, depth: u16) -> NodeId {
        *scene.nodes[parent]
            .children
            .iter()
            .find(|id| scene.nodes[**id].segment.as_ref().unwrap().depth == depth)
            .unwrap()
    }

    #[test]
    fn nested_transforms_visibility_metadata_and_paths() {
        let mut movie = empty();
        definition(&mut movie, 1, CharacterKind::Sprite);
        definition(&mut movie, 2, CharacterKind::Shape);
        let mut parent = object(1);
        parent.name = Some("outer".into());
        parent.visible = false;
        parent.matrix = Matrix {
            scale_x: 0,
            scale_y: 0,
            rotate_skew_0: 65_536,
            rotate_skew_1: -65_536,
            translate_x: 10,
            translate_y: 20,
        };
        parent.color = ColorTransform {
            multiply: [128, 512, 256, 256],
            add: [10, 20, 30, 40],
        };
        let mut nested = object(2);
        nested.name = Some("inner".into());
        nested.matrix.translate_x = 5;
        nested.matrix.translate_y = 7;
        nested.color = ColorTransform {
            multiply: [512, 128, 256, 256],
            add: [8, 9, 10, 11],
        };
        nested.filters.push(Filter {
            kind: 1,
            payload: Span { offset: 0, len: 0 },
        });
        nested.blend_mode = 3;
        nested.ratio = 123;
        nested.bitmap_cache = 1;
        nested.opaque_background = Some([1, 2, 3, 4]);
        timeline(&mut movie, 1, vec![(7, nested)], vec![]);
        let splitter = Rect {
            x_min: 0,
            y_min: 0,
            x_max: 400,
            y_max: 800,
        };
        timeline(
            &mut movie,
            0,
            vec![(2, parent)],
            vec![(78, TagData::ScalingGrid { id: 2, splitter })],
        );
        let scene =
            Scene::first_frame(MovieRef::new(10, &movie), &NoImports, Limits::default()).unwrap();
        let parent = child(&scene, 0, 2);
        let nested = child(&scene, parent, 7);
        assert_eq!(
            scene.nodes[nested].world_matrix.transform(0.0, 0.0),
            [3.0, 25.0]
        );
        assert_eq!(
            scene.nodes[nested].world_matrix.transform(2.0, 3.0),
            [0.0, 27.0]
        );
        assert_eq!(scene.nodes[nested].world_color.multiply, [1.0; 4]);
        assert_eq!(
            scene.nodes[nested].world_color.add,
            [14.0, 38.0, 40.0, 51.0]
        );
        assert!(!scene.nodes[nested].effective_visible);
        let raw = scene.nodes[nested].raw.as_ref().unwrap();
        assert_eq!((raw.blend_mode, raw.ratio, raw.bitmap_cache), (3, 123, 1));
        assert_eq!(raw.filters.len(), 1);
        assert_eq!(scene.nodes[nested].scaling_grids[0].splitter, splitter);
        let path = scene.path(nested).unwrap();
        assert_eq!(path.iter().map(|p| p.depth).collect::<Vec<_>>(), [2, 7]);
        assert_eq!(path[1].name.as_deref(), Some("inner"));
    }

    #[test]
    fn crossing_masks_expire_independently_and_flow_into_sprites() {
        let mut movie = empty();
        definition(&mut movie, 1, CharacterKind::Shape);
        definition(&mut movie, 2, CharacterKind::Sprite);
        timeline(&mut movie, 2, vec![(1, object(1))], vec![]);
        let mut a = object(1);
        a.clip_depth = Some(4);
        let mut b = object(1);
        b.clip_depth = Some(6);
        let mut zero = object(1);
        zero.clip_depth = Some(0);
        timeline(
            &mut movie,
            0,
            vec![
                (1, a),
                (2, b),
                (3, object(2)),
                (4, object(1)),
                (5, object(1)),
                (6, object(1)),
                (7, zero),
            ],
            vec![],
        );
        let scene =
            Scene::first_frame(MovieRef::new(0, &movie), &NoImports, Limits::default()).unwrap();
        let a = child(&scene, 0, 1);
        let b = child(&scene, 0, 2);
        let sprite = child(&scene, 0, 3);
        assert_eq!(scene.nodes[sprite].masks, [a, b]);
        assert_eq!(scene.nodes[child(&scene, sprite, 1)].masks, [a, b]);
        assert_eq!(scene.nodes[child(&scene, 0, 4)].masks, [a, b]);
        assert_eq!(scene.nodes[child(&scene, 0, 5)].masks, [b]);
        assert_eq!(scene.nodes[child(&scene, 0, 6)].masks, [b]);
        assert!(scene.nodes[child(&scene, 0, 7)].masks.is_empty());
        assert!(!scene.nodes[child(&scene, 0, 7)].is_mask);
    }

    #[test]
    fn scripts_opaque_tags_and_button_limits_are_explicit() {
        let mut movie = empty();
        definition(&mut movie, 1, CharacterKind::Sprite);
        definition(&mut movie, 2, CharacterKind::Button);
        let bytes = Span { offset: 0, len: 1 };
        timeline(&mut movie, 1, vec![], vec![]);
        timeline(
            &mut movie,
            99,
            vec![],
            vec![(
                12,
                TagData::Actions {
                    sprite_id: None,
                    bytes,
                },
            )],
        );
        let mut sprite = object(1);
        sprite.has_clip_actions = true;
        sprite.clip_actions.push(ClipAction {
            event_flags: 0x40000,
            key_code: None,
            actions: bytes,
        });
        timeline(
            &mut movie,
            0,
            vec![(1, sprite), (2, object(2))],
            vec![
                (
                    12,
                    TagData::Actions {
                        sprite_id: None,
                        bytes,
                    },
                ),
                (
                    59,
                    TagData::Actions {
                        sprite_id: Some(1),
                        bytes,
                    },
                ),
                (82, TagData::Opaque),
            ],
        );
        let scene =
            Scene::first_frame(MovieRef::new(0, &movie), &NoImports, Limits::default()).unwrap();
        assert_eq!(scene.actions.len(), 4);
        assert!(scene.actions.iter().any(|a| matches!(
            a.kind,
            ActionKind::Clip {
                event_flags: 0x40000,
                ..
            }
        )));
        assert!(scene
            .actions
            .iter()
            .any(|a| matches!(a.kind, ActionKind::SpriteInitializer { sprite_id: 1 })));
        assert_eq!(scene.opaque_nodes.len(), 1);
        assert_eq!(scene.opaque_nodes[0].code, 82);
        assert_eq!(scene.unbound_action_tags.len(), 1);
        assert_eq!(
            scene.nodes[child(&scene, 0, 2)].unsupported,
            [Unsupported::ButtonStateNotInstantiated]
        );
        assert!(Scene::first_frame(
            MovieRef::new(0, &movie),
            &NoImports,
            Limits {
                max_actions: 0,
                ..Limits::default()
            }
        )
        .is_err());
    }

    fn imported(movie: &mut Movie, id: u16) {
        let index = tag(
            movie,
            0,
            57,
            TagData::ImportAssets {
                url: "exact-library".into(),
                symbols: vec![Symbol {
                    id,
                    name: "ExactSymbol".into(),
                }],
            },
        );
        movie.dictionary.insert(
            id,
            Character {
                id,
                kind: CharacterKind::Imported,
                tag_index: index,
                bounds: None,
            },
        );
        timeline(movie, 0, vec![(1, object(id))], vec![]);
    }
    struct Library {
        movies: Vec<Movie>,
        cycle: bool,
        conflicting_id: bool,
    }
    impl ImportResolver for Library {
        fn resolve(&self, request: ImportRequest<'_>) -> io::Result<Option<ResolvedCharacter<'_>>> {
            assert_eq!(request.url, "exact-library");
            assert_eq!(request.symbol, "ExactSymbol");
            let target = if self.cycle {
                1 - request.from_movie
            } else {
                1
            };
            Ok(Some(ResolvedCharacter {
                movie: MovieRef::new(
                    if self.conflicting_id { 0 } else { target },
                    &self.movies[target],
                ),
                character_id: 7,
            }))
        }
    }

    #[test]
    fn imports_require_explicit_bindings_and_retain_cycles() {
        let mut a = empty();
        imported(&mut a, 7);
        let mut b = empty();
        definition(&mut b, 7, CharacterKind::Sprite);
        timeline(&mut b, 7, vec![], vec![]);
        let splitter = Rect {
            x_min: 1,
            x_max: 2,
            y_min: 3,
            y_max: 4,
        };
        tag(&mut a, 0, 78, TagData::ScalingGrid { id: 7, splitter });
        tag(&mut b, 0, 78, TagData::ScalingGrid { id: 7, splitter });
        let library = Library {
            movies: vec![a, b],
            cycle: false,
            conflicting_id: false,
        };
        let scene = Scene::first_frame(
            MovieRef::new(0, &library.movies[0]),
            &library,
            Limits::default(),
        )
        .unwrap();
        assert!(
            matches!(&scene.nodes[1].kind, NodeKind::Import { chain, resolved: Some(r), unresolved: None } if chain.len()==1 && r.movie_id==1)
        );
        assert_eq!(
            scene.nodes[1]
                .scaling_grids
                .iter()
                .map(|grid| grid.source.movie_id)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert!(Scene::first_frame(
            MovieRef::new(0, &library.movies[0]),
            &library,
            Limits {
                max_import_hops: 0,
                ..Limits::default()
            }
        )
        .is_err());
        let unresolved = Scene::first_frame(
            MovieRef::new(0, &library.movies[0]),
            &NoImports,
            Limits::default(),
        )
        .unwrap();
        assert!(matches!(
            unresolved.nodes[1].kind,
            NodeKind::Import {
                unresolved: Some(UnresolvedImport::NoBinding),
                ..
            }
        ));
        let mut library = library;
        imported(&mut library.movies[1], 7);
        library.cycle = true;
        let scene = Scene::first_frame(
            MovieRef::new(0, &library.movies[0]),
            &library,
            Limits::default(),
        )
        .unwrap();
        assert!(matches!(
            scene.nodes[1].kind,
            NodeKind::Import {
                unresolved: Some(UnresolvedImport::Cycle { .. }),
                ..
            }
        ));
        library.conflicting_id = true;
        assert!(Scene::first_frame(
            MovieRef::new(0, &library.movies[0]),
            &library,
            Limits::default()
        )
        .is_err());
    }

    #[test]
    fn recursion_missing_references_and_budgets_never_invent_content() {
        let mut movie = empty();
        definition(&mut movie, 1, CharacterKind::Sprite);
        timeline(&mut movie, 1, vec![(1, object(1))], vec![]);
        timeline(&mut movie, 0, vec![(1, object(1)), (2, object(55))], vec![]);
        let scene =
            Scene::first_frame(MovieRef::new(0, &movie), &NoImports, Limits::default()).unwrap();
        assert_eq!(scene.nodes.len(), 4);
        assert!(matches!(
            scene.nodes[2].unsupported[0],
            Unsupported::RecursiveSprite(_)
        ));
        assert!(matches!(
            scene.nodes[3].kind,
            NodeKind::MissingCharacter {
                character_id: 55,
                ..
            }
        ));
        for limits in [
            Limits {
                max_nodes: 1,
                ..Limits::default()
            },
            Limits {
                max_depth: 0,
                ..Limits::default()
            },
            Limits {
                max_tag_visits: 0,
                ..Limits::default()
            },
            Limits {
                max_metadata_bytes: 0,
                ..Limits::default()
            },
        ] {
            assert!(Scene::first_frame(MovieRef::new(0, &movie), &NoImports, limits).is_err());
        }
        let mut broken = scene;
        broken.nodes[0].parent = Some(0);
        assert!(broken.path(0).is_err());
    }
}
