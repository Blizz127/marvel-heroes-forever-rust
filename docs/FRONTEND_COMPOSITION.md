# Retail front-end composition evidence

**OBSERVED:** The saved front-end map opens `marvelfrontend.frontend_manager`,
not the login dialog directly. The login and character-select movies have real
image and shared-movie dependencies that can be resolved from package records.
These findings describe stored relationships; native startup execution, movie
actions, focus behavior and visual parity have not been observed.

## Saved map graph

In `MarvelFrontEndMap.upk`, the `Loaded and Visible` output of the LevelLoaded
event contains six explicit saved links:

| Target | Saved input / relevant property |
| --- | --- |
| GFxAction_OpenMovie | `In`; Movie=`marvelfrontend.frontend_manager`; MoviePlayerClass=`marvelgame.marvelgfxfrontend`; focus/input-capture flags true. |
| ToggleCinematicMode | `Enable`. |
| Interp | `Play`; saved looping/rewind/force-start flags true; its `Completed` output links back to its own `Play`. |
| ToggleHUD | `Hide`. |
| SetVelocity | `In`. |
| MultiLevelStreaming | `Load`; level name `marvelvista_asgardhub`; unload-other-levels flag true. |

The camera interpolation record describes a UI background scene of approximately
62.10246 seconds. The LevelStreamingKismet package name independently identifies
the Asgard vista. This does not establish runtime loading order or callback timing.

A music action is present in the sequence. Its tagged MusicTrack record contains
auto-play and persistence flags, but this map contains no explicit sound-cue
reference or incoming saved output link to that action. Default values and runtime
triggers remain unknown; the previously sampled music package name alone does not
establish the front-end soundtrack.

**OBSERVED coverage:** 24 of 37 map exports fit the bounded four-byte preamble plus
tagged-property layout. Thirteen actor/component preambles and the native World,
Level and model payloads remain unresolved. No arbitrary offset scanning or guessed
actor-stack skip was used. Exact property offsets and seven saved graph edges are
in private `evidence/frontend-map-startup.json`.

## Movie dependencies

The 20 SwfMovie exports in `MarvelFrontEnd_SF.upk` omit `frontend_manager`; that
object is an import there as well as in the map. A focused, separately pinned
startup-package investigation located the real object:

- **OBSERVED:** `MarvelGame.upk` export 17004 is SwfMovie
  `MarvelFrontEnd.frontend_manager`, at logical offset 5,746,617 with serialized
  size 2,055,945 bytes. Its RawData is a 2,055,748-byte GFX payload and its
  `bUsesFontlib` property is true. The matching `MarvelGFxFrontEnd` class is export
  8764 in the same package. This resolves stored object identity, not script or
  native execution.
- **OBSERVED:** `Startup.upk` contains SwfMovie `marvelflash.fonts`, export 3628.
  No export named `gfxfontlib` was found in the four focused startup packages.
  The alias between the font-movie object and movie import paths remains
  unestablished at this stage.

The four complete copied packages (`Startup`, `GFxUI`, `MarvelGame`, and
`MarvelVista_AsgardHub`) match the owner's original inventory hashes; their
private copy is under `pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/`.
Evidence and original extracts are under `evidence/frontend-startup/`, with
identities in `source-checks.json`. No native client was launched.

The manager serializes 11 texture references, but no ReferencedSWFs property.
Those references select `MarvelGame.upk` exports 17037–17047 under the cooked
outer package `MarvelFrontEnd`, including cursor/dropdown assets and three
render-target-named textures. An absent property does not establish an empty
runtime value; class defaults and script behavior remain separate questions.

**OBSERVED:** Login and character select both declare a 1920 × 1200 GFX stage.
Their ImportAssets records name these libraries:

| Library identifier in the movie | Observed examples |
| --- | --- |
| `../MarvelHUD/!UISource.swf` | Shared buttons, input fields, checkboxes, loaders and tooltip symbols. A `!uisource` movie export is present in the sampled front-end package. |
| `marvel_assets.swf` | Scrollbar/loader symbols; the corresponding `marvel_assets` export is present. |
| `../MarvelFonts/gfxfontlib.swf` | `$Strat2Medium` and `$Strat2Bold`; `marvel_assets` additionally imports `$RobotoBold`. The library is not among the 20 sampled exports. |

These strings are asset identifiers, not permission to fetch URLs or traverse
filesystem paths. Movie symbol IDs are local to their movie dictionary. The
serialized ReferencedSwfs and References properties corroborate the shared UI
movie links; references to the font library may also depend on the saved
font-library flag and startup state.

The original Rust package/property/GFX readers recorded every movie import in
private `evidence/frontend-movie-dependencies.json`; no ActionScript was executed.

## Bitmap-to-texture bindings

**VERIFIED static cross-check:** All 716 external-image records across the 20
movies have one matching object in that movie's serialized References array:
677 records select local Texture2D exports and 39 select imports. These cover all
582 local texture exports and 11 distinct imported texture objects.

**OBSERVED extended resolution:** Every one of those 39 imported image records
also has a unique full outer-object-path match in `MarvelGame.upk`, covering 11
Texture2D exports. The stored file name and the object's outer package name are
different: `MarvelGame.upk` contains `MarvelFrontEnd.*` objects. File-name-only
package resolution would miss them. Saved payload availability does not prove
the runtime contents of render-target-named assets remain unchanged.

The matching procedure used for this check is:

1. Read each movie's own References array and retain full object identity,
   including import/export sign and outer package.
2. Form a name candidate from the image export name. Six records have an empty
   export name and instead match the image filename with its final extension
   removed. Two `!uisource` names containing `.png` match `_png` package names.
3. Compare names without ASCII case differences, then intersect candidates with
   that movie's References set. Require exactly one candidate.

The runtime normalization implementation is **INFERRED**, including the filename
fallback and the two punctuation substitutions. The matching names, dimensions,
reference membership and unique candidate counts are **OBSERVED**. Future assets
with zero or multiple candidates must remain unresolved rather than selecting one
by traversal order. Eighty-eight records have duplicate name candidates before
reference filtering; a global last-name-wins map is incorrect for these samples.
The image ID is not the References-array position, and reference ordering is not
bitmap-ID ordering.

| Movie / bitmap ID | Resolved object | Scope |
| --- | --- | --- |
| Login / 41 | `frontendbg_2560x1440`, export index 168 | Local inline texture, 2560 × 1440. |
| Login / 40 | `legaldocrendertarget`, object reference -20 | Resolves to `MarvelGame.upk` export 17047; runtime updates are unresolved. |
| Character select / 77 | `characterselect_bg_png`, export index 68 | Local inline texture, 2560 × 1600. |
| Character select / 28 | `characterselectrendertarget`, object reference -10 | Resolves to `MarvelGame.upk` export 17037; runtime updates are unresolved. |

Export indices in this document are zero-based; signed object references follow
UE3's one-based import/export convention. Login has 66 local image bindings and
one import; character select has 99 local bindings and two imports.

Keep GFX logical dimensions distinct from Texture2D storage dimensions. In 265
records they differ; for example, two login image records declare 2 × 84 and
31 × 84 while their textures contain 4 × 84 and 32 × 84 pixels. The movies carry
`flashtexturescale_mult4`. These facts do not establish whether the native runtime
resamples, pads or adjusts UVs. The exact bindings and candidate sets are retained
in private `evidence/frontend-image-mapping.json`.

The binder must establish a byte-array schema for RawData by requiring its element
count to equal the payload byte length, and an object-reference schema for
References by requiring four bytes per declared element and validating each
signed index. The generic property parser intentionally preserves arrays without
guessing schemas. Property and object-name casing differs between cooked files.
Duplicate singleton properties, cyclic outer paths, or non-unique name candidates
should remain errors, not implicit choices.

## Evidence and provenance

Private evidence is rooted at
`~/.local/share/marvel-heroes-forever-rust/evidence/`. The five package SHA-256
identities and input scope are recorded in [UPK_FORMAT.md](UPK_FORMAT.md).
[UPK_OBJECTS.md](UPK_OBJECTS.md) records the property and payload reader. Original
project GFX and texture parser outputs supplied the structural inventories used
here; their primary-source provenance is maintained with those parser documents.

The map study used the already recorded UEViewer MIT sources at commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981`, principally `UnObject.cpp`,
`UnCoreSerialize.cpp`, and the package table readers. Their complete source hashes
and exact paths are included in `frontend-map-startup.json`. No copied source,
raw package bytes, movie bytecode, textures or extracted payloads enter this doc.
These are static parser observations, not REA runtime evidence.

Additional proprietary input identities, independently matched to the owner's
private distribution inventory:

| Stored package | SHA-256 |
| --- | --- |
| `Startup.upk` | `beed4cf91ef4c86f5d4c535ce56845d2f23d64766acf5dd29c74fc582284fcd4` |
| `GFxUI.upk` | `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199` |
| `MarvelGame.upk` | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` |
| `MarvelVista_AsgardHub.upk` | `2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153` |

Remaining dependencies include frontend-manager execution and font-library
routing, imported render-target contents, ActionScript/native screen orchestration,
the streamed vista/camera scene and established music defaults. Resolving these is
required before claiming the rendered login front end matches retail.
