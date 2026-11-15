//! Original image-quad renderer. This is infrastructure, not a recovered UI scene.
//!
//! Pixel coordinates start at the top-left. Transfer encoding, filtering and
//! compositing are explicit; matching retail color behavior remains unverified.

use std::{
    borrow::Cow,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::Duration,
};
use wgpu::util::DeviceExt;

#[path = "render/mesh.rs"]
pub mod mesh;

const MAX_IMAGE_BYTES: usize = 256 * 1024 * 1024;
const MAX_QUADS: usize = 65_536;
const VERTEX_BYTES: u64 = 32;
static NEXT_RENDERER: AtomicU64 = AtomicU64::new(1);

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
fn graphics(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageEncoding {
    Linear,
    Srgb,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFilter {
    Nearest,
    Linear,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlendMode {
    /// Straight-alpha source over the accumulated destination in shader space.
    Alpha,
    /// Replace all four destination channels; useful for exact asset inspection.
    Replace,
}

/// Tightly packed, top-to-bottom RGBA8 pixels. Inputs are not premultiplied.
pub struct RgbaImage<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

pub struct GpuImage {
    owner: u64,
    width: u32,
    height: u32,
    _texture: wgpu::Texture,
    binding: wgpu::BindGroup,
}
impl GpuImage {
    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Four corresponding corners/UVs, ordered top-left, top-right, bottom-right,
/// bottom-left. Arbitrary affine placement is supported; concave quads are not
/// assigned special semantics. UVs are clamped at the texture edges.
pub struct Quad<'a> {
    pub image: &'a GpuImage,
    pub corners: [[f32; 2]; 4],
    pub uv: [[f32; 2]; 4],
    /// Multiplies sampled linear RGB and straight alpha, independently.
    pub tint: [f32; 4],
    pub blend: BlendMode,
}
impl<'a> Quad<'a> {
    pub fn rectangle(image: &'a GpuImage, position: [f32; 2], size: [f32; 2]) -> Self {
        let [x, y] = position;
        let [w, h] = size;
        Self {
            image,
            corners: [[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
            uv: [[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
            tint: [1.; 4],
            blend: BlendMode::Alpha,
        }
    }
}

/// Raw composited target channels in RGBA order, without GPU row padding.
///
/// `encoding` describes RGB transfer encoding. Alpha-over-transparent output
/// contains premultiplied RGB; replacement draws retain the source's straight
/// alpha. This API deliberately does not guess or undo the scene's alpha state.
#[derive(Debug)]
pub struct Readback {
    pub width: u32,
    pub height: u32,
    pub encoding: ImageEncoding,
    pub pixels: Vec<u8>,
}

pub struct QuadRenderer {
    owner: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    nearest: wgpu::Sampler,
    linear: wgpu::Sampler,
    alpha: wgpu::RenderPipeline,
    replace: wgpu::RenderPipeline,
    adapter_info: Option<wgpu::AdapterInfo>,
}

impl QuadRenderer {
    /// Create a Vulkan renderer without a window or display connection.
    /// `force_fallback` requests a software adapter and fails if none is found.
    pub async fn headless(format: wgpu::TextureFormat, force_fallback: bool) -> io::Result<Self> {
        validate_format(format)?;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: force_fallback,
                compatible_surface: None,
            })
            .await
            .map_err(graphics)?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Marvel image renderer"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(graphics)?;
        let mut renderer = Self::from_device(device, queue, format).await?;
        renderer.adapter_info = Some(info);
        Ok(renderer)
    }

    /// Reuse an application's device/queue. The caller is responsible for
    /// selecting an adapter compatible with its native surface.
    pub async fn from_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> io::Result<Self> {
        validate_format(format)?;
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("RGBA image binding"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RGBA quad layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Original image quad shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let alpha = pipeline(
            &device,
            &pipeline_layout,
            &shader,
            format,
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );
        let replace = pipeline(&device, &pipeline_layout, &shader, format, None);
        let nearest = sampler(&device, wgpu::FilterMode::Nearest);
        let linear = sampler(&device, wgpu::FilterMode::Linear);
        if let Some(error) = device.pop_error_scope().await {
            return Err(graphics(error));
        }
        Ok(Self {
            owner: NEXT_RENDERER.fetch_add(1, Ordering::Relaxed),
            device,
            queue,
            format,
            layout,
            nearest,
            linear,
            alpha,
            replace,
            adapter_info: None,
        })
    }

    pub fn adapter_info(&self) -> Option<&wgpu::AdapterInfo> {
        self.adapter_info.as_ref()
    }
    pub const fn target_format(&self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn upload_rgba(
        &self,
        image: RgbaImage<'_>,
        encoding: ImageEncoding,
        filter: ImageFilter,
    ) -> io::Result<GpuImage> {
        let required = self.image_size(image.width, image.height)?;
        if image.pixels.len() != required {
            return Err(invalid("RGBA byte count does not match dimensions"));
        }
        let format = match encoding {
            ImageEncoding::Linear => wgpu::TextureFormat::Rgba8Unorm,
            ImageEncoding::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Decoded original image"),
            size: extent(image.width, image.height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * 4),
                rows_per_image: Some(image.height),
            },
            extent(image.width, image.height),
        );
        let view = texture.create_view(&Default::default());
        let binding = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image and sampler"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(match filter {
                        ImageFilter::Nearest => &self.nearest,
                        ImageFilter::Linear => &self.linear,
                    }),
                },
            ],
        });
        Ok(GpuImage {
            owner: self.owner,
            width: image.width,
            height: image.height,
            _texture: texture,
            binding,
        })
    }

    /// Render in slice order to a caller-owned 2D single-sample target.
    /// The view must use this renderer's device and `target_format`, and must
    /// have the supplied width/height and RENDER_ATTACHMENT usage.
    pub fn render_to_view(
        &self,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        clear: [f64; 4],
        quads: &[Quad<'_>],
    ) -> io::Result<wgpu::SubmissionIndex> {
        let vertices = self.prepare(width, height, clear, quads)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Image scene commands"),
            });
        self.encode(&mut encoder, view, clear, quads, &vertices);
        Ok(self.queue.submit([encoder.finish()]))
    }

    /// Render and synchronously read back an RGBA8 target. The native GPU wait
    /// is bounded to 30 seconds. No window, filesystem, or network is accessed.
    pub fn offscreen_rgba(
        &self,
        width: u32,
        height: u32,
        clear: [f64; 4],
        quads: &[Quad<'_>],
    ) -> io::Result<Readback> {
        let vertices = self.prepare(width, height, clear, quads)?;
        self.readback_scene(width, height, |encoder, view| {
            self.encode(encoder, view, clear, quads, &vertices);
        })
    }

    /// Shared bounded readback for child rendering pipelines on this device.
    fn readback_scene(
        &self,
        width: u32,
        height: u32,
        encode: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView),
    ) -> io::Result<Readback> {
        self.image_size(width, height)?;
        let row_bytes = width * 4;
        let padded_row = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer_size = u64::from(padded_row) * u64::from(height);
        if buffer_size > self.device.limits().max_buffer_size {
            return Err(invalid("readback buffer exceeds device limit"));
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Private offscreen image target"),
            size: extent(width, height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen image commands"),
            });
        encode(&mut encoder, &view);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            extent(width, height),
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (sender, receiver) = mpsc::sync_channel(1);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(graphics)?;
        receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(graphics)?
            .map_err(graphics)?;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(self.image_size(width, height)?)
            .map_err(graphics)?;
        for row in mapped.chunks_exact(padded_row as usize) {
            pixels.extend_from_slice(&row[..row_bytes as usize]);
        }
        drop(mapped);
        buffer.unmap();
        if matches!(
            self.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        ) {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }
        Ok(Readback {
            width,
            height,
            encoding: if self.format.is_srgb() {
                ImageEncoding::Srgb
            } else {
                ImageEncoding::Linear
            },
            pixels,
        })
    }

    fn image_size(&self, width: u32, height: u32) -> io::Result<usize> {
        let size = checked_image_size(width, height)?;
        let limit = self.device.limits().max_texture_dimension_2d;
        if width > limit || height > limit {
            return Err(invalid("image dimensions exceed device limit"));
        }
        Ok(size)
    }

    fn prepare(
        &self,
        width: u32,
        height: u32,
        clear: [f64; 4],
        quads: &[Quad<'_>],
    ) -> io::Result<Vec<u8>> {
        self.image_size(width, height)?;
        if clear
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(invalid("clear color must be finite normalized RGBA"));
        }
        if quads.len() > MAX_QUADS {
            return Err(invalid("quad count exceeds renderer limit"));
        }
        let vertex_bytes = quads.len() * 6 * VERTEX_BYTES as usize;
        if vertex_bytes as u64 > self.device.limits().max_buffer_size {
            return Err(invalid("quad vertex data exceeds device buffer limit"));
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(vertex_bytes).map_err(graphics)?;
        for quad in quads {
            if quad.image.owner != self.owner {
                return Err(invalid("image belongs to another renderer"));
            }
            if quad
                .corners
                .iter()
                .flatten()
                .chain(quad.uv.iter().flatten())
                .chain(quad.tint.iter())
                .any(|v| !v.is_finite())
            {
                return Err(invalid("quad data must be finite"));
            }
            if quad.tint[3] < 0. || quad.tint[3] > 1. {
                return Err(invalid("quad alpha tint must lie in [0,1]"));
            }
            for index in [0, 1, 2, 0, 2, 3] {
                let [x, y] = quad.corners[index];
                let [u, v] = quad.uv[index];
                let position = [x / width as f32 * 2. - 1., 1. - y / height as f32 * 2.];
                if position.iter().any(|v| !v.is_finite()) {
                    return Err(invalid("quad coordinates overflow normalized viewport"));
                }
                for value in position.into_iter().chain([u, v]).chain(quad.tint) {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        Ok(bytes)
    }

    fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear: [f64; 4],
        quads: &[Quad<'_>],
        vertices: &[u8],
    ) {
        let vertex_buffer = (!vertices.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Pixel-space image quads"),
                    contents: vertices,
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Original image scene"),
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
            ..Default::default()
        });
        if let Some(buffer) = vertex_buffer.as_ref() {
            pass.set_vertex_buffer(0, buffer.slice(..));
            for (index, quad) in quads.iter().enumerate() {
                pass.set_pipeline(match quad.blend {
                    BlendMode::Alpha => &self.alpha,
                    BlendMode::Replace => &self.replace,
                });
                pass.set_bind_group(0, &quad.image.binding, &[]);
                let start = index as u32 * 6;
                pass.draw(start..start + 6, 0..1);
            }
        }
    }
}

fn checked_image_size(width: u32, height: u32) -> io::Result<usize> {
    if width == 0 || height == 0 {
        return Err(invalid("image dimensions must be nonzero"));
    }
    let size = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| invalid("image byte size overflows"))?;
    if size > MAX_IMAGE_BYTES as u64 {
        return Err(invalid("image exceeds 256 MiB byte budget"));
    }
    usize::try_from(size).map_err(graphics)
}
fn validate_format(format: wgpu::TextureFormat) -> io::Result<()> {
    if !matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        return Err(invalid(
            "only RGBA8/BGRA8 linear or sRGB targets are supported",
        ));
    }
    Ok(())
}
fn extent(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}
fn sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Clamped image sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Image quad pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: VERTEX_BYTES,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}
const SHADER: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};
@vertex
fn vertex_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) tint: vec4<f32>) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    out.tint = tint;
    return out;
}
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
@fragment
fn fragment_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(image, image_sampler, in.uv) * in.tint;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_sizes_and_formats_without_a_device() {
        assert!(checked_image_size(0, 1).is_err());
        assert!(checked_image_size(u32::MAX, u32::MAX).is_err());
        assert_eq!(checked_image_size(3, 2).unwrap(), 24);
        assert!(validate_format(wgpu::TextureFormat::Depth32Float).is_err());
        assert!(validate_format(wgpu::TextureFormat::Bgra8UnormSrgb).is_ok());
    }
}
