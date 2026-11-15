# Frontend ExternalInterface receiver association

**VERIFIED static native path:** a successful new movie load records the owning
movie-player object as movie user data. If that player's receiver slot is null,
the loader assigns the player itself; an existing receiver is preserved. The
external dispatcher later reads those same fields. This establishes an explicit
per-movie owner/receiver contract, conditional on the load path having run.

**OBSERVED package association:** the saved `MarvelGFxFrontEnd` default refers
to `MarvelFrontEnd.frontend_manager`, and its native class inherits the player
localization entry. Initial login instance creation, application of that default,
and the receiver value in a running client have **not** been observed. A separate
pause-menu script constructs the same class. Class identity or asset name alone
therefore cannot identify the active frontend instance.

No native execution, VM execution, handler installation, account access, network
operation, or client configuration read was performed for this increment.

## Three distinct identities

Let **M** be the native movie, **P** the player object owning its load, and **R**
the reflected engine-side receiver. These are distinct from the AS2
`flash.external.ExternalInterface` callable and from a global engine object.

The [AS2 binding study](EXTERNAL_INTERFACE_BINDING.md) establishes selection by
the current invocation Environment: target → AS2 root → M → handler at M+0x238.
The [localization runtime study](FRONTEND_LOCALIZATION_RUNTIME.md) establishes
the handler's reflection and argument/result conversion. This study supplies
the intervening owner and receiver producer:

| Operation | Observed native storage |
| --- | --- |
| Successful new load | P+0x60 receives a movie wrapper; wrapper+0x70 receives P |
| User-data assignment | M is wrapper+0x3c; M virtual+0x140 stores P at M+0x1450 |
| Conditional receiver assignment | If P+0x124 is null, store P there; otherwise retain its value |
| External dispatch | M virtual+0x138 reads M+0x1450 as P; R is P+0x124 |

The user-data accessor table is `0x1424feb70`. Its +0x138 slot at
`0x1424feca8` contains the eight-byte getter `0x140062ab0`; +0x140 at
`0x1424fecb0` contains the eight-byte setter `0x140062ac0`. Both access
M+0x1450. No ownership lifetime or concurrency policy follows from these small
accessors alone.

The complete dispatcher `0x141264c20` calls the getter and reads P+0x124 at
`0x141264caa`. Its existing checks include null user data/name/receiver,
underscore-prefixed names, receiver virtual+0x70, a receiver flag, and reflected
name/function lookup. A state-6 handler alone does not make every callback
callable. The AS2 caller clears its result to undefined before dispatch; failed
prerequisites do not justify fabricating a localization object.

## The concrete producer and its conditions

The named `UGFxMoviePlayerexecStart` registration at `0x142983ef0` binds wrapper
`0x1411a12d0`, which invokes receiver virtual+0x258. The concrete GFxMoviePlayer,
MarvelGFxMoviePlayer and MarvelGFxFrontEnd tables all place `0x14127fdf0` in
that slot, at `0x1421c0578`, `0x14228ad88` and `0x14228b368` respectively.

Complete Start (`0x14127fdf0`, 1,949 bytes) distinguishes two paths:

- With an existing P+0x60 wrapper, it bypasses the new loader and the conditional
  receiver assignment. It may run setup `0x141280590` if wrapper+0x54 is zero,
  sets a player state bit, and returns success. Starting again does not by
  itself restore the receiver to P.
- With no wrapper, the selected path requires a non-null P+0x10c source object
  and calls loader `0x141276a60` at `0x14128012c`. A null returned wrapper fails.
  This study does not independently equate the source offset with a reflected
  property solely from its numeric location.

Complete loader (`0x141276a60`, 1,602 bytes) retains P at `0x141276a89`, calls
the lower create/load helper at `0x141276b14`, and stores its result in P+0x60
at `0x141276b1c`. After success, it stores P in wrapper+0x70 at
`0x141276b54`, invokes the movie's user-data setter at `0x141276b74`, tests
P+0x124 at `0x141276b7a`, and conditionally stores P at `0x141276b84`.
This is the positive default-self proof. No assumption about universal initial
zeroing, class defaults, or an earlier receiver override is needed.

The lower helper `0x141277340` (1,270 bytes) invokes MovieDef virtual+0xc0 at
`0x14127778b`; the Start path supplies a false byte argument. That call returns
before user-data and receiver assignment. The concrete factory's boolean
meaning and any internal frame initialization were not recovered here. It
must not be renamed `initFirstFrame=false` or treated as proof that scripts
cannot run inside movie creation. Subsequent setup is likewise not a proven
full lifecycle scheduler.

## Frontend class and saved startup association

Native class builder `0x1413b8070` passes the exact `MarvelGFxFrontEnd` name and
constructor callback `0x1413bce30`. That callback installs primary table
`0x14228b110` at `0x1413bce59`. The corresponding MarvelGFxMoviePlayer builder
`0x1414e6380` and callback `0x1414ed240` install `0x14228ab30`.

Both tables have localization implementation `0x1414e51c0` at +0x538
(slots `0x14228b648` and `0x14228b068`). The previously recovered named
`UMarvelGFxMoviePlayerexecGetLocalizedObjectDataByLabel` wrapper invokes that
slot. The 64-byte implementation calls `0x141339830` using global
`0x142c31378`, then calls shared localization core `0x1413b7010` at
`0x1414e51ed`. Internal access to shared engine resources does not select a
global `MarvelGameEngine` as R: a FrontEnd player can receive the reflected
method and reach the same core through its inherited implementation.

Owned package metadata supplies an independent saved association:

| Source | Observed identity |
| --- | --- |
| MarvelGame class export 8764 / reference +8765 | MarvelGFxFrontEnd; superclass +8539, MarvelGFxMoviePlayer |
| MarvelGame class export 8538 / reference +8539 | MarvelGFxMoviePlayer; superclass -330, imported GFxUI.GFxMoviePlayer |
| Default export 8765 / reference +8766 | Default__MarvelGFxFrontEnd, class +8765 |
| Default's MovieInfo ObjectProperty | Four-byte reference +17005 at logical offset 2384555; resolves MarvelFrontEnd.frontend_manager, SwfMovie export 17004 |

This is a stored class-default reference, not proof that a particular live
instance received it. The default's member tag is at export offsets 2093–2117;
the value is at 2117–2121. Offsets in this section address the decompressed
logical package/export, not the compressed file's physical byte position.

The original bounded UnrealScript token analysis fully consumes both saved
Start bodies and matches their declared loaded lengths. **OBSERVED** bytes and
references support the following **INFERRED script interpretation**, not an
executed trace:

- FrontEnd.Start, export 8632, script `2367530+192`, calls
  MarvelGFxMoviePlayer.Start with StartPaused at logical 2367535. Later saved
  calls obtain the viewport and size, invoke OnStart at 2367583, and set
  alignment. A conditional later branch concerns the hardware cursor.
- MoviePlayer.Start, export 8369, script `2347066+218`, calls CacheMoviePackage,
  GFxUI.GFxMoviePlayer.Start(StartPaused) at 2347081, and Advance(0) at 2347093.
  Its initialization branch tests bInitialized and calls Config. Later calls
  obtain `manager_mc` as MarvelGFxUIManager, initialize it from MovieSize,
  add padding anchors, assign globals, and load panels. The retained tree
  preserves branches, nested arguments and assignments; this list does not
  flatten them into unconditional successful execution.

The saved Advance call is distinct evidence from native movie creation. The
full selected bodies contain no SetExternalInterface call. That local result
does not exclude other scripts, native writes, reflection, or runtime overrides.

## OnStart caches an instance; the cache does not create one

The exact OnStart registration binds wrapper `0x141376f10` to virtual+0x5e8.
FrontEnd's slot `0x14228b6f8` points to `0x1413c0dd0` (1,035 bytes).
After its null checks, instruction `0x1413c0e81` stores the original player P
at `[[global 0x142c4c660]+0x3a8]+0x54`.

The named MarvelGameViewportClient.GetCachedFrontEnd wrapper `0x1413591f0`
invokes +0x2d0. The named viewport class builder/callback install primary table
`0x142279028`; slot `0x1422792f8` selects `0x141393380` (105 bytes).
It obtains global `0x142c4c660` through getter `0x1413b3ea0`, returns null if
absent, and checks the object at +0x3a8 through helper `0x141383db0` against
the descriptor named ClientGameAdapter. Successful lookup returns adapter+0x54,
the same cache written by OnStart. It performs no frontend allocation.

Descriptor registration `0x140588a10`, its exact name literal and getter
`0x14147c890` ground the adapter type check. A separately selected global
producer calls `0x14174ff60` and stores its return at `0x1413ba131`; a nearby
diagnostic names a client game. This is not proof that global `0x142c4c660`
is the UE3 MarvelGameEngine object, nor a recovered frontend factory.

The package provides a concrete separate producer: the conditional
MarvelHUD.MarvelGFxTogglePauseMenu script, export 5295, assigns a new
MarvelGFxFrontEnd to m_pauseMenu. The assignment begins at logical 2089019;
New is at 2089025, class constant +8765 at 2089029, and subsequent virtual
Init at 2089048. Its complete selected token parse establishes this as a
pause-menu candidate path. It supplies no initial-login creation proof.
GFxMoviePlayer.Init's selected parse stops at an unsupported opcode; its
later apparent Start/Advance sequence is not promoted to a complete call path.

## Receiver replacement and supported implementation boundary

Saved GFxUI.GFxMoviePlayer.SetExternalInterface, export 539, has a 14-byte
script at logical 108313. Its retained token interpretation assigns parameter
H to instance.ExternalInterface; both properties are typed Core.Object.
The field is not restricted to MarvelGameEngine or MarvelGFxMoviePlayer by
that schema. Native P+0x124 is the proven receiver slot; this study does not
independently prove its reflected offset from package metadata alone.

A prior focused scan covered 3,735 Function exports in MarvelGame and GFxUI
and found no direct stored caller/reference beyond the setter itself for its
selected needles. A new four-pattern scan of 3,563 MarvelGame functions found
the pause-menu class constant. Neither scan proves absence of native or
indirect selection. Two of 19 selected function parses retain explicit
unsupported-opcode boundaries; AssignGlobals and Init are not complete.

**INFERRED implementation boundary:** preserve an explicit current movie/root
association, a distinct owner identity P, and a mutable optional receiver R.
A supported successful-new-load transition can install P as user data and
default R to P only if R was absent. Preserve a supplied override. Existing
wrapper Start must not repeat that transition implicitly. Dispatch must use
the receiver associated with the current movie, then enforce the supported
receiver type and reflected localization method contract.

The FrontEnd class/table and saved manager reference are evidence for admitting
an explicitly supplied FrontEnd-player context. They are not authorization to
create one from a filename, use a class default as a live receiver, select the
global engine because it has the same callback name, or claim original startup
completed. Initial login construction/default application, receiver lifetime,
override timing, broader script initialization, and creation-time reentrancy
remain unresolved. This increment enables no product callback by itself.

## Evidence, verification and provenance

All new private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/frontend-external-receiver/`.
Primary sources are the owner's proprietary final client and packages, read
only for interoperability. No implementation from outside source code was used.

| Primary source | SHA256 |
| --- | --- |
| pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe | `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` |
| pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelGame.upk | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` |
| pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/GFxUI.upk | `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199` |

Paths in that table are relative to the private project data root. Rights to
those artifacts remain proprietary; no binary bytes, extracted scripts or
disassembly enter the repository. Consulted original project sources are the
two linked studies, their retained receiver/metadata receipts, original PE and
package-reader helpers indexed by the receipts below, and the REA skill at
`~/.agents/skills/reverse-engineer-anything/SKILL.md`. Installed REA was used as
tooling; no REA implementation was copied. No outside website/source repository
or shared configuration was consulted or edited for this increment.

Canonical `native.bundle.json` has SHA256
`ae0bec3c2061ef99efe4971e7c5e2726c3b6739911883dca29ebbedc68607a4d`:

| REA Evidence record | Selected coverage |
| --- | --- |
| `ev_1982362a54d2f3f32ded8a295464b4a3dd73e5c409835abfe8fbe2ff80579433` | FrontEnd/player named builders, constructor callbacks, tables, localization implementation |
| `ev_44c24491cbe4bd6c5bca34792deff98cc8a005d25fdc1d017691a30b205148af` | Complete Start/loader/lower loader, user-data accessors, concrete table entries |
| `ev_c0d52d517ad7104fb02663163d15572d5d47b1dff9d922649a6d91e79b2a7a68` | Complete OnStart, cache getter, adapter check, named wrappers/registrations |
| `ev_b5eadbe59a84525164ac0c3cd82045c91cd52336d2db1f711179d8f61bb582b5` | Named viewport builder/constructor and ClientGameAdapter descriptor; focused client-game global producer |

Every record completed with exit 0, untruncated output and unchanged executable
hash. `verify_native.py` checks selected instruction bytes and raw data against
the owned PE: **1,763 instruction sequences and 7,740 selected bytes**.
`native-verification.json` SHA256 is
`650299cd959e946904353de86b8da9a8fce2ff3d0a20aa4c694ac3f04339da40`.
Warnings about skipped broad string-search regions do not establish absences;
positive claims use selected exact reads. Exploratory disassembly remains
separate from canonical REA and includes explicitly excluded neighboring bytes.

Supporting receipts preserve discovery and original package analysis:

- `native-producer/receipt.json`, SHA256
  `bdbb05ef4c03869252b0703f7c5b037c3a7db9bf8b54bb7513883b30e67a3781`.
- `native-frontend/receipt.json`, SHA256
  `c2c8833a614a9dc8ccd3ea24d79a92a62ccec13766b4f6f2e86e9cd85e2792db`.
- `metadata/receipt.json`, SHA256
  `b7d2284d753b44189d4ec39a664e896a587adcc20f94207642275c5617ca569f`;
  `metadata/findings.json`, SHA256
  `ff6b820da83ab91fdd1e2463eade4c6d6da3c754cddf20215a9a860b79732aa3`.

Each main supporting receipt indexes 55 files. Seventeen selected package
function parses fully consume their declared spans; the two partial parses
remain identified. `support-verification.json` records independent hash
readback of the three manifests, setter evidence and current primary inputs.
The complete dispatcher is reused from
`frontend-localization-runtime/aggregate-followup/native/final.bundle.json`
(SHA256 `ee2dc52405ce5879a69c7e1ff6ebf97f3202b6bd647694916242c9ee342ceb67`).
No Cargo tests or renderer smoke were run for this documentation-only change.
