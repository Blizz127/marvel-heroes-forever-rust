# Initial texture-query checkpoint

**VERIFIED static operations; INFERRED conditional checkpoint composition.**
This continues [SCALE9_TEXTURE_UPDATES.md](SCALE9_TEXTURE_UPDATES.md). The
selected mip-size helper is now bounded through its concrete element-size
method. The first image query can be described using separate stored
snapshots, without requiring a claim that a Texture2D is immutable for its
whole lifetime. Pre-query backend calls and current scheduling remain
explicit conditions on how those snapshots were obtained.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-texture-query/`.
No renderer option, current-dimension default, product code, native runtime,
network, server or GPU was changed or run.

## The selected mip-size helper is a bounded read

`0x1407450b0..0x1407450c8` is complete at 24 bytes. It reads dword M+0x0c,
calls the selected mip's virtual+0x10, multiplies EAX by the saved dword
with a low 32-bit IMUL result, and returns. Its local body writes neither
the mip nor the original Texture2D. It is not a dimension query and does
not clamp a negative or overflowed count.

The concrete loading route is now tied to its table. The already retained
Texture2D serializer passes O+0xfc, the archive and O to `0x1409c0060` at
`0x1409c0835`. That complete 887-byte array serializer, on its archive+0x14
loading branch, reads the count, allocates 0x48-byte mip descriptors and
stores their pointers in the supplied array. For each nonnull newly allocated
descriptor, it calls `0x14073a830` and then explicitly installs table
**`0x141ecb480`** at `0x1409c01b4`. It subsequently serializes each mip's
bulk data and its dwords+0x40/+0x44. These are descriptor fields; they are
distinct from O+0x180..0x18f.

That table's exact +0x10 slot at `0x141ecb490` selects the complete six-byte
leaf `0x140210e00`, which returns 1 without reading memory or calling anything.
For a selected loaded mip retaining this table, `0x1407450b0` therefore
returns the bit pattern of M+0x0c and has no callback capable of receiving O.
Together with valid disjoint mip/object storage, this closes the selected-mip
receiver left open by the preceding resource-size accounting investigation.
It does not prove arbitrary substituted mip tables, bulk serialization,
reallocation or backend operations are effect-free.

The loading branch and successful allocations are conditions. The existing
non-loading branch serializes supplied descriptors rather than installing
this table. The source file's saved mip count alone does not establish a
live descriptor/table or guarantee later table preservation.

## Two reads produce different snapshots

Use these names to avoid conflating the two native resource structures:

- **O**: the original Engine.Texture2D object.
- **S**: the native resource pointer stored at O+0xd8.
- **Q**: the engine texture wrapper created for the image system.
- **E**: Q's inline record, addressed by the pointer at Q+0x5c. The
  constructor sets that pointer to Q+0x64. E has copied dimensions at
  E+0/+4 and the asset pointer at E+0x10.
- **F**: the returned image, with its own dimension words at F+0x2c/+0x30.

The selected custom image creator calls `0x1407b7710` at `0x1412772f0`,
moves its returned object into the wrapper argument at `0x1412772f5`, then
calls `0x141268d60` at `0x1412772ff`. Thus direct work performed before the
load call returns is upstream of both image/Q queries on this route.

The wrapper performs its Texture2D query at `0x141268d86` and Dynamic query
at `0x141268d91`. On the successful Texture2D branch, its first size
snapshot **W** reads:

| Order | Native load | Word |
|---|---|---|
| 1 | `0x141268d9b` | O+0x184, height |
| 2 | `0x141268da2` | O+0x180, width |

These separate reads are not an atomic pair. They precede the context
virtual+0x1c8 call at `0x141268ded` and manager virtual+0xd0 call at
`0x141268e0d`. The retained register values are later saved unchanged at
F+0x2c/+0x30 at `0x141268e8d`/`0x141268e90`. They are not reread from O
after manager creation and are not replaced with Q's dimensions.

Q's initializer later takes another snapshot. On its successful Texture2D
branch it reads height O+0x184 at `0x141274105`, then width O+0x180 at
`0x14127410b`, storing the selected size in E. These reads follow selected
format-manager virtual+0x90 calls at `0x141274055`, `0x1412740d2` or
`0x1412740ee`. The initializer separately retains the OriginalSize pair in
Q+0x30/+0x34 and selected byte state as described in
[SCALE9_TEXTURE_INPUTS.md](SCALE9_TEXTURE_INPUTS.md).

Consequently W and Q's size snapshot need not be equated merely because they
originate from one O. Equality requires the relevant field/identity stability
over both read intervals and intervening calls. An owned saved 64×64 tuple is
not evidence of that stability. The supplied source dimensions, returned
image dimensions and normalization dimensions remain separate inputs.

## Q initialization can run directly or through service work

The concrete manager+0xd0 method `0x141268c00` calls Q constructor
`0x14125e5d0` at `0x141268c6e`, then `0x140037480` at `0x141268c7d`.
The helper queries manager virtual+0x38 at `0x1400374df`. Its direct branch
reaches Q virtual+0x08 at `0x140037506`, selecting the retained initializer.
The other branch links Q to pending storage, invokes service virtual+0x08
at `0x140037579`, and waits while **state field** Q+0x38 is zero using
`0x1400305c0`. It rejects state 1 at `0x1400375d6`; other nonzero states
can return Q. The checkpoint below additionally requires successful
initialization by the selected concrete initializer, which writes state 2.

This state field is distinct from **Q virtual+0x38**, the normalization
getter. A constructor alone does not show that initialization has completed,
and the current thread/global state is not inferred from saved assets.
Service invocation and waiting also prevent treating a scheduled callback
as necessarily later than the first query.

## Backend work is reachable before the first snapshot

The earlier complete PostLoad route remains conditional on its queue and
flags. When reached, the selected resource-update method `0x1409c6e60`
polls virtual+0x2c8 at `0x1409c6e77`/`0x1409c6e92`. Its concrete poll can
call `0x1409a9080` at `0x1409c6f7d` for state 0x64, or `0x1409a8f80` at
`0x1409c6fdf` for state 2 with the actual caller's EDX=0.

The first wrapper directly calls worker `0x1409b9090` at `0x1409a915f`
when global `0x142c27978` is zero. The second can directly call finalizer
`0x1409b1760` at `0x1409a8fa9` after its thread predicate, or at
`0x1409a9071` when the global is zero. The worker has reachable backend
virtual+0x28 and +0x10 calls at `0x1409b92c6` and `0x1409b931f`; their
arguments include the address of the selected O+0x1c0 counter. These are
possible synchronous calls before the loader returns, not merely future
queue entries.

Alternate stored-wrapper publication does not establish when the wrapper
executes relative to W or Q's later read. No queue-drain, current-thread,
absence-of-worker or dimension-preservation property is invented here.

One additional exact pre-W boundary remains: after the resource continuation
stores the new S at O+0xd8 (`0x1409c6f13`), its nonnull path tail-calls
**`0x140958a00(S)` at `0x1409c6f27`**. That body is not analyzed by this
increment. Its call is before the surrounding load can return normally;
omitting it would create an unjustified load-to-snapshot preservation claim.

The old-resource branch also calls `0x1409646a0(S)` at `0x1409c6ec5` and
can dispatch S virtual+0 at `0x1409c6ede`. These exact release boundaries
remain unexpanded. The concrete manager callbacks and selected mip helper
are bounded separately; that does not close every backend or release call.

## Completed initialization is an intermediate checkpoint

After its last size stores and the existing both-OriginalSize-words-zero
fallback, `0x14127415f` writes **state field Q+0x38=2**. Call this point C.
At C, E+0/+4 and Q+0x30/+0x34 contain separately obtained snapshots. W
remains the earlier wrapper snapshot. No property of C equates these pairs.

On the inline path, the initializer epilogue and helper continuation to
the manager return have no local O-dimension reread or E/Q-pair store.
The concrete caller supplies R8D=0, so the helper's bit-8 continuation is
skipped. The waiting route additionally requires the actual worker's
completion, synchronization and continuation effects; observing only a
nonzero state would not establish those effects or select this initializer.

The wrapper still makes three Q-receiving calls after initialization:

| Call site | Selected operation | Bounded effect or remaining edge |
|---|---|---|
| `0x141268e43` | Q virtual+0x20 → `0x14017b000` | Tail-dispatches Q virtual+0x28 |
| `0x141268e7a` | `0x140031ce0(Q)` | LOCK XADD increments dword Q+8 |
| `0x141268ea7` | `0x140031cf0(Q)` | LOCK DEC of Q+8; zero result can call Q virtual+0 with EDX=1 |

The selected table `0x1421c2dc8` has +0x20=`0x14017b000` and
+0x28=`0x14126d020`. The latter complete 19-byte body reads Q+0x50.
Null returns zero; nonnull tail-dispatches **that format object's
virtual+0x08**. Thus the exact remaining post-C callback is
`[QWORD[Q+0x50]]+0x08`, with QWORD[Q+0x50] as its receiver. Its selected
implementation and effects are not established here. The complete retain
and release bodies are 11 and 30 bytes. A nonzero post-decrement count
avoids the destruction callback; valid lifetime and count arithmetic remain
conditions rather than invented fresh values.

These calls prevent calling C an unconditional mutation barrier. A more
useful adapter checkpoint is **normal successful wrapper/creator return**,
with a valid F and the Q stored at F+0x10, after accounting for the effects
of all calls actually taken. At that checkpoint F+0x2c/+0x30 store W; Q/E
hold their then-current values. The saved file alone does not provide this
checkpoint state.

## Inputs that can drive the bounded query math

The successful creator return is followed by image virtual+0x28 at
`0x1400921ec`. For the selected image table, its complete getter
`0x1400cb740` reads F+0x2c at `0x1400cb740`, then F+0x30 at
`0x1400cb745`. It reads no O dimension and does not substitute E's pair.
These are separate per-axis reads, not an atomic snapshot.

The later image+b0 routine `0x1400372d0` obtains Q using image+a8, then
calls Q virtual+0x38 at `0x140037316` or `0x140037333`. The selected
`0x14126ea10` chooses Q+0x30/+0x34 when Q+0x40 mask 1 is set, otherwise
the pair reached through Q+0x5c. It loads width before height, converts
each zero-extended u32 separately to f32 and divides 1.0 by each result.
There is no O-dimension read, mip selection or zero guard in this getter.
The output destination must not alias the selected input state. The
subsequent matrix multiplication retains its recovered operation order;
these observations do not justify algebraic ratio cancellation.

**INFERRED conditional adapter contract:** an implementation can accept
explicit current F/Q/E identity, the F size pair, Q original-size pair,
E size pair and Q+0x40 selector, each with its checkpoint provenance. It
must keep the image-size and normalization-size inputs distinct. To reuse
checkpoint values at each getter, require pointer/selector and relevant
per-axis field stability through that getter's actual reads, successful
lifetime, valid disjoint storage and the selected method tables. This
admits earlier backend work whose effects are included in the checkpoint;
it does not require a claim that such work was absent or deferred.

The binder's first image-size getter is ordered after the creator return.
A direct uninterrupted path to the first later normalization getter is
not established: binding/cache work and later dispatch intervene. Reusing
the same Q/E values for that later call therefore has its own interval
condition. Nothing here asserts firstness in wall time, an empty queue,
current thread identity, equality of the three pairs or lifelong field
immutability.

For the actual `MarvelHUD.Tooltip_Main_BG_PNG` asset, saved Size and
OriginalSize fields remain separately verified as 64×64 by the earlier
asset receipt. This increment does **not** promote them to current W/E/Q
values. The exact pre-W backend, old-resource release, `0x140958a00`
initialization and post-C format virtual+0x08 boundaries remain visible.
No automatic 64×64 adapter or Scale9 renderer enablement follows.

## Evidence and source ledger

The only new artifact source is the owner's private build 1.52.0.1700
`MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Its licence is proprietary; use is private interoperability study. REA
target-free Rizin captured exact requested instruction/data spans; original
PE/unwind navigation only located and independently checked those spans.
No external engine or parser implementation was read or copied.

New canonical records:

- `rea-selected-mip.json`, Evidence
  `ev_897a476fcece0bc1be1034a9053d6362b8c206d6210426d3f096f6c0e776011b`:
  the mip helper, loading array serializer, selected constant-return method
  and exact element-size table slot.
- `rea-postinit-q-calls.json`, Evidence
  `ev_a1812ee35c26b065cf9ee0ed14bd3853a097bf01e4da5c4b796933381d06312a`:
  the four small Q call-chain bodies and two exact selected table slots.

The original composition receipt `prequery-order/receipt.json`, SHA-256
`df5b51f90d701fbf3bfac7c71ea6d00d5aeca3e85e7274632d6bd47c97de29a2`,
binds 11 retained canonical records, 22 assembly extracts, 3,520
instructions/14,341 instruction bytes, 50 instruction anchors and 12
table slots. Its source ledger names every reused record under
`external-image-route`, `scale9-texture-inputs`, `scale9-texture-load`,
`scale9-texture-resource` and `scale9-texture-updates`. It predates the
additional small Q-call capture above, which closes those local bodies
while retaining the format/destruction edges explicitly.

Focused independent validation of the two new records passed: seven
complete code arrays, **269 instructions/984 code bytes**, plus two data
arrays/**24 bytes**, all matching the pinned PE. Requested ranges and
Evidence IDs, source digest/stability, complete/untruncated/unchanged flags
and normalized/raw output equality were checked. The sealed receipt is
`verification/runs/0910b5ba1bb5fdaedad5/capture-validation.json`, SHA-256
`0fe9c0456ec4a9fa307448883e40b53f60e84f84740e0b6fc80ab33e6c5eb478`.
No executable was launched and no product build or GPU test was run for
this documentation-only increment.

`source-ledger.json` records exact source/helper paths, hashes, licences
and uses, including the REA skill's operational instructions. Reused
project analysis and validators are original project work; game bytes and
all derived captures remain private. `final-index.json` binds the final
document, independent byte-validation receipt and exported Evidence bundle.
