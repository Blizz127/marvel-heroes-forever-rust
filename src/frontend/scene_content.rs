//! Owned CPU preparation of actual scene definitions and declared image bindings.
//! No GPU calls, invented geometry, script execution, or implicit UV/color policy.

use super::library::{MovieLibrary, Resolution, SymbolBinding};
use super::{AssetLocation, AssetSet, ImageBinding, ImageResolution};
use crate::gfx::scene::{
    ActionNode, DefinitionRef, MovieId, NodeId, OpaqueNode, Scene, SceneNode, TagRef,
};
use crate::gfx::shapes::{self, Fill, Shape};
use crate::gfx::tessellate::{self, Mesh};
use crate::gfx::{CharacterKind, TagData};
use crate::texture::{BulkPayload, Descriptor, RgbaImage};
use std::collections::{BTreeMap, BTreeSet};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_meshes: usize,
    pub max_images: usize,
    pub max_textures: usize,
    pub max_triangles: usize,
    pub max_decoded_rgba_bytes: usize,
    pub max_shape_bytes: usize,
    pub max_shape_records: usize,
    pub max_import_hops: usize,
    pub tessellation: tessellate::Options,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_meshes: 4096,
            max_images: 16_384,
            max_textures: 4096,
            max_triangles: 2_000_000,
            max_decoded_rgba_bytes: 256 * 1024 * 1024,
            max_shape_bytes: 64 * 1024 * 1024,
            max_shape_records: 2_000_000,
            max_import_hops: 32,
            tessellation: tessellate::Options::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparationStage {
    ShapeDecode,
    Tessellation,
    BitmapBinding,
    TextureRead,
    TextureDecode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationIssue {
    pub stage: PreparationStage,
    pub error_kind: Option<io::ErrorKind>,
    pub description: String,
}
impl PreparationIssue {
    fn message(stage: PreparationStage, description: impl Into<String>) -> Self {
        Self {
            stage,
            error_kind: None,
            description: description.into(),
        }
    }
    fn error(stage: PreparationStage, error: io::Error) -> Self {
        Self {
            stage,
            error_kind: Some(error.kind()),
            description: error.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Limitation {
    SceneUnsupported(crate::gfx::scene::Unsupported),
    BoundActionsNotExecuted(usize),
    FiltersNotApplied(usize),
    BlendNotApplied(u8),
    ScalingGridsNotApplied(usize),
    BitmapCacheNotApplied(u8),
    OpaqueBackgroundNotApplied,
    MorphRatioNotApplied(u16),
    TextNotPrepared,
    DirectBitmapGeometryNotPrepared,
    UnresolvedOrNonDisplayContent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeContent {
    Group,
    Shape { mesh: usize },
    Bitmap { image: usize },
    Text { definition: DefinitionRef },
    Unrendered,
}

#[derive(Clone, Debug)]
pub struct PreparedNode {
    /// Complete structural instance, including masks, raw placement and source IDs.
    pub instance: SceneNode,
    pub content: NodeContent,
    pub limitations: Vec<Limitation>,
}

#[derive(Clone, Debug)]
pub struct MeshAsset {
    pub definition: DefinitionRef,
    /// Kept even when tessellation fails, including strokes/unsupported source fields.
    pub shape: Option<Shape>,
    pub mesh: Option<Mesh>,
    /// Aligned with mesh.fills; Some references the local movie bitmap-ID cache.
    pub fill_images: Vec<Option<usize>>,
    pub issues: Vec<PreparationIssue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UvPaddingPolicy {
    Unverified,
}

#[derive(Clone, Debug)]
pub struct BitmapAsset {
    pub movie_id: MovieId,
    pub character_id: u16,
    pub resolved_definition: Option<DefinitionRef>,
    pub import_chain: Vec<SymbolBinding>,
    pub binding: Option<ImageBinding>,
    pub logical_dimensions: Option<[u16; 2]>,
    pub texture: Option<usize>,
    /// The adapter must choose and report a sampling/padding policy explicitly.
    pub uv_padding: UvPaddingPolicy,
    pub issues: Vec<PreparationIssue>,
}

#[derive(Clone, Debug)]
pub struct TextureAsset {
    pub location: AssetLocation,
    /// SRGB remains Option<bool>; no absent-property default is inferred.
    pub descriptor: Option<Descriptor>,
    pub mip_index: usize,
    pub physical_dimensions: Option<[u32; 2]>,
    /// RGBA8 bytes without transfer conversion or alpha premultiplication.
    pub image: Option<RgbaImage>,
    pub issues: Vec<PreparationIssue>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub triangles: usize,
    pub decoded_rgba_bytes: usize,
    pub shape_bytes: usize,
    pub shape_records: usize,
    pub stroke_edges_not_expanded: usize,
}

#[derive(Clone, Debug)]
pub struct PreparedScene {
    pub root: NodeId,
    pub nodes: Vec<PreparedNode>,
    pub meshes: Vec<MeshAsset>,
    pub images: Vec<BitmapAsset>,
    pub textures: Vec<TextureAsset>,
    pub actions: Vec<ActionNode>,
    pub opaque_nodes: Vec<OpaqueNode>,
    pub unbound_action_tags: Vec<TagRef>,
    pub stats: Statistics,
    pub tessellation: tessellate::Options,
}

/// Prepare only real definitions referenced by the supplied structural snapshot.
/// Failures specific to an asset remain explicit cache entries; budget/identity
/// failures return Err and never return a successful partial preparation.
pub fn prepare(
    assets: &AssetSet,
    library: &MovieLibrary,
    scene: &Scene<'_>,
    limits: Limits,
) -> io::Result<PreparedScene> {
    if scene.nodes.len() > limits.max_nodes || limits.max_import_hops > 256 {
        return Err(invalid("scene-content node/import budget exceeded"));
    }
    if !limits.tessellation.curve_tolerance.is_finite()
        || limits.tessellation.curve_tolerance <= 0.0
        || limits.tessellation.max_curve_depth > 32
    {
        return Err(invalid("invalid scene-content tessellation options"));
    }
    if scene.root >= scene.nodes.len() {
        return Err(invalid("scene-content root is outside the node arena"));
    }
    for (&id, movie) in &scene.movies {
        let entry = library
            .entry(id)
            .ok_or_else(|| invalid("scene movie is absent from library"))?;
        if !std::ptr::eq(*movie, &entry.asset.movie) {
            return Err(invalid("scene MovieId and library movie identity differ"));
        }
    }
    let mut builder = Builder {
        assets,
        library,
        limits,
        result: PreparedScene {
            root: scene.root,
            nodes: Vec::new(),
            meshes: Vec::new(),
            images: Vec::new(),
            textures: Vec::new(),
            actions: scene.actions.clone(),
            opaque_nodes: scene.opaque_nodes.clone(),
            unbound_action_tags: scene.unbound_action_tags.clone(),
            stats: Statistics::default(),
            tessellation: limits.tessellation,
        },
        meshes: BTreeMap::new(),
        images: BTreeMap::new(),
        textures: BTreeMap::new(),
        catalogs: BTreeMap::new(),
    };
    for (index, node) in scene.nodes.iter().enumerate() {
        if node.id != index {
            return Err(invalid("scene node index/identity mismatch"));
        }
        let mut limitations = node_limitations(node);
        let content = if let Some(definition) = node.kind.definition() {
            builder.check_definition(definition)?;
            match definition.kind {
                CharacterKind::Shape => NodeContent::Shape {
                    mesh: builder.mesh(definition)?,
                },
                CharacterKind::Bitmap => {
                    limitations.push(Limitation::DirectBitmapGeometryNotPrepared);
                    // Retain local import identity, when this placement itself is imported.
                    let local = node
                        .raw
                        .as_ref()
                        .and_then(|raw| raw.character_id)
                        .map(|id| (node.placement_movie, id))
                        .unwrap_or((definition.movie_id, definition.character_id));
                    NodeContent::Bitmap {
                        image: builder.bitmap(local.0, local.1)?,
                    }
                }
                CharacterKind::Text => {
                    limitations.push(Limitation::TextNotPrepared);
                    NodeContent::Text { definition }
                }
                CharacterKind::Sprite => NodeContent::Group,
                _ => {
                    limitations.push(Limitation::UnresolvedOrNonDisplayContent);
                    NodeContent::Unrendered
                }
            }
        } else if matches!(node.kind, crate::gfx::scene::NodeKind::Root) {
            NodeContent::Group
        } else {
            limitations.push(Limitation::UnresolvedOrNonDisplayContent);
            NodeContent::Unrendered
        };
        builder.result.nodes.push(PreparedNode {
            instance: node.clone(),
            content,
            limitations,
        });
    }
    Ok(builder.result)
}

fn node_limitations(node: &SceneNode) -> Vec<Limitation> {
    let mut result: Vec<_> = node
        .unsupported
        .iter()
        .cloned()
        .map(Limitation::SceneUnsupported)
        .collect();
    if !node.actions.is_empty() {
        result.push(Limitation::BoundActionsNotExecuted(node.actions.len()));
    }
    if !node.scaling_grids.is_empty() {
        result.push(Limitation::ScalingGridsNotApplied(node.scaling_grids.len()));
    }
    if let Some(raw) = &node.raw {
        if !raw.filters.is_empty() {
            result.push(Limitation::FiltersNotApplied(raw.filters.len()));
        }
        if !matches!(raw.blend_mode, 0 | 1) {
            result.push(Limitation::BlendNotApplied(raw.blend_mode));
        }
        if raw.bitmap_cache != 0 {
            result.push(Limitation::BitmapCacheNotApplied(raw.bitmap_cache));
        }
        if raw.opaque_background.is_some() {
            result.push(Limitation::OpaqueBackgroundNotApplied);
        }
        if raw.ratio != 0 {
            result.push(Limitation::MorphRatioNotApplied(raw.ratio));
        }
    }
    result
}

struct Builder<'a> {
    assets: &'a AssetSet,
    library: &'a MovieLibrary,
    limits: Limits,
    result: PreparedScene,
    meshes: BTreeMap<(MovieId, u16), usize>,
    images: BTreeMap<(MovieId, u16), usize>,
    textures: BTreeMap<AssetLocation, usize>,
    catalogs: BTreeMap<MovieId, Result<BTreeMap<u16, ImageBinding>, String>>,
}
impl Builder<'_> {
    fn check_definition(&self, definition: DefinitionRef) -> io::Result<()> {
        let entry = self
            .library
            .entry(definition.movie_id)
            .ok_or_else(|| invalid("definition movie not in library"))?;
        let character = entry
            .asset
            .movie
            .dictionary
            .get(&definition.character_id)
            .ok_or_else(|| invalid("definition ID not in library dictionary"))?;
        if character.tag_index != definition.tag_index
            || character.kind != definition.kind
            || character.bounds != definition.bounds
        {
            return Err(invalid(
                "definition reference disagrees with library dictionary",
            ));
        }
        Ok(())
    }
    fn mesh(&mut self, definition: DefinitionRef) -> io::Result<usize> {
        let key = (definition.movie_id, definition.character_id);
        if let Some(&index) = self.meshes.get(&key) {
            return Ok(index);
        }
        if self.result.meshes.len() >= self.limits.max_meshes {
            return Err(invalid("scene-content mesh budget exceeded"));
        }
        let index = self.result.meshes.len();
        self.meshes.insert(key, index);
        let mut asset = MeshAsset {
            definition,
            shape: None,
            mesh: None,
            fill_images: Vec::new(),
            issues: Vec::new(),
        };
        let movie = &self.library.entry(definition.movie_id).unwrap().asset.movie;
        let tag = movie
            .tags
            .get(definition.tag_index)
            .ok_or_else(|| invalid("shape tag index out of bounds"))?;
        add_budget(
            &mut self.result.stats.shape_bytes,
            tag.body.len,
            self.limits.max_shape_bytes,
            "shape source byte",
        )?;
        let shape = match shapes::parse_movie_tag(movie, definition.tag_index) {
            Ok(shape) => shape,
            Err(error) => {
                asset.issues.push(PreparationIssue::error(
                    PreparationStage::ShapeDecode,
                    error,
                ));
                self.result.meshes.push(asset);
                return Ok(index);
            }
        };
        if shape.id != definition.character_id {
            return Err(invalid("decoded shape ID differs from definition"));
        }
        let records = shape
            .records
            .len()
            .checked_add(shape.edges.len())
            .ok_or_else(|| invalid("shape record count overflow"))?;
        add_budget(
            &mut self.result.stats.shape_records,
            records,
            self.limits.max_shape_records,
            "shape record",
        )?;
        let mut options = self.limits.tessellation;
        options.max_triangles = options
            .max_triangles
            .min(self.limits.max_triangles - self.result.stats.triangles);
        match tessellate::tessellate(&shape, options) {
            Ok(mesh) => {
                let mut triangles = 0usize;
                for fill in &mesh.fills {
                    triangles = triangles
                        .checked_add(fill.indices.len() / 3)
                        .ok_or_else(|| invalid("triangle count overflow"))?;
                    asset.fill_images.push(match fill.paint.fill {
                        Fill::Bitmap { character_id, .. } => {
                            Some(self.bitmap(definition.movie_id, character_id)?)
                        }
                        _ => None,
                    });
                }
                add_budget(
                    &mut self.result.stats.triangles,
                    triangles,
                    self.limits.max_triangles,
                    "triangle",
                )?;
                self.result.stats.stroke_edges_not_expanded = self
                    .result
                    .stats
                    .stroke_edges_not_expanded
                    .checked_add(mesh.stroke_edges)
                    .ok_or_else(|| invalid("stroke edge count overflow"))?;
                if mesh.stroke_edges != 0 {
                    asset.issues.push(PreparationIssue::message(
                        PreparationStage::Tessellation,
                        format!(
                            "{} stroke edges retained but not expanded",
                            mesh.stroke_edges
                        ),
                    ));
                }
                if mesh.inferred_winding {
                    asset.issues.push(PreparationIssue::message(
                        PreparationStage::Tessellation,
                        "Shape4 winding interpretation remains inferred",
                    ));
                }
                asset.mesh = Some(mesh);
            }
            Err(error) => {
                // The original tessellator reports its bounded-work failures this way.
                // They are fatal preparation limits, not quietly omitted geometry.
                if error.to_string().contains("budget") {
                    return Err(error);
                }
                asset.issues.push(PreparationIssue::error(
                    PreparationStage::Tessellation,
                    error,
                ));
            }
        }
        asset.shape = Some(shape);
        self.result.meshes.push(asset);
        Ok(index)
    }

    fn bitmap(&mut self, movie_id: MovieId, character_id: u16) -> io::Result<usize> {
        let key = (movie_id, character_id);
        if let Some(&index) = self.images.get(&key) {
            return Ok(index);
        }
        if self.result.images.len() >= self.limits.max_images {
            return Err(invalid("scene-content bitmap binding budget exceeded"));
        }
        let index = self.result.images.len();
        self.images.insert(key, index);
        let mut image = BitmapAsset {
            movie_id,
            character_id,
            resolved_definition: None,
            import_chain: Vec::new(),
            binding: None,
            logical_dimensions: None,
            texture: None,
            uv_padding: UvPaddingPolicy::Unverified,
            issues: Vec::new(),
        };
        let mut current = key;
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    "bitmap import cycle",
                ));
                break;
            }
            let Some(entry) = self.library.entry(current.0) else {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    "bitmap target movie missing",
                ));
                break;
            };
            let Some(character) = entry.asset.movie.dictionary.get(&current.1) else {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    "bitmap character missing from dictionary",
                ));
                break;
            };
            if character.kind == CharacterKind::Imported {
                if image.import_chain.len() >= self.limits.max_import_hops {
                    return Err(invalid("scene-content bitmap import budget exceeded"));
                }
                let Some(binding) = self.library.binding(current.0, current.1) else {
                    image.issues.push(PreparationIssue::message(
                        PreparationStage::BitmapBinding,
                        "imported bitmap has no declared library binding",
                    ));
                    break;
                };
                image.import_chain.push(binding.clone());
                let Resolution::Resolved {
                    movie,
                    character_id,
                    ..
                } = binding.resolution
                else {
                    image.issues.push(PreparationIssue::message(
                        PreparationStage::BitmapBinding,
                        format!("bitmap import unresolved: {:?}", binding.resolution),
                    ));
                    break;
                };
                current = (movie, character_id);
                continue;
            }
            if character.kind != CharacterKind::Bitmap {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    format!("bitmap fill references {:?} definition", character.kind),
                ));
                break;
            }
            let definition = DefinitionRef {
                movie_id: current.0,
                character_id: current.1,
                tag_index: character.tag_index,
                kind: character.kind,
                bounds: character.bounds,
            };
            image.resolved_definition = Some(definition);
            let tag = entry
                .asset
                .movie
                .tags
                .get(character.tag_index)
                .ok_or_else(|| invalid("bitmap definition tag index out of bounds"))?;
            let TagData::ExternalImage(external) = &tag.data else {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    "embedded SWF bitmap decoder is not part of this bridge",
                ));
                break;
            };
            image.logical_dimensions = Some([external.width, external.height]);
            if !self.catalogs.contains_key(&current.0) {
                let catalog = self
                    .assets
                    .bind_images(&entry.asset)
                    .and_then(|bindings| {
                        let mut catalog = BTreeMap::new();
                        for binding in bindings {
                            if catalog.insert(binding.image.id, binding).is_some() {
                                return Err(invalid("duplicate movie image binding ID"));
                            }
                        }
                        Ok(catalog)
                    })
                    .map_err(|error| error.to_string());
                self.catalogs.insert(current.0, catalog);
            }
            let binding = match &self.catalogs[&current.0] {
                Ok(catalog) => catalog.get(&current.1).cloned(),
                Err(error) => {
                    image.issues.push(PreparationIssue::message(
                        PreparationStage::BitmapBinding,
                        error.clone(),
                    ));
                    None
                }
            };
            if let Some(binding) = binding {
                if let ImageResolution::Resolved { location, .. } = &binding.resolution {
                    image.texture = Some(self.texture(*location)?);
                } else {
                    image.issues.push(PreparationIssue::message(
                        PreparationStage::BitmapBinding,
                        format!(
                            "external image binding unresolved: {:?}",
                            binding.resolution
                        ),
                    ));
                }
                image.binding = Some(binding);
            } else if image.issues.is_empty() {
                image.issues.push(PreparationIssue::message(
                    PreparationStage::BitmapBinding,
                    "external image has no declared texture binding",
                ));
            }
            break;
        }
        self.result.images.push(image);
        Ok(index)
    }

    fn texture(&mut self, location: AssetLocation) -> io::Result<usize> {
        if let Some(&index) = self.textures.get(&location) {
            return Ok(index);
        }
        if self.result.textures.len() >= self.limits.max_textures {
            return Err(invalid("scene-content texture budget exceeded"));
        }
        let index = self.result.textures.len();
        self.textures.insert(location, index);
        let mut asset = TextureAsset {
            location,
            descriptor: None,
            mip_index: 0,
            physical_dimensions: None,
            image: None,
            issues: Vec::new(),
        };
        match self.assets.read_texture(location) {
            Ok(texture) => {
                asset.descriptor = Some(texture.descriptor.clone());
                let mip = texture
                    .mips
                    .first()
                    .ok_or_else(|| invalid("texture has no primary mip"))?;
                asset.physical_dimensions = Some([mip.width, mip.height]);
                if matches!(mip.bulk.payload, BulkPayload::Inline(_))
                    && mip.bulk.flags & (0x02 | 0x10 | 0x80) == 0
                {
                    let expected = rgba_bytes(mip.width, mip.height)?;
                    let remaining =
                        self.limits.max_decoded_rgba_bytes - self.result.stats.decoded_rgba_bytes;
                    if expected > remaining {
                        return Err(invalid("scene-content decoded RGBA byte budget exceeded"));
                    }
                }
                match texture.decode_mip(0) {
                    Ok(image) => {
                        if image.pixels.len() != rgba_bytes(image.width, image.height)? {
                            return Err(invalid("decoded texture length is inconsistent"));
                        }
                        add_budget(
                            &mut self.result.stats.decoded_rgba_bytes,
                            image.pixels.len(),
                            self.limits.max_decoded_rgba_bytes,
                            "decoded RGBA byte",
                        )?;
                        asset.image = Some(image);
                    }
                    Err(error) => asset.issues.push(PreparationIssue::error(
                        PreparationStage::TextureDecode,
                        error,
                    )),
                }
            }
            Err(error) => asset.issues.push(PreparationIssue::error(
                PreparationStage::TextureRead,
                error,
            )),
        }
        self.result.textures.push(asset);
        Ok(index)
    }
}

fn rgba_bytes(width: u32, height: u32) -> io::Result<usize> {
    usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| invalid("RGBA size overflow"))
}

fn add_budget(total: &mut usize, amount: usize, limit: usize, name: &str) -> io::Result<()> {
    let next = total
        .checked_add(amount)
        .ok_or_else(|| invalid(format!("{name} total overflow")))?;
    if next > limit {
        return Err(invalid(format!("scene-content {name} budget exceeded")));
    }
    *total = next;
    Ok(())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
