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
`install_movie_clip_metadata` installs Object and a MovieClip constructor identity
with a prototype for inheritance study. This native constructor has no implicit
implementation: any call still reaches the supplied Host. The setup does not
create display objects, native methods, singleton namespaces or a player.

`mhf frontend init-probe <package.upk> <movie>` supplies those limited bindings,
the explicitly supported property-flags intrinsic, and the bounded
[Selection listener host](FRONTEND_SELECTION_HOST.md). It compiles frame zero
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

This driver is original code. Its only external format basis is the Adobe SWF
specification already itemized in [AVM1 execution](AVM1_VM.md); original asset
inputs and native compatibility evidence are recorded there and in
[REA native evidence](REA_NATIVE_ALTERNATIVE.md). No game code or assets are
included in this document or repository.
