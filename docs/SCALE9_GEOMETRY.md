# Primary Scale9 geometry: submitted paths and triangle transfer

This increment closes the selected primary tessellator's **source submission**
contract and its **already-generated triangle index transfer**. It does not close
triangle generation. The new original module is
`src/gfx/native_scale9_geometry.rs`; root owns its module hook and any later
integration. Preview remains diagnostic base geometry.

`PrimarySource::submit_straight_path` accepts explicit native-role style IDs,
resource-presence facts and a `NativeScale9Context`. It transforms START and each
straight END in order, retains repeated points, and stores the native inclusive
path interval. It creates no grid intersections or triangle fan.
`export_triangle_indices` accepts a different domain: caller-supplied generated
triangle connectivity and generated vertex ordinals. It emits the recovered
ordered three-u16 stream. Source path indices are not generated vertex indices.

## Evidence and scope

**OBSERVED, byte-verified:** all native observations concern the owner's pinned
1.52.0.1700 x64 executable, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
under `~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/`.
The proprietary owner installation was studied for compatibility; no SDK code,
native implementation or asset bytes were copied into the Rust module. Addresses
below are preferred VAs, with `0x140000000` omitted in short names.

Three new target-free REA records are retained privately beneath
`evidence/scale9-geometry/native/`:

| Record | Exact Evidence ID | Covered dependency |
| --- | --- | --- |
| `rea-primary-input-generate-export.json` | `ev_66aafda720ebe6405b090d7fdce305935eae9a3dbffedae3e2bc0f8e1dc11d88` | table, input point/path routines, generator dispatcher, descriptor and triangle export |
| `rea-style-preparation-triangulation.json` | `ev_a729e58b24d94bd6570114d3ab7b936ec3f1bfadd8857d4a0dd79bdb8b2eaff1` | selected mode, style bookkeeping, preprocessing and non-AA dispatch |
| `rea-primary-reset-counts.json` | `ev_41377d71b5337cd72bc13b9d9589d26acb54fdb401c04d4f08a191875140035a` | reset, count getters and reset slot |

All commands completed with exit 0, no truncation and unchanged artifact hash.
Independent PE-section mapping checked all 1,217 decoded/invalid rows against
5,038 distinct instruction-span bytes, and all 156 dumped byte values. The
validation deliberately distinguishes byte coverage from semantic decoding:
`ffa10+160` ends in four invalid one-byte rows from a cut instruction; it is only
a prefix. The first `fed20+128` selection ends in the adjacent `fed90` prefix;
the second record captures `fed90` through its returns. The complete reset body
ends in a tail jump to an allocator helper, not an independently decoded allocator.
No claim requires those unknown allocator internals.

The private `verification/receipt.json` hash is
`3027a9350f3078e75b9eac8b57a356684219e31159119be0c481d6d2aadcd4eb`.
`native/final.bundle.json` contains all three records; the investigation was
exported and reset without an active binary or native execution. REA's raw-image
map/decoder choices were caller-selected; the independent PE byte comparison
supplies the mapping check.

Retained earlier REA is reused through `reuse-primary/ledger.json` (nine records,
18 checked instruction spans), SHA256
`9aef04b4eb4cf328d2902b7a3cb51f0d23148c0a7e29506db4bf8ebf520f9493`.
Its sources are the existing `scale9-rendering` and `scale9-output` receipts
cited by [SCALE9_RENDERING.md](SCALE9_RENDERING.md). No external source or new
runtime observation is introduced by that ledger.

## Selected caller and state

**OBSERVED:** `c5d50` selects `c4dd0` on the ordinary layer branch whose two
selectors at +0x10/+0x0c are zero. `c4dd0` takes workspace
`W=[context+0x50]+0x58`, primary tessellator `T=W+0xa0`. Existing constructor
evidence binds its vtable to `0x1425096d0`. The relevant slots are:

| Slot | Target | Observed role |
| --- | --- | --- |
| +0x08 | `ff650` | reset |
| +0x20 | `103700` | append one X/Y pair |
| +0x30 | `103a70` | finalize current path |
| +0x48 | `ff640` | generated vertex count, T+0x320 |
| +0x50 | `ff5e0` | output descriptor count, T+0x360 |
| +0x58 | `ff5f0` | descriptor vertex count |
| +0x60 | `ff620` | selected mesh's triangle count |
| +0x68 | `100020` | copy descriptor and reset its export cursor |
| +0x70 | `100090` | existing recovered vertex export |
| +0x78 | `100160` | triangle index transfer |

The caller resets T at `c4fae`. `ff650` sets source count T+0x78, path count
T+0x50 and pending path start T+0x98 to zero. It also clears resource bitset
storage, resource-present byte T+0x23 and maximum style T+0x40. The caller then
invokes `fed90(T,1)`: T+0x18 becomes 1 and byte T+0x21 becomes 0. This numeric
mode does not collapse nonzero style IDs. No public winding-rule name is assigned
to mode 1 without its consuming branch proof.

`fed20` copies the caller's options[0] to T+0x1c, sets T+0x20 according to its
strict comparison with float zero, and copies options+0x0c to T+0x3b4. `fed60`
stores an explicit float at T+0x3ac and sets byte T+0x3b0 iff it is greater than
zero. `c4dd0` supplies half of options+0x38 when `(flags & 3)==1`, otherwise
zero. The observed half constant is f32 0.5. These generator options are not
guessed from saved GFX tags or hard-coded by the new staging helper.

## Submitted source vertices and styles

**OBSERVED:** the non-morph `c3d30` path reads a start/style triple from provider
virtual +0x40 and edges from +0x48. A zero path code ends the input; a later
path code 2 ends this layer. Equal first and second style IDs skip the path before
point transformation. For an admitted path, start and each straight endpoint
pass through adapter +8 (`bed40→1333c0` for Scale9) before primary +0x20.
The curve branches call additional helpers; they are outside this module.

`103700` writes the incoming two f32 values unchanged into eight-byte source
records, in pages of 16, and increments the u64 count. It performs no deduplication,
closure, region-intersection insertion or f32 arithmetic itself. The point
transform is the existing finite-profile `NativeScale9Context` operation.

For mode 1, `103a70` retains a path only when the submitted count is at least
pending-start+2 and its two u32 style IDs differ. Otherwise it rolls the point
count back to the pending start. A successful 16-byte path record contains:

| Offset | Value |
| --- | --- |
| +0 | u32 pending start |
| +4 | u32 point count minus one, inclusive |
| +8 | first u32 style argument |
| +0x0c | second u32 style argument |

The next pending start becomes the low u32 of the point count. Two coincident
points still qualify; there is no area/unique-point/closed-loop check here.
The helper deliberately preserves native style argument order rather than
inferring a provider's left/right convention from the serialized SWF alone.

For each nonzero style, `103170` expands a bitset as necessary, updates the
maximum u32 style ID, and ORs its bit if the supplied resource pointer was
nonnull. A false value never clears an already-set bit. Style zero is ignored,
including a supplied true bit. `c3d30` obtains the booleans by querying the two
provider fill records and testing their pointer fields; the saved bitmap type
alone does not prove those values. They remain explicit `StyleSide` inputs.

**VERIFIED implementation, conditional native comparison:** `PrimarySource`
models this reset/mode-1 input state with Rust vectors. It admits straight paths
only, at most 65,535 source points, 32,767 paths and style IDs at most 65,535.
These are engineering limits, not native source limits; native counts/indexes
have different widths. They exclude counter truncation/wrap rather than emulating
it. The source cap is not the caller's separate generated-output `<65535` gate.
Capacity/allocation failures, nonfinite/subnormal-sensitive transform failures
and limits fail without publishing partial state. Native allocation-failure
behavior is not claimed. The method stages one whole path transaction rather
than exposing a partially appended native allocator state.

## Triangle transfer, separate from connectivity creation

**OBSERVED:** generator `1115b0` later scans generated 20-byte vertices at
T+0x338. For each vertex whose mesh ID at +0x12 is not 0xffff, it assigns +8 the
next u32 ordinal from that mesh descriptor's +0x18, then increments the descriptor
count. These are generated vertex fields, not submitted X/Y records.

`100160(T,mesh,destination,firstTriangle,count)` reads already-generated triangle
records from T+0x388's selected 32-byte mesh descriptor. Triangle records have
stride 24; their first three u32 fields are global generated-vertex indices.
For each in stored order, it reads that vertex's +8 ordinal and writes its low
16 bits. It advances the destination by six bytes per triangle. It does not
swap corners, alternate winding, create a fan, or insert restart indices.

The existing output caller requests chunks of at most 256 triangles, then passes
the scratch stream unchanged to receiver +0x20 with element counts multiplied
by three. The new `export_triangle_indices` implements that transfer from explicit
Rust slices. It rejects invalid references/ranges and oversized batches rather
than performing native unchecked accesses. Low-16 conversion is explicit;
there is no invented saturation. It cannot supply the missing generated records.

## Actual saved Shape72 composition

**OBSERVED saved source:** the modal chain 129→130→131 uses login Shape72, tag158,
body7997+34, and sprite73's ScalingGrid, tag163. The independent bounded source
oracle confirms one implicit-start path:

`(0,0) → (1280,0) → (1280,1280) → (0,1280) → (0,0)`.

All four edges are straight, style-bank 0, saved left fill 0/right fill 1/line 0.
There is no MoveTo, StateNewStyles, curve, serialized closed-path flag or Shape4
flag. Closure is exact endpoint equality. Native provider return codes and
ordering remain a separate unbound interface. Private oracle:
`saved-input/source-oracle.json`, SHA256
`3ac9c836cdfcc6c8d10b9a00ab51b2e2bd7c09f91299856f201ac0cb9063ec87`.

**VERIFIED controlled composition:** the unchanged saved coordinates, supplied
native style pair [0,1], and supplied resource bits [false,true] pass through the
new helper to five source points and one inclusive [0,4] path. This check uses
an explicitly synthetic identity working context, with the saved splitters as
classification inputs. It does not assert the native runtime selected those
matrices, style-resource pointers or provider representation. No scene/world
matrix is silently substituted. It does not claim native generated geometry.

Five focused module checks additionally cover region-dependent endpoint
transformation, retained closure/duplicate points, style union/reset, skip/short
paths, atomic math/limit rejection and generated-index transfer order/truncation.
The module and existing math dependency compiled with a standalone std-only
Rust harness, running only the new module's tests. The saved-source smoke also
compiled and passed. Sources, oracle and owned GFX input hashes remained unchanged.
Private `checks/receipt.json` SHA256:
`58272d5ab0786d410ba55eb693034036b41a35a030e13eb5ae0f641bd9284ba4`.
One initial test compile used a nonexistent `RectF32::new` spelling; correcting
it to the existing `from_coordinates` API preceded these passing checks.

## Exact remaining generation dependency

**OBSERVED:** `c4dd0` calls `1115b0(T,0)` after submission. The generator always
calls `111130` before producing triangles. Its captured body builds an index
array, calls ordering helper `f6e20`, and then compares/substitutes source Y values
using T+0x1c; the helper's comparator/tie policy has not been decoded. For each
retained source path it calls `103bb0`, then performs further scan processing.
This is why a generic polygon triangulator or a rectangle fan cannot yet replace
the native route under a parity claim.

After preparation and intermediate source-array clearing via `ffa10`, the
generator processes its intermediate path list. With T+0x3b0 clear it calls
`10d680`; that complete captured body consumes linked nodes carrying side bits,
maintains a node stack, and calls `10cbf0` when sides change and at the end. With
the selected mode's T+0x21 clear and T+0x3b0 set it instead calls `10e4c0`.
The bitmap/resource-bit state also activates descriptor preparation through
`109e40` and `109cf0`; those outputs must not be replaced by an assumed one-mesh
layout. The new export adapter does not erase this requirement.

The next bounded native work is therefore `103bb0` path-to-scan input creation,
the exact `f6e20` ordering/Y-merge contract and required scan calls, followed by
the admitted no-AA `10cbf0` or AA `10e4c0` output creation. Actual caller option
values and branch selection must be supplied or proved before choosing a route.
No native triangle connectivity, complete Scale9 edge splitting, final output
receiver/GPU format, or paint/UV policy is established here. Texture dimensions,
image creation and resource Link/PostLoad remain the separate texture lane.
