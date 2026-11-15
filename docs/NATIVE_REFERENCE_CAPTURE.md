# Isolated native reference capture: readiness

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**OBSERVED:** Wine, Xvfb, image-capture tooling and unprivileged namespace
isolation are available. A short, read-only Python process successfully ran in
a separate network namespace containing only loopback and no routes. **A retail
capture is not yet verified:** no Wine prefix was created, no Wineboot or retail
process was launched, no game window was captured, and no GLX/rendering check
was performed. The integrator owns the subsequent launch sequence.

This study used the REA runtime-observation guide to define a bounded capture
plan. It did not start a REA runtime scenario or change REA/client configuration.
The earlier REA PE-import evidence was cross-checked against the pinned file.

## Runtime and isolation observations

| Component | Observed installed identity | Practical limit |
| --- | --- | --- |
| Wine | `wine-10.0 (Ubuntu 10.0~repack-12ubuntu1)` | Version query only; prefix/bootstrap and game compatibility untested. |
| Xvfb | package `2:21.1.22-1ubuntu1.3`, `/usr/bin/Xvfb` | `-version` is unsupported and exited with help; no server was started. |
| xauth | package `1:1.1.2-1.1build1`, `/usr/bin/xauth` | Available for an isolated display; no cookie/display was created. |
| local bubblewrap | `/home/blizz/.local/bin/bwrap`, version `0.11.1` | This binary's hash differs from `/usr/bin/bwrap`; its exact package origin is unestablished. |
| FFmpeg | `/home/blizz/.local/bin/ffmpeg`, `7.0.2-static` | Reports XCB `x11grab`, XWD decoding and PNG support; no capture was performed. |
| Mesa/Vulkan loader | packages `26.0.8-1ubuntu0.3` / `1.4.341.0-1` | Installed files do not establish Wine rendering support. |

`/usr/bin/wine` resolves to the `wine-stable` wrapper, which selects
`/usr/bin/wine64`. That resolves to `wine64-stable`, which executes
`/usr/lib/x86_64-linux-gnu/wine/wine`. The matching server is
`/usr/lib/x86_64-linux-gnu/wine/wineserver`; it is not exposed as `wineserver`
in the current PATH. A future run should use these explicit, recorded paths
with an explicit new `WINEPREFIX`, avoiding inherited wrapper variables.

**VERIFIED, capability probe only:** The recorded bubblewrap invocation used
new user, network, PID, IPC and UTS namespaces, `--die-with-parent`,
`--new-session`, a cleared environment, read-only host root, fresh `/proc` and
`/dev`, and temporary `/home`, `/run` and `/tmp`. Its Python child exited zero,
reported different network/PID namespace identities from its parent, saw only
`lo`, no network routes, no `/home/blizz`, and an empty `/run`. No connection
was attempted. No host firewall, network interface or sysctl was changed.
This establishes namespace creation on this host. It does not validate a
future Wine mount/environment profile; the proposed game profile below uses
an explicit mount allowlist instead of the probe's broad read-only root.

Trusted-host `ldd` checks resolved the Wine loader and Xvfb dependencies.
Wine's X11 driver also resolved when its own `x86_64-unix` library directory
was supplied for that inspection. Its private `ntdll.so`/`win32u.so` search
requirements are not missing-system-library findings. The usual 32-bit Wine,
Wine Mono and Wine Gecko directories were absent; availability elsewhere was
not exhaustively searched. A first-run installer or native prompt therefore
remains possible and must be captured without accepting it.

`/dev/dri` exposed `card1` but no render node in this observation. The first
proposed profile exposes no host GPU/audio device. Software GL and UE3/Wine
compatibility remain untested. A Wine framebuffer would be a Wine-observed
reference, not proof of Windows GPU pixel parity.

## Input identity and snapshot candidates

The pinned target is
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Paths in this section are relative to the private project data root
`~/.local/share/marvel-heroes-forever-rust/`, unless stated otherwise.

**VERIFIED, file identity:** The 28 immediate Win64 regular files, six nested
helper PE files, 27 Base/Default INIs and 28 selected game-data/frontend/shader
files matched the source inventory's SHA-256 values. This is 34 PE files,
27 INIs and 28 selected assets. The complete Win64 tree also contains helper
resources: its candidate list has 90 entries, so copying only the immediate
28 files would omit known distributed resources.

`evidence/native-reference-readiness/assets/snapshot-candidates.json` records
the exact relative filenames, expected hashes and observed availability for:

- The complete Win64 subtree, including `GazillionWebHelper` and its resources.
- All 27 distribution Base/Default INIs under `UnrealEngine3/Engine/Config`
  and `UnrealEngine3/MarvelGame/Config`.
- `Data/Game/Calligraphy.sip`, `mu_cdata.sip`, `Loco/eng.all.locale` and its
  four English string buckets.
- Twenty-one named cooked frontend/startup/shader files, including
  `Startup.upk`, `MarvelGame.upk`, `MarvelEntry.upk`, `MarvelFrontEndMap.upk`,
  `MarvelFrontEnd_SF.upk`, `MarvelFonts_SF.upk`, `MarvelVista_AsgardHub.upk`,
  `GFxUI.upk`, both distributed global/reference shader-cache variants and
  `TextureFileCacheManifest.bin`.
- All 311 source-listed texture caches, 181 engine shader files, 29 selected
  localization/splash entries and six configured startup-movie candidates.

**INFERRED:** This is a candidate dependency set, not a proved minimal or
complete native startup closure. Preserve the distribution's relative `Data`
and `UnrealEngine3` layout. The proposed executable working directory is
`/game/UnrealEngine3/Binaries/Win64` within a separate snapshot mount; native
working-directory/config precedence still requires observation. Exact TFC
and streaming-package dependencies have not been reduced to a safe minimal
set. A completed, verified distribution snapshot avoids guessing that set.

At 09:08:10–09:08:13 UTC the active transfer had 16,057 entries with matching
regular-file sizes and 432 absent entries. Size matches are not full hash
verification. Of 311 TFCs, **17 were present by size and 294 were absent**.
Absent entries comprised 362 cooked files, 14 game localization files,
55 movies and one splash file. All six selected startup-movie candidates
were absent. These are observations of an incomplete transfer, not claims
that the owner's source install lacks them. No transfer destination or
temporary rsync file was changed, renamed, launched or staged by this study.

## Dependencies and configuration limits

**OBSERVED, PE imports:** The main executable requests `MSVCP120.dll`,
`MSVCR120.dll`, `XINPUT1_3.dll`, `D3DCOMPILER_43.dll`, `d3dx9_43.dll`,
`d3dx11_43.dll` and `X3DAudio1_7.dll`. Distributed dependencies additionally
request `MSVCR100.dll`. All eight names have regular Wine builtin files under
`/usr/lib/x86_64-linux-gnu/wine/x86_64-windows/`; their paths, sizes and hashes
are recorded privately. This verifies availability of Wine implementations,
not API completeness or DLL-selection behavior during this game startup.

Delay imports include `mss64.dll` from `bink2w64.dll`, and `nvcuda.dll` from
the APEX GPU/framework files. Whether those paths execute is unresolved.
The client also ships Steam, CEF, Bink, PhysX and related DLLs; successfully
loading the main PE does not establish those subsystems' runtime behavior.
No packages, redistributables, fonts, codecs or alternate DLLs were fetched.

Distribution configuration establishes these exact observations:

| Source | Observed value | Consequence for capture |
| --- | --- | --- |
| `MarvelGame/Config/DefaultEngine.ini:134–135` | `bRelaunchInSteam=True`, app ID `226320` | Offline startup may exit/relaunch before a frontend frame. No Steam account, bypass or override is provided by this study. |
| `DefaultEngine.ini:99` | Documents subdirectory `Marvel Heroes` | Future generated documents must resolve inside the fresh prefix/home. Actual generated path and precedence are unverified. |
| `DefaultSystemSettings.ini:7–9` | 1920×1080, fullscreen | A 1920×1080 virtual screen is a recorded initial candidate, not an observed native display mode. |
| `DefaultEngine.ini:6–8` | Entry/local map `MarvelEntry.umap`, startup map `AnniversaryTest.umap` | Relationship to cooked `.upk` names and runtime selection remains unresolved; do not force a frontend map as if it were the retail path. |
| `DefaultEngine.ini` | Texture-file-cache use enabled | Missing TFCs may prevent complete rendering. |

`Data/Configs/ClientConfig.xml`, shortcut arguments and
`Engine/Config/ConsoleVariables.ini` were excluded from the source inventory
and were not read. Endpoint settings and exact offline launch flags therefore
remain unknown. Existing Wine prefixes, personal Documents, Steam account
settings and user-generated INIs were not inspected. Defaults also mention
`Marvel_loading1.bik`; no matching `.bik` or `.bk2` basename occurs in the
selected distribution manifest. That alone does not prove a startup failure.

## Proposed bounded capture sequence — not executed

1. Select a new private run directory outside `pins/alienware-candidate`.
   Copy only completed, approved distribution inputs into a separate snapshot;
   verify their hashes against the source inventory and record a snapshot
   manifest. Do not launch from the active transfer or source installation,
   use mutable hard links, copy excluded user configuration, or assume reflink
   support. Keep all game bytes and captures outside git/packages.
2. Build and verify an explicit bubblewrap mount profile: read-only required
   host runtime directories and game snapshot; fresh `/proc`, `/dev`, `/tmp`
   and `/run`; only the new run directory writable. Supply narrowly selected
   system library/font configuration as needed. Do not expose owner home,
   source installation, old prefixes, host X/Wayland sockets, session buses,
   Steam sockets, SSH agents, GPU/audio devices or a host network namespace.
   Clear the environment and inherited descriptors. Recheck namespace IDs,
   interfaces, routes and mount visibility before any Wine phase.
3. Set child-only `HOME`, `WINEPREFIX`, `WINEARCH=win64`, XDG paths,
   `WINELOADER` and `WINESERVER` to the fresh run locations/explicit runtime
   paths. Host environment variables are not reassigned. Wine's documented
   prefix-specific registry/server state prevents accidentally selecting an
   existing prefix only if these variables and mounts are actually enforced.
4. Start Xvfb and xauth inside the same isolated process tree, private `/tmp`
   and offline network namespace, with TCP listening disabled. `xvfb-run`
   exists but must itself run inside the boundary. Use a new display and
   authentication file, and retain `-fbdir` output privately. The Xvfb manual
   documents its `Xvfb_screenN` files as XWD; installed FFmpeg can decode them
   or read the isolated X11 display. Its binary can be separately bound
   read-only because the normal `/home/blizz/.local/bin` path stays hidden.
5. When the integrator sequences it, initialize only the fresh prefix as a
   separately bounded phase. Record any prompt and stop without accepting
   downloads, native trust/login/approval dialogs or installers. No Wineboot
   invocation has been performed by this readiness task.
6. Launch only the selected snapshot executable with a recorded working
   directory and no invented server/Steam/map arguments. A first observation
   can use a fixed short deadline, such as 60 seconds, with no keyboard,
   login, account or gameplay interaction. Expected evidence is a frontend
   frame **or** a concrete loader/Steam/file/render failure. Stop on native
   prompts, unplanned external interaction, or deadline. Preserve failures;
   do not silently rewrite game configuration to reach a different path.
7. Record the exact input/runtime hashes, argument/environment subset, display
   parameters, namespace/process identities, timestamps, stdout/stderr, exit
   state and private frame files. Record generated files only under the new
   run directory. If the game requires writable paths beside its executable,
   resolve them into private per-run copies before another run; never make
   the owner's pin or source install writable to the process.
8. Terminate and verify only the owned namespace/process tree and prefix's
   server. Do not use global Wine/Xvfb kills or touch other sessions. Verify
   all spawned children exited and snapshot inputs retained their hashes.
   REA scenario snapshots can record this work but are not an isolation
   boundary and do not replace explicit child cleanup checks.

## Evidence and provenance

All raw receipts remain under
`~/.local/share/marvel-heroes-forever-rust/evidence/native-reference-readiness/`:
`namespace-probe.json`, `runtime-files.json`, `runtime-summary.json`,
`capture-tools-and-manpages.json`, `supplemental-runtime.json`, and
`assets/{receipt,readiness,distribution-availability,snapshot-candidates}.json`.
`final-readiness-receipt.json` records the final evidence hashes and the
explicit unrun checks; its SHA-256 is
`6cb3ccec8ddf365db5cf703e0c10e7bf7befff7e82958f465da6b63909839c88`.
The asset receipt SHA-256 is
`fd09863973e7cdcbf24c9f8d66d641d8440adcf550b44a6f99e8e4a1bc43c370`.
The exact candidate-list SHA-256 is
`ec4e160be78189be978f4f854ede892b64b9395221c8b45eff1df4266ea9754a`.

Outside material read for this increment:

| Source/path | Licence or status | Use |
| --- | --- | --- |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and `references/runtime-observation.md` | Installed REA workflow documentation; no licence assumed from availability | Bounded scenarios, owned-process cleanup and evidence limits. |
| Private source inventory, pinned retail PE/default INI files and named assets listed individually with hashes in `assets/readiness.json` | Owner-supplied proprietary retail distribution, with bundled third-party components; individual component redistribution rights not audited | Paths, hashes, PE imports and distribution defaults; private compatibility study only. |
| `evidence/retail-pe-imports.bundle.json`, REA evidence `ev_7136989a0b4ae1b35612e44bddcbe16bc3fbf8ad0f1245aaed529cbdb780c919` | Original private evidence over the owned binary | Main executable import cross-check. |
| `/usr/bin/wine-stable`, `/usr/bin/wine64-stable`, installed Wine executables/DLLs, `/usr/share/man/man1/{wine-stable,wineserver-stable}.1.gz`, `/usr/share/doc/{wine,wine64}/copyright` | Wine principally LGPL-2.1-or-later; bundled/packaging notices retained in installed copyright file | Wrapper selection, prefix/server variables, version and builtin availability. No Wine code copied. |
| `/usr/bin/Xvfb`, `/usr/bin/xvfb-run`, `/usr/bin/xauth`, `/usr/share/man/man1/Xvfb.1.gz`, `/usr/share/doc/{xvfb,xauth}/copyright` | X.Org MIT/X11-style and component notices | Version, isolated display/auth plan and documented framebuffer format. |
| `/home/blizz/.local/bin/bwrap`, `/usr/bin/bwrap`, `/usr/share/man/man1/bwrap.1.gz`, `/usr/share/doc/bubblewrap/copyright` | Bubblewrap upstream/package LGPL-2.0-or-later; local binary's package identity remains unverified | Namespace help/manual contract, hashes and read-only capability probe. |
| `/home/blizz/.local/bin/ffmpeg` version, licence, decoder/device output | Installed johnvansickle.com FFmpeg build reports GPL-3.0-or-later | Capture/conversion capability only; no binary redistribution. |
| `/usr/share/doc/{libgl1-mesa-dri,libvulkan1}/copyright` | Mesa has MIT and other component notices; Vulkan loader has Apache-2.0/MIT notices | Installed runtime licence metadata only; no renderer implementation study. |
| Installed package metadata, Linux `/proc` capability values, device metadata, trusted-host loader dependency reports | Host operational metadata; no external implementation reused | Record runtime availability and unresolved rendering requirements. |

No executable game bytes, disassembly, raw configuration files, captures or
external implementation code are included in this repository document.
