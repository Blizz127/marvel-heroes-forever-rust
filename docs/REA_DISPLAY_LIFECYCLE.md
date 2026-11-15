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
token, allocates a new token, stamps its generated records, restores the old
token, and returns the new token. The selected caller window
`0x14020dcf4..0x14020ddc4` retains that return value and, on a zero-valued branch
flag, passes it to the filtered drain. The flag's source meaning and all other
caller branches are not recovered by that window. This is concrete evidence
that construction records can be drained separately from other queued records;
it does not establish a single global initialization/placement/frame sequence.

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

## Remaining boundary and implementation consequence

**INFERRED engineering consequence:** Use one explicit runtime-root queue with
separate priority, record kind, insertion cursor and token concepts. The observed
normal drain can support the initializer/Construct/constructor/frame priority
relationship. Construction must retain the actual instance receiver and class
lookup provenance, and a missing-class callback must perform a fresh lookup.
These findings are sufficient to reject an immediate constructor shortcut.

Still unresolved: the complete top-level streaming/advance call graph, the
once-only initializer ledger, every placement path's pre-queue phase ordering,
all token-drain branch flags,
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
