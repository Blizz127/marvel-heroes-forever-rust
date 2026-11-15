# Isolated native reference capture and startup observations

Investigation: 2026-11-15 UTC, task `a3d58c579542`. Initial read-only readiness
inspection was followed by the separately authorized runtime preparation below.

**OBSERVED:** Two separately authorized offline retail observations reached a
native game window, then a concrete settings-initialization verification failure.
The longer observation retained a Wine Program Error dialog. No login/frontend
reference frame was obtained. All 453 frozen game-file hashes remained unchanged,
owned processes were cleaned up, and no UI or debugger input was sent.

This study used the REA runtime-observation guide and REA process capture for
the authorized preparation and retail observations. REA/client configuration was unchanged.
The earlier REA PE-import evidence was cross-checked against the pinned file.

## Bounded offline retail observations

The owner-authorized integrator first requested one attempt, then explicitly
authorized one additional observation allowing the known primary game window
to remain while loading. Both used the same frozen inputs, separate private
copies of the clean prepared prefix, no game arguments and no UI input. No
third attempt was performed by this investigation.

Before copying, available space measured 19,233,791,774,720 bytes. One-second
disk-counter and pressure receipts were retained; I/O full-pressure avg10 was
0.01%, while CPU pressure was substantial. A single `nice -n 19` copy/verification
process ran from 09:24:16 to 09:26:35 UTC. It copied 453 candidate files totaling
5,115,044,673 bytes, including all 90 Win64 distribution files. Source digest
before copying, copied-source digest and destination digest after copying all
matched the source manifest, and source identity/modification metadata stayed
stable through each copy. Every copied file has an independent inode; no hard
links were used. Each clean-prefix clone additionally verified all 1,755 regular
files against the prepared prefix. Its original remains preserved.

The immutable game snapshot is
`evidence/native-retail-reference/run-20261115T092318Z/game/`, relative to the
private project data root. It contains no `ClientConfig.xml`, excluded user
configuration or replacement settings. Its explicit missing-candidate list has
198 TFCs, 14 game-localization files, six startup movies and one splash file.
For example, `UnrealEngine3/MarvelGame/Localization/INT/MarvelGame.int` is
absent. The 672-file candidate list is not a complete retail distribution or
a proved startup dependency closure; unselected packages may also matter.

The runtime profile mounts that snapshot only at read-only `/game`. Individual
new runtime subdirectories are writable at `/capture`; the whole run directory
is not exposed as a writable bind, and `/capture/game` does not exist. Runtime
checks confirm those mount properties, distinct PID/network namespaces,
loopback only, no routes and hidden owner home. Each display uses a fresh
authentication cookie, rejects an unauthenticated connection, and disables TCP
listening. No host GPU/audio device or host display/session socket is exposed.

The sole application command was the recorded Wine loader followed by
`/game/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`, with working
directory `/game/UnrealEngine3/Binaries/Win64` and disconnected stdin. The
independent REA outer bound was 55 seconds, with three seconds of owned-process
finalization. A supervisor deadline is additional protection; it alone cannot
bound a blocking Xlib call or frame write. Passive capture waits are recorded
as scheduled delays, not exact wall-clock guarantees.

**OBSERVED, first attempt:** At 09:27:29–09:27:38 UTC the application created
a 954×508 window titled `Marvel Heroes Omega`. The conservative first-window
guard then stopped the process. Its images are essentially black with a thin
border, and no login or error text is visible. This establishes window creation,
not frontend readiness or a demonstrated application failure. Generated
`Launch.log` and `.dmp` files were zero bytes in this first observation.

**OBSERVED, additional observation:** A fresh verified prefix clone in
`evidence/native-retail-reference/run-20261115T093011Z/` reused the same
read-only snapshot. The monitor allowed the known single primary window and
captured frames approximately every two seconds, with live passive image
inspection. The application started at 09:31:17.632 UTC. At 09:31:25.770 UTC,
the monitor detected a new `Wine Debugger` window; a subsequent painted frame
also shows `Program Error`. It captured the error and stopped without selecting
either native dialog button or issuing debugger commands. Cleanup finished at
09:31:26.669 UTC, well before the outer deadline.

The second private `Launch.log` identifies the running client as Shipping
**1.52.0.1700 CL:479899**, with an empty command line and the expected Win64
base directory. It reports Steam Client API initialization result 0, generated
configuration under the new prefix's Documents hierarchy, and RHI
`PC-D3D-SM3`. Wine diagnostics identify `llvmpipe (LLVM 21.1.8, 256 bits)`;
the game's reported GTX 470 adapter is Wine's emulated adapter identity, not
an observed physical GPU. These facts do not establish graphical parity.

**VERIFIED as retained runtime output:** At log-relative time 6.13 seconds,
`Launch.log` reports a failed verification of
`m_clientAppSettingsPostCoreInit`, attributed to `ClientApp.cpp` line 1488.
Wine subsequently reports a write page fault at instruction `0x1405bf949` to
address `0x3c`, followed by the error windows. The settings verification is the
earlier concrete failure. The later page fault could be part of fatal-error
handling; its root cause is not established. Missing/excluded inputs and Wine
runtime differences remain unresolved factors. No missing configuration was
invented, read from the excluded source, or substituted to bypass the failure.

The second `Launch.log` is 4,147 bytes, SHA-256
`54edcbb62b53cd5921286c837c27d144bf0fbce665a76177478876c42f990878`.
Its 216,944-byte minidump is retained privately, SHA-256
`526709a87c314968d01db727fedfb97ae9867a59814deae11d793dfb228dfa32`.
Generated configuration confirms `Language=INT` and the directory
`prefix/drive_c/users/blizz/Documents/My Games/Marvel Heroes/MarvelGame/`.
Existing owner documents, accounts and prefixes were not available to either
process. Steam/source defaults and server state were not changed.

**VERIFIED, cleanup and input preservation:** Both supervisors report zero
remaining live owned children and stopped their own Xvfb processes. REA reports
the owned process groups cleaned up. Its external-PID EACCES unknowns were
retained; rechecked survivors were kernel workers with parent PID 2 and were
left untouched. All 453 game-file hashes matched the source manifest after
each run. Both REA investigations were exported and reset. Supervisor exit
zero means observation/cleanup completed; it does not mean the game succeeded.

Private receipts and captures:

| Artifact under `evidence/native-retail-reference/` | Identity or use |
| --- | --- |
| `retail-reference-receipt.json` | SHA-256 `a0808c25aca21211a2a065a33f0a1fc4a0b0d9b021b645134eb9963e867afa27`; both runs, hashes, missing inputs and limitations. |
| `run-20261115T092318Z/receipts/frozen-inputs.json` | SHA-256 `8b96c753bb85b6010cf2bc9031f5fa152362f5bfe9eea7c285edc2a28ba6dcdd`; exact copied/missing files and before/after digests. |
| `run-20261115T092318Z/receipts/post-run-snapshot.json` | All 453 hashes match after the first run. |
| `run-20261115T093011Z/receipts/post-run-snapshot.json` | All 453 hashes match after the additional observation. |
| `run-20261115T093011Z/captures/screen-retail-observe-new-window-painted.png` | Observed native error dialog; no button was selected. |
| Each run's `receipts/rea-*.bundle.json` | Complete, untruncated REA capture and retained sampling/network/filesystem unknowns. |

REA capture IDs are
`ev_014163037bf8b55b04f68dce2f1930f5420b37a4aa5b5d0531448445b2a99c63`
and `ev_3f08ef19632a4eb0cbb8344b7d080d2a8928e059665ee3e5243d76249d8a7eff`.
All captured frames, game data, logs, minidumps, prefixes and original private
supervisor scripts remain outside git and build packages.

## Authorized isolated preparation result

The initialized run directory is
`~/.local/share/marvel-heroes-forever-rust/evidence/native-runtime-prep/run-20261115T091948Z/`.
Its `prefix/` is the new Wine prefix. The original private supervisor and exact
argument/environment manifest are retained in `control/supervisor.py` and
`control/profile.json`; they are observation artifacts, not application code.
`evidence/native-runtime-prep/initialized-prefix.json` identifies this run.

**VERIFIED, observed execution:** The successful phase ran at 09:20:06–09:20:26
UTC. Wineboot exited zero at 09:20:22, followed by three seconds of display
observation before cleanup. The prefix contains regular `system.reg`,
`user.reg`, `userdef.reg`, `services.exe`, `wineboot.exe` and `kernel32.dll`
files, with hashes recorded in `receipts/prefix-files.json`. A non-following
walk recorded 1,755 regular files, 1,269,542,562 bytes and two symlinks. These
are initialization and file-presence observations, not API-completeness tests.

The executed sandbox uses new user/network/PID/IPC/UTS namespaces, drops all
capabilities and creates fresh `/proc`, `/dev`, `/tmp`, `/run` and `/home`.
Its only writable host bind is that new run directory at `/capture`. Read-only
inputs are `/usr`, `/etc/fonts`, `/etc/ld.so.cache`, four generated minimal
identity/host-resolution files and the selected FFmpeg executable. Standard
merged-`/usr` loader aliases are reproduced. The game, owner home, source pin,
existing prefixes and host display/session sockets are not mounted. Runtime
assertions check distinct PID/network namespace identities, loopback only, no
routes, hidden owner home and absent `/game` before Wine starts.

The environment is cleared and supplied explicitly. `HOME` and XDG paths
resolve beneath `/capture/home`, `WINEPREFIX` is `/capture/prefix`, and the
observed Wine loader/server paths are explicit. This prefix was initialized
with that namespace pathname; any later authorized use must retain the
recorded isolated mount arrangement. No bare host Wine invocation was used.

**VERIFIED, display authentication:** Xvfb ran as a new owned process on
`:97`, screen 1920×1080×24, with `-nolisten tcp` and a new private xauth
cookie. The supervisor connected using the cookie; a separate connection
without it failed. GLX is advertised by the server, but no GL context was
created or rendered. X11 window inspection sends no keyboard/button events.
All native process stdin was disconnected. The cookie is retained privately
and is not included in this document or printed command arguments.

The first exploratory phase in `run-20261115T091544Z` intentionally stopped
at the first mapped window. The retained frame showed only Wine's buttonless
configuration-update status message. That interruption left an incomplete
prefix. A bounded continuation exited zero but logged missing `services.exe`;
file inspection confirmed the incomplete state. That first run is preserved
as a failed initialization observation and is not the initialized-prefix
candidate. Exit code alone was insufficient to establish preparation success.

The second, fresh run permitted only the exact framebuffer hash of the already
inspected buttonless status image. Any different nontrivial mapped top-level
window would have stopped the phase and captured its frame without input.
The known status appeared and disappeared automatically; the final frame was
the empty Xvfb screen with cursor. No installer, trust, login or approval
dialog was observed in the bounded capture. This is not proof that a future
program cannot present a later prompt. No game files, `ClientConfig.xml`,
Steam settings or source defaults were copied or changed.

Wine's retained diagnostics include unavailable logical-CPU information,
early COM marshaling errors, experimental WoW64 subprocess notices, setup
copy-style messages, failure to start `winebth`, and failed hostname resolution.
They were not suppressed or repaired by installing packages or widening the
sandbox. These observations limit claims about runtime compatibility. The
namespace had no external network route; individual Wine network attempts
were not traced and REA itself does not observe network traffic here.

**VERIFIED, owned cleanup:** The supervisor terminated only processes visible
inside its verified private PID namespace, then stopped its own Xvfb. It
reported no remaining live children, and each REA scenario reported its owned
process group cleaned up. REA also retained EACCES ownership unknowns for
other host PIDs. A separate read-only recheck found those PIDs either gone or
identifiable as kernel workers with parent PID 2; none was signaled. REA's
sampling limitations remain in the exported evidence. No host service,
existing Wine server, launcher, source installation or other session was
stopped. The REA evidence was exported and its investigation reset afterward.

Private artifacts under `evidence/native-runtime-prep/`:

| Artifact | Purpose |
| --- | --- |
| `preparation-receipt.json` | Successful and failed phase results, file hashes, bounded claims, and external-PID recheck; SHA-256 `c992a83a7cdf605c234547d2a198b05334a30358b6c2ded4c54707b2f752ff1f`. |
| `rea-preparation-final.bundle.json` | Three actual REA process captures and their explicit unknowns; SHA-256 `66bef9e84beb1d94e9db5a35afd32f3ea1f9d154898579497fcf1e6caab29b88`. |
| `run-20261115T091948Z/receipts/result-wineboot-fresh.json` | Successful process exits, no native input, no retail launch and owned cleanup. |
| `run-20261115T091948Z/logs/wineboot-fresh.stderr` | Unsuppressed Wine diagnostics. |
| `run-20261115T091544Z/captures/screen-wineboot-first-visible-window.{xwd,png}` | Inspected, buttonless status window from the stopped first phase. |
| `run-20261115T091948Z/captures/screen-wineboot-fresh-settled.{xwd,png}` | Final empty-display observation; no game frame. |

Successful REA capture:
`ev_981ae56fbde2fc4c5fe809086ab8beb83a0cdc14c1a082188c8073d1b608c8c8`.
The interrupted first capture and unsuccessful continuation are
`ev_ed2e436984015e2dbd2342d7f26cbcd5931cb5fbdcf2ce06be56ac5f8d771bc2`
and `ev_eec73e2a9e49647f61c949a26482d6622603d9c7d0769f0165ba4a9f50738420`.

The remaining sections preserve the initial readiness observations and proposed
game-capture sequence. References there to unrun prefix/display checks describe
that earlier inspection; the explicit preparation and retail results above
supersede them. Full frontend startup, asset completeness, Steam behavior and
graphical parity remain unresolved.

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
| Frozen candidate files and generated INIs, `Launch.log`, minidump file identities and private screenshots listed in `evidence/native-retail-reference/retail-reference-receipt.json` | Owner-supplied proprietary game and private runtime output; no redistribution | Completed-copy verification, actual settings-failure stage, observed version and generated-file paths. No source-install personal configuration was read. |
| `evidence/retail-pe-imports.bundle.json`, REA evidence `ev_7136989a0b4ae1b35612e44bddcbe16bc3fbf8ad0f1245aaed529cbdb780c919` | Original private evidence over the owned binary | Main executable import cross-check. |
| `/usr/bin/wine-stable`, `/usr/bin/wine64-stable`, installed Wine executables/DLLs, `/usr/share/man/man1/{wine-stable,wineserver-stable}.1.gz`, `/usr/share/doc/{wine,wine64}/copyright` | Wine principally LGPL-2.1-or-later; bundled/packaging notices retained in installed copyright file | Wrapper selection, prefix/server variables, version and builtin availability. No Wine code copied. |
| `/usr/bin/Xvfb`, `/usr/bin/xvfb-run`, `/usr/bin/xauth`, `/usr/share/man/man1/Xvfb.1.gz`, `/usr/share/doc/{xvfb,xauth}/copyright` | X.Org MIT/X11-style and component notices | Version, isolated display/auth plan and documented framebuffer format. |
| `/home/blizz/.local/bin/bwrap`, `/usr/bin/bwrap`, `/usr/share/man/man1/bwrap.1.gz`, `/usr/share/doc/bubblewrap/copyright` | Bubblewrap upstream/package LGPL-2.0-or-later; local binary's package identity remains unverified | Namespace help/manual contract, hashes and read-only capability probe. |
| `/home/blizz/.local/bin/ffmpeg` version, licence, decoder/device output | Installed johnvansickle.com FFmpeg build reports GPL-3.0-or-later | Capture/conversion capability only; no binary redistribution. |
| Installed `/usr/lib/x86_64-linux-gnu/libX11.so.6`, Python 3.14, and `/usr/share/doc/{libx11-6,python3.14}/copyright` | X11 permissive component notices; Python PSF and bundled component notices | Original supervisor calls installed X11 APIs to observe windows/authentication and uses the Python standard library for process bounds and evidence. No external implementation copied. |
| `/usr/share/doc/{libgl1-mesa-dri,libvulkan1}/copyright` | Mesa has MIT and other component notices; Vulkan loader has Apache-2.0/MIT notices | Installed runtime licence metadata only; no renderer implementation study. |
| Installed package metadata, Linux `/proc` capability values, device metadata, trusted-host loader dependency reports | Host operational metadata; no external implementation reused | Record runtime availability and unresolved rendering requirements. |

No executable game bytes, disassembly, raw configuration files, captures or
external implementation code are included in this repository document.
