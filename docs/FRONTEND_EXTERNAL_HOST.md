# Explicit movie callback routing

**INFERRED implementation from VERIFIED native paths:**
`frontend::external_host::ExternalHost` connects an explicitly adopted attached
VM root to a supplied native player/receiver association and handler state.
`FrontendHost` composes this route with Selection, class registration and display
properties. It does not create the initial Unreal player, install a movie handler,
populate namespaces, or run the native startup lifecycle.

The evidence is [ExternalInterface binding](EXTERNAL_INTERFACE_BINDING.md) and
[frontend receiver association](FRONTEND_EXTERNAL_RECEIVER.md): the current
Environment target selects the movie, whose user data selects its owning player;
that player's mutable ExternalInterface property selects the receiver. `this`,
definition-movie provenance and a global engine object do not select this route.

The original implementation requires an attached root matching `Vm::root()`.
Additional contexts must explicitly bind through the supported immutable
case-3 parent chain to that exact root. Context and receiver counts and parent
depth are bounded. Numeric object identities are local to one supplied VM/native
domain, and must not be reused with another VM. Distinct AS2 roots, target
retirement/reparenting and automatic context admission remain unsupported.

`PlayerAssociation::after_successful_new_load` applies only the proven
successful new-load transition: a null previous receiver becomes the owner;
an existing override is preserved. Existing-wrapper Start uses its explicit
snapshot, without applying that transition. `set_receiver` changes the supplied
state without reconstructing resources or resetting it on later calls.

Calls use binding `0x4d480010` and `HostCall.context`. A missing or unbound script
context rejects even if `this` happens to reference a valid object. With no
handler the native result is Undefined before argument conversion. With a
handler and null receiver, the supported actual-string localization arguments
return Undefined; arbitrary conversion and side effects remain unsupported.
An unknown non-null receiver rejects rather than selecting a fallback. A
registered checked frontend-player receiver invokes its own
`LocalizationCallback`, producing fresh fields through VM `HostResult`
materialization. General reflected dispatch and other receiver classes remain
unsupported. Receiver registration is an explicit caller assertion of the
checked native class/method/state profile, not discovery from a numeric ID.

The supplied `available(context)` query tests handler presence independently of
receiver availability. It is not yet wired to the native property marker.
ExternalInterface's fresh callable graph can use `CALL_BINDING`, while namespace
installation, deferred lookup/cache behavior and event scheduling remain
separate work. No saved-script execution or retail parity is claimed merely
from constructing this host.

Sources are the owner's private pinned executable and packages through REA,
listed in the two linked native studies, plus original project code. No outside
implementation is copied or linked.

**VERIFIED focused Rust checks:** `cargo check --offline -j2` passed. Three
`frontend::external_host` tests passed for current-target admission, absent
handler/null-receiver distinctions, override preservation, unrelated roots and
context-budget rejection. These synthetic checks do not establish native
startup or execution of the original wrapper through this new route.

## Isolated original-script validation

The private integration harness is under
`~/.local/share/marvel-heroes-forever-rust/evidence/frontend-external-host-validation/`.
Its source was frozen before compilation from commit
`49aa8b6a60b99f5f10f43066deadbfd4879cfbd1`, tree
`b5fde825ffcd943bdb4b146c0a36fa8022f850f9`. The immutable archive SHA256 is
`55e215d2f27b87dd4e789eeb9a3dd7c18b8eb1b28164154f27ccbea8084f29ca`.
`source-manifest.before.json` covers 161 source files; dependency and input
manifests separately retain the private harness's toolchain, dependency graph,
and owned input identities. The optional renderer is outside this check.

The validation deliberately supplies an attached root and raw state, the
namespace bindings, player/receiver identities, handler state, configured label
table, and locale catalog. These are controlled inputs to the saved code and
the new host route. They do not establish native initial-player creation,
automatic namespace materialization, saved startup scheduling, or a functioning
login screen. Retail scripts, binaries, data and harness extracts remain private.

**VERIFIED focused Rust integration:** the unchanged selected original function
records ran through the attached root, `Vm::build_external_interface(CALL_BINDING)`,
`FrontendHost`, `ExternalHost`, and the actual `ConfiguredLabels`/`LocaleCatalog`
backend. Three selected caller results and six directly inspected wrapper
objects passed. A supplemental unchanged-wrapper call used the same native
`call` function through a controlled alias object: the recorded native `this`
changed while the active context remained the captured attached root. Its fresh
result also passed, for **10 backend calls and seven distinct inspected result
objects** in this executable. The four returned fields, their flags, prototype
and constructor metadata were checked. Comparison with retained earlier Rust
output passed 12 rows / 78 shared columns; that comparison and the direct
known-member baseline are not native runtime oracles.

A separate controlled executable passed **19 case groups** covering eight
`this` variants, receiver overrides, absent handlers, null and unregistered
receivers, missing/unregistered targets, context/receiver/depth limits, and a
nine-request backend budget whose tenth request rejects without changing its
counters or last resolution. An existing Selection listener and an existing
class registration/event survived the external successes and failures; Selection
routing remained operational afterward. Most controls construct explicit host
requests; the original-script and alias proofs above pass through VM execution.

Both executables built offline with the lockfile, `nice -n 19` and two build
jobs. All 161 source files, 11 runtime/reference inputs, active dependency
metadata, final harness files and executed binaries passed before/after identity
checks. The dependency graph contains only the private harness and frozen
project library, with default features disabled.

The private first control run correctly rejected an attached owner incorrectly
wrapped as an ordinary Object value. The fixture was corrected to use an
ordinary Selection listener and the typed attached value; product code and its
guards were unchanged. A retained-output comparison initially rejected an extra
context annotation; its parser was corrected to compare the shared columns.
Both initial failures and final receipts remain retained. `receipt.json`
SHA256 is `7336fe8f38080d5e932494bf4e404be9bd839f7acdbc78cc864a4c99325aebe3`;
`MANIFEST.json` indexes 237 private artifacts. No native client, full saved
initializer, server, login, renderer or retail parity check was run.
