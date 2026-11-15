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

`setup_vm` permits explicit intrinsic/host bindings before execution starts.
`install_movie_clip_constructor` installs Object and the bounded typed unbound
MovieClip intrinsic. It preserves the established native category and constructor
callback profile; unproved subclass prototype-assignment effects reject before
allocation. It does not create or attach a rendered display object. The older
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

This driver is original code. Its only external format basis is the Adobe SWF
specification already itemized in [AVM1 execution](AVM1_VM.md); original asset
inputs and native compatibility evidence are recorded there and in
[REA native evidence](REA_NATIVE_ALTERNATIVE.md). No game code or assets are
included in this document or repository.
