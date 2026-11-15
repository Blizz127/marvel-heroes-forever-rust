//! Offscreen composition of a prepared structural scene. This is a diagnostic
//! of saved geometry, not script execution or a completed frontend.

use super::scene_content::{self, NodeContent, PreparationIssue, PreparedScene};
use crate::{
    gfx::{
        Rect,
        scene::{NodeId, SceneNode},
        shapes::Fill,
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
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_commands: 65_536,
            max_mask_parts: 65_536,
            max_mask_visits: 2_000_000,
            max_uploaded_bytes: 256 * 1024 * 1024,
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
    pub text_instances_omitted: usize,
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

struct PlannedDraw {
    node: NodeId,
    mesh: usize,
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
            NodeContent::Group => {}
            NodeContent::Text { .. } => {
                coverage.text_instances_omitted += 1;
                skipped_node(
                    id,
                    SkipReason::TextNotPrepared,
                    &mut coverage,
                    &mut limitations,
                );
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
                coverage.shape_instances += 1;
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
                    skipped_node(
                        id,
                        SkipReason::UnsupportedMask(mask),
                        &mut coverage,
                        &mut limitations,
                    );
                    continue;
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
                        mesh,
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
    let mut commands = Vec::new();
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
        let shape_fill = &scene.meshes[draw.mesh].mesh.as_ref().unwrap().fills[draw.fill];
        push_command(
            &mut commands,
            Command::Draw(Draw {
                mesh: shape_fill,
                transform: world_transform(instance, stage_to_pixels)?,
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
        coverage.fill_draws += 1;
        coverage.triangles += shape_fill.indices.len() / 3;
        coverage.nodes[draw.node].submitted_fills += 1;
        coverage.nodes[draw.node].submitted_triangles += shape_fill.indices.len() / 3;
    }
    for _ in &active_masks {
        push_command(&mut commands, Command::PopMask, limits)?;
    }
    coverage.mask_sources_used = used_masks.len();
    coverage.shape_instances_with_submitted_fills = coverage
        .nodes
        .iter()
        .filter(|n| n.submitted_fills > 0)
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
