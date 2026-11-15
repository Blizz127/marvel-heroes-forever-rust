# Scale9 class linking and loaded texture fields

**VERIFIED static dispatch/operation evidence; OBSERVED saved declarations.**
This continues [SCALE9_TEXTURE_INPUTS.md](SCALE9_TEXTURE_INPUTS.md). The selected
native class serializer requests recomputation, and its UClass override forwards
that request. Core.Object's saved header declarations provide a route to derive
the inherited extent from zero instead of treating constructor size as loaded
state. Texture2D serialization also contains a concrete OriginalSize overwrite
whose archive-version gate remains explicit.

New private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-loaded-layout/`.
No native client, server, network or GPU was run. Renderer behavior and product
code are unchanged.

## Selected serializer and Link dispatch

The complete UStruct serializer `0x140751330..0x140751d48` keeps the supplied
archive in RBX and object in RSI. At `0x140751b9f`, archive dword+0x14 must be
nonzero to reach its final Link call. A zero value exits at `0x140751d21`.
The intervening branches may scan the bytecode buffer, but both continuing
paths reach `0x140751d0c`: object virtual+0x278 is called with the same archive
and **third argument1**. This is a native branch contract, not proof of a live
archive's flag value.

The class metadata constructor already proved in the preceding increment
installs table `0x141df82c8`. Its exact +0x278 slot at `0x141df8540` selects
`0x14074a2d0..0x14074a900`. That complete UClass override directly calls
UStruct Link `0x14074a910` at `0x14074a301` with the incoming RCX/RDX/R8
unchanged. It does not suppress recomputation for this serializer call.
Its +0x270 slot at `0x141df8538` selects the complete five-byte leaf
`0x14025f3d0`, returning object+0x78: the selected superclass pointer.

On this nonzero mode, the retained UStruct Link first clears own+0x88 to0 and
own+0xa4 to1. For a nonnull superclass it calls archive virtual+0x20 on that
superclass, adopts its size and alignment, then rounds the initial size to a
four-byte boundary. It next applies the exact linked-property operations from
the previous increment. A null superclass starts at zero. The constructor's
old extent is therefore not an adequate substitute for the first-loop result.

There are two material later conditions:

- Object flag bit21, archive+0x14 nonzero and nonzero Link mode admit a later
  block at `0x14074b0bc`. It obtains linker metadata through archive+0x48,
  can reorder Children and Next pointers, clears bit21 at `0x14074b725`, then
  jumps back to recompute. The saved Children order is not silently treated
  as final when this block is active.
- UClass's post-base work can adjust offsets of children inside separately
  selected fields. This requires nonzero mode, selected field class mask
  0x40000, its owner-helper result matching the original class, and its
  byte+0xd4 bit3. Each accepted child property's +0x8c receives a cumulative
  offset at `0x14074a7a0`; the accumulator advances by selected field+0x88.
  These qualified nested-child writes are not a blanket preservation claim
  for every property in the whole lifecycle.

The existing zero-mode path remains distinct: it saves/restores property
offsets around property linking. This serializer-selected path does not use
that mode. Archive callbacks, actual flag state, name resolution and object
lifetime are still conditions on successful native loading.

## Core.Object has explicit header declarations

The hash-pinned Core package has Core.Object export0 with serialized
SuperField0. Its complete acyclic Children chain contains 392 directly owned
fields, of which exactly 13 are instance properties. The entire direct-export
set is covered; functions, constants and other non-properties are not silently
interpreted as instance storage. All 13 properties have ArrayDim1.

| Saved property | Saved type | Replayed offset from zero |
|---|---|---|
| VfTableObject | Struct Pointer | 0x00 |
| HashNext | Struct Pointer | 0x08 |
| ObjectFlags | Struct QWord | 0x10 |
| HashOuterNext | Struct Pointer | 0x18 |
| StateFrame | Struct Pointer | 0x20 |
| Linker | ObjectProperty | 0x28 |
| LinkerIndex | Struct Pointer | 0x30 |
| ObjectInternalInteger | IntProperty | 0x38 |
| NetIndex | IntProperty | 0x3c |
| Outer | ObjectProperty | 0x40 |
| Name | NameProperty | 0x48 |
| Class | ClassProperty | 0x50 |
| ObjectArchetype | ObjectProperty | 0x58 |

The native Pointer rule from the preceding increment supplies size8/alignment4.
QWord's saved struct consists of two scalar Int fields, A then B. Independently,
the exact registration block `0x1407b2405..0x1407b2489` copies `QWord` including
NUL from `0x141e03cc4`, assigns entry+8=0xc0, and calls the already recovered
name publisher `0x14079cca0`; the resulting ordinary name index is0x60.
That name reaches the same size8/alignment4 override as Pointer in normal
UStruct Link. Successful ordinary name registration is a condition, not a
borrowed host-language QWord definition.

With the selected native property widths and unchanged declaration order,
the final Core.Object extent is **0x60**, with maximum alignment4. This
explains the previously supplied base using owned declarations and native
link operations. It also independently agrees with the consuming native
object fields already recovered at +0x10, +0x38, +0x40, +0x48 and +0x50;
those observations were not used to assign declaration offsets.

The remaining three widths are now tied to exact named native builders and
constructor-installed tables. NameProperty table `0x141e02f00` selects Link
`0x1407c7c50`; ObjectProperty table `0x141e11088` and ClassProperty table
`0x141e116f8` select the same Link `0x1407c7cb0`. Each sets element size8.
All three alignment slots select the already retained return4 leaf
`0x1401545a0`. Object/Class linking additionally updates property flag bit22
only when bits26 and3 are set and bit19 is clear; that does not silently change
the measured width. These are compiled methods, not assumed pointer packing.

`core-header-replay.json` retains the original 13-row replay from initial
extent0, exact input hashes and the connection to the previous Texture replay.
It distinguishes this conditional computation from a live linked class.

The owned Surface declaration inherits Core.Object and has exactly two
Function children, no instance properties. Its ordinary inherited result
is therefore0x60. The named native Surface factory independently passes
instance extent0x60, and Texture's saved properties then produce0xfc as in
the previous increment. Constructor values remain cross-checks rather than
inputs that override the recomputation algorithm.

The metadata receipt is
`surface-factory/core-object-metadata/receipt.json`, SHA256
`2af66185398e0142b14eb75f43ac0b9be7c637d962475cdb0f088f9ce3157e0b`.
Its raw fields, exact type references, category/flag words, complete child
chain and QWord declarations remain private.

## Texture2D has a version-gated OriginalSize overwrite

The concrete Texture2D constructor `0x1409a08d0` installs table
`0x141ecbe70`. Its exact +0x68 slot at `0x141ecbed8` selects the complete
513-byte serializer `0x1409c07e0..0x1409c09e1`, across three adjacent unwind
fragments. The Serialize slot role is independently retained in
[ENTRY_PACKAGE_OBJECTS.md](ENTRY_PACKAGE_OBJECTS.md), with the canonical
UObject table/serializer evidence; it is not chosen from a remembered SDK.

This serializer first calls shared UObject serialization `0x1407a7720`, then
payload helpers. At `0x1409c08e0` it compares signed archive dword+0x08 against
**0x273**. Below that threshold it copies object+0x180 to+0x188 at
`0x1409c08ef`, then object+0x184 to+0x18c at `0x1409c08fb`. No additional
load/save flag gates these two local writes. At or above the threshold the
two writes are skipped.

A subsequent condition uses archive+0x08 <0x27a, byte object+0xc4 equal7,
and a difference between the two pairs to write object+0xf8=1. The body has
no direct stores to+0x180/+0x184. Neither observation proves whole-function
or whole-lifecycle preservation: shared UObject serialization and the invoked
helpers retain their own effects.

The selected owned package's file version is868, but the producer of this
invocation's archive+0x08 has not been traced here. Thus package version alone
does not discharge the comparison. The stored tooltip's equal64 pairs also
cannot show which branch ran. No native load was observed.

One narrow tagged-value handoff is also closed. The retained complete
non-Bool tag-value helper `0x140757b60..0x140757c28` calls property virtual
+0x290 at `0x140757bff`, with the archive and supplied value destination.
IntProperty's exact slot `0x141e02b20` selects the complete 20-byte leaf
`0x1407cc610..0x1407cc624`. It tail-calls `0x1406d66b0` with that archive,
the same destination and byte count4. The complete 121-byte helper calls
archive virtual+0x08 once with four bytes when archive+0x4c is zero; otherwise
it makes four one-byte calls at destination+3, +2, +1 and +0. The callback
has no separate size/default substitution. Correct property selection,
destination bounds, archive mode/error behavior and the effects of later
handlers remain conditions; this is not evidence that a live four-byte read
succeeded.

## Concrete frontier

The initial inherited size no longer needs an unexplained numeric0x60 input
in a **conditional ordinary-link replay**. Admission still needs the actual
selected metadata/type callbacks, successful superclass preload, and either
exclusion or exact application of the reorder/relink path. The class serializer
archive+0x14 gate and UClass forwarding are now explicit native facts.

The next dimension-preservation boundary is narrower: the native archive
version producer, successful tagged-property selection/archive behavior and the actual
post-load dispatch/handlers before the wrapper's two separate size reads.
This study has no verified PostLoad slot binding and does not guess one.
It also does not supply current Q flags, a retained backing-resource lifetime
or GPU mip dimensions. No renderer input is filled from coincidentally equal
saved values, and no renderer profile is enabled.

## Source ledger and validation

All primary bytes are from the same owner-pinned proprietary executable,
SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and the private hash-pinned Core/Engine packages identified in the preceding
document. They were read solely for authorized compatibility research.
The prior original property-link, Pointer, name-publisher, UStruct Link,
serializer-slot and class-factory evidence was reused, including
`scale9-texture-inputs`, `localized-result-object`, `entry-package-objects`
and `world-transform-defaults`. `current-movie-defaults/native.bundle.json`
and `native-verification.json` supply the complete tagged loader and tag-value
helper, particularly `ev_1f25a36082766ba6b9cd3d9c75a77c1e200671ba5deff8302a51785a73cb194e`
and `ev_a86211bb5b4ac356cff1ec42b5c48b338c28857a91130950ab41e2a955475d43`.
Their exact pinned-executable identity and complete byte-validation receipt
were checked before reuse. Each new private subdirectory contains its
exact source/hash ledger. No outside implementation source or SDK was read.

REA target-free Rizin supplies canonical bounded instructions and exact slot
or literal bytes; an independent original PE reader compares every returned
byte against the pinned executable. Original parser/check programs inspect
the saved declarations. These are static evidence and arithmetic checks, not
execution of native game code or proof that a live archive selected a branch.

Three new canonical records passed exact PE comparison: 1,520 instruction
spans / 6,426 code bytes and 186 data bytes. Every requested output returned;
all records report complete execution, no truncation and an unchanged artifact:

| Scope | Evidence ID |
|---|---|
| Complete UStruct serializer, UClass Link and superclass getter | `ev_537dd6bdc72e0f8393c9864194a7624a899a6dab28caada7f8b391d7f07ec2a9` |
| Texture2D constructor/serializer and QWord registration | `ev_8f3822e6d832f8fb970cbfa9158ba9e610ea0914ae8c6b77942523ff8f091a79` |
| Header-property widths, Surface factory chain and IntProperty archive handoff | `ev_a8b67c192babf4def04d0ef33d628f79c2e399fc3a75c5964d7e007bbeaac355` |

`verification/final-verification.json`, SHA256
`280e011f5fc7afc859877e93425b0ee869934c2a2831e694778aa11decd3b6fc`,
retains independent byte and selected operation-order checks.
`core-header-replay.json`, SHA256
`6f605a624e75509208941874a7a614e234d6d8d549c26d08b12c29231ac53fd9`,
records PASS for the conditional header extent and bridge to the prior replay.
The full accumulated 73-record `complete.bundle.json` has SHA256
`eb1393ebcdf6d52112a7b5c4f927a302de6892720a633b349fde7ea023d994a7`.
REA resource close completed. No full suite or native/GPU smoke was run.
