# Native image preview host

Task `a3d58c579542`. `src/window.rs` is an original host for viewing one actual
decoded image through the quad renderer. Its layout is an inspection viewport;
it is not the recovered retail frontend. No login controls, movie execution,
network connection or completed modern-graphics option is added.

## Integration and API

Enable this module only with the optional native-preview feature. Besides the
renderer dependencies, it needs exact `winit = "=0.30.13"`, default features
disabled, features `["rwh_06", "x11", "wayland", "wayland-dlopen"]`, and exact
`pollster = "=0.4.0"`. Winit declares Rust 1.70; the renderer sets the effective
minimum to Rust 1.88. The backend is Vulkan. Cargo and CLI feature wiring belong
to the integrator. X11 also requires native `libxkbcommon-x11.so.0` and its
`libxcb-xkb.so.1` dependency in addition to the installed X11/XCB/base xkbcommon
libraries. Winit's `xkbcommon-dl` loader can panic when these are absent; this
upstream failure is not converted into a normal platform error by the host.

```rust,ignore
let image = PreviewImage {
    width: decoded.width,
    height: decoded.height,
    pixels: decoded.pixels,
    encoding: recovered_input_encoding,
};
let options = PreviewOptions::new(explicit_output_encoding);
let report = run_image_preview(image, options)?;
```

`run_image_preview` must run on the main thread, with one event loop per
process. It owns the RGBA bytes so suspension can release GPU objects and resume
can upload the same image. Callers retain responsibility for selecting the
input and output transfer functions from evidence. No file paths or package
parsing are embedded in this host.

Options specify an initial logical size, title, nearest/linear sampling, opaque
background RGB, explicit output encoding and optional software-adapter request.
Ordinary close/Escape behavior is the default. Optional `frame_limit` and
`close_after` are QA controls. The time limit is checked by the event loop and
does not interrupt blocking driver initialization. The result reports frame and
resume/suspend counts, selected adapter, surface format and close reason.

## Surface and lifecycle behavior

- Winit selects the platform from the caller's existing display environment.
  The host neither discovers an alternative display nor changes authentication.
- Window and surface creation happen in `resumed`. An owned `Arc<Window>` lets
  the surface retain its required native handle. The requested adapter must
  support that surface. Repeated resume events do not duplicate live resources.
- Suspension drops the entire surface/device/image state before returning.
  Resume creates a new compatible surface and renderer. Close, Escape and
  destruction release the surface before the explicit window owner and exit.
- Surface dimensions and image fitting use physical pixels. Resize and DPI
  changes are followed by reading the actual window size. A zero-size window
  is neither configured nor acquired. Nonzero surfaces must fit device limits
  and the 256 MiB pixel budget; oversized surfaces return an error.
- `fit_image_rect` fits the image inside the window with centered letterboxing,
  preserving aspect ratio and fractional coordinates. It does not impose a
  recovered retail snapping rule. The complete original image is sampled.
- Rendering occurs only on redraw events; idle operation waits. Resize and
  visibility recovery request redraw. Frame-limited QA requests additional
  frames until its limit. Occluded windows skip drawing.
- FIFO presentation is required. Submitted work is followed by
  `pre_present_notify` and presentation. Lost/outdated surfaces are reconfigured
  on retry; timeouts retry after 50 ms without a busy loop. Suboptimal images
  are presented before reconfiguration. Other acquisition failures return an
  error. No acquired frame survives across configuration or callbacks.

Input is straight-alpha RGBA8, with the renderer's explicit Linear/Srgb sampling
contract. The surface must support BGRA8 or RGBA8 with exactly the requested
output encoding. The host refuses to silently substitute an sRGB surface for a
linear one or conversely. Source-over blending uses an opaque background;
prefer opaque compositing when supported. Background values are shader/blend
space values, before any requested target sRGB encoding. These choices define
this preview's numerical behavior; they do not prove the retail client's final
display transfer or the compositor's color management.

## Validation and limits

**VERIFIED (2026-11-15):** The offline private harness compiled with rustc
1.98.1. Three focused pure checks passed: physical/fractional aspect fitting,
exact surface-encoding selection and rejection of malformed input before event
loop creation. The build used `nice -n 19`, two Cargo jobs and the renderer's
existing dependency target cache. Unused public renderer API warnings are
limited to the smaller private harness.

Focused validation is recorded privately beneath
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `window-harness/`: source and locked offline dependency graph. Its default
  invocation does not create an event loop or window. Its separate `--preview`
  mode loads the private 2560×1440 frontend background and requests two frames
  with a ten-second event-loop limit.
- `window-build-test.log`: focused compile and pure-test receipt.
- `window-build-native.log`, `window-smoke.log`, `window-xvfb.log`: native
  executable build and isolated-display execution receipts.
- `window-smoke-missing-runtime.log`: initial X11 runtime-loader failure before
  the missing libraries were provided privately.
- `window-validation/checks.json`: exact source identities and check outcomes.
- `window-dependencies.json`, `window-dependency-licences.json`: resolved
  dependency metadata and declared licences, not a legal review.
- `window-reference-files.json`: hashes of API sources read for this host.
- `runtime-libs/`: two locally cached official Ubuntu runtime packages, their
  original copyright notices, extracted libraries, official apt metadata and
  hash/version verification manifest. No shared system packages were modified.

**OBSERVED (2026-11-15):** The worker environment contains neither `DISPLAY` nor
`WAYLAND_DISPLAY`. Unrelated X sockets/processes exist, but none was selected or
attached. The lead authorized a fresh task-local Xvfb display with TCP disabled;
the test used `xvfb-run -a`, a 1600×1000 virtual screen and a 30-second outer
process timeout. Its temporary display was released when the test exited. No
trust or login prompt was pressed.

**VERIFIED:** The native private executable exited successfully after two
presentations and one completed resume, with `FrameLimit` as the close reason.
It loaded the real 2560×1440 frontend background, selected Linear input/output,
linear sampling, opaque black background and a `Bgra8Unorm` surface. The selected
adapter was llvmpipe (LLVM 21.1.8, 256 bits), Vulkan CPU adapter, Mesa
`26.0.8-1ubuntu0.3`. This proves the exercised native creation/upload/draw/present
and bounded-close path. It is not a native screenshot or pixel-comparison check.

The first attempt failed before a window was created because the system lacked
`libxkbcommon-x11`. Existing Forever cache packages matched the installed Ubuntu
26.04/resolute metadata: `libxkbcommon-x11-0` 1.13.1-1 and `libxcb-xkb1`
1.17.0-2ubuntu1. Both cached `.deb` SHA-256 values matched the official apt-index
values. Installed base xkbcommon/XCB versions matched their dependencies.
The archives were copied/extracted only under private evidence, with notices;
`LD_LIBRARY_PATH` was set only for the test child. No download, installation or
shared-cache edit was needed. Exact package identities are in the manifest.

Resize/minimize, DPI changes, user close/Escape, suspension, surface-loss
recovery, Wayland, a physical desktop and physical GPU behavior remain
**unverified at runtime**. The previous offscreen renderer check establishes
exact native-size asset readback for its recorded settings; it does not prove
this native compositor's pixel output. Retail frontend composition, original
display transfer and behavior also remain unverified.

## Source and licence register

All host code is newly written. Sources were used for lifecycle/API constraints;
no application implementation or example assets were copied. Registry paths are
under `/home/blizz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

| Source | Licence | Use |
| --- | --- | --- |
| `winit-0.30.13/{Cargo.toml,LICENSE,src/application.rs,src/event.rs,src/event_loop.rs,src/window.rs}` | Apache-2.0 | Exact cached version/features, event-loop/main-thread constraints, resume/suspend ownership, physical-size events and redraw/presentation notifications |
| `wgpu-27.0.1/src/api/{instance,surface,surface_texture}.rs`; `wgpu-types-27.0.1/src/lib.rs` | MIT OR Apache-2.0 | Surface creation/lifetimes, compatible adapters, supported configuration, acquisition errors and frame lifetime |
| Existing project `src/render.rs`, `docs/RENDERER.md`; `pollster-0.4.0` Cargo metadata | Original project code/docs; pollster MIT OR Apache-2.0 | Renderer integration, existing transfer/alpha contract and synchronous initialization executor |
| Worker display environment; `/tmp/.X11-unix/` directory and running X-server process names | Local operational metadata; no implementation copied | Determine whether an owned display was already selected; no connection performed |
| `/usr/bin/xvfb-run` header/options and `--help`; `/usr/share/doc/xvfb/copyright` opening licence declaration | Installed X.Org package declares MIT/X11 variants and legacy BSD-like notices; no code copied or redistributed | Task-local virtual-display invocation, default TCP-disabled behavior and cleanup contract |
| `xkbcommon-dl-0.4.2/{Cargo.toml,src/x11.rs}` | MIT | Diagnose the observed missing-library panic and expected native sonames |
| `/etc/os-release`, `apt-cache policy/show` and installed package versions | Local distro/package metadata | Establish matching Ubuntu release, official archive origin, dependencies and package hashes |
| `/home/blizz/.cache/forever-client-build/hostlibs/deb/libxkbcommon-x11-0_1.13.1-1_amd64.deb`; same directory `libxcb-xkb1_1.17.0-2ubuntu1_amd64.deb` | xkbcommon: MIT/X11 variants and legacy permissive notices; XCB: MIT/X11 variant; full package notices retained privately | Reuse existing official distro runtime packages, verify against apt-index SHA-256 and extract into the project-private native test environment |
| Private `texture-validation/frontendbg_2560x1440.rgba` path/dimensions from texture lane | Proprietary owner-provided Gazillion/Marvel asset, original project decode | Input named in the private optional native harness; no game bytes copied into source or builds |

Primary upstream projects: [winit](https://github.com/rust-windowing/winit),
[wgpu](https://github.com/gfx-rs/wgpu),
[pollster](https://github.com/zesterer/pollster). The cached exact sources and
lockfile define the studied versions. A distributed private build needs the
dependency notices required by its actual resolved graph.
