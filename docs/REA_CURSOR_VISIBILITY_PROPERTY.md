# Cursor visibility property: conditional native layout

**VERIFIED saved declarations and native Link operations; INFERRED conditional
layout:** the owned Engine declaration `GameViewportClient.bDisplayHardwareMouseCursor`
maps to object offset **`0x108`, mask `0x02`**, under the explicitly admitted
ordinary recomputation profile below. The calculation uses the declarations and
typed native Link methods before comparing its result with the cursor consumer.
It does not identify a current linked UProperty, viewport or Boolean value.

This closes the named-layout gap in [REA_CURSOR_BACKEND.md](REA_CURSOR_BACKEND.md)
for that profile. The earlier document's unconditional/current-state limitation
remains: loaded metadata, reordering, callbacks and later writes must not be
assumed to match this saved-order computation.

The source is the private owner-held 1.52.0.1700 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and pinned Engine/Core packages. Addresses below are preferred image VAs.
No original process, cursor/input API, service or network operation ran.

## Saved declaration and its dependencies

**OBSERVED, bounded bytes checked:** Engine export `30406`, reference `30407`,
is the directly owned BoolProperty `GameViewportClient.bDisplayHardwareMouseCursor`.
Its ArrayDim is `1`, flags are `0x2000`, and Next is reference `30409`. Its
serialized declaration does not contain a decoded native offset or Boolean
mask. The remaining common word at saved offset `+36` is preserved without
assigning it an offset meaning.

The complete ScriptViewportClient child chain contains one instance property,
`VfTable_FViewportClient`, a scalar StructProperty referencing `Core.Object.Pointer`.
Its saved superclass is Core.Object. GameViewportClient's complete owned chain
contains 106 children, including 36 instance properties. Seventy non-property
children precede the first instance property. The native property loop skips
those children; it does not allocate instance storage for them.

The selected GameViewportClient prefix contains three Pointer mirror fields,
one ArrayProperty, three Class/Object properties, one ShowFlags mirror, five
StrProperties, then consecutive scalar Booleans `bShowTitleSafeZone` and
`bDisplayHardwareMouseCursor`. Every selected property has ArrayDim1 and the
expected local owner. This order comes from Children/Next references, not from
export indices or matching native values.

ShowFlags' referenced `ExportShowFlags_Mirror` has no own children and inherits
`ShowFlags_Mirror`. The latter has SuperField0 and exactly two scalar
StructProperties, `flags0` then `flags1`, both referencing `Core.Object.QWord`.
The retained QWord rule gives size8/alignment4, so their ordinary linked extent
is size16/alignment4. No viewport cursor offset is used to choose that extent.

## Concrete Link selection and widths

**VERIFIED:** the named native GameViewportClient builder supplies constructor
`0x14073a870`, whose installed UClass metadata table is `0x141df82c8`.
The retained complete UStruct serializer `0x140751330` reaches virtual
`+0x278` with the supplied archive and third argument `1` when archive DWORD
`+0x14` is nonzero. Table `+0x278` selects complete UClass Link `0x14074a2d0`,
which forwards that mode unchanged to complete UStruct Link `0x14074a910`.
This is a concrete serializer branch, not an assertion about a running archive.

Nonzero mode clears the owner's computed extent, obtains the superclass's
extent/alignment through the archive, and rounds the initial extent to four
bytes. It follows owner Children `+0x80`, then each field's Next `+0x60`,
exiting at the first field whose outer differs. Fields without the property
class mask are skipped. Each accepted property receives the previously
accepted property as its third argument. The owner extent becomes
`property.offset + property.element_size * ArrayDim`, with 32-bit arithmetic.

The selected typed methods establish these widths; ordinary callee return and
the actual referenced metadata are conditions. Their owner-extent branch
requires owner.class DWORD `+0x128` bit `0x08`; this qualification is an explicit
input of the admitted ordinary class/struct profile:

| Type | Size / alignment | Native basis |
|---|---|---|
| Pointer and QWord mirrors | `8 / 4` | Retained exact name-registration overrides and saved mirror declarations |
| ObjectProperty / ClassProperty | `8 / 4` | Named tables select `0x1407c7cb0` and the return4 alignment leaf |
| NameProperty | `8 / 4` | Named table selects `0x1407c7c50` and return4 |
| IntProperty / FloatProperty | `4 / 4` | Named tables select `0x1407c7b20` and return4 |
| ArrayProperty | `16 / 4` | Named constructor table `0x141ded6a0`, Link `0x1407c78f0` |
| StrProperty | `16 / 4` | Named constructor table `0x141ded368`, Link `0x1407c7d30` |
| BoolProperty | `4 / 4`, packed mask | Named table `0x141e10d50`, Link `0x1407c7990` |

Array Link first preloads its inner property through archive `+0x20` and
invokes the inner Link with third argument zero. Those calls must return with
the admitted metadata. Str Link sets size16 and aligns the qualifying owner's
extent; its named builder `0x1407c1230` and retained constructor `0x1407c70b0`
make this a typed width, not a host-language string-size assumption.

For a fresh Boolean word, Bool Link writes mask1 and aligns the qualifying
owner extent to4. It instead reuses the previous Boolean's offset and doubles
its mask when the previous property has the Boolean class bit, owner virtual
`+0x260` is nonzero, and the previous mask has any low-31-bit bit set. The
concrete UClass owner table selects a return1 leaf for that policy. Scalar,
one-hot, same-owner metadata are established inputs here; the Bool method does
not independently validate all of them. The mask is a DWORD field on the
UProperty at `+0xa8`, and the native byte offset is its DWORD `+0x8c`.

## Independent prefix calculation

The retained complete Core.Object header declarations derive extent `0x60`
from a null superclass. ScriptViewportClient's Pointer adds eight bytes,
giving `0x68`. From there the independently replayed GameViewportClient prefix
is:

| Saved property / group | Offset | Extent after linking |
|---|---|---|
| VfTable_FExec | `0x68` | `0x70` |
| Viewport | `0x70` | `0x78` |
| ViewportFrame | `0x78` | `0x80` |
| GlobalInteractions | `0x80` | `0x90` |
| UIControllerClass | `0x90` | `0x98` |
| UIController | `0x98` | `0xa0` |
| ViewportConsole | `0xa0` | `0xa8` |
| ShowFlags | `0xa8` | `0xb8` |
| LoadingMessage | `0xb8` | `0xc8` |
| SavingMessage | `0xc8` | `0xd8` |
| ConnectingMessage | `0xd8` | `0xe8` |
| PausedMessage | `0xe8` | `0xf8` |
| PrecachingMessage | `0xf8` | `0x108` |
| bShowTitleSafeZone | `0x108`, mask1 | `0x10c` |
| **bDisplayHardwareMouseCursor** | **`0x108`, mask2** | `0x10c` |

The original 29-row replay includes the inherited header. Its raw consumer
comparison runs only after computing the result. The package study independently
derives the same prefix from its extracted declarations. The native builder's
declared instance size is a separate constructor checkpoint, not the source
of these individual offsets.

Under this admitted profile, the named property used by the source-script
visibility assignment in [REA_HARDWARE_CURSOR.md](REA_HARDWARE_CURSOR.md)
denotes the same word/bit read by the selected cursor client at `C+0xa8`,
where `C=V0+0x60`. No script interpreter execution was performed here.
Retained Boolean-tag application also uses the property mask to OR a true
value or clear a false value in the destination word. This does not make an
absent serialized tag a false value or establish any effective current default.

## Conditions that remain material

**VERIFIED alternate path:** zero Link mode snapshots and restores property
offsets, and restores Boolean masks, around property linking. Saved declaration
order alone cannot reconstruct those preserved inputs.

On nonzero mode, owner object flag bit21 together with nonzero archive `+0x14`
admits the retained reorder block. It can rewrite Children/Next, clears the
bit, and recomputes. A saved flag or initial chain does not prove later flags,
patch candidates or final order. Referenced mirror metadata and inherited
extents must also retain the admitted loaded definitions through callbacks.
Later UClass work and subsequent native/script writes are separate checkpoints.

The named class initialization's previously unresolved virtual `+0x118` is now
bound to complete `0x14074e2c0`. It calls object-registration helper `0x1407a3dd0`,
conditionally registers the superclass through `0x1407973a0`, resolves the
configuration name, inherits selected class flags, and tails the retained
default resolver `0x140745a20`. The complete 119-byte body and selected
655-byte object-registration body do not directly assign the target property's
offset/mask. Unexpanded dependent calls are not claimed absent or harmless.
This edge supplies no substitute for the concrete loaded Link conditions.

Finally, the backend factory retained `C=V0+0x60`, while frame attachment
later reloaded receiver `V1`. Joining a script assignment on `V1` to the
callback on `V0` still requires actual receiver identity/preservation. Setter
callbacks and backend availability retain their earlier gates. MovieImpl's
state-4 `Mouse.hide` handler remains a separate unresolved provider; this
property layout does not install or identify it.

## Evidence and validation

All source bytes and detailed outputs are private under
`~/.local/share/marvel-heroes-forever-rust/evidence/cursor-visibility-property/`.
Aggregate `receipt.json`, SHA-256
`885685e390cc0cec0b768610800e1111b731e5d8c27f2ec3e344ff10d24c827a`,
rechecks **200 referenced file hashes** and binds the conditional replay.

- `declaration/` contains bounded Engine/Core reads, complete owned child
  coverage, mirror dependencies and the separate saved-order arithmetic.
- `bool-native/receipt.json` rechecks 106 instructions / 443 code bytes and
  50 name/table bytes. Its linked `link-audit/` checks 1,405 instructions /
  5,918 code bytes and 24 slot bytes for mode/order handling.
- `bool-native/array-width/receipt.json` checks 92 instructions / 378 code
  bytes and 44 name/table bytes. Only its eight-byte alignment slot required
  a new capture; the method bodies were reused.
- `native-link/receipt.json` checks 349 instructions / 1,557 code bytes and
  48 data bytes across three new records and three selected retained records.
  Complete unwind extents, explicit leaf endings, exact slot/name bytes and
  immutable input hashes are checked independently against the PE image.
- `replay.py` / `layout-replay.json` retain all 29 prefix rows and the exact
  profile conditions. They do not execute native code or seed the computation
  with the observed cursor offset.

New native records are `ev_d21cc7fb96c26ed70cd5365c13af2b53cde623120a5c4bbeab67c73409de61b6`
(class slot/registration), `ev_ef5fd17372e7623112be0604b5feddc804883f0d61b953172dfcca3f0e0a4b38`
(base registration), `ev_c6db7ebcc73526f9abab5b2eef017c1e6cefbecc3682b2601d2dac96f9cf59a6`
(named string Link), and `ev_4f15248c1676beb7eb3baa610f8bf8467e4511325c9915dd08b28fcd6ef9d96e`
(array alignment slot). All captures completed without truncation or source
mutation. This documentation changes no product code; no startup, input or
visual comparison ran.

## Source and licence ledger

| Source | Licence / use |
|---|---|
| Owner's pinned executable and Engine/Core packages, exact paths/hashes in private receipts | Proprietary owned files; private compatibility study of saved declarations and native layout rules |
| Original `REA_CURSOR_BACKEND`, `REA_HARDWARE_CURSOR`, `FRONTEND_MOVIE_DEFAULTS`, `SCALE9_TEXTURE_INPUTS`, `SCALE9_LOADED_LAYOUT`, `LOCALIZED_RESULT_OBJECT`, `FRESH_MOVIE_DEFAULT_STATE`, `WORLD_TRANSFORM_DEFAULTS` and their private evidence | Existing project research and provenance; retained native identities, serializer/Link contracts and declared mirror widths |
| Existing original project UPK/class readers and private extraction/checking scripts | Original code; exact source hashes retained; Python standard library under PSF licence |
| Installed REA skill/native guide, REA 6.9.0 and configured Rizin | Existing ledger records REA MIT and Rizin LGPL-3.0-only; tooling only, no implementation copied |

No external implementation, SDK, new dependency or current owner state was
introduced into the Rust client.
