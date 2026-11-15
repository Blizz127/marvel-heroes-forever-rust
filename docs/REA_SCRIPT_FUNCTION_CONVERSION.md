# Actual ScriptFunction conversion and captured-frame boundary

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED, static native control flow:** An ordinary direct function value
created by `DefineFunction2` uses interface `0x142513090`. Its callable-reference
slot `+0x60` points to `0x14018c580`. A normal one-argument
`Function(script_function)` call preserves the underlying ScriptFunction object
and returns a reference with a null auxiliary frame pointer and zero reference
flags. It does not preserve an arbitrary input function-value wrapper.

This closes the interface-identification gap described in
[REA_FUNCTION_CALLS.md](REA_FUNCTION_CALLS.md). The previously assumed table
`0x142513260` is not needed for this proof. No original client execution or
runtime parity is claimed.

## Binding the actual producer

The high-opcode dispatch entry for raw opcode `0x8e`, at `0x14020d554`, targets
`0x14020c74e`. The complete handler calls `0x1401fa280` at `0x14020c755`.
An earlier 11-byte handler read split that call; the retained 12-byte decode
supersedes it.

Creator `0x1401fa280..0x1401fa7c2` allocates `0xd0` bytes and calls
`0x1401e14c0` at `0x1401fa2e7`, passing function type `2`. It then reads the
serialized parameter count, register count and flags, storing the latter two
at object offsets `0xcb` and `0xc8`. Constructor `0x1401e14c0` installs:

| Field | Value | Store |
| --- | --- | --- |
| Primary vtable | `0x142513140` | `0x1401e14fe` |
| Object-interface vtable at `+0x20` | `0x142513090` | `0x1401e1501` |
| Function type at `+0xca` | Incoming type `2` | `0x1401e1595` |

REA reads the actual interface slot at `0x1425130f0`, obtaining
`0x14018c580`. This is a producer-to-constructor-to-slot binding, rather than
classification by an adjacent vtable or a similar method.

The creator's later helper `0x14019ce50`, called at `0x1401fa693`, attaches
Function.prototype through this interface's `+0x68` parent setter. Its optional
`0x14018ed40` call assigns the public `constructor` member through the existing
raw setter. These paths update prototype/member metadata without replacing
the ScriptFunction interface.

## Exact conversion result

The existing global `Function` callback proof identifies `0x1401e17a0` and
its one-argument conversion path. This document scopes the input to a direct
native Value with tag `8`; lazy tag-`11` resolution remains separate.

The relevant layouts are:

| Field | Function Value offset | FunctionRef offset |
| --- | --- | --- |
| Function object pointer | `+0x08` | `+0x00` |
| Auxiliary frame pointer | `+0x10` | `+0x08` |
| Reference flags byte | `+0x18` | `+0x10` |

The direct tag-`8` branch of `0x140199640`, at `0x1401997c0`, returns only
the input Value's function-object pointer. It does not read the input wrapper's
auxiliary pointer or flags. It does not mutate that input Value.

`0x1401936f0` recognizes callable object category `0x17` and invokes the
actual interface's `+0x60` method. The complete getter
`0x14018c580..0x14018c5a6`:

1. Subtracts `0x20` from the interface pointer to recover the same object.
2. Writes zero to the destination FunctionRef flags at `+0x10`.
3. Writes null to its auxiliary pointer at `+0x08`.
4. Writes the object pointer at `+0x00`, retains it, and returns the supplied
   destination reference.

There are no calls, allocations or writes to the function's stored code,
target/context fields or scope container in this getter. Its only write into
the function object is reference accounting at object `+0x18`.

The result setter `0x14018de50` constructs the tag-`8` output from that getter
reference. Its auxiliary-attachment helper is skipped because the getter
returned null. The setter has a shortcut for an already-tag-`8` destination
holding the same object, but normal `ActionCallFunction` initializes its result
to tag `0` at `0x14020a78e`, so this conversion does not take that shortcut.
The normal result is therefore the same object, null auxiliary frame, flags zero.

## Captured-frame effect

**VERIFIED, static:** The auxiliary field is not necessarily empty on a
`DefineFunction2` value. At `0x1401fa4d4..0x1401fa509`, the creator reads the
environment's frame-array count at `+0x188`. If a non-null top entry exists
in the array at `+0x180`, it retains that pointer in its FunctionRef. At
`0x1401fa527..0x1401fa54d`, it also installs that pointer in the emitted
tag-`8` Value's auxiliary field. Otherwise the field remains null. Named and
anonymous definitions use this constructed value.

The ScriptFunction invoke method is primary slot `+0x50`,
`0x14020e190`. It stores its incoming auxiliary argument `r8` into invocation
state `+0x18` at `0x14020e1a9`. The type-`1`/type-`2` frame-preparation path
in `0x140205090` reads that field at `0x1402051d0` and assigns it to the
new frame's `+0x28` link at `0x140205204`. A null auxiliary argument therefore
produces a null link there.

Consequently conversion retains the function object while dropping the input
reference's captured-frame association. It does not clear or mutate the original
reference or its frame. It also does not replace the object-held code and
context: invocation still reads the same object's code reference at `+0x78`,
body offset/length at `+0x98`/`+0x9c`, and context container beginning at `+0x80`.
Full variable-resolution behavior through the frame link was not reimplemented
or exercised by this investigation.

## Supported implementation boundary

The temporary rejection of every ScriptFunction input is no longer required by
an unknown interface getter. The existing conservative VM profile containing
only target/global scopes can use function-object identity for this conversion,
provided it has no modeled captured activation/frame association to preserve.
The existing rejection of nested activation and With scopes can remain in place.

An implementation must not generalize this into identity-preserving conversion
for every closure wrapper. Supporting captured frames requires representing the
new reference's null auxiliary association without modifying the original
function reference. This document does not expand support for lazy tag-`11`
references, bound functions, arbitrary closure invocation, dynamic-source
`Function` construction or other argument counts. No VM code was changed here.

## Evidence and validation

The owner pin is
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All paths below are relative to the private `evidence/` directory.

| Evidence | Purpose |
| --- | --- |
| `rea-script-function-conversion/final.bundle.json`, `ev_d9be3386e524eddf79c98e01ae3bb0dcb747b6e7e5b6cdf599492dfcf1e34fa0` | Exact actual interface slot and complete getter. |
| Same bundle, `ev_1f2416d2d53ec618a07f4c183f537d13fa665208f3b250d610a66044174a66a2` and `ev_a56a7f16a34bd88c06e2ea71c4b1726ff5627c45404b3117e8171bda728617a9` | Creator's final parent/constructor metadata helpers. |
| `rea-movieclip-binding/final.bundle.json`, `ev_47aa0a8e7aec2220f324111df61dc655864d6ce2f2ad40188b6dbf5fe88b2643` and `ev_2e1f1b1cf76e2c7855dc69e56a9495b02cd851504e8d20bd6ec2e524d4994646` | Complete DefineFunction2 handler and creator, including frame capture. |
| Same bundle, `ev_a3814671d1847f983bb6650b2354ab75b992592ad2a3b5a0827e323542a725cb` | Actual ScriptFunction constructor and tables. |
| Same bundle, `ev_34ee943aaf37af1a7e8a34152742f96ac3ca4e83a880f46578bbd98a338ec26d` and `ev_704babdfcb3aea3b289230706c1b4bcae3a26072f95048b7bb3e96e8ad34ee37` | Invoke and frame preparation consuming the auxiliary pointer. |
| `rea-script-function-conversion/prior-function-path.json` | Exact reused global callback, input unwrap, output setter, fresh result and prior classification correction. |
| `rea-script-function-conversion/finding-summary.json` and `validation.json` | Field contract, scope, hashes and selected byte checks. |

The three new target-free REA calls completed with exit zero, no output
truncation and unchanged target digest. Their bundle SHA-256 is
`ea0a32dc4489985393f7da4cd69e4293acb4af6d2f77a61d40922f07c15bc1e3`.
The new slot read and 2,051 selected instruction records, including 111 new
getter/metadata-helper records and reused producer/invocation records, match the
original PE bytes. The separate existing-path receipt verifies another 353
selected records. Padding is not used for semantic claims. REA evidence was
exported and the investigation reset; the client was never launched.

## Provenance

| Source | Licence / status | Use |
| --- | --- | --- |
| Owner's pinned Win64 executable | Proprietary owner-held client; private interoperability study | Bounded static instructions, exact interface pointer and byte validation. No recovered implementation copied into Rust. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and connection/native guides | REA toolkit MIT; Rizin LGPL-3.0-only tooling | Supported target-free analysis, evidence and lifecycle instructions. |
| `docs/REA_FUNCTION_CALLS.md`, `docs/AVM1_VM.md`, `src/avm1/vm.rs`, and the prior private bundles identified above | Original project analysis/code; underlying target evidence remains private | Reuse existing conversion, producer and invocation proof; inspect the conservative VM boundary. External pages cited by those documents were not reopened. |
| Python standard library | PSF tooling | Original PE mapping and evidence-byte checks. |
