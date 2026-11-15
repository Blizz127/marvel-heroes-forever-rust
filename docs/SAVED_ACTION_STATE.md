# Persistent saved-action state

`src/avm1/saved_state.rs` adds original state containers for the contracts in
[saved event execution](SAVED_EVENT_EXECUTION.md). They do not install or run
handlers, allocate native activation frames, or change the current diagnostic
executor. Central VM integration remains required before any saved-event
execution claim.

Each installed action has its own `SavedActionState`, with immutable original
source identity, aggregate event mask and derived mode. `SavedActionId` is an
arena handle: all keys split from one installation must keep the same handle.
Two installations of identical source must retain separate states. The helper
does not allocate IDs or infer installation identity from a `CodeId`.

The source span maps file-absolute opcode origins to source-wrapper-relative
offsets. The first **executed** ConstantPool record locks its offset and cached
strings, including an empty pool. Revisiting that offset reuses the pool;
visiting another offset ignores it without calling the decoding closure. Pool
indices outside the retained count return Undefined. Source identity and span
checks precede every visit, including revisits. Opcode identity, immutable bytes,
string interning and total allocation budgets remain the VM's responsibility.

The cache models the supported successful native allocation path. A Rust decode
or budget error leaves this container unchanged and must terminate delivery.
It does not reproduce native failure state or authorize resuming a failed
delivery. The bounded installer rejects zero masks and invalid/empty source
spans; it does not claim those errors are native exceptions.

`SavedEnvironmentState` adopts an explicit owner, target, operand vector and
four-register bank. It never invents an empty stack or Undefined bank at each
call. Bounded pushes preserve existing operands on failure; individual opcodes
choose their own underflow/sentinel handling. Saved-mode register reads above
three return Undefined and writes above three are ignored, as established by
the separate native mode-3/4 branches. This rule does not apply to ordinary
Function2 registers.

Neither persistent state supports `Default` or `Clone`. Their identities are
read-only. The VM must separately validate live owner/target coherence, current
context, Value identities, raw target gates, invocation depth, unchanged-target
admission and the null activation fence. It must also preserve state on the
supported normal-return path and keep unsupported/error outcomes distinct from
completed event delivery. Nothing here creates an automatic default Environment
or bypasses those execution requirements.

## Provenance and validation scope

The implementation uses only original project types and the owned-executable
REA contracts registered in [saved event execution](SAVED_EVENT_EXECUTION.md).
There is no new external dependency or reference implementation. Native bytes,
action data and evidence remain under the private project root.

Focused module checks exercise sharing across split event keys, independence
across installations, first-executed pool selection, skipped decoding,
provenance rejection, failed-decode publication and persistent supplied
Environment state across two synthetic handler steps. These are state checks,
not original bytecode execution or native event comparison. The final private
receipt records the exact module hash and terminal check result.

**VERIFIED, Rust state checks:** all three focused tests pass with exit 0 and
no compiler warnings. The final module SHA-256 is
`d4a1de20b8658f21333a531369dbe84661925f1b2ea2acdfbbaddfc34cd275de`.
The private harness at `evidence/saved-action-state-module/` compiles this exact
module against the root's VM types without adding a central executor hook.
`final-receipt.json` records command, terminal result, output hashes and unchanged
module source; `final-module.rs` retains the checked source. The earlier
Text-pool draft and its passing intermediate check remain separately recorded.
The final representation uses the current Frame's `Arc<Vec<Value>>`, containing
only String values decoded from admitted pool entries.
