# Persistent saved pointer handler

`frontend::pointer::PointerController` owns a persistent VM, one explicit object
association, its original compiled clip action, and the display geometry for one
saved structural scene. It is an intermediate runtime component for connecting
native pointer events to actual saved script and original geometry. It does not
run movie initialization, class constructors, frame actions or other events.

`from_scene` requires exactly one saved clip action whose MouseMove bit is set.
No handler is an error; multiple handlers are unsupported until their native
dispatch order is established. The scene's movie version selects the VM profile.
The explicit plain object is a diagnostic binding, not a native MovieClip factory
or claim about its allocation category. The complete owned original span is parsed
with its original source offset and retained across subsequent invocations.

`pointer_stage_twips(Some(point))` executes the handler with absolute stage-twip
coordinates supplied by the caller. Surface pixels, DPI and fitted viewport
conversion belong to the native window adapter. `None` marks the sample as
unavailable without executing code, moving the cursor or hiding it. Nonfinite
coordinates reject before changing the prior sample. The controller does not
coalesce repeated samples or invent movement when a window resizes.

Every successful action publishes a fresh `nodes()` snapshot from DisplayState.
Only current world transform and effective visibility overlay the immutable
source identity, placements, masks and action bindings. The VM and mutable state
persist, so the next original handler reads the transform produced by the previous
one. Statistics distinguish attempted and completed dispatches and count the
instructions/host calls of completed invocations only.

Actions can mutate state before an unsupported operation or budget error.
Consequently any execution failure permanently halts the controller; it does not
retry, replay or publish that partial snapshot. `display()` remains read-only and
can expose the partial state for diagnosis. `nodes()` remains the last successfully
completed state. Invalid numeric input does not start an action and does not halt.
The diagnostic bounds default to 10,000 dispatches plus the existing VM and display
limits. There is no unbounded heap collector or trace-history retention policy yet.

The existing single-invocation `frontend pointer-probe` CLI uses this same
controller. The native `frontend window-pointer` command is being integrated with
the persistent GPU scene and actual host cursor events; it remains separate from
full startup and general event dispatch.

## Sources and validation scope

All code is original. No new outside source was used. Inherited sources and
limits are recorded in [display state](FRONTEND_DISPLAY_STATE.md),
[AVM1](AVM1_VM.md), [scene structure](GFX_SCENE.md) and
[GPU composition](FRONTEND_PREVIEW.md). Game data and private validation records
remain outside the repository and build packages. Structural or isolated handler
checks do not establish native retail event timing, visual or gameplay parity.
