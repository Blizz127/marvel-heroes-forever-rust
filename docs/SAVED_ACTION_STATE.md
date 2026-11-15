# Persistent saved-action state

`src/avm1/saved_state.rs` provides original state containers for the contracts in
[saved event execution](SAVED_EVENT_EXECUTION.md). `saved_runtime.rs` now retains
them in VM arenas and lends their actual stack, registers and pool to a dedicated
saved Frame. The diagnostic executor remains separate. This bounded execution
entry does not establish native event selection, queue order or lifecycle.

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

## Bounded VM entry

`Vm::install_saved_action(owner, source, original_mask)` allocates a distinct
`SavedActionId`; the source must match the registered Program's entire span.
The installer admits the Construct/Load/combined masks `0x40000`, `1` and
`0x40001`, and rejects empty sources or an initial End byte. The full retained
span includes trailing bytes: there is no terminator trimming or synthesis.
[Native clip-action loading](SAVED_CLIP_ACTION_LOADING.md) documents those
source and per-record mask facts. Callers still establish which actual native
installation belongs to the supplied owner; matching source coordinates alone
does not prove that association.

`register_saved_environment(state)` adopts one explicit Environment per typed
owner. Source movie and owner-selected Environment remain independent facts.
Known loaded versions 7..19, live eligible case-3 owner/target identity, one
adopted root, supplied Value identities and heap/stack budgets are checked.
`saved_action_info`, `saved_environment_state` and `saved_environment_halted`
provide read-only inspection. Saved state is never cloned for replay.

`execute_saved_action(action, environment, host)` initially executes only pure
Construct/Load mode4. Combined-mask mode3 is retained but explicitly rejected
at entry. No mode3 target default or signed-depth field is invented. Mode4 bit4
returns `SavedExecution::TargetBit4Skipped` with zero body work. Otherwise the
Frame receives the persistent operand vector and four registers, target/global
lookup with a null activation fence, typed target `this`, and the retained
wrapper pool. There is no new activation, arguments object or Super object.

The first executed ConstantPool locks its wrapper offset. Later visits to either
the same or another offset do not decode/charge strings again. Invalid saved
constant indices and register indices above three push Undefined; out-of-bank
StoreRegister does nothing, even before looking for an operand. DefineLocal uses
the target's normal member-write route, and DefineLocal2 converts/pops the name
without declaring a property. Ordinary frames keep their existing semantics.

Function definitions, With, target changes, SetVariable and ordinary script calls
remain explicit first-profile boundaries. The latter require a separate shared
Environment stack/register proof; the current ordinary executor owns private
vectors. Once an Environment is adopted, generic diagnostic `execute` and
ordinary script invocation are therefore rejected, avoiding an untracked run
followed by reuse of stale persistent state. Existing bounded native callbacks
remain explicit host/intrinsic paths. Reentry is rejected before borrowing arena
slots, and no callback/table mutation is interpreted as native event delivery.

Admission failures leave the Environment usable. Once body execution begins,
every returned error restores partial stack/register/pool state to the arenas,
then halts that Environment. It cannot be retried implicitly. A successful body
returns `Completed { instructions, host_calls }`, discards the saved call's result
destination, and retains residual operands. Both paths restore the caller's
origin, active context and depth. This halt policy and engineering budget
failures are explicit Rust behavior, not claims about native exception recovery.

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

**OBSERVED historical isolated Rust state checks:** all three focused tests pass with exit 0 and
no compiler warnings. The final module SHA-256 is
`d4a1de20b8658f21333a531369dbe84661925f1b2ea2acdfbbaddfc34cd275de`.
The private harness at `evidence/saved-action-state-module/` compiles this exact
module against the root's VM types without adding a central executor hook.
`final-receipt.json` records command, terminal result, output hashes and unchanged
module source; `final-module.rs` retains the checked source. The earlier
Text-pool draft and its passing intermediate check remain separately recorded.
The final representation uses the current Frame's `Arc<Vec<Value>>`, containing
only String values decoded from admitted pool entries.

**OBSERVED integrated checks:** four new runtime checks retain supplied nonempty
state across repeated calls and independent wrappers, restore partial mutations
before halting on error, keep preflight failures usable, and distinguish a bit4
skip from completion. Together with the three state checks and four focused
ordinary-frame regression checks, eleven pass. Exact source and binary hashes
are retained in `evidence/avm1-runtime/saved-runtime-first-checks.json`, SHA256
`5e70bf086f7397109f605a31895964436baa53988d093fd286064a15cf51260e`.
This is synthetic runtime verification; no original saved handler or native
event comparison was run for this increment.

Five additional opcode checks pass on frozen `7f233ca`: DefineLocal's target
normal-write route and global isolation, DefineLocal2 conversion without
declaration, persistent registers and out-of-bank semantics, branch-selected
first pool with ignored later undecodable strings, and partial state retained on
the supplied stack ceiling. These use an explicitly synthetic authenticated-
profile fixture; they do not assert any native initial state. Private
`evidence/saved-opcode-checks/retry-7f233ca/receipt.json`, SHA256
`bf406110c9ed08758c135d45cd4a11df066bbdf642983272d66dce086323eace`,
records the compile/run success and frozen hashes. The earlier `890e681` test
assertion compilation failure is preserved separately; no tests ran in that
attempt. The clone-compatible assertion fix changed no VM runtime behavior.
