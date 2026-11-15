# AS2 ExternalInterface binding

**VERIFIED static instruction/data paths** in the owner's pinned build
1.52.0.1700 establish a callable `flash.external.ExternalInterface` object,
its separate prototype, its own `call` member, and the lazy-resolution path.
**OBSERVED:** `call` selects a movie from the active invocation Environment;
constructing the namespace does not select an Unreal receiver or install the
movie's external handler. No native invocation, UI startup, or gameplay parity
was tested.

This is the binding needed by the original manager wrapper
`MarvelGlobals.GetLocalizedObjectDataByLabel`, whose body is `110249+64` and
whose ExternalInterface CallMethod is at offset 110311. The existing
[localization runtime study](FRONTEND_LOCALIZATION_RUNTIME.md) establishes its
callback name and argument order. This document supplies the native object
setup; it does not manufacture a complete movie lifecycle.

## Source and evidence

Primary source: the owner's proprietary client, read only for interoperability:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`.
SHA256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses below are preferred-image virtual addresses. No SDK implementation,
outside source code, binary bytes, disassembly, or extracted scripts enter git.

Canonical new REA output is private under
`~/.local/share/marvel-heroes-forever-rust/evidence/rea-external-interface/`:

| Record in `final.bundle.json` | Coverage |
| --- | --- |
| `ev_1babf1ae6b6d390b7a93ebfbdf326636e240b408c9eb09c32ed3b48f51199040` | Factory, method installer, prototype construction/registration, available getter, concrete tables/categories |
| `ev_0c83febe079b3472a36975647fa99a31c1eb1b11ea55b137a933fca5efc108a4` | Lazy resolver/cache, Value-to-object/interface conversion, complete constructor metadata setter, native invocation wrapper |
| `ev_9fac73b3f9ba1bb6420a1745dedf017afc520ae751c7ff4ce1c8a2bcd76ff4e4` | Prototype constructor getter, normal member helper, GetMember case and selected named-CallMethod branches, opcode table slots |
| `ev_128d5f2aafb9e6d92e8e114a4ebea498f1676e033e01e4a9bab069fa8d0be479` | Concrete ExternalInterface lazy-registration wrapper, prototype-parent setter, exact name literals |
| `ev_13a35134d29edfd62e7aecc19583c957c3182e870d48cd99b5cf85505ec9f4ba` | Exact 254-byte lazy-token helper boundary and interpreter entry's Environment argument store |

`verification.json` indexes receipt completion, byte agreement, unchanged source
hash, and reused evidence. `rea-external-interface-candidates/` contains bounded
ordinary discovery only; positive native claims use the canonical records.

Reused original project research (same private evidence root):

- `rea-as2-global-baseline/static-inventory/global-static-inventory.json` and
  `data.bundle.json`: namespace/global bootstrap identities. Its explicit
  ExternalInterface entry records bootstrap calls `0x14020fe1f` and
  `0x14020fe33`; the bootstrap selected REA record is
  `ev_42ed6f4a8b94be3333f5ecf801050c6be06c0132b30e2451950b53f0d6fcba3e`.
- `rea-object-prototype-inventory/namespace-parent.bundle.json`,
  `ev_20dafc3387ab17fb54e3ffcd1f8f934daad76380f44a9854b83c58c052ef029f`:
  namespace helper `0x1401d0850`.
- `frontend-localization-runtime/aggregate-followup/native/final.bundle.json`,
  `ev_32422746078bbb71afc0e97addc6d4bb8c5143c86b35ece3d56f31a4b42bfe69`:
  complete 269-byte constructor `0x1401f2170`, complete 675-byte `call` callback
  `0x1401c55d0`, and selected handler installation. The whole bundle and the
  existing localization document retain the receiver/state-cache evidence.
- `rea-array/prototype-property.bundle.json`,
  `ev_2ae57b7eadfd90dffc216f3a6bd56d5e5ff2ccc642fc8eeabb069f4088006256`:
  complete `0x1401b80f0` constructor/prototype link. Its *partial* selection of
  `0x1401a1560` is superseded here by the complete 347-byte read.
- `rea-movieclip-binding/final.bundle.json`,
  `ev_4d8736b4078ebc24320caf4f28cb6eb55dcbe094e6fabcda5c11f01111d787dd`:
  complete FnCall constructor `0x14019aba0`.
- `rea-attached-movieclip/complete.bundle.json`,
  `ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973`:
  cached builtin-prototype lookup `0x140194830`.

Only these original project analyses, their private helpers, the owned artifact,
and REA workflow/tool schemas were consulted. Rights remain those of the owned
proprietary artifact; original Rust/design prose is not copied implementation.

## Lazy namespace entry and creation trigger

Bootstrap `0x14020fe1f` creates/resolves the exact `flash.external` namespace
with helper `0x1401d0850` and retains it at global context `+0x48`. At
`0x14020fe33`, registration wrapper `0x1401f9970` associates builtin index 28
(root string slot `+0x218`) with factory `0x1401f2280`.

The concrete registration wrapper checks the context's name-keyed factory map
at `+0x20`. Only on an absent map entry does it insert the factory pointer with
a null cached result, then write a **Value tag 11** containing the interned name
token into the namespace's `ExternalInterface` member, using **member flags 0**.
Registration does not invoke the factory. The callback pointer is in the map,
not in the tag-11 string payload. Namespace flags are distinct from function
reference ownership flags and from the global object's later enumeration flags.

The resolution chain is concrete:

1. `0x1401997f0` converts a receiver Value to an object interface. For non-display
   values it uses `0x140199640` and then returns object `+0x20`.
2. `0x140199640` recognizes tag 11 and calls `0x140197860` with the current Env.
   That helper requires a non-null Env, extracts the name token, and calls
   `0x140194670` using **Env `+0xe8`'s context**.
3. `0x140194670` looks up the name through `0x14018efa0` in that context's
   `+0x20` map. A non-null cached function object is returned. Otherwise it
   invokes the entry's factory with the context, reacquires the map entry after
   the call, stores the returned function object in its cached slot, and returns
   it. A missing entry returns null. A null factory result is not a successful
   once-only cache; a later request can invoke the factory again.
4. The ExternalInterface factory itself publishes the resulting **Value tag 8**
   callable into `context[+0x48].ExternalInterface`, with member flags 0, and
   stores the separate prototype in the builtin-prototype cache under index 28.

The current Value being converted need not be rewritten from tag 11; the
factory updates the namespace property and the context cache. A retained token
subsequently resolves through the cache. This is per-context state, not a
process-global singleton or a constructor-code-origin namespace.

The interpreter connection is also grounded. The low-opcode table's GetMember
entry (`0x4e`, table slot `0x14020d484`) points to `0x14020b24d`; it converts
the receiver through `0x1401997f0` at `0x14020b270`. The string-member route
uses normal member helper `0x140194950`. Raw generic lookup can return the
tag-11 member unchanged; merely obtaining the token is not the factory trigger.

CallMethod (`0x52`, slot `0x14020d494`) points to `0x14020b6ef`. For a nonempty
named method and receiver tag 8 or 11, the branch at `0x14020bbae` converts
the receiver through `0x140199640` at `0x14020bbc8`, retrieves the method via
`0x140194950`, builds FnCall via `0x14019aba0`, and invokes the method's function
virtual `+0x50`. Thus the original `.call(...)` can resolve an unmaterialized
ExternalInterface token at this point. Selected named-call branches were read;
this is not a proof of every CallMethod mode or coercion.

## Fresh constructor, prototype, and own members

Factory `0x1401f2280` allocates a `0x70`-byte callable and calls
`0x1401f2170`. Its final primary table is `0x142515b00`, object interface
`0x142515a50`, internal category **23**. It stores native callback
`0x14018c560` at object `+0x68`; that callback clears its result to undefined.
This does not establish `new ExternalInterface` language behavior, which is
outside the supported invocation profile.

The callable's linked parent comes from cached builtin **Function**, index 7,
using `0x140194830` and `0x14018edf0`. The latter also installs the own
`__proto__` metadata marker. This direct cached-prototype path must not be
replaced with a guessed lookup of a possibly modified public `Function` global.

The factory separately allocates a `0xa8`-byte prototype, initializes it through
`0x1401e6f80`, and installs final primary table `0x1425164d0`, interface
`0x1425159a0`, metadata interface `+0x68 = 0x142515998`, category **24**.
Its linked parent is cached builtin **Object**, index 1. Helper `0x1401b80f0`
records the constructor reference and writes the callable's `prototype` property.
The method table supplied for the prototype at `0x14250e1f0` is empty: it adds
no prototype methods. `call` is installed on the callable object itself.

The fresh stored-property ledger, before script/host mutation, is:

| Object | Own key | Initial stored value / meaning | Member flags |
| --- | --- | --- | --- |
| `flash.external` namespace | `ExternalInterface` | Tag 11 name token; factory replaces it with tag 8 callable | 0 |
| ExternalInterface callable | `__proto__` | Tag 10 metadata marker; linked parent is cached Function prototype | 3 |
| ExternalInterface callable | `available` | Tag 10 marker; normal getter computes a Boolean | 0 |
| ExternalInterface callable | `addCallback` | Tag 8 native function, callback `0x1401fdf40` | 7 |
| ExternalInterface callable | `call` | Tag 8 native function, callback `0x1401c55d0` | 7 |
| ExternalInterface callable | `prototype` | Object value referring to the separate prototype | 3 |
| ExternalInterface prototype | `__proto__` | Tag 10 metadata marker; linked parent is cached Object prototype | 3 |
| ExternalInterface prototype | `constructor` | Tag 10 marker; native getter resolves stored constructor reference | 3 |

Flags are the native numeric property flags. They are not the Value tag or
function-reference flags. The established generic property machinery interprets
bits 1/2/4 as enumeration/deletion/write restrictions; the factory does not
apply flags 7 to the whole namespace or prototype.

Method installer `0x1401e9bd0` consumes 16-byte name/callback records from
`0x14250e200`, terminated by a zero name. There are exactly two records:
`addCallback` and `call`. The caller supplies shared flags 7. Each method is a
fresh native function (primary `0x1425150c0`, interface `0x142514430`, category
23), with its callback stored at `+0x68`, cached Function prototype as linked
parent, and a tag-8 function reference with null secondary frame and zero
reference flags. `addCallback`'s callback body is not implemented or claimed
by this study; its exact presence must not be represented by a success stub.

The prototype's raw getter `0x1401e7c40` routes public `constructor` through
`0x1401a16c0`. For the fresh marker, this returns the stored ExternalInterface
constructor as tag 8. A replaced ordinary property value takes a distinct
branch. The proof therefore supports the fresh graph, not arbitrary metadata
rewrites or a universal `constructor` alias.

## `available`, receiver, and movie handler are separate contracts

The callable's normal getter is `0x14018c440`. For exact `available` in
version >6, it returns a Boolean testing whether
`Env.target[+0x20][+0x10][+0x238]` is non-null. Older versions use normalized
name comparison. Other keys go to the object's raw getter `0x140197000`.
The special `available` branch is selected by name and current Env; it does not
read a stored Boolean. Raw inspection of the fresh own slot still sees tag 10.
Mutation/deletion behavior of this special property is not flattened into an
ordinary data-property model here.

For the original named method call, FnCall's `this` is the resolved
ExternalInterface callable interface, and its Env is the active caller's Env.
The concrete argument trail is:

1. Interpreter entry `0x140208ef0` receives Env as its second argument, RDX;
   `0x140208f30` copies it to RSI and `0x140208f33` saves it at `[rsp+0x48]`.
2. The selected named-CallMethod route reloads that Environment slot into R9
   at `0x14020bc39`, then calls FnCall constructor `0x14019aba0` at
   `0x14020bc6a`.
3. That constructor retains the supplied Env pointer at FnCall `+0x30`
   (`0x14019abf8`). Independently, it resolves the receiver interface into
   FnCall `+0x10`. Retaining a receiver function reference does not replace Env.

Here, "active caller" means the interpreter Environment executing this
CallMethod instruction. It does not mean the original outermost caller or
the caller function's code-origin movie. How a preceding script-function call
constructed that interpreter Environment, including captured targets or
cross-movie calls, remains a separate contract. The native method does not
substitute an Environment captured when its function object was created.

The native function invoker `0x14018c5b0` has a separate Super-wrapper route.
ExternalInterface's interface virtual `+0x78` returns false, so the expected
receiver takes the direct callback path with the same FnCall. `0x1401c55d0`
itself does not inspect
FnCall's `this` field to choose a movie or require an ExternalInterface instance.
Starting at `0x1401c55e1`, it obtains:

`FnCall[+0x30] -> Env[+0xe0] -> target[+0x20] -> AS2Root[+0x10]`.

The handler comes from **that MovieImpl `+0x238`**. Consequently the bridge
must use explicit current movie context; neither the native function's creation
context, a source-movie filename, a constructor origin, nor the engine receiver
can silently stand in for it. Arbitrary detached/Super calls are outside the
small supported profile even though this callback has no receiver-type check.

Lazy-factory lookup and handler selection follow different fields: the former
uses Env `+0xe8`'s context; the latter uses Env `+0xe0`'s target. The callback
does not compare them or derive the movie from the factory cache. The ordinary
attached-Environment construction and its root association are grounded in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md); this study does not establish
that arbitrary reassignment or cross-movie execution preserves that association.
An implementation therefore needs the current invocation's explicit target,
context and movie/root association. A fixed host root can admit a known matching
invocation; matching `this` or a source origin alone cannot establish it.

The already retained complete callback shows:

- No cached handler: take its diagnostic-helper branch and set the script
  result to undefined.
- With a handler: convert the first argument through the native string helper
  as callback name; remaining arguments are converted in original order into
  external values through `0x1401b4d60`. Zero supplied arguments use an empty
  name and zero payload count. This is not a strict universal two-string API.
- Clear the selected MovieImpl's AS2 root (`MovieImpl +0x18`) result slot
  `+0x58` to undefined before invoking handler virtual `+8`
  with MovieImpl, name, converted arguments and count; afterward copy that root
  result into FnCall's result destination and release temporary values.

The original localization wrapper's actual admitted case is callback name
`GetLocalizedObjectDataByLabel` plus one string label. It does not justify
silently coercing arbitrary labels or implementing all external value types.
Nested/reentrant handler calls, exceptions, argument allocation failure, and
large argument counts are outside this explicit profile.

The state-6 handler installation and movie state-cache update are separately
proven in [FRONTEND_LOCALIZATION_RUNTIME.md](FRONTEND_LOCALIZATION_RUNTIME.md).
The engine dispatcher `0x141264c20` obtains a selected engine-side receiver
through movie user data `+0x124`, resolves the named reflected function, converts
arguments, invokes it, and converts its result. A non-null state-6 handler does
not prove that receiver or callback exists. The original saved initialization
that selects the concrete receiver remains unproven. Namespace construction
must not invent a default `MarvelGFxMoviePlayer` or use the AS2 callable as that
receiver. The fresh returned localization object is a further independent
contract in [LOCALIZED_RESULT_OBJECT.md](LOCALIZED_RESULT_OBJECT.md).

## Small supported explicit binding

**INFERRED implementation boundary:** an owner-controlled runtime can expose
this native binding with an explicit AS2 root context and live movie/Env identity,
the known namespace/factory state, and a separately configured external-handler
binding. Preserve the native callable and prototype identities and the own
property ledger above. Resolving a lazy token may materialize/cache that graph;
it must not claim to have run movie startup or selected the engine receiver.

For the saved localization call, admit the expected callable receiver, active
movie context, exact callback name, one string label, and a supported explicit
handler/receiver profile. Route its result through the already grounded result
object conversion. Keep `available` tied to handler presence, while exposing an
unconfigured engine receiver as a separate unresolved boundary. Unsupported
`addCallback`, generic external conversions, arbitrary metadata changes, and
unresolved handler lifecycle must be reported explicitly, not filled with
successful no-ops or fabricated localization data.

Five new REA records completed with exit 0, untruncated output and unchanged
executable hash. The private verifier checks 2,196 selected instructions,
648 raw bytes and eight reused receipts. The initial 302-byte selection of
`0x140197860` included its complete 254-byte body plus 48 bytes of padding and
an unrelated partial next function; that excess is excluded explicitly, and
the fifth record independently captures the exact helper boundary. Selected
interpreter branches are distinguished from complete helpers; two padding bytes
after the concrete registration wrapper are retained as padding, not extra
behavior. No Cargo tests, native execution, live handler
installation, or renderer smoke was run for this documentation-only increment.
