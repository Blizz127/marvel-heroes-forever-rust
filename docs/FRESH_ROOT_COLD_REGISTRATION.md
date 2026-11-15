# Cold XML/XMLNode registration in a fresh AS2 root

**VERIFIED conditional static extension:** the late root registration can
insert the previously absent XML and XMLNode factories instead of requiring
the warm key hits in [FRESH_ROOT_PUBLICATION.md](FRESH_ROOT_PUBLICATION.md).
Under the selected original bootstrap input and carry described below, the
deferred-factory map changes from **34 entries / capacity 64 to 36 / 64**,
with every cached function reference still null. The global receives two
deferred name tokens, each with descriptor flags 0. Neither factory runs.

This closes the two late insertion paths, including collisions and typed
global member-table growth/cleanup. It does **not** establish the complete
earlier Object/Function/global construction history from allocation alone.
The exact constructor seed is known, but its carry through every earlier
installer remains an upstream obligation. A list of 46 explicit global write
names is not an unconditional proof of the physical global table's contents.

No native executable, original script, VM, GPU, input, service or network
operation was executed. VERIFIED refers to selected canonical REA bytes,
dispatch identities and hash checks; OBSERVED refers to static control flow.
Arithmetic and implementation recommendations derived from those facts are
INFERRED. Nothing here assigns fresh flags or names to a populated Scene.

## Distinct maps and the actual seed

C is the newly allocated AS2 context, R its root, and G is `[C+0x60]`.
The following storage must remain distinct in this proof:

| Storage | Native role |
| --- | --- |
| `C+0x10` | Numeric prototype cache; the original MovieClip prototype uses key 6 |
| `C+0x18` | `Object.registerClass` associations |
| `C+0x20` | Deferred builtin factory map studied here |
| `C+0x70` | Standard member-name/ID map |
| G's own-member map, `G+0x38` | Ordinary global Values and descriptor flags |
| StringManager `S+0x10` | Interned string-node index |

The complete retained root setup `0x140210180..0x14021034d` requests a
`0x78`-byte context. On successful allocation, it clears EDI at
`0x1402101ad`, holds C in RBX, and stores QWORD `[C+0x20]=0` at
**`0x1402101d4`**. It then calls bootstrap `0x14020ea40` at
`0x1402101e7`. This is an exact null checkpoint, not a guess from allocation.

The distinction between this checkpoint and the first registrar matters.
Earlier bootstrap lookups at `0x14020ebc6` and `0x14020ed6f`, plus nested
installers, can reach `0x140194830 → 0x140194670 → 0x14018efa0`. An actual
null factory map makes the selected lookup miss without invoking a factory.
The constructor store alone does not prove that intervening installers left
the map unchanged. The retained early-bootstrap profile supplies that
empty-input/carry condition; this document does not promote it to an
unconditional live-instance fact.

## The 34-entry predecessor and carry

The early registrar audit in
[FRESH_BOOTSTRAP_BINDINGS.md](FRESH_BOOTSTRAP_BINDINGS.md) verifies 34 distinct
factory names and all 34 wrapper bodies. Nineteen store their deferred token
on G and fifteen on original namespace objects. Both late key offsets,
`0x1f0` and `0x1f8`, are absent from this exact 34-key set. This is separate
from any global-member inventory argument.

With the original empty factory-map input and no intervening cache
population, each early registrar inserts a pair `{factory, cached=null}`.
The complete positive-capacity growth/rehash/assignment helpers preserve that
null-cache invariant. Capacity progresses through 8, 16, 32 and 64, growing
before insertion ordinals 1, 8, 14 and 27. Successful distinct insertion 34
therefore leaves count 34, capacity 64 and null caches. Zero-capacity clearing
and cached-object release are excluded by these derived values, rather than
treated as no-ops.

The namespace interleaving is included in that earlier profile: exact own
namespace tag-6 hits and original Object.prototype misses avoid tag-11
resolution and do not populate the deferred caches. Its original parent and
global constructor applicability remain explicit upstream requirements.

From `0x14020fee7` to the late registration, the selected composition carries
the factory map and ordinary global Values as follows:

| Phase | Included effects and reason the deferred map survives |
| --- | --- |
| Bootstrap suffix | Rebuilds `C+0x70`, including StringManager growth; existing interned identities/bytes/low hash bits survive. The separate factory map is untouched. |
| Final global visitor | ORs descriptor flag 1 into existing own entries while preserving their original tag-6/tag-8/tag-11 Values. The original no-tag-10 inventory excludes the visitor's resolving branch. |
| Reference cleanup | Positive object/function cleanup updates headers and collector roots, including successful page growth. Zero-count destruction or failed growth into collection is outside the selected profile. |
| Root render prefix and fresh target/Environment | Writes root/render storage, new T/O/E buffers and definition refcounts under the prior distinct-allocation/current-record profile. |
| Owner prototype selector | Actual numeric key 6 and own `MovieClip` tag 8 / `prototype` tag 6 avoid the deferred resolver, accessors and tag-11 conversion. This is the original MovieClip construction profile, not warmed XML state. |
| Resource and naming | Performs the proved resource/reference/string/render transitions. Shared builtin StringNodes retain identity, bytes and hash; intern-table relocation does not relocate the factory map. |
| Level publication | Carried `T+0x118=0` bypasses dynamic `onEnterFrame` lookup. Typed projection and queue insertion do not execute the queued callback or write AS2 member data. |

These are composed write/call footprints under the existing ordinary heap,
reference, collector, CRT and disjoint-storage conditions. Preserving the
context pointer or the MovieClip member alone would not prove preservation
of the whole factory map. If an excluded callback, changed own binding,
collection failure, alias or reentrant mutation occurs, this carry does not
apply. It is not evidence that arbitrary native startup is callback-free.

## Exact late wrappers and factory-map effects

R virtual `+0x110` binds complete `0x1401e3860`. It obtains the same C from
R, constructs a stack context with **version byte 8**, and passes G to the
following calls in order:

| Call site | Wrapper | Exact builtin key | Stored factory |
| --- | --- | --- | --- |
| `0x1401e3884` | `0x1401d9bb0` | XML, index 23 / offset `0x1f0` | `0x1402171d0` |
| `0x1401e3896` | `0x1401d9d00` | XMLNode, index 24 / offset `0x1f8` | `0x14021ac70` |

No intervening caller operation invokes another routine. The optional
StateBag-ID-`0x1d` query occurs after these two calls. Each absent-key wrapper
explicitly creates its factory pair with a null cached reference, invokes
`0x1401c2a50`, then publishes its tag-11 token through G's raw setter. The
map helpers read the stack pair without changing its null cached local, so
the wrapper's own cached-reference cleanup is also excluded.

Insertion tests the load threshold before incrementing count. From the
derived predecessor, the first comparison is `5*34 <= 4*64` and the second
is `5*35 <= 4*64`. Thus these two insertions do not grow the factory map.
This is a consequence of the 34-entry producer, not an imposed no-growth
condition or an assertion about a reused context.

Hash collisions remain real. The 32-byte-slot insertion copies/retains a
displaced entry before replacing it through `0x14018ec00`. All incoming and
displaced cache fields are null, so the cached-object release/GC paths are
not reached. Builtin ownership and retain-before-release preserve positive
StringNode references and exclude final key destruction. The exact physical
bucket positions need not be assumed empty for this contract.

The earlier growth closure remains part of the proof: `0x14019a830` uses
positive capacity 8 or a valid doubling, allocates `16 + 32*capacity` bytes,
initializes empty links, rehashes through `0x1401a4520`, and frees only the
old table. Assignments `0x14018ec00` and `0x140188f20` copy factory pointers
as data; they never invoke them. Ordinary successful disjoint allocation,
finite valid chains and nonwrapping size/count arithmetic remain required.

## Global raw insertion, collisions and cleanup

Each wrapper constructs `Value{tag=11, payload=interned key}` and passes it
with descriptor flags 0 to original global interface
`0x142512aa0 + 0x50 → 0x1401e3b20`. The caller's explicit version 8 selects
the exact-key path; this does not depend on a guessed loaded SWF version.
XML/XMLNode differ from the special prototype/listener/resolver/event names,
and incoming tag 11 excludes the object and accessor branches.

For the selected original exhaustive global namespace, the two names are
absent. The raw setter reaches new own-member insertion through
`0x1401cffd0 → 0x1401cb1c0`. Actual bucket positions and capacity are not
claimed. The retained typed own-map closure includes successful growth and
collision relocation, so neither empty buckets nor a fabricated global
capacity of 64 is required.

The original global profile contains 25 native function Values (tag 8),
19 deferred string tokens (tag 11) and two ordinary namespace objects (tag
6). Relocation preserves these Values and their existing descriptor flags:

| Stored value | Selected retention/cleanup contract |
| --- | --- |
| Tag 11 | Retain/copy its interned key; builtin and destination ownership prevent final release. No factory resolution occurs during copying. |
| Tag 6 namespace | Concrete ordinary category excludes function-conversion category `0x17`; copy retains before old-slot release. |
| Tag 8 native function | Original null auxiliary frame and zero reference flags select the bounded native function-reference copy. |

Copies retain before old-slot cleanup. Positive references can still call
`0x1401860e0` and collector-root bookkeeping: a nonfinal decrement alone does
not make cleanup inert. The selected successful free-slot/existing-page/page-
growth contract is included. Failed growth leading to collector processing,
zero-count destruction, arbitrary Value types and unknown auxiliary frames
are outside it. Valid collector/object/member storage must be disjoint from
the deferred map, T and its name state.

After each successful insertion, local tag-11 cleanup releases only its
temporary key reference. The stored token and builtin owner retain the key.
This preserves the null-cache invariant between the two wrappers. Existing
global descriptor flags remain those produced earlier, while the two new
members have flags 0; no second flag-1 visitor is invented.

**Upstream absence boundary:** absence from the 46 explicit write names
alone is insufficient if the earlier global physical construction/history
has not been established. A preexisting same-name global member takes value
replacement, not new-member relocation. In that branch an old tag-6/tag-8
value can lose its last reference without a copied destination protecting
it. This proof does not replace that boundary with a no-destructor assumption.
Use the original exhaustive global producer profile, or separately prove the
actual overwritten value/lifetime; otherwise this selected contract does not
apply.

## Checkpoint and implementation use

Immediately after the second wrapper, before optional-state processing, C's
factory map contains the original 34 entries plus the two exact factory/null
pairs. G has matching deferred tag-11 tokens. Neither XML factory, an
ActionScript constructor, the optional-state callback nor queued resize work
has run as part of these registrations. Known native reference, allocator
and collector-buffer helpers are included in the selected path.

Under the prior publication profile's separate optional-state absence,
pending-assignment miss, typed render and positive final target-reference
conditions, these cold insertions preserve the four target fields and public
`_level0` name already documented there. They replace only the warm XML-map
key-hit requirement; they do not remove other publication qualifications or
establish the outstanding early bootstrap producer history.

**INFERRED:** an original fresh-root implementation can represent these as
two ordered deferred registrations retaining exact names, factory identities,
null cache state, tag-11 global Values and flags 0. Factory resolution remains
a later distinct operation. No VM or runtime implementation is changed here.

## Private evidence and provenance

Evidence is retained under
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-root-cold-registration/`.
Its aggregate receipt binds this document, selected source hashes and the
seed/order, factory-map, carry and global-member audits. The shared typed
member-retention dependency is sealed at
`fresh-bootstrap-bindings/member-retention/receipt.json`, SHA-256
`171bb377d3b33ea442326bae7d4be3bf3f1a589405330d2522ab0a4c52601be3`.
The seed/order receipt is SHA-256
`b034d8f1b1990c71a51d4f3596e171bb0cb6ed357fa69f9d0b8c3c911de8de52`.
The factory-map receipt is SHA-256
`80918ab54983ddb43f412db34a8ccd12ba517478ba593e49dba83875a147c5e1`.
Its six reused records were checked over 848 instructions / 3,187 code bytes
and 304 data bytes. The global join checks 734 instructions / 2,777 code
bytes across nine complete ranges. These cover overlapping functions and
must not be summed as unique executable coverage. No new REA capture was
needed; the actual retained source and body boundaries were checked.

All native bytes come from existing canonical REA records for the owner's pinned build
1.52.0.1700 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

The earlier viewport provider record with unknown completion remains
unknown. It was not replayed for this task. Reusing its independently
completed byte-identical companion does not retroactively change that status.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned proprietary executable and retained REA bundles | Private compatibility evidence; no redistribution licence | Context seed, wrappers, dispatch, table arithmetic and exact helper behavior |
| `fresh-bootstrap-bindings/interval-audit/`, including `lazy-map/`, and the typed member-retention audit | Original project investigation | Early 34 registrations, namespace interleaving and typed own-map insertion/growth |
| `fresh-prototype-bindings/`, `fresh-target-completion/`, `fresh-root-resource/`, `fresh-root-publication/` and corresponding project docs | Original project investigation; underlying binary remains proprietary | Selected intervening write/call footprints, reference/collector and publication contracts |
| Installed REA skill/recovery guidance; REA 6.9.0 / Rizin | Existing MIT / LGPL-3.0 tooling provenance | Canonical static evidence and honest retained provider-status handling |
| Python standard library | PSF tooling; private analysis/verification is original | Exact selection, arithmetic and source/hash checks |

No native code, SDK implementation, extracted byte table, proprietary asset or
private receipt is committed. This scoped source ledger is for integration
into [PROVENANCE.md](PROVENANCE.md).
