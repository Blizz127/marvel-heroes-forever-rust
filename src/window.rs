//! Native host for an actual decoded image. It does not implement a login UI.
//!
//! Call from the application's main thread. This creates only its own window,
//! uses the caller's display environment, and never chooses another display.

use crate::render::{
    BlendMode, GpuImage, ImageEncoding, ImageFilter, Quad, QuadRenderer, RgbaImage,
};
use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

#[path = "window/scene.rs"]
pub mod scene;
pub use scene::{SceneController, ScenePreviewOptions, ScenePreviewReport, run_scene_preview};

const MAX_IMAGE_BYTES: u64 = 256 * 1024 * 1024;
const RETRY_DELAY: Duration = Duration::from_millis(50);

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
fn platform(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

pub struct PreviewImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub encoding: ImageEncoding,
}

/// Settings are for an asset-inspection window, not recovered retail options.
pub struct PreviewOptions {
    pub title: String,
    pub initial_logical_size: [u32; 2],
    /// No fallback from this encoding to a different surface encoding occurs.
    pub output_encoding: ImageEncoding,
    pub filter: ImageFilter,
    pub background_rgb: [f64; 3],
    pub force_fallback_adapter: bool,
    /// Optional QA limits. Ordinary user-close behavior is the default.
    pub close_after: Option<Duration>,
    pub frame_limit: Option<u64>,
}
impl PreviewOptions {
    pub fn new(output_encoding: ImageEncoding) -> Self {
        Self {
            title: "Marvel Heroes Forever — asset preview".into(),
            initial_logical_size: [1280, 720],
            output_encoding,
            filter: ImageFilter::Linear,
            background_rgb: [0.; 3],
            force_fallback_adapter: false,
            close_after: None,
            frame_limit: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseReason {
    EventLoopStopped,
    WindowClosed,
    WindowDestroyed,
    Escape,
    TimeLimit,
    FrameLimit,
}

#[derive(Debug)]
pub struct PreviewReport {
    pub frames_presented: u64,
    pub completed_resumes: u64,
    pub suspensions: u64,
    pub adapter: Option<wgpu::AdapterInfo>,
    pub surface_format: Option<wgpu::TextureFormat>,
    pub close_reason: CloseReason,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Fit source pixels into physical target pixels without distorting aspect.
/// Fractional positioning is retained; no retail pixel-snapping policy is implied.
pub fn fit_image_rect(source: [u32; 2], target: [u32; 2]) -> Option<ImageRect> {
    if source.contains(&0) || target.contains(&0) {
        return None;
    }
    let scale = (f64::from(target[0]) / f64::from(source[0]))
        .min(f64::from(target[1]) / f64::from(source[1]));
    let width = f64::from(source[0]) * scale;
    let height = f64::from(source[1]) * scale;
    Some(ImageRect {
        x: ((f64::from(target[0]) - width) * 0.5) as f32,
        y: ((f64::from(target[1]) - height) * 0.5) as f32,
        width: width as f32,
        height: height as f32,
    })
}

/// Open a resizable native image preview and return when it closes.
///
/// The initial size is logical; all surface and fit calculations use the actual
/// physical window size. Platform errors exposed by winit/wgpu are returned;
/// their required native runtime libraries must already be available.
/// One event loop per process and a main-thread call are winit requirements.
pub fn run_image_preview(
    image: PreviewImage,
    options: PreviewOptions,
) -> io::Result<PreviewReport> {
    validate(&image, &options)?;
    let deadline = options
        .close_after
        .map(|delay| {
            Instant::now()
                .checked_add(delay)
                .ok_or_else(|| invalid("preview deadline overflows"))
        })
        .transpose()?;
    let event_loop = EventLoop::new().map_err(platform)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = PreviewApp {
        image,
        options,
        window: None,
        gpu: None,
        occluded: false,
        retry_at: None,
        deadline,
        failure: None,
        report: PreviewReport {
            frames_presented: 0,
            completed_resumes: 0,
            suspensions: 0,
            adapter: None,
            surface_format: None,
            close_reason: CloseReason::EventLoopStopped,
        },
    };
    event_loop.run_app(&mut app).map_err(platform)?;
    if let Some(error) = app.failure {
        Err(error)
    } else {
        Ok(app.report)
    }
}

fn validate(image: &PreviewImage, options: &PreviewOptions) -> io::Result<()> {
    let bytes = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| invalid("image dimensions overflow"))?;
    if image.width == 0
        || image.height == 0
        || bytes > MAX_IMAGE_BYTES
        || bytes != image.pixels.len() as u64
    {
        return Err(invalid(
            "image needs nonzero dimensions and matching RGBA bytes within 256 MiB",
        ));
    }
    if options
        .initial_logical_size
        .iter()
        .any(|&n| n == 0 || n > 16_384)
    {
        return Err(invalid(
            "initial window dimensions must lie in 1..=16384 logical pixels",
        ));
    }
    if options.title.is_empty()
        || options.title.len() > 1024
        || options.title.chars().any(char::is_control)
    {
        return Err(invalid(
            "window title must contain 1..=1024 bytes without control characters",
        ));
    }
    if options
        .background_rgb
        .iter()
        .any(|n| !n.is_finite() || !(0.0..=1.0).contains(n))
    {
        return Err(invalid("background RGB must be finite and normalized"));
    }
    if options.frame_limit == Some(0) || options.close_after == Some(Duration::ZERO) {
        return Err(invalid("QA frame/time limits must be positive"));
    }
    Ok(())
}

fn choose_format(
    formats: &[wgpu::TextureFormat],
    encoding: ImageEncoding,
) -> io::Result<wgpu::TextureFormat> {
    let candidates = match encoding {
        ImageEncoding::Linear => [
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ],
        ImageEncoding::Srgb => [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ],
    };
    candidates
        .into_iter()
        .find(|format| formats.contains(format))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "surface does not support the requested RGBA8/BGRA8 transfer encoding",
            )
        })
}

struct SurfaceState {
    // Drop the surface before other explicit window owners.
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
    renderer: QuadRenderer,
    image: GpuImage,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    info: wgpu::AdapterInfo,
    background: [f64; 4],
}

enum DrawOutcome {
    Presented { redraw: bool },
    Retry,
    ZeroSize,
}

impl SurfaceState {
    async fn new(
        window: Arc<Window>,
        image: &PreviewImage,
        options: &PreviewOptions,
    ) -> io::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).map_err(platform)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: options.force_fallback_adapter,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(platform)?;
        let caps = surface.get_capabilities(&adapter);
        let format = choose_format(&caps.formats, options.output_encoding)?;
        if !caps.usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || !caps.present_modes.contains(&wgpu::PresentMode::Fifo)
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "surface lacks render-attachment/FIFO support",
            ));
        }
        let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            *caps
                .alpha_modes
                .first()
                .ok_or_else(|| platform("surface has no alpha mode"))?
        };
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Marvel native image preview"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(platform)?;
        let renderer = QuadRenderer::from_device(device, queue, format).await?;
        let texture = renderer.upload_rgba(
            RgbaImage {
                width: image.width,
                height: image.height,
                pixels: &image.pixels,
            },
            image.encoding,
            options.filter,
        )?;
        let mut state = Self {
            surface,
            window,
            renderer,
            image: texture,
            config: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: 1,
                height: 1,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode,
                view_formats: vec![],
            },
            configured: false,
            info,
            background: [
                options.background_rgb[0],
                options.background_rgb[1],
                options.background_rgb[2],
                1.,
            ],
        };
        state.resize(state.window.inner_size())?;
        Ok(state)
    }

    fn resize(&mut self, size: PhysicalSize<u32>) -> io::Result<()> {
        if size.width == 0 || size.height == 0 {
            self.configured = false;
            return Ok(());
        }
        let limit = self.renderer.device().limits().max_texture_dimension_2d;
        let bytes = u64::from(size.width)
            .checked_mul(u64::from(size.height))
            .and_then(|n| n.checked_mul(4));
        if size.width > limit || size.height > limit || bytes.is_none_or(|n| n > MAX_IMAGE_BYTES) {
            return Err(invalid(
                "window surface exceeds renderer size budget or device limits",
            ));
        }
        if !self.configured || (self.config.width, self.config.height) != (size.width, size.height)
        {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(self.renderer.device(), &self.config);
            self.configured = true;
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<DrawOutcome> {
        self.resize(self.window.inner_size())?;
        if !self.configured {
            return Ok(DrawOutcome::ZeroSize);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.configured = false;
                return Ok(DrawOutcome::Retry);
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(DrawOutcome::Retry),
            Err(error) => return Err(platform(error)),
        };
        let rect = fit_image_rect(
            [self.image.width(), self.image.height()],
            [self.config.width, self.config.height],
        )
        .ok_or_else(|| invalid("zero image or surface dimensions"))?;
        let mut quad = Quad::rectangle(&self.image, [rect.x, rect.y], [rect.width, rect.height]);
        quad.blend = BlendMode::Alpha;
        let view = frame.texture.create_view(&Default::default());
        self.renderer.render_to_view(
            &view,
            self.config.width,
            self.config.height,
            self.background,
            &[quad],
        )?;
        let redraw = frame.suboptimal;
        self.window.pre_present_notify();
        frame.present();
        if redraw {
            self.configured = false;
        }
        Ok(DrawOutcome::Presented { redraw })
    }
}

struct PreviewApp {
    image: PreviewImage,
    options: PreviewOptions,
    window: Option<Arc<Window>>,
    gpu: Option<SurfaceState>,
    occluded: bool,
    retry_at: Option<Instant>,
    deadline: Option<Instant>,
    failure: Option<io::Error>,
    report: PreviewReport,
}
impl PreviewApp {
    fn stop(&mut self, event_loop: &ActiveEventLoop, reason: CloseReason) {
        self.report.close_reason = reason;
        self.gpu = None;
        self.window = None;
        self.retry_at = None;
        event_loop.exit();
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: io::Error) {
        self.failure = Some(error);
        self.stop(event_loop, CloseReason::EventLoopStopped);
    }
    fn time_expired(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.stop(event_loop, CloseReason::TimeLimit);
            true
        } else {
            false
        }
    }
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
impl ApplicationHandler for PreviewApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() || self.time_expired(event_loop) || self.gpu.is_some() {
            return;
        }
        if self.window.is_none() {
            let [width, height] = self.options.initial_logical_size;
            let attributes = Window::default_attributes()
                .with_title(self.options.title.clone())
                .with_inner_size(LogicalSize::new(width, height))
                .with_resizable(true)
                .with_visible(false);
            match event_loop.create_window(attributes) {
                Ok(window) => self.window = Some(Arc::new(window)),
                Err(error) => {
                    self.fail(event_loop, platform(error));
                    return;
                }
            }
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        match pollster::block_on(SurfaceState::new(
            window.clone(),
            &self.image,
            &self.options,
        )) {
            Ok(gpu) => {
                self.report.adapter = Some(gpu.info.clone());
                self.report.surface_format = Some(gpu.config.format);
                self.report.completed_resumes = self.report.completed_resumes.saturating_add(1);
                self.gpu = Some(gpu);
                self.occluded = false;
                self.retry_at = None;
                window.set_visible(true);
                window.request_redraw();
            }
            Err(error) => self.fail(event_loop, error),
        }
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if self.gpu.take().is_some() {
            self.report.suspensions = self.report.suspensions.saturating_add(1);
        }
        self.retry_at = None;
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if event_loop.exiting()
            || self
                .window
                .as_ref()
                .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        if self.time_expired(event_loop) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => self.stop(event_loop, CloseReason::WindowClosed),
            WindowEvent::Destroyed => self.stop(event_loop, CloseReason::WindowDestroyed),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                self.stop(event_loop, CloseReason::Escape)
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    if let Err(error) = gpu.resize(size) {
                        self.fail(event_loop, error);
                        return;
                    }
                }
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => self.request_redraw(),
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if !occluded {
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let Some(gpu) = self.gpu.as_mut() else {
                    return;
                };
                match gpu.draw() {
                    Ok(DrawOutcome::Presented { redraw }) => {
                        self.retry_at = None;
                        self.report.frames_presented =
                            self.report.frames_presented.saturating_add(1);
                        if self
                            .options
                            .frame_limit
                            .is_some_and(|limit| self.report.frames_presented >= limit)
                        {
                            self.stop(event_loop, CloseReason::FrameLimit);
                        } else if redraw || self.options.frame_limit.is_some() {
                            self.request_redraw();
                        }
                    }
                    Ok(DrawOutcome::Retry) => self.retry_at = Some(Instant::now() + RETRY_DELAY),
                    Ok(DrawOutcome::ZeroSize) => self.retry_at = None,
                    Err(error) => self.fail(event_loop, error),
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() || self.time_expired(event_loop) {
            return;
        }
        if self.retry_at.is_some_and(|retry| Instant::now() >= retry) {
            self.retry_at = None;
            self.request_redraw();
        }
        let next = self.deadline.into_iter().chain(self.retry_at).min();
        event_loop.set_control_flow(next.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.gpu = None;
        self.window = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_preserves_physical_aspect_and_fractional_offsets() {
        assert_eq!(
            fit_image_rect([1920, 1080], [1280, 1024]),
            Some(ImageRect {
                x: 0.,
                y: 152.,
                width: 1280.,
                height: 720.
            })
        );
        assert_eq!(
            fit_image_rect([1920, 1080], [1080, 1920]),
            Some(ImageRect {
                x: 0.,
                y: 656.25,
                width: 1080.,
                height: 607.5
            })
        );
        assert_eq!(
            fit_image_rect([1080, 1920], [1920, 1080]),
            Some(ImageRect {
                x: 656.25,
                y: 0.,
                width: 607.5,
                height: 1080.
            })
        );
        assert_eq!(
            fit_image_rect([1920, 1080], [1600, 1200]),
            Some(ImageRect {
                x: 0.,
                y: 150.,
                width: 1600.,
                height: 900.
            })
        );
        assert!(fit_image_rect([3, 2], [1, 0]).is_none());
        assert!(fit_image_rect([0, 2], [1, 1]).is_none());
    }
    #[test]
    fn surface_encoding_is_never_silently_changed() {
        use wgpu::TextureFormat::*;
        assert_eq!(
            choose_format(&[Rgba8UnormSrgb, Bgra8Unorm], ImageEncoding::Linear).unwrap(),
            Bgra8Unorm
        );
        assert!(choose_format(&[Bgra8UnormSrgb], ImageEncoding::Linear).is_err());
        assert_eq!(
            choose_format(&[Rgba8UnormSrgb], ImageEncoding::Srgb).unwrap(),
            Rgba8UnormSrgb
        );
    }
    #[test]
    fn invalid_input_is_rejected_before_event_loop_creation() {
        let mut image = PreviewImage {
            width: 1,
            height: 1,
            pixels: vec![0; 4],
            encoding: ImageEncoding::Linear,
        };
        let mut options = PreviewOptions::new(ImageEncoding::Linear);
        assert!(validate(&image, &options).is_ok());
        image.width = 2;
        assert!(validate(&image, &options).is_err());
        image.width = 1;
        options.frame_limit = Some(0);
        assert!(validate(&image, &options).is_err());
        options.frame_limit = None;
        options.initial_logical_size = [0, 720];
        assert!(validate(&image, &options).is_err());
        options.initial_logical_size = [1280, 720];
        options.background_rgb[1] = f64::NAN;
        assert!(validate(&image, &options).is_err());
    }
}
