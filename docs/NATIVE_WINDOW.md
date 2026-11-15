# Persistent diagnostic scene window

Task `a3d58c579542`, dev.6 host increment. This original native host draws
prepared frontend meshes directly to a window surface. It does not implement
retail startup, a timeline scheduler, frame actions, login or invented controls.
The existing decoded-image preview API remains unchanged.

## Controller and host API

The public names are available in `window::scene` and re-exported from `window`:

```rust,ignore
trait SceneController {
    fn nodes(&self) -> &[gfx::scene::SceneNode];
    fn pointer_stage_twips(&mut self, point: Option<[f64; 2]>) -> std::io::Result<()>;
}
let report = window::run_scene_preview(
    prepared, stage, policy, &mut controller, options,
)?;
```

The controller remains caller-owned and inspectable after the function returns.
Its current complete node arena goes directly to `GpuScene::render_to_view`
with `Viewport::Contain`. GPU scene identity checks continue to reject changes
to immutable prepared assets/tree/raw metadata; current world transforms,
colors and effective visibility remain caller responsibilities.

`ScenePreviewOptions::new(output_encoding)` provides title, initial logical
size, explicit surface encoding, fallback-adapter request and optional positive
QA time/frame limits. It has no image-filter or guessed-clear settings: scene
paint and clear behavior come from the explicit `PreviewPolicy`. Surface format
selection uses the existing strict encoding choice and FIFO presentation.

`ScenePreviewReport.window` reuses the ordinary `PreviewReport` presentation,
resume/suspend, adapter/format and close-reason fields. Additional counters retain
GPU generation count, cumulative asset uploads, the most recent generation's
resident RGBA byte count, pointer calls/valid points/invalidations and resize
events. `last_frame` preserves the complete latest render coverage/limitations.
A presentation counter means the host called present after submission; it does
not independently prove compositor display or retail pixel parity.

## Pointer semantics

Only a delivered native `CursorMoved` event can pass `Some([x,y])` to the
controller. Its physical surface position is converted through the current
`GpuScene` contain viewport into original stage twips. Margins/out-of-target
positions produce `None`. The host never uses raw-device deltas as a cursor
position and stores no old point to replay after viewport changes.

Cursor exit, focus loss, resize, scale-factor changes, occlusion, suspension,
initial resume and normal close invalidate availability with `None`. Re-entering
or resuming alone never synthesizes a mouse move. A later actual native move
supplies new coordinates. The controller decides whether a real in-stage move
executes a recovered handler; the host has no VM coupling.

`None` means unavailable pointer coordinates. The host does not hide, move or
alter the saved scene cursor, and does not replace the native system cursor.
Controller errors stop the loop immediately and return the original error;
there are no further controller callbacks after that failure. The caller can
inspect its controller's execution status or error records separately.

## Surface and event lifecycle

Window and Vulkan surface creation occur on the main thread after `Resumed`.
The window stays invisible during initialization. `GpuScene` retains texture
and pipeline resources between frames; pointer events only update controller
state and request redraw. Ordinary rendering is event-driven. There is no
frame-action scheduler, animation tick or continuous polling loop.

Resize reconfigures only nonzero, bounded surfaces and keeps the prepared GPU
assets. Zero-size surfaces wait without acquisition. Lost/outdated acquisitions
schedule reconfiguration; timeout retries use a bounded 50 ms wakeup. Other
surface/render errors stop explicitly. Each acquired frame is submitted and
presented before another configure can occur. Occluded windows stop drawing
until exposure.

A real `Suspended` callback drops the native surface/device generation before
returning. Only the CPU prepared scene is retained for rebuilding on resume;
controller state persists. CPU assets are copied at that rare transition, not
per normal frame. Duplicate resume/suspend notifications are tolerated. New
GPU generations increase upload/build counters explicitly, while normal resize
and redraw do not. This is lifecycle handling, not a claim of platform coverage.

Close, destroy and Escape preserve the existing host-close behavior. Optional
QA deadlines include initialization time but cannot interrupt a synchronous
platform/GPU initialization call; private smoke processes therefore also need
an outer timeout. Frame limits request additional redraws only while that
explicit QA option is active.

## Validation scope

**VERIFIED:** The integrated window-feature build passes. A private fixture using
the actual saved manager presents two frames; pointer/resize testing presents six
frames and checks stage-center mapping, margin invalidation and a 1000x1000 to
1000x800 resize. Exactly one GPU generation and one asset texture upload remain
resident (16,384 bytes); normal frames upload no assets. A deliberately returned
controller error is preserved, and the existing image API still presents two
frames. A fixture's exact fractional f64 comparison was corrected to a 1e-6-twip
tolerance, preserving the initial failure. Records and source hashes are in
`evidence/native-scene-checks/result.json`, `attempt1/result.json`, `sources.json`
and `outside-sources.json`. No suspend/resume or compositor pixel parity was run.

**VERIFIED (integrated original handler):** The final dev.6 CLI's separate fresh
Xvfb check sends movements through the original manager handler, resizes to
1000x1000, and checks the cursor at (9600,12000) twips after surface point (250,500).
The following margin movement leaves it there. The scene retains one texture and
one GPU generation and presents multiple frames. `--seconds 10` closes through
the native host deadline. Receipt: `evidence/dev6-native-pointer-attempt2/receipt.json`.
This is XWarpPointer/XResizeWindow event delivery on llvmpipe, not physical-input,
owner-device, native-retail visual or general initialized frontend parity.

An earlier root check reached its window but synthetic XSendEvent Escape did not
close it. Its timeout is retained in `evidence/dev6-native-pointer/receipt.json`.
Winit requires a focused active window for key handling; absent focus on bare
Xvfb is a supported explanation, not a proven diagnosis or physical Escape defect.
The subsequent check uses the explicit QA deadline without synthetic key input.

No user display, launcher, server session, account or Wine prefix
was touched. Native runtime libraries copied for previous Ubuntu
smokes stay under private `evidence/runtime-libs`; only test-child environment
variables select them. They are not assumed portable to the owner's other OS.

## Source and licence register

All application code is original. The existing project image-window host is
reused as the native lifecycle/API foundation. Cached registry roots below are
`/home/blizz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

| Source read | Licence/provenance | Use |
| --- | --- | --- |
| `winit-0.30.13/{Cargo.toml,src/application.rs,src/event.rs}` | Apache-2.0 | Exact main-thread lifecycle, redundant resume/suspend handling, physical cursor/resize coordinates, redraw/occlusion/scale-factor events |
| `wgpu-27.0.1/{Cargo.toml,src/api/surface.rs}` | MIT OR Apache-2.0 | Configure/acquire/present ordering and prohibition on reconfiguration while a frame remains acquired |
| Original project `src/window.rs`, `src/frontend/preview.rs`, `src/render.rs`, `gfx::scene::SceneNode`, `docs/{WINDOW,FRONTEND_PREVIEW,RENDERER}.md` | Original project source; component provenance already registered | Existing native format selection and diagnostics, reusable GPU assets, exact stage mapping and immutable scene identity contract |
| Private pinned frontend/MarvelGame/MarvelFonts packages and prior original preparation/readback evidence | Proprietary owner-provided data for authorized compatibility study | Actual manager scene used only in private smoke fixtures; no asset or capture enters source commits |
| Existing private `evidence/runtime-libs` manifests/notices and `docs/WINDOW.md` | Verified cached official Ubuntu runtime packages; notices retained privately | Reuse prior isolated native-test runtime without shared installation or portability assumptions |
| `x11-dl-2.21.0/{Cargo.toml,src/xlib.rs,src/xtest.rs}` | MIT | C ABI declarations for the original private Xvfb driver; XTest signature was inspected but not invoked |
| `winit-0.30.13/{src/window.rs,src/platform_impl/linux/x11/event_processor.rs}` | Apache-2.0 | Focus requirement and targeted synthetic-key diagnostic; exact hashes retained in the private source ledger |

The root integrator owns CLI/controller wiring, build numbering, native player
QA handoff and the global provenance ledger. This lane adds no dependency.
