# Fresh source MovieClip prototype selection

**VERIFIED static instructions:** fresh source-owner construction selects a
prototype through the current context's builtin cache and current global
`MovieClip.prototype`. It does not consult `Object.registerClass` in that step.
Registered-class prototype preparation is a later operation on the same display
owner, with separate direct, queued and late-lookup paths.

These are compatibility findings from the owner's build 1.52.0.1700 executable.
No native process, original ActionScript, all34 Construct replay, server or UI
was run. The all34 probe's explicitly supplied prototypes remain controlled
inputs, not recovered construction defaults. This increment changes no product
source and does not establish the actual login globals/registry at construction.

## Actual owner, context and definition roles

For the traced case-3 source target, let `T` be the target, `O=T+0x148` its
concrete AVM owner, `E=O+0x38` its embedded Environment, `R=T[+0x20]` its AS2
root, and `C=R[+0x50]` the root's context. These are recovered pointer roles,
not proposed Rust ABI layouts.

| State | Selection use |
| --- | --- |
| `O+0x18` | Selected display prototype; initially zeroed |
| `O+0x1d8` | Optional bound storage; zeroed before selection |
| `E+0xe0` | Actual target `T`; accessor conversion can use this receiver |
| `E+0xe8`, `E+0xf0` | Context and cached selected-MovieDef version, forming the supplied string-context data |
| `C+0x10` | Numeric builtin-prototype cache |
| `C+0x18` | Separate name-to-registered-function map |
| `C+0x20` | Separate lazy builtin factory/cache map |
| `C+0x28` | Associated MovieImpl; its `+0x18` reaches the root's interned builtin names |
| `C+0x60` | Current global object used by the selector |

Complete owner constructor `0x140203560..0x140203716` constructs the Environment,
binds its target/context/version, allocates and zeros the per-instance frame
ledger, then calls `0x14019b570(C, E, 6)` at `0x1402036b6`. The context supplied
directly to this call is reached through the same target/root path used for the
Environment's context. A common movie filename, constructor code origin or
numeric object ID in another VM is not that association.

The selected MovieDef supplies the cached version through `0x140099650`, target
virtual `+0x208` and MovieDef virtual `+0x20`. It remains distinct from the
definition supplying frame capacity through target `+0x110`, virtual `+0x50`.
The source/import binding conditions remain those in
[TEXTFIELD_SOURCE_ENV_VERSION.md](TEXTFIELD_SOURCE_ENV_VERSION.md); filenames
alone do not establish a selected native resource pair. The prototype selector
does not take a character/linkage name or perform that resource lookup itself.

After selection, `0x1402036bb..0x1402036f4` retains/stores the returned pointer
directly into `O+0x18`, including null. It does not call the later attached
prototype setter. Fresh storage was zero before the selector; arbitrary
factory/getter effects during selection are not proved to preserve that state.

## Exact initial selection rule

Complete selector `0x14019b570..0x14019b6df` has 98 instructions / 367 bytes.
Builtin 6 is `MovieClip`, root name offset `0x168`; `prototype` is builtin 77,
offset `0x3a0`. The adjacent name `__proto__` is builtin 78, offset `0x3a8`, and
is not the property read here.

First, `0x140194830(C, 6)` obtains the fallback. It probes numeric cache
`C+0x10`; on a miss it resolves the builtin name through `0x140194670`, then
reprobes that numeric cache. The fallback can be null. Resolver `0x140194670`
uses `C+0x20`: an existing nonnull cached function returns directly; otherwise
the factory is called, the map entry is reacquired and its result cached.
An absent entry returns null; a null factory result is not a permanent success.

**Order matters:** fallback resolution happens before the current-global read.
A global override therefore does not bypass an uncached builtin factory call,
and the global must be read after any such effects.

The two subsequent reads use actual object raw-member virtual `+0x58`, passing
the caller Environment's string context. They are not own-map-only reads.

| Current raw lookup/conversion result | Returned initial prototype |
| --- | --- |
| Global `MovieClip` read fails | Cached/resolved fallback |
| Global read succeeds but conversion yields null | Fallback |
| Converted global object exists, but its `prototype` read fails | Fallback |
| `prototype` read succeeds and conversion yields an object | That object |
| `prototype` read succeeds and conversion yields null | **Null**, replacing fallback |

The global value need not be callable: direct object conversion is sufficient.
The nonnull test after the second conversion controls reference retention,
not fallback selection. At `0x14019b679`, its result replaces the fallback even
when null. In particular, present Undefined/scalar/null prototype data must not
be treated as a missing property. An ordinary `get` API that collapses those
states cannot implement this rule.

Complete conversion helper `0x140199640..0x1401997e5` distinguishes native tags:

- Tag 6 returns the underlying object pointer; tag 8 returns the primary
  function-object pointer, separate from its retained frame reference.
- Tag 9 obtains the supplied Environment's current target/AVM receiver, calls
  accessor helper `0x14018dec0`, and recursively converts its returned value.
- Tag 11 resolves the retained name through `0x140197860` and that Environment's
  context. This can materialize a builtin and change context/global state.
- Other tags return null. This is not generic truthiness or primitive boxing.

Consequently the complete selector is not generally read-only. Arbitrary
accessor, factory, raw-getter/resolver, reentrant or failure effects remain
outside the proposed bounded implementation profile. The fresh-Environment
constructor checkpoint precedes these calls; its empty state must not be
promoted to the full owner's return state without that closure.

## Registered-class preparation is later

The registry is `C+0x18`, shared by the calling runtime root/context. The retained
modern-version lookup profile uses exact interned names; older-version folding
is unresolved. The producers first derive a linkage name through their selected
definition/resource map. This study does not establish that map's complete
loader alias/duplicate policy from saved export names.

| Established producer path | Prototype and later work |
| --- | --- |
| Initial hit in `0x1401d0f70` | Prepare the registered function's current prototype directly, then enqueue Construct and retained constructor records at priority 3 |
| Initial hit in `0x140203920` | Enqueue callback `0x14019ca30` at priority 1 to prepare the retained function's prototype; enqueue Construct and retained constructor at priority 3, with the documented conditional extra callback |
| Missing-class callback `0x1401cc220` later hits | Read the current context registry, prepare that function's prototype, then execute local Construct/constructor records; those two local records are not queued again |
| Missing-class callback still misses | Execute its event path without a registered constructor |

Initial hits retain the function/frame reference, not a future registry name
lookup or a frozen prototype pointer. Replacing the registry entry alone does
not retarget an already retained function. Mutating that function's `prototype`
before preparation can change the value subsequently read. Late lookup instead
observes the registry when its callback executes.

Complete preparation helper `0x14019c5e0..0x14019c679` raw-reads the retained
constructor's `prototype` using the target Environment's string context. A
failed read does not invoke the setter, leaving the current prototype intact.
A successful read calls `0x140199640` with a **null Environment**, then sends the
returned pointer, including null, to the attached interface setter `+0x68`.
This conversion argument differs from fresh selection. Direct object/function
and ordinary null/Undefined/scalar paths are established; accessors are not
admitted by the proposed profile. The tag-11 null-Environment branch returns a
null function reference without running a factory; no general getter/error
recovery behavior is inferred for this preparation path.

For this concrete owner, setter `0x1401898d0..0x14018998a` obtains/creates bound
storage first, forwards the prototype through that object's interface, then
replaces owner `+0x18`. The storage hook and its supported object categories
remain subject to the existing [attached-owner evidence](ATTACHED_MOVIECLIP.md).
This later mutation must not substitute for the fresh constructor's direct
initial store.

## InitAction and queue ordering limits

Fresh allocation zeros the instance's frame ledger before initial prototype
selection. That is not execution of an InitAction. The separate ledger helper
`0x14017baf0` marks accepted initializer-tag submission, while known DoInitAction
tag methods enqueue priority-2 action buffers. Neither the fresh selector nor
the ledger proves those bytecodes have run.

The normal queue drain orders pending priority-1 prototype callbacks before
priority-2 initializers and priority-3 Construct/registered constructors. Its
insertion-counter restart permits newly inserted earlier-priority work to
intervene. The construction path can instead use a token-filtered drain, which
leaves other-token records pending; some caller branches skip that drain.
Therefore “all InitActions ran before this Construct” is not an unconditional
consequence of the priority numbers. Missing linkage, argument-6 branches and
the version/argument-5 callback remain as recorded in
[REA_DISPLAY_LIFECYCLE.md](REA_DISPLAY_LIFECYCLE.md).

No complete scheduler trigger, constructor call, dynamic event delivery or
actual initial-login class state is established by this investigation.

## Minimal product seam

**OBSERVED current Rust boundary at `e9aecc4`:**
`create_attached_movie_clip[_with_binding]` requires the original global
MovieClip constructor and its original prototype, then initializes the owner
with `Some(builtin_prototype)` and no storage. That is a supported conditional
profile, not a general implementation of native selection.

No additional frontend wrapper was added: the existing API cannot faithfully
install a different initial selection. A create-then-`set_attached_prototype`
adapter would allocate storage and run a later-stage hook. `Vm::get` also
collapses missing and present Undefined, and the numeric builtin-prototype
identity is private VM state.

The smallest useful extension is a VM-validated initial-selection result plus
an allocation path that consumes it and stores `Option<ObjectId>` directly,
preserving all existing source/binding/owner/parent/root guards. Its first
bounded profile should require the correct single context, an already available
builtin cache and supported raw data reads; it must distinguish Missing from
Present Undefined and reject accessor/factory/resolver requirements explicitly.
It should preserve why fallback or the current global property was selected.
No public caller-supplied object ID alone should certify that selection.

A later registered-preparation adapter can use the existing context-owned
registry and attached setter only after it has an explicit producer checkpoint,
authenticated linkage selection, retained-versus-late function choice and a
raw presence-preserving prototype read. It must retain partial setter effects
on error and cannot schedule or execute the constructor. The fresh selection
and later registration path should remain separate APIs.

## Evidence and provenance

All binary material remains under the private evidence root. Original source
PE SHA-256: `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
This increment independently matched **731 instructions / 2,672 bytes across
nine complete ranges** against mapped PE sections and retained each matching
exception-directory range. The factory resolver spans six contiguous unwind
entries; those fragments were preserved. Three static name slots/literals were
also checked (24 pointer bytes and 30 bytes including terminators).

| Canonical retained REA record | Coverage used |
| --- | --- |
| `ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973` | Initial selector `19b570`, fallback cache `194830`, MovieClip name |
| `ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d` | Complete owner constructor `203560` |
| `ev_0c83febe079b3472a36975647fa99a31c1eb1b11ea55b137a933fca5efc108a4` | Conversion `199640`, factory resolver `194670`, map helper `18efa0` |
| `ev_13a35134d29edfd62e7aecc19583c957c3182e870d48cd99b5cf85505ec9f4ba` | Corrected complete 254-byte `197860`; earlier 302-byte over-read excluded |
| `ev_598fe04144afafa708e23ddbfcfe29fe4ca8a2aaaf87b83bdacdb7b24e8ea2f6` | Later registered preparation `19c5e0` |
| `ev_f135766caf5aa8db946bf5589308cd0908b6b0e8b5a868c1331ee0bd95c22a30` | Attached prototype setter `1898d0` |

REA imported and validated 21 existing canonical records from five bundles,
checkpointed them and reset with no target retained. No new disassembler or
native runtime was started. The first metadata summary request mistakenly
included a pagination limit; REA rejected it before analysis, and the corrected
summary request succeeded. That input error produced no target effects.

Private evidence is in `evidence/fresh-movieclip-prototype/`: `reuse-audit/`
contains the independent retained-source audit; `analysis/` contains selected
instructions, PE byte/unwind checks, name checks and the REA ledger checkpoint.
The sealed `analysis/receipt.json` has SHA-256
`b50ac7935fd9cc0f9ee272494d874e1840b75c64419816256fc5049fb9fc29bc`;
the 24-file `MANIFEST.json` has SHA-256
`04a165dc9724e7f39ce16d1093991662cce8de854a032e09724ba5cc4fbc931a`.
The independent reuse audit receipt is
`c026d9565946f0ddd294cba517fb6cf75d1c6493f49196898ad1d60a99419086`.

| Source read | Licence/ownership | Use |
| --- | --- | --- |
| Owner-private `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | Proprietary owner install; authorized compatibility study | Independent PE hash, section, instruction, unwind and name-slot checks |
| `evidence/rea-attached-movieclip/complete.bundle.json`, `rea-asflags-followup-version-proof/environment-owner.bundle.json`, `rea-external-interface/final.bundle.json`, `rea-class-registry/final.bundle.json`, `rea-display-lifecycle/complete.bundle.json` | Retained owner-binary REA observations | Canonical selector/conversion/owner/preparation/setter evidence; existing scheduling evidence |
| Project docs `ATTACHED_MOVIECLIP`, `ATTACHED_TARGET_INITIALIZATION`, `REA_CLASS_REGISTRY`, `REA_DISPLAY_LIFECYCLE`, `FRONTEND_INIT_LEDGER`, `REA_MOVIECLIP_CONSTRUCTOR`, `EXTERNAL_INTERFACE_BINDING`, `REA_SCRIPT_FUNCTION_CONVERSION`, `TEXTFIELD_SOURCE_ENV_VERSION`, `REA_ARRAY`, `FRONTEND_EVENT_DISPATCH`, `PROVENANCE`; targeted searches also included `REA_FUNCTION_CALLS`, `DEFERRED_BUILTIN_VALUES`, `REA_NATIVE_ALTERNATIVE` | Original project compatibility research | Established roles, ordering, names, references and limitations |
| `src/avm1/{attached.rs,vm.rs}`, `src/frontend/{class_registry.rs,init_ledger.rs,movie_binding.rs}` at `e9aecc4` | Original project source | Actual API boundaries and minimal extension proposal; no edits |
| Private `attached-target-initialization/base/{verify.py,selected-instructions.json}` and this increment's independent `reuse-audit/` | Original project analysis helpers/receipts | Existing PE-check pattern and owner/source anchors; original checker written here |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, targeted native-artifact guide passages, REA MCP schemas | Installed owner tooling; licence not inspected, no code copied | Retained-evidence routing, validation and checkpoint/reset |

No third-party implementation was copied. Static byte verification is not a
native execution comparison or proof of complete client startup behavior.
