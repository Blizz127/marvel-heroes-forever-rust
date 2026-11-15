# Earlier bootstrap MovieClip bindings

**VERIFIED static insertion results:** the original numeric-cache insertion
retains the MovieClip prototype, and the original own-member insertion paths
store the MovieClip function/prototype links. The finite subsequent registration
paths can preserve those bindings to `0x14020fee7` under the explicit source,
map, reference and allocation conditions below. This moves the handoff in
[FRESH_PROTOTYPE_BINDINGS.md](FRESH_PROTOTYPE_BINDINGS.md) back to actual
insertion operations rather than assuming their resulting entries exist.

This is not a complete proof from root allocation. Earlier context/map states,
successful construction and the exact unchanged original inventories remain
premises where identified. No native client, original script, VM, renderer,
server or service was executed; product code and adoption defaults are unchanged.

Use `C` for the actual script context, `G=[C+0x60]` for its concrete global,
`F` for the original native MovieClip function, and `P` for the original
MovieClip prototype. Bootstrap's local StringContext explicitly has version 8,
selecting the modern exact-name paths. Storage, counts and identities refer to
actual native objects, not controlled all34 fixture values.

## Numeric cache insertion establishes a retained root

At `0x14020f689`, the original caller passes `RCX=RDX=C+0x10`, a pair
containing pointers to u32 key 6 and the returned `P`, and the original
integer-key hash to `0x1401a3b10`. It does not retain `P` separately at that
call site. The now complete insertion body is 528 bytes across five chained
unwind entries. Its empty-bucket path stores the key, increments the nonnull
prototype's native reference header with the observed mask, then stores `P`.

The complete bootstrap has four direct calls to this insertion primitive:

| Call | Key | Prototype | Home bucket at capacity 8 |
| --- | --- | --- | --- |
| `0x14020f1a4` | 1 | Object.prototype | 6 |
| `0x14020f1f3` | 7 | Function.prototype | 4 |
| `0x14020f603` | 26 | Key prototype | 7 |
| `0x14020f689` | 6 | MovieClip prototype `P` | 3 |

Starting with actual `C+0x10=NULL`, complete reserve `0x1401984c0` creates
eight buckets in a 208-byte allocation, count zero, mask seven and empty links.
The old-null branch excludes rehash, old-reference cleanup and free. The four
keys have different home buckets; existing counts zero through three do not
meet the growth condition `5*count > 4*capacity`. The finite sequence therefore
uses neither collision assignment nor later growth. After the fourth insertion,
count is four and key 6's value at table `+0x68` is the same retained `P`.
This root does not depend on successful retention through `F.prototype`.

This routine increments occupancy unconditionally: it is not a general
duplicate-key replacement API. A reused or already populated numeric cache
must not be treated as this fresh four-key sequence. Allocation success,
valid disjoint storage and nonoverflowing references are required.

The remaining earlier conditions are precise. Root initialization alone does
not prove `C+0x10` stays null until `0x14020f1a4`, or that no intervening
callee inserts another numeric key between the four calls. Before key 7 is
installed, direct and nested method-installation lookups can call
`0x140194670` through a missing cache lookup. With actual `C+0x20=NULL`,
complete lazy-registry lookup `0x14018efa0` returns missing and invokes no
factory; its copied builtin-name retain/release is balanced under valid
ownership. A nonempty registry needs its actual membership/factory proof.
These earlier map/no-factory conditions remain explicit prefix obligations.

## Own-member insertion and exact logical links

Complete `0x1401cffd0` is a 174-byte dispatcher receiving a table slot,
allocation context and pointers to the StringNode key and source Value. Its
lookup `0x140181f40` uses the node's hash and exact interned pointer identity.
An existing key receives normal Value assignment followed by the supplied
property-descriptor byte. A missing key delegates to complete 389-byte
`0x1401cb1c0` with the original hash.

The new-key path allocates eight buckets if needed, grows only at the observed
threshold, retains the key, constructs the stored Value and explicitly copies
the descriptor byte. It never substitutes a function merely because the stored
Value is callable. Native function tag 8 with null auxiliary frame preserves
the exact `F` pointer, retains it and writes zero reference flags/null frame.
Object tag 6 preserves and retains the original object pointer when its concrete
category is not the special function-conversion category `0x17`. These are
typed copy conditions, not permission to treat arbitrary Value variants alike.

The original `1eb640 → 1e5470 → 1b80f0` constructor chain supplies the actual
`P` and original `F` to the linkage operations. It stores a retained `F` in
constructor metadata `P+0x80` and installs an own `constructor` marker,
native tag `0x0a`, descriptor 3. It separately writes the tag-6 data member
`F.prototype=P`, descriptor 3. The constructor marker is not a raw tag-8
function Value; its effective constructor source is the metadata field. The
maps are **not empty** at these two writes: parent setup has already installed
each own `__proto__` marker.

That physical marker matters. Complete parent setter `0x14018edf0` supplies
native tag `0x0a` to the modern raw setter, which stores the static marker
Value with descriptor 3. It separately retains and publishes the actual
parent pointer. The stored marker is not itself a tag-6 copy of the resolved
parent. Its copy transfers the tag; its cleanup does nothing. Unused payload
bytes are not asserted to become meaningful or byte-identical.

The verified string hash gives `__proto__` and `prototype` the same home
bucket 3 at capacity eight; `constructor` has home bucket 5. Thus the initial
`F.prototype` insertion really takes collision relocation, while the initial
`P.constructor` insertion does not. Collision helpers copy the old entry to
the free bucket before reassigning the home bucket, preserve the chain and
descriptor, and retain the replacement key before releasing the old reference.
The displaced first `__proto__` Value is the marker above, not an inferred
Function.prototype object Value.

## Constructor work after the initial links

The MovieClip prototype wrapper does more than link two properties. Its
selected parent path visits the original Object.prototype through the generic
visitor. Options `0x0d` include the raw-marker-suppression bit, so a marker
does not invoke a raw getter. The complete callback classifies a key against
12 builtin identities and may set `P+0x72`; it does not change a member map.
The classifier is a 197-byte read-only leaf. The constructor then explicitly
resets `P+0x72` after the parent visit.

The wrapper installs 41 original native methods. The exact private descriptor
table has distinct ASCII names, excluding `constructor`, `prototype`,
`__proto__` and `on`-prefixed names. The installer allocates function objects,
stores their callback addresses and uses warm cache key 7 for their parent;
it does not invoke those callbacks. The selected prototype setter therefore
avoids its event-name dispatch branch. The final saved write is
`useHandCursor=true`, native Boolean tag 2, descriptor 3. None of these keys
replaces the established constructor/prototype link.

**INFERRED finite history:** an original private mathematical model follows that exact input order through
the recovered hash/relocation/rehash rules. It checks every intermediate key,
logical Value and descriptor: `F` finishes these writes with two keys at
capacity eight; `P` has 44 distinct keys after capacities 8, 16, 32 and 64.
This is a derived successful-insertion history, not a captured allocation or
execution, and it does not model collector callbacks or establish the global
table's capacity.

Actual inherited inventories, concrete visitor/setter interfaces and finite
parent chains remain required. Modified parents, name aliases, getters,
category `0x17`, captured function frames and arbitrary hook implementations
are outside this profile. Fresh object/map state by itself would not justify
skipping the parent visit or later method installation.

Temporary cleanup also remains stateful. The prototype setter's retained
temporary function and the outer function/object Values reach `0x1401860e0`
even when fresh old fields are null. The bounded positive-count/root-buffer
contract in [FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md) applies:
valid references, nonoverflowing counts, successful ordinary disjoint buffer
allocation, no zero-count destruction and no allocation-failure collector
fallback. Null old fields do not make the whole constructor call-free.

## Growth preserves admitted stored Values, not arbitrary objects

Original member-table growth `0x1401a3f20` allocates a fresh table, reinserts
occupied entries in old physical-bucket order through `0x1401c2c30`, releases
each copied old entry and frees the old allocation. Reinsert/collision copies
retain keys and managed Values before old-entry cleanup. The unchanged
original function, object, token, Boolean and marker profiles preserve their
logical identities and descriptor bytes through these operations.

This requires the exact object categories and function-reference shapes.
The original Function.prototype, Object.prototype and ordinary namespace
objects have the concrete category-6 getter; MovieClip prototype has category
`0x0b`. Native function Values have null auxiliary frames. Deferred tag-11
tokens retain their name node without calling the captured factory; internal
marker tag `0x0a` has no retained payload. These observations do not establish
copy/cleanup behavior for arbitrary getters, captured frames, special-conversion
objects or malformed Values.

Table storage must be separate, counts/capacities/indices valid and bounded,
key-node identity stable, and all reached reference decrements remain on the
established positive-reference path. The live destination copy explains why
ordinary retained entries survive relocation, but collector metadata can still
change. Allocator reentry, failure, zero-count destruction, overflow and
arbitrary imported-function behavior are not silently assumed pure.

## Later global and namespace registrations

The original global MovieClip store at `0x14020fb9d` supplies the same `F`
as a native tag-8 data Value. Before `0x14020fee7`, the remaining top-level
interval contains two Value cleanups, one Key store, 34 deferred registration
wrappers and five namespace-helper calls. The deferred names are distinct
and exclude MovieClip. The wrappers store factory tokens and registry rows;
they do not execute those factories.

The separate lazy registry is `C+0x20`. Under its actual fresh-null start,
distinct finite inserted keys, null cached-function fields and the ordinary
successful allocation profile, its insertion/growth helpers retain copied keys
before retiring old entries. Null cached-function fields exclude cached-function
cleanup. Positive eight/double growth excludes the clear path. This is a
different table and growth routine from the own-member hash and numeric cache.

The five namespace calls traverse nine original components. Their exact paths
are retained privately. The admitted fresh path finds either a missing member
or the previously created ordinary `flash` object. It uses concrete global or
ordinary raw getters and direct tag-6 conversion; it does not encounter a
deferred factory token, inherited getter or special conversion object at a
namespace component. The original Object.prototype's nine-key inventory
contains none of the six component names and has a null parent; original
resolver fields are cleared. This produces six missing-component lookups
and three own `flash` hits under the stated fresh upstream state. New ordinary namespace objects and their maps are
separate from `F` and `P`. This finite lookup condition is essential to the
claim that registration leaves cached factory fields null.

After cache6 insertion, the complete top-level interval has 80 call sites.
Its five direct-or-installer numeric-cache lookups use key 7, already present
in the finite sequence. The read-only equality-hit path avoids the lazy factory
resolver. Other selected work uses the separate registry, StringManager,
global/namespace maps, fresh function objects and bounded cleanup. It does not
replace numeric key 6, global MovieClip or `F.prototype` under the documented
original inventories and storage conditions.

## Handoff and limits

This evidence replaces the **results** “cache6 already contains P” and “the
original source writes already produced the F/P links” with concrete insertion,
retention, collision, copy and propagation contracts. It does not replace all
earlier assumptions with an unconditional fresh-startup claim. The remaining
input obligations include actual prefix map states/no extra cache insertions,
original concrete constructed objects and parent inventories, successful
allocations, stable interned names, exact typed Values, valid references,
undamaged frames with normal successful cookie checks, and absence of
concurrent or external mutation.
The original 46-name global inventory establishes exact source names and
supplied Value kinds; this increment does not independently derive every
earlier global producer or its final physical capacity from an empty allocation.

At the guarded `0x14020fee7` handoff, the original `C/cache6=P`,
`G.MovieClip=F` and `F.prototype=P` identities can be carried into
[FRESH_PROTOTYPE_BINDINGS.md](FRESH_PROTOTYPE_BINDINGS.md). That document's
actual `C+0x70=NULL`, later render/root-state and cleanup conditions are still
required. Nothing here executes InitActions, registered constructors, saved
handlers, dynamic methods or event scheduling, or proves native runtime parity.

## Evidence and provenance

Private root: `evidence/fresh-bootstrap-bindings/` below the project data root.
Three target-free REA reads completed with exit zero, untruncated output and
unchanged owner-executable hash. The focused original verifier matched five
selected ranges, **375 instructions / 1,389 bytes**, to the pinned PE and
checked chained unwind extents. The 11 padding bytes after the classifier's
actual final return are excluded. `final.bundle.json` and `reset.json` retain
the canonical records and export/reset with no active target.

Independent cache, linkage, interval and member-retention audits retain exact
source hashes, complete-body boundaries, finite source inventories and
conditional call/write contracts. `receipt.json`, `source-ledger.json` and
`MANIFEST.json` bind the combined result. All native instruction extracts,
binary tables, complete method/name lists and scripts remain private. This
docs-only increment requires no Rust build or runtime test.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's build `1.52.0.1700` `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Exact insertion, constructor, Value, table and PE evidence |
| Project documents cited above; `REA_ENUMERATION.md`, `REA_CLASS_REGISTRY.md`, `REA_MOVIECLIP_CONSTRUCTOR.md`, `REA_FUNCTION_CALLS.md`, `AS2_GLOBAL_BASELINE.md`, `ATTACHED_MEMBER_NAMES.md`; retained canonical bundles and inventories individually hashed in private ledgers | Original project investigation; underlying native artifacts remain proprietary | Reuse established hash, copy, constructor, registry, namespace and cleanup paths without duplicate captures |
| Installed REA skill/guides and REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling | Bounded static reads, complete Evidence export and reset |
| Python standard library | PSF tooling; all new scripts are original | Independent PE/hash checks, finite arithmetic and private receipts |

No external VM or SDK implementation was copied, and no shared configuration,
live data, account, launcher or other project's source was changed.
