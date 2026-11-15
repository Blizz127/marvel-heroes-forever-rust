# AS2 native global baseline

**VERIFIED, bounded static evidence:** a fresh successful call to native
`0x14020ea40` installs 46 explicit own global properties before lazy resolution
and later movie or host writes. The diagnostic VM must not equate its smaller
binding set with retail absence. A registered but unsupported native binding
needs an explicit boundary when read or used.

The target is the owner's pinned Win64 `MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No original client execution occurred in this study. These findings do not
establish the live manager's complete namespace or runtime state.

## Explicit bootstrap

The global object has interface `0x142512aa0`. Its base constructor starts its
own-member hash and parent pointer null. On the fresh owner path, bootstrap
allocates this global object; if owner `+0x60` already holds one, it reuses it.
Allocation success and fresh maps are preconditions for the finite inventory
below. Reinitializing a populated owner is a different state.

The 46 explicit names comprise 21 native functions, four eager native callable
objects, 19 deferred builtin bindings, and two namespace roots. Every one starts
with member flags zero. At the end of bootstrap, visitor `0x14017b7a0` ORs flag
`1` (`dontEnum`) into the global object's own stored properties. It does not
traverse property values or nested namespace objects.

| Requested binding | Fresh stored identity | Final member flags | Native registration anchor |
| --- | --- | --- | --- |
| `Stage` | Deferred builtin token, tag `11` | `1` | Wrapper `0x140201c00`, factory `0x1402014a0` |
| `Mouse` | Deferred builtin token, tag `11` | `1` | Wrapper `0x140201ea0`, factory `0x1401fff70` |
| `Key` | Eager native callable, tag `8` | `1` | Constructor `0x1401fe970`, callback `0x1401894f0` |
| `TextField` | Deferred builtin token, tag `11` | `1` | Wrapper `0x14020e8f0`, factory `0x14020de90` |
| `TextFormat` | Deferred builtin token, tag `11` | `1` | Wrapper `0x1401f8c50`, factory `0x1401f71a0` |
| `flash.external.ExternalInterface` | Deferred token on the `external` namespace | `0` | Wrapper `0x1401f9970`, factory `0x1401f2280` |

The eager callable group is Object, Function, MovieClip and Key. Key's actual
primary/interface tables are `0x142516db0`/`0x142516d00`. Its initial function
reference has auxiliary pointer and reference flags zero. Member flags are
separate from those function-reference flags. Native method behavior is not
established merely by recording this identity.

The native function registration table at `0x142adc940` has 21 entries and a
null terminator. Four subsequent explicit writes replace table-created function
objects with newly allocated objects using the same callbacks; they add no
names. The private receipt preserves the exact final names, pointers, callback
addresses, flags and replacement identities. Its sorted 46-name sequence has
SHA-256 `e11c87fc4fa122c6c93da167ec4216f5888baed174e1d82b7c5c4187143c6075`.

## Namespace objects and deferred registration

Bootstrap constructs or reuses `flash.geom`, `System`, `flash.external`,
`flash.display`, and `flash.filters`. In a fresh context these paths create six
namespace objects, because the four `flash` paths share one root. The helper
`0x1401d0850` performs member lookup for each component. A found object is reused;
a missing component allocates an ordinary object parented to Object.prototype
and writes that component with flags zero. It does not replace every existing
path unconditionally. A non-object existing component and allocation failure
are outside the successful fresh-context inventory.

Each new namespace object's own `__proto__` metadata has flags `3`. The final
global visitor makes only the top-level `flash` and `System` descriptors flags
`1`; nested components and their deferred bindings retain flags zero. Fifteen
of the 34 deferred bindings belong to these nested namespaces. The other 19
belong directly to the global object.

All 34 registration wrappers have the same operations and control flow as the
complete REA-decoded representative `0x1401f8080`, except for consistent builtin
name offsets and factory addresses. Original byte comparison verifies this
finite equivalence. A wrapper first checks owner `+0x20`'s registry. If its key
already exists it returns without installing another token. Otherwise it stores
the factory and empty cached reference in that registry, then writes a tag-11
Value containing the interned name onto the supplied namespace/global object.
It does **not** call the factory during registration.

Therefore this receipt establishes the stored token and factory selection, not
the returned object from first lookup, its methods, repeated-lookup identity or
cache transitions. Those require the selected resolution/factory evidence.
Ordinary stored-member copying alone does not establish lazy resolution.

## `gfx`, `gfx.Extensions`, and `gfxExtensions`

**OBSERVED:** neither `gfx` nor `gfx.Extensions` belongs to the exact explicit
bootstrap inventory. This is a bounded negative finding about stored names,
not a claim about all native lookup results, target shadows, host injections
or later movie bytecode. The original manager's namespace-creating initializers
are a separate source of state; see [AVM1_VM.md](AVM1_VM.md).

**VERIFIED:** the global-specific getter `0x1401db080` recognizes the distinct
single name `gfxExtensions`. Its owner state byte at `+0x68` controls the result:

| Owner state byte | Getter result | Found result |
| --- | --- | --- |
| `0` | Undefined | false |
| `1` | Boolean true | true |
| Any other nonzero value | Boolean false | true |

Fresh owner creation initializes that byte to zero at `0x14021014b` and
`0x1402101dc`, before invoking bootstrap. Other names tail-call generic getter
`0x140197000`. The getter does not synthesize a `gfx` namespace. Later writes to
the state byte were not exhaustively traced here, so its fresh default is not a
claim about its value after the loaded manager has run.

## Conditional host additions

The game host function `0x14127fdf0` has a separate post-creation callback setup.
Its existing-host-object branch returns early; the newly created successful
movie branch reaches callback setup. It queries `0x140053790` for a native mode
byte. The concrete AS2 support/root chain proves this result is `1`: support
vtable `0x1421c3170`, slot `+0x30`, returns byte one; root construction caches it
at `+0x20`; the API getter reads it through MovieImpl `+0x18`.

For mode one the host creates callback wrappers and submits these exact paths
to API `0x1400537b0`:

- `_global.gfxProcessSound`
- `_global.CLIK_loadCallback`
- `_global.CLIK_unloadCallback`

The API delegates to root virtual slot `+0x160`, concretely `0x140206800` on the
AS2 root. Callback creation uses slot `+0x158`, `0x1401e1a90`. These submissions
are later than the generic bootstrap's 46-name inventory. The host callsites do
not inspect setter success. This study does not establish callback allocation
success, final member flags, callback behavior, or that this exact setup was
executed for the privately extracted manager instance.

The distinct mode-two branch resolves
`flash.utils.getDefinitionByName("scaleform.gfx.Extensions")` and assigns the
selected extension members. It does not submit `_global.gfx.Extensions` in this
function. A string mentioning extensions is therefore insufficient evidence
for an AS2 global named `gfx.Extensions`.

## Runtime boundary

A loaded-manager compatibility claim still needs the actual target/global
scope, native host initialization, supported deferred factories and subsequent
movie writes. Special getters, inherited members and current-target precedence
remain separate from the finite stored-key inventory. Unsupported known native
globals must not silently become ordinary undefined values merely because the
diagnostic VM has not installed their behavior. This document does not authorize
invented native method stubs or unconditional namespace creation.

## Private evidence and validation

All paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-as2-global-baseline/finding-summary.json` and original
  `build-receipt.py`: combined claims, exact conditions, validation and limits.
- `rea-as2-global-baseline/static-inventory/global-static-inventory.json`:
  complete stored-name inventory, 34 deferred registrations and exact identities.
  `data.bundle.json` holds new records
  `ev_8e4a6bd2ce946205ea117fbdd672f753a2ed34deda4f9d1f03f0a5b1d245a444`
  and `ev_c27f9431ce20f28d37a61ecfed75dd26cf54ee3cedc0eeccb73b1c3d6c7a496a`.
- `rea-as2-global-baseline/final.bundle.json`, SHA-256
  `56a9393ffab2f0dab29f8f9b2ec9c49c4a8abf3b48f32d0e28196df7ecf75347`:
  host setup `ev_b8608c6e79347f72856935e10e82e692e77fb0ecf34b36f5dd9ecab7e4e51c80`,
  names/API dispatch `ev_2979af4f6a7acc757d1366054faf3ce3d0bbcf4a9e1eef7e36aaa3dc466cbf7b`,
  and complete special getter
  `ev_0dd4feb4dafb69d0496b8d42f61576f8dd316311bb59316779efd14c6fc170d8`.
- `rea-as2-global-baseline/ue3-mode/finding-summary.json` and `final.bundle.json`:
  concrete AS2 mode-byte producer/consumer, with exact root slots. Bundle SHA-256
  `3c0c463277e7a20286a972b390d853c6575fe381e4eb182b3ee5e78f06016454`.
- Reused `rea-class-registry/instantiation.bundle.json`, record
  `ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a`:
  fresh owner state initialization. Namespace/helper and earlier registration
  records are identified individually in the private receipts.

The inventory checker compared 4,444 instruction records and 14 byte arrays
with the pinned file and checked all 34 wrapper equivalents. The host/getter
checker compared 516 new selected instruction records, 10 data reads and 225
reused records. The mode bridge checked 222 instruction records and two data
reads. These counts overlap earlier evidence; they are not runtime tests.
Every new REA result completed without output truncation or target mutation.
Each analysis session was exported and reset. No build, VM implementation or
retail runtime check is claimed by this documentation change.

## Provenance

| Source | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe` | Gazillion/Marvel proprietary; private owner installation | Static registration, namespace, type, state and host-branch evidence for interoperability; binary output remains private. |
| REA skill and `references/connection-and-recovery.md`, `references/native-and-artifacts.md` | REA MIT toolkit | Supported target-free evidence workflow and lifecycle. |
| Rizin provider | LGPL-3.0-only; tool use only | Bounded disassembly and byte/string reads; no provider code linked or shipped. |
| Existing project `REA_QUALIFIED_VARIABLES.md`, `AVM1_VM.md`, `REA_FUNCTION_CALLS.md`, `REA_MOVIECLIP_CONSTRUCTOR.md`, `REA_ARRAY.md`, `REA_CLASS_REGISTRY.md` and cited private evidence | Original project analysis; underlying proprietary output stays private | Reuse exact constructor/member/global evidence and avoid duplicate analysis. |

All analysis and receipt scripts are original. No SDK or external VM
implementation source was used to supply missing behavior.
