# Format query before the first image-size read

**VERIFIED static operations; INFERRED conditional producer composition.**
The post-initialization format callback left open in
[SCALE9_TEXTURE_QUERY_CHECKPOINT.md](SCALE9_TEXTURE_QUERY_CHECKPOINT.md)
now reaches a concrete seven-byte getter for the selected manager/provider
profile. That getter reads a format key and performs no calls or stores.
Its returned key is stored in the image's format word, separately from its
dimension words and optional matrix.

This closes that particular callback's local effects under explicit current
identity/table/lifetime conditions. It does not close the earlier backend
and resource-update boundaries, equate the separate dimension snapshots or
establish current 64×64 dimensions. No renderer or product code changed.
All new captures remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-texture-format/`.

## The manager produces the selected provider type

Use M for the image-system texture manager, C for its shared control object,
Q for the selected texture wrapper, P for a format provider and F for the
returned image. E remains the size/resource record addressed through Q+0x5c.
These objects are distinct from the Engine.Texture2D asset and its resource.

The complete manager constructor `0x14125e8e0` calls base `0x14003a6e0`,
installs table `0x1421c2f50` at `0x14125e913`, and calls `0x141282f80` at
`0x14125ea2d`. The base clears M+0x68/+0x70/+0x78, the provider array,
count and capacity used by the later lookup. It also constructs C: on
successful allocation, `0x14003a800` writes M to C+0x10, and
`0x14003a843` stores C at M+0x48. Allocation and called constructor helpers
retain their normal-success conditions; these stores alone do not prove
current manager identity or successful runtime construction.

`0x141282f80` is complete at 243 bytes. It iterates rows beginning at
`0x1429de340`, with a 28-byte stride and a zero first-word terminator.
For each row it requests a 16-byte provider allocation. On success it
stores table **`0x1421c2da8`** at P+0 and the current row pointer at P+8
(`0x141282feb`/`0x141282fee`). It then appends P to M's provider array.
Provider allocation failure supplies a null array entry; it does not
invent a replacement provider.

The pointer-array capacity helper `0x1412561a0` is complete at 154 bytes.
It requests rounded capacity through the allocator and writes the array
pointer/capacity. It does not select a provider method or replace a
provider table. Allocation, arithmetic, valid disjoint storage and current
array preservation remain conditions; no claim about all allocator effects
is required for the later getter's local read-only contract.

The exact M-table slot +0x90 at `0x1421c2fe0` selects **`0x140037610`**.
That complete 109-byte method examines the current M+0x68 pointer array
and M+0x70 count. It skips null entries, calls each nonnull provider's
virtual+0x08, compares the result with the supplied EDX key, and returns
the matching array entry or null. It rereads array/count during traversal;
this is not a copied-array or concurrent-mutation guarantee. Its index
increments are native 32-bit operations, not an unbounded mathematical
iterator.

For the constructor-installed provider table, exact +0x08 slot
`0x1421c2db0` selects **`0x14126d040..0x14126d047`**. The complete leaf
loads the row pointer from P+8, reads its first u32 into EAX and returns.
There is no memory write, allocator, nested callback or dimension read.
The method requires a valid current P, selected table and readable row.

The finite owned data contains these row keys and the second word used
by the asset-byte mapping:

| Row | Format key | Asset-byte comparison word |
|---|---:|---:|
| 0 | 1 | 2 |
| 1 | 2 | 2 |
| 2 | 3 | 2 |
| 3 | 4 | 2 |
| 4 | 9 | 3 |
| 5 | 50 | 5 |
| 6 | 51 | 6 |
| 7 | 52 | 7 |
| 8 | 200 | 3 |
| 9 | 201 | 3 |
| 10 | 0 | 0, terminator |

Only these fields are interpreted here. The remaining row words are
retained as bytes, not promoted to additional format behavior. This is a
file-backed table observation, not proof that current memory is immutable.
Several comparison words repeat, so the original ordered search matters.

## The selected Q construction supplies the same manager

The retained concrete manager creation method `0x141268c00` loads
C=[M+0x48] at `0x141268c67` and passes C to Q's subclass/base constructors.
The base stores it at Q+0x20 (`0x14125e7e1`). The external initializer
later reads Q+0x20 and then C+0x10 (`0x14127402a`/`0x14127403a`). Together
with the newly verified base-manager construction stores, this establishes
the constructor checkpoint M → C → M. Reusing that identity during
initialization requires those pointers and the selected manager table to
remain applicable; saved asset identity alone does not establish them.
This does not establish that a particular live context virtual+0x1c8
invocation returned this manager subtype.

The concrete wrapper also supplies zero for the optional Q+0x28 override:
R9D is cleared at `0x141268dff`, passed through manager/subclass stack
arguments, then stored at Q+0x28 by `0x14125e7ea`. The initializer tests
the **current** value, so zero at construction is not a lifetime-wide
absence claim.

The retained initializer's three lookup routes stay distinct:

1. If current Q+0x28 is nonnull, call its virtual+0x20, clear result bit 20
   with BTR EDX,0x14, then call manager+0x90 at `0x141274055`.
2. If Q+0x50 is still null, select the current asset format byte using
   the retained type precedence and search the 28-byte rows by their
   second word. The first match supplies its first word to manager+0x90
   at `0x1412740d2`.
3. If Q+0x50 remains null, request key 2 at `0x1412740ee`.

Each lookup result is stored to Q+0x50. The final null result is not
rejected by this initializer. Under the constructed provider-list profile,
every nonnull returned provider has the same read-only getter regardless
of which key won. Closing that getter therefore does not require assuming
the saved PF_DXT5 name became runtime byte 7. The earlier saved-enum/runtime
encoding distinction remains; **if** the current selected byte is 7 and
the retained table applies, the row search requests key 52.

The native row loop reloads the selected asset byte on each iteration.
It does not take one atomic format-byte/table snapshot. A null provider
lookup after the first matching row proceeds to key 2; it does not try
later rows with the same comparison word.

## Exact post-initialization effects

At `0x141268e43`, the wrapper calls Q virtual+0x20. For the selected Q
table this follows the already verified chain:

`0x14017b000` → Q virtual+0x28 → `0x14126d020`.

The latter reads Q+0x50. Null returns zero. Nonnull dispatches P virtual+0x08,
which is now bound to the seven-byte `0x14126d040` for the provider profile
above. The chain reads the current provider row key; none of its selected
bodies stores to Q, E, F, the original asset or the row.

The caller saves returned EAX in EDI at `0x141268e46`, then writes it to
**F+0x28 at `0x141268e8a`**. The next stores write the earlier W snapshot
to F+0x2c/+0x30. These are different fields:

| Field | Role on this route | Effect of the format query |
|---|---|---|
| Q+0x20, including its upper half at +0x24 | Control-object pointer | No store; not a width/height pair |
| Q+0x30/+0x34 | Original-size snapshot | No store |
| Q+0x38 | Numeric initialization state | No store; distinct from virtual+0x38 |
| Q+0x40 | Normalization selector | No store |
| Q+0x50 / Q+0x5c | Provider / E pointers | Provider is read; no store |
| E+0/+4 | Separate size snapshot | No read or store |
| F+0x20 | Optional image-matrix pointer | No store |
| F+0x28 | Stored format key | Caller stores the returned key |
| F+0x2c/+0x30 | Earlier W snapshot | Separate caller stores; no dimension is obtained from the format key |

**INFERRED bounded effect closure:** with valid current Q/P/row identities
and these selected tables, the post-initialization format call cannot
change the stored dimension pairs, normalization selector or image matrix
through its own instructions or callbacks. No assumption of equal pairs,
a particular row key or 64×64 values is needed for that local statement.
This does not exclude unrelated concurrent writes or a substituted table.

The query checkpoint still requires normal successful F/Q construction,
valid lifetime through the relevant reads, disjoint output storage and
appropriate per-getter stability. The separate retain/release operations,
queued initialization continuation, earlier resource/backend callbacks,
and current dimension provenance retain their prior conditions. The
format lookup does not drain queues or make the first image-size getter
and later normalization getter one uninterrupted interval.

## Evidence and sources

The new primary source is the owner's private build 1.52.0.1700 executable,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Licence: proprietary; private interoperability study only. No outside
engine/parser implementation was consulted or copied.

`rea-format-provider.json`, Evidence
`ev_e6048c2f428095d2a25565da41720ae3298811910c435702e0d7933fbfb207df`,
retains the six complete function spans, two exact virtual slots and the
11-row table including its terminator. REA target-free Rizin was used;
original PE/unwind navigation and bounded instruction checks located and
verified those spans. The executable was not launched.

Focused independent validation passed: **303 instructions/1,236 code
bytes**, plus **324 data bytes** across all nine exact requested arrays.
It checked pinned-PE byte equality, Evidence identity, complete/untruncated/
unchanged flags, full raw/normalized result equality and unchanged source
and capture files. The sealed receipt is
`verification/runs/2b122d38271a02910635/capture-validation.json`, SHA-256
`7ef0dbb31f75940eb58d062aaa3dc267da1f0bd7927c0b6760f0e7b86fa1b913`.

`producer/receipt.json` binds the independently composed retained
construction/control, override, initializer and caller-store evidence.
Its checks cover 613 retained instructions/2,231 bytes, five table slots
and 63 anchors; receipt SHA-256
`9f1ca6daec2ea85a298ae2faabbedc4cc79256948b0218c385195bde870e2583`.
`source-ledger.json` records exact reused paths, hashes, licences and uses,
including original analysis helpers and the REA skill's operational
instructions. `final-index.json` binds the final document, private
validation receipt and exported Evidence bundle. All underlying game
bytes, rows and disassembly remain outside git. No product build, native
runtime, network, server or GPU action was performed for this increment.
