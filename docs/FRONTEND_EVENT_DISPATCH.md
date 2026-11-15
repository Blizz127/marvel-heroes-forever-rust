# Native saved Construct and onLoad dispatch

**OBSERVED (REA, owned 1.52.0.1700 Win64 executable):** Event records deliver
to the existing display instance. Saved clip-action functions and a dynamically
resolved event member are separate calls. Saved functions run first; the
dynamic member is looked up afterward. Registered class constructors remain
separate action records. None of these call sites supplies a bytecode-success
result suitable for a successful-delivery stub.

This document specifies selected concrete AS2 display paths. It adds no runtime
hook, VM code or event execution. Pointer offsets identify recovered native
relationships, not a proposed Rust memory layout.

## Queued target, receiver and environment

Complete executor `0x14017cf80..0x14017d059` obtains the retained target from
record `+0x10`. It skips a null target or target byte `+0x6a` bit 4. For raw
record kind 2 it adjusts to `target + 4 * byte[target+0x6d]`, invokes primary
virtual `+0x08`, then calls the returned owner's virtual `+0x128` with the event
descriptor at record `+0x20`.

The concrete owner constructor `0x140203560` installs primary table
`0x142517670`, object-interface table `0x1425175c0` at owner `+0x08`, and the
secondary interface at `+0x30`. Primary `+0x08` returns this owner unchanged.
Its event slot `+0x128` is wrapper `0x1401aa850..0x1401aa93e`; its environment
slot `+0xf0` is exact leaf `0x140203720`, returning owner `+0x38`.
The constructor binds owner `+0x20` and Env `+0xe0` to the actual display target.

Wrapper `0x1401aa850` again checks target `+0x6a` bit 4. For native mask 1 it
clears target word `+0x6a` bit 5, then requires **target byte `+0x88` bit 3 or
target byte `+0x140` bit 7** before calling the common handler. Construct does
not take that mask-1-only gate. These raw gates must be represented from supported
state; a queued onLoad record alone does not prove it is delivered. The full
producer/meaning of these onLoad eligibility bits remains outside this profile.
Other event masks, including the wrapper's mask-4 postprocessing, are excluded.

Common handler `0x1401a9890..0x1401aa129` retains the display target and the
environment's target for the duration of its normal-return path. The invocation
receiver is **owner `+0x08`**, the display's AS2 object interface. It is not the
root/global object, the registered constructor or a replacement generic object.

## Event identity and saved descriptor profile

The native bit-to-ordinal helper `0x140179c70` and exact builtin-name table reads
establish:

| Native mask | Ordinal | Builtin index | Dynamic property |
| --- | ---: | ---: | --- |
| `1` | 1 | 100 | `onLoad` |
| `0x40000` | 19 | 110 | `onConstruct` |

The reviewed producers initialize the remaining saved-event fields to zero,
with DWORD `+0x10 = 0xff00`: byte `+0x10` is zero and signed byte `+0x11` is -1.
For these masks/fields the common handler passes **zero arguments** to the
dynamic callable in both reviewed context-mode branches. Do not apply that
zero-argument conclusion to mouse/key/controller event payloads.

These are native event descriptors. Their numbers and field positions must not
be indiscriminately substituted for serialized SWF ClipEventFlags.

## Saved clip-action installation and source identity

The complete construction producer `0x140203920` visits its supplied event
objects in order and calls complete installer `0x140200570..0x140200723`.
The installer takes the saved event object and the actual display target. It
rejects an absent source at event `+0x18`, source length below one at source
`+0x18`, or a zero first byte through source `+0x10`.

For admitted data it obtains the target's concrete AVM owner and Env, then builds:

| Object | Retained source relationship |
| --- | --- |
| Wrapper from `0x1401850a0..0x14018511b` | Wrapper `+0x10` retains the **original saved event's source object** |
| ScriptFunction from `0x1401e14c0..0x1401e1634` | Function `+0x78` retains that wrapper; `+0x98` is start 0, `+0x9c` is original source length |
| Stored 32-byte Value | Tag 8, primary ScriptFunction pointer, secondary/frame pointer zero and flags zero |

The function's raw mode byte `+0xca` is 4 when the **original aggregate event
mask** is exactly `1`, `4`, `0x200` or `0x40000`; it is 3 otherwise. This is not
the action-queue priority. Both modes skip constructor target capture and leave
function `+0x68`/`+0x70` zero. Mode selection precedes event-mask splitting: a
combined mask containing Construct/onLoad is not automatically mode 4.

Complete splitter `0x1401fc3f0..0x1401fc4e0` ignores mask zero, passes a
single-bit key directly to `0x1401fc260`, and visits a multi-bit mask's set bits
from low to high, copying the key and replacing only its mask. The same callable
Value is installed for each resulting key. Complete appender
`0x1401fc260..0x1401fc3e7` finds or creates the owner `+0x28` map entry and
appends the Value. It has no replacement/deduplication branch. This establishes
per-key insertion order for this supplied-event-list route; it does not by
itself prove every loader's serialization-to-list ordering.

**INFERRED implementation consequence:** Retain the incoming saved action's
resource/span provenance as its source identity. Do not relabel it as the
registered constructor's body or the receiving instance's current frame action.
The native pointer chain above is proven; a complete source-file/CodeOrigin
mapping through every loader/import route is not recovered here.

## Saved handlers run before dynamic member lookup

Common handler first checks the static map through `0x140186fe0`. For this saved
descriptor profile, it then calls `0x1401a9740..0x1401a9886` before any dynamic
property lookup. Complete lookup `0x140181e30..0x140181eb3` hashes/compares the
mask; only masks containing bit `0x20000` additionally use key DWORD `+0x08`.
Construct/onLoad keys therefore use their mask identity, not a 20-byte memcmp.

Each map slot occupies 64 bytes; its inline pointer/count vector contains
32-byte Values. The dispatcher snapshots the count, visits indices zero upward
and reloads the vector data pointer each iteration. It does **not** copy a frozen
handler list. Reentrant map/vector mutation and invalidation are unsupported by
this profile.

For each stored Value the call to `0x14019aed0..0x14019b0c9` supplies:

- result destination: null;
- receiver: owner `+0x08`;
- caller environment: the owner's Env;
- argument count: zero;
- additional method-name pointer: null.

That helper retains a tag-8 Value's primary function and secondary/frame
reference, constructs the native call frame, and invokes function virtual
`+0x50`. A raw tag-11 conversion path also exists; its complete conversion policy
is not part of this supported tag-8 profile.

The static dispatcher ignores every call-helper Boolean. It reports true for
any found vector, including an empty vector or noncallable entries. The common
handler separately records static-map presence. Neither is proof that a handler
body completed, or even that a callable was invoked.

## Dynamic property resolution and invocation

After saved handlers return, the common handler obtains the current root's
interned `onLoad`/`onConstruct` key and invokes **the display object's actual
raw-member slot `+0x58`**. The concrete interface entry
`0x140203840..0x14020385f` adjusts the object-interface receiver back to its AVM
owner and calls `0x1401ddcd0` with no direct Env argument, the `Env+0xe8` string
context, key and output Value. This is not the generic object's getter ABI.

The complete display getter has special-member branches before ordinary data
lookup. Its ordinary fallback route probes the auxiliary object at owner
`+0x1d8`, temporarily clears/restores that object's prototype at `+0x30` during
the own lookup, then checks named display children, then the display prototype
at owner `+0x18`. Version/context-sensitive lookup remains in those actual
interfaces. A plain own-property/prototype shortcut would omit real precedence;
the special-member classification and all child-name edge cases are not reduced
to a new generic lookup rule here.

Because this lookup occurs after static handlers, their member/prototype changes
can affect the dynamic handler selected for this delivery. An own value found
by the getter is not automatically replaced with an inherited callable merely
because that own value is noncallable.

If the result is native tag 9, complete helper `0x14018dec0..0x14018e013`
invokes the property getter with the same display interface as receiver and the
caller Env, then the common handler uses its returned Value. Accessor failure
and missing-getter behavior have additional diagnostic/undefined-value paths;
they must not become a successful callable-delivery stub.

For an ordinary tag-8 function result, the common handler calls `0x14019aed0`
with the display interface, caller Env and zero arguments for the saved profile.
One context flag branch also supplies the event-name string as the additional
method-name hint; the other passes null. The public meaning of that context flag
and every use of the hint are unassigned. The callable's secondary/frame
reference is preserved independently of the receiver.

## Callable environment differs from receiver and source

The source-backed ScriptFunction table `0x142513140` binds virtual `+0x48` to
complete environment selector `0x14017f710..0x14017f7d2` and `+0x50` to complete
invoker `0x14020e190..0x14020e307`.

The selector attempts the function's retained MovieImpl/target-handle route in
fields `+0x68`/`+0x70`; if it resolves an eligible target, it uses that target's
AVM Env. Otherwise it returns call-frame `+0x30`, the supplied caller Env.
The static saved-handler modes above leave those capture fields zero, so they
select the event receiver's supplied Env. Other script functions can select a
captured Env while still receiving the actual display interface as `this`.
Those identities must remain separate in Rust.

The invoker additionally checks the selected Env's target byte `+0x6a` bit 4;
when set, it falls back to the supplied call-frame Env. Retained function source
and the actual `this` receiver are not replaced by this environment fallback.

The invoker passes retained source wrapper, selected Env, stored start/length,
scope data and raw mode to the bytecode entry. It does not substitute constructor
or current-frame code for the retained source. It also has an Env `+0x17c` u16
depth guard: an old depth at least 255 skips the body after restoring the count.
The call helper still returns true when the nonnull function's virtual call
returns normally. Thus its Boolean is not a bytecode-success result or a proof
that this recursion guard admitted execution.

## Registered constructor ordering and retirement

On the reviewed registered hit path, priority-1 preparation attaches the
prototype before pending priority-2 initializer actions in the normal drain.
Priority-3 Construct kind 2 precedes the retained constructor kind 3. Construct
therefore delivers the saved Construct vector and then its dynamic member before
the later registered constructor record, subject to target-state eligibility
and reentrant queue insertion. An optional additional native callback may
intervene on the already documented version/argument branch. Token-filtered
drains must still preserve their separate scope.

An initial registered hit retains its constructor function/frame reference;
changing the class registry during Construct does not alone replace that
reference. The missing-class callback instead performs a fresh lookup when it
executes, then invokes local Construct and constructor records directly. It does
not enqueue those two local records again. The event's dynamic member lookup
remains separate from either constructor lookup.

The queue executor rechecks the target state for each record. Changes during a
Construct handler may therefore make a later constructor record ineligible.
The static vector loop does not add a fresh executor check between its entries;
this document makes no complete unload/mutation/exception-order claim.

On normal returns, queue iteration detaches the next record before retiring the
previous one. Nested drains retain their own pending records. Complete recycler
`0x1401c7ee0..0x1401c7fd3` clears kind, releases retained buffer/target and
function/frame references, clears the argument vector, then pools or frees the
record. Event-handler temporary values and target references are released by
the common handler before its normal return. The drain does not inspect the
event helper's Boolean before continuing.

No reviewed wrapper catches a script failure and turns it into an event-success
result. Interpreter exception flags, native unwinding, VM interruption and
recovery require separate evidence. A bounded Rust executor must surface its
own unsupported/error outcome and preserve ownership; it must not report
retail delivery merely because a record was visited or a helper returned true.

## Implementation boundary and private evidence

The grounded initial profile is: the concrete display type above; admitted
saved Construct/onLoad descriptors; valid retained source/callable references;
static tag-8 functions in stored order; known dynamic display-member lookup;
and normal-return calls with the exact receiver, caller Env and function source
kept separate. Unknown target subclasses, unbound onLoad gates, tag-11 conversion,
unsupported accessors/special members, reentrant static-map mutation and script
error recovery must remain explicit unsupported paths. No guessed event firing,
success stubs or immediate constructor invocation are justified.

The proprietary owner executable has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It was studied only for authorized compatibility. New target-free REA reads are
retained in `evidence/rea-event-dispatch/complete.bundle.json`:

| Evidence ID | Coverage |
| --- | --- |
| `ev_6dfab21ba5f9d8c474ad1c640ca8ade9679527eb868f4bb9b52c0fcd4fe3e63d` | Invocation helper, static-map predicate/lookup, accessor helper, recycler, exact display environment/getter/event slots |
| `ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54` | Saved-handler producer, source/callable constructors, mask splitter/appender, display getter, ScriptFunction environment/invocation slots |

Reused canonical receipts:

- `rea-display-lifecycle/complete.bundle.json`: event wrapper/common handler,
  static vector dispatcher, queue iterators and drains (IDs `ev_d68111b3453d756458db0b3e5c41782de7710993a57d69fd26e97f2c897ace0b`
  and `ev_2fcaf5f332af5e141d733698b8473be2d8028a605236ec86f760b02d01e06b0f`).
- `rea-class-registry/final.bundle.json`: executor, both class-construction
  producers, late lookup and exact owner-return leaf; IDs
  `ev_598fe04144afafa708e23ddbfcfe29fe4ca8a2aaaf87b83bdacdb7b24e8ea2f6`,
  `ev_48e57d0b466d5180f8f3f83037835dd3c251e04f165a6dfbe215ef2b0c83fe0e`,
  `ev_6a3ef294e0f93be00362ab76b3f85adad17bf1242bc644ad9fa95b6009d76dc4`,
  `ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a`.
- `rea-display-lifecycle-followup/complete.bundle.json`,
  `ev_0c9022256088f33e3d5ad1917f7f433e3017c725c6e3c16d0fd5cee9c2e0709a`:
  exact native event-name mapping.
- `rea-asflags-followup-version-proof/environment-owner.bundle.json`,
  `ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d`:
  complete concrete AVM constructor.

Candidate discovery and static-source analysis are retained in
`rea-static-event-dispatch/static-event-handoff.json`; claimed new positive
paths were confirmed with the two listed REA records. No external implementation
source was copied, no AVM1 source or existing document was edited, and all binary
bytes/disassembly remain private. `verification.json` records completion,
zero exit, untruncated output and unchanged source hash checks. No native client
execution, application smoke, full suite or retail parity comparison was run.
The two-record investigation was exported, checkpointed and reset with no
active target retained.
