# Complete selected-distribution offline observation

Task `a3d58c579542`, 2026-11-15 UTC.

**Later argument reconciliation:** this historical run used the inherited
`:8443/SiteConfig.xml` argument. Current local launcher source selects HTTP
port 80 instead. [Service entry evidence](MHSERVEREMU_TEST_ENTRY.md) records the
conflict without changing the sealed run. Its offline observation is not proof
that the supplied service argument matches the current launcher contract.

The preceding [454-file observation](NATIVE_CONFIG_PRESENT_PROBE.md) reached a
successful read-only open of the exact owner configuration but left an
essentially black primary game window. Root authorized one new frozen input set
containing every file in the now-verified selected distribution, plus that same
original configuration. The normal standalone arguments, offline namespace,
fresh prefix, private hostname mapping, no-input rule and bounded cleanup remain
unchanged. [Snapshot planning](NATIVE_FULL_SNAPSHOT_PLAN.md) records the scope and
copy preflight.

Private run: `evidence/native-retail-reference/run-20261115T104858Z/`.
All relative evidence paths below refer to this run unless stated otherwise.

**VERIFIED source selection:** Root's terminal full-pin receipt covers 16,489
regular files and 21,193,180,456 bytes, all matching the owner source manifest.
Receipt `evidence/alienware-full-pin-verification-20261115T1050Z.json` was observed
at 10:48:11 UTC; its filename is an identifier, not the observation time.
SHA-256: `a8994255f33d3bb5dc5da510d755d076b80028fc28d04c0363fffe0d116862c8`.
The exact original 256-byte `Data/Configs/ClientConfig.xml` is the sole addition
to that selection. Its schema/privacy inspection is recorded in
[NATIVE_CLIENT_CONFIG.md](NATIVE_CLIENT_CONFIG.md); values remain private.

This selection has 48 remaining original inventory exclusions. Completing it
does not establish that every possible launcher, platform, online service or
owner-install dependency is present, nor that all files are vendor-pristine.

**VERIFIED snapshot:** 16,490 regular files, 21,193,180,712 bytes and 58
directories including the root. Each source stream and independent destination
hash matches the bound inventory/configuration. Source identities were checked
again through finalization, destination inodes are independent, and all prior
454 paths, sizes and hashes are preserved. Files are `0400`, directories `0500`,
with no symlinks or hard links. The final name was installed without replacement
only after verification; staging is absent.

The initial assembler was stopped by an identity-checked signal to its exact
owned process. A controlled resume removed unnecessary per-game-file `fsync`,
fully rehashed both source and destination of 1,315 existing files, and copied
15,175 new files. No incomplete file required quarantine. Receipt writes still
used `fsync`; no global sync, unrelated signal, source mutation or overwrite
occurred. The resumed assembly completed at 10:57:28 UTC in 171.6 seconds.

| Private receipt | SHA-256 |
| --- | --- |
| `receipts/preparation/frozen-inputs.json` | `69351480fb0cf51f22be28397d4ed9a619e60a63ddb1c505fe62ca9a8534f5a6` |
| `receipts/preparation/full-snapshot.json` | `33e060b62653af0fea0235d169a0fea8e55b712ed02271bceb26159c374254e7` |
| `snapshot-preparation/controlled-resume.json` | `e50d5f9865467ab6c74f0b1c4a0c92105bcbb515ac4f120e40e559f1d8cbbd3a` |
| `receipts/verification/pre-run-snapshot.json` | `7e81e8bf39c02d19e4be546e2c0bc30bc170ae126222ba1bc1d4d64f1f0a9990` |

The prepared run separates large copy/prefix verification manifests from the
small runtime receipt directory observed by REA. Only the latter is mounted as
`/capture/receipts`; this prevents the preceding run's aggregate filesystem hash
budget omission from being silently repeated. A single read-only `/game` mount
has no writable alias. All eight writable mounts belong to this private run.

A clean 1,755-file baseline prefix clone retained every source hash, with two
expected drive symlinks and independent destination inodes. The actual namespace
check passed. A fresh metadata-only fixture exercised all 42 cases: 36 target or
parent calls were captured, and six unrelated negative calls were omitted.
Current coverage receipt SHA-256:
`834f58237237c359f5c6b0b9dc56de3efe4f49cfbae0d8977a103c784393168c`.
This fixture reads no configuration content and does not exercise every
allowlisted syscall form; the limitations in the preceding trace documents
remain.

**OBSERVED:** The tracer started at 10:58:09.504546 UTC. At 10:58:29.005726, the
conservative observer stopped on a second mapped window titled `SplashScreen`,
at `(360, 202)` with dimensions `1200 × 675`. The existing primary game window
remained at `(483, 299)`, `954 × 508`. The saved painted capture 0.275 seconds
later shows the actual Marvel Heroes Omega loading artwork and `LOADING...`
text. No login, trust, approval or error controls are visible in that saved
frame. No native UI input was sent. This is an ordinary loading-splash
observation; it does not establish frontend completion or visual parity.

The original supervisor recorded titles and geometry but did not record
`WM_CLASS` or `_NET_WM_PID`. The owned process list establishes that retail
`MarvelHeroesOmega.exe` was running, alongside Wine services; it does not bind
each X window to a particular process. No missing window-property evidence is
invented.

Owned cleanup began at 10:58:29.283990 and finished at 10:58:29.904433. The tracer
received cleanup SIGTERM, Xvfb exited zero, and no owned live process remained.
REA exited zero. Its additional host-process ownership uncertainties comprise
six subsequently observed kernel-worker PIDs and five already-gone PIDs whose
identity was not established; none was attached or signaled.

The selected configuration-path trace is genuinely empty: zero emitted,
retained or discarded bytes, no collector cap/error, and clean EOF. It provides
no configuration-open observation for this shorter run. General asset opens,
network attempts, content reads and native call stacks are outside this trace's
coverage. The changed visible outcome follows the larger input set, but the
exact asset or startup operation responsible is not isolated by this probe.

Painted XWD SHA-256:
`93ca5bf5c8ac77b3d0c7cb210a5acabaca82c0c4085bb6bc4d3f7889226a660d`.
Privately decoded PNG SHA-256:
`189df38b37f5bd4cd45abf4c9630caa950196a1116f8dd9f14666b6f202c4f56`.
The 13 recorded XWD frames comprise 12 identical black-screen captures and this
one different painted splash frame. No mock frontend was produced.

REA Evidence:
`ev_8e7f06165be7a0eb122f2a5e2e3d0c67e345b50991967a3d39f21d9c500cc5c5`.
Exported `receipts/verification/rea-full-snapshot.bundle.json` SHA-256:
`6a9cd31171032ce08669adc4b0d3525df744c1e82f1357b3c1aedd42c57554bb`.
REA reports no terminal, filesystem or process-budget truncation; its before
and after filesystem hash observations cover 37,224 and 39,034 bytes with no
omissions. Sampled process coverage and prelaunch-image-binding limitations
still apply. The bundle was exported and the investigation reset.

**VERIFIED after cleanup:** All 16,490 inputs retain their exact pre-run sizes
and hashes, including the original configuration. Private hosts and supervisor
hashes match. `Launch.log` remains empty. Post-run receipt SHA-256:
`58764b6564b563ab46c6a4a159f180988eb23846d89bdbdaa71b3ab38d567b1c`.
Aggregate `receipts/verification/finding-receipt.json` SHA-256:
`de3a26c83c083ed605400e4a750cafc9830b5481f51dbfd7f4de0f7afa632b40`.

Sources and licences: owner-selected game distribution and original
configuration, proprietary/private compatibility study only; original snapshot,
supervision, collector and metadata fixture scripts; installed GNU coreutils
9.7 help/version text (GPLv3-or-later), used only to assess unsupported reflink
copy, as recorded in the planning document. Existing Wine, Bubblewrap, Xvfb,
strace, Python and REA sources are recorded in the linked preparation/trace
documents. No new third-party implementation was copied or dependency
installed. Raw game bytes, configuration values, logs, frames and manifests
remain private and outside source control/builds.
