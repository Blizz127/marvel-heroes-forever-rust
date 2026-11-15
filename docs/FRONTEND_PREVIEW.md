# Saved scene geometry preview

Task `a3d58c579542`. `frontend::preview` consumes the owned `PreparedScene` CPU
snapshot and draws its actual fill meshes in saved display order. It does not
interpret ActionScript, change saved visibility, synthesize controls/backgrounds,
connect to a server or declare the frontend complete.

## API and policy

The integrator adds the optional renderer module hook in `frontend.rs` and the
diagnostic CLI. This module adds no crates and performs no filesystem, network
or display access:

```rust,ignore
let frame = preview::render_static(&quad_renderer, &prepared, root_stage, policy)?;
// frame.readback, frame.coverage and frame.limitations belong together.
```

`render_static_with_limits` also accepts explicit node, command, mask traversal,
mask part, uploaded-byte, text-glyph placement and owned text-geometry budgets. `DiagnosticFrame` contains the RGBA
`Readback`, original stage rectangle, selected `PreviewPolicy`, any saved root
background applied, submission coverage and typed limitations. Coverage means
geometry submitted; it is not a visible-pixel or completed-feature count.

The target size is `ceil(stage twip extent / 20)` in each direction. The stage
minimum is subtracted and saved world transforms map local twips directly into
physical target pixels. There is no bounding-box fitting or invented geometry.
Color additions from the scene's 0–255 channel units are divided by 255 before
the mesh shader. Child arrays must be in ascending display depth; parent links,
reachability, duplicate/cyclic nodes and mask identities are checked.

The caller explicitly selects:

- `background`: transparent, or the saved root background tag with transparent
  fallback. The tag's RGB uses the selected source-color interpretation. No
  guessed game/world background is added.
- `colors`, `color_transform_space`, `linear_rgb_transfer`, `gradient_alpha`:
  the mesh renderer's documented numerical policies, without a retail claim.
- `bitmap_mapping`: omit unverified mappings, or select whole-image replacement
  and record every logical/physical dimension pair. Padding/atlas/runtime image
  matrices are not inferred from dimension differences.
- `unknown_texture_encoding`: omit images with absent SRGB metadata, or use an
  explicitly selected encoding and report that choice. Present true/false
  properties select sRGB/Linear uploads directly.
- `effects`: omit affected subtrees, or render saved base geometry while retaining
  filter, blend, scale-nine, caching, opaque-background and morph limitations.

Images follow the stable prepared bitmap-to-texture indices and original
character IDs. Actual RGBA is uploaded once per used texture. Missing data or
unknown encoding produces an explicit omitted fill, never a substitute image.
Solid/gradient meshes and direct Bitmap nodes are not converted to guessed
textured rectangles. Direct bitmap geometry remains omitted. Supported saved
plain EditText consumes the CPU bridge's actual font glyph meshes and placements;
unsupported text retains its typed layout failure.

The scene keeps original asset issues, strokes, inferred winding, script
bindings and opaque tags. Those appear in the frame's limitations even where
base geometry can be drawn. Saved transforms do not establish script-controlled
runtime state or intermediate channel-clamping behavior.

## Saved plain text

Text preparation stays in `scene_content`; the GPU adapter performs no font
lookup, text parsing, reflow or fallback. Each prepared glyph keeps its font
identity, glyph index and EM coordinates. The local EM-to-field matrix composes
with the saved node world matrix and stage mapping. No DefineText TextMatrix is
applied to DefineEditText.

The CPU cache's neutral white paint is replaced by the glyph's saved RGBA before
the selected source-color interpretation and world color transform. Folding
text RGB into world multipliers would incorrectly reorder additive/transfer
operations and is not used. Small owned draw meshes are cached by glyph asset
and saved color; original contour/source records remain in `PreparedScene`.
Missing paint or glyph geometry is an explicit skipped placement.

Each field's exact saved TextBounds becomes a two-triangle stencil clip under
the same world transform, including rotation/shear. This source-defined
rectangle is used only for clipping; it is never a visible control or invented
background. The field clip nests inside the scene's validated masks and is
popped before the next node. Unsupported scene masks still omit affected text.
A field requiring more than the stencil's 255 active levels is omitted with a
typed reason. Zero-area bounds yield empty coverage. Overflow is retained and
clipped, never resized to fit.

Coverage separately reports prepared/empty/omitted text instances, requested/
submitted/omitted/empty-geometry glyph placements, text fill draws/triangles and
fields actually clipped. Aggregate fills/triangles include both shape and text
draws; shape instance counts do not include text. Space glyphs can have no fill
geometry; a valid empty initial field draws nothing without being treated as a
layout failure. Only a field actually clipped discharges its source
`TextBoundsClippingNotApplied` limitation.

Font-binding/font/glyph/text preparation issues, typed layout failures and
overflow flags all survive in `DiagnosticFrame.limitations`. Saved-state-only,
field-border, font-rasterization and selected glyph-fill-rule limitations
remain. HTML, passwords, required wrapping, unsupported static text and missing
font bindings receive no substitute text. There is no input editing, caret,
selection, localization, VM string update, font hinting or retail AA claim.

## Mask handling

Each ordinary draw requests its exact ordered active mask NodeIds. The adapter
keeps the shared prefix, pops obsolete masks and pushes the required independent
geometry. When A ends before B, B is replayed without retaining A's clipping.
Mask-source subtrees are not drawn as color content.

Supported masks are filled opaque-solid geometry, with no unresolved metadata,
filters, nonnormal blend, cache, scale-nine, morph, opaque-background or hidden
state on the mask/its ancestors. Internal masks inside a mask subtree and
non-shape mask content are unsupported. Bitmap, gradient and nonopaque mask
paints are rejected rather than assigned guessed alpha semantics. The ordinary
mesh stencil union/intersection implementation remains responsible for coverage.

If a mask cannot be represented, every affected draw is omitted with the source
mask and rejection reason. Its content is never rendered unmasked. The renderer
does not interpret ClipDepth anew or bake earlier sibling-mask coverage into a
mask that must later stand alone.

## Focused validation

**VERIFIED (2026-11-15):** Offline build and five focused synthetic GPU groups
passed on llvmpipe (LLVM 21.1.8, 256 bits), Mesa 26.0.8-1ubuntu0.3, Vulkan CPU
adapter. Tests cover nonzero stage minima and ceil dimensions, world placement,
depth order, additive channel units, recorded-background selection, crossing
mask replay, unsupported masks omitting affected content, inherited effect
omission, retained text/bitmap/stroke gaps and invalid trees/budgets. The test
process selected the installed software ICD and two llvmpipe worker threads.

**VERIFIED:** Three saved scenes were composed at their native 1920×1200 stage
extent using the original pinned frontend, MarvelGame and MarvelFonts packages.
The private inspection library explicitly aliases `MarvelFonts.gfxfontlib` to
`MarvelFonts.fonts_en`; this is an inspection selection, not a recovered native
locale rule. This first checkpoint predates the saved-text increment below.

| Saved movie | Nodes | Shape instances submitted | Fills / triangles | Textures | Visible Text nodes omitted |
| --- | ---: | ---: | ---: | ---: | ---: |
| `marvel_login_dialog` | 147 | 39 | 64 / 160 | 29 | 33 |
| `marvel_character_select` | 264 | 87 | 90 / 188 | 22 | 17 |
| `frontend_manager` | 6 | 1 | 1 / 2 | 1 | 0 |

All 50 instantiated text definitions in these snapshots are actual DefineEditText
tag37; none is DefineText/DefineText2. The three snapshots have no active masks,
so real masking is not validated by these frames. Login retains 22 filtered
nodes and 19 scale-nine nodes; character select retains 16 filtered nodes, nine
scale-nine nodes and a blend-mode8 group. Those behaviors are not applied under
the chosen `BaseGeometry` policy.

The actual composition checks selected Linear source colors, an Rgba8Unorm
target, shader-space color transforms, piecewise-sRGB linear-gradient transfer,
straight gradient alpha, whole-image replacement, no absent-SRGB fallback and
saved root backgrounds. The saved clears are `[102,102,102]` for login/character
select and `[255,255,255]` for the manager. These are authored stage colors,
not asserted to be the game's final display background.

**OBSERVED:** The inspected login capture contains original logo, panel and
button art, without the omitted text or runtime visibility/effects. Character
select is uniformly black and the manager is uniformly white under these
recorded policies; their submission counts must not be presented as completed
visible interfaces. No visibility/color values were changed to improve the
picture. Source-only investigation records the character background's saved
zero RGB multipliers and the manager cursor's offstage placement/nontransparent
pixel extent. Unexecuted scripts and the unsupported blend/effect policies
prevent a retail-behavior conclusion from these saved snapshots.

Private evidence beneath
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `frontend-preview/{login,character-select,manager}.{rgba,png}`: raw composited
  target bytes and lossless PNG containers. PNG encoding does not edit the pixels.
- `frontend-preview/*-coverage.txt`, `image-hashes.json`, `checks.json`: selected
  policies, all typed omissions, coverage and exact input/source/output identities.
- `preview-harness/`: exact original harnesses, dependency lockfile, stable CPU
  source snapshot and native binaries. The snapshot's module wiring is private;
  no root CLI/library hooks were edited by this lane.
- `preview-build.log`, `preview-actual-smoke.log`, `preview-checks-build.log`,
  `preview-checks-smoke.log`: focused compilation and execution receipts.
- `scene-preview-inputs/`: source-only recipes, movie/instance/text/mask profiles,
  original-source hashes, unsupported raw metadata and blank-frame investigation.

No native window, interaction, authentication, server/world connection,
physical GPU, retail screenshot comparison, effects, scale-nine, scripts
or runtime manager scheduling is validated by the base checkpoint.

## Saved-text increment validation

**VERIFIED (2026-11-15):** The separate saved-text adapter build and all seven
focused synthetic GPU groups pass on the same software Vulkan adapter. Checks
cover saved RGBA before world multiply/add (including alpha addition), rotated
and nonzero-origin field bounds, intersection with a source scene mask,
rejection of an unsupported mask, valid empty fields versus unsupported empty
HTML, missing paint/failed/empty glyph geometry, every text/font issue channel,
invalid cache/layout identities and text geometry/placement budgets.

**VERIFIED:** Fresh original-package snapshots retain the earlier explicit
English alias and numerical policies. Shape submission counts are unchanged.

| Saved movie | Text instances | Prepared (empty) | Omitted | Glyph placements / submitted | Text fills / triangles | Fields clipped |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `marvel_login_dialog` | 33 | 27 (5) | 6 | 243 / 232 | 232 / 95,848 | 22 |
| `marvel_character_select` | 17 | 10 (3) | 7 | 101 / 92 | 92 / 50,760 | 7 |
| `frontend_manager` | 0 | 0 (0) | 0 | 0 / 0 | 0 / 0 | 0 |

The omitted login instances retain HTML-layout failures; character select
retains HTML and required-wrapping failures. Eleven login and nine character
placements have no fill geometry. Submitted glyph counts mean submitted source
triangles, not necessarily visible characters after world alpha/depth.

**OBSERVED:** The login image now includes original saved labels, including
authored placeholder strings, and differs from the earlier frame in 8,600
pixels. Nothing replaced those strings with guessed production labels. The
character-select and manager outputs remain byte-identical opaque black/white
to the base checkpoint. Their actual saved color/placement source diagnosis
remains in `scene-preview-inputs/blank-frame-diagnosis.json`.

Private evidence adds `frontend-preview-text/{login,character-select,manager}`
RGBA/PNG and coverage files, `image-hashes.json`, `checks.json`,
`preview-text-harness/` (exact sources/lockfile/binary),
`preview-text-{build,build-final,actual-smoke}.log`, and
`preview-text-checks/` (seven-group source, immutable input identities, build/
smoke receipts and result). The login RGBA SHA256 is
`16216eaf05514d9ee9b4b5d343ee253f469ec4f5080c283be71288f03557edb7`.

These checks validate saved geometry/paint/clipping composition only. No native
text renderer comparison, physical GPU, interactive text entry, runtime script
state, effects, scale-nine or retail frontend parity was run for this increment.

## Source and licence register

The adapter and private checks are newly authored. No additional external
implementation or proprietary SDK source was used.

| Source read | Licence / provenance | Use |
| --- | --- | --- |
| Original project `frontend::{library,scene_content}`, `gfx::{scene,shapes,tessellate,text,glyphs,edit_layout}`, `texture`, `render::{mesh}` and their component docs | Original project code; external documentation already registered in those components | Respect stable asset IDs, scene traversal/transforms/limitations, exact mesh paint and readback contracts; saved glyph/layout paint and exact clipping contracts; no GPU-module font parser added |
| Private pinned frontend/MarvelGame/MarvelFonts packages and original preparation/profiling evidence | Owner-provided proprietary game data; authorized compatibility study | Actual saved scene composition, serialized background/visibility/transforms, exact texture references and source-only uniform-frame diagnosis |
| `scene-preview-inputs/{recipe.json,manifest.json,README.md}` and immutable original source snapshot | Original helper preparation/provenance, with proprietary inputs retained privately | Reproduce the three structural snapshots and inspect actual Text/mask/effect coverage without guessing missing content |
| Previous `docs/RENDERER.md`, `docs/RENDERER_MESH.md` source/SDK register | wgpu MIT OR Apache-2.0; Adobe/Autodesk documentation terms recorded there | Existing device/readback and explicit stencil/color/UV limitations; no new library or algorithm implementation copied |

The global provenance ledger and root optional CLI/module hooks are maintained
by the integrator. Game files and these rendered captures never enter source
commits or executable packages.
