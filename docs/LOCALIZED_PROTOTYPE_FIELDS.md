# Effective localized prototype fields

**OBSERVED in the owner's build 1.52.0.1700 files:** the login key
`Frontend/UIButtonLogIn` explicitly supplies both localization references.
Its DisplayText is `0x590a017f2861053f`; its TooltipText is zero. The selected
native Calligraphy path copies an explicit data parent before applying local
fields, and its string-ID reader writes the complete eight-byte value even
when that value is zero. This establishes a bounded native scalar contract for
these saved fields. No retail process or startup flow was executed.

## Actual login data and defaults

The original Rust reader and an independent Python cursor agree on this chain:

| Identity | Exact observed value |
| --- | --- |
| Login prototype | `0xee354851ea1813f3`, `Calligraphy/UI/LocalizedInfo/Frontend/UIButtonLogIn.prototype` |
| Explicit data parent | `0x7b47240667e510f2`, `Calligraphy/UI/LocalizedInfo/UILocalizedInfo.defaults` |
| Blueprint/default ID | `0x7b47240667e510f2`, `Calligraphy/UI/LocalizedInfo/UILocalizedInfo.blueprint` |
| Runtime binding name | `UILocalizedInfoPrototype` |
| Blueprint parents/contributors | Both empty; schema ancestry supplies no extra value layer here. |
| Headers | Both PTP flags `3`, reference present, data present, no polymorphic flag. The default's encoded parent is zero. |

The field payload offsets below are zero-based within the named uncompressed
member; ends are exclusive. They are not offsets within the compressed SIP.

| Field | Schema field ID | Default member | Login child member | Child value |
| --- | --- | --- | --- | --- |
| DisplayText | `0x1c9f6529f4c51892` | `[52,60)`, zero | `[35,43)` | `0x590a017f2861053f` |
| TooltipText | `0x47db9e62f63b18a7` | `[69,77)`, zero | `[52,60)` | `0` |

Both schema declarations are simple String references. The default contains
four scalar fields and the child contains two. Only these two prototype bodies
were decoded for the chain: 96 + 62 = 158 bytes. The raw cursor consumed both
members and the selected blueprint completely and matched all six assignments.
The original `PrototypeValues` view records both child assignments as the latest
stored values, with no structural ambiguity flags for this chain.

The class default therefore supplies explicit zero text references. The login
child supplies its own explicit zero tooltip reference as well. This case does
not require an absent-field default, an inferred blueprint parent, or a guessed
tooltip value. The effective prototype reference remains the child's own ID,
whose native result spelling is `0xEE354851EA1813F3`.

## Native scalar assignment contract

All native addresses are preferred-image virtual addresses in the pinned Win64
executable. Native source kind 1 is explicitly named `eDOCalligraphy` by the
assertion in `0x14158f630`. Its header helper `0x1415a5f70` requires the three
bytes `PTP` and version 11, matching both owned members. The alternative kind-2
reader has a different binary-resource header profile. The exact directory
registration that assigns the runtime record's source kind was not reconstructed;
these are observations of the matching static reader, not a captured load.

**OBSERVED with REA:**

1. The selected object factory requests a cleared block before calling the
   constructor. The imported `memset` path and the generated constructor were
   retained in the earlier localization evidence. Allocation clearing describes
   initial state; it does not replace a serialized class default.
2. Calligraphy entry `0x141572820` validates the stream/header, reads the saved
   reference flags and optional eight-byte parent, then calls `0x141605850`.
   This profile requires a reference field and excludes the polymorphic branch.
3. The initializer loads an explicit valid parent through `0x14158e290` at
   `0x141605a0b`, then calls the copy routine `0x141570be0` at `0x141605a50`.
   Missing parents or failed copying take failure paths. It does not silently
   substitute a blueprint's default for an unresolved parent.
4. The copy routine resolves source and destination runtime class metadata and
   requires matching class IDs (`0x141570c9d..0x141570ca3`). It also checks field
   compatibility. At `0x1415710eb..0x1415710fe`, it adds each descriptor's signed
   offset at `+0x38` to its respective object pointer. Native field type 27
   selects the eight-byte copy at `0x141571146` through the retained jump table.
5. After parent copying, the initializer tests the saved data-present bit at
   `0x141605ac5` and visits the locally encoded fields. A reference-only child
   performs no local assignments. The ordinary local dispatcher
   `0x141603390` resolves the field descriptor and computes its destination;
   unsupported composite/mixin branches are outside this profile.
6. The generated localization descriptors have no custom reader override at
   `+0x68`. Native type 27 therefore selects `0x14159b1c0`, which jumps to
   `0x141553180`. That reader requires exactly eight bytes, then writes the full
   value at `0x1415531f8`. It does not skip zero. A successful later local
   assignment replaces the prior copied value.

The independent generated descriptors tie the two fields to the layout used by
the native callback:

| Field | Descriptor address | Native type | Object offset / width |
| --- | --- | --- | --- |
| DisplayText | `0x1429fa160` | 27 | `0x58 / 8` |
| TooltipText | `0x1429fa1f8` | 27 | `0x70 / 8` |

The new copy/read/traversal evidence directly establishes that descriptor
`+0x38` is an object offset. This closes the earlier offset-interpretation gap
in [FRONTEND_LOCALIZATION_RUNTIME.md](FRONTEND_LOCALIZATION_RUNTIME.md).

For a matching native class and the supported simple field shape, these
instructions support parent-first, then file-order local assignment. An omitted
local scalar retains its prior value; an explicit zero replaces it. They do not
establish equivalent rules for embedded objects, property collections, mixins,
ordinary lists, cross-class copying or arbitrary malformed input.

## Postprocessing and class metadata

The actual object vtable `0x1423babe8` selects postprocessing routine
`0x1415a1e10` at slot `+0x40`. It calls traversal wrapper `0x141acfbf0`, which
uses `0x141acf9b0` with callback `0x141ad15d0`.

The callback subtracts 30 from the field type and immediately returns when the
unsigned result exceeds 20. Both text fields are type 27 and take that return
without a field dereference, write or nested call. The other two class fields,
TooltipStyle and TooltipFont, are types 14 and 15 and take the same return.

This uses the concrete class metadata, not an adjacent table: startup function
`0x1405801d0` passes metadata object `0x1429fa458` to class registration
`0x1416e6280`. The object stores vtable `0x1423bac70`; its descriptor getter
returns exactly the four records beginning at `0x1429fa160`. Its parent getter
identifies the base Prototype class. The selected base metadata exposes zero
field descriptors and no further metadata parent.

The remaining selected postprocessing routine updates a base bookkeeping field
at object `+0x18`. Its metadata lookup calls receive base identities rather than
either text slot. This closes the selected class's postprocessing path for these
two scalar references; it is not a claim that all game classes have inert
postprocessing.

## Result resolver boundary

An original resolver can accept this exact schema/class profile, load the
explicit header-parent chain under budgets and cycle checks, require compatible
field definitions, copy supported scalar values parent-first and apply saved
local assignments in order. Retain the origin of each final value. Reject
missing parents, unknown bindings, incompatible classes, unsupported field
shapes and ambiguous metadata rather than supplying invented values.

For this actual login label, supply the pure result resolver with:

- Prototype reference `0xee354851ea1813f3`.
- Effective DisplayText `0x590a017f2861053f`, originating in child bytes `[35,43)`.
- Effective TooltipText `0`, originating in child bytes `[52,60)`.

The native callback computes `hasTooltip` from `TooltipText != 0`, independently
of the looked-up tooltip string. A zero tooltip reference still passes through
the ordinary locale lookup: the core calls `0x1418b1130` unconditionally,
`0x1418c4840` calls `0x1418bb160`, and those complete selected helpers contain no
zero-ID empty-string fast path. Keep the requirement that ID zero be present in
the selected catalog; absence remains a missing/fallback case. The previous
native default-entry/plain-text profile still governs string selection,
formatting and UTF conversion.

This is resource preparation. Native receiver initialization, script class
construction, ExternalInterface installation, the saved label-setting lifecycle
and rendered retail parity remain separate work. Invalid-label native diagnostics
and their uninitialized temporary-tooltip-bit caveat remain unchanged.

## Original resource preparation API

`frontend::localized_prototype::resolve` applies the bounded scalar contract to
an existing `PrototypeSource`, explicit prototype ID and `ValueLimits`.
`read_member` selects an exact member through `PrototypeReader`; it does not
choose the native label-directory configuration or normalize a path.

The admitted profile requires the same blueprint throughout the explicit data
parent chain, the `UILocalizedInfoPrototype` binding, no schema parents or
contributors, and the four recovered simple declarations. Both text declarations
must be distinct String IDs with no subtype. TooltipStyle and TooltipFont retain
their Prototype/Asset declaration types and subtype presence; their values are
not used to synthesize either text reference. No unrelated schema is treated as
the same native class merely because its name resembles this one.

The existing bounded `PrototypeValues` loader checks parent identity, cycles,
encoded/decoded budgets and structural field compatibility. The new preparation
rejects mixed-copy groups, foreign declaring schemas, lists and ambiguous field
assignments. Each text reference must have an explicit stored assignment in the
chain. It never uses zero-filled allocation to invent a missing terminal value.
The selected origin and complete bounded layer/assignment metadata are retained.

`PreparedPrototype::localized_data` passes those references to the independent
[plain result resolver](FRONTEND_LOCALIZED_DATA.md). Neither operation constructs
a VM object, installs ExternalInterface, executes a script setter or changes a
rendered field. The same-blueprint restriction is an intentionally bounded
implementation profile; the native same-runtime-class check may permit other
schema relationships that have not been established here.

## Evidence and provenance

All retained binaries, package members, disassembly, raw text and probes remain
under `~/.local/share/marvel-heroes-forever-rust/evidence/`. The only repository
change for this investigation is this document.

Private root: `evidence/localized-prototype-fields/`.

- `native.bundle.json`: five canonical REA records, including commands, target
  hash and completion metadata. SHA-256:
  `4a999252af1ad9a4ff3954aa31f2f77ff9e745ea1ed6ed9e3893ee3c010ea969`.
- `verification.json`: selected-output checks, actual payload/value assertions,
  type-27 jump-table checks and all helper receipt hashes. SHA-256:
  `58c1355035d43c3fbb5514fc0d62bb513340f57b019c77724fa152d3f988a9bb`.
- `native-candidates/receipt.json`: 55 retained files. SHA-256:
  `b03721909648fe393d03384f0d991cc24dbbff2516e6164294eb939d186d5ca5`.
- `login-data/receipt.json`: original Rust probe, frozen decoder-source identities,
  independent raw checker, exact package-member offsets and 27 retained files.
  SHA-256: `e9c585b09e1dd512e7983b0f6d6cb0bc4f6b9d26e6e1815d4065a35ec599e2f7`.
- `postprocess/receipt.json`: exact class/base metadata and 104 retained files.
  SHA-256: `3dbb1574c3d6a4be5ea1fe8ca2363684f125f9ee35e263308ef8262eacea7f2a`.
- `native-kind/profile-receipt.json` and `native-kind/registration/receipt.json`:
  eight and 13 files for header/source-kind evidence and bounded registration
  searches. SHA-256 values are respectively
  `55308cba8e8f28c04c86a832c846fa86debcfce6968740feedfac448ddd0ab0a` and
  `6650da4c765c0b6881ec610c10b725885d54d52eaa861faba79dabc60398dd11`.

**VERIFIED (focused static/data checks):** the actual-data Rust probe compiled
and exited successfully; the independent cursor matched both bodies and the
blueprint. All 207 retained helper files and all three selected compressed member
slices passed hash readback. Both complete input hashes match the private pins.
All five REA commands completed with exit 0, untruncated output and unchanged
executable hash; selected instruction ranges have no invalid instructions. Raw
jump tables were read as data separately from code. Earlier receipts remain
unchanged. Whole-function and chained-unwind boundaries are recorded privately;
these checks do not establish whole-program side-effect analysis.

The earlier field/allocator and locale-zero evidence remains in
`evidence/frontend-localization-runtime/aggregate-followup/resource-profile/`.
Its unchanged verification receipt is
`71936d5868d94d6dbe6a2eff31d45dfaba11849d3a12446e55074f1f7fd59ff5`.
The complete core lookup is in the earlier
`evidence/frontend-localization-runtime/native/final.bundle.json`; its selected
zero-tooltip lookup is unconditional before computing the boolean.

| Source read | Rights / provenance | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owned client; authorized private interoperability study | Selected native initialization, copy, scalar reader, class metadata and postprocessing through REA. |
| Owner's pinned `Data/Game/Calligraphy.sip`, SHA-256 `1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c` | Proprietary owned client data; authorized private interoperability study | Exact login prototype, default prototype, schema, directory and compressed-member identities. |
| Original project `src/localization.rs`, `src/prototype_values.rs`, SIP/Calligraphy/prototype decoder modules, `docs/PROTOTYPE_VALUES.md`, and original private PE/decoder helpers | Original project code and existing provenance; no external implementation copied | Focused actual-data probe, independent raw checking, bounded address discovery and comparison with the existing resource view. |
| Prior private localization REA bundles and receipts | Same owned executable; immutable existing Evidence | Allocation clearing, field descriptors, return-object contract and zero-reference locale lookup. |
| REA skill and registered tool schemas | Analysis workflow/tool documentation | Target-free read-only commands, canonical Evidence export and completion checks. |

No new external source repository or webpage was consulted. Static observations
are labelled separately from executed checks; no native process, account, server,
network interaction or VM startup was run.
