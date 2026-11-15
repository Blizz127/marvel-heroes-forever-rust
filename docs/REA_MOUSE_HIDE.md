# Native Mouse.hide: deferred graph, return and host dispatch

**VERIFIED static native contract:** the concrete `Mouse.hide` callback writes
**Undefined**, ignores the ActionScript receiver, and selects a user-event
handler through the **active Environment's target/root/MovieImpl**. With zero
arguments it sends event type **0x16**, index **0**, to that handler. The callback
itself does not change cursor visibility. The actual installed game handler and
its engine/window effect are a separate proof boundary.

This document establishes no native execution, successful cursor hide, startup
global state or full Mouse implementation. A handler request that returns success
without a grounded effect must not be used to pass the original script frontier.

## Exact original script frontier

**OBSERVED controlled manager increment 10:** 452 initializers complete with
49,244 initializer instructions; the frame stops after 290 instructions at
**CallMethod 6391**, CodeId 452, named `hide`, argument count 0. The original
source pushes Double +0, looks up scoped `Mouse` with GetVariable 6385, then
calls `hide` at 6391 and discards the result with Pop 6392. This is not an
explicit `_global.Mouse` instruction; actual scope resolution remains required.

The diagnostic's minimal binding failure is not evidence that native Mouse is
absent. Native bootstrap separately establishes the deferred Mouse token. The
later cursor `hitTestDisable`, dimensions and `topmostLevel` writes belong to
future source instructions and are not Mouse.hide callback effects.

Private evidence:

- Owned `frontend-startup/MarvelGame-frontend_manager.gfx`, SHA-256
  `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
- `avm1-attached-manager-frame-action-increment10/receipt.json`, SHA-256
  `7a2347e0d81fb9e9d6e45297e13723e4c47ef7acc742d82a9b1fe6bc7e96f73d`.
- Its `static-context/receipt.json`, SHA-256
  `56ef566a9e6d16e5adb2398983b4bd624e5e8106e78ee4646bcd56181d712605`.

## Deferred registration, context and publication

[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md) establishes bootstrap call
**0x14020fcf1**, wrapper **0x140201ea0**, factory **0x1401fff70**, builtin
index **27**, name offset **0x210**. The checked finite registration-wrapper
comparison binds its registry insertion and raw global write. The initial Value
is **tag 11**, a retained name token; the callback/cache is stored separately in
the current context's lazy factory map. Registration does not invoke the factory.
Fresh global descriptor flags become **1** in the final bootstrap visitor.

The resolver uses the active Environment's **+0xe8 context**, returns a separate
FunctionRef and preserves any retained token alias. Repeated successful
resolution uses the cached callable; a raw token read is not materialization.
No result may be inferred from a missing registry entry or failed factory.

The named CallMethod handler resolves the actual receiver at **0x14020bbc8**,
then performs the real named-member lookup and converts the returned method
Value through **0x140199880 at 0x14020bc22**. Both conversions use the active
Environment. A remembered name `Mouse` or `hide` must not bypass changed scope,
namespace, getter or callable identity.

Complete factory **0x1401fff70..0x14020013a** allocates callable C (0xb0 bytes)
through **0x1401ffa60**, then prototype P (0xa8 bytes) through
**0x1401e7a90**, using cached Object.prototype, index 1. It registers P in the
numeric cache as index 27 at **0x140200072**. Publication loads the actual
context global at **context +0x60** and writes tag-8 C under Mouse at
**0x1402000f1**, supplied flags 0. The raw setter preserves a preexisting
entry's flags; a fresh token's flags 1 therefore remain 1. This is not a
reset-to-zero rule for changed descriptors.

## Callable, prototype and listener graph

Complete C constructor **0x1401ffa60..0x1401ffc42** uses cached
Function.prototype, index 7. C's final primary table is **0x142516fb0** and
script interface **0x142516f00**. The interface category is **0x17**, a native
function object. C's own invocation callback **0x1401894f0** explicitly returns
Null. That direct-call path is different from `hide` and is not required by the
original frontier.

The constructor performs these distinct operations:

1. Install three ASBroadcaster function descriptors—`addListener`,
   `broadcastMessage`, `removeListener`—with flags **1**, through
   **0x1401e9bd0 at 0x1401ffb21**.
2. Construct/write the listener Array through **0x1401dc5b0**.
3. Run **0x14019d780(C, string_context, null Environment)** to retain that
   current Array in internal **C+0x78**.
4. Install `show` and `hide` with flags **7**, through **0x1401e9bd0 at
   0x1401ffb4f**.
5. Construct a separate native function with callback **0x1401b70c0** and
   retain its FunctionRef in **C+0x90**. This is not a named own method install.

Complete retained producer **0x1401dc5b0..0x1401dc693** allocates 0x98 bytes
and calls **0x1401db6e0**, then raw-writes the resulting object under
`_listeners`, flags **1**. The Array constructor uses cached Array.prototype,
index **2**, and initializes empty Array state plus its parent marker. It does
not pass through shared New or add an own `__constructor__`. A true producer
return alone does not certify allocation success; that native failure path is
separate. The supported graph requires the successful fresh Array path.

The listener producer installs a real Array, not an invented empty
Plain object. The complete new **0x14019d780..0x14019d852** raw-reads builtin
`_listeners` at offset 0x3c0, converts its Value with the supplied Environment
(null in this fresh call), and requires native category **7** to retain the
same object into C+0x78. A non-Array conversion clears that pointer; a missing
read leaves it unchanged. Fresh prior installation and initially zero fields
close this path without invoking arbitrary script. Current mutation, accessors
or alternate listener objects remain separate profiles.

P's final primary table is **0x142513d80**, script interface **0x142513cd0**,
and auxiliary metadata table **0x142516268** at P+0x68. Its native category
is **6**, with a true prototype predicate at interface +0x80. Shared metadata
**0x1401b80f0** installs actual tag-6 **C.prototype=P, flags 3**, and marker
**P.constructor, flags 3**, backed by saved C FunctionRef at P+0x70. The hidden
reference begins null; no own P.`__constructor__` is installed by that join.
C/P have `__proto__` markers, flags 3, backed by their actual cached parent links.

P constructor **0x1401e7a90** installs its parent at **0x1401e7add**, then
calls the shared constructor/prototype join at **0x1401e7b2e**. Only after
that constructor returns does the factory install four descriptors—
`addListener`, `removeListener`, `show`, `hide`—at **0x14020004a**, with
flags **1**, all bound to **0x14018d3a0**. That complete callback only
clears its result and writes Undefined. **P.hide is not C.hide.** The actual
C.hide descriptor binds **0x1401b6fe0** with flags **7**. Dispatch by text alone
would confuse two different native operations.

The successful fresh graph has **8 own keys on C**: `__proto__`, the three
broadcaster methods, `_listeners`, `show`, `hide`, `prototype`. P has **6 own
keys**: `__proto__`, `constructor`, and its four methods. Internal C+0x78 and
C+0x90 references are not extra named properties. These counts do not include
inherited members or imply a complete implementation of listener behavior.

Unknown or omitted native members must stay unsupported. A partial graph does
not establish absence, permit complete enumeration, or justify substituting a
Plain singleton for C. No native allocation-count parity is claimed.

## The actual hide member lookup

C's normal getter is specialized **0x1401f38d0..0x1401f3b51**; its raw getter
is **0x140197000**. The normal getter first checks context byte **+0x68 == 1**.
When enabled, it recognizes seven builtin aliases and three exact literal names.
The private pointer/literal checks establish that none is `hide`. Otherwise,
and after all those checks miss, **0x1401f3982** invokes raw interface +0x58
with the original key and active string context. Thus actual own `hide` reaches
its stored function for either value of the extension switch; no guessed fresh
extension-switch default is needed for this exact method.

The same normal getter may create functions or expose special values for other
names. Those branches are not general ordinary-member permission. C's normal
setter **0x1401ffd70** is specialized and remains outside this read/call proof.
P uses the shared normal/raw prototype getters **0x1401e8670/0x1401e7c40**.
Current descriptor replacement, inherited lookups, accessors, resolvers and
changed parent links must retain explicit implementation guards.

## Finite own-member read contract

**VERIFIED retained-evidence follow-up:** the eight exact fresh C keys listed
above all miss the seven canonical interned extension aliases and all three
literal comparisons in **0x1401f38d0**. The literal comparisons include their
terminating NUL, and each admitted spelling differs before that terminator.
Whether context byte +0x68 equals 1 or not, these exact keys reach
**0x1401f3972**, preserve the original key at **0x1401f3979**, and invoke raw
+0x58 at **0x1401f3982**. No extension callback or function-allocation branch
is entered for these names. This does not license other names or aliases.

The finite read profile uses source versions **7..19**, the active canonical
string context, and the unchanged successful Mouse graph. In generic raw
**0x140197000**, exact own-key identity at **0x1401970cf** reaches the
stored Value copy through **0x14018d9d0 at 0x1401971b6**. Thus C's three
broadcaster methods, `_listeners`, `show`, `hide` and `prototype` return their
actual stored own values. Reading a function identity does not implement its
callback, and reading `_listeners` does not implement Array access or mutation.
A missing or changed own descriptor cannot be replaced with guessed absence.

`__proto__` is an explicit exception to ordinary stored-value reading. Raw
getter **0x14019705f..0x140197066** recognizes its builtin key before any
own hash lookup, then reads the **actual internal parent at object+0x30** at
**0x1401971d4**. A nonnull parent is converted through **0x1401936f0**; a
null parent produces found=true with **Undefined**, not Null. Its own metadata
Value, accessor and flags do not choose that result. Canonical C/P reads can
therefore expose cached Function.prototype/Object.prototype respectively only
under their actual internal-parent guards. An implementation that also freezes
the metadata descriptor is imposing a narrower admission profile.

P's normal getter **0x1401e8670..0x1401e867e** is a complete thunk which
passes Environment+0xe8 to raw +0x58. P's raw **0x1401e7c40** first selects
public/hidden constructor names; its four exact own method names miss those
tests and reach **0x140197000 at 0x1401e7cfa**. Their own tag-8 values take
the same direct copy path. No callback invocation or parent lookup occurs on
those own hits. P.`__proto__` uses the internal-parent rule above.

P.`constructor` is a special saved-reference read, not a generic marker Value.
The route **0x1401e7d1b → 0x1401a16c0** first honors an own non-marker
override. For the fresh marker, it selects P's saved public constructor
FunctionRef at **P+0x70** and produces tag-8 C. This route does not reread
GLOBAL.Mouse or C.prototype. Representing it as canonical C is sound under
guards preserving that marker, saved reference and canonical graph; it does not
assert that a changed public C.prototype would itself change this native read.
Null saved references, deletion, alternate descriptors and inherited/resolver
fallbacks remain unsupported in this bounded profile.

The observed construction sequence is C: `__proto__`, three broadcaster
methods, `_listeners`, `show`, `hide`, `prototype`; P: `__proto__`,
`constructor`, `addListener`, `removeListener`, `show`, `hide`. In particular,
P's constructor marker is installed inside its constructor **before** the
factory installs its methods. This is an installation sequence, not proof of
native enumeration order.

Private `rea-mouse-hide/callable-member-review.json` is sealed with SHA-256
`16f5cd9a93a9a8d785781e2d8b3b9749d4245472eedc1acd66fb1c5a2ce6bb86`.
Its original checker rechecks **701 retained instruction spans**, exact branch
anchors, finite alias/literal mismatches, metadata bindings and factory order
against the owned PE; no new REA request or native/script execution was made.
The separate `rea-mouse-hide/prototype-member-review.json` binds 11 retained
canonical source records and has SHA-256
`660ccfd4e980692a1e018d2513108cd85017ebe0390e7d815ba7f32470d9eac6`.
It preserves the constructor/parent special cases and explicit fallback limits.
Neither follow-up establishes a concrete cursor-handler effect.

## Exact zero-argument callback

Complete callback **0x1401b6fe0..0x1401b70bd** takes the native FnCall and:

1. Releases the existing result Value through **0x1401887a0**, then writes
   **tag 0 at 0x1401b6ff9**. The return is explicitly Undefined.
2. Reads **FnCall +0x30 = Environment**. It never reads FnCall +0x10 (`this`).
3. Reads **Environment +0xe0 = target**, **target +0x20 = AS2 root**, then
   **root +0x10 = MovieImpl** at 0x1401b7000..0x1401b700b.
4. Tests **MovieImpl +0x228**, the user-event-handler pointer.
5. For a present handler and argc 0, leaves EAX zero and dispatches the event
   below through the handler's primary virtual **+0x08** at 0x1401b7096.

The exact event fields initialized at 0x1401b7072..0x1401b7087 are:

| Offset | Stored value |
| --- | --- |
| +0x00 | DWORD 0x16 |
| +0x04 | BYTE 0 |
| +0x08 | DWORD 0 |
| +0x0c | DWORD 0 for the zero-argument cursor index |

Bytes +5..+7 are padding not initialized by this callback; they are not claimed
to be zero. Dispatch arguments are **RCX=handler, RDX=MovieImpl, R8=event
pointer**. The handler's return value is not converted into an ActionScript
value. Native invocation wrapper **0x14018c5b0** retains the callback result
destination, and named CallMethod copies that result back to the operand stack.
The original script then pops it.

With argc >=1, a separate path obtains argument zero and calls
**0x1401a3390** before writing the index. Its coercion and range semantics are
not part of the initial zero-argument profile. The callback itself has no
renderer or extension-mode test; extension selection belongs to the preceding
member getter. It performs no direct cursor-state or Mouse-object write.

When the handler pointer is null, the callback takes **0x140186d10** with a
literal diagnostic stating that Mouse.hide failed because no user-event handler
is installed. This branch is not evidence that a cursor was hidden. Its logging
configuration and any broader native error handling are not implemented here.

## Implementation and host boundary

A bounded VM implementation can identify the actual C.hide function, require
zero arguments, retain the canonical deferred graph/current-namespace guards,
and form a typed host request using the active Environment target and its actual
root/MovieImpl association. The receiver does not select the handler. A captured
constructor context, method name, display owner or root with only a matching
numeric ID cannot replace that association.

The host request must have an explicit implementation or reject. Completing a
request as a no-op and returning Undefined would manufacture success. The VM
must not infer a cursor's previous/current visibility, an OS show-count, player
input state, a particular window or a default user-event handler from this
callback. Those effects and availability belong to the actual registered native
handler and the project's frontend/window bridge. The separate host study owns
that proof; this document ends at the dispatch ABI.

A successful supported host dispatch permits the callback's proved Undefined
return. Unknown handler availability, unsupported targets/windows and unresolved
host effects remain explicit gates. No rollback of a completed external cursor
effect is inferred from a later script error. The native result is set before
that effect; implementation error reporting is a bounded admission policy, not
an assertion that retail raises the same exception.

Fresh C/P construction also requires the actual cached parent/Array dependencies
or a fully supported factory route, safe ordinary publication and real resource
budgets. Cached builtin resolution, namespace publication and graph construction
must not leave a fabricated successful cache on an unsupported path. Unfinished
listeners, methods, reflection and mutation remain excluded.

## Evidence and provenance

Private base: `~/.local/share/marvel-heroes-forever-rust/evidence/`.
Owned executable SHA-256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Seven complete, exit-zero, untruncated, unchanged-artifact REA records are
retained in **`rea-mouse-hide/complete.bundle.json`**. The private original
checker matches selected contiguous instructions, tables, alias pointers and
literal bindings to the PE, verifies concrete getter/callback joins, and checks
the unchanged movie plus both controlled-probe receipts. **2,423 contiguous
instruction spans and 1,320 selected data bytes** were checked, with literal
and finite table assertions. It does not run a native process or original
ActionScript.

Canonical new anchors include factory `ev_6e35fff8...`, constructor/prototype
`ev_6cc46cd4...`, concrete tables `ev_d2a07ceb...`, complete hide callback
`ev_30c74594...`, normal getter `ev_5de2fda0...`, listener-retain/no-op/null leaves
`ev_d8174083...`, and correctly mapped .data alias slots `ev_d5f8f20d...`.
Full IDs, selected ranges and bundle hashes are in the sealed verification.
REA was exported and its target-free ledger cleared. No native launch, remote
access, cursor manipulation, renderer change or repository implementation edit
was performed by this study.

The frozen namespace/source review is
`rea-mouse-hide/namespace-source-review.json`, SHA-256
`22a8d5ed7dbc9bf3cab2f9d1ce715b2a640decb597940c2ff7daad7c16447621`.
The frozen shared graph review is
`rea-mouse-hide/shared-graph-listener-review.json`, SHA-256
`8186c483d4a28f77a5ae8b33a172c42ef56c1b96de015a287785604379d9a86e`.
The final `rea-mouse-hide/verification.json` SHA-256 is
`26238131e33099d7d6f3083100470ec7b3ca07db40a625c7446df1b3d57a6b8c`.
It binds both reviews, full canonical records, exact ranges, source hashes and
the private member profile. The separate actual-host proof is not incorporated
as a claimed cursor effect in this receipt.

Sources: Blizz's private executable and manager movie (proprietary owner install,
compatibility/interoperability study); original AS2_GLOBAL_BASELINE,
DEFERRED_BUILTIN_VALUES, REA_ATTACHED_PROPERTY_FLAGS, retained Stage listener
and Array analyses, and native graph analyses,
plus their retained REA evidence (original project analysis); installed REA skill
and tool schemas (owner tooling, no implementation code copied). No public Mouse
API description or external code was used as a substitute for this binary.
