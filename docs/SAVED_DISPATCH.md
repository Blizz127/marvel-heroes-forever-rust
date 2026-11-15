# Saved handler execution phase

`frontend::saved_dispatch::execute_saved_phase` connects an owner's retained
`SavedHandlers<SavedActionId>` table to the VM's persistent saved Environment.
It executes the saved-vector portion of one explicitly requested Construct or
Load delivery. A normal result is `DynamicLookupPending`: the dynamic member
must still be resolved after the saved calls return. This is not successful
completion of the entire event, a constructor call, or a startup scheduler.

The original Rust implementation composes the native contracts already recorded
in [event dispatch](FRONTEND_EVENT_DISPATCH.md),
[saved execution](SAVED_EVENT_EXECUTION.md), and
[handler identity](SAVED_HANDLER_TABLE.md). It reads no new outside source and
does not include original script bytes. The same-VM association of the table,
owner and Environment remains the caller's responsibility because numeric arena
handles are session-local.

Before applying wrapper effects, it checks the adopted Environment's owner and
target, halted state, and every selected table entry against the VM's actual
installed owner and original mask. Only pure Construct or Load installations are
currently executable. Combined masks remain preserved by the installer but
require the separate mode-3 execution profile. Engineering admission failures
leave the VM unchanged; this validation is not a claim about native invalid-input
behavior.

The wrapper then uses the existing native raw-state helper. Target bit 4 skips
before any Load-specific mutation. Otherwise Load clears target bit 5 before
checking its load/source eligibility fields, even if those checks reject.
Construct has neither the Load-only clear nor its additional gate. The current
fields are read from and written back to the same attached owner.

The saved vector is visited in retained insertion order. Each callable keeps its
own installed source/pool state and uses the supplied Environment's persistent
stack and fixed registers. Completed bodies and target-bit skips are reported
separately. A body error stops before later saved handlers and dynamic lookup;
the VM retains partial state and halts that Environment. The phase never retries
or rolls back. Progress totals count completed bodies only; they do not hide
the failed body's possible partial effects. Counter/admission failure after
wrapper entry also leaves the phase stopped for diagnosis.

Immutable borrowing excludes handler-table changes during this bounded loop.
Native reentrant vector mutation, ordinary script calls from saved execution,
dynamic callable/accessor invocation, event generation, queue scheduling and
exception recovery remain separate implementation requirements. An absent static
key still yields a pending dynamic lookup after wrapper admission.

## Focused validation

The synthetic checks exercise cross-handler state, raw wrapper mutation order,
retention after a body failure, untouched later wrappers, and admission before
Load side effects. They are compatibility checks for the Rust composition only;
they do not prove actual event eligibility or native startup timing. Private
terminal receipts are retained under `evidence/saved-dispatch/`.

**VERIFIED:** all four focused checks pass with no compiler warnings. An initial
fixture used the unsupported legacy Add opcode and stopped correctly; the
cross-handler arithmetic fixture now uses the already admitted numeric Add2
path. The failed setup and warning-only intermediate rerun remain recorded in
`initial-fixture-failures.json`, separately from `focused-checks.json`.
