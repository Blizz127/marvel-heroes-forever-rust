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

## Native output coordinates and packing

**VERIFIED static instructions; conditional finite geometry profile.** The
ordinary filled-shape route has a concrete output-coordinate stage after the
Scale9 point helper. The presence of its mathematical contract does not prove
that an executing modal selected this provider, layer, cache or output receiver.
No renderer code or runtime capture is changed by this evidence increment.

The selected chain is `0x1400c5d50 → 0x1400c4dd0 → 0x1400bff70`.
The first caller's layer entries select the ordinary branch when the relevant
`+0x10` and `+0x0c` selectors are zero; alternate branches remain distinct.
On the non-null Scale9-context branch, `0x1400c4dd0` copies the context matrix
`+0x70..+0x8c` to a local matrix. It passes that matrix and the primary tessellator
to `0x1400bff70` at `0x1400c54aa`.

The tessellator is tied to actual construction, not a nearby guessed table:

| Constructor / caller | Observed relationship |
| --- | --- |
| `0x140071960` | Calls `0x140070bf0` with enclosing object `+0x58` |
| `0x140070bf0` | Calls `0x1400ff0a0` with workspace `+0xa0` |
| `0x1400ff0a0` | Installs table `0x1425096d0` at the constructed object |
| `0x1400c4dd0` | Supplies `[context+0x50]+0x58+0xa0` to the output stage |
| Table `+0x38` | `0x1401010d0`, vertex transform and extrema update |
| Table `+0x40` | `0x140103270`, normalization and inverse-matrix result |
| Table `+0x70` | `0x140100090`, unchanged selected-vertex export |

### Inverse outer transform and normalization

At `0x1400c01f4`, the output stage computes the inverse of its supplied matrix
using the already recovered copy-inverse helper, with an identity-initialized
destination. The call at `0x1400c0203` passes this inverse to tessellator `+0x38`.
The concrete method transforms every stored output vertex's X/Y and recomputes
extrema at tessellator `+0x214..+0x220`. Vertices have 20-byte records, in pages
of sixteen, with count `+0x320` and page table `+0x338`.

Its scalar order is `x'=(x*a+y*c)+tx`, `y'=(y*d+x*b)+ty`, each operation f32.
Extrema begin at exact bit patterns `0x7149f2ca` and `0xf149f2ca`; ordered strict
comparisons update them. This stage must not be algebraically cancelled against
an earlier outer transform: the already rounded vertex values are its inputs.

At `0x1400c0233`, tessellator `+0x40` receives destination bounds
`[-32764,-32764,32764,32764]`, exact file-backed f32 constants. It initializes
the output matrix to identity. For finite inputs, invalid/degenerate cached
extrema trigger a fresh scan; normalization proceeds only when both final
extents are strictly positive. Zero width or height leaves vertices unchanged
and returns identity. Thus the call's destination bounds do not themselves
establish a universal coordinate bound.

On the positive-extent branch, it forms source corners TL/TR/BR from the actual
stored extrema and corresponding destination corners from the supplied bounds,
then calls the recovered three-point affine builder. It applies this matrix to
all vertices with `x'=(x*a+y*c)+tx`, `y'=(x*b+y*d)+ty`, preserving the separate
operations. Finally it copies and inverts that forward matrix into the result.
The method does not update its extrema to the normalized destination rectangle.

The selected export method `+0x70` copies all five words of matching 20-byte
vertex records unchanged into the caller's buffer, up to its limit of 256.
It selects records using the descriptor and stored per-vertex ID and advances
the descriptor cursor. There is no further coordinate transform between this
export and the following packing loop.

### Packing is not generic rounding

At `0x1400c0590..0x1400c05d9`, each exported coordinate `v` follows:

```text
adjusted = v < 0 ? f32(v - 0.5) : f32(v + 0.5)
floored = imported MSVCR120.dll!floorf(adjusted)
integer = CVTTSS2SI(floored)
stored_word = integer's low sixteen bits
```

The constant is `0x3f000000`. Thunk `0x141c6f890` binds through IAT
`0x141dbbfd8` to the PE import `MSVCR120.dll!floorf`; this is a static binding,
not an observation of the loaded DLL. The output vertex stride is twelve bytes;
X and Y occupy its first two 16-bit words. There is no saturating conversion on
this path. Finite valid-i32 conditional examples include `-1 → -2`, `+1 → +1`,
and both signed zeros taking the addition branch and producing zero. Pre-floor
f32 rounding matters at adjacent negative half-unit inputs.

Even strictly positive finite extents do not guarantee safe signed16 magnitudes.
An independent synthetic witness with bounds `[100000000,0,100000008,8]`
produces normalized X endpoints `0` and `65536` under the retained operation
order; low16 packing then stores zero for both. A future implementation must
either retain the exact wider conversion/truncation contract or explicitly
admit a smaller numeric profile. It must not infer saturation from the intended
normalization bounds. Nonfinite values, exceptional conversion behavior, actual
MXCSR and loaded-CRT semantics remain outside these finite witnesses.

The inverse normalization matrix is handed unchanged to output-receiver
virtual `+0x08` at `0x1400c0441`. That receiver is the incoming R8 of
`0x1400c5d50`, forwarded through the selected callers. Its concrete construction,
matrix persistence and later reconstruction are not established by these
records. Packed coordinates are therefore not labeled final pixels or world
coordinates merely because their inverse matrix is now known.

### Exact outer-matrix storage and remaining source binding

Both ordinary context constructor `0x14003bf20` and alternate constructor
`0x14003d450` call base `0x14003bda0`. The base copies all eight words of its
incoming matrix to context `+0x70..+0x8c`; the wrappers do not replace them.
On the previously recovered new-context path, this is the same supplied matrix
that the hierarchy source producer copies into source record `I+0x70`.

Two byte-pattern candidates for direct calls to `0x1400b0f10` were checked as
actual complete function instructions. In `0x1400b1220`, the ordinary branch
copies incoming render-state `+0xa0..+0xbc`; a separately gated path calls
`0x1400b0a40`. In `0x1400b1570`, the non-bit4 branch obtains its eight words
from a typed matrix handle reached through node `+0xb0`, using table
`0x142adb5c4`; its bit4 branch also calls `0x1400b0a40`. Neither this direct-call
search nor the copied words prove which native matrix handle an executing modal
held. Saved Scene world matrices remain an explicit binding obligation.

### Image-resource matrix boundary

The previously retained fill route at `0x1400c3290..0x1400c32be` obtains an
argument from `[[context+0x50]+0x50]` virtual `+0x1c8`, passes it to the selected
image's virtual `+0xb0`, and left-composes the returned image matrix with its
current fill matrix. Actual image selection is separate from the point and
quantization math above.

One concrete engine asset route, `0x141268d60`, obtains a Q object through
renderer virtual `+0xd0`, constructs a 0x38-byte wrapper with table
`0x1421c11c0`, stores Q at `+0x10`, leaves optional matrix `+0x20` null, and
records width/height at `+0x2c/+0x30`. General constructor `0x14125e840` installs
the same final table with supplied Q and dimensions. These are static successful
construction profiles, not proof that the saved modal used this wrapper.

For this table, `+0xa8 → 0x140031b60` returns Q, `+0x28 → 0x1400cb740` returns
the stored dimensions, and `+0x30 → 0x14126e4c0` produces an unsigned rectangle
`[0,0,width,height]`. Its `+0xb0 → 0x1400372d0` calls Q virtual `+0x38` for a
matrix; null Q produces identity. With a non-null optional image matrix it takes
a separate composition path. It then converts the image rectangle's first two
u32 coordinates through zero-extended 64-bit conversion and adjusts translation
using explicit products and additions. Zero rectangle offsets are still actual
operations, not permission to drop signed-zero behavior.

A concrete matching manager subtype is installed by `0x14125e8e0`: table
`0x1421c2f50`, whose `+0xd0 → 0x141268c00` allocates and initializes Q with
final table `0x1421c2dc8`. Its constructors initialize `Q+0x40` to zero, then
OR mask `0x04`, and set `Q+0x5c` to an inline size/resource record. The observed
external-resource initialization branch preserves a distinction between the
size at `Q+0x30/+0x34` and the first pair in that record. Asset-query results can
populate them from different fields. Calling one pair "logical" and the other
"physical" is an interpretation, not a recovered field name.

The concrete Q `+0x38 → 0x14126ea10` chooses `Q+0x30/+0x34` when
`Q+0x40 & 0x01` is set, otherwise the pair through `Q+0x5c`. It zero-extends
each u32 to 64 bits, converts to f32, and performs separate `1.0 / width` and
`1.0 / height` operations. The other six matrix words are positive zero.
There is no explicit zero-size or finite check here. Base-table method
`0x14126ea60` always uses `Q+0x30/+0x34`; it must not substitute for this
conditional subtype method. On the bounded fresh external-construction path,
the explicit flag sequence is zero then `0x04`, so the record pair is selected.
This does not establish later lifecycle state or successful asynchronous work.

The recorded source asset queries, in precedence order, select record-size pairs
from returned-object offsets `+0x180/+0x184`, `+0x104/+0x108`, or
`+0xfc/+0x100`. The first result can separately replace `Q+0x30/+0x34` from
`+0x188/+0x18c`. These are offset facts; the actual asset pointers, query branch
and dimensions for the modal remain unobserved.

There is a concrete filename-binding gap. Saved UISource44/tag89 contains the
bare filename `Tooltip_Main_BG_PNG.tga` at `7920+23` and export key
`Tooltip_Main_BG_PNG` at `7900+19`; its declared size is 64×64. The examined
engine ImageCreator table `0x1421c3288`, installed as state ID 11 by
`0x141273a20`, has a path method at `0x141277c30`. That method requires an
initial slash somewhere in its incoming string, skips consecutive slashes,
rewrites selected separators in the remainder and passes a resolved asset to
`0x141268d60`. Directly supplying the bare saved filename would take its null
return path. A native path producer or different route is therefore required;
the filename must not silently be treated as proved input to that factory.
Actual tag1009 loading, manager/asset selection and lifecycle state remain open.
Saved 64×64 alone does not prove a runtime `1/64` UV matrix.

### Conditional saved-input witness and validation

An independent scalar witness uses exactly the modal's four authored outer
corners, the retained native-operation nine matrices, and explicitly supplied
`C=identity, M=G`. This is not an assertion that the native tessellator emits
only those four vertices or that its executing caller supplied those matrices.
After the point stages and inverse-outer stage, extrema are
`[0,-0.000003814697265625,1280,1280]`. The normalization sends the four corners
to exact ±32764 under this supplied profile. Packing produces unsigned-word
pairs `(0x8003,0x8003)`, `(0x7ffc,0x8003)`, `(0x8003,0x7ffc)`,
`(0x7ffc,0x7ffc)` in TL/TR/BL/BR order. The negative endpoint becomes −32765
through the actual signed-half/floor contract. Native AA/tessellation output,
loaded CRT/MXCSR and resulting pixels were not run or compared.

All **11 new REA command records** completed with exit 0, untruncated output and
unchanged artifact digests. Independent PE-section comparisons verify **3,096
instructions / 13,475 bytes**, plus **1,744 data bytes**. Numerical checks retain
twelve scalar packing cases, three normalization/degeneracy witnesses and the
separate saved-input witness above. They are original instruction-derived
checks, not substitutes for native execution.

New evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-output/`.

| Retained source | Use and provenance |
| --- | --- |
| `rea-output-stage.json`, `rea-context-and-quantization-constants.json`, `rea-outer-matrix-callers.json` | Owned executable: output handoff, context matrix copies, caller roles and exact constants |
| `rea-primary-tessellator.json`, `rea-workspace-and-sentinels.json`, `rea-export-vertices.json` | Owned executable: constructor/table identity, concrete transform/normalization/export methods |
| `rea-image-matrix-candidate.json`, `rea-engine-image-construction.json`, `rea-image-getters-and-path-caller.json`, `rea-texture-matrix-route.json`, `rea-image-creator-installation.json` | Owned executable: compiled conditional image/Q routes and explicit selection boundaries |
| `image-candidates/saved-tooltip-lexical.json` | Owned pinned movies: exact source spans and independently reverified complete hashes |
| `verification/`, `tessellator-candidates/`, `image-candidates/` | Original project byte comparisons, numerical derivations and candidate discovery; no external implementation copied |
| Prior `scale9-rendering/rea-pe-imports.json` | Reused canonical PE import binding for `MSVCR120.dll!floorf`; no loaded-DLL observation |

The executable hash and proprietary compatibility-study licence remain those
recorded above. No SDK or other renderer implementation source was read. Installed
REA skill v62/tool schemas, original prior project evidence and original scripts
were the only additional workflow sources. The complete bundle contains the
retained prior 45 records plus these 11 records:
`complete.bundle.json`, SHA256
`bf148c28f171ac9f564ac622f15ff226702e98f08a55d3ca175e47a6123905b9`.
The source/Evidence index is `verification.json`, SHA256
`309646bb2d5455c8179d8d4969c10f6e3dbbe7889443b0df74a398d2a523d08b`;
the independent capture receipt is
`verification/capture-validation.json`, SHA256
`da60df1bee9ec3a5641d3feba18fccb9d601e74e1b9a3b42a6946651db168b2e`.
The saved-input witness is `verification/quantization-independent/saved-modal-witness.json`,
SHA256 `7cc54db97fa2f8d0512ab6785a78bdd8237f112454b9f0e024c85f7aede46fab`.
Evidence was exported and REA resource close completed. No native client,
renderer, GPU or live service was started for this increment.

## Executable primary-output arithmetic

`gfx::native_scale9` now exposes the proved output arithmetic as a separate
conditional primitive. It accepts explicit supplied point positions and an
outer matrix. It does not infer that a scene, provider, layer, output receiver
or runtime cache meets the selected native profile.

| API | Input/output boundary |
| --- | --- |
| `normalize_primary_vertices_finite_ieee_nearest(all_vertices, outer_matrix)` | Inverse-outer transform, native seeded extrema/rescan, positive-extent guards, normalization and its inverse |
| `pack_primary_coordinate_finite_ieee_nearest(value)` | One explicitly selected coordinate to a raw u16 stored word |
| `pack_primary_point_finite_ieee_nearest(point)` | Explicit selected X/Y pair to two raw words; other vertex fields remain outside scope |

Normalization must receive **all** positions whose primary-tessellator bounds
are intended. Selecting an export subset first can change the native result.
The returned `PrimaryNormalizedVertices` preserves input order in `positions`;
the caller explicitly selects any positions to pack afterward. Native mesh IDs,
record filtering and export iteration are not guessed.

The result exposes `inverse_outer`, `pre_normalization_extrema`,
`forward_normalization`, `inverse_normalization`, `normalization_applied` and
`extrema_rescanned`. The extrema are the literal seeded native state after the
inverse-outer stage and before normalization. Positions are in normalization
space if that stage applied; otherwise they remain in post-inverse-outer space.
The inverse normalization matrix is the value handed to the unresolved output
receiver. It does not include the outer matrix or establish final pixels.

The fresh sequence follows the retained leaf contracts: inverse-outer storage
starts from identity, so unused lanes of the supplied outer matrix are not copied
into its inverse. Both native transformation stages keep their distinct product
and addition orders. Extrema use exact finite sentinel words, strict comparisons
and the optional rescan. Either nonpositive final axis span skips normalization
and leaves the already transformed positions unchanged. Empty input retains the
reversed sentinels and identity normalization. This empty leaf sequence does not
claim that the enclosing native output routine enters it past its earlier
mesh/count guards. The implementation does not add a generic area, collinearity
or minimum-point-count predicate.

The existing finite IEEE nearest-rounding profile applies throughout. Standard
host `f32::floor` supplies the finite floor operation; loaded retail CRT/MXCSR
equivalence remains unverified. Packing first performs the separately rounded
signed-half operation, then floor, then checks exact signed32 representability
before the float-to-integer cast. In particular, `i32::MAX as f32` is not used
as an inclusive upper limit, because it rounds up to the invalid `2^31` input.
Values outside the admitted signed32 range return a typed error. Low16 wrapping
within that range is preserved, including negative coordinates and normalization
overshoot. It is not a signed16 saturation or a proof of consumer signedness.

A fixed **65,535-point Rust admission cap** bounds input work and one reserved
output vector; this conservative limit is not asserted as the universal native
tessellator storage limit. Allocation failures, nonfinite/subnormal-sensitive
operations and unsupported integer conversion return errors. Caller inputs are
immutable, and no partially rewritten output escapes on failure. No renderer
profile, image binding, option or output-receiver behavior is enabled here.

Focused checks cover stage-specific extrema, distinct outer unused lanes,
one-axis degeneracy, empty input, negative and adjacent-half packing cases,
low16 wrapping, signed32 endpoint admission and resource limits. Private original
oracles reuse the frozen instruction-derived normalization and saved-modal
witnesses above. Raw source assets and numerical asset fixtures remain private
under `evidence/native-scale9-output-math/`; no new native execution or external
implementation source is used.

**VERIFIED conditional arithmetic replay:** the frozen private harness passes
**49/49 checks in both debug and optimized builds**: forty private oracle/admission
checks plus nine module checks. Expected numeric bits were frozen before the
implementation was inspected and were not adjusted to fit it. Coverage includes
the saved modal's complete supplied-input output witness, exact matrix/extrema
bits, reversed point order, collinear positive boxes, empty/one-axis-degenerate
cases, finite-sentinel edge cases, large-offset low16 wrapping, and signed32
conversion and 65,535/65,536-point admission boundaries. No full test suite or
native/GPU execution was run for this increment.

The private receipt is
`evidence/native-scale9-output-math/oracle/verification.json`, SHA256
`25a25e67b9d64ce913ec348519f03750fd28b8ea5a830f1c8b6f7b14ae3e8fb7`.
It records compiler commands, unchanged oracle hashes and tested source SHA256
`29e70b57343a5ff08bfdbb53f61d2ff345739270475864bcce019b17ef2193de`.
Additional source reads were original project evidence and tools only:
`scale9-output/verification/quantization-independent/`, the prior matrix oracle,
and `native-scale9-output-math/contract/CONTRACT_NOTES.md`. The underlying owned
binary/asset licence and compatibility-study provenance remain as recorded above;
no outside implementation was copied or linked.

## Native tag1009 binding and the post-creation image matrix

This increment closes the compiled tag1009-to-factory argument route and
identifies a matrix mutation after image construction. It supersedes the
earlier suggestion that the saved basename must reach the examined creator
`+0x08` method. It does **not** establish an executing modal's resolver, asset,
cache outcome or selected texture dimensions. The renderer remains unchanged.

**VERIFIED static route:** header reader `0x140085170` produces the tag code;
dispatcher `0x14008f1c0` selects extended table `0x142adbb50[tag-1000]` for
the admitted 1000–1009 range. Entry 9 is `0x14006e4a0`. That callback reads a
u32 resource ID, three u16 values and two length-prefixed byte strings. Helpers
`0x140084f60 → 0x140034c50 → 0x140084ff0` read a one-byte unsigned length,
copy exactly that many bytes and append a native NUL. This path does not prove
UTF8 validation or general embedded-NUL behavior in later string copies.

**OBSERVED saved input:** UISource44/tag89 has header `7887+2`, body `7889+54`,
ID 44, format 13 and declared dimensions 64×64. The first string is the export
key at `7900+19`; the second is the existing bare filename at `7920+23`.
An independent native-order replay consumes exactly all 54 body bytes and
reproduces the handoff to `0x14006e310`. The complete source movie hash is
`49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5`;
the tag-body hash is
`23e6a4406fab16abb335a492d8a0649165e451a84b2d5164780ac40340a613e2`.

The callback masks the resource ID with `0x9ffff` before passing it to
`0x14006e310`, which creates descriptor D:

| Descriptor field | Recovered source |
| --- | --- |
| `+0x10` u32 | Zero-extended saved format |
| `+0x20` string | Second saved string: filename |
| `+0x28/+0x2a` u16 | Declared width and height |
| `+0x2c` u32 | Kind 1, or 3 for the separate masked-ID condition; actual ID44 gives 1 |
| `+0x30` string | First saved string: export key |
| `+0x18` pointer | Address `M+0x80`, where `M=[loader+0x38]`, unless its first u32 is zero, then null |

The metadata pointer is an address inside M, not a pointer loaded from that
address. Its actual contents and upstream producer remain unproved.
`0x140090930` creates the resource datum using interface singleton
`0x142ca59e0`, table `0x142504c20`; `0x140090e30` assigns the ID handle and
queues the datum. Interface `+0x18` is binder `0x140091ee0`. Queuing the datum
does not establish that native binding ran or succeeded.

### Resolved filename and creator selection

The binder clones the descriptor and supplies resolver input with mode 2,
original filename and binding-context string `B+0x70`. A non-null resolver at
`[[B+0x10]+0x18]` receives virtual `+0x08`; otherwise helper `0x140031760`
resolves the string. For this saved ASCII basename the default predicate is
false. A nonempty base is copied, `/` is appended unless the base ends in `/`
or `\`, then the filename is appended. An empty base leaves the basename.
Neither the runtime base nor the presence/type of a custom resolver is observed.

Only the binder's new-resource cache outcome enters creator selection. The
creator comes from `[[B+0x10]+0x20]`; null fails. Existing cached resources
take a distinct path whose state and internals were not reconstructed here.

| Original descriptor state | Creator call and supplied extension |
| --- | --- |
| `D+0x18` non-null | Virtual `+0x18`, mode 2; info `+0x40` metadata pointer, `+0x48` key, `+0x50/+0x54` zero-extended declared dimensions |
| `D+0x18` null | Virtual `+0x10`, mode 1; no mode-2 key/format/dimension extension |

Both calls receive the **resolved filename string** in R8. Mode-2 info
`+0x38` receives the format conversion from `0x14004d5e0`; saved format 13
maps to integer 5. No external enum name is assigned to that value. Other
heap/service fields remain explicit context inputs in the private record.

For the earlier proved engine creator table `0x1421c3288`, virtual `+0x18`
is a thunk to its own `+0x10`, `0x1412770c0`. Thus both descriptor branches
reach this method **if that creator is selected**. Its distinct `+0x08`
method is not substituted for the binder's calls.

The `+0x10` method compares exactly ten narrow bytes against **`/ package/`**,
including the space after the first slash, using imported
`MSVCR120.dll!strncmp`. A match selects the suffix after those ten bytes.
A mismatch supplies null to the next conversion helper and continues with
an empty string; it is not an immediate return from this factory.

The bounded conversion route is `0x1406e2730 → 0x1406e1800`, importing
`KERNEL32.dll!lstrlenA` and `MultiByteToWideChar` with CodePage 0, flags 0 and
counts including NUL. The installed codepage is unobserved; this is not UTF8
proof. The following native order is retained:

1. `0x14077b360` with third argument zero removes the last dot and suffix,
   without checking which path component contains that dot.
2. `0x141265ff0` performs lexical separator/relative-segment processing.
   Complete original helper observations are retained privately; no host
   filesystem normalization is substituted.
3. The factory changes remaining wide backslashes to dots and calls
   `0x1407b7710(class,0,resulting_name,0,0,0,1)`.

The class initializer's exact Engine and Texture literals ground the
Engine.Texture interpretation. The returned pointer passes unchanged to
`0x141268d60([0x142c45980],asset)`. That wrapper's earlier conditional size
queries remain applicable. The creator call does not directly pass the saved
declared width/height as wrapper dimensions. Its all-query-failure fallback
reads caller home-space slots whose initialization at this caller is unproved;
zero or declaration-size fallback must not be invented.

### Dimension ratio, inverse and proxy forwarding

On either successful new-image branch the binder calls the returned image's
virtual `+0x28` to obtain two u32 dimensions. It starts an identity matrix,
computes declared-height/returned-height first at `0x140092225`, then
declared-width/returned-width at `0x140092238`. Declared u16 values are widened
and converted with CVTDQ2PS; returned u32 values are zero-extended through RAX
before CVTSI2SS. Eight separate MULSS operations scale the identity's two rows.
No explicit zero-size guard is present. At `0x1400922a7` the binder calls
image virtual `+0x68` with that matrix and R8=0.

For the selected engine wrapper table **`0x1421c11c0`**, `+0x68` is
`0x140037d00`. It allocates image `+0x20` if needed, copies all eight matrix
words and tail-calls `0x1400378d0`, which invokes the already recovered inverse
`0x140036ee0` using a separate source copy. The optional image matrix therefore
holds the **native inverse of the ratio matrix**. Its constructor-null state
does not survive this successful binding path.

Later image `+0xb0 → 0x1400372d0` obtains the selected Q `+0x38` matrix T and
uses `0x140032800` to form **T × stored inverse** under the existing column
affine convention, followed by the actual rectangle-origin arithmetic.
The native inverse uses its determinant/reciprocal/product sequence; replacing
it with independent reciprocal scale divisions or algebraically cancelling
dimension ratios is not proved bit-equivalent. The saved declaration,
returned-image size and selected Q normalization size are three distinct inputs.
An independent finite-f32 witness uses the actual declared 64×64 but explicitly
synthetic returned size 96×80: the stored inverse X coefficient is
`0x3fbfffff`, while direct `96/64` gives `0x3fc00000`. This one-ULP difference
checks the retained operation order; it is not an observed modal image size.

Only **after** the setter, `0x140091460` creates a resource with embedded image
proxy table **`0x1421c1360`**. The actual image is at resource `+0x48`, equivalently
proxy `+0x28`. Proxy `+0xb0 → 0x14126ea00` forwards to that actual image's
`+0xb0`, preserving the modified matrix. This later proxy table must not be
confused with the actual image table when resolving the earlier `+0x68` call.

The precise remaining gates are the native B/base/custom resolver and creator
identity, metadata presence, new-versus-existing cache outcome, resolved engine
object, successful wrapper/manager/Q initialization, chosen size pair and later
lifecycle changes. Saved import/export identity and a decoded 64×64 Texture2D
do not establish these values. This increment consequently does not enable
`1/64` normalization or claim final Scale9 UV/pixel parity.

### Evidence and source ledger

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/external-image-route/`.
All new native observations use the same proprietary owner-pinned executable
and SHA256 recorded above, solely for private compatibility study. Additional
source reads were the two already pinned original movies, original prior
`scale9-output/` and `scale9-rendering/` evidence, the installed REA skill/tool
schemas, PE metadata and original project scripts. No SDK/parser/renderer
implementation source was read or copied; no native client or GPU was run.

| Record | Evidence ID |
| --- | --- |
| `rea-engine-path-factory.json` | `ev_a762aa8ea3e27f5338dc0649eb304d3851235b06fdce414a4e6d6096bbc29cf4` |
| `rea-external-image-loader.json` | `ev_abee9c866f49da715ea8790d24978c86baf7a7af04f9285eec1837b196ebe581` |
| `rea-wire-to-descriptor.json` | `ev_5a659c32a58199acbf9cfe7658d067817a534a42bf1820d52144eb0f84349667` |
| `rea-binder-matrix-route.json` | `ev_bd9af1f45007c6db3480075ba84c2270a320f164fdfb2da99ade825858f5b631` |
| `rea-registration-engine-conversion.json` | `ev_9a02a7084e6ad436ca043af256a074a409dcae9bfe6d2d683fc817866e77fe22` |
| `rea-import-bindings.json` | `ev_a41215119c76b13f3b13521a3daf14356d843aa5f904a6787905bd836de662d1` |

The six bounded records have unchanged executable digests and untruncated
output. Independent PE-section checks match all returned **2,537 instruction
spans / 9,597 bytes and 732 data bytes**. The loader record also retains an
unsupported `pszj` subcommand: its requested string was not returned despite
process exit 0. The later wire record supplies a separately verified 107-byte
`pxj` observation; the original failure is preserved, not relabelled successful.
Detailed source ledgers and field/arithmetic reviews remain under
`loader-candidates/`, `factory-candidates/` and `verification/`. The complete
62-record bundle, `complete.bundle.json`, has SHA256
`2f3f874e048af8bb17bb99d0a95d4224e3ebfd868bdddce4f081c7d63418bca4`.
It was exported and REA resources closed. No product source,
renderer profile, full-suite result or native execution claim changes here.
