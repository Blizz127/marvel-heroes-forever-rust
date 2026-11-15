# Original image-quad renderer

Task `a3d58c579542`. `src/render.rs` is newly written rendering infrastructure
for decoded original images. It does not invent a login layout, execute a
Scaleform movie, connect to a server or claim the frontend milestone complete.

## API and integration

The module requires `wgpu = "=27.0.1"`, with default features disabled and
`["std", "vulkan", "wgsl"]` enabled. This cached release declares Rust 1.88;
the host project must raise its minimum version if it enables the module.
The renderer itself needs no executor dependency. The private native smoke uses
`pollster = "=0.4.0"`; a host may use its own executor.

- `QuadRenderer::headless(format, force_fallback).await` requests a Vulkan device
  without a window. Fallback means requesting a software adapter, not silently
  replacing a failed hardware request. Adapter identity is available from
  `adapter_info()`.
- `QuadRenderer::from_device(device, queue, format).await` accepts a host-owned
  compatible device for later surface integration. The host selects a device
  compatible with its surface; no native window is created here.
- `upload_rgba(RgbaImage { width, height, pixels }, encoding, filter)` uploads
  tightly packed top-to-bottom RGBA8 bytes. Neither byte order nor alpha is
  transformed on the CPU. Encoding and nearest/linear filtering are explicit.
- `Quad::rectangle(image, position, size)` creates pixel-space geometry with
  the origin at the top left. Public corners, UVs and tint also permit affine
  quads, atlas regions and flips. The renderer does not infer SWF transforms.
- `render_to_view(...)` draws in slice order into a supplied single-sample 2D
  target on the same device and with the renderer's format. The host must supply
  matching dimensions and a `RENDER_ATTACHMENT` view.
- `offscreen_rgba(...)` renders and reads a private in-memory target. Row padding
  is removed; BGRA targets are converted to returned RGBA order. No filesystem,
  network or display access occurs in the module.

Only RGBA8/BGRA8 unorm and sRGB target formats are admitted. Input dimensions,
byte lengths, target/device limits, foreign-renderer images, nonfinite draw
values, clear/alpha ranges, quad count and allocation sizes are checked before
submission. Images/targets are capped at 256 MiB of tightly packed pixels and
scenes at 65,536 quads. GPU readback waits at most 30 seconds, followed by a
bounded mapping-result receive. Errors are returned without retrying GPU work.

## Color and alpha contract

`ImageEncoding::Linear` uses `Rgba8Unorm`; `Srgb` uses `Rgba8UnormSrgb`. The latter
asks the GPU to decode sampled RGB before shader arithmetic. The target's sRGB
variant requests encoding after shader/blend arithmetic. Alpha is not subjected
to the RGB transfer function. Tint multiplies sampled RGB and alpha separately.

`BlendMode::Alpha` takes **straight-alpha source** and uses source-over RGB
and alpha. A transparent clear plus alpha blending accumulates premultiplied
RGB in the target. `Readback` therefore returns raw composited target values,
with their encoding noted, and deliberately does not label every scene as
straight-alpha or unpremultiply it. `BlendMode::Replace` replaces all four
channels and is useful for checking native decoded-asset byte identity.

**Unverified:** Which transfer/blend settings reproduce the original client's
complete UE3/Scaleform display path. Texture SRGB metadata alone does not prove
that display contract. A native asset check can explicitly select the observed
texture metadata and verify those selected settings without claiming parity.
No HDR, AA/upscaler, gamma correction enhancement or other unfinished upgrade is
exposed by this infrastructure.

## Validation and evidence

**VERIFIED (2026-11-15):** The private offline build completed with rustc
1.98.1, wgpu 27.0.1 and no changes to the host Cargo files. Nine focused GPU
smoke groups passed on **llvmpipe (LLVM 21.1.8, 256 bits)**, Vulkan CPU adapter,
Mesa `26.0.8-1ubuntu0.3`. The invocation explicitly selected the installed
`lvp_icd.json` and requested the fallback adapter. This is a software-renderer
observation, not a physical GPU or native-window check.

The checks cover exact upload/replace/readback including transparent RGB;
top-left placement and orientation; both aligned and padded copy rows;
source-over alpha with opaque/transparent clears; order-dependent composition;
explicit sRGB sampling/target conversion; reusable-device BGRA channel order;
linear filtering and independent tint alpha; and rejected bad lengths,
dimensions, NaNs and foreign-renderer handles.

**VERIFIED:** A real frontend texture, `frontendbg_2560x1440` (zero-based export
168), was independently decoded by the texture reader from private package
SHA-256 `af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190`.
Its explicit SRGB property is false. With Linear input, nearest filtering,
Rgba8Unorm target and Replace blending, all **14,745,600 RGBA bytes** of the
2560×1440 GPU readback equal the supplied decoded bytes. Input and output SHA-256:
`3b0654671ef42f1f96f7b511bbb4b8b9bce422554ab8ef52c7c8ad6d98839aec`.
The derived readback PNG was visually inspected: it is the SHIELD-pattern
background image, not a reconstructed login screen.

Private evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `renderer-harness/`: exact original harness, dependency lockfile and binary.
- `renderer-build.log`, `renderer-smoke.log`: compile and execution receipts.
- `renderer-validation/checks.json`: adapter/settings/source/renderer/binary
  identities, passed checks and limits.
- `renderer-validation/frontendbg_2560x1440-readback.{rgba,png}`: private GPU
  readback and lossless PNG container made with the Python standard library.
- `texture-validation/samples.json`: independently supplied texture identity.
- `renderer-reference-files.json`: exact read-reference file hashes.
- `renderer-dependencies.json`, `renderer-dependency-licences.json`: resolved
  offline dependency metadata and declared licences (not individual legal review).

No asset/capture bytes enter the repository. The public render-to-view path
compiles and shares the checked encoder, but an actual native surface/window has
not been opened. Retail frontend composition, original-client comparison,
physical GPU behavior and performance remain unverified. The real-texture check
proves the selected upload/quad/readback path; it does not independently prove
the texture decoder or the original display transfer function.

## Source and licence register

All files below were read as primary API/format documentation; no shader or
renderer implementation was copied. Registry paths are beneath
`/home/blizz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

| Source | Licence | Use |
| --- | --- | --- |
| `wgpu-27.0.1/Cargo.toml`, `LICENSE.MIT`, `LICENSE.APACHE` | MIT OR Apache-2.0; gfx-rs developers | Exact cached feature/version/MSRV and licence declarations |
| `wgpu-27.0.1/src/api/{adapter,instance,device,buffer,queue,texture,command_encoder,render_pass,render_pipeline}.rs` | MIT OR Apache-2.0 | Device/pipeline API signatures, copy/map lifetimes and error-scope behavior |
| `wgpu-types-27.0.1/src/lib.rs`, `src/instance.rs` and Cargo metadata | MIT OR Apache-2.0 | Device limits, image/target formats, alpha blend constants, copy row-alignment and bounded polling |
| `pollster-0.4.0/Cargo.toml` and licence file names | MIT OR Apache-2.0 as declared | Private smoke executor only; no implementation studied |
| Private `texture-validation/samples.json` and decoded `frontendbg_2560x1440.rgba` | Proprietary owner-provided Gazillion/Marvel asset; original project decode/evidence | Actual native-size texture validation, retained privately only |
| Project `docs/GRAPHICS_PLAN.md` | Original project documentation; upstream peer sources registered there | Keep requested settings separate from observed effects, host/render separation and evidence limits |
| `/usr/share/vulkan/icd.d/` directory listing; `lvp_icd.json`; `ldconfig -p` | System configuration/installed runtime metadata; no implementation copied | Discover an already-installed software Vulkan candidate; discovery does not establish device creation |

Upstream API source: [wgpu](https://github.com/gfx-rs/wgpu), exact cached 27.0.1.
Private compiled dependencies are resolved offline; effective transitive crate
identity remains in the harness lockfile. Adding a native renderer to a private
build also requires recording its dependency licence notices in that build's
packaging; no proprietary game assets belong in the build.

The independent API-check helper confirmed the same source-over alpha contract
and proposed numeric oracle cases before execution. Its read-only Vulkan loader
check found installed `libvulkan.so.1` and `libvulkan_lvp.so`; `lvp_icd.json`
declares API version 1.4.335. These discovery observations are separate from the
adapter that an executed check actually selects.
