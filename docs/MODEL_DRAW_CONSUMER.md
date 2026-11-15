# Concrete Model draw-record consumer

**VERIFIED static dataflow:** the normal ModelComponent path passes its actual
constructed record to `0x140ceb180`. That function dispatches the record through
a caller-supplied receiver's virtual slot `+0x38`. Its copied-record paths
preserve the embedded Model descriptor address and ordered index metadata under
the ordinary successful storage contract below. The concrete implementation of
that virtual slot remains unresolved. This does not join a selected Model draw
to the separate generic bound-state or D3D11 calls.

This continues [MODEL_RENDER_RESOURCE.md](MODEL_RENDER_RESOURCE.md) and
[MODEL_INDEX_RESOURCE.md](MODEL_INDEX_RESOURCE.md). The study uses owned build
1.52.0.1700 and exact existing package observations, without native execution,
renderer enablement, service operations or a new asset extraction.

## Actual caller record and distinct receiver identities

The previously verified Engine.ModelComponent constructor path creates a
receiver with vtable `0x141f9bc98` and retains its originating component at
`+0xcc`. Its slot `+0x18` is `0x140bfabc0`, which follows that component's
`+0x238` to the Model. This establishes the Model origin of the following
normal-branch record; it does not identify an invocation of the method.

Define `R` as the outer draw-record address passed at `0x140bfb3fc`. Define its
first element address `E = [R+0xb0]` when that pointer is nonnull, otherwise
`E = R`. The normal branch skips source elements with zero triangle count and
copies their already constructed runtime metadata:

| Source | Destination | Established role |
| --- | --- | --- |
| Source element qword `+0x48` | `E+0x80` | Runtime index-buffer object pointer |
| Source element u32 `+0x50` | `E+0x88` | First index, in u32 index units |
| Source element u32 `+0x54` | `E+0x8c` | Triangle count |
| Source element u32 `+0x58` | `E+0x94` | Writer's unsigned minimum endpoint |
| Source element u32 `+0x5c` | `E+0x98` | Writer's unsigned maximum endpoint |
| Address `Model+0x1b8` | qword `R+0x104` | Embedded descriptor/vertex-factory subobject address |

The last row is an address calculation, **not a read of qword
`[Model+0x1b8]`**. The exact caller loads the Model, adds `0x1b8`, and stores
that result. Native embedded objects and process pointers are not asserted to
exist in the saved package byte stream.

The source index fields are runtime metadata written by the separately studied
`0x140bf5a30` path. Its resolved keys, prior buffer contents, selector gates and
post-callback state remain inputs. A saved material package reference does not
supply a native buffer or material pointer.

The caller also writes wrapper address `proxy.wrapper_base + i*0x3c` to
`R+0xe8`, and an unresolved wrapper referent's virtual `+0x2f8` return value
to `R+0x10c`. It writes zero to `R+0x114` and masks the u32 flags at `R+0xd8`
with `(old & 0xffbe381f) | ((s & 7) << 14)`. That callback result is not
identified by a saved material import or an opaque product key. The first
64 element bytes copy component `+0xc0..+0x100`; the next 64 come from an
unexpanded helper's temporary. Their raw dataflow is retained without assigning
matrix, inverse or coordinate-space semantics. Constructor `0x140800c50`
remains unexpanded, so unspecified defaults/counts are not inferred.

There are two different receivers. `0x140bfabc0`'s own receiver is the
class-bound object above. Its **incoming RDX** is externally supplied, saved
at `0x140bfac00`, then passed as RCX to `0x140ceb180` at `0x140bfb3f7`.
This second receiver is not established as the global graphics backend.

At the handoff, arguments three and five are the same address `0x14292a7c0`;
argument four is a 16-byte helper-produced temporary; argument six is
`qword[proxy+0x10]`; argument seven is a written DWORD zero; and argument eight
points to two zeroed qwords. Their exact addresses, widths and helper gates
are retained in the caller proof. The constant's contents and these values'
semantic types were not read or assigned.

## Concrete next dispatch

The complete `0x140ceb180..0x140cec7d8` body is 5,720 bytes. It saves the
incoming receiver in R12 and the supplied record in R13. A call to
`0x140975a10` using global `0x142c27e70` controls the first branch. When that
helper returns zero, the body immediately calls `[R12.vtable+0x38]` at
`0x140ceb204`, with RCX = R12 and RDX = R13, then returns its result.
The helper's runtime return value is not inferred from the saved map.

Across the full body there are **13** explicit calls through this same slot:
two receive the original record, at `0x140ceb204` and `0x140cec591`, and eleven
receive a temporary record. Every site explicitly reloads the same receiver
and passes the corresponding record address. This count describes static call
sites, not calls executed during a frame or startup.

The other paths depend on additional receiver/view flags, supplied values,
virtual calls and globals. They can substitute an auxiliary pointer at record
`+0x10c`, alter selected record flag bits, and in two paths change the scalar at
`+0xd0`. Their exact raw gates and instruction anchors are retained privately;
no display mode, material semantic, pass name or active branch is inferred.
The function does not itself decode vertex bytes or directly select the
Model declaration handle at resource `+0x22c`.

## Finite copied-record contract

The ten explicit calls to `0x140835310` feed the eleven temporary dispatch
sites; one copied-record path has two alternative dispatches. The helper's
complete 528-byte body copies selected outer fields, including qword
`R+0x104` at `0x1408354cd/0x1408354d4`. It invokes the complete 452-byte
`0x14082a2d0` helper for the ordered element array.

The latter reads the signed source count at `R+0xc0`, chooses the source base
`[R+0xb0]` or the inline `R`, and copies each `0xb0`-byte element as eleven
ordered 16-byte chunks. Destination selection follows the same inline/heap
representation. Under stable valid source pointers/counts and successful
ordinary storage preparation, every element's `+0x80`, `+0x88`, `+0x8c`,
`+0x94` and `+0x98` bits and order survive. Allocation/growth helper
`0x14081b890`, invalid counts, failures, overlapping storage, mutation and
exception behavior remain outside this contract.

This is not whole-outer-record byte identity: outer padding and some high flag
bits are not copied. In the inspected dispatcher, all temporary calls replace
`+0x10c` with a local replacement-record address. Four additionally OR the u32
flags at `+0xd8` with `0x10`; the last two of those also store an SSE subtraction
result at `+0xd0`. No direct pre-dispatch overwrite of the copied `+0x104`
address or inner index fields is present. Effects of intervening virtual calls
are not covered by a blanket immutability claim.

## Relation to the retained 36-byte resource

The `R+0x104` address closes the connection to the previously studied embedded
Model object, rather than merely a similarly shaped vertex buffer:

| Native member | Retained construction relationship |
| --- | --- |
| `Model+0x19c` | Saved ordinary 868/3 packed records, stride 36 |
| `Model+0x194` | Embedded array interface exposes that data and count |
| `Model+0x168` | Resource consumes the array interface under its initialization gates |
| `Model+0x1b8` | Descriptor consumer with vtable `0x141e35018`, carried by this draw record |
| Descriptor resource `+0x22c/+0x234` | Previously proved primary/alternate declaration handles |

The typed setter supplies the same source-resource identity and stride 36 for
its five selected descriptors. Their initial offsets/types are `(0,3)`,
`(12,5)`, `(16,5)`, `(20,2)` and `(28,2)`. Existing fallback streams, duplicate
coordinates, global-gated offset changes and backend mappings remain separate
conditions. This call chain has not selected a primary/alternate declaration
or passed its handle to the generic bound-state wrapper.

## Saved data, explicit product computation and unresolved state

The retained selected FrontEnd package has ten ModelComponents, twelve ordered
element-node occurrences and 48 packed 36-byte records. Existing readers preserve their
source spans, duplicate occurrences and imported material identities. These
are saved observations, not a snapshot of the native draw receiver or buffers.

`ModelIndexResource` computes bounded integer indices and element metadata only
from an explicit `AfterElementCallbacks` snapshot: resolved identity keys,
initial map/buffers and component order. `frontend::model_index_binding::prepare`
retains the selected AssetSet/archive identity and corresponding borrowed records.
Its output buffer ordinals and opaque object keys are not process addresses.
The existing hypothetical fixture outputs must not be relabeled as observed
native index buffers or startup state.

A new decoder or process-record constructor is not justified by this bridge:
the saved packages do not serialize the required receiver, callbacks, embedded
vtable objects or runtime pointer fields. Existing typed APIs already expose
the grounded saved and explicitly computed data without manufacturing them.

The next finite native dependency is the concrete invoker of proxy slot
`+0x18` and its supplied draw receiver, or an independently grounded constructor
for that receiver's `+0x38` target. The named retained evidence reviewed here
contains no such binding; this is not an exhaustive caller-absence claim.
Scale9's known bound-state wrapper route is kept separate. Selected material
and shader-variant → handle → owned bytecode-record identity is also unresolved.
No native draw, graphics API success, rendering, shader decode or runtime parity
is claimed.

## Evidence and provenance

The private evidence root is
`~/.local/share/marvel-heroes-forever-rust/evidence/model-draw-consumer/`.
The owned executable SHA-256 remains
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Two fresh target-free REA/Rizin records cover the three complete functions:
`ceb180` (5,720 bytes), `835310` (528), and `82a2d0` (452). All 6,700 bytes
and 1,475 instruction rows were independently compared to the owned PE,
including complete exception-directory extents. The exported bundle SHA-256
is `e3d31b735d75ab5db7567b9605cb6c4d1327276155efa249c7aa27f47aa9f8b6`.
Both calls completed without truncation or target changes. REA was reset after
export, leaving no open target, pending cleanup or retained records.

The fresh Evidence IDs are
`ev_0cf5068c86225bb7e2a8fba007257a60aafb7d96a3ca9690a83696790325a58a`
and `ev_084141541cbb0f6ee1de4b2a890c0b465268faaee12054c6e121fdd31e523921`.
`native-route/receipt.json` and its dispatch-gates child seal exact call sites,
copy operations and conditions. `record-handoff/receipt.json` SHA-256
`2cd6697c10e3a6458cca91b07b270a57bdd619ecbb08657140a8258293ab6882`
independently checks the retained 2,196-byte caller and 519 instruction rows,
reusing its original owned-PE/REA/class evidence without another native probe.
`source-boundary/` separates current original APIs from process-only state;
`resource-join.json` records the exact prior embedded-resource receipts.

All source paths, hashes, purposes and licence categories are registered in
private `sources.json` and the component source ledgers. The underlying owned
retail executable is proprietary and remains private. Previous original Model,
index, declaration and API studies are reused by identity; no new outside
implementation or SDK source was introduced or copied. GNU objdump and the
original private PE helpers are analysis tools only. Product source was read
from baseline `a4566d25182f1e2d376629f1968bb97ae64170f8`.

Focused document whitespace and local-link checks passed. This change adds
only this document: no product test/build, native process, renderer, shader
compiler, live data or service was run.
