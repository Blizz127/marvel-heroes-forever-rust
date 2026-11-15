# Dynamic lookup immediately after the original saved Construct phases

VERIFIED: one controlled execution at source commit
`f6934e3a59cc5e1fa30599dde20c755e7b0e1064` ran all 34 retained original
login Construct vectors and immediately called
`frontend::saved_dynamic::lookup_after_saved` after each successful saved
phase. All 34 lookups returned `Member::Missing` for the exact member
`onConstruct`. No dynamic callback was invoked. This establishes the lookup
frontier in the supplied fixture, not complete Construct delivery or the
native initialized frontend's member state.

The caller used the returned `saved_dispatch::Outcome`, the same VM, and the
same retained Environment. No script or event intervened. The explicit
`Profile { same_completed_phase: true }` assertion follows that control flow;
it is not an unforgeable VM token or native scheduling evidence.

## Preserved setup and source identity

The private input is the retained `evidence/construct-saved-phases/all34`
harness, whose earlier execution stopped at `DynamicLookupPending`. This
increment preserves its full 147-node saved login Scene, 75 Clip instances,
33 TextField instances, selected 34-target order, original body spans and
hashes, raw target/focus/load/source state, prototypes, children, authenticated
native-name profile, display/TextField state, and distinct nonempty supplied
Environment stacks and four-register banks. The selection order is a
controlled request, not an observed native queue order. The 39 unsupported
unnamed scene nodes remain represented as before.

Both the earlier source `f251a7e44c30c31beada860f116c0140a1f0043c` and this
freeze use the same `scene_adoption.rs`: `one_builtin_context: true` is an
explicit fixture input, and the adopter binds every Clip owner to that one
context. No new binding, inferred default, prototype mutation, callback,
factory, visibility write, or successful Host stub was introduced. The
existing nine rejecting Host overrides are unchanged. The current additional
`mouse_hide` Host default rejects; no route attempted it.

The harness delta consists of importing `saved_dynamic`, calling the lookup
immediately after each saved-phase return, checking returned identities,
recording its typed result, and counting classifications. Its Cargo package
name and path dependency select the archived source freeze. The exact
original/new source diff is retained privately in `harness.delta.patch`.

All action code belongs to scene movie 13, header version 15. Owner definition
sources remain distinct: 23 owners use movie 13/version 15, ten use movie
19/version 11, and one uses movie 5/version 10. The lookup reports the actual
owner definition and Environment movie/header separately from the retained
caller's Environment identity. The private `fixture-review/target-map.tsv`
retains the complete 34-target/source map, including the Clip owners named
`username_tf` and `password_tf`; a suffix was not used to reclassify them.

## Observed results

| Observation | Result |
| --- | ---: |
| Original saved Construct phases executed | 34 |
| Immediate dynamic lookups | 34 |
| `Missing` | 34 |
| `PresentNonCallable` | 0 |
| `CallablePending` | 0 |
| `DeferredConversionPending` | 0 |
| Typed lookup failures | 0 |
| Dynamic invocations | 0 |
| Original instructions | 1,686 |
| Verified owner assignments | 556 |
| Verified soundMap assignments | 118 |
| Host attempts | 0 |
| VM Object count | 126 → 194 |

All 674 typed final-value observations and the 34 original instruction counts
match the earlier independent source oracle. All 2,505 earlier output rows,
including their order, are byte-for-byte identical after removing this
increment's 34 lookup rows and one summary row. The comparison passed 516
focused checks. No instruction count or Object allocation was added by a
lookup. All saved Environments remain unhalted with their supplied target,
stack, and register values intact. Observed globals, owner/source identities,
saved Scene and movie bytes, and display/TextField state remain unchanged.

Each successful lookup retains the exact owner, VM Object, display target,
definition source, Environment movie/header, retained Environment ID, event,
member name and saved-phase progress. `lookup-details.json` records these
facts without discarding a value or error. The harness would retain a typed
lookup failure and proceed only to the next independently requested target;
an unexpected saved-body failure stops the probe. No failure occurred, and
neither stage was replayed.

OBSERVED: the retained source bodies assign ordinary `visible` data and
`enableInitCallback`. They do not assign `_visible` or `onConstruct`. Their
assignments therefore supply no evidence of native visibility mutation or
dynamic callback invocation. `Missing` is an observation after authoritative
member lookup under this fixture's supplied prototype/child state; it does
not establish absence after native class preparation or later initialization.

## Private receipts and verification

All paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/construct-dynamic-lookup/`.
No client assets, code bodies, extracted tables, captures, or binaries enter
the repository.

- `source-freeze.json`: all 263 tracked source-file hashes and Git identity;
  `source/` is its private archive.
- `preparation.json`, `harness.delta.patch`, and `run-freeze.json`: exact
  fixture change, retained old evidence, harness and ten original input hashes.
- `build.json`, `build.stdout`, `build.stderr`: first focused offline locked
  build, `nice -n 19 cargo build --offline --locked -j2`, debug information
  disabled, Rust/Cargo 1.98.1. It passed without a compile retry.
- `execution-started.json`, `execution.json`, `result.tsv`, `run.stderr`: one
  guarded binary execution; exit zero and empty stderr. Source, input and
  harness hashes were checked before and after.
- `comparison.json`, `lookup-details.json`, `api-review/`, `fixture-review/`:
  read-only API, source/oracle and captured-result verification.
- `receipt.json`: aggregate artifact identities and scope.

The executed binary SHA-256 is
`01992cd848199ecfb03087667e475c45a20c3f5a74745da1528af96c50abccec`.
The captured output SHA-256 is
`1ec48a62a404cd377ce9604dbe1475952b3988834db7b0b6c6a82719f22924ec`.

## Scope and provenance

The lookup uses the original Rust implementation documented in
[SAVED_DYNAMIC.md](SAVED_DYNAMIC.md), composed after
[SAVED_DISPATCH.md](SAVED_DISPATCH.md) and
[SAVED_SCENE_HANDLERS.md](SAVED_SCENE_HANDLERS.md). These project sources,
the retained original project harness/oracle, and their existing private
owner-file evidence are the only sources read for this increment. Their
existing repository provenance applies; no new third-party code, network
source, dependency, or licensing claim is introduced.

This check did not execute a native client, constructor schedule, later frame,
ExternalInterface callback, renderer, login, or service. VM-internal builtin
Object allocation already performed by the saved bodies remains part of
their 68-object delta; it is not dynamic callback construction. No saved
constructor completion, native lifecycle equivalence, rendering behavior or
retail parity is claimed. Callable/deferred/noncallable/error branches remain
unexercised by these actual 34 lookups.
