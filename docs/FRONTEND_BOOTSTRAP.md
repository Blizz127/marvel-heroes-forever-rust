# Bounded frontend initializer phase

`frontend::bootstrap::InitSession` retains one VM and stable movie/action identities
while compiling saved root-frame programs. `compile_frame` separates DoInitAction
records from ordinary frame actions. `initialize` runs only the initializers, in
their original serialized order, once per compiled movie/frame in that session.
It does not wait for a sprite placement to run its initializer.

The ordering follows Adobe's SWF v19 DoInitAction specification (printed page108,
PDF page110), already registered in [AVM1 execution](AVM1_VM.md): on the first
encounter with the frame, these initializers precede ordinary frame actions and
implicit ActionScript sprite creation. Duplicate initializer IDs and references
to non-sprite definitions are rejected. AVM2 movies are outside this profile.
Streaming, unload/reload, imported movie ordering, display-object construction and
ordinary timeline execution remain unimplemented.

Plans retain their exact source identity, action ordering, offsets and program IDs.
A modified plan cannot drop or reorder initializers. A successful repeated call
reports `AlreadyInitialized`. Failure retains partial state for diagnosis and
halts the session; execution and registration cannot resume on it. A new session
is required. This applies to VM errors and the cumulative phase budget.
The VM also enforces each action's instruction budget; the cumulative check runs
after an action and can exceed its threshold by at most one bounded action.

`into_initialized` consumes a session only when execution has begun, every
compiled frame has completed its initializer phase and the session is not halted.
It returns `InitializedFrames { vm, frames }`, retaining the same heap, program
identities and compiled plans for an explicit caller-owned continuation. It does
not run ordinary actions, create display objects or establish native queue order.
Pending and failed phases cannot be released through this API. This handoff is
an original bounded-state policy, not a recovered native ABI.

`setup_vm` permits explicit intrinsic/host bindings before execution starts.
`install_movie_clip_constructor` installs Object and the bounded typed unbound
MovieClip intrinsic. It preserves the established native category and constructor
callback profile, including the evidenced subclass allocator and prototype mouse
event-name marker. Other native/display member behavior remains gated. It does
not create or attach a rendered display object. The older
`install_movie_clip_metadata` API remains an explicitly non-executing inheritance
study binding whose constructor calls reach the supplied Host; the CLI now uses
the typed intrinsic. Neither setup creates a complete movie player.

`mhf frontend init-probe <package.upk> <movie>` supplies the typed unbound binding,
the explicitly supported Object/Array/Function/property-flags intrinsics, and the bounded
[Selection listener host](FRONTEND_SELECTION_HOST.md) and
[AS2 root class registry](FRONTEND_CLASS_REGISTRY.md). `FrontendHost` combines
those stateful bindings and rejects other native operations. It compiles frame zero
and executes its initializer phase; unsupported host operations are rejected. It
reports a nonzero exit when an unsupported operation stops execution. Completion
of this isolated phase would still not establish movie startup or retail parity.

## Validation and provenance

Focused fixtures exercise rejecting altered plans, once-only execution without
placements, keeping ordinary Stop actions separate, and refusing retries after
both host rejection and cumulative budget exhaustion. Original-manager results
are retained privately under `evidence/frontend-bootstrap/`.

**OBSERVED:** The root CLI probe registered 452 manager initializers and retained
one ordinary frame action separately. With Object and explicit MovieClip class
metadata, it executed 218 instructions before the unbound ASSetPropFlags call at
source offset12374; zero initializers completed. Its nonzero exit and exact source
and binary hashes are in `cli-before-property-flags.json`. This is owned-script
execution, not native-client execution.

The current driver explicitly binds its validated loaded root/header version to
the VM. The native loaded-target path is now traced to that header byte; supported
versions7–19 therefore select the verified clear-zero property-flags branch.
Arbitrary/unbound objects retain the ambiguity gate. Function objects now inherit
through a shared Function-prototype object to Object.prototype. These changes
allow further original initializers to run; the probe still reports its exact
first failure and does not continue past partial failure.

**OBSERVED (integrated dev.7 CLI):** The typed MovieClip subclass path reaches
118 complete manager initializers and 18,572 instructions, retaining one Selection
listener. The next initializer begins at source757488; its call into UIObject
stops on Enumerate2 at source750939 (CLI CodeId114), at the then-unsupported native
hash-order enumeration boundary. Code IDs are session-local. Receipt:
`evidence/dev7-integration/initialization.json`. The native scheduling queue is a
separate bounded module and is not used by this isolated initializer driver.

**OBSERVED (integrated dev.8 CLI):** All 452 initializers complete in 49,244
instructions with six host calls, one Selection listener and five class
registrations. This is a minimal-global model with an initially parentless Plain
root, not the complete [native global baseline](AS2_GLOBAL_BASELINE.md) or an
attached display root. The separate original first frame action stops after 31
instructions at source855912 on the unimplemented MovieClip member boundary.
Receipts: `evidence/build-0.1.0-dev.8-smoke.json` and
`evidence/avm1-runtime/manager-frame-after-inits.tsv`.

**VERIFIED (dev.9 handoff):** Three focused bootstrap tests pass, including
rejection of pending/halted handoffs and retention of heap values and ordinary
code. A private harness against the integrated crate repeats all 452 initializers,
consumes the session and reaches the same 31-instruction original frame boundary
using the retained program, listener and registry. Receipt:
`evidence/dev9-bootstrap-handoff/receipt.json`. These checks do not expand the
native-global or attached-display scope above.

This driver is original code. Its only external format basis is the Adobe SWF
specification already itemized in [AVM1 execution](AVM1_VM.md); original asset
inputs and native compatibility evidence are recorded there and in
[REA native evidence](REA_NATIVE_ALTERNATIVE.md). No game code or assets are
included in this document or repository.
