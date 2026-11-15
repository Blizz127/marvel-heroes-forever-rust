# Native loading splash and the next startup stage

Task `a3d58c579542`, 2026-11-15 UTC. This is a static postmortem of the
[saved offline observation](NATIVE_SPLASH_CONTINUATION.md). No new retail/Wine
session, UI input, debugger operation, endpoint request or configuration change
was made.

**VERIFIED static result:** The client has a dedicated native bitmap splash
window. Its startup wrapper starts that window before initializing ClientApp
and before the engine initialization call. ClientApp attempts site configuration
synchronously; completing that attempt is on this startup path, but a successful
HTTP result is not an unconditional early initialization gate. Neither the splash
capture nor these static calls establish that the captured process initialized
its game renderer, opened the frontend map or executed GFx.

## Saved bitmap comparison

The original selected distribution contains
`UnrealEngine3/MarvelGame/Splash/PC/Splash.bmp`, 2,430,056 bytes, SHA-256
`d8619d39330b1a75ab17931e443b7772bad75d06c8014430b39fc46baaed2460`.
Its bounded header decode reports a 1,200 × 675, bottom-up, uncompressed 24-bit
bitmap, pixel offset 54 and row stride 3,600. Two bytes after the computed pixel
array remain uninterpreted. The source inventory's separate
`SplashScreenShader.bin` filename does not establish shader use by this window.

**OBSERVED:** That size equals the recorded `SplashScreen` rectangle at
`(360, 202)` in the 1,920 × 1,080 XWD capture. The original comparison decoded
the BMP's BGR triples and the XWD's stated TrueColor masks into top-down RGB,
then compared all 810,000 pixels without masking, resizing, correction or
tolerance:

| Measurement | Result |
| --- | --- |
| Exactly equal pixels | 33,890 |
| Differing pixels | 776,110 |
| Difference bounds within splash rectangle | `(0, 0)` through `(1199, 674)` |
| Mean absolute R / G / B difference | 10.942096 / 10.959348 / 9.798364 |
| Maximum absolute difference | 255 in each channel |

Canonical source RGB SHA-256 is
`f9b3a4616a0a4bb8fd93a5f5cbfc35d29486e9b3491b07f8515721c6d1530af4`;
captured-rectangle RGB SHA-256 is
`e5a94a9b8925c9b5c23e285d09d401b40431c87a8a3189ed536168a7fdd7c5bb`.
The capture file itself retains SHA-256
`93ca5bf5c8ac77b3d0c7cb210a5acabaca82c0c4085bb6bc4d3f7889226a660d`.

**Correction:** The original private receipt's `limits[0]` mistakenly claimed
pixel identity despite its explicit `exact_rgb_match = false` and mismatch
counts. The preserved original receipt has SHA-256
`d0cfcfe171da2117f91ab7edc45324fa7265a4698fd66f9fb5aefa307e5d36c8`.
The explicit erratum and independently recomputed channel statistics are in
`run-20261115T110131Z/receipts/verification/splash-comparison-followup.json`,
SHA-256 `390c121a0209bf1e33ee4384df76e6f81f0315c2dd6f407698534144fede1ecd`.
The broad mismatch remains unexplained. The same artwork and geometry are
visually recognizable, but no exact image-source or pixel-parity claim follows.

## What the native splash draws

**VERIFIED from REA instructions plus observed PE imports/literals:**

| Native address | Static behavior |
| --- | --- |
| `0x1413317b9` | Selects the saved `PC\\Splash.bmp` path for the non-editor branch. |
| `0x141330576` | Calls splash setup `0x140998ea0`; the next work call is `0x141331880`. |
| `0x14099900c` | Calls imported `CreateThread`, with entry `0x1409960a0`. |
| `0x14099619c` | Calls `LoadImageW` with bitmap type, zero requested dimensions and flags `0x10`; stores its returned handle at `0x142c27da0`. |
| `0x140996245` | Creates the native `SplashScreen` window, class `SplashScreenClass`, with bitmap-derived dimensions and centered placement. |
| `0x140995d88` / `0x140995dd1` | The paint-message branch calls `BeginPaint`, then `DrawStateW` with that bitmap handle, zero origin/dimensions and numeric flags `4`. |
| `0x140995f64` / `0x140996009` | Additional text drawing uses imported `TextOutW`; the painter iterates up to five saved text slots. |
| `0x140996045` | Finishes the paint operation with `EndPaint`. |

Microsoft documents the file-loading flag, bitmap handle argument and native
paint model in [LoadImageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadimagew),
[DrawStateW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-drawstatew)
and [WM_PAINT](https://learn.microsoft.com/en-us/windows/win32/gdi/wm-paint).
The client does not request `LR_CREATEDIBSECTION` here. These API contracts do
not establish Wine's actual color conversion or explain the captured pixel
differences. The native window class above is distinct from the X11 `WM_CLASS`
reported as `marvelheroesomega.exe` in the existing observation.

**INFERRED runtime attribution:** Owned-process identity, matching title and
dimensions, the selected bitmap path and the native paint chain associate the
observed loading artwork with this splash implementation. There was no native
paint-call trace or captured bitmap handle. This provides a much narrower
reference than game-renderer/GFx output; it does not establish graphics-device
creation, shader execution, first presentation or login UI rendering.

## Startup ordering and original configuration

The verified wrapper starts the splash and calls work routine
`0x141331880`. At `0x14133192e`, that routine directly calls
`0x1413bb9d0`, associated by embedded source strings with
`MarvelGameEngine.cpp`. That function constructs the derived app at
`0x1413bc30d`, stores it at global `0x142c4c658`, and invokes its virtual slot
`+0x08` at `0x1413bc32b`. The constructor writes table `0x1422db778`; its
`+0x08` entry is `0x1414486a0`. That entry calls `ClientApp::Init`
(`0x141756270`) at `0x1414487a9`.

This is a direct app-setup call before the later engine virtual call. An early
candidate which subtracted `0x260` from the app-table entry was rejected; it is
not a valid engine-vtable binding.

The [previous settings postmortem](NATIVE_STARTUP_SETTINGS.md) proves the
original module-relative path:

`<executable directory>/../../../Data/Configs/ClientConfig.xml`

Retained main-init instructions call the original XML loader at
`0x141756ab3` and command-line overrides at `0x141756abf`. The previous
isolated run observed successful metadata checks and a read-only handle for
the exact original 256-byte file. It did not trace XML consumption. This phase
reused those receipts and did not reread configuration values. The original
file's SHA-256 remains
`a319ef50b21d3d7e84b5b3c0894b41805c809e7d0eae728b7b4be7f1d3717c5b`.

## Site configuration: attempt, wait and success are different gates

**VERIFIED static control flow:**

1. `SiteConfig::Init` (`0x1418ef580`) allocates its host/URL strings and parses
   primary and backup locations through `0x1418f1530`. A failed parse/setup
   returns false. ClientApp tests this result at `0x141756b67`. This routine
   itself is not the network fetch.
2. ClientApp calls fetch wrapper `0x141758270` at `0x141756b80`. The wrapper
   calls `0x1418edee0`, which invokes `GetSiteConfigFromUrl` (`0x1418eda40`)
   for the primary strings and, if result field `+0x2df8` remains nonzero,
   for the backup strings. The calls occur under the site's critical section.
3. The fetch calls helper `0x1405bd750`, then `0x1405bd7b0`, associated with
   `SimpleHttpRequest.cpp`. It creates a semaphore and calls submission helper
   `0x1405bd350`. If that call returns true, `0x1405bd8fc` calls a wait helper
   with timeout `0xffffffff`. That helper calls imported
   `WaitForMultipleObjects` at `0x1405bcb6e`, with one handle and `waitAll=false`.
   The API's `INFINITE` timeout waits for signaling, as documented by
   [Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitformultipleobjects).
   Worker dispatch, its callback and internal network timeout policy were not
   expanded in this study.
4. The fetch examines request return, error value, HTTP status 200 and response
   presence. Distinct failure results remain nonzero; its response path reads
   XML, deserializes configuration and sets result zero. The wrapper maps zero
   to app state `+0x58 = 2`, failure to `0`, and returns a Boolean.
5. Main initialization does **not** test that Boolean: its next instruction at
   `0x141756b85` calls `InitializeProtocols()` (`0x14064e8b0`). It tests the
   protocol result, then continues toward adapters and game databases.

Thus this startup path attempts site configuration and must return from that
attempt before advancing. It does not require HTTP success at this early call
site. An offline submission may fail promptly, or a submitted request may wait
for completion; this analysis does not establish which happened in the saved
run. The 45-second observation cannot be attributed to the wait from static
code or from its configuration-only syscall trace.

There is a separate verified call site in `MarvelGFxFrontEnd.cpp` routine
`0x1413bf9f0`: a cached zero result skips refetch, otherwise
`0x1413bfc7a` calls the wrapper and immediately tests its Boolean. Failure takes
calls at `0x1413bfc93` and `0x1413bfca1`, then cleanup; success reaches a
different virtual-call path. That routine takes two string arguments, but its
exact operation name, caller and activation stage remain unresolved. It is
**not established as a frontend-display prerequisite**. No login action or
network request was performed to investigate it.

## Handoff toward the saved frontend

Later in the same splash-covered work routine, the client resolves the engine
class through the recorded `engine-ini:Engine.Engine.GameEngine` key. At
`0x141331cc4` it emits the saved initialization-start diagnostic, then calls
the engine object's virtual slot `+0x260` at `0x141331cd3`. At
`0x141331ce7`, after that call returns, it emits
`Initializing Engine Completed`. Returning to the outer wrapper can also call
splash teardown `0x140998d60`, conditional on a global flag whose semantics
were not established here. Teardown posts the quit-producing window message,
waits for the owned splash thread and releases its resources.

**Concrete next stage to distinguish in a future authorized observation:**
completion of that engine virtual initialization call, identified by the
existing completion diagnostic or a directly bound call-return observation.
The diagnostic sink/verbosity was not validated; the prior `Launch.log` was
empty, so absence of that text is not proof the call was never reached.
Conditional splash disappearance is a separate window observation, not proof
of frontend completion. No additional execution or logging change is made here.

**OBSERVED saved defaults, not a validated runtime merge:**

| File / field | Stored value or relationship |
| --- | --- |
| `BaseEngine.ini`, URL map fields | `Entry` |
| `DefaultEngine.ini`, URL `Map` / `LocalMap` | `MarvelEntry.umap` |
| `DefaultEngine.ini`, `Engine.Engine.GameEngine` | `MarvelGame.MarvelGameEngine` |
| `DefaultEngine.ini`, viewport class | `MarvelGame.MarvelGameViewportClient` |
| `DefaultEngine.ini`, packages-to-cook list | Includes `MarvelFrontEndMap`; this is not a runtime map-open instruction. |
| `DefaultGame.ini`, default game/server game | `MarvelGame.MarvelGameInfo` |

The original `MarvelEntry.upk` is 1,221 bytes, SHA-256
`2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33`.
The existing original Rust parser reads 20 names, six imports and five exports
for Engine Level, Model, Polys, World and WorldInfo structures. Those tables name
no frontend asset. All five native preambles remain unsupported, preserving
1,215 serialized payload bytes as opaque; this does not certify an empty map
or exclude an indirect transition. The focused PE literal search also found no
exact ASCII/UTF16 `MarvelFrontEndMap` spelling; that bounded negative cannot
rule out package-driven routing.

[FRONTEND_COMPOSITION.md](FRONTEND_COMPOSITION.md) separately establishes the
saved frontend-map graph: its loaded-and-visible event links to opening
`marvelfrontend.frontend_manager` with `marvelgame.marvelgfxfrontend`, and to
loading the Asgard vista. The manager object resides in `MarvelGame.upk`.
The runtime transition from the entry map into that saved graph remains a gap;
this study does not replace it with a guessed direct login-movie launch.

## Evidence and scope

Private evidence root: `~/.local/share/marvel-heroes-forever-rust/evidence/`.
Current files are under `native-frontend-startup/`, with the older captures and
pixel receipts under `native-retail-reference/`.

| Current evidence | SHA-256 / coverage |
| --- | --- |
| `final.bundle.json` | `b7990cd43ffa983342e5d01f68555f7fdacdf906b3faba505ebc6074c015d7df`; six complete REA records, no unknown records |
| `finding-receipt.json` | `cfe56ed12bad7d3aec51bfd6d8bd655900646e39707801fa6a20e70219c18e58`; anchors, source hashes and explicit limits |
| `final-decoded.json` | `23fe5915a7d31f0e805a567b6f2b0e8301bbe541d870f342cb7804a00106c398`; 19 arrays, 3,709 returned records byte-checked against the PE |
| `reused-main-context.json` | `6bec6092a5762364758fe84806828d6045c26f7a0cb09e1371dbb7cb6b84ad50`; original main-init Evidence reuse |
| `saved-engine-config-fields.json` | `07d7e8fdd8f353fc0c9de47ef6f9c3628db0b9ed9831f973f1ce8d72f78f061f`; selected distribution INI fields and hashes |
| `request-wait-imports.json` | `0edfdfd640ecc11411c536b8bbe5586d393a7024b45c94350f27e2f230e8fcf8`; original PE import names and verified wait-call linkage |
| `candidates/entry-map/receipt.json` | `c4f8e42b14da2fbacee4c1e77070ce469a86395b3d2e5eabf57ba02deeef440f`; actual small entry-map parse, unchanged source/module hashes |

All six target-free Rizin calls completed exit zero without truncation or target
mutation. The target is the owned build 1.52.0.1700 executable, 48,544,080 bytes,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The raw code mapping `0x140000c00` was used only for `.text`; data and imports
used PE section maps. Of the 3,709 returned byte-checked records, 42 decode
padding/jump-table bytes after the two splash routines' final returns; these
are excluded from instruction/control-flow claims. Matching bytes alone are
not evidence that such tail data executes. Fifty-five paint/window call records
were linked to exact import slots, and 13 selected control-flow anchors were
checked separately. REA was exported and reset, with no retained target.

REA IDs:
`ev_c2301b79f1024eca9149515a9b2ff9da1af21b37b096d5066838f9de727138ba`,
`ev_7b2a0be3ea64a3af107270c5ce1b60fe514410c47bdacc498156d7fa52d15659`,
`ev_cc0b52550ec07aeb8fb55ce5f287c51566b74116426d284d127381a065a87bd5`,
`ev_4a8f7d1e83934e06baaaee59e17a3cec6e8ed194e8d5ece3fac9b4f75caaa9b2`,
`ev_bd2689c8fd29f47a7abdca465d824be1e46f251c633897dea817cd296c790766`,
`ev_79c26998c4db230e0af6f9a46168a75b5cbbb771dd0105c49b8a33903d4b8d9c`.

## Source ledger

- Owner's pinned executable, selected `Splash.bmp`, `MarvelEntry.upk`, three
  distribution INI files and existing private captures/inventory: proprietary
  compatibility evidence. No game implementation, binary bytes, configuration
  values, extracted payloads or images enter git. The existing configuration
  schema/hash receipt was reused without reading values in this phase.
- Existing original project UPK modules, `FRONTEND_COMPOSITION.md`, prior
  startup/launcher/capture receipts and original PE-helper conventions:
  project-authored evidence/code, used for metadata parsing and cross-checks.
  Exact parser/module hashes are in the entry-map receipt. No external parser
  implementation was copied for this increment.
- `~/.agents/skills/reverse-engineer-anything/SKILL.md` and
  `references/runtime-observation.md`: installed REA 6.9.0 workflow guidance;
  no licence assumed from availability. Target-free instruction evidence only.
- `/usr/bin/x86_64-linux-gnu-objdump`, GNU Binutils 2.46, GPL-3.0-or-later:
  bounded candidate discovery; semantic claims above were checked through REA.
  `/home/blizz/.local/bin/ffmpeg` 7.0.2-static, reporting GPL-3.0-or-later:
  private lossless BMP-to-PNG view only; the strict pixel comparison uses
  original bounded format decoders, not that view.
- `/usr/include/X11/XWDFile.h` and `/usr/include/X11/X.h`: Open Group/DEC
  permissive notices, header definitions only. Used for XWD header/color-entry
  layout and X11 format constants. Hashes and notices are retained in the
  original splash-comparison receipt.
- Microsoft Learn primary API pages linked above: CC-BY-4.0 per the retained
  MicrosoftDocs `sdk-api` / `win32` licence receipts. Used for native paint,
  image-load and wait contracts, not implementation code. Exact URLs and usage
  are in `primary-api-sources.json`; page source commits were not captured.
- Primary bitmap documents read for the original private comparison:
  [BITMAPFILEHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader),
  [BITMAPINFOHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader),
  [Bitmap Storage](https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage),
  [RGBTRIPLE](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-rgbtriple),
  CC-BY-4.0; [archived BITMAPINFOHEADER](https://learn.microsoft.com/en-us/previous-versions/dd183376(v=vs.85)),
  Microsoft copyright with archive-specific licence unverified, study only.
  Used for bounded offsets, row order/stride and channel interpretation; exact
  retrieved-content hashes are in the comparison receipt. Licence sources:
  [MicrosoftDocs/sdk-api](https://raw.githubusercontent.com/MicrosoftDocs/sdk-api/docs/LICENSE)
  and [MicrosoftDocs/win32](https://raw.githubusercontent.com/MicrosoftDocs/win32/docs/LICENSE).
