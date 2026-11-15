# Native Scale9 geometry and selected-fill mapping

**VERIFIED static native paths and OBSERVED saved assets; conditional contract.**
This increment follows the original saved grid into native display state and
recovers the nine-region point transform, its matrix builder, and the separate
selected-fill mapping. It does not execute the client or establish runtime
rendering parity. Renderer integration remains outside this evidence increment.

The earlier [GFX_SCALE9.md](GFX_SCALE9.md) CPU helper retains its explicitly
conditional scope. Its generic fixed-border formula and paint policy must not
be treated as native proof merely because they produce plausible images.

## Actual saved profiles

The simple selected login-scene case is the compatibility modal background:
`87:compatLevelWarning_mc/1:background_mc/1:<unnamed>`, nodes **129 → 130 → 131**.
It is a later saved display-list candidate; a first-frame inventory does not
prove its actual script-controlled visibility during login.

| Saved element | Exact source fact |
| --- | --- |
| Parent node 129 | Login movie13 sprite172/tag976; translation `(12292,10795)` twips |
| Grid node 130 | Login movie13 sprite73/tag159, placed at depth1; local scale `(737097/65536,122904/65536)`, translation `(-920,-74)` |
| Shape node 131 | Login movie13 Shape72/tag158; explicitly identity child MATRIX, depth1 |
| Shape bounds | `[0,1280] × [0,1280]` twips; DefineShape body `7997+34`, RECT `7999+7` |
| Grid | DefineScalingGrid tag163, record `8053+11`, body `8055+9`; ID73, RECT `8057+7`, splitters X `[371,892]`, Y `[364,896]` |
| Fill | One bitmap fill, type `0x41`, bitmap71, fill MATRIX `20 × identity` |
| Bitmap association | Imported `Tooltip_Main_BG_PNG` → UISource44/tag89, external image 64×64 |

The accumulated saved Shape-to-grid transform is identity; the enclosing parent
is translation-only. Prior complete scene enumeration finds no ancestor or
descendant grid for this instance. The saved affine extent is
`14396.42578125 × 2400.46875` twips. This is an arithmetic fact about serialized
placements, not proof of native target dimensions or effective bounds.

The real login button at node86 is also recorded for comparison: UISource sprite
1039/grid tag10924, Shape1032/tag10834, bounds2080×640, bitmap208 (104×32).
Its container additionally contains text and another group, so the modal is the
narrower geometry profile. Neither profile authorizes dropping its import,
placement, color, mask or source identity.

## Saved grid to native display state

The existing complete loader `0x14006dc90..0x14006de9c` reads the resource ID
and calls rectangle reader `0x140085750`. That reader consumes a five-bit width,
sign-extends the four encoded coordinates, converts signed i32 to f32, and stores
them in Xmin/Ymin/Xmax/Ymax order. There is no factor20 conversion on this saved
wire route. The loader requires positive dimensions before its resource-kind
dispatch. Resource kind `0x8400` stores the four coordinate words through `+0x50`;
kind `0x8100` uses `+0x58`. No outer-shape clipping occurs in those retained
storage helpers.

For the concrete source MovieClip case, factory `0x140178440` calls constructor
`0x1400b6d40`. The constructor retains the source resource at target `+0x110`,
recognizes resource kind `0x8400`, copies its `+0x50` rectangle (or zeros when
absent), and calls **`0x1403cd2a0(T,rectangle)` at 0x1400b6e92**. It then marks
target `+0x140` bit6. This is a saved-data bridge independent of an AVM setter.

The same concrete target table is `0x1425069d0`; slot `+0x88` points to
`0x1403cd2a0`, and `+0x90` points to `0x1400b38c0`. The separate AVM1
`scale9Grid` ID37 property path converges on that `+0x88` slot. Its version,
rectangle-object and coercion gates are preserved in private candidate receipts;
they are not required to pretend the saved constructor used a script setter.

Setter `0x1403cd2a0` reads the prior rectangle via `0x14009bd50` and compares all
four coordinates using ordered equality. It always calls `0x14009bd30`, even
when equal. It updates target `+0x6a` bit0 from its dimension comparisons; only
the virtual `+0x90` invalidation is gated by a changed rectangle. NaNs force the
changed path, and the target flag and state-writer predicates differ for
unordered values. The admitted rendering profile requires finite coordinates.

`0x14009bd30` obtains the render object through `0x14009a760` and tail-calls
`0x1400aaf90`. That writer obtains mutable render data with change mask `0x10000`.
Strictly ordered positive dimensions allocate/copy a rectangle payload whose
coordinates are at `+0x10..+0x1c`, then insert render state **key2** through
`0x1400984c0`. Nonpositive or unordered dimensions remove key2 through
`0x1400986f0`. Read helper `0x14009a590` looks up key2 via `0x1400a94f0` and copies
the payload, returning four zeros if absent. No implicit outer-bounds clipping
is added to these paths.

## Native point transform

Mesh producer `0x1400c5d50` reads its supplied context's `+0x60` pointer. When
present, it constructs a `0x1c0`-byte working record through `0x140137880` and
supplies it to the mesh generation callers. One transform adapter table at
`0x142506f28` has a point method `0x1400bed40`, which tail-calls
**`0x1401333c0`**. The ordinary affine adapter at `0x142506f00` is distinct.
The source record is produced by the hierarchy path below; its caller-supplied
matrix and cache admission state remain explicit inputs.

For distinct coordinate pointers and a supplied initialized record `S`, the
point helper performs three stages:

1. Apply the matrix at `S+0x40`.
2. Compute an outcode against splitter rectangle `S+0x70..+0x7c`, choose a matrix
   at `S+0xa0 + 32*region`, and apply it.
3. Apply the matrix at `S+0x80`.

Every stage uses separate scalar f32 operations, in this order:

```text
next_x = (y * m01 + x * m00) + m03
next_y = (y * m11 + x * m10) + m13
```

Both components use the stage's input pair. This is not a fused operation, and
the documented order must survive an eventual original implementation. Native
runtime MXCSR settings have not been observed.

The raw outcode is `right | bottom<<1 | left<<2 | top<<3`, using strict ordered
comparisons. A point equal to a splitter is in the interior on that axis. The
exact sixteen-byte table at `0x14250a140` is
`[4,5,7,8,3,0,6,0,1,2,0,0,0,0,0,0]`. Thus finite ordered splitters select:

| | Left | Interior X | Right |
| --- | ---: | ---: | ---: |
| Top | 0 | 1 | 2 |
| Interior Y | 3 | 4 | 5 |
| Bottom | 6 | 7 | 8 |

The return value remains the raw outcode, not the region index. Malformed
splitters, aliased mutable coordinates, nonfinite state and unobserved arithmetic
environment are not silently generalized from the selected profile.

## Hierarchy source record

`0x1400b0f10` calls **`0x1400afef0`** with its current node, provider, supplied
matrix, node `+0xcc` selector, output key and stack source record. The latter
enters the grid search only when node `+0x4a` bit7 is set. It starts at the
current node if `node+0x20` is non-null, otherwise at `node+0x30`. It obtains
the corresponding render data through the observed page/entry mapping. This
study does not assign guessed C++ names to those pointers.

At each step it queries render-data state key8. A non-null payload can supply
an alternate parent from its `+0x18` field; sentinel pointer `0xbad` is excluded.
It then queries **key2**. The first key2 record terminates the search. If absent,
it prepends the current render-data matrix at `+0x10` to an identity-initialized
accumulator and advances to the alternate parent if available, otherwise
`node+0x30`. This proves the first-match behavior of this invocation, not a
universal assertion that nested grids are ignored everywhere in the renderer.

On a match, it fills the `0x90`-byte source record `I`:

| Source | Destination |
| --- | --- |
| Key2 payload rectangle `+0x10..+0x1c` | `I+0x10..+0x1c` |
| Matched render-data bounds `+0x70..+0x7c`, before adjustment | `I+0x20..+0x2c` |
| Accumulated matrices below the matched node | `I+0x30..+0x4c` |
| Matched render-data matrix `+0x10..+0x2c` | `I+0x50..+0x6c` |
| Supplied matrix argument | `I+0x70..+0x8c` |

Bounds adjustment `0x1400aa0d0` queries key3. The absent-key, null-payload and
empty-entry cases leave bounds unchanged. Nonempty key3 behavior remains outside
the explicit absent/empty-key3 profile. A successful grid record contributes flag
`0x10`. Producer `0x1400b0f10` skips source allocation/installation when its
candidate already has a non-null `+0x18` context. On the new-context path,
flag `0x10` enables allocation/copy through `0x1400b0930`; `0x1400af120` retains
the result at mesh context **`+0x60`** after the reference-count swap. Allocation
failure supplies null. Cache reuse must not be represented as unconditional
source replacement.

Thus the source of the grid, bounds and matrices is concrete. The origin of the
supplied outer matrix, cache bit7, provider/layer selection, cache reuse and
native render-data bounds construction remain caller obligations. Serialized
Scene matrices are not silently substituted for every native matrix field.

## Matrix inputs and builder

Constructor `0x140137880(output,input,third_matrix)` copies these fields:

| Input | Working record | Observed use |
| --- | --- | --- |
| `+0x10..+0x1c` | `+0x10..+0x1c` | Grid rectangle |
| `+0x50..+0x6c` | `+0x20..+0x3c` | Matrix used by the nine-matrix builder |
| `+0x30..+0x4c` | `+0x40..+0x5c` | Point pre-transform |
| `+0x20..+0x2c` | `+0x60..+0x6c` | Supplied source bounds |

It separately constructs the post-transform using inversions/composition and
the third matrix, then calls builder **`0x140134a10`**. If `G` is the matrix
copied to `S+0x20`, `C` the matrix at `S+0x40`, and `M` the third argument,
the post-transform is `M * inverse(C) * inverse(G)`, using the native helper
orders below. The ordinary mesh producer supplies `I+0x70` as M; another branch
uses identity and has additional behavior. The matrix names describe observed
roles, not guessed parent/local/world identities.

The builder forms nine source/destination triangle pairs and invokes affine
builder `0x140079330` for each. It has native behavior beyond the earlier
strict CPU helper: a `0.9f32` bounds adjustment, an exact-zero transformed
edge-length replacement of `0.001f32`, and an undersize branch that divides
opposing border ratios by their sum plus exact `0.05f32` when the sum exceeds
one. These constants are direct file-backed REA reads. Edge-length calculation
calls the executable's `MSVCR120.dll!sqrtf` import; no loaded-CRT or native
floating-environment execution was performed.

For the modal's saved values with an explicitly supplied local matrix in the
builder's matrix field, an independent instruction-derived scalar trace gives
top-left triangle `(0,0),(371,0),(371,364)` mapping to
`(-920,-74),(-549,-74),(-549,290)`. The center destination triangle is
`(-549,290),(13088.42578125,290),(13088.42578125,1942.46875)`.
This is a conditional arithmetic check; it does not prove the complete native
cache invocation and outer-matrix inputs for an executing instance.

The affine helper contracts are also retained. `0x1400378d0` makes an eight-f32
copy before calling `0x140036ee0`, so in-place inversion does not rely on unsafe
aliasing. The determinant is separately rounded `d*a - c*b`. Exact ordered
zero produces an identity linear part with negated translation; it does not
return an error or a zero matrix. The nonzero/unordered branch uses reciprocal
determinant and preserves destination's unused lanes `+8/+0x18`.

Composition `0x140032970(A,B)` writes `B*A`, with separate SIMD multiplies,
addition of the two products, then addition of B's translation; masks force
the unused third column to positive zero. The three-point helper writes
`destination_basis * inverse(source_basis)`. These finite operations introduce
small real differences from ideal piecewise formulas: the modal's top-right
piece has both diagonal coefficients `0.9999999403953552`, and the bottom-left
piece's X coefficient is `1.0000001192092896` in the explicit supplied-input
trace. Replacing native matrix construction with exact unit border slopes would
erase those differences.

## Path geometry and output coordinates

The complete selected path consumer **`0x1400c3d30`** applies transform adapter
virtual `+8` to the path start/end/control points before tessellation. Its
straight-edge branch transforms the endpoint and emits it directly. The two
curve branches transform their control/end points before the corresponding
curve helpers. This does not establish grid-edge subdivision of finished
triangles. The earlier CPU helper's post-tessellation triangle clipping must
remain separate from the observed native path algorithm.

The result of `0x1401333c0` is an intermediate coordinate stage. In the selected
nonnull-grid caller `0x1400c4dd0`, a copy of mesh-context matrix `+0x70` is passed
to generic output helper `0x1400bff70`. Its retained prefix computes that
matrix's inverse and supplies it to the tessellator interface virtual `+0x38`
at `0x1400c0203`, followed by virtual `+0x40`. Concrete virtual implementation,
later vertex normalization/quantization and final paint-coordinate association
remain explicit downstream obligations. The grid output must not be sent
directly to a renderer under an invented final coordinate-space assumption.

## Geometry and paint use different mappings

The separate fill method **`0x1400c2f00`** invokes **`0x1400c2bb0`** when its
context has a `+0x60` record. The latter enumerates paths for the selected layer,
accepts paths whose left or right style matches the supplied fill-style ID,
and accumulates their geometric bounds through `0x1400c0cc0`. Positive bounds
produce three corners: `(xmin,ymin)`, `(xmax,ymin)`, `(xmax,ymax)`.

It constructs a working grid record with an identity third matrix, transforms
those three corners through `0x1401333c0`, then creates one affine mapping from
the original triangle to the transformed triangle using `0x140079330`. The
caller inverts that mapping and composes it into the fill matrix. It can then
compose a further image-resource matrix through an indirect method. Thus the
observed vector-fill path does not simply preserve each region's original UVs.

Selection helper `0x1400bea80` uses a 20-byte provider layer entry. In the
non-stroke branch, `layer.base_fill + subfill` indexes a provider fill-ID table;
the resulting nonzero ID is passed to the shape's fill reader and to the bounds
matcher. It is the native fill ID, not the zero-based subfill index. Zero returns
null without supplying an ID. Provider table contents must remain explicit.

The modal's own saved shape stream independently decodes to right fill1,
left fill0, no line style, and straight endpoints
`(0,0) → (1280,0) → (1280,1280) → (0,1280) → (0,0)`.
On the selected native start/type1-line path, the bounds accumulator transforms
each endpoint by its supplied matrix and expands component minima/maxima.
The bounds caller supplies identity, so these finite decoded endpoints yield
`[0,0,1280,1280]` under the explicit matching-fill/path-input profile. This
cross-check does not pretend a live provider table or native path reader was
executed. Curve extrema and morphing behavior remain outside the simple profile.

Live provider fill/layer binding, morphing, the image-resource matrix and final
output coordinates require their respective
contracts before renderer admission. A single-fill rectangle is a useful
bounded case, but the native source bounds are not replaced automatically with
the authored DefineShape bounds. General text, standalone Bitmap, strokes,
filters, nested grids and rotated hierarchy remain separate cases.

## Validation and remaining work

The saved-profile receipt passed 26 cross-checks against independent source
parsing, the retained scene probe and renderer audit, including full hashes of
both movies. Native captures use target-free REA Rizin with exact byte ranges
against the pinned owned executable. Independent comparison checks instruction
bytes and data against the PE section map; no configuration changes or native
execution occur.

All **11 new command records**, comprising **4,547 instructions / 20,270 bytes**
and **328 data bytes**, completed with untruncated output and unchanged artifact
digests and were independently byte-matched. The separate pure PE import record
was validated as a descriptor/ILT/IAT binding, without inventing command-result
fields. Independent checks cover all nine point regions, splitter equalities,
noncommuting transform stages, f32 rounding, four builder branch profiles and
the matrix helper's singular/lane behavior. These are static/conditional checks,
not native execution or rendered-image comparison.

The conditional point/matrix contract is grounded. Before enabling a rendered
profile, bind the native caller/cache inputs to the actual scene, complete the
selected fill/bounds and output-coordinate dependencies, and retain explicit
unsupported profiles. Inherited first-match traversal is observed; full nested,
alternate-parent and rotated rendering are not inferred from the existence of
general matrices. No central renderer, VM, asset or option file is changed here.

## Private evidence and source ledger

Evidence root: `~/.local/share/marvel-heroes-forever-rust/evidence/scale9-rendering/`.
Original byte-derived scripts and outputs remain private. Outside implementation
source is not used or copied.

| Source | Licence / purpose |
| --- | --- |
| Owner's `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | Proprietary, compatibility study. SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`, 48,544,080 bytes; selected native instructions, tables, literals and PE imports. |
| Owner's pinned login/UISource movies and their existing original scene/asset readers | Proprietary asset study / original project code. Exact files, complete hashes and every selected tag span are in `saved-profile/source-ledger.json` and `selected-profiles.json`. |
| `evidence/rea-scale9-native/`, `gfx-scale9-*`, and `runtime-text-visibility/scene/path-audit/scale9-context/` | Existing original project evidence; reuse loader/storage proof, actual grid census and exact saved modal record. |
| `evidence/rea-asflags-followup-version-proof/final.bundle.json`, `rea-attached-name-map/`, and `textfield-runtime/` | Existing original project evidence; source factory/constructor, canonical names and render-object access. |
| Installed REA skill v62 and tool schemas; REA6.9.0/Rizin | Workflow/instruction decoding only; no toolkit implementation copied or linked. |

Current primary capture files are `rea-target-grid-setter.json`,
`rea-render-grid-state.json`, `rea-region-transform.json`,
`rea-grid-constructor-and-mesh.json`, `rea-nine-matrix-builder.json`,
`rea-fill-matrix.json`, `rea-wire-rect-and-constants.json`, and
`rea-pe-imports.json`, together with `rea-path-geometry-and-fill-bounds.json`,
`rea-mesh-adapters-and-matrix-helpers.json`, `rea-matrix-data-masks.json` and
`rea-hierarchy-source-producer.json`. Candidate discovery is separate from canonical REA output.
`native-validation/` retains byte comparisons and independent point-formula
checks; `builder-review/` retains conditional scalar traces with explicit
supplied inputs and unresolved runtime assumptions.

The complete 45-record retained bundle is `complete.bundle.json`, SHA256
`aa2312886968f50ac32f85a5ded9fde9bf7493a7eb9b4ef046ebacc42ba85ae4`.
The aggregate `verification.json` indexes all twelve new Evidence IDs and
selected independent receipts; SHA256
`26471b47c3cb30e66cf7cc51d6145f5e61a3aed7bed199cde027ac2144d78b78`.
Key new native records are:

| Evidence ID | Boundary |
| --- | --- |
| `ev_7203112f5d38cae105cfb5fa1ec75b0b9c6620c24c2c1f5066879613a81c597f` | Display target grid setter |
| `ev_3473d9e60b57673316f19299ea3784726dfaba7616e98071694845ac3eab8bc1` | Render state key2 storage/readback |
| `ev_260acd797ae6347aaf6dfedbc59dfc200c37fc16cbfe19f4eea9ca698e72a9e7` | Nine-region point transform and exact table |
| `ev_41541b731925019e7936453c3c357a61e4cab53cca8dd47000dfa55e4ad6ad08` | Source constructor, mesh entry and selected-fill affine |
| `ev_99ca396fbf3596b61399dffcb92e4ccfda3b7fb4eb9790f4051e9371fa7d5b99` | Nine-matrix builder |
| `ev_92580983e3680646ac72fe2cb956f9ad636740ff173cad53ed13c651318d5fd3` | Fill-matrix composition |
| `ev_83b9fa38f50a20412d699b85d875d6323028f8563798a33f798fbc7f5fb1f7a2` | Path geometry, selected fill identity and bounds |
| `ev_2606a5cd4f58f08e63076faf650fa6b57c08b665233ffe0f07209b3cf5b1be5c` | Mesh adapter callers and matrix helpers |
| `ev_750ae0b66891e3b33fa03caa2dd4ad99b89f49924335faf744f3b1b1019559ce` | Hierarchy source producer and installation |

The capture index also retains the separate wire/constants, correctly mapped
`.data` masks and pure PE import records. No target remains open; needed REA
Evidence was exported and the resource close completed successfully.

## Original executable arithmetic primitive

`gfx::native_scale9` implements the recovered point, inverse, composition,
three-point and nine-affine builder contracts. Its inputs are explicit native
record roles. It does not select a scene ancestor, infer source bounds, resolve
cache freshness, convert units, split triangles or choose image coordinates.
The existing renderer and legacy CPU clipping helper are unchanged.

The checked public API is:

| API | Contract |
| --- | --- |
| `Matrix2x4::from_rows` | Native `[a,c,unused,tx]`, `[b,d,unused,ty]` layout; all eight lanes checked |
| `RectF32::from_coordinates` | Ordered finite `[xmin,ymin,xmax,ymax]`; equality allowed for supplied point contexts |
| `Matrix2x4::transform_point` | Exact recovered separate product/sum order |
| `inverse_into(destination)` / `inverse()` | Native zero-determinant fallback and unused-lane behavior; value semantics avoid pointer aliasing |
| `left_compose(left)` | `left * self`, including translation-add order and positive-zero lane masks |
| `three_point_affine(source,destination)` | Exact basis/inverse/composition; no idealized fixed-edge slope substitution |
| `NativeScale9Context::from_parts` | Explicit pre-transform, splitter, nine matrices and post-transform |
| `context.transform_point` | Position plus both raw native outcode and selected region index |
| `build_finite_ieee_nearest(SourceInputs)` | Callable nine-affine construction using standard `f32::sqrt` and the recovered bounds/length/undersize branches |

`SourceInputs` supplies `grid`, `source_bounds`, `grid_matrix` (G),
`shape_to_grid` (C) and `outer_matrix` (M). C's name identifies its observed
accumulator role; it does not prove that an arbitrary Scene matrix is suitable.
The resulting context preserves the original grid as its splitter and computes
the post-transform as `M * inverse(C) * inverse(G)`. Accessors expose the explicit
context components for audit and later integration.

The builder admits positive grid dimensions and ordered source bounds, including
source bounds equal to the grid. It preserves the native upper-bounds repair
using the lower grid coordinate, exact-zero length replacement and undersize
normalization. These paths are not silently replaced with a conventional
nine-slice policy. All operations have fixed storage bounds: two-row matrices,
nine regions and a four-by-four temporary point grid; there is no allocation or
asset-derived loop count.

### Floating-point admission and limits

The explicitly selected profile is **finite IEEE binary32 nearest rounding**.
Arithmetic results use separate f32 operations, with no fused multiply-add,
combined transform or algebraic reassociation. Standard `f32::sqrt` makes the
builder callable; exact agreement with the unobserved loaded retail
`MSVCR120.dll!sqrtf` and its MXCSR remains **unverified**. The implementation does
not inspect or modify host floating-environment state. A caller that changes
rounding mode has not established this profile.

Nonfinite values, subnormal inputs and subnormal-sensitive operations return
typed errors. The latter admission also rejects tiny exact nonzero results
that might round or flush to zero. A wider scalar calculation is used only to
check that magnitude before each operation; it never supplies the f32 result.
This is a deliberately narrower supported profile than all possible native
bit patterns. No epsilon is introduced into native determinant or length tests.
Positive and negative zero remain distinct except where the recovered native
instruction order/masks change them. Unused lanes are checked and their native
calculation/masking behavior is retained, rather than silently accepting an
unproved floating-exception profile.

### Focused implementation checks

The standalone module's five synthetic checks cover all nine regions and exact
splitter equalities, noncommuting stages, a cancellation that distinguishes
separate multiplication from FMA, singular inversion, destination-lane retention,
composition order, a hand-checkable two-unit-border builder profile, and rejected
nonfinite/subnormal-sensitive inputs. Original asset bytes and actual modal
fixtures stay outside git. These are arithmetic checks, not native execution or
live pixel comparisons.

The independent private oracle was frozen before its author inspected this
module. It reuses the retained instruction-derived builder traces and separately
derived matrix/point contracts. Original helper formulas match all **432** source
and destination scalar bits across four retained builder profiles. The actual
modal profile, quarter-scale undersize case, zero-horizontal-length case and
equal-bounds repair case remain explicit supplied-input witnesses; no executing
native cache invocation is claimed.

**VERIFIED conditional math replay:** the private harness passes **37/37** checks
in both debug and optimized Rust builds. Its exact-bit comparisons cover all
36 builder matrices, 64 point probes, 16 matrix-helper cases, 36 three-point
affines and 108 direct triangle-point evaluations. The total includes the five
module checks and two additional Rust admission checks. The 108 point witnesses
retain the native-operation rounding differences from ideal triangle endpoints;
55 endpoints differ in at least one bit. They are not rounded back to an ideal
geometry oracle. Standard host sqrt produces the retained builder outcomes for
these four cases; broad loaded-CRT equivalence remains unverified.

Additional original evidence lives under
`~/.local/share/marvel-heroes-forever-rust/evidence/native-scale9-math/`:

| Private source | Purpose / provenance |
| --- | --- |
| `builder-contract/CONTRACT.md` and `formula-comparison.json` | Original formulas derived from retained owned instructions; no outside implementation source |
| `oracle/point-builder-oracle.json` | Frozen builder/point expected bits, independently prepared before product-source inspection |
| `oracle/matrix-independent/matrix-oracle.json` | Original exact rational binary32 rounding oracle for matrix helper cases |
| `oracle/verification.json` | Debug/optimized focused-check commands, results and unchanged implementation/oracle hashes |

The original retained inputs are `scale9-rendering/builder-review/scalar-trace.json`
(SHA256 `3473bcb64c633c6dcced9c4c921fdff0af649039a539c6e82c0a7f02eef6baa5`)
and `scale9-rendering/matrix-helper-review/scalar-comparison.json`
(`9ee2e152c361aeee957f37928d9409bd3650ad4f43193bd1bf0583a2dd6fa718`).
The frozen point/builder oracle is
`29389f0666fa7eaa4175bc9e5ce55a0f170743675a463aae53b92268c205dae0`;
the matrix oracle is
`da31e8936e60fc23c1671c10106c1532914f09e9ca4759e6bdc2c9b1925ed46d`.
All source licensing remains as recorded above: owned proprietary files for
compatibility evidence, original project arithmetic/checking code, no copied or
linked outside parser/renderer implementation.
