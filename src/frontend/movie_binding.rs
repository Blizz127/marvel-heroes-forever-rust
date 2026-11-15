//! Saved movie-binding facts under explicit, caller-supplied native conditions.
//! Does not observe native loading or create an AVM binding witness.
use super::{
    AssetLocation,
    library::{ImportOrigin, MovieLibrary, Resolution, SymbolBinding},
};
use crate::gfx::{
    CharacterKind, Header, Movie, StaticFrame, TagData,
    scene::{DefinitionRef, MovieId, NodeId, NodeKind, Scene, TagRef},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// These are caller assertions, not conclusions inferred from saved files.
/// No Default: every use must explicitly select this conditional profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConditionalProfile {
    pub final_ordinary_pair: bool,
    pub no_resource_substitution: bool,
    pub loader_compatibility_succeeded: bool,
    pub selected_resource_non_null_and_eligible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::scene::{ImportRequest, ImportResolver, MovieRef, ResolvedCharacter};

    struct Catalog {
        movies: Vec<Movie>,
        exports: Vec<BTreeMap<String, BTreeSet<u16>>>,
        bindings: Vec<SymbolBinding>,
    }
    impl Sources for Catalog {
        fn source(&self, id: MovieId) -> Option<Source<'_>> {
            self.movies.get(id).map(|movie| Source {
                location: AssetLocation {
                    package: 0,
                    export: id,
                },
                path: if id == 0 {
                    "Example.Main"
                } else {
                    "Example.Other"
                },
                movie,
                exports: &self.exports[id],
            })
        }
        fn binding(&self, id: MovieId, character: u16) -> Option<&SymbolBinding> {
            self.bindings
                .iter()
                .find(|b| b.source_movie == id && b.local_character_id == character)
        }
    }
    impl ImportResolver for Catalog {
        fn resolve(
            &self,
            request: ImportRequest<'_>,
        ) -> std::io::Result<Option<ResolvedCharacter<'_>>> {
            Ok(self
                .binding(request.from_movie, request.local_character_id)
                .and_then(|b| {
                    if let Resolution::Resolved {
                        movie,
                        character_id,
                        ..
                    } = b.resolution
                    {
                        Some(ResolvedCharacter {
                            movie: MovieRef::new(movie, &self.movies[movie]),
                            character_id,
                        })
                    } else {
                        None
                    }
                }))
        }
    }
    fn profile() -> ConditionalProfile {
        ConditionalProfile {
            final_ordinary_pair: true,
            no_resource_substitution: true,
            loader_compatibility_succeeded: true,
            selected_resource_non_null_and_eligible: true,
        }
    }
    fn tag(code: u16, body: &[u8]) -> Vec<u8> {
        assert!(body.len() < 63);
        let mut b = ((code << 6) | body.len() as u16).to_le_bytes().to_vec();
        b.extend(body);
        b
    }
    fn sprite(id: u16) -> Vec<u8> {
        let mut b = id.to_le_bytes().to_vec();
        b.extend(1u16.to_le_bytes());
        b.extend(tag(1, &[]));
        b.extend(tag(0, &[]));
        tag(39, &b)
    }
    fn place(id: u16) -> Vec<u8> {
        let mut b = vec![6, 1, 0];
        b.extend(id.to_le_bytes());
        b.push(0);
        tag(26, &b)
    }
    fn movie(version: u8, tags: Vec<Vec<u8>>) -> Movie {
        let mut b = vec![b'G', b'F', b'X', version, 0, 0, 0, 0, 8, 0, 0, 24, 1, 0];
        for t in tags {
            b.extend(t);
        }
        b.extend(tag(1, &[]));
        b.extend(tag(0, &[]));
        let size = b.len() as u32;
        b[4..8].copy_from_slice(&size.to_le_bytes());
        Movie::parse(&b).unwrap()
    }
    fn catalog(imported: bool) -> Catalog {
        if !imported {
            return Catalog {
                movies: vec![movie(15, vec![sprite(1), place(1)])],
                exports: vec![BTreeMap::new()],
                bindings: vec![],
            };
        }
        let mut import = b"other.swf\0".to_vec();
        import.extend([1, 0, 7, 0]);
        import.extend(b"Control\0");
        let mut export = vec![1, 0, 9, 0];
        export.extend(b"Control\0");
        Catalog {
            movies: vec![
                movie(15, vec![tag(57, &import), place(7)]),
                movie(11, vec![sprite(9), tag(56, &export)]),
            ],
            exports: vec![
                BTreeMap::new(),
                BTreeMap::from([("Control".into(), BTreeSet::from([9]))]),
            ],
            bindings: vec![SymbolBinding {
                source_movie: 0,
                local_character_id: 7,
                url: "other.swf".into(),
                symbol: "Control".into(),
                resolution: Resolution::Resolved {
                    movie: 1,
                    character_id: 9,
                    origin: ImportOrigin::SavedReference,
                },
            }],
        }
    }
    fn scene(c: &Catalog) -> Scene<'_> {
        Scene::first_frame(MovieRef::new(0, &c.movies[0]), c, Default::default()).unwrap()
    }

    #[test]
    fn local_root_sprite_and_import_preserve_distinct_versions_and_tags() {
        let c = catalog(false);
        let s = scene(&c);
        let mut v = Core::new(&c, &s, profile(), Limits::default()).unwrap();
        assert_eq!(v.validate(0).unwrap().route, Route::LocalRoot);
        let f = v.validate(1).unwrap();
        assert_eq!(f.route, Route::LocalSprite);
        assert_eq!(f.source.header.version, 15);
        assert_eq!(f.placement.as_ref().unwrap().character_id, 1);
        let c = catalog(true);
        let s = scene(&c);
        let mut v = Core::new(&c, &s, profile(), Limits::default()).unwrap();
        let f = v.validate(1).unwrap();
        assert_eq!(f.source.header.version, 11);
        assert_eq!(f.source.movie_id, 1);
        let p = f.placement.as_ref().unwrap();
        assert_eq!((p.movie_id, p.character_id, p.depth), (0, 7, 1));
        assert_eq!(p.character_tag, p.contributing_tags[0]);
        assert!(
            matches!(&f.route, Route::SavedReferenceImport { import_tag, export_tag, url, symbol, .. }
            if import_tag.movie_id == 0 && export_tag.movie_id == 1 && url == "other.swf" && symbol == "Control")
        );
        let before = v.usage.tag_visits;
        v.validate(1).unwrap();
        assert_eq!(before, v.usage.tag_visits);
    }
    #[test]
    fn profile_source_identity_and_header_must_be_explicit_and_exact() {
        let c = catalog(false);
        let s = scene(&c);
        for missing in 0..4 {
            let mut p = profile();
            match missing {
                0 => p.final_ordinary_pair = false,
                1 => p.no_resource_substitution = false,
                2 => p.loader_compatibility_succeeded = false,
                _ => p.selected_resource_non_null_and_eligible = false,
            }
            assert!(matches!(
                Core::new(&c, &s, p, Limits::default()),
                Err(Error::Profile(_))
            ));
        }
        let cloned = c.movies[0].clone();
        let mut s = scene(&c);
        s.movies.insert(0, &cloned);
        assert!(matches!(
            Core::new(&c, &s, profile(), Limits::default()),
            Err(Error::Source { .. })
        ));
        let mut c = catalog(false);
        c.movies[0].header.version = 11;
        let s = scene(&c);
        assert!(matches!(
            Core::new(&c, &s, profile(), Limits::default()),
            Err(Error::Source { .. })
        ));
    }
    #[test]
    fn mismatched_placement_definition_parent_and_multihop_reject() {
        let c = catalog(true);
        for case in 0..8 {
            let mut s = scene(&c);
            match case {
                0 => s.nodes[1].raw.as_mut().unwrap().character_id = Some(8),
                1 => s.nodes[1].placement_movie = 1,
                2 => s.nodes[1].parent = Some(1),
                3 => s.nodes[0].children.push(1),
                4 => {
                    if let NodeKind::Import { chain, .. } = &mut s.nodes[1].kind {
                        chain.push(chain[0].clone());
                    }
                }
                5 => {
                    if let NodeKind::Import {
                        resolved: Some(d), ..
                    } = &mut s.nodes[1].kind
                    {
                        d.tag_index += 1;
                    }
                }
                6 => {
                    if let NodeKind::Import { chain, .. } = &mut s.nodes[1].kind {
                        chain[0].symbol = "Other".into();
                    }
                }
                _ => {
                    let mut duplicate = s.nodes[1].clone();
                    duplicate.id = s.nodes.len();
                    s.nodes[0].children.push(duplicate.id);
                    s.nodes.push(duplicate);
                }
            }
            assert!(
                Core::new(&c, &s, profile(), Limits::default())
                    .unwrap()
                    .validate(1)
                    .is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn alias_unresolved_wrong_tags_and_duplicate_export_reject() {
        for case in 0..5 {
            let mut c = catalog(true);
            match case {
                0 => {
                    if let Resolution::Resolved { origin, .. } = &mut c.bindings[0].resolution {
                        *origin = ImportOrigin::ExplicitAlias {
                            logical_object_path: "Example.Other".into(),
                            target_object_path: "Example.Other".into(),
                        };
                    }
                }
                1 => c.bindings[0].resolution = Resolution::MissingReferenceProperty,
                2 => c.movies[0].tags[0].code = 56,
                3 => {
                    let index = c.movies[1].dictionary[&9].tag_index;
                    c.movies[1].tags[index].code = 2;
                }
                _ => {
                    let tag = c.movies[1]
                        .tags
                        .iter()
                        .find(|t| matches!(t.data, TagData::ExportAssets(_)))
                        .unwrap()
                        .clone();
                    c.movies[1].tags.push(tag);
                }
            }
            let s = scene(&c);
            assert!(
                Core::new(&c, &s, profile(), Limits::default())
                    .unwrap()
                    .validate(1)
                    .is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn request_source_and_work_budgets_reject_without_facts() {
        let c = catalog(true);
        let s = scene(&c);
        for case in 0..9 {
            let mut l = Limits::default();
            match case {
                0 => l.nodes = 1,
                1 => l.movies = 1,
                2 => l.ancestor_depth = 1,
                3 => l.tag_visits = 0,
                4 => l.symbol_visits = 0,
                5 => l.frames = 0,
                6 => l.frame_objects = 0,
                7 => l.metadata_bytes = 0,
                _ => l.child_visits = 0,
            }
            let result =
                Core::new(&c, &s, profile(), l).and_then(|mut v| v.validate(1).map(|_| ()));
            assert!(
                matches!(result, Err(Error::Budget { .. })),
                "case {case}: {result:?}"
            );
        }
        let mut l = Limits::default();
        l.requests = 1;
        let mut v = Core::new(&c, &s, profile(), l).unwrap();
        v.validate(1).unwrap();
        assert!(matches!(
            v.validate(1),
            Err(Error::Budget {
                resource: "requests",
                ..
            })
        ));
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub nodes: usize,
    pub movies: usize,
    pub ancestor_depth: usize,
    pub tag_visits: usize,
    pub symbol_visits: usize,
    pub frames: usize,
    pub frame_objects: usize,
    pub metadata_bytes: usize,
    pub requests: usize,
    pub child_visits: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            nodes: 100_000,
            movies: 256,
            ancestor_depth: 256,
            tag_visits: 2_000_000,
            symbol_visits: 250_000,
            frames: 25_000,
            frame_objects: 100_000,
            metadata_bytes: 32 * 1024 * 1024,
            requests: 100_000,
            child_visits: 2_000_000,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub tag_visits: usize,
    pub symbol_visits: usize,
    pub frames: usize,
    pub frame_objects: usize,
    pub metadata_bytes: usize,
    pub requests: usize,
    pub child_visits: usize,
    pub validated_nodes: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Profile(&'static str),
    Budget {
        resource: &'static str,
        limit: usize,
    },
    Node {
        node: NodeId,
        message: &'static str,
    },
    Source {
        movie: MovieId,
        message: &'static str,
    },
    Unsupported {
        node: NodeId,
        message: &'static str,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(m) => write!(f, "unestablished conditional movie profile: {m}"),
            Self::Budget { resource, limit } => {
                write!(f, "movie binding {resource} budget exceeded ({limit})")
            }
            Self::Node { node, message } => write!(f, "movie binding node {node}: {message}"),
            Self::Source { movie, message } => write!(f, "movie binding source {movie}: {message}"),
            Self::Unsupported { node, message } => {
                write!(f, "unsupported movie binding node {node}: {message}")
            }
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn node_error(node: NodeId, message: &'static str) -> Error {
    Error::Node { node, message }
}
fn source_error(movie: MovieId, message: &'static str) -> Error {
    Error::Source { movie, message }
}
fn charge(n: &mut usize, amount: usize, limit: usize, resource: &'static str) -> Result<()> {
    *n = n
        .checked_add(amount)
        .filter(|v| *v <= limit)
        .ok_or(Error::Budget { resource, limit })?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovieSource {
    pub movie_id: MovieId,
    pub location: AssetLocation,
    pub object_path: String,
    pub header: Header,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacementSource {
    pub movie_id: MovieId,
    pub timeline_id: u16,
    pub frame_index: u16,
    pub depth: u16,
    pub character_id: u16,
    /// Last PlaceObject tag that explicitly assigned this character ID.
    pub character_tag: TagRef,
    /// All placement/update tags contributing to the currently saved instance.
    pub contributing_tags: Vec<TagRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    LocalRoot,
    LocalSprite,
    SavedReferenceImport {
        import_tag: TagRef,
        import_symbol_index: usize,
        url: String,
        symbol: String,
        export_tag: TagRef,
        export_symbol_index: usize,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingFacts {
    pub node_id: NodeId,
    pub parent: Option<NodeId>,
    pub placement: Option<PlacementSource>,
    /// Definition source; under the supplied profile it is also the selected movie.
    /// This is not a captured native MovieDef pointer or Environment identity.
    pub source: MovieSource,
    /// Root timeline has no dictionary character; no sprite ID is invented for it.
    pub definition: Option<DefinitionRef>,
    pub route: Route,
    pub conditional_profile: ConditionalProfile,
}

struct Source<'a> {
    location: AssetLocation,
    path: &'a str,
    movie: &'a Movie,
    exports: &'a BTreeMap<String, BTreeSet<u16>>,
}
trait Sources {
    fn source(&self, id: MovieId) -> Option<Source<'_>>;
    fn binding(&self, id: MovieId, character: u16) -> Option<&SymbolBinding>;
}
impl Sources for MovieLibrary {
    fn source(&self, id: MovieId) -> Option<Source<'_>> {
        self.entry(id).map(|e| Source {
            location: e.asset.location,
            path: &e.asset.object_path,
            movie: &e.asset.movie,
            exports: &e.exports,
        })
    }
    fn binding(&self, id: MovieId, character: u16) -> Option<&SymbolBinding> {
        MovieLibrary::binding(self, id, character)
    }
}

/// Reuses immutable source identities and saved first-frame replays.
/// Failed requests can charge work and cache valid ancestors; they return no facts
/// for the failed node. Neither the library nor the scene is modified.
pub struct Validator<'a, 'movies> {
    core: Core<'a, 'movies>,
}
impl<'a, 'movies> Validator<'a, 'movies> {
    pub fn new(
        library: &'a MovieLibrary,
        scene: &'a Scene<'movies>,
        profile: ConditionalProfile,
        limits: Limits,
    ) -> Result<Self> {
        Ok(Self {
            core: Core::new(library, scene, profile, limits)?,
        })
    }
    pub fn validate(&mut self, node: NodeId) -> Result<&BindingFacts> {
        self.core.validate(node)
    }
    pub fn usage(&self) -> &Usage {
        &self.core.usage
    }
}

#[derive(Clone)]
struct Trace {
    character_tag: Option<TagRef>,
    tags: Vec<TagRef>,
}
struct Frame {
    value: StaticFrame,
    traces: BTreeMap<u16, Trace>,
}
struct Core<'a, 'movies> {
    sources: &'a dyn Sources,
    scene: &'a Scene<'movies>,
    profile: ConditionalProfile,
    limits: Limits,
    usage: Usage,
    movies: BTreeMap<MovieId, MovieSource>,
    frames: BTreeMap<(MovieId, u16), Frame>,
    facts: BTreeMap<NodeId, BindingFacts>,
    exports: BTreeMap<(MovieId, String), (TagRef, usize)>,
}
impl<'a, 'movies> Core<'a, 'movies> {
    fn new(
        sources: &'a dyn Sources,
        scene: &'a Scene<'movies>,
        profile: ConditionalProfile,
        limits: Limits,
    ) -> Result<Self> {
        for (ok, condition) in [
            (
                profile.final_ordinary_pair,
                "final ordinary local/direct-export pair",
            ),
            (
                profile.no_resource_substitution,
                "no relevant resource substitution",
            ),
            (
                profile.loader_compatibility_succeeded,
                "successful compatible loading",
            ),
            (
                profile.selected_resource_non_null_and_eligible,
                "nonnull eligible selected resource",
            ),
        ] {
            if !ok {
                return Err(Error::Profile(condition));
            }
        }
        if scene.nodes.len() > limits.nodes {
            return Err(Error::Budget {
                resource: "nodes",
                limit: limits.nodes,
            });
        }
        if scene.movies.len() > limits.movies {
            return Err(Error::Budget {
                resource: "movies",
                limit: limits.movies,
            });
        }
        if scene.root >= scene.nodes.len() {
            return Err(node_error(scene.root, "missing root"));
        }
        let mut core = Self {
            sources,
            scene,
            profile,
            limits,
            usage: Usage::default(),
            movies: BTreeMap::new(),
            frames: BTreeMap::new(),
            facts: BTreeMap::new(),
            exports: BTreeMap::new(),
        };
        for (&id, movie) in &scene.movies {
            let source = sources
                .source(id)
                .ok_or(source_error(id, "missing library source"))?;
            if !std::ptr::eq(*movie, source.movie) {
                return Err(source_error(
                    id,
                    "scene movie is not the immutable library movie",
                ));
            }
            let bytes = source.movie.bytes();
            if bytes.len() < 8
                || bytes[3] != movie.header.version
                || !(1..=19).contains(&bytes[3])
                || u32::from_le_bytes(bytes[4..8].try_into().unwrap())
                    != movie.header.declared_bytes
                || movie.header.declared_bytes as usize != bytes.len()
                || !matches!(
                    (&bytes[..3], movie.header.format),
                    (b"FWS", crate::gfx::Format::Swf) | (b"GFX", crate::gfx::Format::Gfx)
                )
            {
                return Err(source_error(
                    id,
                    "header disagrees with retained movie bytes",
                ));
            }
            charge(
                &mut core.usage.metadata_bytes,
                source.path.len(),
                limits.metadata_bytes,
                "metadata bytes",
            )?;
            core.movies.insert(
                id,
                MovieSource {
                    movie_id: id,
                    location: source.location,
                    object_path: source.path.into(),
                    header: movie.header,
                },
            );
        }
        for (id, node) in scene.nodes.iter().enumerate() {
            if node.id != id {
                return Err(node_error(id, "arena index disagrees with node ID"));
            }
            if matches!(node.kind, NodeKind::Root) != (id == scene.root) {
                return Err(node_error(id, "root kind disagrees with scene root"));
            }
        }
        Ok(core)
    }
    fn validate(&mut self, requested: NodeId) -> Result<&BindingFacts> {
        charge(
            &mut self.usage.requests,
            1,
            self.limits.requests,
            "requests",
        )?;
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut at = requested;
        loop {
            if self.facts.contains_key(&at) {
                break;
            }
            if chain.len() >= self.limits.ancestor_depth {
                return Err(Error::Budget {
                    resource: "ancestor depth",
                    limit: self.limits.ancestor_depth,
                });
            }
            if !seen.insert(at) {
                return Err(node_error(at, "parent cycle"));
            }
            let node = self
                .scene
                .nodes
                .get(at)
                .ok_or(node_error(at, "missing node"))?;
            chain.push(at);
            if let Some(parent) = node.parent {
                at = parent;
            } else {
                if at != self.scene.root {
                    return Err(node_error(at, "detached non-root node"));
                }
                break;
            }
        }
        for id in chain.into_iter().rev() {
            let facts = self.one(id)?;
            self.facts.insert(id, facts);
            self.usage.validated_nodes += 1;
        }
        self.facts
            .get(&requested)
            .ok_or(node_error(requested, "missing validation result"))
    }
    fn movie(&self, id: MovieId) -> Result<&Movie> {
        self.scene
            .movies
            .get(&id)
            .copied()
            .ok_or(source_error(id, "movie absent from scene"))
    }
    fn one(&mut self, id: NodeId) -> Result<BindingFacts> {
        let scene = self.scene;
        let node = &scene.nodes[id];
        if !node.unsupported.is_empty() {
            return Err(Error::Unsupported {
                node: id,
                message: "node retains unsupported structural state",
            });
        }
        if id == self.scene.root {
            if node.parent.is_some()
                || node.raw.is_some()
                || node.segment.is_some()
                || !matches!(node.kind, NodeKind::Root)
            {
                return Err(node_error(id, "root has a placement or parent"));
            }
            self.frame(node.placement_movie, 0)?;
            let source = self
                .movies
                .get(&node.placement_movie)
                .ok_or(source_error(node.placement_movie, "root movie missing"))?
                .clone();
            self.metadata(source.object_path.len())?;
            return Ok(BindingFacts {
                node_id: id,
                parent: None,
                placement: None,
                source,
                definition: None,
                route: Route::LocalRoot,
                conditional_profile: self.profile,
            });
        }
        let parent_id = node.parent.ok_or(node_error(id, "missing parent"))?;
        let parent = self
            .facts
            .get(&parent_id)
            .ok_or(node_error(id, "unvalidated parent"))?;
        let parent_movie = parent.source.movie_id;
        let timeline = parent.definition.map_or(0, |d| d.character_id);
        if node.placement_movie != parent_movie {
            return Err(node_error(
                id,
                "placement movie disagrees with parent source",
            ));
        }
        let siblings = &self.scene.nodes[parent_id].children;
        charge(
            &mut self.usage.child_visits,
            siblings.len(),
            self.limits.child_visits,
            "child visits",
        )?;
        if siblings.iter().filter(|&&v| v == id).count() != 1 {
            return Err(node_error(id, "parent does not contain child exactly once"));
        }
        let mut depths = BTreeSet::new();
        for &child in siblings {
            let sibling = self
                .scene
                .nodes
                .get(child)
                .ok_or(node_error(id, "parent has an invalid child"))?;
            let part = sibling
                .segment
                .as_ref()
                .ok_or(node_error(id, "child has no placement depth"))?;
            if sibling.parent != Some(parent_id) || !depths.insert(part.depth) {
                return Err(node_error(
                    id,
                    "sibling parent or unique placement depth mismatch",
                ));
            }
        }
        let segment = node
            .segment
            .as_ref()
            .ok_or(node_error(id, "missing placement depth"))?;
        let raw = node
            .raw
            .as_ref()
            .ok_or(node_error(id, "missing raw placement"))?;
        if segment.name != raw.name {
            return Err(node_error(id, "path name disagrees with raw placement"));
        }
        if raw.class_name.is_some() {
            return Err(Error::Unsupported {
                node: id,
                message: "class-name placement",
            });
        }
        let character = raw
            .character_id
            .filter(|v| *v != 0)
            .ok_or(node_error(id, "no nonzero raw character ID"))?;
        self.frame(parent_movie, timeline)?;
        let frame = &self.frames[&(parent_movie, timeline)];
        if frame.value.objects.get(&segment.depth) != Some(raw) {
            return Err(node_error(
                id,
                "raw placement differs from saved first frame",
            ));
        }
        let trace = frame
            .traces
            .get(&segment.depth)
            .ok_or(node_error(id, "placement has no saved tag trace"))?;
        let character_tag = trace
            .character_tag
            .ok_or(node_error(id, "character has no assigning tag"))?;
        let contributing_tags = trace.tags.clone();
        self.metadata(contributing_tags.len() * std::mem::size_of::<TagRef>())?;
        let (definition, route) = match &node.kind {
            NodeKind::Definition(d) => {
                if d.movie_id != parent_movie || d.character_id != character {
                    return Err(node_error(id, "local definition disagrees with placement"));
                }
                self.sprite(id, *d)?;
                (*d, Route::LocalSprite)
            }
            NodeKind::Import {
                chain,
                resolved: Some(d),
                unresolved: None,
            } => {
                if chain.len() != 1 {
                    return Err(Error::Unsupported {
                        node: id,
                        message: "import route is not one hop",
                    });
                }
                let link = &chain[0];
                if link.source.movie_id != parent_movie
                    || link.local_character_id != character
                    || link.target != Some((d.movie_id, d.character_id))
                {
                    return Err(node_error(
                        id,
                        "import chain disagrees with placement or target",
                    ));
                }
                if d.movie_id == parent_movie {
                    return Err(Error::Unsupported {
                        node: id,
                        message: "same-movie import/cycle outside profile",
                    });
                }
                let movie = self
                    .scene
                    .movies
                    .get(&parent_movie)
                    .copied()
                    .ok_or(source_error(parent_movie, "placement movie missing"))?;
                let dict = movie
                    .dictionary
                    .get(&character)
                    .ok_or(node_error(id, "missing imported dictionary entry"))?;
                if dict.id != character
                    || dict.kind != CharacterKind::Imported
                    || dict.tag_index != link.source.tag_index
                    || dict.bounds.is_some()
                {
                    return Err(node_error(id, "import dictionary identity mismatch"));
                }
                let tag = movie
                    .tags
                    .get(dict.tag_index)
                    .ok_or(node_error(id, "import tag out of bounds"))?;
                if !matches!(tag.code, 57 | 71) {
                    return Err(node_error(id, "wrong import tag code"));
                }
                let TagData::ImportAssets { url, symbols } = &tag.data else {
                    return Err(node_error(id, "import tag has wrong kind"));
                };
                charge(
                    &mut self.usage.symbol_visits,
                    symbols.len(),
                    self.limits.symbol_visits,
                    "symbol visits",
                )?;
                let matches: Vec<_> = symbols
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.id == character)
                    .collect();
                let [(import_symbol_index, symbol)] = matches.as_slice() else {
                    return Err(node_error(id, "import character is not a unique symbol"));
                };
                if url != &link.url || symbol.name != link.symbol {
                    return Err(node_error(id, "import URL/symbol mismatch"));
                }
                let binding = self
                    .sources
                    .binding(parent_movie, character)
                    .ok_or(node_error(id, "missing library import binding"))?;
                if binding.source_movie != parent_movie
                    || binding.local_character_id != character
                    || binding.url != *url
                    || binding.symbol != symbol.name
                {
                    return Err(node_error(
                        id,
                        "library binding disagrees with import source",
                    ));
                }
                match &binding.resolution {
                    Resolution::Resolved {
                        movie,
                        character_id,
                        origin: ImportOrigin::SavedReference,
                    } if *movie == d.movie_id && *character_id == d.character_id => {}
                    Resolution::Resolved {
                        origin: ImportOrigin::ExplicitAlias { .. },
                        ..
                    } => {
                        return Err(Error::Unsupported {
                            node: id,
                            message: "explicit alias is not a saved-reference route",
                        });
                    }
                    _ => return Err(node_error(id, "library target unresolved or mismatched")),
                }
                let import_symbol_index = *import_symbol_index;
                let url = url.clone();
                let symbol = symbol.name.clone();
                self.sprite(id, *d)?;
                let (export_tag, export_symbol_index) =
                    self.export(id, d.movie_id, &symbol, d.character_id)?;
                self.metadata(url.len() + symbol.len())?;
                (
                    *d,
                    Route::SavedReferenceImport {
                        import_tag: link.source,
                        import_symbol_index,
                        url,
                        symbol,
                        export_tag,
                        export_symbol_index,
                    },
                )
            }
            NodeKind::Import { .. } => {
                return Err(Error::Unsupported {
                    node: id,
                    message: "unresolved import or cycle",
                });
            }
            _ => {
                return Err(Error::Unsupported {
                    node: id,
                    message: "node is not a local/imported sprite",
                });
            }
        };
        let source = self
            .movies
            .get(&definition.movie_id)
            .ok_or(source_error(
                definition.movie_id,
                "definition movie missing",
            ))?
            .clone();
        self.metadata(source.object_path.len())?;
        Ok(BindingFacts {
            node_id: id,
            parent: Some(parent_id),
            placement: Some(PlacementSource {
                movie_id: parent_movie,
                timeline_id: timeline,
                frame_index: 0,
                depth: segment.depth,
                character_id: character,
                character_tag,
                contributing_tags,
            }),
            source,
            definition: Some(definition),
            route,
            conditional_profile: self.profile,
        })
    }
    fn metadata(&mut self, bytes: usize) -> Result<()> {
        charge(
            &mut self.usage.metadata_bytes,
            bytes,
            self.limits.metadata_bytes,
            "metadata bytes",
        )
    }
    fn sprite(&self, node: NodeId, d: DefinitionRef) -> Result<()> {
        if d.kind != CharacterKind::Sprite {
            return Err(Error::Unsupported {
                node,
                message: "selected definition is not a sprite",
            });
        }
        let movie = self.movie(d.movie_id)?;
        let c = movie
            .dictionary
            .get(&d.character_id)
            .ok_or(node_error(node, "target definition missing"))?;
        if c.id != d.character_id
            || c.kind != d.kind
            || c.tag_index != d.tag_index
            || c.bounds != d.bounds
        {
            return Err(node_error(
                node,
                "resolved definition disagrees with dictionary",
            ));
        }
        let tag = movie
            .tags
            .get(d.tag_index)
            .ok_or(node_error(node, "sprite tag out of bounds"))?;
        if tag.code != 39 {
            return Err(node_error(node, "wrong sprite tag code"));
        }
        let TagData::Sprite { id, frame_count } = tag.data else {
            return Err(node_error(node, "sprite tag kind mismatch"));
        };
        if id != d.character_id || id == 0 {
            return Err(node_error(node, "sprite tag character mismatch"));
        }
        let timeline = movie
            .timelines
            .get(&id)
            .ok_or(node_error(node, "sprite timeline missing"))?;
        if timeline.id != id || timeline.frame_count != frame_count {
            return Err(node_error(node, "sprite timeline identity mismatch"));
        }
        if frame_count == 0 {
            return Err(Error::Unsupported {
                node,
                message: "empty sprite timeline",
            });
        }
        Ok(())
    }
    fn export(
        &mut self,
        node: NodeId,
        movie_id: MovieId,
        symbol: &str,
        character: u16,
    ) -> Result<(TagRef, usize)> {
        let source = self
            .sources
            .source(movie_id)
            .ok_or(source_error(movie_id, "export source missing"))?;
        let ids = source
            .exports
            .get(symbol)
            .ok_or(node_error(node, "export symbol missing from library"))?;
        if ids.len() != 1 || !ids.contains(&character) {
            return Err(node_error(node, "export target ambiguous or mismatched"));
        }
        let key = (movie_id, symbol.to_owned());
        if let Some(result) = self.exports.get(&key) {
            return Ok(*result);
        }
        let mut found = None;
        for (tag_index, tag) in source.movie.tags.iter().enumerate() {
            charge(
                &mut self.usage.tag_visits,
                1,
                self.limits.tag_visits,
                "tag visits",
            )?;
            if let TagData::ExportAssets(symbols) = &tag.data {
                if tag.code != 56 {
                    return Err(node_error(node, "wrong export tag code"));
                }
                charge(
                    &mut self.usage.symbol_visits,
                    symbols.len(),
                    self.limits.symbol_visits,
                    "symbol visits",
                )?;
                for (index, entry) in symbols.iter().enumerate().filter(|(_, e)| e.name == symbol) {
                    if entry.id != character || found.is_some() {
                        return Err(node_error(node, "export is not one direct declaration"));
                    }
                    found = Some((
                        TagRef {
                            movie_id,
                            tag_index,
                        },
                        index,
                    ));
                }
            }
        }
        let found = found.ok_or(node_error(node, "no saved ExportAssets declaration"))?;
        self.metadata(symbol.len())?;
        self.exports.insert(key, found);
        Ok(found)
    }
    fn frame(&mut self, movie_id: MovieId, timeline_id: u16) -> Result<()> {
        if self.frames.contains_key(&(movie_id, timeline_id)) {
            return Ok(());
        }
        charge(&mut self.usage.frames, 1, self.limits.frames, "frames")?;
        let movie = self
            .scene
            .movies
            .get(&movie_id)
            .copied()
            .ok_or(source_error(movie_id, "placement movie absent"))?;
        let timeline = movie
            .timelines
            .get(&timeline_id)
            .ok_or(source_error(movie_id, "placement timeline missing"))?;
        if timeline.id != timeline_id || timeline.frame_count == 0 {
            return Err(source_error(movie_id, "invalid placement timeline"));
        }
        let mut traces: BTreeMap<u16, Trace> = BTreeMap::new();
        let mut shown = false;
        for &tag_index in &timeline.tags {
            charge(
                &mut self.usage.tag_visits,
                2,
                self.limits.tag_visits,
                "tag visits",
            )?;
            let tag = movie
                .tags
                .get(tag_index)
                .ok_or(source_error(movie_id, "timeline tag outside movie"))?;
            if tag.timeline_id != timeline_id || tag.frame_index != 0 {
                return Err(source_error(movie_id, "first-frame tag identity mismatch"));
            }
            let reference = TagRef {
                movie_id,
                tag_index,
            };
            match &tag.data {
                TagData::Place(p) => {
                    if !matches!(tag.code, 4 | 26 | 70) {
                        return Err(source_error(movie_id, "wrong placement tag code"));
                    }
                    if !p.move_existing {
                        traces.insert(
                            p.depth,
                            Trace {
                                character_tag: None,
                                tags: Vec::new(),
                            },
                        );
                    }
                    let trace = traces.get_mut(&p.depth).ok_or(source_error(
                        movie_id,
                        "update has no prior saved placement",
                    ))?;
                    if p.character_id.is_some() {
                        trace.character_tag = Some(reference);
                    }
                    charge(
                        &mut self.usage.metadata_bytes,
                        std::mem::size_of::<TagRef>(),
                        self.limits.metadata_bytes,
                        "metadata bytes",
                    )?;
                    trace.tags.push(reference);
                }
                TagData::Remove { depth, .. } => {
                    if !matches!(tag.code, 5 | 28) {
                        return Err(source_error(movie_id, "wrong remove tag code"));
                    }
                    traces.remove(depth);
                }
                TagData::ShowFrame => {
                    if tag.code != 1 {
                        return Err(source_error(movie_id, "wrong ShowFrame code"));
                    }
                    shown = true;
                    break;
                }
                _ => {}
            }
        }
        if !shown {
            return Err(source_error(movie_id, "saved frame has no ShowFrame"));
        }
        let value = movie
            .static_frame(timeline_id, 0)
            .map_err(|_| source_error(movie_id, "saved first-frame replay failed"))?;
        charge(
            &mut self.usage.frame_objects,
            value.objects.len(),
            self.limits.frame_objects,
            "frame objects",
        )?;
        self.frames
            .insert((movie_id, timeline_id), Frame { value, traces });
        Ok(())
    }
}
