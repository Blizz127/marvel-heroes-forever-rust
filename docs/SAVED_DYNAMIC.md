# Post-static dynamic event-member lookup

`frontend::saved_dynamic::lookup_after_saved` advances a successfully returned
`saved_dispatch::Outcome::DynamicLookupPending` to the current attached member
lookup. It does not invoke that value. A missing member, a present noncallable
value, a function value awaiting invocation, and a retained deferred builtin
name awaiting unsupported conversion remain separate results. None represents
complete event delivery or constructor execution.

The caller supplies `Profile { same_completed_phase: true }` to assert that the
outcome came from the completed saved phase of this delivery in this VM, with
no intervening event/script execution. Public arena IDs and the Copy outcome
are not an unforgeable execution receipt. The function also checks the live
attached owner, saved Environment owner/target, nonhalted Environment and
explicit builtin-context membership. Same-root ancestry alone does not supply
the context association. A skipped wrapper is rejected before member lookup;
the wrapper's eligibility tests and any Load bit-5 transition are not repeated.

The returned `Context` retains the actual attached ID, owner object, display
target, definition source movie/header, Environment movie/header, retained caller
Environment, exact event name and saved-phase progress. The caller Environment
is not a claim about the Environment a future callable would select. The
function's retained captures, invocation support, method-name hint and actual
body effects remain outside this step.

## Grounded lookup boundary

**VERIFIED native static evidence, reused:**
[FRONTEND_EVENT_DISPATCH.md](FRONTEND_EVENT_DISPATCH.md) establishes that saved
wrappers run before the current `onLoad` or `onConstruct` member is read through
the actual display receiver's raw-member interface. The ordinary route checks
own auxiliary storage, named children, then the display prototype. A found
Undefined or other noncallable value is not replaced by an inherited function.
The native event mappings are Load mask 1 to `onLoad`, and Construct mask
`0x40000` to `onConstruct`.

The implementation delegates that lookup to `Vm::read_attached_member`; it
does not approximate it using `Vm::get`, `attached_load_present` or an own-map
shortcut. Existing native-name inventory authentication, child snapshot and
receiver guards, resolver/accessor restrictions and prototype-depth limits
remain authoritative. Unsupported children, accessors, omitted builtin members
or unbound profile state produce errors rather than Missing. Reading uses the
current VM state, with no value cached before the saved phase.

`Vm::classify_callability` validates the returned value and classifies its actual
VM object kind without a call. A function-valued but unsupported callable still
produces `CallablePending`, never a promise that invocation is implemented.
Native tag-11 values preserve their name token as `DeferredConversionPending`;
no factory, cache or namespace mutation occurs. Native accessor invocation and
tag-11 conversion are separate unresolved event-path branches.

## State and failure semantics

The API takes `&Vm` and no `Host`. It cannot mutate the Environment's stack or
registers, create an object, execute a getter, invoke a dynamic function or make
a host call. Errors retain the exact incoming phase and the failing stage:
admission, receiver, Environment, builtin context, member lookup or value
classification. This lookup introduces no partial VM effects. Prior wrapper or
saved-body effects remain, including on an error; the diagnostic phase is not
permission to replay those bodies. A missing/noncallable result also makes no
event-completion, lifecycle, queue or registered-constructor claim.

No additional source parsing or retained native tables are introduced. Existing
VM limits bound the underlying member lookup. Returned scalar/object identities
are preserved, and retained strings/tokens share their existing bounded value
storage. The API does not establish a cross-VM identity system or a native
schedule.

## Focused validation

The new tests use invented source, identifiers, state and name inventories;
none embeds original game data. They exercise both event names, missing versus
present Undefined, nearest-prototype shadowing, noncallable plain objects versus
callable native handles, named-child precedence, unsupported-child failure,
deferred-token retention without factory effects, unbound phase/context/name or
child state, invalid Environment identity, and a synthetic saved `SetMember`
whose value is observed only after that phase returns. The lookup keeps the
supplied operand stack/registers and existing saved effects intact.
Additional negative cases retain an inherited-accessor error, reject a different
owner's Environment, and reject a stale phase after a synthetic body failure
halts its Environment. Accessor setup uses the supported Rust `addProperty`
intrinsic before Environment adoption; the getter is never invoked.

The independent read-only source/inventory audit is retained privately in
`evidence/saved-dynamic-contract/AUDIT.md` (SHA-256
`dd7f5fde2a8888248165b0ef233add780a17c50ab064eaef9e5d5c92c33576ed`),
with receipt `8d21cc564ed4a52c302cb212dfcc9d16ebd8ee92413ddbcf678ef7c7bd97c7b9`.
It rechecked six frozen source hashes and the inventory hash; it did not execute
VM code or measure a fresh native runtime. Its conditional name-classification
finding does not establish that a live target lacks either member.

Own `on*` writes remain rejected by the existing native pre-write-hook boundary;
these tests do not bypass that gate to manufacture own data. The existing
attached getter's own-data precedence is reused. No original Construct body,
native client, server, UI or rendering run is required or claimed by this step.

**VERIFIED Rust checks:** eight focused tests passed with
`nice -n 19 cargo test --offline -j1 frontend::saved_dynamic::tests -- --nocapture`
in a private harness. It imports the actual product dependency and this new
module directly, leaving the product module hook to the integrator. The frozen
dependency was `970084d` plus read-only accessor commit `febf575` (local
`a9162dc`). Final module commit `65cf30ae588977e8a366f520d5e8980d9db89f81`
has SHA-256 `a06998026f6af7c89318b0446989c89eadd16e616f531427bec7d3b265a43d30`.
All 91 inventoried source/Cargo files remained byte-identical across the final
check; its before/after manifests record the separately committed module and
the documentation-only work outside that compiled inventory.

The initial attempt passed six checks and rejected two fixture setups at the
existing loaded-movie version gate. Those invented child fixtures were corrected
to supply an explicit different-version binding and the required encoded
case-3 owner-offset flags. No product lookup or VM gate changed. Both attempts,
the failed fixture source, final source, harness, lockfile and compiled binary
hash are retained privately under `evidence/saved-dynamic-check/`. The final
receipt is `6d4221ecad233a4dcf484e9ef93fc8e575fdc5ca3d80af1e9182f08818c09361`;
the manifest is `ad40aa3d7f5576ad9e4740ab57d0708a52ba4002c9b17427267f207592b6523b`.
The dependency emitted its existing unused `ObjectId` import warning from
`attached_visibility.rs`; the new module emitted no warnings. No full suite or
original 34-body replay was run.

## Provenance

| Source | Licence/ownership | Use |
| --- | --- | --- |
| This repository's `docs/FRONTEND_EVENT_DISPATCH.md`, `src/frontend/saved_dispatch.rs`, `src/frontend/saved_handlers.rs` at `970084dfbf31d93700956332a840a730ed73f6f4` | Original project code and compatibility research | Post-static ordering, exact names, phase results and failure boundaries |
| This repository's `src/avm1/vm.rs`, `attached.rs`, `attached_events.rs`, `deferred.rs`, `saved_state.rs`, `saved_runtime.rs`, `src/attached_name_profile.rs` at the same commit | Original project code | Existing identity, member, context, state, classification and admission contracts; invented test fixture patterns |
| Original VM read-only seams `febf5751e59f933669d880334c9077a3e1d969bd`, applied locally as `a9162dc` | Original project code | Public value classification and actual builtin-context map membership; no invocation gate changes |
| Owner-private `evidence/rea-event-dispatch/complete.bundle.json`, through the dispatch document | Proprietary owner executable, compatibility study | Previously retained REA getter/callable evidence; no new binary analysis or copied implementation |
| Owner-private `evidence/standard-names/inputs/native-name-profile.v1.tsv`, SHA-256 `902b4f15629d3ae32aa6867165d275f2ef3ea289ced3696b9bf132155fd00b91` | Facts recovered from the owner's executable | Helper read-only audit: exact event names absent from the 117 standard, 44 MovieClip and 9 Object inventory rows; conditional Ordinary/Absent admission, not actual live-member absence |

No external implementation source was used or copied for this increment. The
owner executable underlying the reused receipts has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
That source hash was established by the retained research, not remeasured here.
