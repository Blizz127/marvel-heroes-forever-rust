# Draw-record mesh identity through ordinary upload

The native preparation driver constructs its upload collection directly from
the **second pointer of each 16-byte draw record**. It supplies the same batch
to the upload callback. Combined with the retained ordinary receiver evidence,
this establishes the pointer projection needed to associate uploaded geometry
with the mesh's retained `+c0` inverse-normalization matrix. It does not select
a live device, resource, destination format, matrix environment or pixel shader.

This is a static, conditional path proof using the owner's x64 1.52.0.1700
executable. No retail process or GPU was run. The independent 88-word matrix
comparison and original position-transfer implementation remain unchanged.

## Names and prerequisites

Addresses omit preferred image base `0x140000000`. P denotes a record/batch
owner, B a batch, Q the preparation state and M a mesh pointer. These are
concrete argument/field roles, not recovered public C++ class names.
R below specifically denotes the engine renderer initialized by `1202f50`.
This study does not equate R with the separate `071960` options owner or with
M's `+50` field merely because earlier studies used similar role names.

The selected route is an ordinary upload through the concrete cache-owner
callback `1255030`, with the staging branch already described in
[SCALE9_OUTPUT_RECEIVER.md](SCALE9_OUTPUT_RECEIVER.md). Existing cache hits,
direct output, unsupported provider types and failed allocations are not
silently treated as new staging uploads. This study does not establish that
any particular scene currently selects this route.

## Batch ownership and destination-format assignment

**OBSERVED:** factory `03aa60(P, kind, first, count)` allocates a 0x50-byte batch
and stores the exact incoming P at `B+40`. It stores kind at `+28`, first at
`+4c` and count at `+48`, and initially clears `+20`, `+30` and byte `+38`.
The proof assumes a returned allocation; the factory does not supply a Rust
allocation-failure policy.

Preparation `079de0` obtains P from an existing batch's `+40` and passes that
pointer unchanged into `03aa60` when splitting a batch. It links the new batch
into the existing list and adjusts the old batch's first/count fields. This
preserves owner identity rather than associating batches by equal contents.

The same preparation body assigns the resulting `B+30` destination format:

- For a one-record ordinary batch, it selects `Q+30`.
- For the selected multi-record ordinary batch, it selects `Q+38`.
- Numeric batch kind 2 selects `Q+40` on its distinct path.

These are concrete pointer assignments at `07a15c..07a171` and
`07a1c2..07a1e1`; they do not by themselves prove any format equals packed
descriptor `142adb240` or contains a particular position code.

Initializer `079bb0` establishes where these fields come from. It saves its
incoming P at `Q+20`, renderer argument at `Q+10`, and cache-owner argument at
`Q+18`. It reads `F = [[P+20]+30]`, saves F at `Q+28`, and, for nonnull F,
calls renderer virtual `+1f0` with F, the numeric fill field `[[P+20]+10]`,
and output pointers `Q+30`, `Q+38`, `Q+40`. The concrete table/mapper binding
below identifies this callback. Null F clears all three output fields.

Consequently the source format passed into this mapper is exactly the F later
read by `1255030` through `B+40=P`, and the selected mapper output is exactly
the G later read from `B+30`. This is a pointer-producer association, not a
claim that every F or G equals the packed descriptor or a particular code.

## Exact record-to-upload projection

**OBSERVED complete driver `07a270`:** on a batch requiring upload, it uses
the current B from `Q+50` and constructs this stack header:

```text
P     = [B + 0x40]
first = B.u32(+0x4c)
count = (B.u32(+0x28) == 2) ? 1 : B.u32(+0x48)
C.data   = [P + 0x40] + 16 * first + 8
C.count  = count
C.stride = 16
```

The instructions at `07a2e0..07a316` construct the header. At `07a405..07a418`
the driver passes entry RCX=`[Q+18]`, RDX=the **same B**, R8=`&C` and R9B=its
current flag to the cache owner's virtual `+48`. The header's dword at `+18`
also contains a pointer-derived hash; this does not replace or reorder its
mesh pointer sequence.

For an admitted, in-bounds ordinary batch, iteration i therefore reads:

```text
C[i] = [[P+0x40] + 16*(first+i) + 8] = record[first+i].second
```

The prior canonical `1255030` passes that incoming C unchanged to `03eeb0`.
The latter selects `C[i]` for receiver `R+18`, and the ordinary begin callback
copies its inverse-normalization matrix into that exact M's `+c0..dc`.
Upload subsequently obtains vertex/index offsets and counts from the same
`C[i]`. Neither equal offsets nor a matching vtable slot number substitutes
for this explicit pointer chain.

There are earlier cache-hit paths in `07a270`: a nonnull `B+20` skips this
upload, and a matching cached mesh-pointer sequence can install an existing
cache association. The new-upload path is not asserted for those branches.
Upload callback failure sets `Q+48 = 1` and returns 1; this receipt does not
replace that with successful rendering.

## Count helper and batch-kind boundary

The complete `03b6e0..03b80e` count helper independently reads the same P and
the same second-pointer projection. It sums M's `+44` vertex and `+48` index
counts with native u32 operations. Before doing so, it may repair batch first
indices using P's linked list at `+28` and cached count at `+58`.

Kind 2 counts and submits only one mesh even when its batch record count is
larger. Preparation contains a separate repeated-pointer grouping path.
Consequently this document's ordinary one-record/multi-record association
must not be turned into an unsupported general instancing implementation.
Indices, record bounds, live allocation validity and mutation ordering remain
part of the caller's required native state.

## Renderer and format callback identity

Constructor `1202f50` installs primary table `1421ba230`. Its `+110` cell is
the retained draw function `1246580`; its `+1f0` cell is wrapper `12541d0`.
The wrapper adds `0x650` to its renderer argument and tail-jumps to the already
recovered destination-format mapper `12541e0`.

The constructor also invokes the verified cache-owner constructor `1203560`
on renderer `+314`. That constructor installs table `1421ba140`, whose `+48`
is `1255030`. These are actual constructor/table bindings, not guesses based
on adjacent pointer cells. The active global GPU device remains unknown.

The existing draw body computes its matrix-record start from the incoming P's
record array plus `16 * B.first`, and the matrix consumer uses each record's
second pointer plus `c0`, as detailed in
[SCALE9_POSITION_RECONSTRUCTION.md](SCALE9_POSITION_RECONSTRUCTION.md).
The packed-word conversion/declaration contract remains separately documented
in [SCALE9_PACKED_VERTEX_FORMAT.md](SCALE9_PACKED_VERTEX_FORMAT.md).

## Queue owner and draw forwarding

The bounded leaf initializer `03d1d0..03d25a` installs table `1424fd5f0`
at object `P+10`. It also stores its supplied fill pointer at `P+20`,
initializes the list at `P+28`, and clears the record pointer/count fields.
That table's `+08` callback is `07a590`, and its `+10`
callback is `079db0`. Both callbacks subtract `0x10` from their incoming
interface pointer, recovering the same P. The table cells and installation
instructions are captured directly; upstream allocation and record-insertion
callers are not inferred from them.

`07a590` admits its route only when context dword `+2c` is zero. When its key
differs from the preparation state's current key, it calls `079bb0` with
Q=`context+48`, P as recovered above, renderer=`[context]`, and cache owner=
`[context+8]`. It otherwise resumes the existing Q. The association here is
for the initialized P/key and its valid resumed state; arbitrary replacement
of a key or memory behind Q is not supported by this proof.

`079db0` separately requires context dword `+30` zero, then passes that same P,
Q=`context+48`, emission cursor=`context+108`, and renderer=`[context]` into
`079b30`. The latter preserves incoming P in RBX and tail-dispatches renderer
virtual `+110` with **RDX=P**, R8=selected first batch and R9=selected stop.
It does not replace P with a resource, cache entry or a reconstructed scene ID.
The concrete renderer table binds this call to `1246580`.

On a new emission key the start cursor comes from `P+30`; on the matching
preparation key the stop is `Q+50`. The other-key path takes its stop from
P's list. This preserves partial preparation/emission boundaries rather than
inventing an unconditional draw of every record. The raw context gates and
key/cursor behavior are retained without assigning unproved lifecycle names.

Thus, for the same prepared ordinary batch and stable record sequence, both
the upload projection and the draw matrix reader address the same M. This is
a downstream pointer proof: it does not require guessing which source-cache
getter or insertion slot originally populated the record array, nor does it
establish those upstream insertion semantics.

## Concrete context-to-cache binding

Renderer construction calls `0432f0(context=R+508, queue=R+70, renderer=R)`.
This 49-byte leaf stores R at `context+0` and **clears** `context+8`; a fresh
constructor alone does not make its cache pointer usable.

The later `043330` initializer obtains the cache through renderer virtual
`+1d0`. The exact installed renderer table selects the eight-byte getter
`1247170`, which returns `R+314`, the same object constructed by `1203560`.
Renderer virtual `+1e0` selects `1247420`, returning `R+508`. The previously
retained caller `03fd30` obtains that context and calls `043330`; it was
correctly rejected earlier as a direct upload caller, but this specific
context-initialization call now has its pointer targets bound.

`043330` invokes `03de80(cache=R+314, destination=&context+8)`. When the
cache's stored destination differs, `03de80` writes the **exact incoming cache
pointer** into that slot at `03deaf`. It also replaces the cache's back-pointer,
clears the prior slot and its associated flag bit when present, and updates
a current flag bit through cache virtual `+30`. The same-slot branch preserves
an existing valid binding; it must not be turned into a general repair of
arbitrarily overwritten context memory.

The concrete cache table's `+30` callback `1205cb0` only reads byte
`cache+1c8`; it does not replace the slot after that pointer store. Its `+10`
callback used earlier by `043330` is `210e00`, which returns numeric 1.
Those exact table cells are retained in the earlier cache-constructor record,
and both small callback bodies are independently captured here. No unexpanded
virtual callback is assumed to preserve the new pointer.

After that native binding path, the normal queue entry passes the following
identities to `079bb0`:

| Stored field | Exact source |
| --- | --- |
| `Q+10` renderer | `[context+0] = R` |
| `Q+18` cache owner | `[context+8] = R+314` |
| `Q+20` record owner | incoming queue-interface pointer minus `10` |

The cache object's constructor-installed table binds the driver's virtual
`+48` to `1255030`. This completes the ordinary upload identity chain; the
null context-constructor state, explicit rebind behavior and resumed-state
requirements remain visible rather than assuming a cache from a filename.

Both preparation and draw may call `03acf0` to refresh batch first indices.
Its selected complete leaf walks P's batch list, subtracts each batch count
from the total to write `B+4c`, and updates `P+58`. It does not rewrite the
record array or mesh pointers. With unchanged counts/list it is idempotent;
this does not justify reusing uploaded bytes after a later record mutation.

## Evidence and remaining scope

All shipped-artifact study uses the private executable SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Private records are retained under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-draw-mesh-identity/`.
No native bytes, third-party implementation or SDK source enters git; this
increment changes documentation only.

| Private file under `native/` | REA Evidence ID | Selected scope |
| --- | --- | --- |
| `rea-batch-count-collection.json` | `ev_ac286c878afa68a6885ab203507e3cc699750f42c3d895778e126757d94c22d3` | Count-helper entry fragment |
| `rea-batch-count-tail-and-format-wrapper.json` | `ev_06441f91214865208e611b2795f85b789cfe699c6906665fb5a878c77102d805` | Remaining count-helper fragments and exact format wrapper |
| `rea-batch-preparation-candidate.json` | `ev_2b0b7586b3e5d1bc6ed93bdde855c5439cf7633f25700e9568444a8964851d85` | Merge path and concrete wrapper pointer cell |
| `rea-preparation-and-renderer-constructor.json` | `ev_bfcda7c78796207070bd0573de776725d18a2d7c0a586bafd12676a12cf12f13` | Batch splitting/format assignment; renderer and cache construction |
| `rea-preparation-owner.json` | `ev_5fffb05ba8e2a1e9585e9d76f6c02ea816df99c88259eb7e712c9d2a64ac654c` | Exact upload collection header and dispatch |
| `rea-batch-factory-and-queue-entry.json` | `ev_a8ea291af968aa8494e548851363d544dba5d88ea4427012e71aafd378f7b9f6` | Batch owner storage and both queue preparation entry paths |
| `rea-queue-initializer.json` | `ev_3056aac4edb2e5d9632b6fbcbb4ebcf2a8f5ec479e711da9f7c2acbe57c8ec9f` | P/renderer/cache preservation and source-to-destination format callback |
| `rea-context-and-draw-entry.json` | `ev_eadbcfc6c2d556fea12de01b7d3093b8f8da7538f19e730cb085387676563c38` | Context construction, paired draw entry and interface table |
| `rea-context-cache-and-draw-forwarding.json` | `ev_f35dfafacb10372ced438d6171f5c91df59c467907a229311ca2f4ceef280798` | Context initialization, renderer getters and exact P forwarding |
| `rea-cache-binding-and-index-refresh.json` | `ev_113683111eec6e31243c1716704406f6cbdcd6f02ac922844e7dad3c7876cef5` | Exact context cache-slot write and batch-index refresh |
| `rea-record-owner-constructor.json` | `ev_6f2b958eca2812e6cf63af7acb4509b4224320084a670014f033b0c5de77c705` | Complete selected leaf initialization of P and its embedded interface |
| `rea-cache-binding-flags.json` | `ev_e252f6109bab3d05c44819861d6f4c7857c9e17255cd692b0746e94a6bbac6a9` | Concrete cache flag callback and constant getter |

All twelve commands completed with exit 0, no output truncation and unchanged
target hash. They were exported to `native/final.bundle.json`, SHA256
`9c21508becb857c5648add1fb2f1ea196ab8938bacd25c5cc5796bc6892e658c`,
then reset with no retained native target. No product implementation changed,
so no build, renderer test or repetition of the previous arithmetic tests was
needed or claimed for this documentation increment.

**VERIFIED byte identity:** independent PE mapping checked 1,085 decoded rows,
3,994 distinct code/alignment bytes and 64 distinct dumped data bytes across
the twelve new captures. The eleven-byte constructor-store overlap is counted
once; twenty trailing `int3` bytes are classified as alignment. There are zero
invalid rows, byte mismatches or selected-window boundary crossings. All
capture records match the exported bundle. The earlier cache-table cells and
the retained `03fd30 → 043330` call span were also rechecked without modifying
their files. Final audit receipt `verification/receipt.json`, SHA256
`fe0bc4c27a1d6a3b70098ee6bce0385ae3291f01802c0ef986aca78baa668897`.

The first count-helper selection covered its 141-byte entry fragment. Its
separately captured 161-byte continuation completes the contiguous 302-byte
family through `03b80e`; the entry fragment alone is not a complete function.
The raw mapping base for these code/readonly-data captures is `0x140000c00`.

`reuse/reuse-ledger.json` indexes the retained ordinary receiver, upload and
draw proofs plus the new count helper: eleven canonical sources and eighteen
checked spans, SHA256
`b6e4b9ee91bdc1c115f3890e07d50d64380f119428202c2ca3a475982e564d3c`.
This ledger distinguishes the formerly missing pointer join from the new
driver evidence above; earlier sealed receipts were not rewritten.

Two earlier records also supply exact auxiliary bindings without recapture:
`scale9-position-reconstruction/native/rea-cache-owner-constructor.json`,
Evidence `ev_fa5b6d09c9b713a6fa9fdbf4224e950aaa38dc14a911b1458ec0e020193237b6`
(SHA256 `b06c8d9ec29f5963cdc69c461f41482e89f47467b857ff713e7c1e55c0175d0b`),
and `rea-upload-collection-join.json` in that same directory, Evidence
`ev_f2a15ceebc482f2f18321c4ae6fc0ebddc195d5e0f47b9594d851c3cadfaaec7`
(SHA256 `0ce99bc6fb99c3dd10a5bc1adc35884f1402e077fe841b0f138fc7cebfa49f49`).

Exact descriptor-reference navigation is separately retained in
`source-format-navigation/receipt.json`, SHA256
`d4f642684a43e2ff88e52702b8d3aefbef7616ee85ab4e2a1cfa74ece1c1d504`.
Its sixteen exact references do not prove that the selected upload's nested
source-format field equals `142adb240`. A candidate writes that descriptor
into an output record at `+18`; the outer-object relationship remains unbound.
The search stops there rather than treating the field offsets as identity.

Runtime source/destination format choice, packed-coordinate interpretation by
the active GPU, mesh provider/AA profile, texture/resource selection, selected
matrix handles, environment matrix and final pixel comparison remain explicit
limits. No renderer enablement, arithmetic change or product hook is included.
