# Model vertex fields and the limits of their interpretation

This study follows the owned build's 36-byte Model `+0x19c` records from their
native producer into the vertex-declaration consumer. It complements
[MODEL_RENDER_RESOURCE.md](MODEL_RENDER_RESOURCE.md) and
[MODEL_INDEX_RESOURCE.md](MODEL_INDEX_RESOURCE.md). Saved bytes, mathematical
producer operations, declaration fields and downstream shader meanings are
separate evidence claims. No renderer or native client was executed.

The existing `ModelRenderResource::record` view retains all source bits. A
complete renderer conversion cannot be inferred merely from the resemblance
of its two packed triples to the existing StaticMesh API.

## Stored fields and exact producer roles

**VERIFIED (static, conditional producer path):** the complete native writer
`0x140bf6340` emits 36-byte records in the already established Node-base
namespace. Its invocation remains subject to the documented native gates.
For one selected c8 record, define these original three-component source values:

- `O`: Model `+0xb0`, indexed by signed c8 `+0x0c`.
- `A`: Model `+0x98`, indexed by signed c8 `+0x14`.
- `B`: Model `+0x98`, indexed by signed c8 `+0x18`.
- `C`: Model `+0x98`, indexed by signed c8 `+0x10`.

These are native member offsets; the serialized c8 record has different
reference widths. Existing readers retain its exact fields and spans.

| Saved record bytes | Proved producer operation | Unproved interpretation |
| --- | --- | --- |
| `0..12` | Copy the selected source position's three IEEE32 words | World/view transform, camera and handedness |
| `12..15` | Quantize the three components of transformed `A′` | Final shader semantic name or exact original-vector recovery |
| `15` | Write literal byte `0x80` | Treating this byte as an ordinary fourth source component |
| `16..19` | Quantize transformed `C′`, or its sign-bit-flipped components in the optional second block | Final shader semantic name or culling interpretation |
| `19` | Write a byte selected from the determinant sign of the unpacked basis rows | Exact missing-direction reconstruction after quantization |
| `20..28` | Two IEEE32 dot-product/scaling results using original `A/B/O` | Texture/material channel names or shader use |
| `28..36` | Copy the selected pool record's eight bytes at `+8..+0x10` | Lightmap meaning or an arithmetic producer formula |

The source helper's previously proved ordinary successful copy contract
preserves position/pair count, order and bits. Duplicated source positions
remain distinct packed occurrences. This does not establish that the native
producer runs or replaces the selected saved records during frontend startup.

## Basis transformation is not a complete orthogonal basis construction

**OBSERVED static operation sequence:** helper `0x1407636e0` transforms the
three separate stack triples. Under ordinary finite arithmetic, its formulas
can be written:

```text
A0 = A - C * ((C dot A) / (C dot C))
B0 = B - C * ((C dot B) / (C dot C))

A1 = B0 cross C  if squared_length(A0) < threshold_a, otherwise A0
B1 = A1 cross C if squared_length(B0) < threshold_a, otherwise B0

A′ = conditional_normalize(A1)
B′ = conditional_normalize(B1)
C′ = conditional_normalize(C)
```

`threshold_a` has exact IEEE32 bits `0x2edbe6fe` (approximately `1e-10`).
The second fallback's order is **A1 cross C**. There is no subtraction of
the A component from B: A and B are projected independently off C, and are
not thereby guaranteed mutually orthogonal. The helper uses separate scalar
single-precision arithmetic, not an ideal real-number formula or fused
multiply-add. The dot sums order the x/y sum before z, and the C squared length
is recomputed for the B projection. Zero, nonfinite,
exception and denormal behavior must not be silently replaced with an
idealized normalized-vector operation.

The complete 205-byte normalize leaf `0x14074c4f0..0x14074c5bd` compares
squared length against the caller-supplied exact threshold `0x322bcc77`
(approximately `1e-8`). It changes the triple only when the squared length
compares greater. Otherwise it returns zero and leaves the input unchanged;
it does not replace a small vector with zero. Unordered comparisons take the
skip branch when floating exceptions are masked.

The active path uses `RSQRTSS` followed by a refinement sequence and three
component multiplications. Its nominal refinement, with stable initialized
literal cache values, is `0.5 * r * (3 - q*r*r)` where `q` is squared length
and `r` is the instruction's approximate reciprocal square root. This expression
does not authorize regrouping the separately rounded instruction operations.
Exact regenerated values depend on the instruction/environment and are not
reproduced by replacing this path with Rust `sqrt` or normalizing stored bytes.

The native cache is also explicit state. Flag bits at `0x142b18a50` choose
whether to load literal `3.0`/`0.5` or cached low lanes at `0x142b18a40` and
`0x142b18a60`. The missing-bit paths write the flag before the corresponding
cache vector. No synchronization, exclusive-writer or live cache value was
established. These globals are in a virtual section tail without file-backed
bytes; no invented file zeros are used as evidence. The literal refinement
description assumes ordinary completed initialization and stable cache values.

## Byte quantization and the fourth-byte distinction

For each stored component, the writer performs f32 add 1, f32 multiply by
127.5, then signed integer conversion with `CVTTSS2SI`. A negative result
selects zero; otherwise the smaller of that signed result and 255 is stored
as a byte. The arithmetic and exception environment are part of this native
sequence. A mathematically ideal clamp is not asserted equivalent for every
nonfinite, overflow, rounding-mode or denormal input.

Even in ordinary exact illustrative cases this quantization loses information:
source component zero produces byte 127, while the biased representative
`127 / 127.5 - 1` is `-1/255` in exact arithmetic. The literal fourth byte of the
first packed word is 128, whose exact biased representative is `+1/255`;
the separately rounded Rust expression need not equal a rounded rational
calculation bit for bit. That byte is not a copied
fourth source component or proof of an exact zero.

The fourth byte of the second word comes from helper `0x140809d50`, using a
four-row stack matrix populated with unpacked `A′/B′/C′` and a final identity
row. The byte is zero for an effective ordered negative comparison and 255 for
an effective ordered nonnegative comparison, including either signed zero.
DAZ can make a raw negative subnormal compare as negative zero. An unordered result
takes the zero-byte path when the comparison's exceptions are masked. This
decision occurs **before** quantizing the stored triples. `B′` itself is not
stored in these two words. The sign and quantized A′/C′ therefore do not prove
exact B′ reconstruction or a particular shader cross-product convention.

When c8 bit `0x100` selects a second block, the writer sign-bit flips each C′
component before packing it and computing this scalar sign. A′ and source
position/pair ordering remain unchanged. No face, culling or winding name is
assigned solely from this operation.

## Coordinate arithmetic and copied pairs

Let `d = position - O`. The first block's coordinate at offset 20 is computed
as `((dx*Ax + dy*Ay) + dz*Az) * (1/128)`, using original A rather than A′.
Its coordinate at offset 24 uses original B in the corresponding expression.
The exact scalar multiplier is `1/128`.

The optional second block reverses the first two additions for its A-derived
coordinate: `((dy*Ay + dx*Ax) + dz*Az) * (1/128)`. The B-derived path retains
the x/y-before-z order. The mathematical expressions coincide for real finite
numbers, but this is not a claim of identical floating-point exception,
NaN-payload or rounding behavior. Saved coordinate bits are retained rather
than regenerated.

The eight bytes at offset 28 come directly from the source pool's paired data.
The native writer does not calculate them as these dot products. A declaration
type or a plausible numeric range alone does not establish a material channel,
lightmap convention or required coordinate conversion.

## Declaration dispatch and conditional numeric formats

**VERIFIED (static):** the previously established resource consumer builds
descriptors with raw source types 3/5/5/2/2 at offsets 0/12/16/20/28. Its primary
and alternate submitters reach wrappers `0x1406db860` and `0x1406db830`.
The complete 48-byte and 45-byte wrappers load global `0x142c25c78` and call
its receiver's vtable slots `+0x48` and `+0x50`, respectively. Neither wrapper
decodes a vertex component.

A targeted global-write study binds four conditional factory paths from
`0x14089a3f0` to that global's write at `0x14089a655`. Successful constructors
install the following primary tables; the three ordinary `+0x48` slots share
forwarder `0x1412e7690`, which reaches that same receiver's `+0x50` slot.
These are observed routes, not a claim of complete global-writer inventory or
the active runtime selection.

| Conditional receiver table | `+0x50` route | Mapper | Selected raw type 2 / 3 / 5 output |
| --- | --- | --- | --- |
| `0x1421e5448` | `0x1412cd120` | `0x1412c4b20` | Numeric fields `16 / 6 / 28` |
| `0x1421f0818` | `0x1412e7600` | `0x1412e2020` | Numeric bytes `1 / 2 / 5` |
| `0x14221c7e8` | `0x141305020` | `0x1412fe610` | Tuples `(0x1406,2,0) / (0x1406,3,0) / (0x1401,4,0)` |
| `0x141e9d9a8` | `0x140937750` | No descriptor mapping in this body | Allocates a small result object; does not read the source declaration list |

The first mapper creates 32-byte records, the second eight-byte records and
the third 24-byte records. Their copied stream/offset, type mappings and
additional metadata are retained in private proof. The third leaves three
stack bytes untouched in each initial record. The second has a support-table
guard/log path whose runtime contents and effects are not established.

These numbers are not assigned external graphics-API enum names without a
read primary header/spec and an established downstream use. For example,
raw types 5 and 7 share numeric 28 in the first mapper, but become 5 and 8 in
the second and differ in the third tuple's final byte (0 versus 1). An assumed
universal component-normalization rule is therefore not justified by the
internal raw type code alone.

The first mapper uses **literal strings from the owned executable** for its
primary IDs. Applied to the selected Model setter, they produce these
conditional declaration labels:

| Source field | Owned literal label and secondary ordinal |
| --- | --- |
| Offset 0 | `POSITION`, 0 |
| Offset 12 | `TANGENT`, 0 |
| Offset 16 | `NORMAL`, 0 |
| Offset 20 and repeated final descriptor | `TEXCOORD`, 0 through 3 |
| Offset 28 | `COLOR`, 0 |
| Separate fallback descriptor | `COLOR`, 1 |

These strings close the declaration-label question on that conditional path.
They do not prove that a particular material shader consumes the fields, that
`COLOR` is a color-space value or that the copied pair represents a lightmap.
They also do not prove a particular decode of the packed fourth bytes.

The three bounded sorting helpers preserve complete 32/8/24-byte records as
permutations. For the upstream byte-sized stream/offset keys, they sort by
ascending stream then offset; equal-key order is unstable. The second mapper
then appends its exact sentinel and computes an opaque hash. Sorting does not
supply shader semantics or silently normalize the fields.

The earlier declaration transformation remains separate: when its own global
gate selects it, duplicate offsets can move to 36/44/52 even though the selected
saved stride is 36. The backend maps the supplied descriptor offset; this study
does not establish which transformation/gates are active or authorize reads
beyond a saved record. Runtime support state, resource allocation/virtual effects,
cache/upload handling, final API calls, shader signatures and actual shader
arithmetic remain finite boundaries.

## Existing mesh APIs are separate contracts

The original project `static_mesh::PackedNormal` exposes `xyz()` and `w()` as
`byte / 127.5 - 1`, without renormalization. Its format-source ledger attributes
that biased representation to previously studied UEViewer sources. Those
existing API semantics are inspectable, but do not independently bind the
Model declaration, shader inputs, fourth-byte role or reconstruction behavior.
No external UEViewer implementation was newly read or copied in this study.

`skeletal_mesh::SkinVertex` retains raw four-byte tangent/normal words, while
`UvData` retains binary16 or binary32 coordinate bits. It deliberately avoids
speculative normal/tangent reconstruction. Neither mesh reader supplies native
Model consumer proof by virtue of sharing similar byte fields. These modules
remain unchanged.

## Selected saved records and numeric checks

**OBSERVED:** Entry has no saved Model resource records. FrontEnd has 48
records with twelve distinct eight-byte basis patterns, each repeated four
times. The first word's fourth byte is always 128; the second word's fourth
byte is zero in 24 records and 255 in 24. These are sample observations, not
additional parser admission rules.

The two pairs at offsets 20 and 28 have eleven and 28 distinct pair patterns,
respectively. All 192 scalar values in those two pairs are finite and none
has a negative-zero bit pattern. The original Model `+0x98` source vectors
include a negative-zero component, which the inventory preserves. Independent
original inventories agree on field bits, record/source associations and
exact IEEE32 rational values; their outputs remain private.

**VERIFIED (finite numerical association):** for all 48 selected positions,
both saved values at offsets 20/24 equal the exact rational expression
`dot(position - O, original A/B) / 128`. All **96** signed rational residuals
are zero; a second independent calculation agrees on all 96. Each comparison
retains its saved bit pattern, source indices/spans
and three exact terms. This checks the selected source association; it does
not emulate instruction rounding, normalization, cache state or MXCSR, and
does not prove the producer runs on startup. The copied pair at offset 28 is
not substituted into this formula.

## Evidence, provenance and implementation boundary

The owned executable SHA256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All native bytes, captures, exact inventories and private helpers remain under
`~/.local/share/marvel-heroes-forever-rust/evidence/model-vertex-interpretation/`.

The producer audit verifies four bodies totaling 4,189 bytes, including the
new complete 205-byte normalization leaf and three previously sealed bodies.
Its `producer/receipt.json` SHA256 is
`f34fe0720987d3852539e599edff3a421a3fcdf221bd7f7755eb0eeb93e8d9ea`.
The fresh declaration audit covers 21 function/code groups / 10,832 code bytes plus 298
inline-table, padding and vtable-slot bytes. Code, embedded tables and padding
are distinguished explicitly. The four successful untruncated fresh REA
records are in `rea-bundle.json`, SHA256
`ad281cf7cefac61b71a5626d36af906d98cc7799760c85e6a0b8279a483f8f20`.
They include the normalize leaf separately from the declaration bodies.
REA evidence was exported and reset; no target or cleanup work remains open.
The declaration receipt is `declaration-backend/receipt.json`, SHA256
`aa6318b3ab882fbd8e4ca071411698fe0d29ea0603fa5ee5d00e72daf83ec926`.
The joined study receipt `receipt.json` is
`749a22f474491fd4f0d3d34eae267cfdb015c8593ebeef48cd91d400654dd500`;
its parent/component-entry manifest is
`634ed8c25490d8b23432ee3608009169c0f28ceae24d04186adc7314b734f582`.

Private independent receipts include:

| Receipt | SHA256 |
| --- | --- |
| `independent/raw-fields/verification.json` | `2d315f8a5ff5c7ea8a590378f014d91833d0896d43c38db472bfb9f0a7458ade` |
| `independent/source-api-comparison/verification.json` | `ce43031f825f04642b77fa13e272dd9c2b5ae968beb855e0dc99261910661dc7` |
| `independent/coordinate-rational-comparison/comparison.json` | `59a42f458b798e381f24a15f11ae2f499df3b4dac2f1f5055b344001718d09d9` |
| `independent/coordinate-rational-independent/comparison.json` | `8b81d53ca33da1188e82d093f97c32a7455eef138628437246c4a427e6a99bfe` |

Outside inputs are the already registered owner-owned proprietary PE/packages
and MIT REA skill/guides. The StaticMesh/SkeletalMesh sources and previous
evidence helpers are original project code. No new external implementation,
SDK, graphics header or shader source was read or copied. Exact paths, source
identities and uses are retained in the private source ledgers; common source
registrations are in [PROVENANCE.md](PROVENANCE.md).

This increment adds documentation and private evidence only. It adds no
conversion module: exact original-basis recovery, an unconditional normalized
vertex-input interpretation and actual shader decode remain unestablished.
Existing raw readers preserve the information without imposing those choices.
No product build or test suite was required for this documentation change;
all reported checks are source, native-byte or finite numeric observations.
No native client, GPU resource creation, renderer, server or gameplay test ran.
