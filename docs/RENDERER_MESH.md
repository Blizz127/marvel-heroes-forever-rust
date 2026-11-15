# Recovered triangle fill renderer

Task `a3d58c579542`. `render::mesh` draws original `gfx::tessellate::FillMesh`
triangles. It never replaces their geometry with bounds. The caller owns scene
ordering, visibility, movie/import identity, texture resolution, text layout,
scale9 handling and clip-depth interval changes. This is an original GPU fill
pipeline, not a completed retail frontend.

## Integration

The module uses the existing exact wgpu 27.0.1 dependency and Vulkan device;
there are no new crates. Construct `MeshRenderer::new(&quad_renderer).await`.
The borrowed quad renderer supplies its device, queue, target format and image
ownership identity. Its public API remains unchanged. Both pipelines use the
same bounded private readback helper after independently preparing their draws.

`MeshRenderer::{render_to_view,offscreen_rgba}` accept physical width/height,
clear RGBA and an ordered `Command` slice. Target view/device/format requirements
match the quad renderer. Each render call clears its color target and a private
stencil attachment; it does not append to a prior pass or retain mask state.

- `Command::Draw(Draw)` borrows an actual fill mesh. `Draw` provides an affine
  transform, color transform, explicit `Paint`, Alpha/Replace blend and optional
  physical-pixel scissor.
- `Command::PushMask { parts, scissor }` unions the actual triangles from its
  `MaskPart` slice, intersecting that coverage with the previous mask level.
- `Command::PopMask` removes the most recent mask. Underflow, unclosed masks and
  nesting beyond 255 fail before submission. An empty mask has empty coverage.

`Affine([a,b,c,d,tx,ty])` maps `x'=a*x+c*y+tx`, `y'=b*x+d*y+ty`. It uses `f64`
until shader vertex conversion. Draw results must be physical target pixels;
ordinary shape inputs are local twips. `Affine::from_swf` alone maps twips to
twips. `then(outer)` applies the inner transform first. Append the stage/viewport
mapping and 1:20 conversion explicitly. Glyph-derived meshes may instead retain
font units if their supplied transform explicitly maps those units to pixels.
No automatic snapping, glyph substitution or inferred geometry is performed.

`ColorTransform` uses normalized-channel `multiply` and `add`; `from_swf`
converts the saved multiplier by `/256` and additive channel by `/255`. A host
with already composed 0–255 additive terms must divide those by 255. The shader
clamps after the supplied transform. Intermediate parent/child clamping and
retail rounding remain unverified.

## Fill coordinates and sampling

Fill coordinates are calculated from each **local** mesh position by the
preserved inverse fill matrix, before the placement/viewport transform. Bitmap
inverse coordinates are image pixels; dividing by 20 again would be incorrect.

`Paint.bitmap` must be present exactly for bitmap fills. It records the original
character ID, an uploaded image from this quad renderer and nonzero logical
dimensions. Mismatched IDs and foreign-device images are rejected. Solid or
gradient fills reject an extraneous bitmap binding.

The implemented bitmap mapping is
`uv = inverse_fill(local_shape_point) / logical_size`, sampling the whole uploaded
texture. This is an explicit **whole-image replacement** policy. It handles a
texture rescaled to a logical image. It does not reconstruct padding, a subimage
rectangle, an atlas transform or the runtime image matrix. The inference that
the observed GFX image dimensions are the correct logical dimensions, especially
when decoded dimensions differ, is not established retail behavior. The host
must report known unsupported mappings rather than silently treating them as
whole-image replacements.

The recovered repeat flag selects Repeat or ClampToEdge. Adobe describes the
clipped variant as extending edge colors; it does not make out-of-bounds samples
transparent. Recovered smoothing selects bilinear or nearest sampling. These
choices override the sampling filter used when the image was first uploaded.
There is one uploaded mip: retail minification/mipmap/atlas-edge behavior has
not been recovered.

Gradient coordinates use the original canonical range:

```text
linear: (inverse_x + 16384) / 32768
radial: length(inverse_xy) / 16384
stop:   ratio / 255
```

Pad clamps; repeat uses the fractional part; reflect folds a period of two.
Negative coordinates follow those definitions. Original stop order is retained,
including duplicate ratios. Exact duplicates use a right-continuous boundary
choice; the retail boundary convention is unverified. Single-stop gradients are
constant. Reserved modes, empty/out-of-order stop lists and singular matrices
fail explicitly.

Nonsingular focal-radial gradients use the ray from the saved focus to the unit
circle, with the relative radial distance as the parameter. Exact focal endpoints
±1 and out-of-range values return `Unsupported`: no undocumented regularization
is applied. No stroke expansion or morph interpolation is implemented here.

## Explicit color and alpha policies

`Paint` deliberately requires numerical choices. They are infrastructure inputs,
not advertised graphics upgrades or evidence of the original display contract.

- `colors` chooses Linear/raw-unorm or sRGB interpretation of solid/gradient
  colors. A bitmap retains its encoding from `upload_rgba`.
- Saved normal-RGB gradients interpolate encoded stop components before optional
  color decoding. Saved linear-RGB gradients convert, interpolate and convert
  back. `linear_rgb_transfer` explicitly selects piecewise sRGB or power 2.2;
  the located primary documentation does not establish the exact retail curve.
- `gradient_alpha` selects straight-component interpolation or premultiplication
  in the selected interpolation space followed by unpremultiplication. Zero
  accumulated alpha in the latter produces zero RGB. Retail selection is unverified.
- `transform_space` selects arithmetic on shader values or an explicit sRGB
  encode/transform/decode path. Alpha is not gamma transformed.

All paint results supplied to blending have straight alpha. Alpha blending uses
source-over; replacement preserves all four resulting channels. Accumulation
over a transparent clear contains premultiplied RGB, as documented for the quad
readback API. The renderer does not relabel or automatically unpremultiply it.

## Binary masks and clips

Mask parts use triangle geometry only, without color writes or paint sampling.
Stencil Equal(current) plus IncrementClamp makes overlapping pieces a union,
without incrementing repeatedly where they overlap. Nested pushes intersect.
Pop replays the saved triangles with Equal(current) plus DecrementClamp. Mask
scissors are replayed too. Rectangular draw scissors affect only that draw.

These are **binary geometry masks**. Alpha-bearing/cached masks, gradient masks,
filters and undocumented transparent-fill thresholds are not implemented. The
scene owner must report such unsupported semantics. Ordinary vector mask stroke
outlines are not synthesized; stroke-only shapes have no filled mask area.

SWF ClipDepth intervals can cross rather than nest: A at depth1 through4 and B
at2 through6 require A∩B at depths3–4, then B alone at5–6. The scene layer must
pop/replay its active mask set at that transition using B's actual geometry and
world transform. Baking A's coverage permanently into B would be wrong. This
renderer exposes a strict stack and does not interpret ClipDepth itself.

Input/target bounds, finite transforms, index ranges, triangle completeness,
paint bindings, scissors and shader-coordinate arithmetic range are checked.
Per-scene vertex bytes are limited to 256 MiB and the device buffer limit;
commands and newly prepared draw/mask parts are each limited to 65,536. Target
limits and 30-second GPU readback waits match the existing renderer. No MSAA,
coverage antialiasing, hardware performance or GPU-loss recovery is claimed.

## Validation and private evidence

**VERIFIED (2026-11-15):** Offline compilation and 13 focused synthetic GPU groups
passed on llvmpipe (LLVM 21.1.8, 256 bits), Mesa 26.0.8-1ubuntu0.3, Vulkan CPU
adapter. Checks exercise actual triangles and affine placement, exact source
alpha replacement, source-over/channel transforms, bitmap addressing/filtering,
logical-versus-decoded dimensions, rotated fill matrices, gradient modes/spread,
radial/focal parameterization, explicit transfer/alpha policies, mask union and
intersection, crossing-interval replay, scissors, malformed inputs and the
unchanged quad readback API.

**VERIFIED:** All 676 decoded shapes from 21 original movies completed the
offscreen diagnostic batch: 1,117 actual fill groups and 43,599 triangles,
comprising 249 solid, 754 bitmap and 114 gradient fills. All used bitmap inputs
resolve through 729 image/import mappings to 594 original textures. Source
textures explicitly report SRGB=false. No fabricated texture or polygon was
substituted. Eight stroke-only shapes have no fill geometry and are not counted
as implemented stroke rendering.

Each shape was independently fitted into a 128×128 diagnostic viewport using
its actual vertex bounds, preserving triangle geometry. This is not movie layout.
Settings were Linear input/solid colors, Rgba8Unorm target, identity channel
transform in shader space, explicit piecewise-sRGB linear-gradient policy,
straight gradient-alpha interpolation, saved bitmap repeat/smoothing, source-over
and transparent clear. Texture mapping uses the whole-image policy described
above. Of the 676 outputs, 47 have zero alpha: eight have no fills, 35 have only
alpha-zero fills under the identity color transform, and four contain thin
nontransparent details that fall between single-sample pixel rows when fitted
into this small viewport. The four details were subsequently rendered at native
twip-to-pixel scale: all produce nonzero-alpha pixels (1,300, 361, 450 and 580).
Source vertices, UV bounds and alpha diagnosis are retained privately. This
exposes the diagnostic viewport's subpixel-coverage limit; it is not a reason to
alter original geometry or assert visibility in the retail movie.

Representative login bitmap and gradient shape readbacks were converted to
lossless PNG containers and inspected: an original spinner and gradient-backed
shape are present. This visual observation is separate from numeric checks and
does not establish complete UI or retail pixel parity. Raw target alpha/channel
semantics remain unchanged by PNG container encoding.

Focused execution receipts are under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `mesh-harness/`: exact original focused and actual-shape harnesses, lockfile,
  source snapshots and compiled private binaries.
- `mesh-build.log`, `mesh-smoke.log`, `mesh-actual-smoke.log`: build and execution
  receipts, separate from the retained initial shader-validation failure.
- `mesh-thin{,-solid}-smoke.log`, `mesh-validation/native-thin-*.tsv`: focused
  native-scale checks of the four subpixel diagnostic cases, with raw readbacks.
- `mesh-validation/`: numerical check records, all per-shape fill readbacks,
  per-shape receipts and hashes. Images are diagnostic shape viewports, not a
  recovered frontend screen.
- `mesh-assets/`: 21 original movie inputs, actual texture decodes, exact saved
  reference/import/export binding chains, hashes and source snapshots.
- `mesh-reference-files.json`, `mesh-dependencies.json`: exact local reference
  identities and the resolved dependency graph.

Runtime results and their precise limits are recorded in the check manifest.
All game inputs, extracted pixels and captures remain private. A successful
shape render exercises selected settings and inputs; it does not establish
script-controlled visibility, complete text/UI layout, retail sampling/color,
native-frame equivalence or frontend completion.

## Source and licence register

All renderer/shader code and validation logic were authored here. No third-party
application, rendering algorithm implementation or proprietary SDK source was
copied. The integrator maintains the global PROVENANCE.md ledger.

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Exact local `wgpu-27.0.1/src/api/{render_pass,render_pipeline,texture,command_encoder,device}.rs`, `wgpu-types-27.0.1/src/lib.rs`, previous renderer reference register | MIT OR Apache-2.0 | Pipeline/uniform bindings, stencil state, scissor and readback API contracts |
| Original project `gfx.rs`, `gfx/{shapes,tessellate}.rs`, frontend/texture binding evidence and GFX documentation | Original project code/docs; game-derived inputs proprietary | Consume actual meshes, preserve fixed-point metadata and exact bitmap identity; no additional source licence applies to newly authored renderer code |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), pinned `evidence/gfx-fonts/adobe-swf-spec-19.pdf`, MATRIX/CXFORM, ClipDepth, fills and gradients | ©2006–2012 Adobe, all rights reserved; study only | Matrix units, clipped/repeated bitmap behavior, gradient canonical space, saved flags, channel transforms and clipping intervals |
| Adobe AS3 reference hosted by AIR SDK: [Graphics](https://airsdk.dev/reference/actionscript/3.0/flash/display/Graphics.html), [InterpolationMethod](https://airsdk.dev/reference/actionscript/3.0/flash/display/InterpolationMethod.html), [SpreadMethod](https://airsdk.dev/reference/actionscript/3.0/flash/display/SpreadMethod.html), [DisplayObject](https://airsdk.dev/reference/actionscript/3.0/flash/display/DisplayObject.html), [ColorTransform](https://airsdk.dev/reference/actionscript/3.0/flash/geom/ColorTransform.html), [Transform](https://airsdk.dev/reference/actionscript/3.0/flash/geom/Transform.html) | Pages state ©2004–2022 Adobe, all rights reserved | Reference-helper study of smoothing, gradient modes, mask caching and transform composition; exact native rounding/transfer not established |
| AIR SDK [masking guide](https://airsdk.dev/docs/development/display/display-programming/manipulating-display-objects/masking-display-objects); Adobe [AS3 developer guide](https://help.adobe.com/en_US/as3/dev/as3_devguide.pdf) search excerpts | AIR guide prose CC BY3.0, samples MIT (none copied); Adobe guide no open-source grant established | Ordinary filled masks versus outlines/cached alpha masks; full Adobe PDF fetch was size-limited |
| Autodesk Scaleform C++ reference pages `00725`, `04337`, `04345`, `04357`, `04395`, `04404`, [04410](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/04410.html) under `help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/` | Proprietary documentation; no OSS grant established | Helper study of ImageCreator replacement sizing, image matrices, UV generation, padding and subimage offsets |
| Autodesk Scaleform documentation: `gfxexport_reference/usage.html`, `gfxexport2_reference/usage.html`, `flash_support/as3_class.html`, `best_practices/content_creation.html`, `faqs/rendering.html`, `clik/clik_as3_user_guide/art_details.html` under the same host's `scaleform_help/` | Proprietary documentation; no OSS grant established | Rescale/pad/atlas distinction and stencil/gradient-mask support; old pre4.0 FAQ details are not treated as retail facts |

The reference helper performed only prose/API research. The asset helper used
the original project parsers and exact serialized links; its complete source,
licence and input ledger is retained privately in `mesh-assets/README.md` and
`manifest.json`. No game data belongs in source commits or private executable
packages.
