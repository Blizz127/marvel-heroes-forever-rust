//! Native event-driven host for prepared scene assets and caller-owned state.
//! No script scheduler, fabricated controls or native input injection lives here.

use super::{CloseReason, PreviewReport, RETRY_DELAY, choose_format, invalid, platform};
use crate::{
    frontend::{
        preview::{FrameReport, GpuScene, PreviewPolicy, Viewport},
        scene_content::PreparedScene,
    },
    gfx::{Rect, scene::SceneNode},
    render::{ImageEncoding, QuadRenderer},
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

/// The caller owns script/event interpretation and consistent descendant state.
/// Only a real CursorMoved event can supply Some(stage_twips). None invalidates
/// availability on margins/leave/resize/focus loss/suspend/close; it is not a
/// synthetic MouseMove and must not imply hiding or relocating saved content.
pub trait SceneController {
    fn nodes(&self) -> &[SceneNode];
    fn pointer_stage_twips(&mut self, point: Option<[f64; 2]>) -> io::Result<()>;
}

pub struct ScenePreviewOptions {
    pub title: String,
    pub initial_logical_size: [u32; 2],
    pub output_encoding: ImageEncoding,
    pub force_fallback_adapter: bool,
    pub close_after: Option<Duration>,
    pub frame_limit: Option<u64>,
}
impl ScenePreviewOptions {
    pub fn new(output_encoding: ImageEncoding) -> Self {
        Self {
            title: "Marvel Heroes Forever — diagnostic scene".into(),
            initial_logical_size: [1280, 720],
            output_encoding,
            force_fallback_adapter: false,
            close_after: None,
            frame_limit: None,
        }
    }
}
#[derive(Debug)]
pub struct ScenePreviewReport {
    pub window: PreviewReport,
    /// Successful GPU/device generations; normal resize does not add one.
    pub gpu_builds: u64,
    pub asset_texture_uploads: u64,
    pub resident_rgba_bytes: usize,
    pub pointer_updates: u64,
    pub pointer_in_stage: u64,
    pub pointer_invalidations: u64,
    pub resize_events: u64,
    pub last_frame: Option<FrameReport>,
}

/// Run on the main thread, once per process. The host acquires/presents native
/// surfaces and calls the controller only for pointer availability updates.
/// Current scene nodes are rendered directly, never via a framebuffer upload.
/// Controller/render/platform failures end the loop and return Err; the caller
/// retains its controller for inspection. No ordinary periodic redraw is added.
pub fn run_scene_preview(
    prepared: PreparedScene,
    stage: Rect,
    policy: PreviewPolicy,
    controller: &mut impl SceneController,
    options: ScenePreviewOptions,
) -> io::Result<ScenePreviewReport> {
    validate(&options)?;
    let deadline = options
        .close_after
        .map(|delay| {
            Instant::now()
                .checked_add(delay)
                .ok_or_else(|| invalid("scene preview deadline overflows"))
        })
        .transpose()?;
    let event_loop = EventLoop::new().map_err(platform)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = SceneApp {
        prepared: Some(prepared),
        stage,
        policy,
        controller,
        options,
        window: None,
        gpu: None,
        occluded: false,
        retry_at: None,
        deadline,
        failure: None,
        stopped: false,
        report: ScenePreviewReport {
            window: PreviewReport {
                frames_presented: 0,
                completed_resumes: 0,
                suspensions: 0,
                adapter: None,
                surface_format: None,
                close_reason: CloseReason::EventLoopStopped,
            },
            gpu_builds: 0,
            asset_texture_uploads: 0,
            resident_rgba_bytes: 0,
            pointer_updates: 0,
            pointer_in_stage: 0,
            pointer_invalidations: 0,
            resize_events: 0,
            last_frame: None,
        },
    };
    event_loop.run_app(&mut app).map_err(platform)?;
    if let Some(error) = app.failure {
        Err(error)
    } else {
        Ok(app.report)
    }
}
fn validate(options: &ScenePreviewOptions) -> io::Result<()> {
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
    if options.frame_limit == Some(0) || options.close_after == Some(Duration::ZERO) {
        return Err(invalid("QA frame/time limits must be positive"));
    }
    Ok(())
}

struct SceneSurface {
    // Drop the surface before explicit window owners on native suspension.
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
    device: wgpu::Device,
    scene: GpuScene<'static>,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    info: wgpu::AdapterInfo,
}
enum Outcome {
    Presented { redraw: bool, report: FrameReport },
    Retry,
    ZeroSize,
}
impl SceneSurface {
    async fn new(
        window: Arc<Window>,
        prepared: PreparedScene,
        stage: Rect,
        policy: PreviewPolicy,
        options: &ScenePreviewOptions,
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
                label: Some("Marvel diagnostic scene window"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(platform)?;
        let renderer = QuadRenderer::from_device(device.clone(), queue, format).await?;
        let scene = GpuScene::new(&renderer, prepared, stage, policy).await?;
        let mut result = Self {
            surface,
            window,
            device,
            scene,
            configured: false,
            info,
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
        };
        result.resize(result.window.inner_size())?;
        Ok(result)
    }
    fn resize(&mut self, size: PhysicalSize<u32>) -> io::Result<()> {
        if size.width == 0 || size.height == 0 {
            self.configured = false;
            return Ok(());
        }
        self.scene
            .viewport(size.width, size.height, Viewport::Contain)?;
        if !self.configured || [self.config.width, self.config.height] != [size.width, size.height]
        {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.configured = true;
        }
        Ok(())
    }
    fn stage_point(&self, point: [f64; 2]) -> io::Result<Option<[f64; 2]>> {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(None);
        }
        Ok(self
            .scene
            .viewport(size.width, size.height, Viewport::Contain)?
            .stage_point(point))
    }
    fn draw(&mut self, nodes: &[SceneNode]) -> io::Result<Outcome> {
        self.resize(self.window.inner_size())?;
        if !self.configured {
            return Ok(Outcome::ZeroSize);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.configured = false;
                return Ok(Outcome::Retry);
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(Outcome::Retry),
            Err(error) => return Err(platform(error)),
        };
        let view = frame.texture.create_view(&Default::default());
        let submitted = self.scene.render_to_view(
            &view,
            self.config.width,
            self.config.height,
            nodes,
            Viewport::Contain,
        )?;
        let redraw = frame.suboptimal;
        self.window.pre_present_notify();
        frame.present();
        if redraw {
            self.configured = false;
        }
        Ok(Outcome::Presented {
            redraw,
            report: submitted.report,
        })
    }
}

struct SceneApp<'a, C: SceneController> {
    prepared: Option<PreparedScene>,
    stage: Rect,
    policy: PreviewPolicy,
    controller: &'a mut C,
    options: ScenePreviewOptions,
    window: Option<Arc<Window>>,
    gpu: Option<SceneSurface>,
    occluded: bool,
    retry_at: Option<Instant>,
    deadline: Option<Instant>,
    failure: Option<io::Error>,
    stopped: bool,
    report: ScenePreviewReport,
}
impl<C: SceneController> SceneApp<'_, C> {
    fn pointer(&mut self, point: Option<[f64; 2]>) -> io::Result<()> {
        self.report.pointer_updates = self.report.pointer_updates.saturating_add(1);
        if point.is_some() {
            self.report.pointer_in_stage = self.report.pointer_in_stage.saturating_add(1);
        } else {
            self.report.pointer_invalidations = self.report.pointer_invalidations.saturating_add(1);
        }
        self.controller.pointer_stage_twips(point)
    }
    fn finish(&mut self, event_loop: &ActiveEventLoop, reason: CloseReason) {
        self.report.window.close_reason = reason;
        self.stopped = true;
        self.gpu = None;
        self.window = None;
        self.retry_at = None;
        event_loop.exit();
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: io::Error) {
        self.failure = Some(error);
        self.finish(event_loop, CloseReason::EventLoopStopped);
    }
    fn stop(&mut self, event_loop: &ActiveEventLoop, reason: CloseReason) {
        match self.pointer(None) {
            Ok(()) => self.finish(event_loop, reason),
            Err(error) => self.fail(event_loop, error),
        }
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
    fn invalidate(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Err(error) = self.pointer(None) {
            self.fail(event_loop, error);
            false
        } else {
            true
        }
    }
}
impl<C: SceneController> ApplicationHandler for SceneApp<'_, C> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() || self.time_expired(event_loop) || self.gpu.is_some() {
            return;
        }
        if !self.invalidate(event_loop) {
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
        let Some(prepared) = self.prepared.take() else {
            self.fail(event_loop, platform("scene CPU state missing on resume"));
            return;
        };
        match pollster::block_on(SceneSurface::new(
            window.clone(),
            prepared,
            self.stage,
            self.policy,
            &self.options,
        )) {
            Ok(gpu) => {
                let resources = gpu.scene.resources();
                self.report.gpu_builds = self.report.gpu_builds.saturating_add(1);
                self.report.asset_texture_uploads = self
                    .report
                    .asset_texture_uploads
                    .saturating_add(resources.asset_texture_uploads as u64);
                self.report.resident_rgba_bytes = resources.resident_rgba_bytes;
                self.report.window.adapter = Some(gpu.info.clone());
                self.report.window.surface_format = Some(gpu.config.format);
                self.report.window.completed_resumes =
                    self.report.window.completed_resumes.saturating_add(1);
                self.gpu = Some(gpu);
                self.occluded = false;
                self.retry_at = None;
                window.set_visible(true);
                window.request_redraw();
            }
            Err(error) => self.fail(event_loop, error),
        }
    }
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() || !self.invalidate(event_loop) {
            return;
        }
        if let Some(gpu) = self.gpu.take() {
            // Preserve only CPU assets for a new native device/surface generation.
            // Normal resize/loss recovery never takes this path or reuploads assets.
            self.prepared = Some(gpu.scene.prepared().clone());
            self.report.window.suspensions = self.report.window.suspensions.saturating_add(1);
        }
        self.retry_at = None;
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if event_loop.exiting()
            || self.window.as_ref().is_none_or(|w| w.id() != id)
            || self.time_expired(event_loop)
        {
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
            WindowEvent::CursorMoved { position, .. } => {
                let point = match self.gpu.as_ref().filter(|_| !self.occluded) {
                    Some(gpu) => match gpu.stage_point([position.x, position.y]) {
                        Ok(point) => point,
                        Err(error) => {
                            self.fail(event_loop, error);
                            return;
                        }
                    },
                    None => None,
                };
                if let Err(error) = self.pointer(point) {
                    self.fail(event_loop, error);
                    return;
                }
                self.request_redraw();
            }
            WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false) => {
                if self.invalidate(event_loop) {
                    self.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                self.report.resize_events = self.report.resize_events.saturating_add(1);
                if !self.invalidate(event_loop) {
                    return;
                }
                if let Some(gpu) = self.gpu.as_mut() {
                    if let Err(error) = gpu.resize(size) {
                        self.fail(event_loop, error);
                        return;
                    }
                }
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if self.invalidate(event_loop) {
                    self.request_redraw();
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if occluded {
                    self.invalidate(event_loop);
                } else {
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
                match gpu.draw(self.controller.nodes()) {
                    Ok(Outcome::Presented { redraw, report }) => {
                        self.report.last_frame = Some(report);
                        self.retry_at = None;
                        self.report.window.frames_presented =
                            self.report.window.frames_presented.saturating_add(1);
                        if self
                            .options
                            .frame_limit
                            .is_some_and(|limit| self.report.window.frames_presented >= limit)
                        {
                            self.stop(event_loop, CloseReason::FrameLimit);
                        } else if redraw || self.options.frame_limit.is_some() {
                            self.request_redraw();
                        }
                    }
                    Ok(Outcome::Retry) => self.retry_at = Some(Instant::now() + RETRY_DELAY),
                    Ok(Outcome::ZeroSize) => self.retry_at = None,
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
    fn exiting(&mut self, _: &ActiveEventLoop) {
        if !self.stopped && self.failure.is_none() {
            if let Err(error) = self.pointer(None) {
                self.failure = Some(error);
            }
        }
        self.gpu = None;
        self.window = None;
    }
}
