//! Offscreen composition of a prepared structural scene. This is a diagnostic
//! of saved geometry, not script execution or a completed frontend.

use super::scene_content::{self, NodeContent, PreparationIssue, PreparedScene};
use crate::{
    gfx::{
        Rect, Span,
        edit_layout::EditLayoutError,
        scene::{NodeId, SceneNode},
        shapes::{Color, Fill, FillRule, FillStyle, StyleRef},
        tessellate::{FillMesh, Position},
    },
    render::{
        self, BlendMode, GpuImage, ImageEncoding, ImageFilter, QuadRenderer, Readback,
        mesh::{
            self, Affine, BitmapBinding, ColorTransform, Command, Draw, GradientAlpha,
            LinearRgbTransfer, MaskPart, Paint, TransformSpace,
        },
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Background {
    Transparent,
    SavedRootTagOrTransparent,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapMapping {
    OmitUnverified,
    WholeImageReplacement,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effects {
    OmitAffectedSubtree,
    /// Render saved geometry without implementing the reported effects.
    BaseGeometry,
}
/// Every approximation is a caller selection. There is no retail preset here.
#[derive(Clone, Copy, Debug)]
pub struct PreviewPolicy {
    pub background: Background,
    pub colors: ImageEncoding,
    pub color_transform_space: TransformSpace,
    pub linear_rgb_transfer: LinearRgbTransfer,
    pub gradient_alpha: GradientAlpha,
    pub bitmap_mapping: BitmapMapping,
    pub unknown_texture_encoding: Option<ImageEncoding>,
    pub effects: Effects,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_commands: usize,
    pub max_mask_parts: usize,
    pub max_mask_visits: usize,
    pub max_uploaded_bytes: usize,
    pub max_text_mesh_bytes: usize,
    pub max_text_glyphs: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_commands: 65_536,
            max_mask_parts: 65_536,
            max_mask_visits: 2_000_000,
            max_uploaded_bytes: 256 * 1024 * 1024,
            max_text_mesh_bytes: 64 * 1024 * 1024,
            max_text_glyphs: 100_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkipReason {
    Hidden,
    MaskSubtree,
    EffectAncestor(NodeId),
    MeshUnavailable(usize),
    BitmapMappingUnverified(usize),
    BitmapUnavailable(usize),
    BitmapLogicalSizeUnavailable(usize),
    TextureUnavailable(usize),
    UnsupportedMask(NodeId),
    TextNotPrepared,
    TextPaintUnavailable,
    GlyphUnavailable(usize),
    TextClipNestingExceeded,
    DirectBitmapGeometryNotPrepared,
    UnrenderedContent,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaskRejection {
    HiddenNode(NodeId),
    EffectOnNodeOrAncestor(NodeId),
    UnsupportedNodeMetadata(NodeId),
    NestedMaskInsideMask(NodeId),
    UnsupportedContent(NodeId),
    MeshUnavailable(NodeId),
    PaintNotOpaqueSolid { node: NodeId, fill: usize },
}
#[derive(Clone, Debug)]
pub enum Limitation {
    /// Shape pixels do not establish original runtime, transfer or AA parity.
    StaticGeometryDiagnostic,
    ScriptsNotExecuted {
        bound: usize,
        unbound: usize,
    },
    OpaqueTagsNotExecuted(usize),
    SourceNode {
        node: NodeId,
        issue: scene_content::Limitation,
    },
    SourceMesh {
        mesh: usize,
        issue: PreparationIssue,
    },
    SourceBitmap {
        image: usize,
        issue: PreparationIssue,
    },
    SourceTexture {
        texture: usize,
        issue: PreparationIssue,
    },
    SourceText {
        text: usize,
        issue: PreparationIssue,
    },
    SourceFontBinding {
        text: usize,
        issue: PreparationIssue,
    },
    SourceFont {
        font: usize,
        issue: PreparationIssue,
    },
    SourceGlyph {
        glyph: usize,
        issue: PreparationIssue,
    },
    TextLayoutError {
        text: usize,
        error: EditLayoutError,
    },
    TextOverflow {
        text: usize,
        x: bool,
        y: bool,
    },
    GlyphFillRuleSelected {
        rule: FillRule,
    },
    SkippedTextGlyph {
        node: NodeId,
        placement: usize,
        reason: SkipReason,
    },
    SkippedNode {
        node: NodeId,
        reason: SkipReason,
    },
    SkippedFill {
        node: NodeId,
        fill: usize,
        reason: SkipReason,
    },
    StrokeEdgesNotRendered {
        node: NodeId,
        count: usize,
    },
    InferredFillRule {
        node: NodeId,
    },
    WholeImageMappingSelected {
        image: usize,
        logical: [u16; 2],
        physical: [u32; 2],
    },
    UnknownTextureEncodingSelected {
        texture: usize,
        encoding: ImageEncoding,
    },
    UnknownTextureEncodingOmitted {
        texture: usize,
    },
    TextureUploadFailed {
        texture: usize,
        kind: io::ErrorKind,
        description: String,
    },
    MaskRejected {
        mask: NodeId,
        reason: MaskRejection,
    },
}
#[derive(Clone, Debug, Default)]
pub struct NodeCoverage {
    pub node: NodeId,
    pub submitted_fills: usize,
    pub source_fill_count: usize,
    pub submitted_triangles: usize,
    pub skip: Option<SkipReason>,
}
#[derive(Clone, Debug, Default)]
pub struct Coverage {
    pub source_nodes: usize,
    pub shape_instances: usize,
    pub shape_instances_with_submitted_fills: usize,
    pub fill_draws: usize,
    pub triangles: usize,
    pub mask_sources_used: usize,
    pub mask_pushes: usize,
    pub mask_parts_submitted: usize,
    pub uploaded_textures: usize,
    pub uploaded_rgba_bytes: usize,
    pub text_instances: usize,
    pub text_instances_prepared: usize,
    pub text_instances_with_submitted_fills: usize,
    pub text_instances_empty: usize,
    pub text_instances_omitted: usize,
    pub text_glyphs: usize,
    pub text_glyphs_submitted: usize,
    pub text_glyphs_omitted: usize,
    pub text_glyphs_without_geometry: usize,
    pub text_fill_draws: usize,
    pub text_triangles: usize,
    pub text_fields_clipped: usize,
    pub direct_bitmap_instances_omitted: usize,
    pub stroke_edges_unrendered: usize,
    pub source_actions_unexecuted: usize,
    pub unbound_action_tags_unexecuted: usize,
    pub nodes: Vec<NodeCoverage>,
}
#[derive(Debug)]
pub struct DiagnosticFrame {
    pub readback: Readback,
    pub stage: Rect,
    pub policy: PreviewPolicy,
    /// Some means the actual saved root background tag supplied the clear.
    pub saved_background_applied: Option<[u8; 3]>,
    pub coverage: Coverage,
    pub limitations: Vec<Limitation>,
}

enum PlannedGeometry {
    Shape(usize, usize),
    Glyph(usize),
}
struct PlannedDraw {
    node: NodeId,
    geometry: PlannedGeometry,
    local: Affine,
    text_placement: Option<usize>,
    fill: usize,
    image: Option<usize>,
}
struct MaskGeometry {
    parts: Vec<(NodeId, usize, usize)>,
}

pub fn render_static(
    source: &QuadRenderer,
    scene: &PreparedScene,
    stage: Rect,
    policy: PreviewPolicy,
) -> io::Result<DiagnosticFrame> {
    render_static_with_limits(source, scene, stage, policy, Limits::default())
}

pub fn render_static_with_limits(
    source: &QuadRenderer,
    scene: &PreparedScene,
    stage: Rect,
    policy: PreviewPolicy,
    limits: Limits,
) -> io::Result<DiagnosticFrame> {
    let (width, height, stage_to_pixels) = stage_mapping(stage)?;
    let device_limit = source.device().limits().max_texture_dimension_2d;
    if width > device_limit || height > device_limit {
        return Err(invalid("native stage exceeds device texture dimensions"));
    }
    let order = validate_tree(scene, limits)?;
    let mut limitations = source_limitations(scene);
    let mut coverage = Coverage {
        source_nodes: scene.nodes.len(),
        source_actions_unexecuted: scene.actions.len(),
        unbound_action_tags_unexecuted: scene.unbound_action_tags.len(),
        nodes: (0..scene.nodes.len())
            .map(|node| NodeCoverage {
                node,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let mut draws = Vec::new();
    let mut colored_glyphs: Vec<FillMesh> = Vec::new();
    let mut glyph_cache: BTreeMap<(usize, [u8; 4]), Vec<usize>> = BTreeMap::new();
    let mut text_clips: BTreeMap<NodeId, FillMesh> = BTreeMap::new();
    let mut text_mesh_bytes = 0usize;
    let mut mask_cache: BTreeMap<NodeId, Result<MaskGeometry, MaskRejection>> = BTreeMap::new();
    let mut in_mask = vec![false; scene.nodes.len()];
    let mut effect_ancestor = vec![None; scene.nodes.len()];
    let mut mask_visits = 0usize;
    let mut mask_parts = 0usize;
    for &id in &order {
        let node = &scene.nodes[id];
        let instance = &node.instance;
        in_mask[id] = instance.is_mask || instance.parent.is_some_and(|p| in_mask[p]);
        effect_ancestor[id] = instance
            .parent
            .and_then(|p| effect_ancestor[p])
            .or_else(|| has_effect(instance).then_some(id));
        let skip = if in_mask[id] {
            Some(SkipReason::MaskSubtree)
        } else if !instance.effective_visible {
            Some(SkipReason::Hidden)
        } else if policy.effects == Effects::OmitAffectedSubtree {
            effect_ancestor[id].map(SkipReason::EffectAncestor)
        } else {
            None
        };
        if let Some(reason) = skip {
            skipped_node(id, reason, &mut coverage, &mut limitations);
            continue;
        }
        match node.content {
            NodeContent::Text { .. } => coverage.text_instances += 1,
            NodeContent::Shape { .. } => coverage.shape_instances += 1,
            _ => {}
        }
        if matches!(
            node.content,
            NodeContent::Shape { .. } | NodeContent::Text { .. }
        ) {
            let mut rejected = None;
            for &mask in &instance.masks {
                if !mask_cache.contains_key(&mask) {
                    let result =
                        mask_geometry(scene, mask, limits, &mut mask_visits, &mut mask_parts)?;
                    if let Err(reason) = &result {
                        limitations.push(Limitation::MaskRejected {
                            mask,
                            reason: reason.clone(),
                        });
                    }
                    mask_cache.insert(mask, result);
                }
                if mask_cache[&mask].is_err() {
                    rejected = Some(mask);
                    break;
                }
            }
            if let Some(mask) = rejected {
                if matches!(node.content, NodeContent::Text { .. }) {
                    coverage.text_instances_omitted += 1;
                }
                skipped_node(
                    id,
                    SkipReason::UnsupportedMask(mask),
                    &mut coverage,
                    &mut limitations,
                );
                continue;
            }
        }
        match node.content {
            NodeContent::Group => {}
            NodeContent::Text { definition } => {
                let Some(text_index) = node.text else {
                    coverage.text_instances_omitted += 1;
                    skipped_node(
                        id,
                        SkipReason::TextNotPrepared,
                        &mut coverage,
                        &mut limitations,
                    );
                    continue;
                };
                let text = scene
                    .texts
                    .get(text_index)
                    .ok_or_else(|| invalid("node text index outside prepared assets"))?;
                if text.definition != definition {
                    return Err(invalid(
                        "node and prepared text definition identities differ",
                    ));
                }
                let (Some(field), Some(layout)) = (&text.field, &text.layout) else {
                    coverage.text_instances_omitted += 1;
                    skipped_node(
                        id,
                        SkipReason::TextNotPrepared,
                        &mut coverage,
                        &mut limitations,
                    );
                    continue;
                };
                if field.id != layout.field_id
                    || field.bounds != layout.bounds
                    || text.glyphs.len() != layout.glyphs.len()
                {
                    return Err(invalid("prepared text field/layout alignment differs"));
                }
                if instance.masks.len() >= 255 {
                    coverage.text_instances_omitted += 1;
                    skipped_node(
                        id,
                        SkipReason::TextClipNestingExceeded,
                        &mut coverage,
                        &mut limitations,
                    );
                    continue;
                }
                coverage.text_instances_prepared += 1;
                coverage.text_glyphs = coverage
                    .text_glyphs
                    .checked_add(text.glyphs.len())
                    .ok_or_else(|| invalid("text glyph placement count overflow"))?;
                if coverage.text_glyphs > limits.max_text_glyphs {
                    return Err(invalid("preview text glyph placement budget exceeded"));
                }
                if text.glyphs.is_empty() {
                    coverage.text_instances_empty += 1;
                    continue;
                }
                mask_parts = mask_parts
                    .checked_add(1)
                    .ok_or_else(|| invalid("mask part count overflow"))?;
                if mask_parts > limits.max_mask_parts {
                    return Err(invalid("preview text clip/mask part budget exceeded"));
                }
                text_clips.insert(id, field_clip(field.bounds, field.body)?);
                for (placement, glyph) in text.glyphs.iter().enumerate() {
                    let original = &layout.glyphs[placement];
                    let asset = scene
                        .glyphs
                        .get(glyph.glyph)
                        .ok_or_else(|| invalid("text glyph index outside prepared assets"))?;
                    if glyph.layout_index != placement
                        || asset.font != original.font
                        || asset.glyph_index != original.glyph_index
                    {
                        return Err(invalid("prepared glyph identity/layout alignment differs"));
                    }
                    let reason = if glyph.color.is_none() {
                        Some(SkipReason::TextPaintUnavailable)
                    } else if asset.mesh.is_none() {
                        Some(SkipReason::GlyphUnavailable(glyph.glyph))
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        coverage.text_glyphs_omitted += 1;
                        limitations.push(Limitation::SkippedTextGlyph {
                            node: id,
                            placement,
                            reason,
                        });
                        continue;
                    }
                    let geometry = &asset.mesh.as_ref().unwrap().mesh;
                    if geometry.fills.iter().all(|f| f.indices.is_empty()) {
                        coverage.text_glyphs_without_geometry += 1;
                        continue;
                    }
                    let key = (glyph.glyph, glyph.color.unwrap());
                    if !glyph_cache.contains_key(&key) {
                        let mut indices = Vec::new();
                        for source_fill in &geometry.fills {
                            if !matches!(source_fill.paint.fill, Fill::Solid(_)) {
                                return Err(invalid("prepared glyph has non-solid paint"));
                            }
                            let bytes = source_fill
                                .positions
                                .len()
                                .checked_mul(size_of::<Position>())
                                .and_then(|v| {
                                    source_fill
                                        .indices
                                        .len()
                                        .checked_mul(size_of::<u32>())
                                        .and_then(|n| v.checked_add(n))
                                })
                                .and_then(|v| text_mesh_bytes.checked_add(v))
                                .ok_or_else(|| invalid("text draw geometry byte count overflow"))?;
                            if bytes > limits.max_text_mesh_bytes {
                                return Err(invalid(
                                    "preview text draw geometry byte budget exceeded",
                                ));
                            }
                            text_mesh_bytes = bytes;
                            let [red, green, blue, alpha] = key.1;
                            indices.push(colored_glyphs.len());
                            colored_glyphs.push(FillMesh {
                                style: source_fill.style,
                                paint: FillStyle {
                                    source: source_fill.paint.source,
                                    fill: Fill::Solid(Color {
                                        red,
                                        green,
                                        blue,
                                        alpha,
                                    }),
                                },
                                // Original contours remain in the prepared glyph; only GPU
                                // positions/indices and source paint provenance are needed here.
                                contours: Vec::new(),
                                positions: source_fill.positions.clone(),
                                indices: source_fill.indices.clone(),
                                branching_vertices: source_fill.branching_vertices,
                            });
                        }
                        glyph_cache.insert(key, indices);
                    }
                    let m = glyph.local_matrix;
                    let local = Affine([m.a, m.b, m.c, m.d, m.tx, m.ty]);
                    if local.0.iter().any(|v| !v.is_finite()) {
                        return Err(invalid("text glyph local transform is nonfinite"));
                    }
                    for (fill, &colored) in glyph_cache[&key].iter().enumerate() {
                        coverage.nodes[id].source_fill_count += 1;
                        if colored_glyphs[colored].indices.is_empty() {
                            continue;
                        }
                        if draws.len() >= limits.max_commands {
                            return Err(invalid("preview fill draw budget exceeded"));
                        }
                        draws.push(PlannedDraw {
                            node: id,
                            geometry: PlannedGeometry::Glyph(colored),
                            local,
                            text_placement: Some(placement),
                            fill,
                            image: None,
                        });
                    }
                }
            }
            NodeContent::Bitmap { .. } => {
                coverage.direct_bitmap_instances_omitted += 1;
                skipped_node(
                    id,
                    SkipReason::DirectBitmapGeometryNotPrepared,
                    &mut coverage,
                    &mut limitations,
                );
            }
            NodeContent::Unrendered => skipped_node(
                id,
                SkipReason::UnrenderedContent,
                &mut coverage,
                &mut limitations,
            ),
            NodeContent::Shape { mesh } => {
                let asset = scene
                    .meshes
                    .get(mesh)
                    .ok_or_else(|| invalid("node mesh index outside prepared assets"))?;
                let Some(geometry) = &asset.mesh else {
                    skipped_node(
                        id,
                        SkipReason::MeshUnavailable(mesh),
                        &mut coverage,
                        &mut limitations,
                    );
                    continue;
                };
                if geometry.fills.len() != asset.fill_images.len() {
                    return Err(invalid(
                        "fill/image index alignment differs from prepared mesh",
                    ));
                }
                coverage.nodes[id].source_fill_count = geometry.fills.len();
                if geometry.stroke_edges != 0 {
                    coverage.stroke_edges_unrendered += geometry.stroke_edges;
                    limitations.push(Limitation::StrokeEdgesNotRendered {
                        node: id,
                        count: geometry.stroke_edges,
                    });
                }
                if geometry.inferred_winding {
                    limitations.push(Limitation::InferredFillRule { node: id });
                }
                for (fill, shape_fill) in geometry.fills.iter().enumerate() {
                    let image = asset.fill_images[fill];
                    let reason = match &shape_fill.paint.fill {
                        Fill::Bitmap { character_id, .. } => {
                            let Some(index) = image else {
                                return Err(invalid("prepared bitmap fill lacks image index"));
                            };
                            let bitmap = scene.images.get(index).ok_or_else(|| {
                                invalid("fill image index outside prepared assets")
                            })?;
                            if bitmap.character_id != *character_id {
                                return Err(invalid("prepared bitmap character identity mismatch"));
                            }
                            bitmap_reason(scene, index, policy)?
                        }
                        _ => {
                            if image.is_some() {
                                return Err(invalid("non-bitmap fill has prepared image index"));
                            }
                            None
                        }
                    };
                    if let Some(reason) = reason {
                        limitations.push(Limitation::SkippedFill {
                            node: id,
                            fill,
                            reason,
                        });
                        continue;
                    }
                    if draws.len() >= limits.max_commands {
                        return Err(invalid("preview fill draw budget exceeded"));
                    }
                    draws.push(PlannedDraw {
                        node: id,
                        geometry: PlannedGeometry::Shape(mesh, fill),
                        local: Affine::IDENTITY,
                        text_placement: None,
                        fill,
                        image,
                    });
                }
            }
        }
    }

    let image_ids: BTreeSet<_> = draws.iter().filter_map(|d| d.image).collect();
    let texture_ids: BTreeSet<_> = image_ids
        .iter()
        .map(|&id| scene.images[id].texture.unwrap())
        .collect();
    let mut gpu_images: BTreeMap<usize, GpuImage> = BTreeMap::new();
    for &texture in &texture_ids {
        let asset = &scene.textures[texture];
        let image = asset
            .image
            .as_ref()
            .ok_or_else(|| invalid("planned texture image disappeared"))?;
        let encoding = match asset.descriptor.as_ref().and_then(|d| d.srgb) {
            Some(false) => ImageEncoding::Linear,
            Some(true) => ImageEncoding::Srgb,
            None => match policy.unknown_texture_encoding {
                Some(encoding) => {
                    limitations
                        .push(Limitation::UnknownTextureEncodingSelected { texture, encoding });
                    encoding
                }
                None => {
                    limitations.push(Limitation::UnknownTextureEncodingOmitted { texture });
                    continue;
                }
            },
        };
        let total = coverage
            .uploaded_rgba_bytes
            .checked_add(image.pixels.len())
            .ok_or_else(|| invalid("uploaded texture byte count overflow"))?;
        if total > limits.max_uploaded_bytes {
            return Err(invalid("preview texture byte budget exceeded"));
        }
        match source.upload_rgba(
            render::RgbaImage {
                width: image.width,
                height: image.height,
                pixels: &image.pixels,
            },
            encoding,
            ImageFilter::Nearest,
        ) {
            Ok(gpu) => {
                gpu_images.insert(texture, gpu);
                coverage.uploaded_textures += 1;
                coverage.uploaded_rgba_bytes = total;
            }
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
                limitations.push(Limitation::TextureUploadFailed {
                    texture,
                    kind: error.kind(),
                    description: error.to_string(),
                });
            }
            Err(error) => return Err(error),
        }
    }
    for &image in &image_ids {
        let asset = &scene.images[image];
        let texture = asset.texture.unwrap();
        if gpu_images.contains_key(&texture) {
            let actual = scene.textures[texture].image.as_ref().unwrap();
            limitations.push(Limitation::WholeImageMappingSelected {
                image,
                logical: asset.logical_dimensions.unwrap(),
                physical: [actual.width, actual.height],
            });
        }
    }

    let mut masks: BTreeMap<NodeId, Vec<MaskPart<'_>>> = BTreeMap::new();
    for (&id, geometry) in &mask_cache {
        if let Ok(geometry) = geometry {
            let mut parts = Vec::new();
            for &(node, mesh, fill) in &geometry.parts {
                parts.push(MaskPart {
                    mesh: &scene.meshes[mesh].mesh.as_ref().unwrap().fills[fill],
                    transform: world_transform(&scene.nodes[node].instance, stage_to_pixels)?,
                });
            }
            masks.insert(id, parts);
        }
    }
    let mut clip_parts = BTreeMap::new();
    for (&node, geometry) in &text_clips {
        clip_parts.insert(
            node,
            [MaskPart {
                mesh: geometry,
                transform: world_transform(&scene.nodes[node].instance, stage_to_pixels)?,
            }],
        );
    }
    let mut commands = Vec::new();
    let mut active_text_clip = None;
    let mut clipped_fields = BTreeSet::new();
    let mut submitted_glyphs = BTreeSet::new();
    let mut active_masks = Vec::new();
    let mut used_masks = BTreeSet::new();
    for draw in draws {
        let bitmap = if let Some(image) = draw.image {
            let asset = &scene.images[image];
            let texture = asset.texture.unwrap();
            let Some(gpu) = gpu_images.get(&texture) else {
                limitations.push(Limitation::SkippedFill {
                    node: draw.node,
                    fill: draw.fill,
                    reason: SkipReason::TextureUnavailable(texture),
                });
                continue;
            };
            Some(BitmapBinding {
                character_id: asset.character_id,
                image: gpu,
                logical_size: asset.logical_dimensions.unwrap().map(u32::from),
            })
        } else {
            None
        };
        if active_text_clip.is_some_and(|id| id != draw.node) {
            push_command(&mut commands, Command::PopMask, limits)?;
            active_text_clip = None;
        }
        let instance = &scene.nodes[draw.node].instance;
        let common = active_masks
            .iter()
            .zip(&instance.masks)
            .take_while(|(a, b)| a == b)
            .count();
        for _ in common..active_masks.len() {
            push_command(&mut commands, Command::PopMask, limits)?;
        }
        active_masks.truncate(common);
        for &mask in &instance.masks[common..] {
            let parts = masks
                .get(&mask)
                .ok_or_else(|| invalid("planned mask geometry unavailable"))?;
            push_command(
                &mut commands,
                Command::PushMask {
                    parts,
                    scissor: None,
                },
                limits,
            )?;
            active_masks.push(mask);
            used_masks.insert(mask);
            coverage.mask_pushes += 1;
            coverage.mask_parts_submitted = coverage
                .mask_parts_submitted
                .checked_add(parts.len())
                .ok_or_else(|| invalid("mask submission count overflow"))?;
        }
        if draw.text_placement.is_some() && active_text_clip.is_none() {
            let parts = clip_parts
                .get(&draw.node)
                .ok_or_else(|| invalid("text clip geometry unavailable"))?;
            push_command(
                &mut commands,
                Command::PushMask {
                    parts,
                    scissor: None,
                },
                limits,
            )?;
            active_text_clip = Some(draw.node);
            clipped_fields.insert(draw.node);
        }
        let shape_fill = match draw.geometry {
            PlannedGeometry::Shape(mesh, fill) => {
                &scene.meshes[mesh].mesh.as_ref().unwrap().fills[fill]
            }
            PlannedGeometry::Glyph(colored) => &colored_glyphs[colored],
        };
        push_command(
            &mut commands,
            Command::Draw(Draw {
                mesh: shape_fill,
                transform: draw.local.then(world_transform(instance, stage_to_pixels)?),
                color_transform: world_color(instance)?,
                paint: Paint {
                    bitmap,
                    colors: policy.colors,
                    transform_space: policy.color_transform_space,
                    linear_rgb_transfer: policy.linear_rgb_transfer,
                    gradient_alpha: policy.gradient_alpha,
                },
                blend: BlendMode::Alpha,
                scissor: None,
            }),
            limits,
        )?;
        if let Some(placement) = draw.text_placement {
            submitted_glyphs.insert((draw.node, placement));
            coverage.text_fill_draws += 1;
            coverage.text_triangles += shape_fill.indices.len() / 3;
        }
        coverage.fill_draws += 1;
        coverage.triangles += shape_fill.indices.len() / 3;
        coverage.nodes[draw.node].submitted_fills += 1;
        coverage.nodes[draw.node].submitted_triangles += shape_fill.indices.len() / 3;
    }
    if active_text_clip.is_some() {
        push_command(&mut commands, Command::PopMask, limits)?;
    }
    coverage.text_fields_clipped = clipped_fields.len();
    coverage.text_glyphs_submitted = submitted_glyphs.len();
    coverage.text_instances_with_submitted_fills = coverage
        .nodes
        .iter()
        .filter(|n| {
            n.submitted_fills > 0 && matches!(scene.nodes[n.node].content, NodeContent::Text { .. })
        })
        .count();
    limitations.retain(|issue| !matches!(issue,
        Limitation::SourceNode { node, issue: scene_content::Limitation::TextBoundsClippingNotApplied }
            if clipped_fields.contains(node)));
    for _ in &active_masks {
        push_command(&mut commands, Command::PopMask, limits)?;
    }
    coverage.mask_sources_used = used_masks.len();
    coverage.shape_instances_with_submitted_fills = coverage
        .nodes
        .iter()
        .filter(|n| {
            n.submitted_fills > 0
                && matches!(scene.nodes[n.node].content, NodeContent::Shape { .. })
        })
        .count();
    let saved_background_applied = match policy.background {
        Background::Transparent => None,
        Background::SavedRootTagOrTransparent => scene.nodes[scene.root].instance.background,
    };
    let clear = saved_background_applied.map_or([0.; 4], |color| {
        let rgb = color.map(|c| {
            let value = f64::from(c) / 255.;
            if policy.colors == ImageEncoding::Srgb {
                srgb_decode(value)
            } else {
                value
            }
        });
        [rgb[0], rgb[1], rgb[2], 1.]
    });
    let renderer = pollster::block_on(mesh::MeshRenderer::new(source))?;
    let readback = renderer.offscreen_rgba(width, height, clear, &commands)?;
    Ok(DiagnosticFrame {
        readback,
        stage,
        policy,
        saved_background_applied,
        coverage,
        limitations,
    })
}

/// Source-defined TextBounds, used only for stencil coverage, never color.
fn field_clip(bounds: Rect, source: Span) -> io::Result<FillMesh> {
    if bounds.x_max < bounds.x_min || bounds.y_max < bounds.y_min {
        return Err(invalid("text field bounds have negative extents"));
    }
    let positions = [
        (bounds.x_min, bounds.y_min),
        (bounds.x_max, bounds.y_min),
        (bounds.x_max, bounds.y_max),
        (bounds.x_min, bounds.y_max),
    ]
    .map(|(x, y)| Position {
        x: f64::from(x),
        y: f64::from(y),
    })
    .to_vec();
    Ok(FillMesh {
        style: StyleRef { set: 0, index: 1 },
        paint: FillStyle {
            source,
            fill: Fill::Solid(Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            }),
        },
        contours: Vec::new(),
        positions,
        indices: if bounds.x_min == bounds.x_max || bounds.y_min == bounds.y_max {
            Vec::new()
        } else {
            vec![0, 1, 2, 0, 2, 3]
        },
        branching_vertices: 0,
    })
}

fn stage_mapping(stage: Rect) -> io::Result<(u32, u32, Affine)> {
    let width = i64::from(stage.x_max) - i64::from(stage.x_min);
    let height = i64::from(stage.y_max) - i64::from(stage.y_min);
    if width <= 0 || height <= 0 {
        return Err(invalid("stage must have positive twip extents"));
    }
    let width = u32::try_from((width + 19) / 20).map_err(|_| invalid("stage width overflows"))?;
    let height =
        u32::try_from((height + 19) / 20).map_err(|_| invalid("stage height overflows"))?;
    let bytes = u64::from(width) * u64::from(height) * 4;
    if bytes > 256 * 1024 * 1024 {
        return Err(invalid("native stage exceeds 256 MiB target budget"));
    }
    Ok((
        width,
        height,
        Affine([
            0.05,
            0.,
            0.,
            0.05,
            -f64::from(stage.x_min) / 20.,
            -f64::from(stage.y_min) / 20.,
        ]),
    ))
}
fn validate_tree(scene: &PreparedScene, limits: Limits) -> io::Result<Vec<NodeId>> {
    if scene.nodes.len() > limits.max_nodes || scene.root >= scene.nodes.len() {
        return Err(invalid("preview scene node/root budget or identity"));
    }
    let mut seen = vec![false; scene.nodes.len()];
    let mut order = Vec::new();
    let mut stack = vec![(scene.root, None, 0usize)];
    while let Some((id, parent, depth)) = stack.pop() {
        let node = scene
            .nodes
            .get(id)
            .ok_or_else(|| invalid("scene child outside node arena"))?;
        if seen[id] || node.instance.id != id || node.instance.parent != parent || depth > 256 {
            return Err(invalid(
                "scene parent/child identity, cycle or depth violation",
            ));
        }
        seen[id] = true;
        order.push(id);
        let mut mask_ids = BTreeSet::new();
        for &mask in &node.instance.masks {
            if mask == id
                || !mask_ids.insert(mask)
                || !scene.nodes.get(mask).is_some_and(|m| m.instance.is_mask)
            {
                return Err(invalid("invalid/duplicate scene mask source"));
            }
        }
        if node.instance.masks.len() > 255 {
            return Err(invalid("scene mask nesting exceeds renderer capacity"));
        }
        let mut previous = None;
        for &child in &node.instance.children {
            let child_node = scene
                .nodes
                .get(child)
                .ok_or_else(|| invalid("child outside scene"))?;
            let segment = child_node
                .instance
                .segment
                .as_ref()
                .ok_or_else(|| invalid("child has no display depth"))?;
            if previous.is_some_and(|p| p >= segment.depth) {
                return Err(invalid(
                    "children are not in increasing display-depth order",
                ));
            }
            previous = Some(segment.depth);
        }
        stack.extend(
            node.instance
                .children
                .iter()
                .rev()
                .map(|&child| (child, Some(id), depth + 1)),
        );
    }
    if order.len() != scene.nodes.len() {
        return Err(invalid("prepared scene contains unreachable nodes"));
    }
    Ok(order)
}
fn source_limitations(scene: &PreparedScene) -> Vec<Limitation> {
    let mut out = vec![Limitation::StaticGeometryDiagnostic];
    if !scene.actions.is_empty() || !scene.unbound_action_tags.is_empty() {
        out.push(Limitation::ScriptsNotExecuted {
            bound: scene.actions.len(),
            unbound: scene.unbound_action_tags.len(),
        });
    }
    if !scene.opaque_nodes.is_empty() {
        out.push(Limitation::OpaqueTagsNotExecuted(scene.opaque_nodes.len()));
    }
    for node in &scene.nodes {
        for issue in &node.limitations {
            out.push(Limitation::SourceNode {
                node: node.instance.id,
                issue: issue.clone(),
            });
        }
    }
    for (mesh, asset) in scene.meshes.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceMesh {
                mesh,
                issue: issue.clone(),
            });
        }
    }
    if !scene.texts.is_empty() {
        out.push(Limitation::GlyphFillRuleSelected {
            rule: scene.glyph_fill_rule,
        });
    }
    for (text, asset) in scene.texts.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceText {
                text,
                issue: issue.clone(),
            });
        }
        if let Some(binding) = &asset.font_binding {
            for issue in &binding.issues {
                out.push(Limitation::SourceFontBinding {
                    text,
                    issue: issue.clone(),
                });
            }
        }
        if let Some(error) = &asset.layout_error {
            out.push(Limitation::TextLayoutError {
                text,
                error: error.clone(),
            });
        }
        if let Some(layout) = &asset.layout {
            if layout.overflow_x || layout.overflow_y {
                out.push(Limitation::TextOverflow {
                    text,
                    x: layout.overflow_x,
                    y: layout.overflow_y,
                });
            }
        }
    }
    for (font, asset) in scene.fonts.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceFont {
                font,
                issue: issue.clone(),
            });
        }
    }
    for (glyph, asset) in scene.glyphs.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceGlyph {
                glyph,
                issue: issue.clone(),
            });
        }
    }
    for (image, asset) in scene.images.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceBitmap {
                image,
                issue: issue.clone(),
            });
        }
    }
    for (texture, asset) in scene.textures.iter().enumerate() {
        for issue in &asset.issues {
            out.push(Limitation::SourceTexture {
                texture,
                issue: issue.clone(),
            });
        }
    }
    out
}
fn skipped_node(
    node: NodeId,
    reason: SkipReason,
    coverage: &mut Coverage,
    limitations: &mut Vec<Limitation>,
) {
    coverage.nodes[node].skip = Some(reason.clone());
    limitations.push(Limitation::SkippedNode { node, reason });
}
fn has_effect(node: &SceneNode) -> bool {
    !node.scaling_grids.is_empty()
        || node.raw.as_ref().is_some_and(|raw| {
            !raw.filters.is_empty()
                || !matches!(raw.blend_mode, 0 | 1)
                || raw.bitmap_cache != 0
                || raw.opaque_background.is_some()
                || raw.ratio != 0
        })
}
fn world_transform(node: &SceneNode, stage: Affine) -> io::Result<Affine> {
    let m = node.world_matrix;
    let result = Affine([m.a, m.b, m.c, m.d, m.tx, m.ty]).then(stage);
    if result.0.iter().any(|v| !v.is_finite()) {
        return Err(invalid("world/stage transform is nonfinite"));
    }
    Ok(result)
}
fn world_color(node: &SceneNode) -> io::Result<ColorTransform> {
    let multiply = node.world_color.multiply.map(|v| v as f32);
    let add = node.world_color.add.map(|v| (v / 255.) as f32);
    if multiply.iter().chain(&add).any(|v| !v.is_finite()) {
        return Err(invalid("world color transform exceeds finite shader range"));
    }
    Ok(ColorTransform { multiply, add })
}
fn bitmap_reason(
    scene: &PreparedScene,
    image: usize,
    policy: PreviewPolicy,
) -> io::Result<Option<SkipReason>> {
    let bitmap = &scene.images[image];
    if policy.bitmap_mapping == BitmapMapping::OmitUnverified {
        return Ok(Some(SkipReason::BitmapMappingUnverified(image)));
    }
    if bitmap
        .logical_dimensions
        .is_none_or(|size| size.contains(&0))
    {
        return Ok(Some(SkipReason::BitmapLogicalSizeUnavailable(image)));
    }
    let Some(texture) = bitmap.texture else {
        return Ok(Some(SkipReason::BitmapUnavailable(image)));
    };
    let texture_asset = scene
        .textures
        .get(texture)
        .ok_or_else(|| invalid("bitmap texture index outside prepared assets"))?;
    if texture_asset.image.is_none() {
        return Ok(Some(SkipReason::TextureUnavailable(texture)));
    }
    Ok(None)
}
fn mask_geometry(
    scene: &PreparedScene,
    mask: NodeId,
    limits: Limits,
    visits: &mut usize,
    parts: &mut usize,
) -> io::Result<Result<MaskGeometry, MaskRejection>> {
    let root = &scene.nodes[mask].instance;
    let mut ancestor = Some(mask);
    while let Some(id) = ancestor {
        if has_effect(&scene.nodes[id].instance) {
            return Ok(Err(MaskRejection::EffectOnNodeOrAncestor(id)));
        }
        ancestor = scene.nodes[id].instance.parent;
    }
    let mut stack = vec![mask];
    let mut geometry = MaskGeometry { parts: Vec::new() };
    while let Some(id) = stack.pop() {
        *visits = visits
            .checked_add(1)
            .ok_or_else(|| invalid("mask visit count overflow"))?;
        if *visits > limits.max_mask_visits {
            return Err(invalid("preview mask traversal budget exceeded"));
        }
        let node = &scene.nodes[id];
        let instance = &node.instance;
        if !instance.effective_visible {
            return Ok(Err(MaskRejection::HiddenNode(id)));
        }
        if has_effect(instance) {
            return Ok(Err(MaskRejection::EffectOnNodeOrAncestor(id)));
        }
        if !instance.unsupported.is_empty() {
            return Ok(Err(MaskRejection::UnsupportedNodeMetadata(id)));
        }
        if (id != mask && instance.is_mask) || instance.masks != root.masks {
            return Ok(Err(MaskRejection::NestedMaskInsideMask(id)));
        }
        match node.content {
            NodeContent::Group => {}
            NodeContent::Shape { mesh } => {
                let asset = scene
                    .meshes
                    .get(mesh)
                    .ok_or_else(|| invalid("mask mesh index outside prepared assets"))?;
                let Some(mesh_asset) = &asset.mesh else {
                    return Ok(Err(MaskRejection::MeshUnavailable(id)));
                };
                for (fill, mesh_fill) in mesh_asset.fills.iter().enumerate() {
                    let opaque = match mesh_fill.paint.fill {
                        Fill::Solid(color) => {
                            let alpha = f64::from(color.alpha) * instance.world_color.multiply[3]
                                + instance.world_color.add[3];
                            alpha.is_finite() && alpha >= 255.
                        }
                        _ => false,
                    };
                    if !opaque {
                        return Ok(Err(MaskRejection::PaintNotOpaqueSolid { node: id, fill }));
                    }
                    *parts = parts
                        .checked_add(1)
                        .ok_or_else(|| invalid("mask part count overflow"))?;
                    if *parts > limits.max_mask_parts {
                        return Err(invalid("preview mask part budget exceeded"));
                    }
                    geometry.parts.push((id, mesh, fill));
                }
            }
            _ => return Ok(Err(MaskRejection::UnsupportedContent(id))),
        }
        stack.extend(instance.children.iter().rev().copied());
    }
    Ok(Ok(geometry))
}
fn push_command<'a>(
    commands: &mut Vec<Command<'a>>,
    command: Command<'a>,
    limits: Limits,
) -> io::Result<()> {
    if commands.len() >= limits.max_commands {
        return Err(invalid("preview GPU command budget exceeded"));
    }
    commands.push(command);
    Ok(())
}
fn srgb_decode(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
