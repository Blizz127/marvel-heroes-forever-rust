# Ordinary applied Function2 `super` preparation

Task `a3d58c579542`, 2026-11-15 UTC. **VERIFIED static control flow:**
`Function(cls).apply(attachedRoot, null)` reaches ordinary ScriptFunction
invocation, and that invocation can prepare `super` without a construction flag.
For this null-method-name route, superclass selection starts from the actual
receiver interface's current prototype. It does not automatically start from
`cls.prototype`, a recorded class relation, or the receiver's own hidden
constructor property.

Nested unnamed `super()` calls preserve two distinct things: the attached
receiver used for `this`, and the prototype from which the next superclass is
selected. Collapsing those into only a receiver/class pair can repeat the first
base constructor. No client executable or original native function was run.
The evidence establishes this bounded route, not arbitrary method dispatch.

## Original caller and evidence boundary

**OBSERVED owned movie:** the saved manager function has definition offset
1,993,628, body 1,993,639 + 35 bytes, Function2 flags `0x19`, three registers and
zero parameters. The first Push supplies argument count zero, register 2 and an
Undefined method name; CallMethod is at 1,993,654. Body SHA-256:
`e504fa451e66d9831805d72df36c30a7b1240422897edfb3b6f2e9aae838a711`.
With preload-this `0x01`, suppress-arguments `0x08` and preload-super `0x10`,
the relevant preloaded values are this in register 1 and super in register 2.
Suppress-super `0x20` is clear, so the named local super is also installed.

The AVM1 lane identifies this saved function as MenuManager and reports the
preceding original `Function(cls).apply(attachedRoot, null)` call at 855,918.
That lane's actual frame and receiver-prototype state are a separate execution
witness. The bytes above alone do not prove which function/receiver a native
run would select. In particular, MenuManager's class relation may be used only
after checking the actual current receiver prototype and its hidden binding.

The private movie is `evidence/frontend-startup/MarvelGame-frontend_manager.gfx`,
SHA-256 `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
The only native artifact studied is the owner pin at
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
build 1.52.0.1700, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses below are preferred-image virtual addresses; field offsets are
hexadecimal.

## Apply invokes normally with a null method-name hint

Existing [Function conversion](REA_SCRIPT_FUNCTION_CONVERSION.md) evidence
binds the actual ScriptFunction primary table's invoke slot `+0x50` to
`0x14020e190`. Conversion keeps that function object but supplies a null
auxiliary frame reference; it does not turn the call into construction.

**VERIFIED:** the complete apply callback at `0x14019cf00..0x14019d379`
sets R9 to zero at both `0x14019d100` and `0x14019d1da` before invoking the
selected function's primary `+0x50`, at `0x14019d14c` / `0x14019d22f`.
There is no intervening call that supplies a different R9. The forwarded
FnCall stores the converted receiver interface at `+0x10`; its Environment
at `+0x30` remains the caller Environment. The attached-receiver conversion
and reverse-this bridge are the separately scoped coercion-lane proof.

Script invoke `0x14020e190` builds an invocation state with these fields:

| State offset | Source |
| --- | --- |
| `+0` | Invoked ScriptFunction object |
| `+8` | Selected execution Environment |
| `+0x10` | Incoming FnCall |
| `+0x18` | Incoming auxiliary frame argument R8 |
| `+0x20` | Incoming method-name hint R9; null for this apply route |

At `0x14020e232` it calls frame preparation `0x140205090` after its recursion
limit check. This is the same call for ordinary application and shared-New
invocation; no construction-mode branch guards frame preparation or the
Function2 super block. Captured-target Environment selection remains governed
by [the existing source/Environment proof](TEXTFIELD_SOURCE_ENV_VERSION.md).
The selected Environment is not inferred from `this` or code-file origin.

## Current receiver prototype selects the first Super

Frame preparation loads FnCall `+0x10` into R14 at `0x140205130`. At
`0x140205208..0x14020520b`, it saves that original interface separately at
stack location `[RBP+0x67]`. Only afterward does it test the interface's Super
predicate `+0x78`; if true, R14 becomes interface `+0x58`, the stored actual
receiver. The saved interface is not changed by that unwrapping.

The attached case-3 MovieClip owner interface is `0x1425175c0`, category 2.
Its `+0x78` slot points to the false leaf `0x1405b93b0`, so the first apply
call leaves R14 unchanged. Owner `+0x18`, equivalently interface `+0x10`, is
the current display prototype. This is the owner/interface field, not the
lazy script-storage object's identity; see [ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md).

For Function2, `0x140205b1b..0x140205b30` enters super preparation when
preload-super is set **or** suppress-super is clear. In the selected `0x19`
case it enters through preload-super. The relevant successful sequence is:

1. `0x140205b36..0x140205b41`: read the saved incoming interface's `+0x10`
   prototype object into RBX, here called **P**; a null P skips construction.
2. `0x140205b5d..0x140205b65`: inspect invocation-state `+0x20`. The proved
   null apply hint takes the direct path to `0x140205c21`. The nonnull hint
   searches a prototype chain for a named member and is a different profile.
3. `0x140205c21..0x140205c39`: call P's interface virtual `+0x70` with the
   selected Environment's string/global context and a FunctionRef destination.
4. `0x140205c46..0x140205c65`: allocate a 0x98-byte Super object; on successful
   allocation pass P's linked parent `[P+0x30]`, the actual R14 receiver, and
   the selected FunctionRef to constructor `0x1401da160`.

For the bounded class-prototype path, P's interface is `0x142513cd0`.
The exact table slots bind `+0x70` to `0x14019ad30` and `+0x58` to
constructor-aware getter `0x1401e7c40`. The first helper asks that getter for
builtin name offset `+0x3b8`, hidden `__constructor__`. It accepts **only tag
8** as a FunctionRef. Missing or other-tag results produce a null reference;
a tag-11 deferred value is not materialized by this helper.

Hidden and public constructor metadata remain separate. Getter
`0x1401e7c40` distinguishes name `+0x3b8` from public name `+0x3b0`.
Helper `0x1401a16c0` first admits an own non-marker value, including Undefined.
Only the marker/missing path uses stored constructor metadata; a null stored
reference follows the linked parent with the same selected name. Consequently
an own Undefined hidden binding shadows inheritance rather than falling back
to a class relation.

The original Extends path installs an **own tag-8 hidden property with flags
3**, through `0x14018ec90` and the generic raw setter. The class-prototype
constructor initializes its internal public and hidden FunctionRefs to null.
Extends does not populate that internal hidden field, and it does not replace
the public constructor. These facts distinguish this selection from the
public-constructor allocation step in [the New study](REA_MOVIECLIP_CONSTRUCTOR.md).

## Super state, forwarding and nested progression

The complete Super constructor `0x1401da160..0x1401da23f` preserves these
distinct fields. Let **Q = P's linked parent** and **R = actual receiver**:

| Super object field | Stored meaning |
| --- | --- |
| `+0x30`, also interface `+0x10` | Q, the linked prototype |
| `+0x68` | Q retained for Super lookup bookkeeping |
| `+0x70` | Null for the fresh wrapper |
| `+0x78`, also interface `+0x58` | R, the actual incoming receiver interface |
| `+0x80` | Hidden-constructor FunctionRef obtained from P; auxiliary association handled separately |

Its interface is `0x142511cb0`; category slot `+0x10` returns 6,
Super-predicate slot `+0x78` returns true, and constructor-reference slot
`+0x70` is `0x1401da660`, which returns the stored FunctionRef.
The selected successful constructor path requires a nonnull Q; the constructor
dereferences it. This study does not generalize null-parent or failed-allocation
behavior into a working Rust Super context.

At `0x140206274..0x1402062c9`, preload-super places this object into the next
register using `0x1401936f0`. For the observed flags the preceding this preload
has advanced the register index to 2. With suppress-super clear, the later
block also writes the same Super object to local name `super`.

The unnamed CallMethod handler, starting at `0x14020bcc2`, identifies a Super
object by its predicate and retrieves its stored constructor reference through
virtual `+0x70`. It separately supplies the Super object as this. FnCall helper
`0x14019aba0` converts that tag-6 object to its own interface; it does not unwrap
the Super. At `0x14020be55`, the handler again supplies a null method-name hint
and invokes the superclass function's primary `+0x50` at `0x14020be68`.

If that superclass is another ScriptFunction, its new frame saves the incoming
**Super interface** for prototype selection, then unwraps only R14 to R. Thus:

```text
first ordinary application: P = R.current_prototype
first wrapper:              hidden(P), next prototype Q = parent(P), receiver R
next script super frame:    P = incoming_super.linked_prototype = Q
next wrapper:               hidden(Q), next prototype parent(Q), receiver R
```

This is a derived composition of the verified field assignments and branches.
It does not mutate R's current prototype merely to advance superclass dispatch.
Preload-this uses the actual R through `0x1401937f0`. For the attached category-2
receiver, the separate retained bridge returns the same target's public tag-7
display handle. Do not replace that value with the lazy storage object or a
generic ObjectId.

If the selected base is the separately bound original native MovieClip callable,
its complete wrapper `0x14018c5b0..0x14018c6d9` tests the incoming Super predicate
at `0x14018c5e3`, loads its actual receiver from interface `+0x58` at
`0x14018c603`, and puts that receiver into a temporary FnCall at
`0x14018c61a`. The result destination, Environment, argument count and argument
index are preserved; the stored callback is invoked at `0x14018c642`.

The complete callback `0x1401955c0..0x14019561f` queries that actual interface's
category and compares it with **11** at `0x1401955da`. The attached owner's
verified category is **2**, so it takes `0x14019560a`, cleans the prior result,
and writes tag zero (**Undefined**) at `0x140195616`. It does not allocate,
write receiver/display properties, inspect arguments or require construction
mode. It does not return a category-11 lazy storage object or unbound instance.
Subsequent Super cleanup sees its fresh null `+0x70` field and leaves the saved
lookup bookkeeping unchanged.

This terminal outcome is a verified static branch composition conditional on
selecting the original native MovieClip callable and the live attached owner.
A successful New allocation instead supplies category 11 and takes the
same-object return branch. This study does not assert that an actual native
MenuManager run reaches either terminal without its class/prototype witness.

## Implementation admission and limits

**INFERRED bounded design:** an ordinary apply route can prepare a distinct
Super dispatch value carrying the selected prototype progression, the exact
hidden callable reference, and a typed actual receiver Value. It must validate
the current owner prototype/parent/hidden binding; a class-base cache may check
those identities but cannot replace their lookup. Preserve the incoming Super
interface's prototype separately from the actual receiver when entering a
script base. The applied function's identity is not a substitute for that state.

A narrower first implementation may additionally require P to equal the current
function's recorded Extends prototype, preserve the original flags-3 data
descriptors, and check original class/parent function.prototype links. These
are conservative admission checks; the native null-hint super path does not
perform those public-link checks or require a particular property-flags value.
They must not replace reading the actual selected cursor and hidden binding.

Admit only the proved null hint, unchanged supported class-prototype getter,
nonnull parent, direct callable hidden value, live attached receiver bridge,
and separately established active Environment. Keep missing/nonfunction/deferred
hidden references, arbitrary plain/proxy/native prototype kinds, nonnull hints,
named-super methods, captures not represented by the VM, graph mutation,
allocation failure and target retirement explicit unsupported boundaries.
Neither a New-only context restriction nor assigning the applied class as an
unverified context reproduces this ordinary apply route.

## Retained evidence and validation

All evidence below is private under
`~/.local/share/marvel-heroes-forever-rust/evidence/`. The original checker
`rea-applied-super/verify-retained.py` parses PE sections and independently
matches **3,462 complete instruction spans and 464 table bytes** to the pin.
Its `verification.json` lists exact record IDs, bundle hashes and end-exclusive
ranges. All selected producer records are complete, exit-zero, untruncated and
unchanged-target observations. Original function-header/body checks are included.
Padding and neighboring paths are not used as semantic proof.

| Retained bundle / primary record | Selected proof |
| --- | --- |
| `rea-function-calls/apply-entry.bundle.json`, `ev_7316eaaa5f9553e5f89d3ec0dcdeb39bcd416dd6dfef009b4b497f9aec9d1178` | Complete apply callback and null R9 hints |
| `rea-movieclip-binding/final.bundle.json`, `ev_34ee943aaf37af1a7e8a34152742f96ac3ca4e83a880f46578bbd98a338ec26d` | Complete ScriptFunction invoke |
| Same bundle, `ev_704babdfcb3aea3b289230706c1b4bcae3a26072f95048b7bb3e96e8ad34ee37` | Complete frame preparation, saved receiver, flags, prototype selection and preloads |
| Same bundle, `ev_2e1f1b1cf76e2c7855dc69e56a9495b02cd851504e8d20bd6ec2e524d4994646` | Hidden-constructor reference getter |
| Same bundle, `ev_4d8736b4078ebc24320caf4f28cb6eb55dcbe094e6fabcda5c11f01111d787dd` | Super constructor, table, stored-reference getter and FnCall construction |
| Same bundle, `ev_c30fe045f6816838418671b866ce0831b4cd4ac3db79f382c18e2b07511de525` | Complete CallMethod handler, cleanup and false predicate |
| Same bundle, `ev_2b9a38a649ee1f149e3c0b78ea965ce1d2fa965f9175ea193db5d982a187fb7e` | Value-to-interface conversion without Super unwrapping |
| `rea-function-calls/function-binding.bundle.json`, `ev_ceb0b6b4241d0b1b4625bb26bb29866a83de58568e4eac93b099ca54166e5092` | Object/display result conversion and category-6 leaf |
| `rea-attached-movieclip/complete.bundle.json`, `ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973` | Attached category and Super-predicate slot |
| `rea-native-alternative/asflags-scope-final.bundle.json`, `ev_b86d375ba8c9ca1cdfd66172094d00fc60a72a691fd09a3e93dbfebeddf0b700` | Actual class-prototype constructor/table binding |
| `rea-script-allocation/final.bundle.json`, `ev_9fd9990217da498c36df2a72340d70e064e28785765b56d6ddb7fca0b475f167` and `ev_e6e7458caf6984be916e740ee98507d14876e1fc97ed2b9b494626dd79325f44` | Constructor-aware raw getter, own-value precedence and same-name inheritance |
| `rea-native-alternative/extends-baseline-final.bundle.json`, `ev_b1581a5e345f23786489233d0206cad994918ebfca3304e02b1d2abea26eef0f` | Exact hidden setter's tag, name slot and flags |
| `rea-asflags-followup-return/final.bundle.json`, `ev_baa1bc0e27483ab8df8eee94de03d3795c651dc840a70de0902a005289939e72` | Complete native invocation wrapper and actual receiver forwarding |
| `rea-movieclip-constructor/callback-final.bundle.json`, `ev_c3e031f37a7c9e8d5a56ef3da74939f9d25c2b9c9c688ced4e6955e2c5f5debc` | Complete MovieClip callback; attached category-2 terminal result is Undefined |

No fresh native REA analysis was necessary: existing complete records answer
this question. One REA retained-view request reported that execute-rizin-command
records have no supported analysis view; its remedy was followed by reading the
original complete exported records directly. No retry, engine session, target
execution, shared configuration change or reset of another lane occurred.

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned executable and private manager movie | Proprietary owner-held artifacts; private compatibility study | Static instruction/table validation and exact caller header/body facts |
| Existing project documents linked above, `REA_FUNCTION_CALLS.md`, and named private REA receipts | Original project research; underlying artifact output remains private | Reuse binding, frame, constructor, attached identity and conversion evidence |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/native-and-artifacts.md`, connected REA tool schemas | REA MIT documentation; underlying retained Rizin observations use LGPL-3.0-only tooling | Evidence reuse, supported inspection and lifecycle boundaries |
| Python standard library | PSF tooling | Original private section mapping, source/hash and byte-validation checker |

No SDK/VM implementation was read or copied. This change adds research prose
only; decoded instructions, original bytecode, tables and validation artifacts
remain private. It does not implement or demonstrate native UI/runtime parity.
