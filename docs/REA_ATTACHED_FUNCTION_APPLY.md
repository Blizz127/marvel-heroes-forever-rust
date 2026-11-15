# Function.apply with a live attached MovieClip receiver

**VERIFIED static native paths:** the pinned client's `Function.apply` accepts a
public display Value as its explicit receiver, resolves it to the attached
owner's object interface, and forwards that interface through an ordinary
function invocation. For a live case-3 MovieClip, Function2 entry converts the
same owner interface back to **Value tag 7** for both preloaded and named `this`.
It does not expose the owner or lazy storage as an ordinary script object.
The function's selected Environment remains a separate identity.

The bounded Rust consequence is to preserve `Value::DisplayClip(clip)` when
applying an ordinary ScriptFunction2 to an admitted live clip. This is not
permission to treat Apply as construction, supply a constructor context, or
bypass the independent Super-preload gate.

## Owned script dependency

The manager movie is the owner's retained
`evidence/frontend-startup/MarvelGame-frontend_manager.gfx`, 2,055,748 bytes,
SHA-256 `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
An independent bounded header parse confirms:

| Definition | Body | Function2 flags | Registers / parameters |
| --- | --- | --- | --- |
| `MarvelUtils.registerClass`, 855859 | 855879 + 41 | `0x2a` | 3; `mc` in r1, `cls` in r2 |
| Applied `MenuManager`, 1993628 | 1993639 + 35 | `0x19` | 3; no parameters |

The original registerClass body first assigns `mc.__proto__`, then calls global
`Function(cls)` at 855912 and its `apply(mc, null)` at 855918. Its body digest is
`3f206a92aed74019c2c1d37e81e7429d60399c5f8371bd91a85acb794db1b7dd`.
The applied body's digest is
`e504fa451e66d9831805d72df36c30a7b1240422897edfb3b6f2e9aae838a711`.
No original body was patched or executed by this study. The containing-frame
probe and its runtime stop belong to the VM integrator's separate evidence.

## Display value to forwarded receiver interface

The already identified AS2 apply callback is `0x14019cf00`. Its complete body
ends at `0x14019d379` exclusive. With a first argument present, it calls
`0x1401997f0(value, activeEnv)` at `0x14019cf91`.

The complete converter `0x1401997f0..0x14019987b` has a distinct tag-7 path:

1. Require nonnull Environment and input public handle at Value `+8`.
2. Read the caller Environment target at `+0xe0`, then its AS2 root association
   through target `+0x20`, association `+0x10`.
3. Call `0x14009a540` with the input handle and that association. Its live
   shortcut returns nonnull handle `+8` directly. A null live pointer enters a
   resolver path; this profile excludes it.
4. Require resolved target `+0x6a` bit 7. Recover its owner using
   `target + 4 * byte[target+0x6d]`, call owner primary virtual `+8`, and return
   that result plus eight bytes.

For the proved case-3 owner, primary virtual `+8` is `0x1405b93f0`, which returns
its receiver unchanged. Therefore conversion yields **owner +8**, the attached
object interface at table `0x1425175c0`. Its category method `0x140072380`
returns **2**. This is distinct from the category-11 lazy storage interface.
[Attached MovieClip evidence](ATTACHED_MOVIECLIP.md) supplies the factory,
owner binding, target field, and storage distinction.

Apply retains this interface separately. Its category-2 path reads the actual
target from interface `+0x18` (owner `+0x20`) and increments target `+8` at
`0x14019cfcf`. Cleanup releases that retained target through `0x140031cc0` at
`0x14019d346`. This establishes an invocation-lifetime hold, not a complete
retirement or rebinding implementation.

At `0x14019d13d` or `0x14019d211`, Apply writes the resolved receiver interface
to forwarded **FnCall +0x10**. It preserves the incoming Environment in forwarded
FnCall `+0x30` and invokes the selected function's primary virtual `+0x50` at
`0x14019d14c` or `0x14019d22f`. Neither route is an allocation/constructor call. Both set the method-hint
argument R9 to zero; this does not establish a construction context.
For the explicit null or undefined second argument, `0x140199640` returns null;
Apply takes the zero-forwarded-argument route. [The prior Apply study](REA_FUNCTION_CALLS.md)
already establishes that list conversion and return forwarding.

## Forwarded interface back to script-visible this

The actual ScriptFunction primary table `0x142513140`, slot `+0x50`, points to
`0x14020e190`. That invoker passes its invocation context to frame preparation
`0x140205090`. Preparation loads FnCall `+0x10` into r14 at
`0x140205123..0x140205134`.

Before using the receiver, preparation tests its object-interface virtual
`+0x78` at `0x1402052a1`. The case-3 slot at `0x142517638` points to
`0x1405b93b0`, a false-return leaf. The branch that replaces r14 with a Super
wrapper's underlying receiver is therefore not taken for this owner. r14
continues to designate the same attached interface supplied by Apply.

**PreloadThis:** the Function2 path at `0x140205d02..0x140205dbb` starts the
preload register index at one and tests flag `0x01`. For a nonnull receiver it
calls `0x1401937f0(destinationValue, r14)` at `0x140205d4f` or
`0x140205d68`, depending on register-storage capacity. The complete helper
`0x1401937f0..0x140193895` recognizes category 2, reads interface `+0x18` to
recover the target, and reads target `+0x70` for its public handle. It writes
**tag 7 at `0x140193853` and that handle at Value +8 at `0x140193856`**.
The register is a public display value, not the owner-interface pointer.

**Unsuppressed named this:** flag `0x02` controls suppression. The branch at
`0x140205dcc..0x140205e5b` similarly reads category 2, target, and target `+0x70`.
It writes tag 7 at `0x140205e2e` and the handle at Value `+8` at
`0x140205e32`. The following span through `0x140205fa4` places that Value in
the local-frame name slot at builtin-context offset `0x320`. The retained
builtin table layout plus a separately labeled ordinary PE pointer/literal
check establishes that slot as `this`: table index 61 at `0x142add4c8` points
to `0x14250bff8`. That pointer check is not presented as a new REA record.

Both paths can create a public handle through `0x14009e770` if target `+0x70`
is null. The admitted profile instead requires the existing coherent handle;
handle allocation and failure are outside this implementation consequence.
The applied function's flags `0x19` take both PreloadThis and unsuppressed-this
paths. Its separate PreloadSuper bit remains a distinct requirement.

## Environment and identity requirements

ScriptFunction invocation selects its Environment through primary virtual
`+0x48` (`0x14017f710`) before frame preparation. Its stored public target
handle at function `+0x70` and association at `+0x68` are independent of the
receiver. Code is separately stored at `+0x78`. The live captured-target route
recovers that target's owner Environment. Invocation can substitute caller
FnCall `+0x30` when selected target `+0x6a` bit 4 is set. These routes and their
bounds are documented in [the source/Environment proof](TEXTFIELD_SOURCE_ENV_VERSION.md).
Changing `this` must not silently replace the captured function target.

The bounded Apply admission requires all of the following:

- Exactly two explicit arguments: a proved live case-3 display receiver and
  null or undefined list; the selected callable is an ordinary ScriptFunction2.
- A valid active caller Environment, its target and root association. The
  receiver converter dereferences those fields before calling the live-handle
  helper, even though that helper's successful shortcut ignores the association.
- A canonical coherent receiver public handle: Value handle equals target
  `+0x70`, and handle `+8` points to that same live target; target bit 7 is set,
  its owner binding is valid, and no retirement, rebind, replacement or unsupported
  receiver transformation occurs during this invocation.
- The existing supported captured-function Environment and closure profile.
  Preserve captured target/global scope rather than inferring it from the
  receiver or code-source movie. Initial admission requires captured scopes
  exactly `[target, global]`, matching the admitted `Function(cls)` conversion
  profile; captured activations and With scopes are not admitted. Auxiliary
  function-reference wrapper state must not be inferred to survive extraction.
  The no-rebinding and bit-4/caller-fallback boundaries remain intact.

The initial VM implementation may further require caller, captured target and
receiver to share one adopted root. That is a conservative admission restriction;
the native live-handle shortcut itself does not compare their root identities.

The handle condition matters: conversion resolves the input handle to a target,
whereas script entry regenerates `this` from that target's canonical handle.
Same display-value identity follows from their coherence; it is not a theorem
about arbitrary crafted tag-7 wrappers. The existing typed attached clip identity
can represent the admitted coherent pair without exposing native pointers.

TextField/category 4, other display categories, dead/null/noncanonical handles,
primitive/null receivers, Function.call, general argument arrays, native/Super
callables, legacy DefineFunction, arbitrary captured-frame wrappers, and context
failures remain outside this extension. Native category checks sometimes admit
more branches; their presence alone does not prove those broader profiles.
[Function conversion evidence](REA_SCRIPT_FUNCTION_CONVERSION.md) also retains
its distinction between callable object identity and auxiliary wrapper state.

## Evidence, checks, and provenance

Only the owner's pinned executable was used, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
48,544,080 bytes, under
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`.
Private receipts are in `evidence/rea-attached-function-apply/` beneath the
same private project root. `verification.json` SHA-256 is
`08d421613f556b91d27d0a32402159c3f89997b830263da34e2ca0c8c6c45b45`.

The original `verify-retained.py` checks selected instructions and tables against
PE section mappings: **11 canonical REA records, 1,020 instructions, 3,693 unique
instruction bytes, and 312 table bytes**. All selected records report complete,
exit-zero, untruncated results and unchanged artifact hashes. Exact ranges,
full record IDs and source-bundle hashes are private in that receipt. Whole
frame-preparation completeness and unselected trailing partial decodes are not
claimed. No new REA engine session or native execution was needed; retained REA
results were reused following the toolkit's evidence-first workflow.

| Retained bundle | Principal proof |
| --- | --- |
| `rea-function-calls/function-binding.bundle.json` | Complete Apply (`ev_7316e...`), receiver converter (`ev_aa26b...`), interface-to-Value helper (`ev_ceb0b...`) |
| `rea-movieclip-binding/final.bundle.json` | ScriptFunction constructor/table (`ev_a3814...`), invoke (`ev_34ee9...`), selected frame-preparation spans (`ev_704ba...`), false leaf (`ev_c30fe...`) |
| `rea-attached-movieclip/complete.bundle.json` | Case-3 interface/category and non-Super slot (`ev_a2567...`) |
| `source-version-env/complete.bundle.json` | Captured-target Env getter (`ev_fe0ed...`), live-handle shortcut (`ev_a35bb...`), case-3 owner/Env leaves (`ev_02263...`) |

Outside-source ledger: the owned executable/movie are proprietary copyrighted
artifacts, used privately for interoperability; no target bytes or disassembly
are committed. Existing project documents and REA records above are original
project research. REA 6.9.0 skill/guide at
`~/.agents/skills/reverse-engineer-anything/` supplied workflow and evidence
handling, with its documentation licence not established here. No external VM
implementation or SDK source was consulted or copied. The earlier Function
reference sources retain their own documented licences; this study adds no new
web source. No runtime behavior, full-method success, Super behavior, rendering,
or retail parity is claimed by these static findings.
