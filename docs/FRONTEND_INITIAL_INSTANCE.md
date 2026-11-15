# Initial frontend map and movie-player creation

**OBSERVED saved initial-map path:** the owner's `MarvelFrontEndMap.upk`
contains a LevelLoaded event whose `Loaded and Visible` output links to an
actual GFxAction_OpenMovie instance. That instance explicitly names the
`MarvelGFxFrontEnd` class and `frontend_manager` movie. Its native action
creates a player from its class field, assigns its movie field, and invokes
the reflected `Start` method. This supplies a concrete initial-map producer,
separate from the previously recovered pause-menu construction.

**VERIFIED static native behavior is conditional:** the map must be loaded,
the event/action must be admitted by the sequence runtime, class and asset
imports must resolve, and construction must succeed. No initial map selection,
sequence scheduling, native execution or live receiver value was observed.
The action has no saved ExternalInterface variable binding, but that absence
does not prove the constructed player's effective receiver field is null.
Native default finalization remains an explicit boundary.

## Exact saved producer and event links

The pinned map has 37 exports. All references in this section are signed
package object references; positive values are export index plus one. Offsets
address the decompressed logical package, not its physical compressed bytes.
Stored imported names are retained with their actual spelling.

| Saved source | Observed value |
| --- | --- |
| Export 1 / reference +2 | `theworld.persistentlevel.main_sequence.gfxaction_openmovie_0`; class import -29, `gfxui.gfxaction_openmovie` |
| Movie property value, logical 9521 | Import -43, `marvelfrontend.frontend_manager`, SwfMovie |
| MoviePlayerClass value, logical 9549 | Import -30, `marvelgame.marvelgfxfrontend`, Class |
| Export 32 / reference +33 | SeqEvent_LevelLoaded; saved output 0 is `Loaded and Visible` |
| Output 0, link 0 | LinkedOp +2 at logical 49302; InputLinkIdx 0 at 49330 |
| Export 34 / reference +35 | SeqVar_Player, supplied through the action's `Player Owner` variable link |

The OpenMovie instance's 2,520-byte tagged body and its nested link arrays are
fully consumed by the original bounded reader. It explicitly stores
bTakeFocus=true and bCaptureInput=true. Its variable-link records contain:

| Link description | Saved linked variables |
| --- | --- |
| `External Interface` | Empty; array count 0 at logical 10651 |
| `Player Owner` | One reference, +35 at logical 11089 |
| `Movie Player` | Empty; array count 0 at logical 11517 |

The two action output records, `Success` and `Failed`, both have empty link
arrays. All three variable records retain MinVars=1 and MaxVars=255 even where
their arrays are empty. These editor/schema values do not manufacture a
receiver or establish runtime rejection of the authored action.

LevelLoaded output 0 is explicitly enabled: bDisabled=false,
bDisabledPIE=false, and its saved activation delay is float zero. It contains
**six** links in serialized order: OpenMovie input 0, ToggleCinematic input 0,
Interp input 0, ToggleHUD input 1, SetVelocity input 0 and MultiLevelStreaming
input 0. Their exact references are +2, +31, +27, +32, +30 and +28. The
MainSequence's saved member list includes both event +33 and action +2.
This is saved graph order, not a captured scheduler trace. A replay
limited to movie creation must preserve the other five actions as unexecuted;
it cannot report complete map initialization or infer their effects.

## Native action identity, creation and Start invocation

REA grounds the action independently of its package name. Builder
`0x14126dc80` passes the exact UTF16 class name GFxAction_OpenMovie at
`0x1421bd192` and constructor callback `0x141276000`. That callback installs
primary table `0x1421c1e50`; its virtual+0x300 slot at `0x1421c2150` is
`0x141262880`. The complete 1,507-byte body is retained.

Let A be the action, P the movie player it produces, and M its later native
movie. The relevant native operations are:

1. Require non-null global `0x142c31378` and its +0x6dc object. This is an
   engine-side prerequisite, not a choice of that global as the external
   receiver. An additional helper `0x140d14140` runs before creation.
2. If both A+0x108 and A+0x110 are non-null, call `0x1407a9f00` at
   `0x141262912`, using A+0x110 as class and A+0x40 as outer. The selected
   call supplies zero name/flags/template/context arguments and the existing
   error-output global. This is a generic construction path, not a direct
   invocation of the frontend native constructor.
3. Pass its return through `0x14125bd80`, a class/superclass-chain check for
   GFxMoviePlayer. Store the result at A+0x118. Copy A+0x108 into P+0x10c at
   `0x14126292d`. The association of the action's two source fields with its
   Movie/MoviePlayerClass schema is supported by their use, declared types,
   and concrete action identity; native property-linking offsets are not
   independently reconstructed from serialized declaration order.
4. Query `Movie Player` variable links and publish P to returned output
   locations. If P is null afterward, take the failure-output branch.
5. Query `External Interface`. Only if the returned list is nonempty does
   `0x1412629f0` assign its first object's value to P+0x124. The actual map
   has no saved binding for this link. This instruction supports an explicit
   override; it does not clear P+0x124 when the list is empty.
6. Query `Player Owner`, derive the owner's local-player entry through the
   selected helpers, and store its index at P+0x120. The actual map supplies
   SeqVar_Player, whose resolved runtime object is a prerequisite. This
   study does not invent a player object or select index zero.
7. Transfer action flags and other settings, including selected input/display
   state. Construct two DWORDs: StartPaused from A+0x120 bit 2, followed by
   an initially zero return slot. Resolve the reflected name at
   `0x142c45910` on P through `0x140799ae0`, then invoke P virtual+0x210 at
   `0x141262d6e`. Success/failure output selection reads the resulting return
   DWORD. The effective StartPaused bit is not inferred from an absent tag.

The name global is independently grounded: selected initialization
`0x141205e69..0x141205ea0` passes exact UTF16 `Start` to `0x140793e60`, reads
the returned name value and stores it in `0x142c45910` at `0x141205e99`.
The next literal is prepared before this store; it must not be mistaken for
the value being stored. Generic reflected invocation and the selected
UnrealScript Start interpretation are separate from direct virtual+0x258
native Start dispatch.

The action has paths for existing P and failed prerequisites. Its creation
branch dereferences the cast result while assigning the movie source; this
study does not claim a universally safe successful return for invalid class,
failed allocation or missing asset. An original implementation should reject
unsupported prerequisites rather than simulate a successful open.

## Joining the player to the frontend manager and receiver

The [receiver study](FRONTEND_EXTERNAL_RECEIVER.md) already grounds the
FrontEnd class hierarchy, concrete native table, inherited localization method,
saved Start scripts, native loader and owner/receiver storage.

For the supplied FrontEnd class, the selected saved Start interpretation is
FrontEnd.Start → MarvelGFxMoviePlayer.Start → GFxMoviePlayer.Start; afterward
the player script calls Advance(0), obtains `manager_mc`, initializes it and
continues its saved setup. The full selected Start parses and their branches
are retained in that study. This is script interpretation from original bytes,
not evidence that those calls have executed successfully here.

Successful **new** native Start records P as M's user data and assigns
P+0x124=P only if that slot is null. The exact map action now supplies a
concrete class/asset producer for this path. It also shows that the movie source
is explicitly assigned by the action: choosing frontend_manager does not
depend solely on assuming that a class default's MovieInfo was applied.

The resulting explicit chain is:

`loaded/visible map event → saved OpenMovie action → supplied class construction
→ explicit movie assignment → optional receiver override → reflected Start
→ native new movie load → M.userData=P → conditional default receiver P`.

Every arrow retains its own prerequisites. In particular, an empty saved
External Interface link establishes no override in the authored graph; it does
not close the constructed receiver's default-value question. A runtime bridge
must retain explicit receiver state until that initialization is established.
The global engine, the CDO, the AS2 ExternalInterface callable and P must remain
distinct identities.

## Saved defaults and the remaining application boundary

Three fully decoded top-level class-default property lists establish:

| Saved default | Relevant explicit values |
| --- | --- |
| Default__MarvelGFxFrontEnd, MarvelGame export 8765 | MovieInfo +17005 → MarvelFrontEnd.frontend_manager at logical 2384555; bDisplayWithHudOff=false |
| Default__MarvelGFxMoviePlayer, MarvelGame export 8539 | bAutoPlay=true; m_uninitializedPanelCount=1 |
| Default__GFxMoviePlayer, GFxUI export 37 | bDisplayWithHudOff=true |

Neither ExternalInterface nor bInitialized appears in those three top-level
lists. The exact GFxMoviePlayer.ExternalInterface declaration is GFxUI export
66: array dimension 1, raw 64-bit property flags 0, object-class reference -15
Core.Object. MovieInfo, export 95, likewise has raw flags 0 and references
SwfMovie. These stored flags do not by themselves establish final runtime
property metadata, transient/config behavior or a null effective object field.

The selected native frontend constructor `0x1413bce30` calls base
`0x14125ea70`, sets its table and clears four additional fields. The base
clears its native movie wrapper at +0x60 and several container fields.
Neither writes movie source +0x10c or receiver +0x124. Base UObject constructor
`0x140793fd0` only sets its table and returns the object. Consequently a C++
constructor-only replay does not implement the stored UObject defaults.

The existing [native allocation/default study](ANIMATION_RUNTIME.md) retains
the exact generic helpers also used here:

- `0x1407a9f00` calls allocation/initialization `0x1407a8d20`, then the
  class constructor callback at class+0x1dc on the admitted path.
- The allocator has a fresh path reaching zero-fill of class size, and
  separate reuse paths. Default source selection can call `0x140745a20`,
  whose cache is class+0x1d4 and whose fresh default construction supplies
  the parent default. It is not an unconditional all-zero final object.
- Default initialization/copy `0x1407a5d10` → `0x14079d6d0` includes bulk
  copy beginning at object offset 0x60 and property-specific loops.
- Conditional default finalization `0x140741410`, loaded package data,
  property metadata and possible subsequent initialization remain material.

Those receipts are reused rather than treating allocation zeroing as the
final receiver value. Applying the shared helpers requires a class- and
field-specific proof; an admitted profile for another class does not supply
this one. No effective receiver-null, bInitialized=false,
StartPaused=false or fully initialized panel state is asserted here.

## Startup scope and useful exclusions

This closes a **saved initial-map creation route**. The mechanism choosing
MarvelFrontEndMap as the initial map, the live level-loaded/visible event,
sequence event admission, import loading and player resolution were not run.
No configuration file or command-line state was read to replace those facts.
The native event scheduler and the five other linked actions remain outside
the supported frontend-only replay.

Other evidence helps avoid incorrect shortcuts:

- FrontEnd.StaticClass `0x1413a3d20` resolves a cached class object. Its
  bounded direct references lead to class registration, not a player instance.
- Seventeen selected ASCII and four UTF16 full-class-path references are
  settings-section arguments in the inspected native functions. They are not
  evidence of a factory call, and no referenced settings file was opened.
- Saved GameInfo.SpawnPlayerController branches on native InGameState to
  choose PlayerControllerClass or FrontendPlayerControllerClass. The actual
  GameInfo default explicitly stores MarvelPlayerController for both.
  A separately declared MarvelFrontendPlayerController is not selected merely
  by its name. This player path alone does not create the movie player.
- The pause-menu New FrontEnd path and the OnStart/GetCachedFrontEnd cache
  remain separate observations in the prior receiver study.

## Evidence and provenance

All extracted data, scripts, disassembly and receipts remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/frontend-initial-instance/`.
Sources are the owner's proprietary build 1.52.0.1700, original project
readers/research and installed REA tooling. No external implementation was
copied; no public source repository or website was consulted.

| Owned primary artifact | SHA256 |
| --- | --- |
| MarvelHeroesOmega.exe, pinned alienware-candidate Win64 path | `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` |
| MarvelFrontEndMap.upk, pinned alienware-candidate CookedPCConsole path | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` |
| MarvelGame.upk, pinned frontend-startup CookedPCConsole path | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` |
| GFxUI.upk, pinned frontend-startup CookedPCConsole path | `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199` |

Canonical `native.bundle.json` SHA256 is
`9ceaea4174da6cb38fd6f4caa83d3891bd7d06638898730464ff5c8812be2063`:

| New REA record | Coverage |
| --- | --- |
| `ev_2af212850f7b948b5197ba88f099c387aa9e9eb1c905beca42e587df9baf2a07` | Exact base/frontend constructors and StaticClass getter |
| `ev_37eb2c5eb341690de09e06d3d2fc1adfe8f3c1ceb1666453463d77635c8a17e5` | Complete OpenMovie action and player cast, focused Start name producer, selected labels |
| `ev_933330495946be86fca6af8064623c748eca0a0c92539346612db1b56f9b36a7` | Named action builder, constructor/table slot and player class builder |

The separate final label record
`ev_2433df4f51f0538baa34b2a55f40e66ed9e75c740a4e62992e7079716c8bca32`
completes the Player Owner literal and grounds the cast helper's GFxMoviePlayer
class name. `literals.bundle.json` SHA256 is
`e6838291b251dc3421461dad8574cd6b2273278c38f64283f62374803a34167c`.

All four completed with exit 0, untruncated output and unchanged executable
hash. `verify_native.py` checks **560 instruction sequences and 2,569 selected
stored bytes**. The 16-byte class-cache read is in an unbacked PE virtual range:
provider zeros there are explicitly excluded from stored-byte verification and
are not claimed as observed live state. `native-verification.json` SHA256 is
`93ce05303d50805c660383f5426652656e08830b27d36f1f9e55bdaff1ffe73e`.

Reused canonical generic-construction records reside in
`evidence/animation-runtime/native/`, indexed by ANIMATION_RUNTIME.md and the
new private support receipt. Their six selected complete functions were checked
against the executable again: **1,641 instruction sequences and 6,834 bytes**.
`support-verification.json` SHA256 is
`23c2c6c6abda428b14b1115cac7a82cf439fe01e791f6cfaecf40f84f73f6831`;
it also verifies 255 unique supporting files and the three current package
hashes. Completion and byte agreement do not close the documented semantic
default-finalization gaps.

The combined `package/receipt.json` SHA256 is
`5b06a3f1caac9c4a5b3d7e21601e371503e8afbbc1266b7d1a6beea791db75a5`.
Its map component `package/map/receipt.json` has SHA256
`3571c7b3cd67e8a80fb928a547770f80a8622f6dd045b9ddc8e67263664cbf37`:
14 selected bodies, ten nested Kismet lists, 574 typed properties and 15 array
references checked against their source bytes. Main package scripts have 15
complete selected profiles and five explicit partial profiles; none of the
partial parses establishes an initial movie-creation path.

Original discovery helpers, property/token readers, metadata, source identities,
limitations and hashes are retained in `native-candidates/`, `defaults/` and
`package/`. The mandatory REA workflow
was read at `~/.agents/skills/reverse-engineer-anything/SKILL.md`. No native
execution, VM execution, network, account access or configuration read occurred.
No Cargo tests or renderer smoke apply to this documentation-only increment.
