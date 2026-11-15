# Saved Model resource records and their native consumers

`upk::model_render_resource::ModelRenderResource` inspects the original saved
36-byte records in Model `+0x19c` for the admitted ordinary **868/3** profile.
It retains the complete existing Model result, original export bytes, bulk
descriptor and exact record spans. It does not initialize native resources,
regenerate vertices, create index buffers, evaluate materials or render.

This increment follows the saved ModelComponent path. The preceding
[NODE_RENDER_GEOMETRY.md](NODE_RENDER_GEOMETRY.md) describes a separate Level
rebuild branch that the ten saved FrontEnd components bypass. Its local fan
recipe is not substituted for the path documented here.

## Checked raw-record API

`parse_export(package, export_index, bytes, limits)` uses a zero-based export
index. It delegates the exact Engine.Model import identity, version, object
prefix, property stream, bulk layout and reference checks to `ModelObject`.
The resulting `records` descriptor is the admitted `Field { native_offset:
0x19c }` container. `record(index)` checks a zero-based ordinal and returns:

| Record bytes | API field | Established role and retained information |
| --- | --- | --- |
| `0..12` | `position_bits` | Three original IEEE32 position words |
| `12..20` | `basis_bytes` | Eight original packed basis bytes, without conversion |
| `20..28` | `coordinate_bits` | Two original IEEE32 coordinate words |
| `28..36` | `paired_bytes` | Eight original paired-data bytes |
| `0..36` | `raw`, `span` | Entire record and its exact export-relative span |

All signed-zero, nonfinite and otherwise unusual bit patterns survive. The
reader does not enforce the observed packed constant/sign bytes, relationships
to Nodes, component ordering, material equality or correspondence to regenerated
output. Those would require additional contracts. Unknown Model suffix bytes
remain available through the complete base result.

The default inspection limits are 32 MiB of serialized export bytes and
1,048,576 resource records; callers may lower them. These are inspection limits,
not asserted native rejection rules. Exceeded limits return `Unsupported`.
Invalid framing, truncated records, overflowing spans and out-of-range record
ordinals return `InvalidData`; unsupported versions/classes/strides retain the
base reader's `Unsupported` result. The record view allocates no record array:
individual lookups copy only one fixed 36-byte value from borrowed bytes.

## Exact Model resource identity

**VERIFIED (static):** the known Engine.Model constructor
`0x140bed980..0x140bedc27` installs these embedded-object bindings:

| Model offset | Installed vtable | Relevant member relationship |
| --- | --- | --- |
| `+0x168` | `0x141f9afe8` | Resource whose `+0x18` slot is `0x140c07450` |
| `+0x194` | `0x1420108e0` | Array interface; pointer at `+8` is Model `+0x19c`, count at `+0x10` is Model `+0x1a4` |
| `+0x1b8` | `0x141e35018` | Descriptor consumer whose `+0x18` slot is `0x1408e64f0` |

The existing serializer proof in [UPK_MODEL_POLYS.md](UPK_MODEL_POLYS.md) binds
the ordinary file's 36-byte bulk container to native Model `+0x19c`. This is the
same pointer supplied by the embedded array interface after loading; the native
subobject and its vtable are not asserted to exist as serialized file bytes.

The array interface's slot `+0` selects `0x140de0470`, which returns its pointer
at `+8` after the native count diagnostic. Slot `+8` selects the twelve-byte
leaf at `0x140de0510`, which computes count times 36. Slot `+0x18` selects the
established false-return leaf `0x1405ac2d0`.

The typed Model routine `0x140c0f6a0` passes Model `+0x168` to
`0x140958a00`. Its immediate path invokes the resource's `+0x28` slot
`0x140961270`; its queued command binds the same receiver. The latter links
the resource into a native list and, when global `0x142c27870` is nonzero,
calls virtual slots `+8` and `+0x18`. The known first slot is a return leaf;
the second is `0x140c07450`. Existing initialization-bit and global gates remain
part of this contract; no runtime gate values are asserted.

`0x140c07450` queries the array interface's byte count. For a nonzero low word,
it calls `0x1406db7f0` with that count, the same array interface and raw value 1.
The wrapper passes them to indirect slot `+0xe0` of the object held by global
`0x142c25c78`. Returned-handle operations use its `+0x668` and `+0x738` slots.
This establishes the exact resource/interface data flow; backend class,
allocation/device behavior and actual upload remain unobserved.

Before this path, `0x140c0f6a0` can call packed generator `0x140bf6340` under its
separate global/predicate gates. Thus saved records are inspectable source
state, without a claim that every runtime path uploads them unchanged.

## Source-copy dependency

**VERIFIED (static, under ordinary successful allocation/copy and stable
inputs):** helper `0x140b3b930..0x140b3b9fd` assigns ordered 12-byte arrays.
The two direct calls in `0x140828460` preserve position count, order and bits.
They close returned position count `+0x18 == Node.u8 +0x36`, including zero,
for that selected path. The maximum position copy on this byte-count path is
3,060 bytes. Self-assignment preserves the destination; nonpositive external
source counts clear the destination instead of preserving negative counts.

The copy target is the exact owned import `MSVCR120.dll!memcpy`, reached through
`0x141c6ed86`. No external CRT implementation was copied or executed. Allocation
failure and native error/exception behavior remain outside the raw reader.

The additional complete 205-byte helpers `0x140db7200` and `0x140cfdf10` assign
four-byte and eight-byte arrays through that same copy target. Composed with
the complete `0x140828460` body under the same success/stability conditions,
the returned object has exactly one nested index list `[0,1,...,N-1]`, position
count `N` and paired-data count `N`, where `N` is Node byte `+0x36`. At `N=0`
there is one empty nested list. Paired values preserve the selected pool
record's eight bytes at `+8` in order. This closes an explicit source contract;
it does not call the generator or impose that relationship on arbitrary saved
records in the product.

## Descriptor slots and conditional translation

**VERIFIED (static):** `0x140bfd220` receives Model `+0x1b8` and `+0x168` from
the typed Model caller, directly or through its exact queued-command table. It
stores a descriptor block at resource `+0x26c`. Five selected descriptors share
the Model resource pointer, stride byte 36 and raw metadata word zero:

| Source in resource | Initial offset | Raw type byte | Primary/secondary IDs used by `0x1408e64f0` |
| --- | ---: | ---: | --- |
| `+0x27c` | 0 | 3 | `0 / 0` |
| `+0x28c` | 12 | 5 | `5 / 0` |
| `+0x29c` | 16 | 5 | `4 / 0` |
| First entry at `+0x2ac` | 20 | 2 | `1 / 0`, repeated through secondary ID 3 when count is one |
| `+0x2f0` | 28 | 2 | `7 / 0` |

A separate `7 / 1` descriptor uses resource `+0x300` or its explicit native
fallback. The selected setter leaves that pointer null; the fallback has its
own key, offset zero, stride zero and raw type 8. The general consumer also
retains optional/alternate branches. No C++ member name, shader semantic or
graphics-API enum name is assigned from these numeric IDs.

The exact translators `0x140e540f0` and `0x140e54010` reuse or append a stream
key consisting of the resource pointer, zero-extended stride byte and raw
metadata word. They copy the source offset/type and caller-supplied IDs into
the output descriptor. Raw type is not converted; bytes `+5..+7` are not
initialized by those bodies.

**Conditional boundary:** when global `0x142936fd8` is zero, final submission
helper `0x140e5c5c0` calls `0x140e61160`. It moves duplicate stream/offset entries
to the end and changes their offsets using its native type-size table. The
table gives types 1/2/3/4 sizes 4/8/12/16, types 5..12 size 4, and other types
size zero. This proves those size increments, not full backend format semantics.
The operation uses the owned `MSVCR120.dll!memmove` import.

An explicitly synthetic evaluation of the selected setter descriptors moves
the repeated secondary-1/2/3 coordinates from offset 20 to 36/44/52 when this
transformation is enabled. This does not establish valid accesses beyond a
36-byte record, backend stride adjustment or the runtime value of the global.
Initial setter offsets must not be presented as an unconditional final shader
layout. The primary/alternate lists reach `0x1406db860`/`0x1406db830`; their
backend translation and conditional declaration variants remain a boundary.

## Saved component index path

The exact Engine.ModelComponent vtable `0x141f9b2a8`, slot `+0x460`, selects
`0x140bf8ad0`. That body allocates a 0x100-byte receiver and passes the same
component to constructor `0x140bed200`, which stores it at receiver `+0xcc`
and installs vtable `0x141f9bc98`. Its `+0x18` slot is `0x140bfabc0`.
This binds the consumer's `receiver +0xcc → component +0x238 → Model` chain.

The ordinary branch of that consumer reads component elements and requires
element `+0x54 > 0`. It copies element `+0x48/+0x50/+0x54/+0x58/+0x5c` into a
draw record, binds Model `+0x1b8`, and passes the record to `0x140ceb180`.
Those element fields are absent from the saved element serialization. They
must be initialized by a separate native path; neither the saved reader nor
the component-element allocator establishes their values or a zero default.

The known component vtable's `+0xb0` slot selects `0x140c0b910`. Its null-global
branch calls `0x140bf5a30`, which supplies these element fields from a native
index resource. The caller's tested global is `0x142c35830`. The writer uses
element reference `+8` as the lookup/insertion key for a resource in Model
`+0x120`; map helper internals and preexisting map contents are not inferred
from the selected file.

For each element it sets `+0x48` to that resource, snapshots the resource's u32
index count into `+0x50`, initializes triangle count `+0x54` to zero, and sets
`+0x58/+0x5c` to `0xffffffff/0`. It visits the element's saved u16 Node list.
Record flag bit 26 skips a Node. Bit `0x100` permits a second pass; otherwise
there is one. Pass `s` is accepted when Node byte `+0x34 + (1-s)` equals the
component's raw `+0x240` value, or that value is `-1`.

For each accepted pass, the writer consumes the returned nested index list
`L` and position count `N` from `0x140828460`. For `d=2..L.len()-1` it appends
three four-byte values, using the native 32-bit additions:

```text
Node[+0x18] + s*N + L[0]
Node[+0x18] + s*N + L[d]
Node[+0x18] + s*N + L[d-1]
```

This is the packed-output Node-base namespace, distinct from the Level rebuild
group's temporary 80-byte vertex base. The destination is the resource's array
at `+0x2c`, count `+0x34`, capacity `+0x38`. Element `+0x54` increments once per
triple; `+0x58` takes the unsigned minimum first endpoint and `+0x5c` the unsigned
maximum current-`d` endpoint. With the proved ordered source list, those
endpoints bound each emitted pass when the index additions do not wrap. Counts
at most two emit no triangles in this
loop and leave its initialized bounds unchanged. No geometry-area test is
introduced by this record-count behavior.

Thus the ordinary consumer receives an index-resource pointer, first index,
triangle count and bounds produced by a separate native initialization path.
It does not read a saved index buffer from the component export. Complete
consumer/initialization evidence is retained in private `index-binding/` and
`node-source-indices/`. Runtime invocation, preexisting resource/map state,
global gates, the nonnull-global path and downstream `0x140ceb180` remain open.
No runtime triangle count or screen winding is asserted.

## Original selected maps

**OBSERVED:** Entry Model 2 has no saved resource records. FrontEnd Model 13 has
48 records, exactly 1,728 bytes. The twelve Node `+0x18` raw values are
`0,4,...,44`, each with saved corner count four. All 48 corresponding saved
position spans match the existing typed position bytes; all 48 final eight-byte
spans match their selected pool `+8..+0x10` bytes. These comparisons preserve
original bits and do not execute the generator or claim an active draw.

All twelve saved `+0xc8 +8` words are `0x00000e00`; bit `0x100` is clear. Each
of the ten saved components has one element with header zero. Its reference
slots preserve the same component and the saved Engine.Material import:
references `-33` for components 14–17 and 22–23, and `-32` for components 18–21.
The selected element reference and selected `+0xc8` reference agree for all
twelve Node associations. These observations are not universal equality rules.
The saved component `+0x240` words are 1 for the first material group and 2 for
the second; their native role here is the explicit byte/pass comparison above.
An independent saved-input check confirms that Node byte `+0x35` matches that
word in all twelve selected associations. With the observed flag bits, the
explicit predicate accepts pass zero for all twelve, with no skip, second-pass
flag or wildcard. This evaluates retained input fields only; it does not
invoke the native writer or simulate its buffer state.

Component 18's node list is `[4,7,8]`; its raw candidate record intervals are
`[16,20)`, `[28,32)` and `[32,36)`. Component order therefore is not silently
converted into ascending global saved-record order or one contiguous range.

## Validation and evidence

Private evidence root:
`/home/blizz/.local/share/marvel-heroes-forever-rust/evidence/model-render-resource/`.
The independent saved inventory is sealed separately from native semantics and
product checks.

**VERIFIED:** exactly three new module tests pass, covering raw/nonfinite bits,
record order, repeated lookup, backing identity and retained suffix; malformed
framing, limits, truncation and record/span bounds; and the empty resource plus
inherited exact class/profile checks. One focused offline build and one actual
retained-capture run passed. No baseline or full suite was run.

The actual two-Model comparison matches all 58 inventory rows and 50 separately
projected field rows: 48 records/1,728 bytes, 144 position words, 96 coordinate
words, 384 basis bytes and 384 paired bytes. The independent field projection
checks 388 spans. Empty Entry and four actual out-of-range `InvalidData` results
pass. Complete base Model equality and the same borrowed backing are preserved.
The module remained unchanged at SHA-256
`0cf15d1aee9c14bb4dbd42b3617b2b5f9fd6a6c2242bf8ba7f1012ca9855d08e`,
on baseline `017bec4add4ce9052d97568e1920459ec1f89443`.

Private receipts include `independent/product-check/verification.json`, SHA-256
`6eca12495479ad88fb8eca16f065231e29cd61f9cde5b2ecaa6e0442d21f0d44`,
and `independent/inventory-verification.json`, SHA-256
`4d57dcf20b731254fc9c067be851883789e20f083f4e1708e1a0dbc588468b85`.
These establish the raw reader and finite saved-byte comparisons; native or
Rust rendering, device creation and runtime parity were not tested.

The primary REA bundle, `rea-bundle.json`, has SHA-256
`be21b751ea993b435bf43a59d07eb63ad13b8d7dc2fd049acab8e6c44ac8c44b`.
The byte audit found that the initial one-byte query at `0x14058fea0` captured
only the first byte of `RET imm16`. That incomplete observation is preserved.
A bounded three-byte correction established `c2 00 00`; its separate bundle is
`rea-leaf-correction-bundle.json`, SHA-256
`1d9c112e118841b84fdb5e9f35e3fc7c717c8ac770d066c8afda1c236bbda07c`.
No product discrepancy or additional product test run resulted from this
evidence-coverage correction. The declaration mutator's 48-byte inline table
is separately retained as data, rather than counted as instructions.
The final audit verifies 31 unique effective code ranges totaling 12,281 bytes:
27 complete chained unwind roots, three complete explicit leaves and that
mutator's exact 526-byte code prefix. All nine process-result observations are
successful, unchanged and untruncated; the corrected short instruction remains
visible in the evidence history. No unresolved byte discrepancy remains.

Native component receipts retained under the private evidence root:

| Receipt | SHA-256 |
| --- | --- |
| `resource-path/receipt.json` | `77818296fd904cb7d875974bf69e54ef60375eb8919f3a28712ac304577c94c1` |
| `node-source-copy/receipt.json` | `3c61da25db77051774d8ea32bbdb8b948b8f5b8ed31029eb9ad8099ebaa1b3c8` |
| `node-source-indices/receipt.json` | `e69e273d704fbfc50da3a050e57ea75ef50ae5a65283b9d08df15d4de821a5d0` |
| `vertex-declaration/receipt.json` | `582a9d3614a53ea694e16e838c3005a4390bd98e25d4a650cd28ca1bf94d4496` |
| `index-binding/receipt.json` | `e1ffba89a98a1e3fc7350b2588e38714f8c5432326c1a5dc1fd4550ba5690e6f` |
| `rea-byte-audit/receipt.json` | `9f37ae020f220b9acde0857166f282e3b9dd0a918c778ab63afec19f0a9f3043` |

`native-receipt.json` joins these with the exact REA byte audit.
`final-receipt.json` joins that native result with the raw product checks and
independent saved-input comparisons. Runtime limitations remain explicit in
each component. `rea-final-cleanup.json` observes no open target, investigation
records or pending cleanup. No shared client configuration was changed.

All implementation and verification glue is original. The only shipped inputs
are the same owner-held executable and Entry/FrontEnd package pins registered
in [UPK_MODEL_NODE_GEOMETRY.md](UPK_MODEL_NODE_GEOMETRY.md). REA 6.9.0 and its
MIT skill/guides use the established provenance in [PROVENANCE.md](PROVENANCE.md).
GNU objdump, Python and Rust are verification tools; no third-party implementation
source or CRT implementation was read or copied. The private `source-ledger.json`
records exact source identities, ownership/licences and uses.
