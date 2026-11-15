//! Owned CPU preparation of actual scene definitions and declared image bindings.
//! No GPU calls, invented geometry, script execution, or implicit UV/color policy.

use super::library::{MovieLibrary, Resolution, SymbolBinding};
use super::{AssetLocation, AssetSet, ImageBinding, ImageResolution};
use crate::gfx::edit_layout::{self, EditLayoutError, EditTextLayout};
use crate::gfx::glyphs::{FontIdentity, GlyphMesh, GlyphOutline, OutlineLimits, ResolvedFont};
use crate::gfx::scene::{
    ActionNode, Affine, DefinitionRef, MovieId, NodeId, OpaqueNode, Scene, SceneNode, TagRef,
};
use crate::gfx::shapes::{self, Color, Fill, FillRule, Shape};
use crate::gfx::tessellate::{self, Mesh};
use crate::gfx::text::{EditText, FontDefinition};
use crate::gfx::{CharacterKind, TagData};
use crate::texture::{BulkPayload, Descriptor, RgbaImage};
use std::collections::{BTreeMap, BTreeSet};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_meshes: usize,
    pub max_texts: usize,
    pub max_fonts: usize,
    pub max_glyphs: usize,
    pub max_text_bytes: usize,
    pub max_font_bytes: usize,
    pub max_glyph_records: usize,
    pub max_glyph_placements: usize,
    pub glyph_fill_rule: FillRule,
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
            max_texts: 4096,
            max_fonts: 128,
            max_glyphs: 16_384,
            max_text_bytes: 16 * 1024 * 1024,
            max_font_bytes: 64 * 1024 * 1024,
            max_glyph_records: 2_000_000,
            max_glyph_placements: 1_000_000,
            glyph_fill_rule: FillRule::EvenOdd,
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
    TextDecode,
    FontBinding,
    FontDecode,
    TextLayout,
    GlyphDecode,
    GlyphTessellation,
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
    SavedTextStateOnly,
    TextBoundsClippingNotApplied,
    TextBorderNotPrepared,
    FontRasterizationUnverified,
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
    /// Text cache entry, including explicit unsupported or failed text records.
    pub text: Option<usize>,
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

/// Exact local font reference and its library-resolved target. No fallback.
#[derive(Clone, Debug)]
pub struct FontBinding {
    pub movie_id: MovieId,
    pub character_id: u16,
    pub import_chain: Vec<SymbolBinding>,
    pub font: Option<usize>,
    pub issues: Vec<PreparationIssue>,
}

#[derive(Clone, Debug)]
pub struct FontAsset {
    pub definition: DefinitionRef,
    pub font: Option<FontDefinition>,
    pub issues: Vec<PreparationIssue>,
}

/// A cached original glyph in font EM units with a neutral white fill.
/// TextGlyph.color supplies the saved paint BEFORE the scene color transform.
#[derive(Clone, Debug)]
pub struct GlyphAsset {
    pub font: FontIdentity,
    pub glyph_index: usize,
    pub outline: Option<GlyphOutline>,
    pub mesh: Option<GlyphMesh>,
    pub issues: Vec<PreparationIssue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyph {
    pub layout_index: usize,
    pub glyph: usize,
    /// EM -> field-local twips; compose world_matrix.compose(local_matrix).
    pub local_matrix: Affine,
    /// None is unresolved paint, never an implied white/black default.
    pub color: Option<[u8; 4]>,
}

#[derive(Clone, Debug)]
pub struct TextAsset {
    pub definition: DefinitionRef,
    pub field: Option<EditText>,
    pub font_binding: Option<FontBinding>,
    /// Original saved initial plain text only. No localization or VM updates.
    pub layout: Option<EditTextLayout>,
    pub layout_error: Option<EditLayoutError>,
    /// Aligned with layout.glyphs; failed geometry remains a referenced cache entry.
    pub glyphs: Vec<TextGlyph>,
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
    pub text_bytes: usize,
    pub font_bytes: usize,
    pub glyph_records: usize,
    pub glyph_placements: usize,
}

#[derive(Clone, Debug)]
pub struct PreparedScene {
    pub root: NodeId,
    pub nodes: Vec<PreparedNode>,
    pub meshes: Vec<MeshAsset>,
    pub texts: Vec<TextAsset>,
    pub fonts: Vec<FontAsset>,
    pub glyphs: Vec<GlyphAsset>,
    pub glyph_fill_rule: FillRule,
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
            texts: Vec::new(),
            fonts: Vec::new(),
            glyphs: Vec::new(),
            glyph_fill_rule: limits.glyph_fill_rule,
            images: Vec::new(),
            textures: Vec::new(),
            actions: scene.actions.clone(),
            opaque_nodes: scene.opaque_nodes.clone(),
            unbound_action_tags: scene.unbound_action_tags.clone(),
            stats: Statistics::default(),
            tessellation: limits.tessellation,
        },
        meshes: BTreeMap::new(),
        texts: BTreeMap::new(),
        fonts: BTreeMap::new(),
        glyphs: BTreeMap::new(),
        images: BTreeMap::new(),
        textures: BTreeMap::new(),
        catalogs: BTreeMap::new(),
    };
    for (index, node) in scene.nodes.iter().enumerate() {
        if node.id != index {
            return Err(invalid("scene node index/identity mismatch"));
        }
        let mut limitations = node_limitations(node);
        let mut text = None;
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
                    let index = builder.text(definition)?;
                    let asset = &builder.result.texts[index];
                    if asset.layout.is_none() {
                        limitations.push(Limitation::TextNotPrepared);
                    } else {
                        limitations.push(Limitation::SavedTextStateOnly);
                        limitations.push(Limitation::TextBoundsClippingNotApplied);
                        limitations.push(Limitation::FontRasterizationUnverified);
                    }
                    if asset
                        .field
                        .as_ref()
                        .is_some_and(|field| field.flags.border())
                    {
                        limitations.push(Limitation::TextBorderNotPrepared);
                    }
                    text = Some(index);
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
            text,
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
    texts: BTreeMap<(MovieId, u16), usize>,
    fonts: BTreeMap<(MovieId, u16), usize>,
    glyphs: BTreeMap<(FontIdentity, usize), usize>,
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

    fn text(&mut self, definition: DefinitionRef) -> io::Result<usize> {
        let key = (definition.movie_id, definition.character_id);
        if let Some(&index) = self.texts.get(&key) {
            return Ok(index);
        }
        if self.result.texts.len() >= self.limits.max_texts {
            return Err(invalid("scene-content text budget exceeded"));
        }
        let index = self.result.texts.len();
        let mut asset = TextAsset {
            definition,
            field: None,
            font_binding: None,
            layout: None,
            layout_error: None,
            glyphs: Vec::new(),
            issues: Vec::new(),
        };
        let movie = &self.library.entry(definition.movie_id).unwrap().asset.movie;
        let tag = movie
            .tags
            .get(definition.tag_index)
            .ok_or_else(|| invalid("text tag index out of bounds"))?;
        add_budget(
            &mut self.result.stats.text_bytes,
            tag.body.len,
            self.limits.max_text_bytes,
            "text source byte",
        )?;
        if tag.code != 37 {
            asset.issues.push(PreparationIssue::message(
                PreparationStage::TextDecode,
                "static or other text definition is not prepared by the EditText bridge",
            ));
        } else {
            match EditText::parse(tag.body.bytes(movie.bytes())?, tag.body.offset) {
                Err(error) => asset
                    .issues
                    .push(PreparationIssue::error(PreparationStage::TextDecode, error)),
                Ok(field) => {
                    if field.id != definition.character_id {
                        return Err(invalid("decoded EditText ID differs from definition"));
                    }
                    // These unsupported formats are rejected by layout_initial before
                    // font selection or content layout, including empty passwords.
                    if !field.flags.password() && !field.flags.html() {
                        if let Some(local_id) = field.font_id {
                            asset.font_binding =
                                Some(self.font_binding(definition.movie_id, local_id)?);
                        }
                        let potential = field.initial_text.as_deref().unwrap_or("").chars().count();
                        if potential
                            > self.limits.max_glyph_placements - self.result.stats.glyph_placements
                        {
                            return Err(invalid(
                                "scene-content glyph placement preflight budget exceeded",
                            ));
                        }
                    }
                    let resolved = asset
                        .font_binding
                        .as_ref()
                        .and_then(|binding| binding.font)
                        .and_then(|index| {
                            let cached = &self.result.fonts[index];
                            cached.font.as_ref().map(|font| ResolvedFont {
                                identity: FontIdentity {
                                    movie_id: cached.definition.movie_id,
                                    character_id: cached.definition.character_id,
                                },
                                definition: font,
                            })
                        });
                    match edit_layout::layout_initial(&field, |id| {
                        if Some(id) != field.font_id {
                            return Err(EditLayoutError::UnresolvedFont { local_id: id });
                        }
                        resolved.ok_or(EditLayoutError::UnresolvedFont { local_id: id })
                    }) {
                        Err(error) => {
                            if matches!(error, EditLayoutError::Limit(_)) {
                                return Err(invalid(format!(
                                    "scene-content text layout limit: {error}"
                                )));
                            }
                            asset.layout_error = Some(error);
                        }
                        Ok(layout) => {
                            add_budget(
                                &mut self.result.stats.glyph_placements,
                                layout.glyphs.len(),
                                self.limits.max_glyph_placements,
                                "glyph placement",
                            )?;
                            if field.color.is_none() && !layout.glyphs.is_empty() {
                                asset.issues.push(PreparationIssue::message(
                                    PreparationStage::TextLayout,
                                    "saved text color is absent; paint must remain unresolved",
                                ));
                            }
                            for (layout_index, glyph) in layout.glyphs.iter().enumerate() {
                                let glyph_index = self.glyph(glyph.font, glyph.glyph_index)?;
                                asset.glyphs.push(TextGlyph {
                                    layout_index,
                                    glyph: glyph_index,
                                    local_matrix: edit_glyph_matrix(glyph),
                                    color: glyph.color,
                                });
                            }
                            asset.layout = Some(layout);
                        }
                    }
                    asset.field = Some(field);
                }
            }
        }
        self.texts.insert(key, index);
        self.result.texts.push(asset);
        Ok(index)
    }

    fn font_binding(&mut self, movie_id: MovieId, character_id: u16) -> io::Result<FontBinding> {
        let mut result = FontBinding {
            movie_id,
            character_id,
            import_chain: Vec::new(),
            font: None,
            issues: Vec::new(),
        };
        let mut current = (movie_id, character_id);
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current) {
                result.issues.push(PreparationIssue::message(
                    PreparationStage::FontBinding,
                    "font import cycle",
                ));
                break;
            }
            let Some(entry) = self.library.entry(current.0) else {
                result.issues.push(PreparationIssue::message(
                    PreparationStage::FontBinding,
                    "font target movie is absent",
                ));
                break;
            };
            let Some(character) = entry.asset.movie.dictionary.get(&current.1) else {
                result.issues.push(PreparationIssue::message(
                    PreparationStage::FontBinding,
                    "font character is absent from dictionary",
                ));
                break;
            };
            if character.kind == CharacterKind::Imported {
                if result.import_chain.len() >= self.limits.max_import_hops {
                    return Err(invalid("scene-content font import budget exceeded"));
                }
                let Some(binding) = self.library.binding(current.0, current.1) else {
                    result.issues.push(PreparationIssue::message(
                        PreparationStage::FontBinding,
                        "imported font has no declared library binding",
                    ));
                    break;
                };
                result.import_chain.push(binding.clone());
                if let Resolution::Resolved {
                    movie,
                    character_id,
                    ..
                } = binding.resolution
                {
                    current = (movie, character_id);
                    continue;
                }
                result.issues.push(PreparationIssue::message(
                    PreparationStage::FontBinding,
                    format!("font import unresolved: {:?}", binding.resolution),
                ));
                break;
            }
            if character.kind != CharacterKind::Font {
                result.issues.push(PreparationIssue::message(
                    PreparationStage::FontBinding,
                    format!("text references {:?} instead of a font", character.kind),
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
            result.font = Some(self.font(definition)?);
            break;
        }
        Ok(result)
    }

    fn font(&mut self, definition: DefinitionRef) -> io::Result<usize> {
        let key = (definition.movie_id, definition.character_id);
        if let Some(&index) = self.fonts.get(&key) {
            return Ok(index);
        }
        if self.result.fonts.len() >= self.limits.max_fonts {
            return Err(invalid("scene-content font budget exceeded"));
        }
        let movie = &self.library.entry(definition.movie_id).unwrap().asset.movie;
        let tag = movie
            .tags
            .get(definition.tag_index)
            .ok_or_else(|| invalid("font tag index out of bounds"))?;
        add_budget(
            &mut self.result.stats.font_bytes,
            tag.body.len,
            self.limits.max_font_bytes,
            "font source byte",
        )?;
        let mut asset = FontAsset {
            definition,
            font: None,
            issues: Vec::new(),
        };
        match FontDefinition::parse(tag.code, tag.body.bytes(movie.bytes())?, tag.body.offset) {
            Ok(font) => {
                if font.id != definition.character_id {
                    return Err(invalid("decoded font ID differs from definition"));
                }
                asset.font = Some(font);
            }
            Err(error) => asset
                .issues
                .push(PreparationIssue::error(PreparationStage::FontDecode, error)),
        }
        let index = self.result.fonts.len();
        self.fonts.insert(key, index);
        self.result.fonts.push(asset);
        Ok(index)
    }

    fn glyph(&mut self, identity: FontIdentity, glyph_index: usize) -> io::Result<usize> {
        let key = (identity, glyph_index);
        if let Some(&index) = self.glyphs.get(&key) {
            return Ok(index);
        }
        if self.result.glyphs.len() >= self.limits.max_glyphs {
            return Err(invalid("scene-content glyph mesh budget exceeded"));
        }
        let font_index = *self
            .fonts
            .get(&(identity.movie_id, identity.character_id))
            .ok_or_else(|| invalid("layout font was not prepared"))?;
        let font = self.result.fonts[font_index]
            .font
            .as_ref()
            .ok_or_else(|| invalid("layout font has no decoded definition"))?;
        let movie = &self.library.entry(identity.movie_id).unwrap().asset.movie;
        let mut asset = GlyphAsset {
            font: identity,
            glyph_index,
            outline: None,
            mesh: None,
            issues: Vec::new(),
        };
        let mut limits = OutlineLimits::default();
        limits.max_records = limits
            .max_records
            .min(self.limits.max_glyph_records - self.result.stats.glyph_records);
        match GlyphOutline::from_font_with_limits(font, glyph_index, movie.bytes(), limits) {
            Err(error) => {
                if error.to_string().contains("limit exceeded") {
                    return Err(error);
                }
                asset.issues.push(PreparationIssue::error(
                    PreparationStage::GlyphDecode,
                    error,
                ));
            }
            Ok(outline) => {
                let records = outline
                    .records
                    .len()
                    .checked_add(outline.edges.len())
                    .ok_or_else(|| invalid("glyph record count overflow"))?;
                add_budget(
                    &mut self.result.stats.glyph_records,
                    records,
                    self.limits.max_glyph_records,
                    "glyph record",
                )?;
                let mut options = self.limits.tessellation;
                options.max_triangles = options
                    .max_triangles
                    .min(self.limits.max_triangles - self.result.stats.triangles);
                // Shape winding override is separate from the explicit glyph policy.
                options.fill_rule_override = None;
                match outline.tessellate(
                    Color {
                        red: 255,
                        green: 255,
                        blue: 255,
                        alpha: 255,
                    },
                    self.limits.glyph_fill_rule,
                    options,
                ) {
                    Err(error) => {
                        if error.to_string().contains("budget") {
                            return Err(error);
                        }
                        asset.issues.push(PreparationIssue::error(
                            PreparationStage::GlyphTessellation,
                            error,
                        ));
                    }
                    Ok(mesh) => {
                        let count = mesh.mesh.fills.iter().try_fold(0usize, |total, fill| {
                            total
                                .checked_add(fill.indices.len() / 3)
                                .ok_or_else(|| invalid("glyph triangle count overflow"))
                        })?;
                        add_budget(
                            &mut self.result.stats.triangles,
                            count,
                            self.limits.max_triangles,
                            "triangle",
                        )?;
                        asset.mesh = Some(mesh);
                    }
                }
                asset.outline = Some(outline);
            }
        }
        let index = self.result.glyphs.len();
        self.glyphs.insert(key, index);
        self.result.glyphs.push(asset);
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

/// The layout owns the exact rational pen; conversion happens only at this
/// geometry boundary. No bounds-origin shift or implicit pixel conversion.
fn edit_glyph_matrix(glyph: &edit_layout::EditGlyph) -> Affine {
    let scale = f64::from(glyph.height_twips) / f64::from(glyph.units_per_em);
    Affine {
        a: scale,
        b: 0.0,
        c: 0.0,
        d: scale,
        tx: glyph.x.as_f64(),
        ty: glyph.baseline.as_f64(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Span;
    use crate::gfx::edit_layout::{EditGlyph, TextSource, Twips};

    #[test]
    fn glyph_em_scale_and_fractional_baseline_precede_world_rotation() {
        let glyph = EditGlyph {
            source: TextSource {
                bytes: 0..1,
                utf16: 0..1,
                movie_bytes: None,
            },
            unicode: 'A',
            code: 65,
            glyph_index: 4,
            shape_source: Span { offset: 0, len: 0 },
            font: FontIdentity {
                movie_id: 3,
                character_id: 9,
            },
            local_font_id: 2,
            line_index: 0,
            x: Twips {
                numerator: 1025,
                denominator: 2048,
            },
            baseline: Twips {
                numerator: -7169,
                denominator: 2048,
            },
            advance: Twips {
                numerator: 0,
                denominator: 2048,
            },
            height_twips: 25,
            units_per_em: 1024,
            color: Some([17, 31, 42, 64]),
        };
        let world = Affine {
            a: 0.0,
            b: 2.0,
            c: -3.0,
            d: 0.0,
            tx: 100.0,
            ty: 200.0,
        };
        let point = world
            .compose(edit_glyph_matrix(&glyph))
            .transform(20.0, -30.0);
        // Exact rational reference: local (2025,-8669)/2048 twips.
        assert_eq!(point, [100.0 + 26007.0 / 2048.0, 200.0 + 4050.0 / 2048.0]);
    }
}
