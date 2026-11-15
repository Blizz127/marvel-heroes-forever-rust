# Saved movie defaults and native application boundaries

The selected OpenMovie action has a real class default that explicitly stores
`bDisplayWithHudOff=true`. The selected frontend player has a separate class
default that explicitly stores `bDisplayWithHudOff=false`. These are defaults
for different objects. Neither value alone establishes the player's value when
Start is invoked. `bStartPaused` is declared on OpenMovie but is absent from
both the saved map instance and the action's complete saved default list.

This study identifies the actual class/default links, verifies Boolean tag
application, and narrows native flag packing. It does **not** replace missing
values with false or implement an effective-default adapter. Class loading,
configuration, native field registration and later initialization still have
material conditions. No sequence, original handler, native client or service
was executed.

## Saved identities and explicit values

**OBSERVED**, with raw package spans **VERIFIED**: the four class records
consume their complete bounded 868/3 profiles. Each terminal class-default
reference points to the recorded default export; this is stronger than finding
an object whose name starts with `Default__`. Positive references below are
one-based export references. Export indices elsewhere are zero-based.

| Package and class reference | Terminal default reference | Default's serialized archetype |
| --- | --- | --- |
| GFxUI, GFxAction_OpenMovie +146 | +147 | Engine.Default__SequenceAction, import -55 |
| GFxUI, GFxMoviePlayer +37 | +38 | Core.Default__Object, import -46 |
| MarvelGame, MarvelGFxMoviePlayer +8539 | +8540 | GFxUI.Default__GFxMoviePlayer, import -727 |
| MarvelGame, MarvelGFxFrontEnd +8765 | +8766 | Default__MarvelGFxMoviePlayer, local +8540 |

Class-super references independently agree with these declared inheritance
paths. The serialized UClass grammar is an original bounded profile previously
corroborated against UELib; complete consumption does not prove every field's
runtime purpose. The native serializer below independently supports the
terminal default pointer, flags and configuration-name fields.

All four default objects have complete property lists and zero remaining native
tail. Their relevant explicit contents are:

| Source object | Explicit saved input | Logical value offset |
| --- | --- | --- |
| OpenMovie default +147 | MoviePlayerClass → GFxUI.GFxMoviePlayer +37 | 80424 |
| OpenMovie default +147 | bDisplayWithHudOff = true | Boolean byte 80452 |
| GFxMoviePlayer default +38 | bDisplayWithHudOff = true | Boolean byte 73762 |
| GFxMoviePlayer default +38 | bAllowInput = true; bAllowFocus = true; bCloseOnLevelChange = true | Boolean bytes 73812, 73837, 73862 |
| GFxMoviePlayer default +38 | Priority = 1; SplitscreenLayoutYAdjust = -27 | 73895, 73920 |
| MarvelGFxMoviePlayer default +8540 | m_uninitializedPanelCount = 1 | 2362534 |
| MarvelGFxMoviePlayer default +8540 | bAutoPlay = true | Boolean byte 2362601 |
| MarvelGFxMoviePlayer default +8540 | TimingMode = saved enum name TM_Real | 2362758 |
| MarvelGFxFrontEnd default +8766 | MovieInfo → SwfMovie MarvelFrontEnd.frontend_manager +17005 | 2384555 |
| MarvelGFxFrontEnd default +8766 | bDisplayWithHudOff = false | Boolean byte 2384583 |
| MarvelGFxFrontEnd default +8766 | Priority = 2 | 2384616 |

The MarvelGFxMoviePlayer default also saves UIPackageName and one SoundThemes
entry. The FrontEnd default saves RememberUsername=true, 19 blackout-list
names, 13 PanelData records and 13 WidgetBindings records. These are packaged
defaults, not inspection of an owner's remembered username or runtime account
state. This increment retains array structure and spans without interpreting
their contents as initialized panels.

The actual map instance explicitly selects MarvelGFxFrontEnd and
frontend_manager, overriding the action default's generic MoviePlayerClass.
It also explicitly saves bTakeFocus=true and bCaptureInput=true. Its selected
route and source spans remain in [FRONTEND_STARTUP_MAP.md](FRONTEND_STARTUP_MAP.md)
and [FRONTEND_INITIAL_INSTANCE.md](FRONTEND_INITIAL_INSTANCE.md).

`bStartPaused` is an actual BoolProperty declaration, GFxUI export 137,
reference +138, outer +146. Its absence is therefore not a missing declaration
or a failed property parse. It is an absent saved tag on a known field.
ExternalInterface and bInitialized are absent from all three player default
lists; their declarations do not make those absent values null or false.

## Native class and Boolean contracts

**VERIFIED static instructions**, exact owned PE bytes checked:

- UClass's established primary table `0x141df82c8`, slot +0x68, points to
  complete serializer `0x140750850..0x140750ad0` (640 bytes). After the parent
  serializer, it loads the flags word into class+0x124 and the configuration
  FName into class+0x138. The loading branch passes class+0x1d4 to the archive
  object-reference reader at `0x140750a63`. This agrees with the retained
  default resolver's cache field, and supports the profile's terminal default
  reference. If global `0x142aee960` is nonzero it then calls `0x14079d590`;
  pointer deserialization alone does not finish default initialization.
- The exact BoolProperty class builder is `0x1407c0940`, using its owned
  UTF16 name, cast mask 0x20000 and constructor `0x1407c6d80`. The constructor
  installs table `0x141e10d50`. Its property-link slot +0x280 is
  `0x1407c7990`; the runtime field mask is property+0xa8 and byte offset is
  property+0x8c. The element size is four bytes. This four-byte storage word
  must not be confused with the one-byte Boolean in a saved property tag.
- Complete tagged loader `0x140755f10..0x140757b5f` (7,247 bytes) reaches
  value application helper `0x140757b60..0x140757c28` (200 bytes). The helper
  checks the exact BoolProperty class, loading flag and tag's inline byte.
  A true byte ORs the field's mask into the destination word; false ANDs its
  complement. An explicit false tag really clears its selected bit. It is not
  equivalent to an absent tag. Other types dispatch through property slot
  +0x290. The previous complete tag reader supplies the inline Boolean byte
  for version 868.
- Complete class link `0x14074a2d0..0x14074a900` calls struct link
  `0x14074a910..0x14074b923` (4,115 bytes). Its field-relink mode traverses
  class+0x80 Children, then field+0x60 Next, restricting local fields by outer
  identity and property cast mask 0x8000. It passes the previous property as
  the third argument to each property-link method. A distinct mode preserves
  existing property offsets and Boolean masks around the same call.
- BoolProperty link starts a word at mask 1, aligned to four bytes, or shares
  the preceding Boolean's offset and doubles its mask if the owner permits
  packing and the previous mask has a low-31-bit bit set. UClass's owner
  predicate slot +0x260 returns 1. These facts establish the algorithm, but
  do not select the relink mode or reconstruct an earlier native registration.

The action's local linked declarations are `Movie`, `MoviePlayerClass`,
`MoviePlayer`, then the consecutive Booleans `bTakeFocus`, `bCaptureInput`,
`bStartPaused`, `bEnableGammaCorrection`, `bDisplayWithHudOff`, followed by
RenderTextureMode, RenderTexture, CaptureKeys and FocusIgnoreKeys. Its initial
Children item is a function followed by Movie; non-property fields are skipped.
The entire 13-node chain has checked references, local outers and bounds.

**INFERRED, conditional on the field-relink mode:** those five Booleans receive
masks 1, 2, 4, 8 and 16 respectively. Do not promote that conditional result
to a captured native layout: the offset-preserving mode is concrete evidence
that declaration order alone is insufficient for every loading path.

## Action-to-player writes and configuration

The previously complete action body `0x141262880..0x141262e63` writes the movie
source into player+0x10c after construction. That explicit assignment supplies
frontend_manager independently of whether the FrontEnd CDO's matching
MovieInfo tag was inherited. It performs these exact flag transfers before
reflected Start, where A is the action and P is the player:

| Native operation | Verified bit operation | Name-mapping status |
| --- | --- | --- |
| `0x141262aa2` sequence | A+0x120 bit 3 → P+0x114 bit 2, other bits preserved | Action-side bEnableGammaCorrection only under the relink inference |
| `0x141262ac0` sequence | A+0x120 bit 4 → P+0x114 bit 1, other bits preserved | Action-side bDisplayWithHudOff only under the relink inference |
| `0x141262d2c` sequence | A+0x120 bit 2 → reflected Start's first Boolean argument | Named StartPaused by retained call/schema evidence; effective bit unresolved |

The first two transfers must not be labeled bTakeFocus/bCaptureInput merely
because those two tags are explicit on the map instance. The conditional
declaration mapping instead associates them with gamma correction and HUD-off
display. This study does not prove the destination property's named mask.

It is consequently wrong to merge the action and player default tables by
property name, or to let the closest frontend default automatically win at
invocation. A checked implementation needs separate action and player state,
then the recovered action assignments in their native order.

The saved class flags also expose a real initialization condition:

| Class | Saved flags | Configuration name | Class bit 0x4 |
| --- | --- | --- | --- |
| GFxAction_OpenMovie | 0x08000092 | None | Clear |
| GFxMoviePlayer | 0x08000096 | UI | Set |
| MarvelGFxMoviePlayer | 0x08000096 | UI | Set |
| MarvelGFxFrontEnd | 0x08000096 | UI | Set |

The retained default-finalization helper `0x14079de30` tests that class bit.
Player configuration is therefore not globally excluded. This does not assert
that every listed property is configurable; individual property metadata and
the helper's filters still apply. The action's saved flag excludes that one
gate if flags remain unchanged. All four classes clear 0x20 and 0x10000000;
neither observation excludes every later callback or write. No configuration
file or private user setting was inspected for this increment.

## What a later checked adapter may admit

The saved evidence can supply typed provenance immediately: explicit instance
tag, explicit linked-class-default tag, declared-but-absent tag, unsupported
class profile, or missing class/default resource. These categories must remain
separate from effective runtime state and from actual invocation.

The following remain necessary before deriving an effective omitted value:

1. Select the actual class link/native registration path and verify the final
   relevant field offsets/masks, including destination player masks.
2. Establish the relevant allocation/default-source extent and package default
   application order, with field-specific configuration and subsequent-write
   gates. Fresh allocation zero-fill alone is insufficient.
3. Apply the action's explicit overrides and native writes in order, then pass
   the actual resolved player/receiver and Start arguments. The saved graph
   does not establish execution, a live local player, or a null receiver.

No product adapter is added in this receipt because those conditions remain
unresolved. In particular startup preparation must retain unknown effective
StartPaused and receiver state. It may report the newly established explicit
action default and the separate player defaults without conflating them.

## Evidence, validation and provenance

All raw data, disassembly and tooling output remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/current-movie-defaults/`.

- `package-audit/receipt.json`, SHA256
  `e6f72cfff5870a82c2c5b621c853d7ed9cc1a2cf82d2898919c7ad1047051013`:
  original Rust parser checks 39 selected exports, four complete default lists,
  nine Boolean bytes and the complete action declaration chain. Twenty-one
  retained extracts match fresh reads byte-for-byte; both package hashes are
  unchanged. Four bounded UClass profiles consume 4,367 bytes exactly.
- `default-contract/followup/receipt.json`, SHA256
  `23f0882e8b8ceb62d50e2ae8cac44ae6b42e725a08f06ba9c6b6415cd12e0ce1`:
  class flags/configuration spans and complete UClass serializer verified;
  tagged-loader pdata has no direct or transitive chained fragments.
- `native.bundle.json`, SHA256
  `e12a42d077ae61cc8344f60da03a6dc36a48faf9a2031afcd12fa824420d41ce`:
  six successful, complete, untruncated REA target-free Rizin records. Key IDs:
  `ev_faca4d7535519ce6911cb773e40c8730974e1d201237fed857388740a282eae6`
  (UClass), `ev_b8ca0468842cf7b62869e08d5cac795b87ebe28f7714aa508cc6a737a6089c3e`
  (Boolean methods), `ev_1f25a36082766ba6b9cd3d9c75a77c1e200671ba5deff8302a51785a73cb194e`
  (tagged loader), `ev_a86211bb5b4ac356cff1ec42b5c48b338c28857a91130950ab41e2a955475d43`
  (tag application/class link), and
  `ev_db0f320d27d2b9e043aa4f7e3aceed5f039cc24f5db718d9790c4a81ed305604`
  (struct link). The sixth record independently repeats the small exact-name
  BoolProperty builder; its earlier retained proof was found afterward.
- `native-verification.json`, SHA256
  `b79fa39d60b7acdbaf03a2a5d2217fbbdca52a3d2cd7be5602757184817a15b7`:
  original `verify_native.py` checks 3,685 instruction rows / 15,689 bytes and
  eight vtable pointers against the owned PE. An initial constructor probe
  requested 39 bytes beyond its actual 57-byte body; the overrun, padding and
  partial next-function rows are explicitly excluded. Executable hash unchanged.

REA records were exported and the investigation reset; no active target was
opened. No routine full suite or product build was appropriate for this
documentation-only study. Package parsing and native byte verification are
static compatibility evidence, not a native object snapshot or retail parity.

Sources read or reused, with their scope:

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned MarvelHeroesOmega.exe, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-supplied build 1.52.0.1700; private compatibility study | Bounded static native contracts and exact bytes |
| Owner's GFxUI.upk, SHA256 `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199`; MarvelGame.upk, SHA256 `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` | Same proprietary owner assets | Actual class, declaration and complete default records |
| Owner's MarvelFrontEndMap.upk, SHA256 `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` | Same proprietary owner assets; retained original parse reused | Exact selected instance and startup route |
| [Unreal-Library](https://github.com/EliotVU/Unreal-Library), commit `8236c39b79e30887a487ecf9b018204e430a5ebc` | MIT, Eliot van Uytfanghe | Prior corroboration of the original bounded UClass profile; no new external implementation was read or copied here. Exact earlier paths/hashes are in `evidence/world-transform/defaults/class-default-link/sources.json` |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/native-and-artifacts.md`, `references/evidence-workflows.md`; installed rea-agents 6.9.0 | MIT toolkit, as recorded in the existing REA provenance | Target-free inspection, Evidence handling and bounded recovery |
| Rizin via installed REA; prior GNU objdump 2.46 output | Rizin LGPL-3.0-only; GNU binutils GPLv3-or-later; tool use only | Disassembly; no external implementation copied |
| Original project docs FRONTEND_STARTUP_MAP, FRONTEND_INITIAL_INSTANCE, FRONTEND_EXTERNAL_RECEIVER, FRONTEND_VISTA, FRONTEND_COMPOSITION, WORLD_TRANSFORM_DEFAULTS, ANIMATION_RUNTIME and FRONTEND_LOCALIZATION_RUNTIME | Original project work and its existing provenance | Reused scope, package and native findings |

The private `default-contract/sources.json` lists all 27 reused evidence paths
and their hashes, including entry-package-objects tagged-property receipts,
world-transform default-copy receipts and frontend-localization Boolean identity
proof. The package-audit receipt lists its original parser/profile inputs.
No new third-party source implementation, SDK, network request or service
interaction was used. All new verification and analysis code is original.
