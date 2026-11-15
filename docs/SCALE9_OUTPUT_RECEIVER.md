# Native packed-geometry receiver

The ordinary output receiver is now bound to a concrete construction site and
vtable. It retains the inverse-normalization matrix on the same mesh context
that owns the vertex and index allocations. Its successful staging route copies
the submitted vertex bytes and ordered 16-bit indices into separate spans of
that context's cache allocation. A subsequent engine upload converts the vertex
format and adds the accumulated vertex offset to each index.

These are static instruction and ownership observations. They do not establish
that packed X/Y words are final pixels, identify the selected native draw matrix,
or supply a UV association. No renderer is enabled by this study.

## Evidence and provenance

**OBSERVED native instructions, VERIFIED bytes:** only the owner's pinned x64
1.52.0.1700 executable was studied, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The proprietary executable and all byte-bearing output remain private under
`~/.local/share/marvel-heroes-forever-rust/`. This adds no external SDK source or
copied native implementation. Addresses below omit preferred base
`0x140000000`; field offsets are hexadecimal unless explicitly marked otherwise.

Private evidence root: `evidence/scale9-output-receiver/`.

| Record under `native/` | Evidence ID | Selected dependency |
| --- | --- | --- |
| `rea-context-table.json` | `ev_02a93aeb199c674438609afc94a84354dfa18cca3cc986f69642202216cf8f9a` | Ordinary mesh context vtables |
| `rea-context-leaves-provider.json` | `ev_fbdeca61053e44eed4c8809def2596ec5b5fb38e9dfa12f7549aaf3a92ce4636` | Context leaves and second provider constructor |
| `rea-provider-dispatch-receiver.json` | `ev_fa8e86092b28f6a28e7c35304735d27345dd55f1b960a05d48a844f3e0cf5536` | Both receiver construction sites, first provider constructor |
| `rea-output-tables.json` | `ev_80ca84c9178ded27eb6a5c41e3c41316c3078d1d4ccb4fdf8c9221114117120e` | Receiver and provider slots |
| `rea-ordinary-output-methods.json` | `ev_46439b6a8ea4576e4425b991789f363d90fedcdcd5fdbd709d4189a464adc463` | Ordinary begin/finish/vertex/index methods, packed format |
| `rea-staging-and-batches.json` | `ev_d05a7e68d64eb217518962d4e874a74a5f82bacd585160e4990d84dfcd6f8117` | Staging allocation fields and descriptor copy |
| `rea-caller-and-matrix-candidate.json` | `ev_a1cdc27adbcb11046e48df5d166eac3c0f53c40d716f2f931243532bed55d3ab` | Engine caller and rejected unrelated matrix candidate |
| `rea-engine-matrix-consumer.json` | `ev_87ff6a3d76892f2c46998f7abe32554e82dd1554e0ab0c3ea81bf74762621b6f` | Rejected unrelated destructor candidate |
| `rea-upload-conversions.json` | `ev_28249ed9f242e9dc8242a9d45cb8097af9b5c97256ad6fb469ce246e4b85d6db` | Vertex format dispatch and exact ordered index adjustment |

Retained dependencies from [SCALE9_RENDERING.md](SCALE9_RENDERING.md) and
[SCALE9_GEOMETRY.md](SCALE9_GEOMETRY.md) are indexed with hashes and exact
instruction spans in `reuse/receiver-caller-ledger.json`, SHA256
`88e30a33ccfff6463d36e6ba35a6234d5c8f5d8159b15439fc4d310a3ceecada`.
Those existing records establish `c5d50 → c4dd0 → bff70` and the source/generated
geometry distinction. This study adds the receiver identity rather than
reinterpreting that earlier output as screen coordinates.

All nine REA commands completed with exit 0, no truncation and unchanged target
hash. Independent PE mapping verified 1,340 decoded rows, 5,136 distinct code
bytes and 448 dumped bytes. One earlier short selection ends on byte `75` at
`03b6cf` inside the next helper; this invalid cut remains recorded. The later
complete `03b6c0` selection supplies the full instruction, so every selected
code byte is also covered by a non-invalid row. No semantics rely on the cut.
The leaf index windows include explicitly recorded trailing `int3` padding;
the vertex converter matches its four contiguous `.pdata` ranges.

REA used the caller-selected raw mapping base `0x140000c00`. For `.data`, the
two dumped query addresses `142ada640` and `142ada568` independently map to
preferred `142adb240` and `142adb168`; the `.text`/`.rdata` identity translation
must not be copied to `.data`. Final audit receipt:
`verification/final-receipt.json`, SHA256
`99ee3ac41a47f2ee203efab7052d59ef1c868dc9dc140f8d5d2c8d09fdf7f5c3`.
All nine records are retained in `native/final.bundle.json`, SHA256
`25f44f198d8bb3dc80eb336f80fa7cf1d6bfe165810dcf3a651ff916d870d309`.
The investigation was exported and reset with no retained native target.

## Receiver and mesh identity

**OBSERVED:** retained base constructor `03bda0` sets mesh context M's `+58`
provider pointer, `+50` renderer/cache owner and `+70..+8c` incoming eight-word
matrix. Derived constructor `03bf20` installs ordinary context vtable
`1424fd5c0`, initializes `M+c0..dc` to the identity matrix and clears `M+e0`.
The latter matrix is separate from `M+70`.

Provider constructor `c9740` installs interface vtable `142507100` at object
`+10`; constructor `122130` installs `142509cb0` at the same interface offset.
Both tables' `+18` slot is `c5d50`. Its entry adjusts the interface pointer by
`-10` to recover the provider object. This establishes the exact table base;
adjacent preceding pointer data is not treated as that vtable.

The ordinary receiver R is constructed on the stack by `03eeb0`. For each
eligible context M in its supplied collection, the function loads `M+58` and
calls provider `+18` with RDX=M, R8=&R and R9D=`M+98` at `03f014` or `03f0c3`.
R has this observed initialization:

| Receiver field | Value |
| --- | --- |
| `+00` | vtable `1424fd658` |
| `+08` | incoming cache owner |
| `+10` byte | 0 |
| `+18` | the exact M passed to the provider |
| `+20` | incoming source-format pointer |
| `+28`, `+30` | null |
| `+38` dword | status 6 |
| `+40`, `+48` | null |

The constructor also tracks preexisting staging allocations and reference counts.
The engine caller below uses the branch with its fifth argument false; the
other branch additionally filters a context's existing fill/cache association.
This is not a fresh-allocation assumption for every invocation.

The ordinary vtable slots are:

| Slot | Function | Observed role |
| --- | --- | --- |
| `+00` | `03e3d0` | destruction entry; body not expanded here |
| `+08` | `03ec50` | begin, allocate/select route and retain matrix |
| `+10` | `03e2a0` | finalize |
| `+18` | `03e2c0` | vertex transfer |
| `+20` | `03e350` | index transfer |

**Explicit alternative:** `03e960` constructs receiver vtable `1424fd630` and
also calls provider `+18` with a stack receiver. Its slots are `03e700`,
`03de60`, `03e820`, `03e880` for begin/finish/vertices/indices. Its context-field
interpretations differ. The ordinary implementation below must not be applied
to that receiver merely because both reach `c5d50`.

## Begin and matrix lifetime

**OBSERVED:** `c5d50` retains incoming R8, passes it as `c4dd0`'s fifth argument,
and `c4dd0` passes it unchanged as `bff70`'s R9. The latter calls R's `+08` with
its generated batch-descriptor array, descriptor count and the address of the
inverse-normalization matrix produced by the primary tessellator. AL gates
the subsequent transfer callbacks.

In `03ec50`, `R+18` selects M. The successful staging branch calls
`03e5f0(owner+68, M, vertexCount, sourceStride, indexCount)`, with counts from the
first submitted descriptor and stride from the format at `R+20`. It records
status 0 on allocation success or 2 on failure. The direct-cache branch calls
the owner's `+60`, translating observed return codes to statuses 1,3,5,7.
It can set `M+e0` and populate destination/cache pointers. Those return values
are kept numeric here; undocumented enum names are not assigned.

At `03edb7..03ee10`, **all eight input matrix words are copied into the same
M at `+c0,+c4,+c8,+cc,+d0,+d4,+d8,+dc`**. This happens after the allocation branch,
including its failure paths, and before the final signed `status <= 1` return.
The matrix therefore outlives `bff70`'s temporary stack storage. A failed begin
does not justify rolling that field update back as presumed native behavior.

The ordinary begin body does not use its submitted descriptor count before
overwriting that argument register. It reads the first descriptor's counts;
the vertex/index methods accept only batch ordinal 0. This is a concrete
single-batch receiver restriction, not evidence that every provider route has
only one generated tessellator descriptor. Empty tessellator descriptors and
the separately selected output batch domain must remain distinct.

## Successful staging ownership and transfers

**OBSERVED:** `03e5f0` reserves
`align_up_16(vertexCount * stride + 2 * indexCount)` bytes. On success it stores:

| M field | Value |
| --- | --- |
| `+28` | total aligned allocation size |
| `+30` | vertex staging offset |
| `+38` | vertex staging offset + vertexCount × stride |
| `+44` dword | vertex count |
| `+48` dword | index count |

It links `M+10` into the cache owner's staging list. The reserve helper `03e4a0`
is outside this proof; allocation success is an explicit condition.

`03e2c0` and `03e350` return without transfer when the batch ordinal is nonzero
or status is greater than 1. For status 0:

- Vertex destination is `[owner+68] + [M+30] + firstVertex * sourceStride`.
  It copies `count * sourceStride` bytes from the supplied scratch pointer.
- Index destination is `[owner+68] + [M+38] + 2 * firstIndex`.
  It copies `2 * count` bytes from the supplied scratch pointer.

The vertex-callback products above are 32-bit. The index receiver is different:
`03e363` zero-extends `firstIndex` through a DWORD move before 64-bit address
scaling. `03e376` loads `indexCount` into R8D, and `03e37f` doubles it with
64-bit `add R8,R8`. Its destination offset is therefore `2*zext(firstIndex)`
and its copy length is `2*zext(indexCount)`, without 32-bit doubling wrap.
Earlier caller `c08d9/c08dc` computes `3*firstTriangle` and `3*triangleCount`
using **32-bit** LEAs, so that distinct seam retains its modulo-2^32 behavior.
Allocation-size arithmetic in `03e5f0` also uses 64-bit registers after
receiving zero-extended counts/stride. Bounded Rust adapters must preserve each
seam's width and reject invalid spans; a blanket width assertion is incorrect.

This corrects the earlier sentence assigning 32-bit products to both callbacks.
The independent retained-byte audit checks 138 instructions / 659 bytes against
the existing receiver and output-caller REA records, without a new capture.
Private receipt: `evidence/native-primary-output/oracle/packing-triangle-audit/triangle/receipt.json`,
SHA-256 `d3eb1f9aca63c26c71d7f456f8683e05994d8877e7b49133ef2208f71dfa3775`.
The status-zero and batch-ordinal-zero conditions remain unchanged.

Status 1 instead delegates vertex conversion to owner virtual `+50` and index
transfer to owner `+58`. The ordinary finalize `03e2a0` calls owner `+68` with
`R+50` when status is odd. It does nothing for status 0. These branches are
not equivalent to an unconditional direct byte copy.

## Packed format and the engine upload

The selected packed descriptor at preferred `142adb240` has stride 12 and
element-array pointer `142adb168`. The observed eight-byte element records are
`(offset,typeCode)`:

| Offset | Type code |
| --- | --- |
| 0 | `0x132` |
| 4 | `0x251` |
| 8 | `0x1211` |
| 9 | `0x2211` |
| 0 | 0 terminator |

The earlier producer writes 16-bit X/Y at offsets 0/2, a dword at 4 and bytes
at 8/9. This table alone does not prove signed interpretation by the eventual
GPU consumer. Bytes 10/11 are within the copied stride, but their initialization
and role were not proved; filling them with invented zeros is not byte parity.

**OBSERVED engine caller `1255030`:** for the selected ordinary path, entry
RDX identifies batch B and entry R8 the mesh-context collection. It obtains
source format F from `[[B+40]+20]+30`, destination format G from `B+30`, and
asks its cache owner to allocate the destination buffers. On observed allocation
return code 3 it calls `03eeb0(owner, collection, F, false, null)` via the
stack constructor ABI. After that call, it traverses the same context collection:

1. `032c90(F, [owner+68]+M.vertexOffset, G, destinationVertices,
   M.vertexCount, auxiliaryBatchOrdinalPointer)` converts staged vertices.
2. `033050(destinationIndices, [owner+68]+M.indexOffset, M.indexCount,
   cumulativeVertexCount.low16)` transfers adjusted indices.
3. Destination pointers advance by destination stride × vertex count and
   2 × index count. The cumulative vertex count advances by `M+44`.

`032c90` iterates destination-format elements, locates source elements by masked
semantic bits and selects a conversion entry or strided-copy fallback. It uses
an 8,192-byte scratch budget divided by destination stride. The conversion-table
entries and actual runtime F/G identities are not all closed by this study;
it must not be replaced with an assumed identity conversion for arbitrary G.

`033050` selects scalar `032ee0` or SSE `032fa0`. Both preserve index order and
perform word addition: each destination word is
`(sourceWord + cumulativeVertexCount) mod 65536`. The SSE route uses `paddw`;
its alignment branches and scalar tails produce the same word operation. This
is an index rebasing step, not triangle reconstruction or position conversion.

The engine caller passes no inverse-normalization matrix to `032c90` or
`033050`. M retains that matrix independently at `+c0` throughout this upload.
The native begin callback takes its stride from caller-supplied F, not from the
submitted descriptor's format pointer. Equality of those pointers has not been
proved for the selected live cache configuration and must remain an explicit
input constraint before reproducing its packed-byte layout.

## Remaining boundary

The complete same-context matrix persistence and staging association is now
grounded. The downstream reader that combines `M+c0` with draw transforms,
its vertex-format signedness/normalization interpretation and the resulting
position space remain separate dependencies. Consequently this increment is
documentation and private evidence only: a generic buffer-owning Rust object
would not close the still-unproved position reconstruction boundary.

Two selected matrix-looking navigation candidates were explicitly rejected:
`0b18e0` consumes a different transform-bearing structure whose first fields are
floating-point values, and complete `12606a0` is an unrelated destructor.
No absence claim follows from the bounded pattern searches.

No retail process, native renderer, GPU draw or visual parity check was run.
