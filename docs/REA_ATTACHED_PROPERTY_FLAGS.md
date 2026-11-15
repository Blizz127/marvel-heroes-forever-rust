# Selected property flags on an attached MovieClip

Task `a3d58c579542`, 2026-11-15 UTC. **VERIFIED static native behavior:**
`ASSetPropFlags(attachedRoot, "dispatchQueue", 1)` converts its first argument
from a public display Value to the attached owner's interface. Its selected-name
hooks obtain script storage and delegate to that storage's own-property lookup
and flag setter. With existing storage, an existing own ordinary property, three
arguments and active Environment version 15, the result is **old flags OR 1**.
The stored property value is preserved; the selected path does not run a getter
or normal member-write hook.

The storage requirement matters. Even the attached raw lookup invokes the lazy
get-or-create factory. Missing storage cannot generally be treated as an empty
own-property map with no effects. The initial implementation can require
existing storage and the requested own ordinary data entry, making that broader
allocation path an explicit unsupported boundary.

## Original call and what the probe establishes

**OBSERVED owned bytes:** `gfx.events.EventDispatcher.initialize` has
Function2 definition offset 17,005, body 17,024 + 351 bytes, flags `0x12a`,
three registers and parameter `target` in register 2. Preload-global supplies
register 1. The original named CallMethod is at 17,373. Body SHA-256:
`3229c35cab571ca8311eeb6ff6bc871d05718a20aa6dcf219a54b0d92cce7953`.

The separate controlled VM probe, frozen at `e41ceea`, completes 452 initializers
and reaches this call after 154 frame instructions. Its source/context analysis
identifies `_global.ASSetPropFlags(target, "dispatchQueue", 1)`, with attached
root as target. The earlier original SetMember at 17,176 assigns the instance's
dispatchQueue member value to target. The helper reports no register overwrites
that would change this argument flow. The observed stop is the VM's primitive
property-receiver gate, before its flag mutation.

This is a source/context derivation, not an intrinsic argument dump. That probe
did not capture the current public dispatchQueue value or its own flag byte.
Do not invent an initial flag value of zero. The implementation must validate
the actual own descriptor and preserve its existing flags. Existing root
storage was separately observed after prototype assignment; native behavior
below is conditional on that storage and own-entry state.

The probe receipt is private
`evidence/avm1-attached-manager-frame-action-increment3/receipt.json`, SHA-256
`d0affcefd6b9255b5534603938dae30c061ffcd6e920ba4353fc2966fc1a04e0`.
The original unmodified movie is
`evidence/frontend-startup/MarvelGame-frontend_manager.gfx`, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Neither this study nor that Rust probe establishes retail execution or parity.

## Argument zero resolves to the attached owner

The canonical ASSetPropFlags callback is `0x1401cb620..0x1401cba6c`, bound by
the earlier [native property-flags study](REA_NATIVE_ALTERNATIVE.md). Its code
reads argument zero using the active FnCall Environment and argument index,
then calls `0x1401997f0` at `0x1401cb67f`. The returned interface is retained
in R14 for the later selected-name calls. **FnCall.this is not the flags target.**

The converter's tag-7 path is the already proved live display-handle route:
it requires an active Environment/target/root association, resolves the input
handle through `0x14009a540`, requires target `+0x6a` bit 7, obtains the owner
through `target + 4 * target[+0x6d]` and primary virtual `+8`, then returns
owner `+8`. The case-3 primary method is the identity leaf `0x1405b93f0`.
The resulting object interface is `0x1425175c0`, category 2.

This is the attached interface, not the category-11 lazy storage or an ordinary
Object conversion. The coherent live-handle, current context and no-retirement
bounds remain those of [REA_ATTACHED_FUNCTION_APPLY.md](REA_ATTACHED_FUNCTION_APPLY.md).
The conversion does not grant a general script Object identity to the owner.

## Exact attached hooks and storage delegation

**VERIFIED new complete REA instructions**, with retained exact table bindings:

| Interface / slot | Bound function | Selected behavior |
| --- | --- | --- |
| Attached owner `0x1425175c0`, `+0x28` | `0x140189840..0x1401898ca` | Obtain storage; delegate raw own lookup to storage `+0x28` |
| Same owner, `+0x38` | `0x1401897b0..0x14018983c` | Obtain storage; delegate requested flag byte to storage `+0x38` |
| Owner primary `0x142517670`, `+0xc8` | `0x1401de510` | Tailcall storage factory `0x1401de450` |
| Storage `0x142511d60`, `+0x28` | `0x1401a1540` | Generic own-hash lookup, using supplied context's version |
| Same storage, `+0x38` | `0x1401e3ef0` | Generic own-property flag replacement |

Both attached hooks subtract eight from their interface pointer to recover the
owner, then invoke primary `+0xc8`. Lookup does so at `0x140189865`; flag setting
does so at `0x1401897d6`. For nonnull storage they retain its reference, call its
interface at storage `+0x20`, release the reference, and return the delegated
boolean. The name, context and lookup destination / requested flag byte are
forwarded unchanged. A null factory result returns false.

Factory `0x1401de450` checks owner `+0x1d8`. When nonnull, its fast path at
`0x1401de4f5..0x1401de502` directly returns the existing pointer: there is no
allocation, initialization or prototype visitation. If null, the factory can
allocate 0x78 bytes, call `0x1401da850`, and publish the new storage pointer.
That constructor has its own target/prototype/event-state work documented in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md); it cannot be silently omitted.
This study admits only the existing-storage path.

For the selected ASSetPropFlags loop, the owner lookup happens first, at
`0x1401cb9ea`. Only a true result reaches owner flag setter `+0x38` at
`0x1401cba0d`. The latter obtains the same existing storage again and its generic
setter repeats the own lookup before writeback. No normal attached getter,
named-child lookup, ordinary member assignment or event-name write hook is used.

## Own-property lookup, values and the exact flag update

The string selector path converts the tag-5 String and splits on commas. For
the exact ASCII name `dispatchQueue`, no comma occurs and the existing final
suffix path yields that one unchanged selector. Broader splitting/coercion
behavior is not required for this case.

With active context version greater than six, generic wrapper `0x1401a1540`
passes storage's own hash container, interface `+0x18` = object `+0x38`, to
`0x1401938a0`. Its leaf `0x140181fd0` searches only this hash's collision chains
and compares the interned key. It does not traverse a prototype or search
display children. A match copies the raw stored Value through `0x14018d9d0`
and copies its flag byte separately. An own value of Undefined remains present.

Setter `0x1401e3ef0` performs that raw own lookup, replaces only the temporary
flag byte at `0x1401e3f30`, and calls `0x1401cffd0` on the same own hash.
The existing-entry branch locates the exact key through `0x140181f40`, copies
the same raw Value back at `0x1401d0044`, and writes the requested flags at
`0x1401d004e`. It does not call the missing-entry insertion helper on this
matched path. The raw Value copier retains a tag-9 property/accessor reference
without invoking the getter. The actual requested scope is an ordinary data
property; unmodeled accessor representations can remain an implementation gate.

Thus an inherited-only ordinary property is not changed by this selected-name
path, and no own shadow is created. For already existing storage, a missing own
name produces no flag write. These statements must not be generalized into
no-allocation claims when storage itself is absent.

The callback masks the supplied set value to its low three bits. For exactly
three arguments, its clear-mask default is seven only when active Environment
`+0xf0` equals five; otherwise it is zero. Active version 15 therefore gives:

```text
set = 1; clear = 0
new_flag_byte = (old_flag_byte & ~clear) | set = old_flag_byte | 1
```

All existing bits, including high bits outside the three public mask bits, are
preserved in this byte. Read-only or nondeletable flags do not redirect this raw
flag update through ordinary assignment. The value/function identity, own-key
membership, insertion history and linked prototype are unchanged on the
admitted existing-entry path; reference/key accounting remains native internal
bookkeeping. There are no getter, normal setter, child or script-event calls in
the traced matched-entry path.

The governing version is the supplied active Environment's version, not a value
inferred solely from the receiver's source movie. Current root/header 15 must
be joined to the established [Environment selection](TEXTFIELD_SOURCE_ENV_VERSION.md).

## Return value and bounded Rust consequence

The original call uses named CallMethod. Its handler initializes its result
local to tag zero at `0x14020b757`, supplies that destination in FnCall `+8`,
and later copies it back to the operand stack. Native wrapper `0x14018c5b0`
preserves that destination. ASSetPropFlags never writes it, so this canonical
successful callback route returns **Undefined**, not the setter's internal
boolean. This join uses the named-call handler rather than assuming the separate
CallFunction result-initialization proof applies automatically.

**INFERRED bounded implementation:** accept a typed live DisplayClip first
argument through its established owner/context association, require already
existing bound storage and the exact own ordinary data property, then apply
the existing raw selected-name flag machinery to that storage. Preserve the
stored Value and compute from the current old flags. Do not expose storage as
the public receiver or call ordinary property read/write helpers that could
traverse prototypes, execute accessors, or run attached write hooks.

An initial exact-name/argc-three/set-one/version-15 gate is a supported narrower
profile. Standard native names, null/all-property selectors, arbitrary selector
arrays/coercions, lazy-storage creation, dead/noncanonical handles, other display
categories, broader masks and unknown context versions remain separate work.
The native raw hooks admit more behavior than this implementation scope; their
existence alone is not permission to fabricate its effects.

## Evidence, checks and provenance

The owner executable is private
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses above are preferred-image VAs; offsets are hexadecimal.

One new target-free REA observation,
`ev_06be8c217d08ace13793837b975063e7735fa0d3d374de7387f11e0b1b692871`,
contains both complete attached hooks: **78 instructions / 278 bytes**.
It reports complete, exit zero, untruncated output and unchanged artifact.
The private `rea-attached-property-flags/owner-hooks.bundle.json` SHA-256 is
`698b5d873333ed0da290e07daac982edd186734dd1677bf456070d53263ea706`.
REA exported and reset exactly its one record, with no retained target;
`reset-receipt.json` records cleanup. No executable was launched, shared
configuration changed, or other lane's session used.

The original private `verify-evidence.py` independently checks **1,469 complete
instruction spans and 352 table bytes** across 14 new/retained records against
PE section mappings. It also checks the exact Function2 header/body hash and
CallMethod byte, and the sealed controlled-probe receipt digest.
`verification.json` SHA-256 is
`80495c3f476645f59ce3ee4579f06551d2bb073fd7d1a879dff7c32341c87b4a`.
It lists exact range boundaries, IDs and source-bundle hashes. Padding and a
partial neighboring decode after the generic raw getter are excluded.

| Retained private evidence | Reused proof |
| --- | --- |
| `rea-asflags-native/final.bundle.json`, `ev_3dc579...`, `ev_3efb471...` | Complete callback, first-argument converter, selected loop and masks |
| `rea-attached-movieclip/complete.bundle.json`, `ev_a256724...`, `ev_f9c7c7...` | Exact owner/storage slots, get-or-create factory, category and constructor provenance |
| `rea-native-alternative/asflags-scope-final.bundle.json`, `ev_3a27c39...`, `ev_98b99cd...` | Generic raw lookup and flags-setter wrappers |
| `rea-asflags-selected-name/final.bundle.json`, `ev_19d408...`, `ev_71a378...` | Same-own-hash lookup/writeback and leaf finders |
| `rea-asflags-followup-return/final.bundle.json`, `ev_41547e...`, `ev_baa1bc...` | Raw Value/accessor-reference copying and native invocation wrapper |
| `rea-movieclip-binding/final.bundle.json`, `ev_c30fe045...` | Exact named CallMethod result initialization, dispatch and stack result |
| `source-version-env/complete.bundle.json`, `ev_a35bbbf...`, `ev_02263db...`; `rea-attached-function-apply/verification.json` | Live public-handle shortcut and case-3 owner identity |
| `rea-asflags-comma-selector/finding-summary.json` and its named complete records | Unchanged String identity, exact ASCII delimiter and final suffix selection |

Sources read: the executable/movie are proprietary owner-installed artifacts
used for private interoperability; existing project documents and named REA
receipts are original research. The installed
`~/.agents/skills/reverse-engineer-anything/SKILL.md`, native guide and connected
tool schemas supply REA workflow guidance (REA toolkit MIT; retained/new Rizin
tooling LGPL-3.0-only). Python's PSF-licensed standard library supplies the
original private checker. No outside VM/SDK implementation was read or copied.
No bytecode, instruction listing, table, asset or extracted data is committed.
This change is research prose only; it contains no VM implementation or native
runtime/retail-parity claim.
