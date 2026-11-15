# Mouse state preservation after fresh loader construction

**VERIFIED (static, conditional):** the selected engine path forwards state
registration into the freshly constructed loader bag, and the selected AS2
movie path uses a distinct MovieDef bag as the MovieImpl bag's parent. The
MovieDef bag in turn retains the loader bag as its parent. The state-4 cache
therefore depends on three live maps and their preservation, not just the
fresh constructor's default categories.

**Not established:** that a current or fully initialized frontend has no state
4, that every intervening engine/loader callback preserves these maps, or that
`Mouse.hide` completes. This investigation supplies precise checkpoints and
remaining call boundaries. It does not select the null-handler behavior or
study the diagnostic at `0x140186d10`.

Target: owned build **1.52.0.1700**, pinned
`MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
See [REA_MOUSE_HOST.md](REA_MOUSE_HOST.md) for the callback and cache contract,
and [REA_FRESH_MOUSE_STATE.md](REA_FRESH_MOUSE_STATE.md) for the fresh-bag
input to this investigation.

## Identities and registration

Names below distinguish public and secondary interfaces. Offsets are hex.

| Name | Selected identity |
|---|---|
| `E` | Engine constructed by `0x14125dd90` |
| `L` | Public loader embedded at `E + 44` |
| `O` | Outer object returned by `0x1400515a0`, stored at `L + 8` |
| `C_L` | Loader backing bag, `[O + 20]` |
| `D` | Fresh selected MovieDef |
| `C_D` | Separate definition backing bag, `[D + 20]` |
| `M` | MovieImpl |
| `C_M` | Separate MovieImpl backing bag, `[M + 80]` |

`0x140031ad0` installs table `0x1424fd1f8` on `L` and invokes
`0x1400319a0`. The latter stores the fresh `O` at `L + 8`. Registration has
**two** forwarding adapters:

1. `L` virtual `+10` is `0x1400314a0`; its virtual-zero getter
   `0x140087860` returns `O + 10`.
2. That secondary interface has table `0x1424fe218`, again using
   `0x1400314a0`. Its getter `0x14004e330` returns `C_L + 10`.
3. Table `0x1424fe410` then selects concrete setter `0x140050a60`.

For a nonnull state, the map helper reads the **actual object's DWORD at
`+10`**, not the caller's numeric `EDX`, as the key. Null registration instead
removes the numeric key. The independently sealed map proof is
`evidence/rea-fresh-state-map/finding-summary.json`, SHA-256
`01294142136280dfabd127a667cef8fbf5bf663c49dad3f04e3a78b8cb73449d`.
It preserves existing pointers/categories under its stated valid-map,
ordinary allocation, lock, ownership and nonreentry conditions; final releases
remain relevant.

The selected constructed categories are grounded as follows. Numeric values
here are decimal.

| Stage | Actual state categories, in order |
|---|---|
| `0x1400319a0`, with engine-supplied inputs | 9, 13, 26 |
| `0x1412706b0` | replacement 9, 10, replacement 2, 35, 36 |
| `0x14125dd90`, after that setup | 5, 6; later 19 after the intervening initializers |

The supplied 26 and 9 are constructed inline by the engine, with category
writes at `0x14125de20` and `0x14125de61`. State 13 is constructed inline at
`0x140031a5d`. The setup constructs 9, 10 and 35 inline; `0x140052db0`
constructs the category-2 wrapper. The newly completed constructor
`0x1402e8710..0x1402e8766` writes category 36 at `0x1402e8720`.
Its two calls to `0x14006bd50` install function pointers in a separate bounded
global dispatch array, without invoking them or touching a bag. The later
state 5 is **not** the mouse state.

The fresh defaults are 2, 11, 22 and 23 under the sibling fresh-constructor
proof. Consequently the selected registrations do not themselves insert 4.
This conclusion is conditional on the actual constructor/ownership profile,
not a pattern search for literal `EDX = 4`.

## Releases and the engine preservation frontier

The earlier vector-constructor boundary is closed for normal successful
execution. At `0x14125dee4`, `0x141c6ee74` calls the fixed constructor
`0x14059c400` five times at 16-byte strides starting at `E + 7c`. That
constructor only zeros the element's two qwords. The successful path does not
invoke the supplied exception-cleanup destructor. These elements do not overlap
`L` or its separately allocated bags.

Replacement can invoke destructors even when the new category is not 4:

- Replaced engine state 9 uses table `0x1421c12d8`, destructor
  `0x141261a20`, and base `0x140031cb0`. The selected bodies only update the
  dying object's vtable and optionally pass that object to allocator free.
- Replaced default state 2 uses wrapper destructor `0x140052e30`, which
  releases its `+20` payload. For the actual fresh payload, table
  `0x1421c0ff8` selects `0x1412611d0`, then `0x140047e30`. The latter clears
  one global singleton pointer only if it equals the dying payload, and
  tail-calls the base destructor. This does not write a bag. It is not a
  proof of diagnostic/log-call behavior.

Under the fresh/map witnesses plus successful distinct allocation, ordinary
free, no reentry, valid reference counts and no concurrent mutations, these
selected operations preserve the no-4 projection through the known prefix
before the call at `0x14125e17a`.

At that address the already published engine enters `0x141270c20`. Its full
7,974-byte body is retained. The first call is `0x140793e60` at
`0x141270cb1`, which calls `0x14079d210` at `0x140793e77`. The complete latter
body still reaches unexpanded global/name initialization helpers, starting
with `0x14077bed0` at `0x14079d252` and conditionally `0x1407b1170` at
`0x14079d25c`. This is the first remaining transitive-call boundary of the
selected engine prefix. It is **not** evidence that those helpers install 4,
nor a claim that an arbitrary callback was observed there.

The subsequent complete `0x1412708b0` first calls unexpanded
`0x1406f4d20` at `0x14127090c`. Complete `0x141272b50` removes state 17 and
conditionally installs 20 and 1 through the same loader, but contains further
configuration/construction branches and calls. Its category-1 inline write is
proved; this investigation does not infer all constructors' return categories
from registration arguments. These later bodies cannot be treated as blanket
no-mutation barriers. The successful prefix is not a proof through engine
constructor return.

## Actual parent-bag lineage and definition publication

On the selected nonempty-filename, fresh-lookup-success path:

1. `0x140031710` loads `O = [L + 8]` and passes it to `0x14004ffc0`.
   The latter constructs context `B` through `0x1400927a0`, which retains
   exactly `O` at `B + 68`.
2. `0x14004e4b0` loads `[B + 68] = O`, then `[O + 20] = C_L`, passing
   `C_L` as the sixth argument to `0x14008d520`.
3. That constructor allocates a separate bag and calls
   `0x1400505d0(new_bag, C_L)` at `0x14008d734`.
4. Complete `0x1400505d0..0x140050653` initializes its local map to null,
   retains the supplied `C_L`, and stores that pointer as the new bag's
   `+20` parent. It does not copy the source map. `0x14008d752` publishes
   this `C_D` at `D + 20`; the remainder of that constructor is only its
   return epilogue.

This is an exact constructor-return checkpoint, not preservation through all
of movie loading. The next definition-exposing calls in `0x14004e4b0` are:

| Selected branch | First subsequent exposure |
|---|---|
| Optional task-output pointer nonnull | `0x14004e591`: calls `0x14008a8e0` with `D` in `R8` |
| Task-output pointer null | `0x14004e645`: calls `0x14004c930` with `D` in `RDX` |

Both successful paths reach the second call; a later optional lookup-object
virtual `+10` call at `0x14004e65a` and cleanup are also preservation boundaries.
The source does not directly overwrite `D + 20` after its constructor, but
that fact cannot substitute for these callees' effects.

For the selected successful AS2 case-3 root, the root factory retains the
original `D` in the target's `+b0` field. Its actual virtual `+208` getter is
`0x1400a12b0`, which returns that field. The level-zero branch of
`0x14005fdb0` calls this getter, installs `D` at `M + 48`, then loads
`C_D = [D + 20]` and writes it to `C_M + 20` at `0x14005ff5f`, retaining it
and releasing the previous parent. This is not inferred from a filename,
script origin, state snapshot or AS3 factory.

The resulting selected relationship is:

`C_M local map → C_D local map → C_L local map → null parent`.

Every arrow must still retain its proved identity and every map must still
have its supplied contents at the observation point.

## Cache-refresh conditions

`0x140063b13` queries state IDs starting with 4 from `C_M + 10`, with output
slots initially null. Concrete `0x140050920` uses the local map, then falls
back to the retained parent's secondary interface. A full-chain miss leaves
state-4 output null; `0x140063b2b` places that result in `M + 228` after
releasing the previous cached object. A nonnull old cache can itself introduce
a destructor boundary, so the fresh-cache profile must supply null or a
separately proved old-handler release.

One precisely bounded route into that query has `M + 4310` bit 20 clear and
`M + 50` nonnull. It reaches `0x140063aa5`, then calls `0x1400612f0`.
The newly complete, split-unwind body is `0x1400612f0..0x1400613fa`:

- If `[M + 4358] == 0`, it returns without calls or object writes.
- Otherwise it can release retained objects, compact an array, and invoke
  root virtual `+1c0` at `0x1400613c3`, followed by further helpers. These
  are explicit additional preservation boundaries.

Thus a guarded state-4 miss can be derived from current empty child maps,
known parent identities, a preserved no-4 loader map, empty pre-refresh list,
null previous cache, and the selected refresh branch. Constructor facts do
not establish those current conditions automatically. No implementation or
runtime result was changed by this investigation.

## Evidence and provenance

Private records live under `evidence/rea-mouse-state-preservation/`:

- `sealed.bundle.json`: seven completed, untruncated REA records, exit 0,
  unchanged target hash. `verification.json` byte-checks 3,024 nonsuperseded
  instruction records and 152 table bytes. One short destructor window ended
  inside its tail jump; its four pseudo-instructions remain preserved and
  are explicitly superseded by the complete five-byte jump capture.
- `engine-categories/finding-summary.json` and `verification.json`: retained
  category/receiver proof, 659 instructions and 2,775 bytes checked.
- `engine-categories/lineage-finding-summary.json` and
  `lineage-verification.json`: retained definition/root lineage, 1,338
  instructions, 5,123 bytes and 16 table bytes checked. Counts are separate
  and may overlap; they are not a unique whole-program coverage total.
- `finding-summary.json`: combined facts, dependency hashes, frontier and
  verification receipt.

Key new records:

| Evidence ID | Purpose |
|---|---|
| `ev_537767c79e687a067482311ab6f5daafcf000562c9b81ec99db690d2f40666a8` | Category 36 constructor; complete engine stages |
| `ev_df97bcfdd9c60a6e415b912e0c149c71526f18f43c9815973e0b2b78a7d437d7` | Fixed vector construction; release and pre-cache paths |
| `ev_54b8508d8d8e8ec5a33b6491f6f4a954419672e9ca2ab3141ac1b97065cfd698` | Exact adapter tables, refcount helpers and initial name-map helpers |
| `ev_cfe62b1c185c3037250d01e75692d7a40b0bfb9cf9dab6d7013d086a91650824` | Definition bag retains its parent; name-helper call boundaries |
| `ev_ef6ad2b76f41e7f954b1891bb35edcc442938f7f8cf940911aeef44a98aa3010` | Actual replaced state-2 wrapper/payload destructor binding |
| `ev_669b84c30ba26fc8f301ddf0bababb150254c81c4779d7c1d731c15a65ed8cf4` | Corrected complete destructor tail jump |

Sources read: the owner's proprietary pinned client, for authorized static
compatibility study; the original project docs `REA_MOUSE_HOST.md`,
`REA_NATIVE_ALTERNATIVE.md`, `FRESH_ROOT_NAME.md`, `REA_DISPLAY_LIFECYCLE.md`
and the retained private mouse-host, frontend-image-context, class-registry,
version-proof and root-constructor records referenced by the receipts. These
are project findings, not reused implementation code. The REA skill and native
artifact guide supplied the workflow; installed Rizin produced bounded static
disassembly. Original Python standard-library scripts verified file bytes and
receipts. No third-party VM source, client execution, network request, shared
configuration change, asset publication or diagnostic invocation occurred.
REA evidence was exported and the investigation reset.

## Follow-up: MovieImpl public secondary interface

**VERIFIED (static):** the secondary interface at **`M + 10`** is distinct
from the secondary interface at **`C_M + 10`**. The complete MovieImpl
constructor loads table `0x1424feb48` at `0x1400650b8` and stores it at
`M + 10` at `0x1400650c2` (retained evidence
`ev_149d3f952db9806b9f26bcba22adae820ef892183dd23cf8afafbedb7bdbc709`).

Its virtual `+18` is forwarding adapter `0x1400314e0`. This calls the
receiver's virtual-zero getter, `0x140062be0`. That getter reads
`[receiver + 70] = [M + 80] = C_M` and returns `C_M + 10` when nonnull.
The adapter then tail-calls that returned interface's virtual `+18`, preserving
the requested numeric state ID. The concrete backing table `0x1424fe410`
selects **single-state lookup `0x140050840`**, not bulk lookup `0x140050920`.

Complete `0x140050840..0x140050919` compares the requested ID with each
candidate state's actual DWORD at `+10`, retains a local hit, and otherwise
calls the parent's secondary interface `+18` or returns null when the parent
is null. Therefore an ID-2 request on `M + 10` follows the actual MovieImpl
bag and its parents. This proves dispatch/ownership, not the current presence
or contents of state 2, and does not invoke or inspect a diagnostic helper.

Canonical new evidence:
`ev_4283c0a7e86560c60e930ba8a0e3b0dc28a041f53bd9ef1319f7cc00f0d6543c`,
`evidence/rea-mouse-state-preservation/movieimpl-secondary.bundle.json`.
The corresponding verification checks 85 instructions / 274 bytes and 80
vtable bytes against the unchanged owned PE. Receipt:
`movieimpl-secondary-receipt.json`, SHA-256
`4544cd46fde7efd683a67b02c21a8d5129c2ed7c3bbc33f11c079340bef6c161`.

## Follow-up: optional task exposure at `0x14004e591`

The previous `0x14008a8e0` boundary is now narrower. Its complete body spans
`0x14008a8e0..0x14008aab4`; multiple unwind ranges do not represent multiple
independent constructors. It receives the new task, load context `B`, selected
`D`, and the caller's fourth argument. These identities must remain distinct.

Base constructor `0x14004eae0` initializes task metadata, retains `B`, stores
it at task `+18`, and calls `0x14004dc00([B + 68], task)`. Since
`[B + 68] = O`, this call registers the task in **the outer object's list**,
not in a state map. Complete `0x14004dc00..0x14004dc77`:

- Takes the lock at `O + 40`.
- Allocates a separate 24-byte list node and stores the task pointer at node
  `+10`.
- Links that node into the valid intrusive list headed at `O + 30`.
- Releases the lock without directly invoking a task/provider callback.

A null allocation is not a successful profile: the native code subsequently
dereferences the node. Preservation requires a valid list, successful distinct
allocation, ordinary lock/allocator effects, and no concurrent/reentrant
execution of the newly exposed task. Interior pointers alone do not prove
these conditions.

The remaining constructor retains `D + 38`, stores it at task `+38`, and
loads the DataDef from that binding task's `+18`. On the selected concrete
DataDef profile, constructor `0x140090520` installs table `0x142504680`;
its virtual `+98` is exactly `0x1400863c0`. The entire eight-byte getter reads
`[[receiver + 40] + 78]` into `EAX` and returns, with no calls or writes.
The task stores bit 4 of this result as a Boolean byte.

The provider tested next is **snapshot state 28**, not state 4. Complete
snapshot constructor `0x140065b60` queries IDs
`9, 10, 11, 15, 16, 27, 28` and stores their retained results in the seven
snapshot fields. Its last result, state 28, is at snapshot `+48`; the task
reads it through `[B + 10]` at `0x14008a967`.

The actual branch predicates are:

| Supplied current profile | Selected path |
|---|---|
| Snapshot state 28 is null | Skip provider calls |
| State 28 is nonnull, but the concrete getter's bit 4 is set | Skip provider calls |
| State 28 is nonnull and that bit is clear | Enter provider path |

On either skip path, the task writes its own `+58` field to null at
`0x14008aa91` and returns. After the proved base-list registration and concrete
getter, that path has no additional calls. Under the stated ownership,
receiver-table, snapshot, allocation, lock and concurrency conditions it does
not change `D + 20`, `C_D`'s map/parent, or `C_L`'s map/parent. A fresh loader's
initial keys alone do **not** supply the current state-28 snapshot condition.

The remaining provider path is explicit. MovieDef virtual `+90` at
`0x14008a9fd` is not itself an unknown callback: the original MovieDef table
`0x1425048b0` maps it to complete nine-byte getter `0x140087740`, returning
`[[D + 38] + 10]`, with no calls or writes. The first still-unproved provider
invocation is **state-28 object's virtual `+10` at `0x14008aa1f`**. Its
returned object's virtual `+8` is then invoked at `0x14008aa4a`, receiving the
binding task. Neither invocation is admitted as preserving the bags without
its concrete provider contract.

The getter receiver lineage is also corroborated: MovieDef constructor
`0x14008d520` passes its input DataDef to binding-task constructor
`0x14008c9a0`, then stores the returned binding task at `D + 38`. That binding
constructor retains the exact DataDef at its `+18`, and stores the allocator
at `+10`. This is a producer assignment, not a new proof through every earlier
constructor callee. Its first unexpanded post-assignment call is
`0x14008ca0e → 0x1400c9a60`; later synchronization constructors and a conditional
DataDef virtual call also remain outside this post-MovieDef task scope.
Consequently the concrete DataDef/MovieDef table and pointer identities are
explicit use-time conditions. They are not inferred solely from the assignment.

Evidence:

- Retained `ev_d1ce3551393a22d2f8defda1c432949f4eec4846e4ff5751b665a8518425e77b`
  proves the complete task and base bodies; retained
  `ev_f44f02ac96fc0430d1e48cf519c943bb71515d58ce48dd012d8c5c7483278f22`
  proves the snapshot constructor. `optional-task-retained-verification.json`
  checks 227 instructions / 883 bytes.
- New `ev_74e04ab0990e01cd068b0ef8d7c1379ffb2441b249835430f26510f2aa9d6db7`
  proves list registration and the seven-state ID table.
- New `ev_0dac96f8d1101678bfd805a27b491bf52a525e9bff40eee35921231ff1e401c9`
  proves both vtable slots and the complete DataDef getter. The MovieDef
  getter was already retained in
  `ev_d37bdccbdab5682db5e6270ea46dc77ec4ccc20dda3863619e3b3b6a1940fbf8`.
- `ev_aa71dfe465b1626a3f592ab27268c777d63dd5dec77fafe2deee716fa39dc964`
  captures the binding constructor's missing tail from `0x14008cb1c` through
  `0x14008cbb0`. This supersedes two pseudo-instructions from an old short
  capture ending inside the instruction at `0x14008cb1c`; the old result is
  preserved and is not used as semantic evidence.

Private `optional-sealed.bundle.json` contains four completed, untruncated
new records, including the preceding secondary-interface follow-up. Its
verification checks 158 instructions / 549 bytes and 124 table bytes against
the unchanged pinned PE. No native client or script execution occurred; REA
was exported/reset. Additional retained sources are the project's private
header/version and binding-task evidence, recorded with exact paths and hashes
under `optional-getters/`; their ownership/licence is the same private
interoperability study described above.

## Follow-up: cache publication and the later key release

The `0x14004e645 → 0x14004c930` call is now bound to concrete cache/key
objects. Let `F` be the live pending cache entry, `K` its resource-key payload,
and `I` the key-interface object. `I` is the **object at `0x142adbe00`**;
its first qword is vtable `0x142504598`. Confusing this object with the vtable
would give the wrong indirect-call interpretation.

`0x1400884c0` builds `K` from the original DataDef and state snapshot, then
`0x14004b9a0` stores the pair `(I, K)` with a retained key reference. The
concrete methods are:

| Interface slot | Native body and effect |
|---|---|
| `+8` | `0x140086100 → 0x140031ce0`, retains `K` |
| `+10` | `0x140126d70 → 0x140031cf0`, releases `K` |
| `+18` | `0x140210e00`, returns key kind 1 |
| `+20` | `0x140086df0`, hashes the DataDef and seven snapshot pointer identities |
| `+28` | `0x140086e20`, compares interface, DataDef and those snapshot pointer identities |

The complete hash/equality bodies have no calls or writes. Fresh entry
constructor `0x14004c760` retains another copy of this same pair at
`F + 28 / +30`, stores its cache owner at `F + 10`, and initializes resource
`F + 20` to null and state `F + 18` to zero. Actual insertion into the cache's
generic table uses helpers `0x14004bc40 / 0x14004bff0`, not expanded here.
A valid matching pending entry is therefore an explicit publication
precondition, not a consequence assumed from unchecked generic map code.

Complete `0x14004c930` does perform real writes. It retains `D`, stores
`F + 20 = D`, marks `F + 18 = 1`, replaces the matching table cell with `D`,
and sets **`D + 10` to the cache owner**. It does not directly write
`D + 20`, `C_D + 20` (parent), or `C_D + 28` (local map). Describing this
routine as entirely read-only would be incorrect.

Its lookup through `0x14004ba80` is bounded by encountered entry kinds:

- A pending entry carrying original interface `I` uses the proved pure
  comparator. The strongest preservation profile admits only these entries
  along the traversed collision chain.
- A resolved resource invokes its virtual `+8` at `0x14004baec`; a foreign
  key interface can invoke an unknown comparator at `0x14004bb4b`. These
  remain explicit callback boundaries.
- The actual MovieDef table's `+8` is identified as `0x140088b10`, which
  builds the same kind of key from that definition. This is useful positive
  binding evidence, but its allocation/lifetime path is not silently included
  in the pending-only profile.

Publication then calls completion signaling `0x14002fe00` on event
`Q = F + 40`. Fresh event construction passes `allow_handlers = false`,
initializing both **`Q + 10`** and the nested mutex handler-container field
**`(Q + 28) + 10`** to null. These are constructor checkpoints, not assertions
about their current values after arbitrary loading or provider execution.

The signal path changes event flags, processes its separate waiter list,
and unlocks the nested mutex. Waiter processing invokes imported `SetEvent`,
so it can schedule other threads. Null handler containers do not remove this
concurrency boundary. If a handler container is nonnull, callbacks are real:

| Current handler container | Native callback route |
|---|---|
| Nested mutex container | `0x140030bf5 → 0x14002f5a0` after the applicable unlock/count conditions |
| Outer completion-event container | `0x14002fe54 → 0x14002f5a0` |

The handler visitor invokes stored function pointers at `0x14002f5d1` or
`0x14002f614`. No generic no-effect notification claim is justified.

The later `0x14004e65a` call is also resolved: it invokes original `I + 10`
on the caller's local `K`, not an arbitrary MovieDef method. With both the
local pair and live pending entry retaining `K`, and no intervening reference
mutation, this release cannot reach zero and cannot invoke `K`'s destructor.
The caller's subsequent handle cleanup through `0x14004c720 / 0x140031cf0`
is beyond this selected key-release endpoint and remains explicit.

**DERIVED conditional preservation:** through publication and the selected
local-key release, the bag fields remain unchanged when storage is live,
distinct and well formed; original key/entry tables and a valid matching
pending-only collision chain are supplied; both current handler containers
remain null; and synchronization/imports, allocations and reference counts
have ordinary effects without reentry or concurrent mutation. This profile
still does not establish the current value of state 4 or `Mouse.hide` behavior.

Private evidence is under `publication-cache/`. Four completed, untruncated
REA records were exported/reset, with 966 instruction spans / 3,454 bytes and
72 table bytes independently checked. Canonical records:

- `ev_d3fac7b60e34b33571898b0f9475ba01d2496900f0de75261c066ca3f0351fd9`
- `ev_5b70711e6e25577a118ebbefce5a1d37569c71a247bec63f0bac253d5e487654`
- `ev_0e8f92d7acb96087d0654181027be288b9aaab59d3b864ccb249849139097433`
- `ev_466fb8f1387baeba333c629825dd2b12d435c51441e73c72d0b31278d034388c`

`publication-cache/finding-summary.json` SHA-256:
`6ca05a479a72150e873258a518e4ad49df2259e4277845de4fab22a11b925880`.
A client-side preview was truncated; the full exported record was reused and
verified without repeating its producer. Semantic leaf ranges exclude trailing
neighbor/padding decodes. Sources remain the owned pinned PE, retained original
project evidence and REA workflow already listed; no runtime execution or
repository implementation change was made.

## Follow-up: successful fresh-handle cleanup

**VERIFIED (static):** this follow-up supersedes the earlier cleanup frontier
only for the selected successful **fresh status-3** path. The caller's
discriminator is material:

| `0x14004d1a0` result | Selected handle/cleanup distinction |
|---|---|
| 1 | Already resolved resource; later `0x14004c720` branch |
| 2 | Existing pending entry; separate wait/lookup continuation |
| 3 | Fresh pending entry `F`; the successful path later calls `0x140031cf0(F)` |

The fresh producer stores `F` in the caller's output and writes status 3 at
`0x14004d2c6`. `0x14004e4ff` selects that fresh route. After publication and
local-key release, the status-1 test at `0x14004e660` is false; the
status-at-least-2 test at `0x14004e6d1` reaches `0x14004e6da`, releasing `F`.
The selected normal path does not rewrite the local discriminator or pass its
address to later calls. The `0x14004c720(D)` call at `0x14004e5ae` belongs to
task-construction failure and is not part of successful publication.

The entry release cannot be replaced by a generic no-effect operation. Fresh
`F` starts with one reference. Its cache insertion stores the tagged pointer
without retaining `F`; the key-hash work balances its temporary **key** retains
and releases. The map's existing-entry resize happens before insertion of the
new `F`. Under the existing no-reentry/no-concurrency/reference-preservation
profile, `F` therefore still has one reference at `0x14004e6da`, so this call
executes its actual destructor.

The original local key is likewise not an arbitrary callback object. `K`
starts at one reference, the local pair adds one, and its construction temporary
then drops one. `F` adds its retained key reference. With the balanced hash work
and no other reference changes, the local `I +10` release at `0x14004e65a`
changes `K` from two references to one. Final `F` cleanup must follow the
actual key destructor when it releases that remaining reference.

The independent surviving holds are essential:

- The caller retains the original new `D` result in `RBX` and returns that
  same pointer at `0x14004e6df`. Publication separately retains `D` in `F +20`.
- `D +30` owns a retained reference to the same snapshot used by `K`.
- `D +38` owns the binding task, whose `+18` retains the same DataDef used
  by `K`.
- The outer loader `O +28` and live load context `B +60` retain the cache
  owner `H`, independently of `F +10`. `O +28` is the object loaded from
  the resource library's `+10`, not the public resource-library object itself.

These are concrete ownership sources, not an invented cache-held `F`
reference. Extra unrelated references may exist; the named live holds suffice
as lower bounds when assessing releases of `D`, its DataDef/snapshot and `H`.
They do not justify omitting the final `F` or `K` destructor.

The selected entry's deleting destructor is `0x14004cf00`, which invokes
`0x14004c830` and then frees `F` through the allocator. Its body first locks
`H +18`. Only unpublished state zero removes the pending-map entry through
`0x14004bdc0`; published state one follows the `F +20` resource release instead.
The separate caller hold keeps `D` alive, so this release does not reach the
cache-owner detach virtual or `D`'s deleting destructor. The lock is then
released before the entry's event, string, key and cache-owner cleanup.

The final key release is the same original-interface dispatch identified at
`0x14004e65a`: `I +10 → 0x140126d70 → 0x140031cf0`. The actual key's deleting
destructor is `0x140088460`. It releases its snapshot once through
`0x140031cf0`, releases its DataDef once through `0x14004c720`, then performs
base cleanup and allocator release. The shared base destructor `0x140031cb0`
only installs its base table and returns; it invokes no callback. The surviving
`D +30` and binding-task
`+18` references prevent either member release from invoking their destructors
under this profile. Thus `0x14004c720` does occur inside final key cleanup,
although the caller's status-1 handle branch was not selected. The two uses
must not be conflated.

The event teardown is concrete rather than an omitted notification:

- `Q = F +40` is destroyed by `0x14002f760`. Its `Q +50` pointer, when
  nonnull, owns the waiter manager destroyed by `0x140030940 → 0x1400302a0`.
  The manager traverses its free-list at `+28`, closes each OS handle, frees
  each node, zeros fields `+28/+30/+38`, and tears down its critical section.
- Nested mutex `J = Q +28` is destroyed by `0x1400308d0`; its nonnull `J +20`
  OS record is closed and freed. Both the mutex and outer event then execute
  their handler-container base cleanup through `0x14002f680`.
- The current-null conditions `Q +10 = 0` and `J +10 = 0` skip the
  handler-container destruction branches. The strongest admitted cleanup
  profile additionally requires no active waiters and current empty waiter
  lists. Imported `CloseHandle` and `DeleteCriticalSection`, and allocator
  release, retain ordinary lifetime/nonreentry conditions. This is not a
  claim about arbitrary concurrent notification or waiter teardown.

The entry's string field `F +38` also has a real release. Its constructor
retains the node initially selected at `0x142adb2f0`; the selected successful
publication does not replace that field. Cleanup masks its low two bits,
decrements the referenced node's count, and calls allocator release if that
count reaches zero. The image's initial count is not substituted for a current
reference-count invariant. Valid live-node ownership and ordinary allocator
behavior remain explicit conditions, including the zero-count branch.

Finally, releasing `F +10` cannot destroy `H` while the independent load-context
hold survives. The selected outer loader caller keeps the newly created `B`
alive across its `0x14004ef00(B)` call and releases it only after that call
returns. Its selected nested `0x14004e4b0` calls therefore execute while
`B +60` still retains `H`; the optional `O +28` hold is additional evidence,
not the sole lifetime argument.

**DERIVED conditional endpoint:** for the admitted successful fresh lookup,
publication and cleanup path, execution returns the original `D` at
`0x14004e6df` after the actual final `F` and `K` destruction. Their selected
cleanup changes owned entry/key/event/string/refcount storage but does not
change `D +20`, the definition bag's parent/local map, or the loader bag's
parent/local map. This extends the earlier publication endpoint to the
successful return; it does not include failure cleanup or existing-resource
and existing-pending continuations.

The endpoint retains the earlier concrete key/task/table identities, successful
distinct allocation and ordinary reference-count/lock/import assumptions. The
state-28 provider path, foreign-key/resolved-resource collision callbacks,
nonnull notification containers, active waiters, allocator reentry and concurrent
mutation remain explicit gates. It does not establish current state-4 absence,
whole-engine preservation, or a completed `Mouse.hide` invocation.

The joined private receipt is
`evidence/rea-mouse-state-preservation/cleanup-receipt.json`, SHA-256
`98e979ca50652158a88f2abaa46763a1078cf0e5719860ef0fba1b7bea005d2b`.
It binds the caller selection, destructor mechanics, surviving ownership and
prior publication receipts. The six new complete REA records cover 517
instruction spans / 1,906 instruction bytes plus 32 data bytes. Reused
base/refcount helpers add 16 spans / 52 bytes; caller and survivor checks cover
267 and 1,759 spans respectively, with overlap explicitly recorded rather
than summed as unique coverage. All selected bytes match the unchanged pinned
PE. New native evidence was exported and the investigation reset; no native
execution occurred.

Canonical cleanup Evidence IDs:

- `ev_9fa31832b6aa474b2abf75dce40f5aa34e351798fdcb3fca329c98d90e0ab925`
- `ev_eb78d9244758e505acb47cfcbc29ca0ca28c5a49400a3405510783f83edf6bdb`
- `ev_de9a484c35723c14e7fafbcaadfa0ef11a31319a4004c9f7b05f05cdac33f54f`
- `ev_9cfa63deb575ec874e5433ff6a3094422110f143dca3bde0e10c2cf4dd9d046b`
- `ev_d9c4acc91ad1c59e1065ec6064f33f4d6d699f787fb98c12552ccbfbf1f3335d`
- `ev_62739bdc74d743882af1f506e8bae7494c500ef2430d1287bf1227a8477bd575`

The survivor verifier rejected an invalid final decode in an older enclosing
caller capture. Its accepted caller slice ends at `0x14004f4b2`, after both
relevant calls; no whole enclosing-tail proof is claimed. The corrected
binding-task continuation and semantic leaf bounds likewise remain explicit.
Sources are the same proprietary owner-provided PE and retained project REA
Evidence already recorded above, used only for interoperability study. No
client bytes, extracted data, or implementation code enter this documentation
commit.
