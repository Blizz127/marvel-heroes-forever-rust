# Measured Scale9 geometry and unresolved paint policy

`src/gfx/scale9.rs` supplies bounded triangle deformation for one selected grid
with a positive, axis-aligned local scale. It preserves original geometry and
source coordinates separately. It does not choose grid precedence, execute
scripts, invent replacement rectangles or silently decide the retail paint rule.
The integrator adds the module hook; this increment changes no renderer API.

## Actual resolved first-frame scenes

**OBSERVED:** A private probe loaded the pinned frontend, MarvelGame and frontend
font packages through `frontend::library::MovieLibrary`, with the explicitly
selected English font alias. `gfx::scene::Scene::first_frame` supplied resolved
instances and retained import chains/grid source tags. The probe retained complete
instance paths, local and world transforms, grid rectangles and descendant bounds.

| Scene | Grid instances | Nonunit positive local scales | Unit local scales |
| --- | ---: | ---: | ---: |
| Login | 19 | 15 | 4 |
| Character select | 9 | 8 | 1 |
| Startup manager | 0 | 0 | 0 |

Every grid-bearing instance has exactly one retained grid record, one descendant
Shape, no ancestor or descendant grid, and complete structural bounds. The
accumulated transform from each Shape into its grid-bearing ancestor is exactly
identity. Ten grid-bearing instances also contain text. No standalone Bitmap or
gradient descendant occurs. Twenty-two nonunit cases have bitmap-filled Shapes;
one nonunit character tile list is solid-only. Some Shapes contain several fills.

Five unit local transforms need no local deformation, including two whose world
transforms include enclosing-parent scale. Two unit scrollbar-thumb instances
have splitter X bounds [-25,370] outside the measured Shape X bounds [0,340];
the strict constructor rejects those grids. Their future nonunit behavior remains
unsupported, and the present unit case uses ordinary placement without warping. The other 23 have source geometry
crossing active splitter regions and sufficiently large target extents. Examples
include username/password background scale X ≈ 7.0526, login button X ≈ 2.6346,
and login panel X ≈ 1.0278 / Y ≈ 2.4072. These are saved placements, not evidence
that scripts make every instance visible during actual login.

Bounds were measured from resolved descendant definition rectangles, transformed
into the grid instance's local coordinates. They coincide with the sole Shape's
bounds in this profile. This is a static measurement, not proof of native dynamic
bounds computation for text, filters, strokes or later animation frames.

## Supported helper contract

`GridWarp::new(Identity, Bounds, splitter, local_transform)` requires positive
scales without local rotation/skew, a nonempty interior grid inside the supplied
bounds, and target extents greater than the fixed border sums. `Identity` retains
the scene-node ID and original grid movie/tag. The caller must independently keep
resolved definition/import identity and select one unambiguous grid.

`GridWarp::triangles(positions, source_positions, indices, Limits)` clips each
actual triangle at active grid lines. It emits triangles within individual
regions, preserving interpolated source coordinates separately from destinations.
Input geometry is in grid-local twips; output geometry is in the grid placement's
parent coordinates, including local translation. Apply the enclosing parent's
transform afterward; do not apply the grid placement a second time. Source
values may remain in the descendant Shape's coordinate space. The observed
identity child transforms make that conversion trivial for the measured scenes.

For each axis, let original outer bounds be `[a,b]`, inner splitters `[l,r]` and
scale `s`. Destination outer bounds remain `[s*a,s*b]`; inner splitters become
`s*a + (l-a)` and `s*b - (b-r)`. The two border regions have slope one, and the
middle spans the remaining extent. Local translation is applied afterward. A
unit scale is a direct translation and does not trigger triangle subdivision.
This preserves corner dimensions and the original object origin without using
bounds as substitute geometry. Input curves are already flattened; callers must
choose their tessellation tolerance for the largest resulting region scale. The
warp does not claim to recover curvature lost by an earlier coarse tessellation.

Default limits are 250,000 input triangles and 1,000,000 output triangles. Bad
indices/array sizes, nonfinite values, geometry outside declared source bounds,
resource exhaustion and unsupported constructor cases return errors without a
partial mesh. Undersized, flipped, disabled, out-of-bounds, locally rotated or
skewed grids remain explicit exclusions; native fallbacks are not guessed.

## Paint is a separate, unresolved native contract

**OBSERVED in Autodesk documentation:** Standalone bitmaps are automatically
sliced. Vector shapes with bitmap/gradient fills deform their outlines, while
paint follows an average bounding-box scale. Text scales normally. The guide
allows nested sub-symbols, but does not establish general nested-grid precedence.

The measured descendants are vector Shapes, so blindly using preserved original
source coordinates as their paint coordinates would select the standalone-bitmap
behavior and contradict that documented distinction. The helper returns those
coordinates as information, not as an automatic shader input policy.

`bounds_linear_sources(source_bounds, destination_bounds, positions)` provides an
explicit affine mapping between two chosen rectangles. **INFERRED:** using this
formula as the native average-fill operation. The public documentation does not
specify the precise formula or whether averaging uses the entire Shape, an
individual fill region, or an enclosing symbol. Callers must label that policy,
retain the selected bounds and keep it separate from verified geometry parsing.
No default paint-policy choice is made here.

The renderer already has independent destination and fill coordinates in GPU
vertices, but its current public Draw API derives both from one position array.
The renderer lead was given the measured contract and can add an explicit input
for separate coordinates when integrating. No GPU API was assumed or modified.

The Autodesk guide describes Scaleform 4.5. Matching its complete behavior to the
retail client's actual runtime version and native implementation remains separate
REA/runtime work. Imported-grid overrides, nested-grid precedence, dynamic bounds,
undersized fallback and arbitrary-transform decomposition remain unverified.

## Checks and private receipts

**VERIFIED:** Five focused synthetic tests cover fixed corner widths and origin,
translation, exact triangle splitting, independent source coordinates, unit-grid
behavior, distinct explicit average mapping, rejected unsupported cases, malformed
arrays and input/output budgets. They check area preservation against the known
destination rectangle and ensure triangles do not cross deformation boundaries.

**OBSERVED:** All 23 nonunit instances complete the bounded CPU warp: 34 used
fill meshes grow from 76 original triangles to 452 split triangles. The five unit
cases bypass local deformation. The retained first attempt records the strict
rejection of an out-of-bounds unit thumb grid; no unsupported fallback was added.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `gfx-scale9-probe.rs` / `.tsv`: original scene measurement, full paths, import
  identities, source grids, local/world transforms, bounds and descendant counts.
- `gfx-scale9-child-probe.rs`, `gfx-scale9-child-transforms.tsv`: all 28 exact
  Shape-to-grid identity transforms and definition references.
- `gfx-scale9-smoke.rs`, `gfx-scale9-smoke-scenes.tsv`, `gfx-scale9-smoke.tsv`:
  real triangle-warp receipt, including per-fill counts and numerical difference
  between sliced-source and explicit average-bounds coordinates.
- `gfx-scale9-tests.txt`: focused test output.
- `gfx-scale9-smoke-first-attempt.tsv`: the retained strict-constructor rejection
  that exposed the out-of-bounds unit scrollbar grid.
- `gfx-scale9-evidence.json`: source identities, receipts and supported-case counts.

These are static CPU geometry checks. No native screenshot comparison, shader
paint-policy verification or complete 9-sliced retail border claim is made.

## Focused native follow-up: paint bounds still unresolved

**VERIFIED (REA static inspection):** Two selected-command calls completed with
exit code zero, complete untruncated output and unchanged executable SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The inspected target is the private Win64 build 1.52.0.1700 executable identified
in FINDINGS.md. It was never launched. REA's active target was initially closed;
target-free Rizin commands used explicit byte ranges, without automatic
whole-image analysis, debugger attachment or shared configuration changes.

**OBSERVED (REA):** The complete decoded loader range
`0x14006dc90..0x14006de9c` directly references the `Scale9GridLoader` diagnostic.
It reads a 16-bit resource ID, obtains four floating-point rectangle values,
compares each lower coordinate against its upper coordinate, and leaves without
storing the grid when either dimension is nonpositive. It then looks up the
resource and dispatches masked resource-kind values `0x8400` and `0x8100` to two
storage helpers. C++ class names for those raw kinds are not recovered here.

**OBSERVED (REA):** Complete helpers `0x14006c190..0x14006c212` and
`0x14006c220..0x14006c2cc` allocate a 16-byte rectangle when needed and copy its
four 32-bit coordinate words through pointers at resource offsets `+0x50` and
`+0x58`, respectively. The decoded storage paths contain no outer-shape-bound
comparison or coordinate clipping. This narrows a useful question: loader storage
does not explain away the two observed out-of-bounds scrollbar splitters. Their
later rendering, especially under nonunit scale, remains unverified.

Evidence IDs:

- `ev_661492f76f10ff2863fb3c5b5deef4a0c714fe5e5529890ad6e110a88be3cefb`:
  loader, selected mesh-output ranges and initial property-wrapper candidates.
- `ev_e13ab07788d7163c302122383de4417ef64b8b2f76158c0a1862b756b5c37304`:
  both storage helpers, corrected property-table bytes, the property name and a
  selected getter-wrapper fragment.

The read-only candidate scan's initial loose association of a nearby wrapper
with `scale9Grid` was corrected. REA table bytes establish the two named records
at `0x142520b38` and `0x142520b60`, referencing `0x14027a9c0` and `0x14026def0`.
The nearby numeric wrapper `0x1402686b0` belongs to `scaleX`. The selected
64-byte getter range is only a fragment, not a complete getter implementation.
The setter wrapper's selected indirect-call location lies beyond the file-backed
extent of `.data` and reads as zero in Rizin's static mapping; its runtime
initialization and eventual target were not recovered.

**Stopped search boundary:** The `ShapeMeshProvider` diagnostic led to generic
vertex-output code at `0x1400bff70`, not a demonstrated paint-bounds selector.
The candidate scan followed its two direct-call sites and searched aligned
64-bit pointers in `.rdata` to four exact entries: `0x1400bff70`, `0x1400bdc50`,
`0x1400c4dd0`, and `0x1400c56d0`. None matched. This bounded negative result does
not establish absence of virtual tables or paint transforms elsewhere. The
branch was stopped without broadening into whole-program analysis.

**UNRESOLVED:** No native averaging formula, whole-shape/fill-region/enclosing
bounds selector, or runtime-version proof was recovered. A nearby compatibility
version string does not identify the Scaleform SDK version. The renderer's
paint policy and the explicitly inferred `bounds_linear_sources` status remain
unchanged. Ordinary candidate byte scans are inference/discovery evidence;
only the selected decoded instructions/data above are attributed to REA.

Complete private records, derived selected disassemblies and candidate scripts
are in `evidence/rea-scale9-native/`. `native-summary.json` indexes findings,
coverage and receipt hashes; `final.bundle.json` contains both immutable REA
records. REA's transient investigation was checkpointed and reset afterward,
retaining no active target. The automatic string-inventory warnings about large
skipped regions remain in the records; no complete string inventory is claimed.

## Sources and licences

All implementation is original. No SDK or runtime implementation was copied.
The integrator mirrors these entries into the global PROVENANCE.md ledger.

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Adobe-authored [SWF19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), printed pp. 60–61, DefineScalingGrid | ©2006–2012 Adobe, all rights reserved; document study only | Serialized identity/rectangle, basic fixed-border behavior and disabled-grid conditions. |
| Adobe [DisplayObject.scale9Grid](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/DisplayObject.html#scale9Grid) | Adobe documentation; no open-source licence established | Local coordinate model and distinctions for text/fill scaling. |
| Autodesk [Scale9Grid guide](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/scale9grid.html), [bitmap processing](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/scale9grid/bitmap_processing.html), [transformations](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/scale9grid/transformations.html), [interactive windows](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/scale9grid/interactive_windows.html), [CLIK component scaling](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/clik/clik_user_guide/component_scaling.html), [legal notice](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/welcome/legal.html) | ©2015 Autodesk, all rights reserved; document study only | GFx versus Flash behavior and unsupported/ambiguous cases. No sample implementation copied. |
| Owner's pinned frontend/MarvelGame/fonts packages, existing original library and scene APIs | Proprietary game data for owner-authorized interoperability study; project code original | Actual instantiated grid/transform measurements and CPU geometry validation. No derived data enters git or builds. |

Additional native follow-up sources:

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Owner's private `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`, identity above | Proprietary Gazillion client; owner-authorized interoperability study | Selected static loader/storage instructions, property table and bounded candidate searches. No client code or bytes copied into implementation. |
| Installed `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/native-and-artifacts.md`, `references/evidence-workflows.md`; REA MCP schemas/results | REA guidance version 62; installed toolkit 6.9.0 MIT, no copied implementation | Supported static routing, complete Evidence handling and checkpoint cleanup. |
| Installed Rizin through REA, as documented in REA_NATIVE_ALTERNATIVE.md | Installed tool identified there as 0.9.1, LGPL-3.0-only headers; not linked or redistributed | Selected instruction/data decoding only. REA records themselves leave provider version unknown. |
