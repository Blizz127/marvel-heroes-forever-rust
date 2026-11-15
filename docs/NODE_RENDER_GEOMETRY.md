# Native Node emission and coordinate roles

This study separates three things: saved Model records, the Level builder's
temporary geometry groups, and the Model's packed vertex/resource path. The
existing checked readers remain unchanged. No renderer, material evaluation or
runtime parity is established by static arithmetic or by reproducing it over
the original package bytes.

## Entry condition and geometry-group target

**VERIFIED (static):** the class-bound Level routine
`0x140bc4760..0x140bc66e4` reads `Level +0xe0`, the ModelComponents count, at
`0x140bc47e6`. A nonzero count branches to `0x140bc6469`, bypassing the vertex
and triangle construction discussed below. The zero-count branch constructs
components and geometry groups. The owned FrontEnd package saves ten component
links, so these new construction loops are **not** evidence that the routine
emits triangles when entered with that unchanged saved state. Current runtime
state and other call paths have not been observed.

The typed Model call to `0x140c05a10..0x140c0722c` creates raw 0x130-byte
geometry-group records and inserts their pointers into Model `+0x4e4`. The
20-byte container entry holds a four-byte key followed by the pointer at `+4`.
The Level routine's subsequent group iteration loads that pointer into RBX.
This is a recovered allocation and data-flow relationship, not a recovered
UObject class or final GPU-buffer identity.

| Geometry-group field | Operation in the Level builder |
| --- | --- |
| `+0`, count `+8` | Ordered Node indices visited by this group |
| `+0xf4`, count `+0xfc` | Generated vertices, **80-byte native stride** |
| `+0x104`, count `+0x10c` | Appended four-byte triangle vertex indices |
| `+0x114`, count `+0x11c` | One four-byte Node `+0x14` record index per triangle |
| `+0x90` | Generated coordinate matrix |
| `+0xec`, `+0xf0` | Two generated dimensions |

The Node list's grouping and order come from native construction. They are not
assumed equal to arbitrary ascending export/Node order or to a one-to-one saved
component partition.

## Triangle order and index namespace

**VERIFIED (static):** `0x140bc59a4..0x140bc59aa` captures the group's current
generated-vertex count as `base`. The corner loop appends one 80-byte vertex per
corner, in the Node's saved pool order. The fan block
`0x140bc6195..0x140bc635d` runs only when the unsigned-byte corner count `n` is
greater than two. For each `d` from 2 through `n-1`, it appends:

```text
(base, base + d, base + d - 1)
```

It also appends the Node's raw `+0x14` record index once per emitted triangle.
Counts zero, one and two emit no triangles in this block. There is no position,
area, repeated-point or degeneracy test inside the fan block. Earlier selection
and grouping conditions still apply.

This specifies index order relative to the saved corner sequence. It does not
by itself specify clockwise/counterclockwise screen orientation, camera axes,
culling state or a renderer's final index-buffer format. The base belongs to
the group's generated 80-byte vertex array; it is **not** established to be the
Node's stored `+0x18` value, a position-array index or a global Model vertex base.

The separately retained helper `0x140bf3380..0x140bf3542` also emits a fan, but
uses Node `+0x18`, a pass selector and a `+0xc8 +8` flag. Its Model argument and
final consumer were not class-bound by that older study. Matching the local
fan formula does not merge these two index namespaces or prove the helper is
the final renderer path.

**OBSERVED (saved bytes and an explicit local recipe):** Entry has no Nodes.
Each of the twelve selected FrontEnd Nodes has four ordered corners, so applying
the local fan formula yields `(0, 2, 1)` and `(0, 3, 2)` for each Node: 24 local
triangles. The independent recipe retains all 48 corner joins, 72 triangle
position occurrences and 24 `+0xc8` references. It deliberately leaves group
bases, global indices and correspondence to saved Model `+0xe0` unassigned.

A separate exact-rational calculation over the retained IEEE32 values finds
that all 24 local cross products have negative dot products with their selected
record's `+0x30` vector. This is a numeric observation about those saved inputs;
it neither names that vector a semantic normal nor establishes screen winding,
native floating-point rounding or actual execution of the bypassed builder.

## Generated vertex fields

**VERIFIED (static):** the original Model joins are the ones described in
[UPK_MODEL_NODE_GEOMETRY.md](UPK_MODEL_NODE_GEOMETRY.md). The generated vertex
contains separate position, basis and coordinate fields:

| Generated offset | Source and operation |
| --- | --- |
| `+0..+8` | Exact three-word position from Model `+0xb0` through the selected pool point index |
| `+0x0c..+0x14` | Normalized vector from Model `+0x98` selected by `+0xc8` record `+0x14` |
| `+0x18..+0x20` | Normalized vector selected by record `+0x18` |
| `+0x24..+0x2c` | Normalized vector selected by record `+0x10` |
| `+0x30`, `+0x34` | Two scaled dot products using the **unnormalized** selected vectors |
| `+0x38`, `+0x3c` | Two affine coordinates from the group's generated matrix |
| `+0x40..+0x4f` | Not assigned a meaning or guaranteed initialization by this study |

The surface-selected base point is Model `+0xb0[record +0x0c]`. Let `D` be the
componentwise difference between the current position and that base point.
The first coordinate pair uses `D` dotted with vectors selected by record
`+0x14` and `+0x18`, then multiplies each by the exact single-precision value
`1/128` (`0x3c000000` at `0x141f911a0`). The instruction order differs slightly
between the two dot products; reassociating float arithmetic is not justified.

The basis normalization computes length squared in scalar single-precision
operations. Its fast copy, small-value zero and `RSQRTSS` plus one refinement
paths use exact constants 1, approximately `1e-8` (`0x322bcc77`), 3 and 0.5.
Unordered comparisons, signed zero, the reciprocal-square-root approximation,
and MXCSR rounding/denormal state prevent treating a generic portable
`vector / sqrt(length_squared)` expression as verified bit parity. No float
normalization has been added to the product.

Original names such as TextureU, TextureV or Normal have not been recovered
solely from these operations. The API still retains their saved raw fields.

## Projection matrix and second coordinate pair

The Level builder calls the exact ModelComponent receiver helper
`0x140c050b0..0x140c0577a` with the chosen `+0xc8` record index, integer scale 1,
two dimension destinations, matrix destination `group +0x90` and the group's
Node list as an explicit override. Without an override, this helper instead
visits the component's saved u16 node list and filters by the requested record
index. These are distinct selection modes.

The helper passes the three words at record `+0x30` to
`0x140766460..0x1407666a5`, which constructs two direction vectors using
absolute-component comparisons, axis selection, projection subtraction,
normalization and a cross product. This does not impose unit-length input or
guarantee exact orthogonality for arbitrary stored bits. It projects the selected
position triples onto those directions and accumulates two coordinate bounds.
The surface word at `+0x40`, interpreted by native scalar float arithmetic, combines with the
integer scale. Rounding operations and clamping to 4..1024 feed the two output
dimensions. The generated matrix uses those projected extents and record
`+0x30..+0x3c`; an empty visited-corner set returns zero dimensions and the
retained fixed matrix fallback.

The caller applies this matrix to each position to produce generated
`+0x38/+0x3c`. It copies that pair back into the selected 24-byte pool record's
`+8/+0x0c` at `0x140bc6076..0x140bc607b`. Thus those pool fields can contain
generated coordinate data. This pair is not the base-point/vector dot-product
pair at `+0x30/+0x34`.

These operations support a projected-coordinate role. A lightmap texture,
runtime sampling channel, original member name or final shader binding has not
yet been proved by this data flow alone. The documentation keeps those claims
separate instead of renaming the saved words by convention.

## Material reference consumer

The first `+0xc8` reference is already preserved by `ModelC8Record.links[0]`.
At `0x140bc4c4d..0x140bc4c7a`, the typed Level routine loads that reference and,
when nonnull, invokes its virtual slot `+0x2e8`. If the returned pointer is
nonnull, a second invocation supplies the object whose byte `+0x4e5` is
compared with 2. A match clears bit `0x400` from a grouping value initially
computed as `record +8 & 0xe00`.

That closes a specific dynamic consumer, without recovering the virtual
method's class/type contract or the meanings of its returned byte and flags.
The selected package references resolve to Engine.Material imports, but no
native material object was loaded or evaluated. Nulls, import identities and
raw flags remain explicit; no default material is supplied.

## Saved output records and remaining boundary

The existing parser retains Model arrays `+0xe0`, `+0xf0` and `+0x19c` with
admitted strides 4, 4 and 36. Their saved counts in FrontEnd are 78, 7 and 48;
Entry's counts are all zero. Count or byte-pattern agreement does not prove
that the first two arrays are the intermediate triangle indices or that an
80-byte group vertex is a saved 36-byte record.

The typed Level caller invokes `0x140c0f6a0..0x140c0f88b` with its Model both
before and after construction. Under global and virtual-predicate gates this
calls `0x140bf6340..0x140bf6e7c` with the same Model. That bounded path is studied
separately from the geometry groups. It writes Model `+0x19c`, with count at
`+0x1a4`, capacity at `+0x1a8` and **36-byte stride**, and writes the cumulative
packed-output base into Node `+0x18` at `0x140bf640d`.

Its per-Node source comes through `0x140828460..0x140828a50`; its count is read
from the returned source record's `+0x18`. The `+0xc8` record's `+8` bit `0x100`
can double the contribution. This is not replaced by an unconditional
`Node.corner_count` count or by the zero-components builder's group-local base.
The source helper's direct `0x140b3b930` calls at `0x14082867e` and `0x14082873e`
remain unexpanded; returned count/order equality is an explicit frontier.

| Packed record span | Native output established by the bounded path |
| --- | --- |
| `0..12` | Three position float words |
| `12..20` | Eight packed basis bytes |
| `20..28` | Two float dot-product coordinate words |
| `28..36` | Eight bytes from the paired array addressed by returned source `+0x20` |

Within the packed eight bytes, the first and second three-byte groups come from
the transformed first and third basis vectors. Byte 15 is fixed `0x80`; byte 19
comes from the sign of the scalar-helper result, with nonfinite/exception
behavior still qualified. Component packing uses scalar float add 1 and
multiply 127.5, truncating conversion to a signed integer and native signed
clamping. The optional second output block preserves source ordinal traversal
and flips the third transformed vector's component sign bits before packing.
This does not by itself establish a front/back-face or culling meaning.

Those offsets support a future raw inspection reader without rerunning float
normalization or quantization. Their physical layout does not establish packed
basis accuracy from a portable reimplementation, final shader attribute names,
a final GPU draw target, or runtime activation of the guarded update path.
Immediate basis/scalar dependencies are bounded separately; no hypothetical
default or whole-buffer semantic rule is introduced here.

The complete bodies of `0x1407636e0..0x140763969` and
`0x140809d50..0x140809f1b` have exact byte evidence. Their deeper basis and scalar
formulas are not yet interpreted. The packed-path audit did not compare these
native output fields with the independently retained saved 36-byte samples.

No new product module is added in this increment. The existing reader continues
to retain these bytes. A final-render triangle binding still needs the actual
writer/receiver path and index namespace, and packed regeneration needs the
remaining source/packing and floating execution contract. The intermediate fan
recipe is useful evidence but is not substituted for that renderer contract.

## Evidence and provenance

Private evidence root:
`/home/blizz/.local/share/marvel-heroes-forever-rust/evidence/node-render-geometry/`.
It retains new REA observations, exact owned-PE ranges, scalar constants,
native grouping/triangle proof and independent original-byte inventories.
Static recipe calculations are labeled separately from saved output and runtime
observations. This documentation increment runs no product or native game tests.

The final REA bundle contains three successful, untruncated observations. Its
SHA-256 is `643557a573f6452096b525ab17759d74136c13d1e315042efdfd807f26f7f89e`.
The exact-byte audit covers eight complete bodies and 14,486 instruction bytes
against the unchanged owned executable. The earlier typed Level body and class
binding are reused by their retained hashes. The cleanup receipt observes no
open target, retained investigation records or pending cleanup.
`final-receipt.json` joins the native and independent receipts below;
`MANIFEST.json` seals their retained files and this documentation's identity.

| Private receipt under the evidence root | SHA-256 |
| --- | --- |
| `triangle-support/receipt.json` | `0263758d6cfa61ed2b5aa8ccd6e08b991932c38d92089374be685880459d8d0f` |
| `triangle-support/packed-rea-verification/receipt.json` | `3fe212f60142bfb4fd88725166def65f00dd3d21038e6294504483132e0ba6cd` |
| `basis-support/receipt.json` | `2a5f22c099574ece0514d8ae76c8abd43cafa6705db075ad9d90f0bf6a95b3a3` |
| `triangle-support/packed-vertex-support/receipt.json` | `c21252ecf9bc94a5fc8d32f1e6dca09f8b55d0bf377b208b466ee15c09e704ad` |
| `independent/inventory-verification.json` | `62962c6b28b425c7499347e6de3ef6294312fc47bc5a6d24c7617e8441ab5295` |
| `independent/local-triangle-recipe/verification.json` | `49011b00d926e53192cc9c67ee0d1a0d7316ca1e0387c70392a6d48f9c71b5b4` |
| `independent/local-triangle-recipe/orientation-numeric/verification.json` | `8bebdf24b7204afb1ec534459ec8d25417c8244e0c638432b87a5004215b995c` |

The source executable and two packages are the same owner-held pins identified
in [UPK_MODEL_NODE_GEOMETRY.md](UPK_MODEL_NODE_GEOMETRY.md). Their proprietary
bytes stay private. REA 6.9.0/skill guides are MIT as already registered in
[PROVENANCE.md](PROVENANCE.md); existing PE tooling, Rust readers and private
expectation glue are original project work. GNU objdump and Python are used
for byte verification and bounded data analysis. No additional third-party
implementation source was read or copied. Exact sources and licences are
recorded in the private `source-ledger.json`.
