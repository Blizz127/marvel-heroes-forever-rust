# Fresh state-bag return and conditional state-4 absence

**VERIFIED bounded static construction:** on the successful fresh path of
`0x1400515a0..0x140051831`, the newly allocated backing state bag has a null
parent and receives actual state categories **2, 11, 22 and 23**, in that order.
The category comes from each constructed state's `+0x10` field, not the numeric
registration argument. The category-23 member-array initialization is separate
from the state map and does not manufacture category 4.

**Conditional conclusion:** with the ordinary successful allocation, reference,
lock and no-reentry profile below, and the separately verified state-map
preservation contract, state 4 is absent from this fresh backing bag at the
constructor's successful return. The local receipt does not certify that
separately owned map contract: its final verification must be joined before
this condition is used as an admitted return profile. This advances the earlier empty-bag checkpoint
in [REA_MOUSE_HOST.md](REA_MOUSE_HOST.md) past default construction. It does
not establish a current MovieImpl cache, preserve absence through the later
loader/engine, prove a current Mouse handler missing, or implement its diagnostic
or a cursor effect.

The owned Win64 build 1.52.0.1700 PE SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All findings are static. No game process, original script, service or host
callback was executed.

## Exact objects and return checkpoint

Use distinct identities:

| Symbol | Native role |
| --- | --- |
| O | Caller-supplied outer object initialized by `0x1400515a0` |
| C | Fresh 0x58-byte backing state bag, published at O+0x20 |
| I | Secondary state-bag interface C+0x10 |
| S | One constructed typed state retained by the backing collection |

The outer constructor initially clears O+0x20 and O+0x28. An optional input
can supply an associated reference later held at O+0x28; that field is **not**
the backing bag's parent. The fresh allocation at **0x140051656** becomes C.
Its primary table is **0x1424fe438**, secondary table **0x1424fe410**. The
constructor clears **C+0x20 (parent)** at **0x140051695**, and **C+0x28 (map)**
at **0x140051699**. Following the ordinary lock initialization at C+0x30, it
again writes a null parent at **0x1400516b0**, and publishes C into O+0x20 at
**0x1400516c7**. No later direct constructor store changes that parent.

The successful final local release is at **0x140051810**. The epilogue restores
registers, places the original O pointer in RAX at **0x140051824**, and executes
**RET at 0x140051830**. The complete function ends at exclusive address
**0x140051831**. The checkpoint means the caller has returned from that RET
with this same fresh C and completed default registrations; the exclusive end
address is a coverage boundary, not an instruction executed after return.

A null C allocation takes an early return with O+0x20 null. That is not the
nonnull fresh-bag profile. In particular, final state allocation failure is
not uniformly a harmless skipped default: if the 0x30-byte category-23
allocation at **0x1400517d3** returns null, **0x140094320** is still called
with null at **0x1400517e9** and reads `[RCX+0x20]`. This study admits successful
nonnull allocations; it does not invent allocation-failure parity.

## Actual default categories and registration

The complete outer body contains four default registration routes:

| State category, decimal (hex) | Actual constructor store | Constructor/default caller | Registration |
| --- | --- | --- | --- |
| 2 (0x02) | `0x140052dd1`, state+0x10=2 | `0x140052eb0` from `0x140051714`, constructs through `0x140052db0` | Supplied I+0x10 inside the wrapper |
| 11 (0x0b) | `0x140067231`, state+0x10=0x0b | `0x140067210` from `0x140051744` | I+0x10 at `0x140051765` |
| 22 (0x16) | `0x1400940f0`, state+0x10=0x16 | `0x1400940f0` from `0x140051791` | I+0x10 at `0x1400517b2` |
| 23 (0x17) | `0x140094280`, state+0x10=0x17 | `0x140094280` then `0x140094320` from `0x1400517de`/`0x1400517e9` | I+0x10 at `0x140051805` |

The first separate **0x10-byte** allocation at `0x1400516e5` is a reference
payload passed to the state-2 wrapper. It is not a state with category 0x10,
and no category is inferred from its allocation size. The state-2 constructor
retains that nonnull payload. The category-11 constructor is called with a
null payload, bypassing its optional payload retain. All four state constructors
initialize their own reference count to 1.

I's concrete +0x10 slot is **0x140050a60**. On a nonnull state this wrapper
retains the supplied pointer at **0x140050a8b**, forwards it to **0x140050770**
at **0x140050aa1**, then releases its temporary reference at
**0x140050aa9**. Registration obtains the stored category from the actual
state object. The caller's EDX category is relevant to the separate null-state
removal route; it is not sufficient evidence for a nonnull state's identity.

The outer releases its payload/state local references at **0x140051721**,
**0x140051770**, **0x1400517bd** and **0x140051810**; the state-2 wrapper also
releases its constructed local state. The common **0x140031ce0** atomically
increments count+8. **0x140031cf0** atomically decrements it and invokes the
object's primary destructor slot only if the count becomes zero.

For a new distinct state, the preserving map path retains before publication:
constructor count 1 → wrapper temporary 2 → map ownership 3 → wrapper release
2 → caller local release 1. Thus these successful post-registration releases
do not enter a destructor. This reasoning requires actual map retention,
valid counts without overflow, and no concurrent/untracked release. Arbitrary
zero-reference destructor effects are not assumed harmless.

## Category-23 array resize is not state registration

Constructor **0x140094280..0x1400942b4** sets S+0x10 to 0x17 and zeros the
three vector fields **S+0x18 (data), S+0x20 (length), S+0x28 (capacity)**.
The subsequent complete initializer **0x140094320..0x140095266** retains S in
RSI, performs 24 finite record insertions, then returns **the same S pointer
at 0x14009522e**. Its only direct calls are 24 calls to **0x140094130** and
24 calls to the memory-move thunk **0x141c6ec1c**. The body itself invokes neither a state-bag method nor a user
callback; its allocation/import callees have the explicit conditions below.

Each resize receives the vector address S+0x18. Complete
**0x140094130..0x1400941b5** compares the requested length with old length
and capacity. On growth beyond capacity it requests `n + floor(n/4)` entries
from **0x1403d2f30**, then stores the new length. Each newly exposed 16-byte
record has its +4 DWORD initialized to -1. This is record storage, not a typed
state object's category field.

The complete reserve helper **0x1403d2f30..0x1403d2fca** rounds a nonzero
requested capacity up to a multiple of four and multiplies it by 16 bytes.
With no old data it calls the global allocator's +0x50 entry; with existing
data it calls +0x58. A zero-capacity request can call +0x60 to free old data,
but this branch is not reached by the fresh 1-through-24 growth sequence.
The allocator is selected through the pointer at **0x142c947f0**. Returned
buffer pointers and capacity are written only into the supplied vector.
Allocator virtuals remain part of the explicit ordinary-allocation/no-reentry
profile, not proved arbitrary-callback absence.

Under successful bounded growth from the constructor zeros, the derived
capacity sequence is:

| New length needing capacity | Rounded entries | Buffer bytes |
| --- | --- | --- |
| 1 | 4 | 64 |
| 5 | 8 | 128 |
| 9 | 12 | 192 |
| 13 | 16 | 256 |
| 17 | 24 | 384 |

The resulting vector has length 24 and capacity 24 after all insertions. These
are statically derived arithmetic values, not a native runtime capture. The
internal record inventory is retained privately and is not embedded here.

The move thunk is a jump through **0x141dbbef8**, identified by the owned PE's
per-DLL terminated import table as **MSVCR120.dll!memmove**. The conditional
profile uses an unmodified ordinary implementation operating on the valid
member-array ranges; it does not assume an arbitrary redirected IAT is pure.
Successful distinct allocation/reallocation and these bounded copies preserve
S+0x10 and the separate backing bag. In particular, a reserve store to
`vector+0x10` means **S+0x28 capacity**, not **S+0x10 category**.

## Conditions and preservation boundary

The constructor-return conclusion is deliberately conditional on:

- Valid distinct outer, backing, state, payload and buffer allocations of the
  required sizes; successful reallocations preserve their ordinary memory
  contents and return valid storage.
- Unmodified selected tables/imports, ordinary lock and allocation primitives,
  no reentry or concurrent mutation, and no alias that lets those operations
  write the backing parent/map or another state's category unexpectedly.
- Valid positive reference counts without overflow and the proved map-owned
  reference before each local release; no untracked mutation or release.
- The finite state-map insertion/growth/removal contract preserving actual
  state pointers and categories, rather than treating registration EDX as the
  stored key.

The fresh map starts empty with null parent. Registering only actual categories
2, 11, 22 and 23 under that preserving contract leaves no local category-4
entry. The null parent removes inherited state lookup at this checkpoint.
Array reallocations and reference transfers do not add another category.

This conclusion belongs to **C at this return**. Later parent adoption, state
registration/removal, reflected callbacks, native overrides or cache refresh
require their own preservation proof. `Mouse.hide()` reads MovieImpl+0x228
rather than querying this fresh map. Establishing a later null handler requires
both the actual bag lineage and the correctly timed owned-reference cache
selection described in [REA_MOUSE_HOST.md](REA_MOUSE_HOST.md). The loader/engine
lane owns that next join. The diagnostic `0x140186d10` and real cursor effect
are separate studies and were not inspected here.

## Evidence and provenance

All private paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. Original checker
`rea-fresh-mouse-state/verify.py` validates **1,495 selected instruction spans**,
40 retained interface-table bytes, complete selected routine boundaries,
actual category stores and the exact PE import binding. Its
`verification.json` SHA-256 is
`5bcdd09c9b8a3d4ea6db47b7e79afa9c143dc89a0439e3efb22ff14dbb2256b7`.
The private selected instructions remain in the same directory.

The independent retained constructor review enumerates all **23 static call
sites** in `0x1400515a0` and ten complete selected ranges, including the return,
actual category constructors, registration wrapper and reference leaves:
`rea-fresh-mouse-state/constructor-review.json`, SHA-256
`500797b5800b0e93818f03d6ffe77810780527df608913765a2da704ecff2990`.
It excludes the disjoint map-helper study and is not a separate native run.

| Canonical retained source | Selected use |
| --- | --- |
| `frontend-image-context/complete.bundle.json`, `ev_f44f02ac…` | Complete outer constructor, category-11 constructor, concrete registration wrapper and state-bag interface table |
| `mouse-host/installation/complete.bundle.json`, `ev_6e58c561…`, `ev_9f9db0ca…`, `ev_28206742…` | Category-2 wrapper/constructor, complete category-22/23 constructors and category-23 record initializer |
| `mouse-host/installation/storage-helpers.bundle.json`, `ev_36180bcd…` | Complete array resize and memory-move import thunk |
| `scale9-texture-query/native.bundle.json`, `ev_a1812ee3…` | Complete atomic reference-count increment/decrement leaves |
| `rea-fresh-mouse-state/resize.bundle.json`, `ev_68923e43…` | Complete previously unexpanded member-array reserve helper |

The single new REA record is
**ev_68923e43c35f4f45427ffa38a34edb7620cd9bc75a8abdac26f665019b4fadfa**.
Its canonical `resize.bundle.json` SHA-256 is
`462d13140b60dbe02577e28f85162c2c3b518b160242e3570945ffc653a4d1e6`.
It completed with exit zero, no truncation and unchanged owned-artifact hash.
The investigation was exported and reset; the reset bundle is byte-identical.
Full reused IDs, ranges and bundle hashes are in the local verification.
Earlier short constructor captures are not substituted for the later complete
60-byte and 52-byte category-22/23 bodies.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| Owner's pinned MarvelHeroesOmega.exe | Proprietary owner installation; authorized private interoperability study | Instructions, fields, type stores, interface/import metadata and hashes |
| REA_MOUSE_HOST and its installation/effect evidence; retained frontend-image and reference-count studies | Original project research; target bytes remain private | Existing constructor/call/ownership anchors and precise prior limits |
| Installed reverse-engineer-anything skill and REA 6.9.0/Rizin tooling | Owner tooling; REA MIT, Rizin LGPL-3.0-only | Bounded static capture and evidence lifecycle; no implementation copied |
| Original private PE parser, verifier and constructor review | Original project research; Python standard library PSF | Independent byte joins, routine boundaries, import binding and derived array capacities |

This document contains no original client bytes, member inventory or native
implementation. Its construction conclusion remains conditional on the stated
primitives and map proof; later preservation and actual Mouse behavior are not
claimed from the fresh checkpoint.
