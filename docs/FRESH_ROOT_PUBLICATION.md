# Conditional fresh AS2 root publication

**VERIFIED static conditional result:** the same fresh case-3 target T from
[FRESH_ROOT_RESOURCE.md](FRESH_ROOT_RESOURCE.md) can pass from
`0x1402102dd` through the selected root constructor's normal return at
`0x14021034c` with the following state. This result requires the actual
call-time conditions below; it is not an observed running frontend instance.

| Selected state | At `0x1402102dd` | At selected normal return |
| --- | --- | --- |
| DWORD `T+0x6a` | `0x52004f80` | `0x52004f80` |
| WORD `T+0x88` | `0x000a` | `0x0008` |
| WORD `T+0x140` | `0x0011` | `0x0011` |
| QWORD `T+0xb8` | `0x0000000000000010` | `0xffff000000200190` |
| Native name | Not supplied by this entry checkpoint | Exact `_level0`, after successful native assignment |

Naming clears bit 1 of the load WORD; level publication changes status/list
state. Thus this is a transition proof, not preservation of all four input
values. If only the supplied MovieImpl `+0x4314` bit 3 is set instead, the
final status QWORD is `0xffff000000a00190`. The clear-bit result above remains
conditional on that bit actually being clear when read.

No native executable, original script, VM, renderer, input or service was run.
VERIFIED denotes exact selected instruction, table, import and hash checks;
OBSERVED denotes static control flow. The proposed implementation use is
INFERRED. Queued work remains pending, and this checkpoint precedes its
delivery and frame-zero child creation. It cannot be applied to an already
populated saved Scene or adopted diagnostic target by stamping these fields.

## Caller, bindings and ordered effects

The retained complete caller is `0x140210180..0x14021034d`, 461 bytes. R is
the concrete AS2 root, M is `[R+0x10]`, and T is the factory-return target.
The caller derives owner O as `T + 4 * BYTE[T+0x6d]`; for the supplied DWORD
the byte is `0x52`, hence O is `T+0x148`. The low flag WORD is **`0x4f80`**.

| Call site | Concrete selected call | Result relevant here |
| --- | --- | --- |
| `0x1402102e8` | `0x1401871a0(O,0)` | Insert render child, assign owner level 0, format/intern/set `_level0` |
| `0x1402102f6` | `0x14005fdb0(M,0,T)` | Insert level, update target status, definitions/state inheritance and viewport |
| `0x140210301` | R virtual `+0x110` → `0x1401e3860` | Register deferred XML-related builtins and consult optional state |
| `0x140210311` | R virtual `+0x118` → `0x140196a30` | Consult pending assignments for the target's exact path |
| `0x140210326` | T virtual `+0x128` → `0x14009c820(T,55.0)` | Update projection side storage and render matrices |
| `0x140210330` | `0x140031cc0(T)` | Drop the constructor's temporary reference |

The caller then sets AL to 1 and returns. It **does not test** publication's
Boolean result. A duplicate-level false return, failed name allocation, or
callback-bearing branch cannot be promoted to successful publication merely
because the outer caller returned true.

The selected tables are R `0x142511240`, T `0x1425069d0`, MovieImpl
`0x1424feb70` and MovieDef `0x1425048b0`. These are static dispatch witnesses,
not proof of the identities of live native objects.

## Required entry state and ordinary-operation profile

Carry the same eligible target/source/owner/context from the prior factory
and resource studies, including their effective prototype, heap and lifetime
conditions. Additional concrete conditions used by this bounded profile are:

1. T has actual null parent `T+0x38`, QWORD `T+0x118=0`, signed DWORD
   `T+0x2c=-1`, QWORD `T+0x98=0` and native child count QWORD `T+0xe0=0`.
   The count belongs to the native list based at `T+0xd8`; it is distinct
   from the render container's child storage and the Rust saved Scene.
2. M's level array is actually empty: pointer/count/capacity at
   `M+0x58/+0x60/+0x68` are null/0/0. The selected prior list head is null
   or a valid node distinct from T. M's actual `+0x4310` bit 19 and
   `+0x4314` bit 3 are clear. Their constructor checkpoints alone do not
   establish survival through preceding calls.
3. The exact render tree is valid and disjoint from T's protected fields.
   Its required current change records exist. A needed node allocation has
   a valid nonempty render-node free list; the unclosed pool-expansion
   branch is excluded. The parent container's actual first-child slot is
   empty for the selected first insertion. Matrix states are valid typed
   states as described below; they need not be empty.
4. Name interning/copy, required arrays, side storage and ordinary heap
   allocations succeed. Allocations are distinct from live objects;
   matching frees preserve them. References, sizes, indices, hash chains
   and auxiliary string chains are valid and finite, without overflow,
   underflow, concurrent mutation or reentry. Imported CRT and locking
   calls obey their ordinary selected contracts. Generic heap/CRT/lock
   implementations and exceptional callbacks were not audited or run.
5. Publication has actual `M+0x30=null` and old `M+0x48=null`, excluding
   the additional source callback and old-definition destructor paths.
   An old inherited StateBag reference is null or has a proved nonzero
   count after its release. A count of 1 is insufficient. New references
   are retained before old references are released.
6. The final builtin, optional-state and pending-assignment gates below
   are supplied at their read points. The queued-resize record, if used,
   is freshly initialized or actually cleared and valid; membership in
   the queue free list alone does not establish this.

The private seed audit ties T's integer, depth, link, null parent and child
count to exact constructor stores. The closed selected writes carry them to
their uses, including the mandatory base projection before the child-count
read at `0x1400b3872`. In contrast, the two MovieImpl policy bits remain
explicit actual-entry requirements. This distinction is intentional.

## Render insertion and exact name

Before formatting, `0x1401871a0` finds the insertion index from M's level
array and inserts T's render node below `R+0x28`. The empty array selects
index 0. An existing `T+0x48` node is returned directly; a missing node uses
concrete target creator `0x1400a1cd0`. The selected successful path initializes
its distinct payload, allocates a record from the admitted free list and
publishes `T+0x48`. The supplied local-visible bit is set; the initialized
backing already has the same visible bit, so its change branch is skipped.

`0x14004a420` takes the parent's existing-current-record branch. The selected
empty first-child slot lets `0x1400a96e0` store the child without growth.
The wrapper retains the node, sets its parent and performs the bounded list
notification when required. These writes affect render storage and `T+0x48`,
not the four selected fields or name handle under the disjointness conditions.
This closure does not infer the free-list/current-record/empty-slot guards
from child payload initialization.

[FRESH_ROOT_NAME.md](FRESH_ROOT_NAME.md) proves the fixed formatter's exact
seven bytes `_level0` plus NUL. This increment additionally closes successful
copied-string storage: a 7-byte name uses the short-string pool, and an empty
pool can allocate its page and populate the free list before the copy. No
invented preexisting short-string slot or name handle is required.

Cache lookup itself does not retain the returned string node; the producer's
increment is the temporary retain. Failed node/copy allocation can return a
default string, so exact naming requires successful interning/copy rather
than mere normal return. The actual nonempty setter clears WORD `T+0x88`
bit 1 before its handle branch. Existing-handle replacement and the
null-parent new-handle path are bounded, including name-only path assembly,
string release and finite hash removal. Successful assignment gives
`0x000a & 0xfffd = 0x0008`; the other three fields survive this stage.
The wrapper also clears `T+0xf8`. Public `_name` reads handle `+0x10`;
internal path `+0x18` is separate. A new handle derives that path only when
the actual root byte selects the path-building branch; otherwise it keeps
the manager's default path reference. Both branches are retained, so later
pending-assignment lookup must use the actual handle path, not an invented
`_level0` path identity.

## Level insertion and status

Complete `0x14005fdb0..0x14006007d` is 717 bytes across seven unwind
fragments. An existing equal level returns false; the selected empty-array
profile excludes it. First insertion grows count 0 to 1 through
`0x14005a360` and `0x140057270`. Capacity rounds to 4 slots and the native
heap request is 64 bytes. Old-null storage excludes realloc/free, and index
0 excludes memmove. Only the active slot is initialized; unused capacity
is not asserted zero. The slot retains T and records level 0.

T virtual `+0x3e0` binds `0x1400b3140`. It ORs the low status DWORD with
`0x180`, giving `0x190`, then inserts the target into the selected list via
`0x1400a12d0`. Flags 4 and 12 are clear and depth is -1. The previous head
must differ from T so the back-link write does not overwrite T's carried
zero link.

The `+0x330` predicate receives false. Its `+0x390` getter `0x1400b6f60`
sees low-WORD flag 11 set and returns the actual zero DWORD at `T+0x118`.
Consequently `0x1400b60d0` returns 1 through its zero-result branch:
`0x140053270` and the saved/dynamic `onEnterFrame` query are **not reached**.
No assumed event absence or opaque predicate result is needed here.

`0x1400a1380` observes M policy bit 19 clear and the accepted list predicate,
inserts into the active list, sets status bit 21, clears bit 22 and copies
M `+0x4314` bit 3 to status bit 23. The final store is WORD `T+0xbe=0xffff`.
This yields QWORD `0xffff000000200190` under the selected clear policy bit.
The remaining three fields and exact name are preserved at this checkpoint.

The publication routine then stores `M+0x50=T`, retains the exact MovieDef
from `T+0xb0`, replaces `M+0x48`, and installs the definition's inherited
StateBag reference. Actual null/positive-after-release guards exclude the
old-object notification/destructor branches. The optional `M+0x30` branch
through `0x1400a1fc0` remains outside this profile.

## Viewport, projections and deferred work

Concrete scalar getters obtain FPS and stage dimensions from the selected
MovieDef data. Dimensions are bounds deltas multiplied by the exact stored
f32 value approximately 0.05, then passed to imported `MSVCR120.dll!ceilf`.
Publication stores `1.0/FPS`; no native zero/NaN guard is invented here.
When M policy bit 0 allows viewport setup, the real nonzero-stage path is
included rather than requiring an unchanged or zero viewport.

`0x14005d380` compares/copies the 52-byte viewport, updates M's matrix/rect
state and can invoke each level target's virtual `+0x50`. There is exactly
one level in this profile. Its concrete bridge `0x1400b3860` calls
`0x14009c5a0` before reading the native child count. The selected typed
projection writes preserve that count at zero, so child virtual calls are
excluded at the actual read. This does not claim that already-created
frame-zero children can be ignored.

Viewport changes can also call R virtual `+0x98`, `0x1402003e0`, which
allocates and inserts kind-4 work with a retained level-zero target and
callback `0x1401f6c50`. The callback pointer is stored, **not invoked**.
Queue insertion is cursor-relative. A reused slot must have actually cleared
payload/arguments; the allocator clears its next link but does not prove
that its old target references are null. The profile excludes destructor
effects from stale payloads and mutation between target scan and reselection.

The later T virtual `+0x128` receives the exact double 55.0. It can allocate
projection side storage `T+0x58` and again invoke real projection. Concrete
matrix setters operate on T's node and, for the root, M's render nodes.
Both initial insertion and repeated key-5/key-6 matrix replacement are
covered with the proved descriptor methods, positive valid counts and
typed matrix payloads. Last-reference matrix replacement performs the
proved base-table update and ordinary matching heap free. Array expansion
retains copied payloads before releasing the old array. An existing shared
array's matching-key update is observed; no copy-on-write behavior is added.

These operations change projection storage, render flags/maps, matrices,
references and change records. Under the distinct typed graph they do not
write T's four selected fields, name, or native child count. Arbitrary
descriptors, missing current records, pool growth and failing/reentrant
allocation remain outside the selected closure.

## Final callback gates and return

R `+0x110` installs deferred factories for exact XML/XMLNode keys through a
version-8 context. It does not execute those factories. The smallest closed
no-callback profile supplies both keys already present in the factory map;
the native wrappers then perform their exact lookup and return. **Their
presence is not a fresh-object default or a consequence of MovieClip
registration.** This fast path does not require particular cached values or
global XML/XMLNode property contents. The private audit also bounds selected
missing-key insertion and simple member-bucket cases, but generic collision types, collector
expansion and raw-global insertion are not silently treated as inert.

The same call queries optional StateBag ID `0x1d`. Absence must hold through
the finite concrete local/inherited getter chain, with ordinary locking and
no mutation. Publication has already installed inheritance from the MovieDef;
an initially empty local StateBag does not prove absence. A found-state
callback or foreign getter is outside this profile.

R `+0x118` uses the successful existing name handle to query M's pending
assignment map. Actual null map or exact key miss excludes delivery. A match
can perform ordinary member setters, including renaming and script effects;
the matched and present-key/null-list branches are not covered by an assumed
empty initialization. Under the admitted miss, `_level0` survives to return.

Publication retains T temporarily and for its level slot, then releases the
temporary. Optional queued viewport work adds its own retained reference.
The final root release must leave a positive count; its decrement then
avoids the deleting virtual. Valid nonoverflowing ownership and the admitted
no-removal/no-reentry sequence supply this, without inventing a universal
initial count: the actual count at that last release is at least 2. Known
native descriptor retain/release methods and matrix cleanup still run inside
this profile; their bounded effects are included above. The return invariant
ends before any queued callback, later
member assignment, original ActionScript constructor, frame script or input
event runs.

**INFERRED implementation contract:** a future bounded fresh-root adapter can
consume explicit entry/profile facts, perform these ordered transitions,
and emit the resulting fields/name plus pending work and retained source
identities. Unsupported callback or allocation branches should remain
explicit outcomes. This is not permission to seed native flags into an
adopted saved Scene, skip required work or claim a complete native loader.

## Evidence and provenance

Private root:
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-root-publication/`.
The aggregate `receipt.json` binds this document, the following independently
sealed subreceipts and the complete private artifact ledger. Counts are per
audit and overlap; they must not be summed as unique executable coverage.

| Audit | Exact selected coverage / receipt SHA-256 |
| --- | --- |
| `render-prelude/` | 14 complete ranges, 718 instructions / 2,689 bytes; receipt `f71b1960632318f115ce9333c1e7b8e3ccb7b95e6fff409557879d0ae224deb0` |
| `name-audit/` | Seven newly closed complete functions, 303 instructions / 1,088 bytes, plus retained formatter/interner/name evidence; receipt `bf30f47b2c2442d9bdcbec1f2a181de05a5f912206418a0f69b68e2f1639dfc8` |
| `publication-audit/` | 2,276 unique instructions / 9,914 bytes; receipt `3c78d9b6b7b7bd0e63738ed717c3ea66c8aab9ebd643229737a8e4209e7c0d3a` |
| `tail-virtuals/` | 2,695 instructions / 11,236 bytes; verification `028d17f22c6657b61ea4189d1d3e524c68cec548543e5f09130e991c1d277ea8` |
| `seed-carry/` | Eight reused constructor ranges / 2,133 bytes and conditional carry to actual reads; receipt `a4734efc50ab21794329b3740287847e72148000b27ed20fcb76efa7690f16e8` |

Canonical new and retained bundles, exact code extents/chained unwind
coverage, pointer/import witnesses and PE-byte checks remain private.
Viewport helper inline jump-table data is distinguished from instructions.
One 20-second provider call retains unknown process completion; the already
started retry changed the timeout to 60 seconds and completed with identical
stdout. Both records remain immutable; successful retry/reset does not prove
the first process completed. No further replay was used. All ledgers were
exported and reset with no retained target.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe`, build 1.52.0.1700, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private compatibility study | Native dispatch, function extents, fields, literals and imports |
| Existing `FRESH_ROOT_NAME`, `FRESH_ROOT_RESOURCE`, fresh target/completion, attached initialization/visibility, class-registry and display-lifecycle docs/private receipts | Original project investigation; underlying artifact remains proprietary | Caller, constructor seeds, prior formatter/resource closures and selected helper reuse |
| `fresh-prototype-bindings/root-prefix-audit/closure.json` and `string-audit/` | Original project evidence | Concrete child insertion and StringManager hash operations; no inference about the distinct factory map |
| Installed `~/.agents/skills/reverse-engineer-anything/SKILL.md` and native guidance; REA 6.9.0 / Rizin | Existing MIT / LGPL-3.0 tooling provenance | Bounded target-free static reads and canonical Evidence |
| Python standard library | PSF tooling; verification/receipt code is original | Independent PE mapping, finite byte checks and hash readback |

Exact reused-file identities and hashes are retained in the private source
ledgers. No SDK or outside game code was copied. No extracted bytes, binaries,
assets or private Evidence enter git. This is the scoped source ledger for
integration into [PROVENANCE.md](PROVENANCE.md); no central file or runtime
implementation is changed by this task.
