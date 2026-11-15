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
