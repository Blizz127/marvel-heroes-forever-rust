# Fresh global bootstrap producers and map-state carry

**VERIFIED conditional static result:** the successful fresh-context path in
root `0x140210180` produces the Object, Function and global state described
below and carries the empty factory registry to the first deferred registration
at **`0x14020fc25`**. The numeric cache's initial emptiness and four original
insertions follow from the same selected path. They are no longer independent
opening-map assumptions for this profile.

The result composes actual producer stores with
[FRESH_BOOTSTRAP_BINDINGS.md](FRESH_BOOTSTRAP_BINDINGS.md). It retains valid
source objects, successful disjoint allocations, bounded reference/collector
state, ordinary imports and absence of reentry as explicit conditions. It does
not establish unconditional live-process defaults. No native client, original
script, VM, renderer, server or service was executed, and no product code changed.

## Context seed and the first cold lookup

Let `R` be the concrete AS2 root, `M` its supplied movie implementation, and
`C` the new context allocation. On successful distinct allocation, root setup
explicitly stores null numeric map `C+0x10` at `0x1402101cc` and null factory
registry `C+0x20` at `0x1402101d4`, together with the other fresh context slots.
It calls bootstrap `0x14020ea40` at `0x1402101e7`. These are actual stores;
the allocation is not assumed zero-filled.

Bootstrap stores `C+0x28=M`, copies the movie heap into `C+0x30`, and builds
a local StringContext containing `C` and literal version byte 8. The initial
global slot is null, so the selected path allocates a fresh global `G`,
initializes it, then publishes it at `C+0x60`. Until the first cold Function
prototype lookup at `0x14020ebc6`, the only calls are successful ordinary
allocations and the bounded global base constructor. All other writes address
the context's movie/heap/global slots, new receivers or the stack. Neither map
is written.

Complete base constructors `0x1401ccd80` and `0x1401cce10` initialize the
new receiver's count, tables, parent, own map, resolver/watch fields and other
listed fields. They write a collector pointer into receiver `+8`; they do not
register the receiver with that collector. Each immediately rereads the parent
it just cleared, with no intervening call, excluding its sole old-parent release.
Their selected writes stay in the fresh receiver and stack. Unlisted bytes are
not asserted initialized.

Consequently the first key-7 cache lookup is an actual miss, and the factory
lookup sees actual `C+0x20=NULL`. Complete `0x14018efa0` returns missing
without calling a factory. Its temporary builtin-name retain/release is bounded
by valid nonoverflowing ownership. This supplies the base of the induction:
the succeeding raw setters, metadata links and method installers preserve both
empty maps until the first explicit numeric insertion. Each later cold lookup
uses the state established by preceding operations rather than assuming a miss
because the operation is called a constructor.

## Actual Object and Function producers

The original bootstrap allocates two `0x70`-byte callable objects and two
`0xa8`-byte prototype objects. Inline stores and exact helper arguments establish:

| Identity | Concrete interface | Produced state before numeric publication |
| --- | --- | --- |
| Object callable `OC`, held in R13 | `0x142514430` | Native callback stored as data; own `__proto__`, `registerClass`, `prototype=OP`; parent ultimately `FP`. |
| Function callable `FC`, held in R12 | `0x142514430` | Native callback stored as data; own `__proto__`, `prototype=FP`; parent ultimately `FP`. |
| Object.prototype `OP` | `0x142513cd0` | Null parent; constructor metadata retains `OC`; constructor marker and eight methods. |
| Function.prototype `FP` | `0x142515be0` at completion | Parent `OP`; constructor metadata retains `FC`; parent/constructor markers and four methods. |

`OP` never calls the parent setter or generic parent visitor in this producer.
It therefore has **no initial own `__proto__` entry**, in addition to its
explicitly null parent. Its own-map, resolver and parent stores establish the
nine-key/null-parent inventory used by the later namespace proof. `FP` installs
its own marker and separately retains/stores `OP` as its parent.

`OP` and `FP` constructor metadata begins at `+0x68`, with retained constructor
reference at `+0x70`. This differs from MovieClip prototype metadata at `+0x78`
and constructor reference at `+0x80`. Their public constructor entries are
tag-10 markers, not raw function Values. Their callable's `prototype` entry is
a tag-6 data Value. The exact category getter returns 6. During FP linkage its
temporary interface is `0x142513cd0`; the later `0x142515be0` interface has the
same read-only category getter. The conversion does not rely on FP being callable.

The one Object-callable method, eight OP methods and four FP methods are all
created **before cache key 7 is published**. Their cold lookup returns null,
so all 13 method functions initially have null internal parents. Their own
`__proto__` entries are nevertheless stored markers. The two later calls at
`0x14020f143` and `0x14020f159` relink only `OC` and `FC` to `FP`; they do
not revisit those 13 methods. Native callback addresses are stored, not invoked.

**INFERRED finite map history**, using the previously verified exact hash and
typed insertion/collision rules:

| Object | Own key count | Capacity after these writes |
| --- | --- | --- |
| `OC` | 3 | 8 |
| `FC` | 2 | 8 |
| `OP` | 9 | 16 |
| `FP` | 6 | 8 |

Metadata/parent properties have descriptor 3; OP/FP methods use descriptor 1;
the `registerClass` method uses descriptor 7. Installing that native method
does not itself run class registration. Temporary reference cleanup remains
stateful and uses the established positive-reference/root-buffer contract.

## Key, Array and the remaining numeric-cache sequence

The first two explicit cache insertions publish `OP` under key 1 and `FP`
under key 7. Subsequent Function-parent lookups are concrete warm reads.
Before Key's later key-26 publication, its listener-Array constructor requests
key 2. At that point the numeric cache contains only keys 1 and 7, and the
factory registry is still null. That Array lookup is therefore another actual
miss with no factory invocation; the new Array's internal parent is null.

Complete Array constructor `0x1401db6e0` and its shared base write the fresh
Array. Array interface `0x142512b70` binds its category getter to the read-only
leaf returning **7**, and its raw setter to `0x1401e3b20`. This is the ordinary
tag-6 object conversion path, not function conversion. The `_listeners` write
uses Key's separately verified raw-setter slot, sets its listener-Array flag
at Key `+0x61`, and retains the new Array in the own map.

Two additional shared-state effects are explicit:

| Operation | Exact selected effect |
| --- | --- |
| Array storage helper `0x140035d30` | Atomically increments the static storage header at `0x142adb2f8` and installs its tagged empty-storage pointer in Array `+0x88`; no calls or context-map writes. |
| Key registration `0x1400532f0` | Calls the five-byte leaf `0x140179de0` for six movie input-state records. Each stores the supplied `Key+0x70` pointer at that record's `+0x10`. It calls/releases no previous listener and invokes no listener callback. |

These writes affect static Array storage and movie input-state fields, not
the context maps. Their storage must be valid and distinct from the context,
global and other live objects; no concurrency or reentry is admitted.

Key's 18 scalar constants also require an explicit extension to the earlier
typed member contract: native tag 4 copies the **32-bit** payload at Value `+8`
without a call or retain. Its cleanup bypasses managed-value release; assignment
still releases any old managed destination through the existing contract.
Descriptors are copied separately. Unused Value bytes are not asserted initialized.

Key and MovieClip construction then use the sealed finite constructor,
method-table, parent-visit, metadata and own-member contracts. No such selected
callee inserts another numeric entry or populates the factory registry.
The complete original numeric sequence is therefore keys **1, 7, 26, 6**,
count four/capacity eight, with actual retained MovieClip prototype `P` under
key 6. The old no-extra-insertion and early-empty-registry conditions have been
replaced by this bounded producer/call composition.

## Global production comes from an empty own map

The fresh global constructor explicitly clears `G`'s own map, internal parent,
resolver/watch and external fields. Bootstrap installs global tables, stores
its owner back-reference `G+0x68=C`, and publishes it in `C+0x60`.
The preceding Object/Function/Key/MovieClip work writes their own distinct
receivers, metadata, strings and reference state; it preserves this empty
global map until the actual native table installer.

At `0x14020f6b0`, complete `0x1401e9bd0` installs the exact original
21-record native-function table into `G`, with descriptor zero. Each successful
iteration allocates a native function, writes the table callback without
invoking it, assigns the now-warm `FP` parent and stores a tag-8 function Value
with null auxiliary frame and zero reference flags. Each such function has
only one own entry: its `__proto__` marker.

Four later source writes replace four of those keys with newly allocated
function identities. Exact literals/callback pointers establish replacement;
these writes add no keys. Four further stores publish `OC`, `FC`, the original
MovieClip callable and the original Key callable under distinct eager names.
The original source order and typed map rules therefore yield:

| Checkpoint | Own global count | Inferred capacity |
| --- | --- | --- |
| Native table complete | 21 | 32 |
| Four same-key replacements complete | 21 | 32 |
| Four eager stores complete, before first deferred registration | 25 | 32 |

This is a finite history derived from constructor-established emptiness and
actual source writes. It does not treat the later 46-name inventory as an
opening condition. Map growth and relocation use the exact supported Value
types and copy-before-release contracts already established in
[FRESH_BOOTSTRAP_BINDINGS.md](FRESH_BOOTSTRAP_BINDINGS.md).

## Four real zero-reference cleanups

These four replacements expose a necessary limitation of the earlier
positive-reference-only profile. For each old table function, allocation starts
at count one; temporary and stored copies add references, then registrar cleanup
leaves only the global holder. Replacement releases that holder to **zero**.
Earlier positive cleanup may already have buffered the object in the collector;
the root buffer is not another strong reference.

The old functions have a concrete common primary table, a single marker Value,
null resolver/watch/external fields and one object edge to `FP`. The new
replacement has already retained both `FP` and the same marker key. The
complete selected destruction path is now established:

1. With an initially idle valid collector and old queue bit clear,
   `0x1401860e0` initializes its child-list sentinel and dispatches primary
   slot zero to `0x140192490`, then `0x14018e430`.
2. The one tag-10 member and null resolver select no-op child paths in
   `0x140188b80` and `0x140188a60`; the null watch field skips its loop.
   Releasing `FP` remains positive because the replacement and established
   cache/constructor roots retain it. The child queue stays empty, excluding
   recursive zero-child processing and `0x14017e070`.
3. If the old function is buffered, complete `0x14017dfe0` removes its valid
   slot. The tail case selects only the decreasing-length branch of
   `0x14017dee0`, with no allocation. Header/index bookkeeping changes.
4. Primary slot eight selects `0x1401d3730`. Complete `0x140197e20` drops the
   old map's marker key, whose replacement holder prevents final string-node
   destruction, and performs no-op marker cleanup. It frees the old member
   table. Null watch/external state excludes its remaining callbacks.
5. The outer release frees the old function allocation. Its stored native
   callback is never invoked; the collector returns inactive with an empty
   child queue.

This closes the **actual four old-function shapes**, rather than assuming all
zero-count releases harmless. Allocation-failure collector fallback, extra
old-object fields, captured frames, populated resolver/watch graphs, queued old
objects, invalid root-buffer indices or arbitrary allocator behavior remain
outside this profile. Ordinary disjoint non-reentrant free/allocation and valid
nonoverflowing counts remain conditions. The cleanup changes collector and
reference bookkeeping while preserving context maps and surviving object data.

## Conditional handoff

Immediately before `0x14020fc25`, the same source-derived context has:

| State | Produced result |
| --- | --- |
| Numeric prototype cache `C+0x10` | Four original keys 1/7/26/6, including the retained MovieClip prototype. |
| Deferred factory registry `C+0x20` | Still null; no factory has been invoked by the preceding cold lookups. |
| Name-map slot `C+0x70` | Still null at this checkpoint; the later rebuild remains separate. |
| Original global `C+0x60` | 25 own entries at modeled capacity 32, including the original eager MovieClip function. |
| Object/Function parents and metadata | The produced identities, null/linked parents, constructor markers and data links described above. |

These results supply the formerly explicit prefix inputs to the sealed
34-registration/namespace proof. That continuation retains its actual names,
typed Values, source order and allocation/reference conditions. The later
name-map rebuild, root render setup, source-owner selection and resource/name
publication remain separate documented transitions.

The surviving premises are the concrete root/movie/StringManager/heap inputs,
successful valid distinct allocations, actual idle/valid collector conditions
where selected, finite tables and counts, normal successful frame/cookie/import
behavior, and no concurrent, script or external mutation. No current live
retail state was observed. Nothing here runs InitActions, a registered
constructor, saved handler, dynamic method or scheduler, or promotes an adopted
instance's supplied fields into fresh defaults.

## Evidence and provenance

Private root: `evidence/fresh-global-bootstrap/` below the project data root.
Four target-free REA calls completed with exit zero, untruncated output and
unchanged source hash. The original focused verifier matches **13 selected
ranges, 567 instructions / 2,071 bytes**, and two pointer reads / 16 bytes to
the pinned PE. Of the instruction bytes, six are an explicitly redundant Array
getter cross-check: retained coverage was found after that combined request
began. They are not claimed as newly recovered body coverage. Chained unwind
entries and small leaf bodies are handled separately.

`new-reads.bundle.json` and `reset.json` retain canonical evidence and successful
export/reset with no retained target. Shared-foundation, prototype-producer,
context-carry and global-producer ledgers preserve their exact dependencies,
source hashes, selected stores/calls, finite inventories and independent checks.
`verification.json`, `receipt.json` and `MANIFEST.json` bind the combined result.
No native bytes, tables, full name inventories or extracted assets enter git.
This docs-only increment has no Rust build or runtime-test claim.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's build `1.52.0.1700` `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Exact root/bootstrap producers, Key/Array leaves, zero-release bodies and PE metadata |
| Project documents cited above; retained original constructor, class-registry, array, enumeration, global-baseline and reference-cleanup evidence, individually named/hashed in private ledgers | Original project investigation; underlying artifacts remain proprietary | Reuse complete prior bodies and compose concrete producers with bounded map/reference contracts |
| Installed REA skill/guides and REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling | Bounded static reads, complete Evidence preservation and reset |
| Python standard library | PSF tooling; new scripts are original | PE/hash verification, finite map histories and private receipts |

No external VM or SDK implementation, shared configuration, account, live data,
launcher or other project's source was changed.
