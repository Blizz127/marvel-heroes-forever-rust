//! Triangle fills from recovered SWF geometry, with explicit numerical policies.
//! This module owns no display list, scripting state, texture resolver or movie.

use super::{
    BlendMode, GpuImage, ImageEncoding, QuadRenderer, Readback, extent, graphics, invalid,
};
use crate::gfx::{
    self,
    shapes::{Fill, GradientKind},
    tessellate::{FillMesh, Position, fill_coordinates},
};
use std::{borrow::Cow, io, ops::Range};
use wgpu::util::DeviceExt;

const VERTEX_BYTES: usize = 16;
const MAX_VERTEX_BYTES: usize = 256 * 1024 * 1024;
const MAX_COMMANDS: usize = 65_536;
const MAX_STOPS: usize = 16;
const PARAM_BYTES: usize = 96 + MAX_STOPS * 32;
const STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

/// x' = a*x + c*y + tx; y' = b*x + d*y + ty.
/// Draw inputs are shape twips; the result must be physical target pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine(pub [f64; 6]);
impl Affine {
    pub const IDENTITY: Self = Self([1., 0., 0., 1., 0., 0.]);
    pub const TWIPS_TO_PIXELS: Self = Self([0.05, 0., 0., 0.05, 0., 0.]);
    /// The SWF matrix alone maps twips to twips, not to target pixels.
    pub fn from_swf(matrix: gfx::Matrix) -> Self {
        Self([
            f64::from(matrix.scale_x) / 65536.,
            f64::from(matrix.rotate_skew_0) / 65536.,
            f64::from(matrix.rotate_skew_1) / 65536.,
            f64::from(matrix.scale_y) / 65536.,
            f64::from(matrix.translate_x),
            f64::from(matrix.translate_y),
        ])
    }
    /// Apply self first, then outer. No rounding or pixel snapping is applied.
    pub fn then(self, outer: Self) -> Self {
        let [a, b, c, d, x, y] = self.0;
        let [e, f, g, h, u, v] = outer.0;
        Self([
            e * a + g * b,
            f * a + h * b,
            e * c + g * d,
            f * c + h * d,
            e * x + g * y + u,
            f * x + h * y + v,
        ])
    }
    fn apply(self, p: Position) -> [f64; 2] {
        let [a, b, c, d, x, y] = self.0;
        [a * p.x + c * p.y + x, b * p.x + d * p.y + y]
    }
}

/// Normalized-channel transform: clamp(source * multiply + add, 0, 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTransform {
    pub multiply: [f32; 4],
    pub add: [f32; 4],
}
impl ColorTransform {
    pub const IDENTITY: Self = Self {
        multiply: [1.; 4],
        add: [0.; 4],
    };
    pub fn from_swf(value: gfx::ColorTransform) -> Self {
        Self {
            multiply: value.multiply.map(|v| v as f32 / 256.),
            add: value.add.map(|v| v as f32 / 255.),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformSpace {
    /// Apply channel arithmetic directly to the sampled shader values.
    Shader,
    /// Encode shader RGB as sRGB, transform, then decode. Alpha remains linear.
    EncodedSrgb,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinearRgbTransfer {
    /// Piecewise sRGB transfer for the SWF linear-RGB interpolation mode.
    Srgb,
    /// Power 2.2 transfer for the SWF linear-RGB interpolation mode.
    Power2_2,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientAlpha {
    Straight,
    /// Premultiply in the chosen interpolation space, interpolate, unpremultiply.
    Premultiplied,
}

#[derive(Clone, Copy)]
pub struct BitmapBinding<'a> {
    pub character_id: u16,
    pub image: &'a GpuImage,
    /// Serialized image dimensions, independent of decoded texture dimensions.
    pub logical_size: [u32; 2],
}
/// Select numerical behavior explicitly; none of these choices establishes
/// the retail Scaleform/UE3 color contract. The mesh retains its original paint.
#[derive(Clone, Copy)]
pub struct Paint<'a> {
    pub bitmap: Option<BitmapBinding<'a>>,
    /// Solid/gradient encoded channel interpretation. Bitmap encoding is chosen
    /// when the image is uploaded, and is not changed by this field.
    pub colors: ImageEncoding,
    pub transform_space: TransformSpace,
    pub linear_rgb_transfer: LinearRgbTransfer,
    pub gradient_alpha: GradientAlpha,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scissor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Draw<'a> {
    pub mesh: &'a FillMesh,
    pub transform: Affine,
    pub color_transform: ColorTransform,
    pub paint: Paint<'a>,
    pub blend: BlendMode,
    pub scissor: Option<Scissor>,
}
pub struct MaskPart<'a> {
    pub mesh: &'a FillMesh,
    pub transform: Affine,
}
/// Masks are binary geometry coverage, combined as union within a push and
/// intersection across nesting. Paint alpha, cached alpha masks and filters are
/// outside this contract. The caller must reject those unsupported scene modes.
pub enum Command<'a> {
    Draw(Draw<'a>),
    PushMask {
        parts: &'a [MaskPart<'a>],
        scissor: Option<Scissor>,
    },
    PopMask,
}

pub struct MeshRenderer<'a> {
    source: &'a QuadRenderer,
    params_layout: wgpu::BindGroupLayout,
    alpha: wgpu::RenderPipeline,
    replace: wgpu::RenderPipeline,
    mask_push: wgpu::RenderPipeline,
    mask_pop: wgpu::RenderPipeline,
    samplers: [wgpu::Sampler; 4],
    white: GpuImage,
    mask_params: wgpu::BindGroup,
}

struct PreparedPaint {
    image: wgpu::BindGroup,
    params: wgpu::BindGroup,
}
#[derive(Clone, Copy)]
enum Operation {
    Color(BlendMode),
    Push,
    Pop,
}
struct PreparedDraw {
    range: Range<u32>,
    scissor: Scissor,
    stencil_reference: u32,
    operation: Operation,
    paint: Option<PreparedPaint>,
}
struct Prepared {
    vertices: Vec<u8>,
    draws: Vec<PreparedDraw>,
}
struct SavedMask {
    ranges: Vec<Range<u32>>,
    scissor: Scissor,
}

impl<'a> MeshRenderer<'a> {
    pub async fn new(source: &'a QuadRenderer) -> io::Result<Self> {
        let device = &source.device;
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Recovered mesh fill parameters"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(PARAM_BYTES as u64),
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Recovered mesh layout"),
            bind_group_layouts: &[&source.layout, &params_layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Original recovered-fill shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let alpha = pipeline(
            device,
            &layout,
            &shader,
            source.format,
            Operation::Color(BlendMode::Alpha),
        );
        let replace = pipeline(
            device,
            &layout,
            &shader,
            source.format,
            Operation::Color(BlendMode::Replace),
        );
        let mask_push = pipeline(device, &layout, &shader, source.format, Operation::Push);
        let mask_pop = pipeline(device, &layout, &shader, source.format, Operation::Pop);
        let samplers = [
            fill_sampler(device, false, false),
            fill_sampler(device, false, true),
            fill_sampler(device, true, false),
            fill_sampler(device, true, true),
        ];
        let white = source.upload_rgba(
            super::RgbaImage {
                width: 1,
                height: 1,
                pixels: &[255; 4],
            },
            ImageEncoding::Linear,
            super::ImageFilter::Nearest,
        )?;
        let mask_params = parameter_binding(device, &params_layout, &[0; PARAM_BYTES]);
        if let Some(error) = device.pop_error_scope().await {
            return Err(graphics(error));
        }
        Ok(Self {
            source,
            params_layout,
            alpha,
            replace,
            mask_push,
            mask_pop,
            samplers,
            white,
            mask_params,
        })
    }

    pub fn render_to_view(
        &self,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        clear: [f64; 4],
        commands: &[Command<'_>],
    ) -> io::Result<wgpu::SubmissionIndex> {
        let prepared = self.prepare(width, height, clear, commands)?;
        let mut encoder =
            self.source
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Recovered triangle commands"),
                });
        self.encode(&mut encoder, view, width, height, clear, &prepared);
        Ok(self.source.queue.submit([encoder.finish()]))
    }

    pub fn offscreen_rgba(
        &self,
        width: u32,
        height: u32,
        clear: [f64; 4],
        commands: &[Command<'_>],
    ) -> io::Result<Readback> {
        let prepared = self.prepare(width, height, clear, commands)?;
        self.source.readback_scene(width, height, |encoder, view| {
            self.encode(encoder, view, width, height, clear, &prepared);
        })
    }

    fn prepare(
        &self,
        width: u32,
        height: u32,
        clear: [f64; 4],
        commands: &[Command<'_>],
    ) -> io::Result<Prepared> {
        self.source.image_size(width, height)?;
        if clear
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(invalid("clear color must be finite normalized RGBA"));
        }
        if commands.len() > MAX_COMMANDS {
            return Err(invalid("mesh command budget exceeded"));
        }
        let mut prepared = Prepared {
            vertices: Vec::new(),
            draws: Vec::new(),
        };
        let mut masks: Vec<SavedMask> = Vec::new();
        let mut parts = 0usize;
        for command in commands {
            match command {
                Command::Draw(draw) => {
                    parts += 1;
                    if parts > MAX_COMMANDS {
                        return Err(invalid("mesh draw budget exceeded"));
                    }
                    let scissor = check_scissor(draw.scissor, width, height)?;
                    let paint = self.paint(draw)?;
                    let range = self.vertices(
                        &mut prepared.vertices,
                        draw.mesh,
                        draw.transform,
                        Some(draw.paint),
                        width,
                        height,
                    )?;
                    prepared.draws.push(PreparedDraw {
                        range,
                        scissor,
                        stencil_reference: masks.len() as u32,
                        operation: Operation::Color(draw.blend),
                        paint: Some(paint),
                    });
                }
                Command::PushMask {
                    parts: mask_parts,
                    scissor,
                } => {
                    if masks.len() == 255 {
                        return Err(invalid("binary mask depth exceeds stencil capacity"));
                    }
                    parts = parts
                        .checked_add(mask_parts.len())
                        .ok_or_else(|| invalid("mask part overflow"))?;
                    if parts > MAX_COMMANDS {
                        return Err(invalid("mesh/mask part budget exceeded"));
                    }
                    let scissor = check_scissor(*scissor, width, height)?;
                    let mut ranges = Vec::new();
                    for part in *mask_parts {
                        let range = self.vertices(
                            &mut prepared.vertices,
                            part.mesh,
                            part.transform,
                            None,
                            width,
                            height,
                        )?;
                        ranges.push(range.clone());
                        prepared.draws.push(PreparedDraw {
                            range,
                            scissor,
                            stencil_reference: masks.len() as u32,
                            operation: Operation::Push,
                            paint: None,
                        });
                    }
                    masks.push(SavedMask { ranges, scissor });
                }
                Command::PopMask => {
                    let depth = masks.len() as u32;
                    let mask = masks
                        .pop()
                        .ok_or_else(|| invalid("binary mask stack underflow"))?;
                    for range in mask.ranges {
                        prepared.draws.push(PreparedDraw {
                            range,
                            scissor: mask.scissor,
                            stencil_reference: depth,
                            operation: Operation::Pop,
                            paint: None,
                        });
                    }
                }
            }
        }
        if !masks.is_empty() {
            return Err(invalid("unclosed binary mask stack"));
        }
        Ok(prepared)
    }

    fn vertices(
        &self,
        output: &mut Vec<u8>,
        mesh: &FillMesh,
        transform: Affine,
        paint: Option<Paint<'_>>,
        width: u32,
        height: u32,
    ) -> io::Result<Range<u32>> {
        if !mesh.indices.len().is_multiple_of(3) {
            return Err(invalid("mesh indices must contain complete triangles"));
        }
        if transform.0.iter().any(|v| !v.is_finite()) {
            return Err(invalid("mesh transform must be finite"));
        }
        let bytes = mesh
            .indices
            .len()
            .checked_mul(VERTEX_BYTES)
            .and_then(|n| output.len().checked_add(n))
            .ok_or_else(|| invalid("mesh vertex bytes overflow"))?;
        if bytes > MAX_VERTEX_BYTES || bytes as u64 > self.source.device.limits().max_buffer_size {
            return Err(invalid("mesh vertex buffer exceeds budget/device limit"));
        }
        output
            .try_reserve_exact(bytes - output.len())
            .map_err(graphics)?;
        let start = (output.len() / VERTEX_BYTES) as u32;
        for &index in &mesh.indices {
            let point = *mesh
                .positions
                .get(index as usize)
                .ok_or_else(|| invalid("mesh index outside positions"))?;
            if !point.x.is_finite() || !point.y.is_finite() {
                return Err(invalid("mesh position must be finite"));
            }
            let [x, y] = transform.apply(point);
            let mut coord = [0., 0.];
            if let Some(paint) = paint {
                if let Some(local) = fill_coordinates(&mesh.paint.fill, point)? {
                    coord = match &mesh.paint.fill {
                        Fill::Bitmap { .. } => {
                            let bitmap = paint
                                .bitmap
                                .ok_or_else(|| invalid("bitmap mesh lacks binding"))?;
                            [
                                local.x / f64::from(bitmap.logical_size[0]),
                                local.y / f64::from(bitmap.logical_size[1]),
                            ]
                        }
                        Fill::Gradient { .. } => [local.x / 16384., local.y / 16384.],
                        Fill::Solid(_) => unreachable!(),
                    };
                }
            }
            let values = [
                x / f64::from(width) * 2. - 1.,
                1. - y / f64::from(height) * 2.,
                coord[0],
                coord[1],
            ];
            if coord.iter().any(|v| v.abs() > 1.0e15) {
                return Err(invalid(
                    "fill coordinates exceed safe shader arithmetic range",
                ));
            }
            for value in values {
                let value = value as f32;
                if !value.is_finite() {
                    return Err(invalid("mesh vertex overflows finite shader coordinates"));
                }
                output.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(start..(output.len() / VERTEX_BYTES) as u32)
    }

    fn paint(&self, draw: &Draw<'_>) -> io::Result<PreparedPaint> {
        if draw
            .color_transform
            .multiply
            .iter()
            .chain(&draw.color_transform.add)
            .any(|v| !v.is_finite())
        {
            return Err(invalid("mesh color transform must be finite"));
        }
        let mut meta = [0u32; 4];
        let options = [
            u32::from(draw.paint.colors == ImageEncoding::Srgb),
            u32::from(draw.paint.transform_space == TransformSpace::EncodedSrgb),
            u32::from(draw.paint.gradient_alpha == GradientAlpha::Premultiplied),
            u32::from(draw.paint.linear_rgb_transfer == LinearRgbTransfer::Power2_2),
        ];
        let mut solid = [0.; 4];
        let mut extra = [0.; 4];
        let mut stops = Vec::<([f32; 4], f32)>::new();
        let mut texture = &self.white;
        let mut sampler = &self.samplers[0];
        match &draw.mesh.paint.fill {
            Fill::Solid(color) => {
                solid = rgba(*color);
            }
            Fill::Bitmap {
                character_id,
                repeating,
                smoothed,
                ..
            } => {
                meta[0] = 1;
                let bitmap = draw
                    .paint
                    .bitmap
                    .ok_or_else(|| invalid("bitmap fill has no resolved image binding"))?;
                if bitmap.character_id != *character_id {
                    return Err(invalid("bitmap character ID does not match recovered fill"));
                }
                if bitmap.image.owner != self.source.owner {
                    return Err(invalid("bitmap image belongs to another renderer"));
                }
                if bitmap.logical_size.contains(&0) {
                    return Err(invalid("bitmap logical dimensions must be nonzero"));
                }
                texture = bitmap.image;
                sampler = &self.samplers[usize::from(*repeating) * 2 + usize::from(*smoothed)];
            }
            Fill::Gradient {
                kind,
                spread,
                interpolation,
                stops: source_stops,
                focal_point,
                ..
            } => {
                meta[0] = match kind {
                    GradientKind::Linear => 2,
                    GradientKind::Radial => 3,
                    GradientKind::FocalRadial => 4,
                };
                if *spread > 2 || *interpolation > 1 {
                    return Err(unsupported("reserved gradient spread/interpolation"));
                }
                meta[1] = u32::from(*spread);
                meta[2] = u32::from(*interpolation);
                if source_stops.is_empty() || source_stops.len() > MAX_STOPS {
                    return Err(invalid("gradient requires 1..=16 stops"));
                }
                if source_stops.windows(2).any(|s| s[0].ratio > s[1].ratio) {
                    return Err(invalid("gradient ratios must be ordered"));
                }
                meta[3] = source_stops.len() as u32;
                for stop in source_stops {
                    stops.push((rgba(stop.color), f32::from(stop.ratio) / 255.));
                }
                if *kind == GradientKind::FocalRadial {
                    let focal = f32::from(
                        focal_point.ok_or_else(|| invalid("focal gradient lacks focal value"))?,
                    ) / 256.;
                    if focal.abs() >= 1. {
                        return Err(unsupported(
                            "focal radial endpoints/out-of-range focal values are not implemented",
                        ));
                    }
                    extra[0] = focal;
                } else if focal_point.is_some() {
                    return Err(invalid("non-focal gradient contains focal value"));
                }
            }
        }
        if !matches!(draw.mesh.paint.fill, Fill::Bitmap { .. }) && draw.paint.bitmap.is_some() {
            return Err(invalid("non-bitmap fill received a bitmap binding"));
        }
        let mut bytes = Vec::with_capacity(PARAM_BYTES);
        for value in meta.into_iter().chain(options) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in solid
            .into_iter()
            .chain(draw.color_transform.multiply)
            .chain(draw.color_transform.add)
            .chain(extra)
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for i in 0..MAX_STOPS {
            let (color, ratio) = stops.get(i).copied().unwrap_or(([0.; 4], 0.));
            for value in color.into_iter().chain([ratio, 0., 0., 0.]) {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        debug_assert_eq!(bytes.len(), PARAM_BYTES);
        let view = texture._texture.create_view(&Default::default());
        let image = self
            .source
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Recovered fill image/sampler"),
                layout: &self.source.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });
        let params = parameter_binding(&self.source.device, &self.params_layout, &bytes);
        Ok(PreparedPaint { image, params })
    }

    fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        clear: [f64; 4],
        prepared: &Prepared,
    ) {
        let stencil = self.source.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Binary vector-mask stencil"),
            size: extent(width, height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: STENCIL_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let stencil_view = stencil.create_view(&Default::default());
        let vertices = (!prepared.vertices.is_empty()).then(|| {
            self.source
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Actual recovered triangle vertices"),
                    contents: &prepared.vertices,
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Recovered mesh fills and binary masks"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear[0],
                        g: clear[1],
                        b: clear[2],
                        a: clear[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &stencil_view,
                depth_ops: None,
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Discard,
                }),
            }),
            ..Default::default()
        });
        if let Some(vertices) = &vertices {
            pass.set_vertex_buffer(0, vertices.slice(..));
            for draw in &prepared.draws {
                if draw.range.is_empty() || draw.scissor.width == 0 || draw.scissor.height == 0 {
                    continue;
                }
                pass.set_pipeline(match draw.operation {
                    Operation::Color(BlendMode::Alpha) => &self.alpha,
                    Operation::Color(BlendMode::Replace) => &self.replace,
                    Operation::Push => &self.mask_push,
                    Operation::Pop => &self.mask_pop,
                });
                let (image, params) = draw
                    .paint
                    .as_ref()
                    .map_or((&self.white.binding, &self.mask_params), |p| {
                        (&p.image, &p.params)
                    });
                pass.set_bind_group(0, image, &[]);
                pass.set_bind_group(1, params, &[]);
                pass.set_stencil_reference(draw.stencil_reference);
                pass.set_scissor_rect(
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.width,
                    draw.scissor.height,
                );
                pass.draw(draw.range.clone(), 0..1);
            }
        }
    }
}

fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}
fn rgba(color: gfx::shapes::Color) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha].map(|c| f32::from(c) / 255.)
}
fn check_scissor(value: Option<Scissor>, width: u32, height: u32) -> io::Result<Scissor> {
    let rect = value.unwrap_or(Scissor {
        x: 0,
        y: 0,
        width,
        height,
    });
    if rect.x > width
        || rect.y > height
        || rect.width > width - rect.x
        || rect.height > height - rect.y
    {
        return Err(invalid("scissor lies outside physical target"));
    }
    Ok(rect)
}
fn fill_sampler(device: &wgpu::Device, repeat: bool, smooth: bool) -> wgpu::Sampler {
    let address = if repeat {
        wgpu::AddressMode::Repeat
    } else {
        wgpu::AddressMode::ClampToEdge
    };
    let filter = if smooth {
        wgpu::FilterMode::Linear
    } else {
        wgpu::FilterMode::Nearest
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Recovered fill addressing/filtering"),
        address_mode_u: address,
        address_mode_v: address,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
fn parameter_binding(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    bytes: &[u8],
) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Recovered fill parameters"),
        contents: bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Recovered fill parameter binding"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}
fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    operation: Operation,
) -> wgpu::RenderPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    let mask = matches!(operation, Operation::Push | Operation::Pop);
    let face = wgpu::StencilFaceState {
        compare: wgpu::CompareFunction::Equal,
        fail_op: wgpu::StencilOperation::Keep,
        depth_fail_op: wgpu::StencilOperation::Keep,
        pass_op: match operation {
            Operation::Push => wgpu::StencilOperation::IncrementClamp,
            Operation::Pop => wgpu::StencilOperation::DecrementClamp,
            _ => wgpu::StencilOperation::Keep,
        },
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Recovered fill/mask triangle pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: VERTEX_BYTES as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: STENCIL_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: face,
                back: face,
                read_mask: 255,
                write_mask: if mask { 255 } else { 0 },
            },
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(if mask {
                "mask_fragment"
            } else {
                "fragment_main"
            }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: match operation {
                    Operation::Color(BlendMode::Alpha) => Some(wgpu::BlendState::ALPHA_BLENDING),
                    _ => None,
                },
                write_mask: if mask {
                    wgpu::ColorWrites::empty()
                } else {
                    wgpu::ColorWrites::ALL
                },
            })],
        }),
        multiview: None,
        cache: None,
    })
}

const SHADER: &str = r#"
struct Stop { color: vec4<f32>, position: vec4<f32> }
struct Params {
    kind_flags: vec4<u32>, options: vec4<u32>, solid: vec4<f32>, multiply: vec4<f32>, add: vec4<f32>, extra: vec4<f32>,
    stops: array<Stop, 16>,
}
@group(0) @binding(0) var source_image: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> params: Params;
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) coordinate: vec2<f32> }
@vertex fn vertex_main(@location(0) position: vec2<f32>, @location(1) coordinate: vec2<f32>) -> VertexOut {
    var out: VertexOut; out.position = vec4<f32>(position, 0.0, 1.0); out.coordinate = coordinate; return out;
}
fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(v / 12.92, pow((v + 0.055) / 1.055, vec3<f32>(2.4)), v > vec3<f32>(0.04045));
}
fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(v * 12.92, 1.055 * pow(v, vec3<f32>(1.0 / 2.4)) - 0.055, v > vec3<f32>(0.0031308));
}
fn interpolation_decode(c: vec3<f32>) -> vec3<f32> {
    if params.options.w == 1u { return pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(2.2)); }
    return srgb_decode(c);
}
fn interpolation_encode(c: vec3<f32>) -> vec3<f32> {
    if params.options.w == 1u { return pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2)); }
    return srgb_encode(c);
}
fn interpolate_colors(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    var left = a.rgb; var right = b.rgb;
    if params.kind_flags.z == 1u { left = interpolation_decode(left); right = interpolation_decode(right); }
    let alpha = mix(a.a, b.a, t);
    if params.options.z == 1u { left *= a.a; right *= b.a; }
    var rgb = mix(left, right, t);
    if params.options.z == 1u {
        if alpha > 0.0 { rgb /= alpha; } else { rgb = vec3<f32>(0.0); }
    }
    if params.kind_flags.z == 1u { rgb = interpolation_encode(rgb); }
    return vec4<f32>(rgb, alpha);
}
fn spread(value: f32) -> f32 {
    if params.kind_flags.y == 1u { return 1.0 - abs(1.0 - (value - floor(value / 2.0) * 2.0)); }
    if params.kind_flags.y == 2u { return value - floor(value); }
    return clamp(value, 0.0, 1.0);
}
fn gradient(value: f32) -> vec4<f32> {
    let t = spread(value);
    var left = params.stops[0];
    if t < left.position.x { return left.color; }
    for (var i = 1u; i < params.kind_flags.w; i += 1u) {
        let right = params.stops[i];
        if t < right.position.x {
            let fraction = clamp((t - left.position.x) / (right.position.x - left.position.x), 0.0, 1.0);
            return interpolate_colors(left.color, right.color, fraction);
        }
        left = right;
    }
    return left.color;
}
@fragment fn fragment_main(input: VertexOut) -> @location(0) vec4<f32> {
    var color = params.solid;
    if params.kind_flags.x == 1u {
        color = textureSample(source_image, source_sampler, input.coordinate);
    } else {
        if params.kind_flags.x == 2u { color = gradient((input.coordinate.x + 1.0) * 0.5); }
        if params.kind_flags.x == 3u { color = gradient(length(input.coordinate)); }
        if params.kind_flags.x == 4u {
            let focal = params.extra.x;
            let delta = input.coordinate - vec2<f32>(focal, 0.0);
            let q2 = dot(delta, delta);
            var position = 0.0;
            if q2 > 0.0 {
                let root = sqrt(focal * focal * delta.x * delta.x + (1.0 - focal * focal) * q2);
                position = q2 / (root - focal * delta.x);
            }
            color = gradient(position);
        }
        if params.options.x == 1u { color = vec4<f32>(srgb_decode(color.rgb), color.a); }
    }
    if params.options.y == 1u { color = vec4<f32>(srgb_encode(color.rgb), color.a); }
    color = clamp(color * params.multiply + params.add, vec4<f32>(0.0), vec4<f32>(1.0));
    if params.options.y == 1u { color = vec4<f32>(srgb_decode(color.rgb), color.a); }
    return color;
}
@fragment fn mask_fragment() -> @location(0) vec4<f32> { return vec4<f32>(0.0); }
"#;
