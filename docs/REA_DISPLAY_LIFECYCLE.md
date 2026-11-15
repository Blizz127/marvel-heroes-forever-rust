# Native AVM1 display-constructor scheduling

**OBSERVED (REA, pinned 1.52.0.1700 Win64 executable):** Native display-class
construction uses priority-ordered action records. It cannot be reproduced by
calling the registered constructor immediately when a saved display placement
is parsed. One registered path first prepares the prototype at priority 1.
The normal drain orders pending DoInitAction buffers at priority 2,
Construct-event records and registered constructors at priority 3, and ordinary
DoAction buffers at priority 4. A separate token-filtered drain deliberately
leaves other records pending, so that priority table is not a universal order
for every nested timeline operation.

This increment records static compatibility evidence only. No native client
execution, synthetic queue test, rendered comparison or complete lifecycle
parity claim is made. Runtime class lookup itself is documented in
[the native registry](REA_CLASS_REGISTRY.md).

## Container, insertion and consumption

**OBSERVED:** Known callers pass `AS2root+0x80` to record allocator/inserter
`0x1401c7dc0..0x1401c7ed2`. The supplied priority is a separate integer from
the record kind written by the caller afterward.

| Container-relative field | Recovered role |
| --- | --- |
| `0x18 * priority + 0x00` | Bucket head |
| `0x18 * priority + 0x08` | Insertion cursor |
| `0x18 * priority + 0x10` | Bucket tail |
| `+0x90` | Counter incremented on insertion |
| `+0x98`, `+0xa4` | Reusable-record list and its count |
| `+0xa0` | Current token copied to record `+0x70` |
| `+0xa8` | Token counter used by observed scoped producers |
| `+0xb0` | Allocation owner |

New records are 0x78 bytes. If the bucket cursor is null, insertion prepends to
the head; otherwise it inserts after that cursor. The new record becomes the
cursor, and becomes the tail only when its next pointer is null. This is not an
unconditional tail append. Consecutive calls with no intervening cursor change
retain their insertion order, but reentrant insertion must respect the cursor.

Complete normal iterator `0x1401c7fe0..0x1401c80b9` scans priorities 0 through 5
in ascending order. It unlinks the selected head before returning it, updates
the bucket cursor/tail as required, and retires the previously returned record
on the next iteration. If the insertion counter has changed since the previous
iteration, it restarts at priority 0. Thus an executing action can add an earlier
priority that runs before the remaining later-priority records.

Complete normal drain `0x1401ccd10..0x1401ccd77` repeatedly obtains that next
record, passes it to `0x14017cf80`, and cleans up the final record. Complete
filtered drain `0x1401d3590..0x1401d3604` instead uses iterator
`0x1401c80c0..0x1401c81b0`. That iterator also scans priorities 0 through 5 and
restarts after insertion, but selects only records whose `+0x70` equals the
requested token. It removes matches from within a bucket and preserves
nonmatching records. Token filtering is not a priority cutoff.

REA confirms the normal drain at root-vtable slot `+0x38`. Complete short
wrapper `0x1400b2c50..0x1400b2c68` reaches it through its input's `+0x18` owner
pointer. Complete AVM methods `0x14017a2a0..0x14017a3bc` and
`0x14017a3c0..0x14017a40f` reach it through receiver-to-target-to-root pointers
after other target virtual calls. Their complete public operation names and
the top-level advance boundary are not recovered. These positive call sites
do not justify assuming that the normal drain occurs only once at frame end.

The allocator does not validate a caller-supplied priority range in the decoded
body. This is a native internal contract, not permission for a new public API to
accept arbitrary priorities or unchecked indices.

## Named actions and raw record kinds

**OBSERVED:** Loader diagnostics and created tag vtables distinguish DoAction
from DoInitAction. Their complete execution callbacks feed the same enqueue
helper `0x1401ccbc0..0x1401ccc47`. That helper passes its third argument as
the allocator's priority and writes record kind 1, retaining the target and
action-buffer object.

| Source | Native producer | Priority | Record kind/data |
| --- | --- | ---: | --- |
| Registered prototype preparation on one path | `0x140203920` | 1 | Kind 4, callback `0x14019ca30` and retained function |
| DoInitAction | `0x140178a60..0x140178b0b` | 2 | Kind 1, action buffer |
| Registered Construct event | `0x1401d0f70` and `0x140203920` | 3 | Kind 2, event mask `0x40000` |
| Registered constructor | Same display producers | 3 | Kind 3, retained function/frame reference |
| Missing-class fallback | Same producers, subject to their branch conditions | 3 | Kind 4, callback `0x1401cc220` and linkage name |
| DoAction | `0x140178770..0x140178812` | 4 | Kind 1, action buffer |

The broader producer at `0x140203920` creates the priority-1 callback when a
registered class is found. Complete callback `0x14019ca30..0x14019cb84` recovers
the display receiver from the native call frame, retains the supplied function
reference, and calls prototype helper `0x14019c5e0`; it does not invoke the
constructor. Its priority therefore places prototype preparation before pending
priority-2 initializers in the normal drain. The narrower producer at
`0x1401d0f70` sets prototype-related state directly before creating its kind-2
and kind-3 records. These are distinct native paths, not interchangeable snippets.

For the narrower registered path, the Construct record is inserted before the
constructor record into the same priority bucket. With no intervening insertion
or target-state change, the Construct event therefore executes before the
constructor body. Lower-priority-number work inserted during execution can still
intervene. The executor's target-state checks can skip a record after target
state changes; no claim of unconditional invocation is made.

## Actual executor and the late lookup

Complete executor `0x14017cf80..0x14017d059`, retained in the registry Evidence
bundle, first checks the target pointer and a target-state bit. It obtains the
target's adjusted AVM object and dispatches record kinds through that object's
virtual interface:

| Raw kind | Data passed | Virtual slot |
| ---: | --- | --- |
| 1 | Retained action-buffer object at record `+0x18` | `+0x120` |
| 2 | Event descriptor beginning at record `+0x20` | `+0x128` |
| 3 | Function/frame reference at `+0x38`, arguments at `+0x58` | `+0x130` |
| 4 | Native callback at `+0x50`, arguments at `+0x58` | `+0x138` |

The missing-class record stores the name rather than permanently caching a miss.
When callback `0x1401cc220` executes, it queries the current root-context registry
again. On a hit, it sets prototype-related state, then executes local kind-2 and
kind-3 records through the executor directly. Those two callback-local records
are not inserted into the queue again. On a miss, its observed path executes
the event record without a registered constructor. Registration changes before
the callback is reached can therefore affect the result.

By contrast, an initial hit retains the selected function/frame reference in its
records. It does not store a name for a fresh lookup at constructor execution.
A replacement of the registry entry alone therefore does not retarget that
already retained reference. Mutations of the retained function or its prototype
are separate operations and are not excluded by this statement.

The complete common native-callback dispatcher is
`0x140198d40..0x140198f29`; it prepares a native call frame with the current AVM
receiver and invokes the saved callback. The class callback's late registry read
is consequently part of execution, not a static asset-linking rule.

## Token-scoped ordinary actions and construction

**OBSERVED:** Complete frame-action runner `0x1401d4330..0x1401d4447` validates
the requested frame, obtains its tag vector, allocates a token, and visits tag
pointers in vector order. It calls only tags whose virtual `+0x20` predicate is
true. The confirmed DoAction vtable returns true from this predicate; DoInitAction
returns false. This runner therefore selects ordinary DoAction tags, not the
initializer phase. It restores the previous current token before draining the
new token's records.

The observed construction producer `0x140203920` similarly saves the current
token, allocates a new token, stamps subsequent records, restores the old
token, and returns the new token. Follow-up REA decodes the complete caller
`0x14020d8d0..0x14020de66`, installed at secondary-interface slot `+0xb8`.
Its incoming argument 7 (counting the receiver), at entry stack `+0x38`, supplies
raw bit 2. The caller forwards that bit as argument 6 to `0x140203920`.

A zero bit forces the new-object branch; a nonzero bit permits existing-item
branches, including a return before the producer is called. On the new branch,
only a result with target word `+0x6a` bit 7 set is selected for the construction
producer. The caller then publishes the display entry. For that eligible
result, bit 2 clear calls the filtered drain with the returned token; bit 2 set
skips this drain. Its root receiver is reached from the *parent* secondary
interface through its target and root pointer, not substituted with the new
child target. Public flag names and every producer of these flags remain
unassigned.

Re-reading the complete retained producer gives these additional conditions:

| Producer condition | Observed records |
| --- | --- |
| Linkage lookup hits | Priority-1 prototype callback, priority-3 Construct, then priority-3 retained constructor, independent of argument 6 |
| Linkage lookup misses and argument 6 is nonzero | Priority-3 callback retaining the linkage name for late lookup |
| No linkage name and argument 6 is nonzero | Priority-3 Construct record, without a retained registered constructor |
| Linkage misses and argument 6 is zero | No late class-lookup callback from this branch |

On a hit, a separate priority-3 native callback at `0x14019cb90` can intervene
between Construct and the retained constructor when the queried target version
is at least 6 and incoming argument 5 is nonnull. Its payload is that argument;
its implementation and public meaning are not assigned here. On the zero-flag,
no-constructor branch, the same version/argument condition can enqueue this
callback. This must not be silently collapsed into constructor arguments.

Before allocating its new token, the producer also processes supplied event
records. With argument 6 nonzero, a matching native mask `0x200` and a nonempty,
nonterminating action buffer can enqueue a priority-1 kind-2 event under the
*previous* token. The event's public name is not established here. Thus the
constructor token does not cover every record produced by this function.
These concrete branches reject a single unconditional placement schedule.

## Construct handlers and the constructor receiver

**OBSERVED:** The known AVM table maps event slot `+0x128` to wrapper
`0x1401aa850..0x1401aa93e`; the other traced display table maps it directly to
common handler `0x1401a9890..0x1401aa129`. The wrapper reaches that common
handler after its target-state checks. For the observed Construct descriptor,
the common handler can execute its event-action vector before resolving the
dynamic event member.

Complete vector helper `0x1401a9740..0x1401a9886` looks up the event descriptor
in the AVM object's event map at `+0x28`, snapshots the vector count, and visits
32-byte entries from index zero upward, invoking `0x14019aed0` on each. This
establishes stored-vector order. The insertion/parser path that determines that
vector's relation to serialized ClipAction order was not decoded here, so these
findings must not be expanded into a file-order proof. The published SWF
Construct flag identity is already recorded in the project's GFX/AVM1 readers.

**OBSERVED:** Constructor slot `+0x130` resolves to complete dispatcher
`0x140198b40..0x140198d37`. It obtains the environment from the existing AVM
receiver, uses receiver `+0x08` as the call-frame `this` interface, pushes the
record's arguments in reverse stack order, and invokes the retained function's
virtual `+0x50` with its retained frame pointer. The reviewed producers do not
populate constructor arguments; freshly allocated records have an empty vector.
The returned value is cleaned up and discarded;
this dispatcher does not substitute a returned object for the existing display
instance.

The pre-constructor helper `0x14019c5e0` is retained in the prior registry bundle.
It reads the constructor property selected by root builtin `+0x3a0`, confirmed
as `prototype` by the independently retained Array investigation, converts the
result to an object, and calls the existing display interface's virtual
`+0x68`. For the traced display type, the exact slot points to complete setter
`0x1401898d0..0x14018998a`. That setter retains the supplied object, releases
the old reference and stores it at interface `+0x10`, which is full AVM receiver
`+0x18`. If receiver virtual `+0xc8` returns an object, it also forwards the
assignment through that object's interface; that optional object's full role
was not recovered. Do not substitute the distinct generic-object setter merely
because both occupy virtual slot `+0x68`.

This proves prototype attachment to the existing display receiver rather than
creation of a replacement generic object. Full null/non-object/error behavior
of the property-conversion path remains outside the supported object-prototype
profile and must not be filled in from a generic JavaScript `new` model.

## Recovered initializer eligibility and once-only storage

**OBSERVED (follow-up REA):** `0x14017baf0..0x14017bbb3` implements a
per-instance, per-frame byte ledger. Its receiver is the concrete AVM owner's
secondary interface at owner `+0x30`; the ledger pointer is at interface
`+0x1b0`, hence full owner `+0x1e0`. It returns immediately if the selected frame
byte is nonzero. Otherwise it requests that frame's initializer pointer/count
from target `+0x110` through definition virtual `+0x60`.

If that getter succeeds and the count is nonzero, the helper invokes every
initializer tag's virtual `+0x08` in ascending pointer-vector order, then writes
1 to the frame byte. It performs no ordinary-action predicate filtering. A
failed getter or an empty list leaves the byte zero and remains eligible for a
later call. This ledger records dispatch of the initializer tags; DoInitAction
tag dispatch enqueues a priority-2 buffer, so the byte is set before that
buffer's later bytecode execution, not after successful completion of its code.

Existing complete REA constructor `0x140203560` installs the secondary table
`0x1425174b0`, whose `+0xd8` entry is the helper above. It allocates the byte
vector at owner `+0x1e0` using a value queried from definition virtual `+0x50`,
and explicitly zeros the resulting range. This establishes a fresh ledger on
that AVM instance. No SpriteId, constructor definition, caller CodeOrigin or
global movie-filename set is read by the once-only helper. It is a frame-list
ledger, not a separate entry for each DoInitAction tag.

The actual root construction chain is grounded in reused receipts: root setup
`0x140210180` places `[MovieDef+0x38]+0x18` into CreateInfo field zero;
factory `0x140178440` passes that field to target constructor `0x1400b6d40`,
which stores it at target `+0x110`. The independently recovered DataDef
constructor and MovieDef binding establish this as the root's DataDef route.
The root target's AVM owner is constructed by `0x140203560`. Other target
subclasses must not inherit these concrete offsets without their own binding.

The definition getter `0x140087410` follows DataDef `+0x40` to complete helper
`0x1400868e0..0x140086966`. Under its lock, the helper checks the requested
index against the initializer-array length at state `+0x1d8`; an unavailable
index returns false. Otherwise it copies the 16-byte pointer/count entry from
state `+0x1d0` and returns true. This explains the retry eligibility of unavailable
frames without assigning a complete streaming policy.

On the producer side, the previously identified DoInitAction loader appends its
tag pointer to temporary loader vector `+0x3f8`; complete appender
`0x14006f120` does not use its second argument. Ordinary tags use indexed vectors
beginning at `+0x3c8`. The named ShowFrame path calls `0x140088b40`, which first
calls transfer `0x140091830`. On the reviewed non-special branch, the transfer
copies the initializer vector into owned pointer storage, resets the temporary
vector and sends the pointer/count to `[loader+0x38]` virtual `+0x18`.
Concrete setter `0x140086710..0x1400867d4` writes the entry for the current
frame into the definition-owned initializer array. The publication wrapper also
links a frame record and increments the published-frame counter. Loader index 1
has a separate branch that returns before this initializer transfer; its complete
meaning is not assigned here.

## Frame-zero and requested-frame submission

**OBSERVED:** The target table `0x1425069d0` has slot `+0x240` bound through
`0x1400a0e00` to `0x1400996e0`. That helper adjusts to the target's AVM owner
and invokes primary slot `+0x48`, whose concrete implementation is
`0x1401cbec0..0x1401cc031`. It queues a kind-2 native event mask 1 at priority 4
when the matching static event-map entry exists, otherwise at priority 5. It
then submits the initializer list for frame zero, followed by
`0x1400b61d0`, which invokes the ordinary-frame helper for zero. This route
itself does not invoke the normal queue drain.

Complete ordinary-frame helper `0x1400b5f20..0x1400b5fe6` checks target state
and the requested frame against target virtual `+0x370`. It obtains the
ordinary tag vector through definition virtual `+0x58` and invokes every tag's
virtual `+0x10` with raw priority 4. The DoAction and DoInitAction tables share
`0x140178820..0x1401788cd` in that slot: it skips empty/terminating action
buffers and otherwise forwards the supplied priority. This differs from their
fixed-priority `+0x08` methods. The frame helper's final secondary slot `+0xd0`
is a no-op for the proven concrete AVM table.

Primary AVM slot `+0x10`, `0x140204c00`, returns the secondary interface at
owner `+0x30`. This binds the following target transitions to the same initializer
ledger, rather than merely matching similar virtual offsets:

| Concrete target entry | Initializer and ordinary-tag order |
| --- | --- |
| `+0x228`, `0x1400b6790..0x1400b6824` | Clears the stored frame state, submits frame-zero initializers, then ordinary frame zero; this is a reset-like route, not a proved increment operation |
| `+0x378`, `0x1400b6870..0x1400b68e9` | Returns if target byte `+0x140` bit 3 is already set; otherwise sets it, submits frame-zero initializers, ordinary frame zero, then secondary `+0xc8` |
| `+0x368`, `0x1400b8670..0x1400b8925` | Clamps the requested destination, handles reconstruction, submits eligible initializer lists, then the destination's ordinary tags |

The last entry requires target word `+0x6a` bit 11 set, bit 12 clear and signed
field `+0x2c` at least -1. An equal destination submits no tags. A backward
transition submits the destination's initializer list before its ordinary tags.
For a forward skip, intermediate initializer lists are submitted in ascending
frame order, followed by the destination initializer list and ordinary tags.
The same per-instance ledger suppresses repeat initializer submissions. The
state-reconstruction helpers and callers' public method names are not assigned
by this evidence; this is not a complete seek implementation.

Secondary `+0xc8` is complete `0x1401ccb40..0x1401ccbbf`. It enqueues kind 2,
native mask 1, at priority 5 on the target's root queue. REA separately binds
native mask 1 through `0x140179c70`, ordinal 1 and builtin index 100 to `onLoad`;
native mask `0x40000` maps through ordinal 19 and builtin index 110 to
`onConstruct`. These native event identities must not be blindly treated as
serialized SWF ClipEventFlags for every low bit.

## Concrete deferred level startup drain

**OBSERVED:** Complete level attachment `0x14005fdb0..0x14006007d` receives
the MovieImpl, level number and target. On the insertion path it sets MovieImpl
field `+0x4310` bit 8, inserts a 16-byte level-number/target entry in the
`+0x58`/`+0x60` array, and stores level-zero target at `+0x50`. The previously
proved root initializer calls this helper with its actual newly created target.

MovieImpl constructor `0x140065080` installs primary table `0x1424feb70`;
slot `+0xc0` is complete routine `0x140062cf0..0x140064fa4`. Its retained
receiver stays in R15 throughout the body. Within that routine, the selected
branch at `0x1400644ac..0x140064514`:

1. Requires field `+0x4310` bit 8 and a nonzero frame count from the level-zero
   target's virtual `+0x370`.
2. Clears bit 8 before processing the level list.
3. Visits the level array from the last stored entry backward, invoking each
   target's virtual `+0x378`.
4. Calls `[MovieImpl+0x18]` virtual `+0x38`, the proven AS2 root normal drain.

For the concrete root target above, this establishes a deferred startup route:
per-level frame-zero initializer submission, ordinary-tag submission and
priority-5 `onLoad` submission, followed by the root's priority drain. The
per-target bit-3 guard prevents repeating that particular first-frame helper.
Neither flag is a global DoInitAction-by-SpriteId ledger.

The receiver binding is independently closed: existing complete factory
`0x140178340` creates `0x140065080`, passes that exact result to the known AS2
root constructor, and returns it. The root constructor/binder stores the root
at MovieImpl `+0x18`. No similarly shaped wrapper is substituted for MovieImpl.

The same complete `+0xc0` routine contains additional root drains at
`0x140064768`, `0x1400647d6` and `0x1400648cf`, around unexpanded work and
float/timing state. Their presence disproves a single universal frame-end drain,
but their full ordinary-advance contract is not assigned here. The deferred
startup branch above does not establish when the game invokes `+0xc0`, every
streaming/paused transition, or every target subclass's dispatch.

## Remaining boundary and implementation consequence

**INFERRED engineering consequence:** Use one explicit runtime-root queue with
separate priority, record kind, insertion cursor and token concepts. The observed
normal drain can support the initializer/Construct/constructor/frame priority
relationship. Construction must retain the actual instance receiver and class
lookup provenance, and a missing-class callback must perform a fresh lookup.
These findings are sufficient to reject an immediate constructor shortcut.

Still unresolved: the complete top-level streaming/advance call graph,
every placement path's pre-queue phase ordering,
public meanings and all producers of the recovered token-drain branch flags,
event-vector insertion order, and mutation/unload behavior during event dispatch.
The independently documented Adobe initializer ordering in
[AVM1 execution](AVM1_VM.md) is a separate published contract; this native trace
does not replace its first-encounter rule with an assumed placement trigger.

No root VM, renderer, frontend source or Cargo file is changed by this increment.
No full native timeline order is claimed from selected queue consumers alone.

## Private evidence and provenance

The target is Blizz's private retail executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Target-free REA Rizin calls selected explicit function/data ranges, without
launching the client, debugger attachment, automatic whole-image analysis or
shared configuration changes. Canonical receipts and bounded derived readings
remain under `~/.local/share/marvel-heroes-forever-rust/evidence/rea-display-lifecycle/`.

| Evidence ID | Selected findings |
| --- | --- |
| `ev_c5ea9ede6d788104941050af6e3ef6eae341186468547a865af9d6acdee7b72d` | Record insertion and action-dispatch slots |
| `ev_d68111b3453d756458db0b3e5c41782de7710993a57d69fd26e97f2c897ace0b` | Both drains, event wrapper, function and native-callback dispatchers |
| `ev_2fcaf5f332af5e141d733698b8473be2d8028a605236ec86f760b02d01e06b0f` | Both iterators, named tag loaders, common event handler/vector execution |
| `ev_047746b8ba7cefef8fe9083beab76a4a4ae84b3b389ac83d96965dafe7190840` | Named tag slots/executors, priority forwarding, token frame runner and selected construction caller |
| `ev_f135766caf5aa8db946bf5589308cd0908b6b0e8b5a868c1331ee0bd95c22a30` | Display prototype setter/slot and bounded full-drain call sites |
| `ev_1cad08f6e0309bf426f0013a86127ddbdf635e8d279741e3ac1e4017f1a6fbcc` | Complete priority-1 prototype-preparation callback |

**VERIFIED:** All six calls completed with exit code zero, untruncated output
and unchanged pinned executable hash. `complete.bundle.json` is the final
six-record canonical bundle; `verification.json` checks those receipts and the
current target hash. The earlier `final.bundle.json` is an intentionally retained
five-record intermediate checkpoint. Automatic string-inventory warnings about
large image regions do not establish an exhaustive inventory; all claimed reads
are the explicitly selected ranges. No routine test suite was run for this
documentation-only change.
The independent REA ledger was exported, checkpointed and reset with no active
target retained.

Reuse from `rea-class-registry/final.bundle.json`:
`ev_48e57d0b466d5180f8f3f83037835dd3c251e04f165a6dfbe215ef2b0c83fe0e`
contains both complete construction producers;
`ev_6a3ef294e0f93be00362ab76b3f85adad17bf1242bc644ad9fa95b6009d76dc4`
contains the complete late lookup callback;
`ev_598fe04144afafa708e23ddbfcfe29fe4ca8a2aaaf87b83bdacdb7b24e8ea2f6`
contains the action-record executor and prototype-related helper. Those calls
were reused without rerunning their producers.

`rea-array/prototype-property.bundle.json`,
`ev_2ae57b7eadfd90dffc216f3a6bd56d5e5ff2ccc642fc8eeabb069f4088006256`,
independently confirms builtin pointer-table entry `0x142add548` names
`prototype` (builtin index 77, root offset `+0x3a0`). Its complete, untruncated,
zero-exit, unchanged-hash receipt was inspected and reused.

Source ledger: the owned executable is proprietary and studied only for
authorized compatibility/interoperability; private project receipts are derived
owner evidence. Installed REA skill/tool instructions have an uninspected local
licence and were used only to operate the tool. Existing project Adobe citations
were read as project context, with no new external implementation source read or
copied. All instruction listings, target bytes and generated private receipts
remain outside git and builds.

Follow-up canonical evidence resides in
`rea-display-lifecycle-followup/complete.bundle.json`:

| Evidence ID | Follow-up finding |
| --- | --- |
| `ev_39e32fa5a4dc99ddf4b5d863329de22a154860e67f5af2d4a38646678770a650` | Separate temporary initializer/ordinary tag appenders |
| `ev_84439e17253e9bed52c13ccc1a284a1d2d88f5f3ef3ee83065a94e36ef8d34ed` | Bounded cleanup and root-setup candidates; no frame-boundary claim from them |
| `ev_96d70256aa54c2ff170f6fe88d7797f3753cc775430af834f95df5cf3e1910df` | Named ShowFrame publication and initializer transfer |
| `ev_4c7df2c0814e75260a90540619b68c284912691da0bb3a52c6e9676021001eae` | Complete once-only helper and definition-state table |
| `ev_0c9022256088f33e3d5ad1917f7f433e3017c725c6e3c16d0fd5cee9c2e0709a` | Initializer array setter/getter, frame-zero caller, exact slots and native event names |
| `ev_2b47475c0ded442abb805d6ee6494a887163f000ee439531cdb25399221e728d` | Complete first-frame/requested-frame transitions, interface conversion and supplied-priority tag dispatch |
| `ev_0f5f5a57b16a8e9c9af612ed4f9b30e22c53830e17937046b9f3cf5cca948eac` | Complete construction caller, raw flag and filtered-drain eligibility |
| `ev_b641750924a6ec93fcf8b79c2301d748f816dfbb359a0e6c4b59df51260e5fac` | Complete level attachment and selected startup drain branch |
| `ev_c358997b1bc57f9a07ebd8dec1afa3f88ad0ca6c91b4b27267080a53b95311f9` | Complete MovieImpl slot `+0xc0` body and priority-5 onLoad producer |
| `ev_8615b05f212ecf00f42122f6478fa0948eeeb7020582b36f60f0f9a29df80670` | MovieImpl constructor table store and exact `+0xc0` binding |

**VERIFIED:** All ten follow-up REA calls completed with zero exit status,
untruncated output and unchanged pinned executable hash. The private
`verification.json` records these checks and a fresh independent target hash.
The final bundle was checkpointed and the independent investigation ledger
reset without retaining a target. No client execution or routine test suite was
run for this documentation-only follow-up.

Reused complete construction/binding receipts:
`rea-asflags-followup-version-proof/final.bundle.json`,
`ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d`
and `ev_63afb28a4e5a8587d3d55625096a552ed22a947f9c384cac9c1ace7f5dfa051c`;
`rea-class-registry/owner.bundle.json`,
`ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a`;
and the concrete DataDef/MovieDef chain indexed by
`rea-native-alternative/movie-version-proof-receipt.json`. No new construction
producer calls were needed for those reused links.

The MovieImpl factory link reuses
`rea-asflags-followup-version-proof/owner-receiver.bundle.json`,
`ev_602090f2f3ff4f8c6e714c749b345ef623941fc34b644b007a5ec6837b3f6b60`.
Ordinary candidate discovery is retained in the follow-up directory's
`init-candidate-summary.json`, `binding-frame-summary.json` and
`final-boundary-candidate-handoff.json`; substantive positive findings above
were then confirmed with the explicitly listed REA reads. Rejected cleanup,
list-only attachment and setup candidates are not promoted to frame-boundary
claims.
