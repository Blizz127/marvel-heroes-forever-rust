# Conditional primary rectangle tessellation

`src/gfx/native_scale9_tessellation.rs` now closes one source-to-generated
geometry route: a fresh mode-1 primary tessellator, one closed axis-aligned
rectangle in the recovered source order, native styles `[0,1]`, resource style
1, zero relative tolerance, zero antialias half-width and disabled generator
postprocess. It expands the observed scan/monotone/three-node branches into
original bounded Rust. It does not choose native runtime options, working
matrices, provider input, texture resources or UVs.

The result is four generated vertices, triangles `[0,1,2]` and `[3,2,1]`, and
the two native descriptors including empty descriptor 0. Resource geometry
belongs to descriptor 1. This order comes from the recovered linked polygon and
orientation calculation; no generic rectangle fan is substituted.

Root owns the module hook and later integration. Renderer enablement is not part
of this change. [SCALE9_GEOMETRY.md](SCALE9_GEOMETRY.md) remains the source-path
staging and generated-index transfer contract; this module connects those two
domains only for the admitted rectangle profile.

## Evidence and provenance

**OBSERVED native instructions, VERIFIED bytes:** seven new bounded target-free
REA records study only the owner's pinned x64 1.52.0.1700 executable, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The proprietary installation remains private under
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/`.
No native implementation, asset bytes or third-party SDK code enters this module.
All code is original; this study adds no outside source beyond the already
recorded owner binary and original saved-source parser evidence.

Private evidence root: `evidence/scale9-tessellation/`.

| Native record | Evidence ID | Exact selected dependency |
| --- | --- | --- |
| `rea-path-scan-order.json` | `ev_4e30e38084ec601bdd08035cf3c35d826f665a21166b0e44020f9be6ac210f75` | 103bb0 path-to-runs, f6e20 source ordering, 103e20 work arrays |
| `rea-chain-constructor-style.json` | `ev_37895911f30aa931f3c6e706f971e3b6a1fce5d40430c6e424f5b380b83da61e` | 103760 chain records, constants, 109cf0 resource descriptor |
| `rea-scan-and-triangle-consumers.json` | `ev_88d6af9e2b1d0432d658714d3fbac0308e113e5a8b305a492e16d24e1ffbbbd9` | 103f60 scan, 10ec80 boundary/monotone processing, 10cbf0 polygon triangulation |
| `rea-order-vertex-monotone.json` | `ev_97d601ca13cafbc89edf3e132931dbbcdb3261c1ce1c279310ae5c4b164f5d1d` | chain ordering, 1048a0 vertex creation, 10ea50/104c70 branches, ffd20 styles |
| `rea-triangle-and-polygon-append.json` | `ev_1e0ef3adfd2a474137681bc9c974b072bbffca435a7f3f544444d8f2cbb90ca5` | 10c8e0 triangle emission, 103560, 104b20 fresh monotone |
| `rea-generated-vertex-style.json` | `ev_1db84d0a913d8e59f7f6c5be10622cbbc4e07e6c6c4581791a8a977e7e50ff33` | 10a110 generated vertex association/reuse |
| `rea-generator-intermediate-clear.json` | `ev_3bc10a40655eeff0c4f61b30180a57bc51da49fd4138edba157ff7042c4434c5` | complete ffa10 intermediate clear |

Addresses below omit preferred base `0x140000000`. All commands completed with
exit 0, no truncation and unchanged executable hash. Independent PE-section
mapping verified 5,033 decoded rows, 21,818 distinct instruction-span bytes and
24 dumped bytes; zero invalid rows. Seventeen selected ranges match contiguous
`.pdata` extent unions. Two leaf selections have explicit limits: f5ff0 includes
its complete comparator, padding and a partial adjacent f6030 prefix; ffa10
ends at `ffd13 ret` followed by padding. No neighboring-function completeness
claim is made. The latter capture supersedes the previously cut ffa10 prefix.

`verification/receipt.json` SHA256:
`861800a8e0436719bcf0e409272790b35088b9a77562826ab025f19a18a04b84`.
REA's raw-image mapping and decoder were caller-selected; the PE comparison
checks them independently. `native/final.bundle.json` retains all seven records.
The investigation was exported and reset; no retail process or active native
analysis session was launched.

## Source ordering and increasing-Y chains

**OBSERVED:** `111130` starts with the submitted source array from the earlier
staging component. It constructs source indices and calls `f6e20`. For at most
nine indices the native branch uses adjacent swaps only when previous Y is
strictly greater than next Y; ties retain order. For the five saved rectangle
points the result is `[0,1,4,2,3]`.

The next loop starts previous Y at bits `0xf149f2ca`, exactly
`-1.0000000150474662e30`. It retains a new scanline iff binary32
`Y - previousY > abs(Y) * relativeTolerance`; otherwise it overwrites that
source Y with previous Y. The absolute operation is independently bound to the
double mask `0x7fffffffffffffff`, then converted back to f32. With the admitted
zero tolerance and valid rectangle coordinates the retained indices are `[0,2]`.
The source helper rejects unsupported coalescing or numeric cases rather than
ignoring this stage.

`103bb0` traverses a path forward, then backward. It finds maximal strictly
increasing-Y runs in each direction and skips horizontal edges. For the admitted
source order it calls `103760` with:

| Construction order | Source start | Edge count | Direction | Styles |
| --- | --- | --- | --- | --- |
| 0, right chain | 1 | 1 | +1 | 0,1 |
| 1, left chain | 4 | 1 | −1 | 0,1 |

`103760` writes eight-byte edge records containing the source start index and
binary32 `(nextX-startX)/(nextY-startY)`. Both slopes are positive zero in this
rectangle profile. It then appends a 44-byte chain:

| Offset | Initial value |
| --- | --- |
| +0/+4 | inclusive first/last edge index |
| +8 | starting Y |
| +0x0c | starting X |
| +0x10 | first slope |
| +0x14 | i16 direction |
| +0x16 | zero flags |
| +0x18/+0x1a | low u16 of the two original u32 styles |
| +0x1c/+0x20 | zero |
| +0x24 | 0xffffffff |
| +0x28 | zero |

The earlier source path retains full u32 styles; truncation happens at this
chain boundary. The new implementation admits only styles 0/1, so it does not
guess behavior for aliased high style IDs. `f5ff0` compares Y, then X, then slope;
the two chains consequently enter the scan in left/right order `[1,0]`.

## Two scan passes and the linked polygon

**INFERRED conditional execution from complete selected bodies:** fresh reset
establishes empty active/intersection/boundary/monotone/generated arrays. For
these two disjoint vertical one-edge chains, `103f60` produces:

| Pass | Y interval | New chains | Return flags | Intersection events |
| --- | --- | --- | --- | --- |
| first | low→high | 2 | 1 | 0 |
| terminal | high→high | 0 | 2 | 0 |

The zero-height terminal pass is required. The first scan marks both chains with
bit 8 and sets their start/end X. The terminal scan marks finished bit 2 and
copies end X to current X. `ffd20` selects current styles 1 then 0 for the first
pass; on the terminal pass it clears bit 4 and skips both finished chains.
`111130` therefore takes the direct no-intersection route through `10ec80` on
both passes. This fact is separate from AA mode.

Every `10ec80` entry resets its X sentinel T+0x210 to `0xf149f2ca`.
`1048a0` creates vertices only for endpoints with X greater than that sentinel
and the preceding emitted X. It uses exact source endpoints in both admitted
passes, not interpolation. The generated order is:

`0=(left,low), 1=(right,low), 2=(left,high), 3=(right,high)`.

At the first left boundary, `10ea50` calls `104b20(style1)`. The fresh monotone
has null head, null auxiliary pointer +0x18, style1 at +0x14, and 0xffffffff
in +8/+0x0c/+0x10. The selected inline append branches produce linked node words:

`[0x80000000, 0x00000001, 0x80000002, 0x00000003]`.

The old-only terminal calls to `104c70` receive a null second argument and
return immediately; auxiliary synthesis paths are not taken. This is supported
by the captured dataflow and fresh monotone constructor, not an assumed default.

The complete `ffa10` intermediate clear preserves the monotone array, linked
nodes, generated vertices, style matrix and descriptor array. It clears scan
work and triangle storage, which `1115b0` then rebuilds. Its 105 explicit qword
clears were checked against those surviving field ranges. Allocator storage
validity/nonaliasing and successful allocations remain conditions; allocator
addresses, capacity strategy and allocation failure behavior are not reproduced.

## Exact triangles and generated descriptors

`10d680` begins with the first two linked nodes. It appends the next node and
calls `10cbf0` when the adjacent bit-31 side changes, then retains the previous
last node and the new node. Thus this polygon invokes the three-node path for
`[0,1,2]`, then `[1,2,3]`; the final two-node call returns without emission.
The four-point scoring helper is not reached.

`10cbf0` evaluates its f32 orientation in the recovered instruction order.
For these rectangular triples the first sign is positive and the second
negative. The emission orders are therefore `[0,1,2]` and `[3,2,1]`.
The Rust component performs that calculation with checked intermediate values;
it does not assume signs merely because the input passed a bounds check.

With T+0x3b0 clear, `10c8e0` visits each corner through `10a110`, never the AA
helper. The first visit binds the generated vertex to style1/mesh1 with flags
0x0a. Repeated visits to the same style and mesh reuse the original vertex.
Only the first three u32 triangle fields are modeled; unused bytes in native
24-byte storage records are not asserted.

The generator appends an empty descriptor before resource setup. `109cf0`
allocates descriptor1 for resource style1; it is not mesh0. The later scan of
generated vertices assigns per-mesh ordinals in generated vertex order.
Final logical records under the admitted profile are:

| Record | Values |
| --- | --- |
| vertices 0..3 | positions above; ordinals 0..3; styles 1/1; flags 0x0a; mesh1 |
| triangle 0 | 0,1,2 |
| triangle 1 | 3,2,1 |
| descriptor0, seven u32 words | 0,0,0,0,0,0,0 |
| descriptor1, seven u32 words | 1,1,1,0x8008,0x8000,0,4 |

The existing generated-index transfer then emits u16 `[0,1,2,3,2,1]`.
Native `1115b0` postprocess is bypassed by the selected caller's false argument;
the new component has no alternate postprocess mode.

## Original API, limits and checks

`tessellate_rectangle(&PrimarySource, RectangleProfile) -> Result<RectangleMesh>`
returns fixed-size vertices, triangle connectivity, descriptors and a trace of
the ordering/chain/scan/tag boundaries. It borrows source state without mutation.
The caller supplies the actual native-role `relative_tolerance` and
`antialias_half_width`; both must numerically equal zero. These are not assumed
retail defaults. Nonzero tolerance, AA, different winding, arbitrary contours,
multiple paths, other style/resource combinations, curves, morphing, strokes,
open paths and changed topology fail closed.

Coordinates must match bitwise along the required rectangle, with positive
width/height and left/low coordinates strictly greater than the native sentinel.
Finite inputs alone are insufficient: native orientation products may overflow
or underflow. The component follows the existing finite IEEE binary32 nearest
profile and rejects nonfinite/subnormal-sensitive intermediate operations.
The f64 prechecks only admit operations; they do not provide the f32 results.
CRT/MXCSR equivalence in a running native client remains unobserved.

**VERIFIED Rust checks:** five focused module tests pass, covering the complete
hand-derived trace, shifted positive/negative rectangles, exact descriptor and
index transfer, source/profile/style rejection, native sentinel boundaries,
overflow and subnormal-sensitive orientation. Standalone std-only compilation
includes the two existing native Scale9 modules; only the new module tests ran.

**VERIFIED controlled saved-source composition:** unchanged saved Shape72
coordinates pass through the original staging module and this module, yielding
four vertices/two triangles and matching all 13 comparison groups against an
independent static oracle. This fixture explicitly supplies synthetic identity
working matrices, saved splitters, the `[0,1]` provider style pair, resource1,
zero tolerance/AA width and disabled postprocess. It does not prove an ordinary
native provider supplied these fields or that runtime selected those matrices.
All source, oracle and asset hashes remained unchanged.

Private check receipt: `checks/receipt.json`, SHA256
`1434f23806631f2e421343253c581de30acd5e8dede32db93090180f1312ea88`.
Independent conditional output oracle: `scan-analysis/receipt.json`, SHA256
`a44cbd781ad148ebb7180554eefee16e147a59966a8f7253fb25f320f2c74da4`.
Independent source/chain trace: `reuse/rectangle-chain-trace.json`, SHA256
`77759152a4b78194fd8293007264876a43cdd1d16aabd2b34e68799db284e0a4`.

No retail execution, GPU render, scene visibility change, actual provider
binding, image load or full tessellator parity was tested. Native point matrices,
resource selection/dimensions, final normalization/packing receiver and bitmap
paint/UV arithmetic remain separate contracts. Root's UV lane and the texture
resource lane remain disjoint from this geometry component.
