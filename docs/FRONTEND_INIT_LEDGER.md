# Per-instance initializer submission ledger

`src/frontend/init_ledger.rs` implements the bounded submission bookkeeping
recovered from native `0x14017baf0..0x14017bbb3`. It owns one entry per frame of
one concrete AVM display instance. It has no global SpriteId, linkage-name,
filename, constructor-definition or CodeOrigin key.

This increment is not connected to the runtime. It neither creates loader
triggers nor executes bytecode. [Native lifecycle evidence](REA_DISPLAY_LIFECYCLE.md)
separately records first-frame/transition callers and the unresolved special
loader-index-1 branch; this ledger does not fill those gaps.

## Recovered native rule

**OBSERVED (existing canonical REA):** The secondary AVM interface at full
owner `+0x30` reads its ledger pointer at interface `+0x1b0`, or full owner
`+0x1e0`. Frame index is a u32. A nonzero frame byte returns before definition
lookup. Otherwise the routine requests the initializer pointer/count through
target `+0x110`, definition virtual `+0x60`.

| Lookup/submission result | Native ledger effect |
| --- | --- |
| Getter returns false | Leave byte zero; a later call remains eligible |
| Getter succeeds with count zero | Leave byte zero; a later call remains eligible |
| Getter succeeds with a nonempty list | Invoke each tag's virtual `+0x08` in stored pointer order; write byte 1 after all calls return |
| Byte was already nonzero | Skip lookup and all tag calls |

The native loop reads a DWORD count, does not apply the ordinary-action tag
predicate, does not inspect tag return values and does not call the queue drain.
Known DoInitAction tag methods submit priority-2 actions. Therefore the mark
records tag dispatch, not successful execution of the submitted bytecode.
The mark precedes later execution when the supplied dispatch hooks only enqueue,
as the recovered DoInitAction path does.

A nonempty list still marks the frame when its tag methods legitimately perform
no action—for example, an empty action buffer can be skipped by a tag method.
Counting newly enqueued buffers instead of accepted tag dispatches would change
the recovered rule. An available empty list remains a distinct state.

The complete AVM constructor `0x140203560` allocates and zeros a fresh ledger
using the definition's virtual `+0x50` capacity. Two instances sharing a
definition have separate ledgers. The Rust capacity is fixed at construction;
dynamic resizing or transferring completion from another instance is unsupported.

## API

`InitLedger::new(capacity, Limits { frames, tags_per_frame })` checks the explicit
frame bound before reserving storage. All entries begin `Eligible`; allocator
failure is returned. There is no native pointer arithmetic or unchecked frame
access in the Rust interface.

`submit_with(frame, lookup, dispatch)` invokes a lazy read-only lookup returning
`Unavailable` or `Available(&[T])`. Lookup is skipped for submitted frames.
Dispatch receives a zero-based tag index and the borrowed tag. It must accept or
submit the tag without draining bytecode. No queue type, VM value or renderer
type is required by this module.

| Outcome | Meaning |
| --- | --- |
| `AlreadySubmitted { tag_count }` | Existing completed frame; lookup/dispatch were skipped |
| `Unavailable` | Getter-equivalent unavailable state; remains eligible |
| `Empty` | Available zero-tag list; remains eligible |
| `Submitted { tag_count }` | Every tag dispatch returned successfully; marked before returning to the caller |

`state(frame)` reports an entry or `None` for an out-of-range frame.
`quarantine()` exposes the instance-wide engineering failure gate described
below. Capacity, frame indices and admitted tag counts are u32; the configured
limits can be smaller. The module borrows the list and does not copy asset data.

## Bounded failure policy

**Engineering policy, not a recovered native Result branch:** Out-of-range
frames, lookup errors and tag-limit violations fail before any tag dispatch and
leave existing ledger state unchanged. Empty/unavailable lists remain retryable.

External tag dispatch can fail after it has already changed a queue. A failed
callback therefore quarantines this AVM instance, including failure on the first
tag. The record includes expected tag count, callbacks completed successfully,
callbacks attempted and the failing frame. Attempted includes the failed call;
its partial effects are unknown. No frame is marked `Submitted` by that failure,
later tags are not attempted, and subsequent calls on this instance fail before
lookup, even for a different frame.

An interruption marker is installed immediately before each callback. If Rust
dispatch unwinds, the instance remains quarantined with an inspectable
`InterruptedDispatch` reason. A normal callback error records `DispatchFailed`
and returns its error value. Successful completion writes the submitted mark
and clears the temporary marker. Read-only lookup must not submit actions;
lookup errors/unwinding occur before this dispatch quarantine begins.

This does **not** roll back records already queued by earlier callbacks or by
the failing callback. The runtime must inspect the returned error/quarantine and
stop execution of partial initialization while it decides recovery. There is no
reset or retry-partial API: clearing only this ledger could duplicate earlier
effects. Instance discard/recreation, queue rollback and recovery triggers need
their own supported runtime route. The ledger does not drain, cancel or erase
pending queue records on the caller's behalf.

Instance-wide quarantine, bounds and panic retention are explicit safe Rust
policies. The native helper has no corresponding checked callback error path.
No exception, interruption or reentrant-execution parity is claimed.

## Focused validation and provenance

Eight standalone Rust tests cover unavailable/empty retry, lazy lookup skipping,
stored tag order, nonempty no-op dispatch, marking before a separate execution
stand-in, independent instances/frames, preflight failure atomicity, partial
effects with quarantine, interrupted dispatch and capacity checks. No game
startup, native execution, rendered comparison or full test suite was run.

Independent hand-derived native traces and their explicit engineering-failure
extension are retained privately in
`evidence/rea-display-lifecycle-followup/init-ledger-hand-traces.json`.
`init-ledger-verification.json` records the focused compilation/test run and
source hash. These tests establish the bounded helper contract, not a complete
client lifecycle.

Source is the owner's proprietary build 1.52.0.1700 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
studied only for authorized compatibility. Reused canonical evidence:

- `rea-display-lifecycle-followup/complete.bundle.json`,
  `ev_4c7df2c0814e75260a90540619b68c284912691da0bb3a52c6e9676021001eae`:
  complete initializer ledger helper.
- `rea-asflags-followup-version-proof/environment-owner.bundle.json`,
  `ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d`:
  complete AVM constructor, fresh allocation/zeroing and instance ownership.

The earlier lifecycle verification checks complete, zero-exit, untruncated REA
receipts and the unchanged target. This increment reuses those receipts without
new target analysis. All implementation is original and uses only Rust's
standard library; no third-party implementation or extracted asset bytes enter
the repository. Root owns the module hook, runtime integration and central
provenance update.
