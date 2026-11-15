# Bounded native action-queue semantics

`src/frontend/action_queue.rs` is original Rust scheduling code derived from the
selected native operations documented in [display lifecycle](REA_DISPLAY_LIFECYCLE.md).
It is not connected to the frontend runtime in this increment. It supplies no
executor, constructor shortcut, initializer ledger, clock or frame trigger.

**OBSERVED:** The owned build has six numeric priorities, separate record kinds,
an insertion cursor for each bucket, u32 tokens and an insertion-counter restart
rule. **VERIFIED:** Focused synthetic traces exercise this implementation of
those operations. They do not establish native startup or rendered parity.

## API and ownership

| Type or operation | Contract |
| --- | --- |
| `ActionQueue<T>::new(Limits)` | One queue for one runtime root; caller explicitly supplies pending-record and nested-drain bounds |
| `Priority::from_raw` | Admits exactly 0 through 5 |
| `RecordKind` | Separate ActionBuffer=1, Event=2, Function=3 and NativeCallback=4; unsupported raw kinds are rejected |
| `enqueue(priority, kind, payload)` | Inserts at the current cursor and stamps the current token; failure returns the payload unchanged |
| `allocate_token()` | Increments the token counter with native u32 wrapping, without changing the installed token |
| `swap_token(token)` | Installs a token and returns the old one; the producer must restore it explicitly |
| `begin_drain(All or Token)` | Creates an independent scan, without clearing queue contents, cursors or the current token |
| `next(handle)` | Detaches one eligible owned record; the caller executes and retains/releases its payload |
| `end_drain(handle)` | Ends only the current scan; pending records and token state remain untouched |

Handles belong to a particular queue. Only the innermost active handle may
advance or end; an outer drain resumes after its nested drain ends. This permits
reentrant VM work without holding a Rust borrow of the queue across execution.
Queue identity and handle serials are checked u64 engineering identifiers;
they are not native tokens and do not wrap. Exhaustion fails explicitly.

The queue caps pending records and active drains, but not the size of opaque
payloads or the total actions produced over time. It performs no unbounded
automatic drain loop. The caller must apply its execution budget and preserve
an interrupted drain handle if it intends to resume it. No native cancellation,
exception or starvation policy is inferred from these bounds.

## Exact insertion and removal

Each bucket stores ordered records and a cursor index. Head and tail are the
first and last elements; the representation does not replace cursor insertion
with FIFO append.

| Operation | Head, cursor and tail behavior |
| --- | --- |
| Insert with null cursor | Prepend to head; cursor becomes the inserted record; old tail remains unless the bucket was empty |
| Insert with nonnull cursor | Splice immediately after cursor; cursor becomes the new record; tail changes only when insertion is last |
| Normal removal | Remove the head; if it was the cursor, cursor follows its successor; clear cursor and tail when empty |
| Filtered removal | Remove the first matching token in stored order, leaving skipped records in place; if it was the cursor, cursor follows the successor, or falls back to the predecessor when there is no successor |
| Filtered tail removal | Tail becomes the predecessor, including null when the bucket becomes empty |
| Filtered scan without a match | Does not move the bucket cursor |

A drain begins at priority zero with observed insertion counter zero. It
continues at the current priority after removal. A counter mismatch resets its
priority to zero and updates its snapshot. Only successful insertion increments
the queue counter; removal and drain creation do not. Thus lower-numbered work
inserted while an action executes can run before remaining higher-numbered work.
Token filtering is equality on the stamp, not a priority threshold.

The complete native root constructor `0x1401d4490` initializes insertion counter,
token counter and installed token to **1**. The Rust queue does the same. Both
native counters are u32: allocation uses a 32-bit increment and insertion uses
a DWORD increment. Rust uses `wrapping_add(1)`; zero is valid after wrap, and
token equality may alias after a full wrap. It does not invent reserved-zero,
widened-token or exhaustion semantics. Handles use a separate checked policy.

Only the reviewed insertion/removal operations are exposed. Other native
writers that reposition insertion cursors remain untraced. Tests seed explicit
nonnull-middle and null-on-nonempty cursor states to check those operations;
these seeds are not a runtime placement rule or a public cursor-reset API.

## Record lifetime and deliberate boundaries

Native iteration detaches the next record, retires the previously returned
record, then returns the selected record. Nested drains retain their own
previous records. The Rust queue returns an owned `Record<T>`; it does not
pretend that dropping a native refcounted action is equivalent to clearing a
generic Rust value. A future executor can preserve the recovered sequence by
holding the previous record until *after* `next` detaches its successor, then
dropping the previous record before executing the selected one. That executor
must define payload cleanup and error behavior separately.

The native raw record pool, destructor paths and pool reuse are not reproduced.
Public record kinds stay separate from priority and opaque payload; selecting a
kind does not invoke its native meaning. No class registry, CodeOrigin, object
prototype, display hierarchy or initializer state is changed here.

After terminal `None`, Rust safely returns `None` on another unchanged scan;
insertion can restart it through the counter rule. The native normal iterator's
terminal epilogue addresses beyond the six bucket entries. This implementation
does not emulate that memory aliasing or infer a seventh priority. Normal callers
end their scan after `None`; repeated-terminal safety is an explicit Rust boundary.

## Validation and provenance

Focused checks compile the standalone module as a library and as a Rust test
binary, then run nine tests. They cover cursor successor/predecessor behavior,
head/tail removal, prepend, nested token drains, reentrant lower-priority work,
fresh-root values, u32 wrap, stale/cross-queue handles and failure atomicity for
capacity/depth/identity rejection. A separate nested hand trace confirms the
caller can retain and release records in the recovered order. Allocator exhaustion was not forced. No
application startup, native execution, full suite or gameplay comparison ran.

Independent hand-derived traces were prepared from the native operations without
reading the Rust implementation. Private receipts:

- `evidence/rea-display-lifecycle/complete.bundle.json`: inserter
  `ev_c5ea9ede6d788104941050af6e3ef6eae341186468547a865af9d6acdee7b72d`,
  iterators `ev_2fcaf5f332af5e141d733698b8473be2d8028a605236ec86f760b02d01e06b0f`,
  drains `ev_d68111b3453d756458db0b3e5c41782de7710993a57d69fd26e97f2c897ace0b`.
- `evidence/rea-native-alternative/extends-name-binding-final.bundle.json`:
  complete root constructor
  `ev_5de603be82d92b5907b35369248200732fda049b1d4c60fe5880ced8d9b92b76`.
- `evidence/rea-display-lifecycle-followup/queue-semantic-receipt.json` and
  `queue-hand-traces.json`: exact semantic ledger, receipt integrity and original
  hand-derived traces. `action-queue-verification.json` records the focused run.
  Native evidence remains private, outside git/builds.

The target is Blizz's proprietary build 1.52.0.1700 executable, studied for
authorized compatibility; no implementation source or third-party code was
copied. This module uses the Rust standard library only. Root owns the module
hook, runtime integration and central provenance update.
