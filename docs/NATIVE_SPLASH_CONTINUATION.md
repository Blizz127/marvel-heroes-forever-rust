# Offline continuation with the known loading splash permitted

Task `a3d58c579542`, 2026-11-15 UTC.

The [first full selected-distribution observation](NATIVE_FULL_DISTRIBUTION_PROBE.md)
stopped conservatively when the ordinary loading splash appeared. After visually
inspecting that frame and verifying all 16,490 input hashes again, root authorized
one fresh-prefix attempt using the **same** immutable game snapshot. No asset was
copied again. The normal arguments, offline namespace, exact original
configuration, private hosts, no-input rule, 45-second inner deadline and
55-second outer limit were preserved.

Private run: `evidence/native-retail-reference/run-20261115T110131Z/`.
Game mount source remains
`evidence/native-retail-reference/run-20261115T104858Z/game/`.
The preceding complete post-run receipt, SHA-256
`58764b6564b563ab46c6a4a159f180988eb23846d89bdbdaa71b3ab38d567b1c`,
and fresh inode/type/size/mtime/mode checks cover every reused file. A new clean
baseline prefix clone verifies 1,755 files and two expected symlinks; its receipt
SHA-256 is `bb6bb9cbf33cd1d12a069e8e6dac9207976fb6825e11c9f62061a5378e29bc6d`.

The observer permits only the exact previously observed title/geometry pairs:

| Title | Position | Size |
| --- | --- | --- |
| `Marvel Heroes Omega` | `(483, 299)` | `954 × 508` |
| `SplashScreen` | `(360, 202)` | `1200 × 675` |

Duplicate titles, a third visible window, changed geometry, override-redirect
windows, unknown titles or a present PID property that cannot be bound to the
owned retail process cause a stop. New prompt/error observation still requires
stopping with no input. The first capture had no `WM_CLASS` or `_NET_WM_PID`
receipt; no prior class value was invented. The new observer reads both
properties passively. Its explicitly authorized missing-PID fallback would
retain only the private authenticated X-server/PID-namespace context, with no
window-to-process binding claim.

**VERIFIED preparation:** Ten focused policy cases passed, including unknown
title, wrong geometry, duplicate title, unrelated/unverified PID and
override-redirect rejection. A separate unmapped synthetic X window checked
missing properties, round-tripped an 8-bit class and native-long 32-bit PID
property, and rejected an oversized property reply. That fixture launched no
Wine/game process, sent no UI input, used separate private authentication and
cleaned up its owned X server. The namespace check and all 42 metadata-filter
cases passed again (36 selected records, six unrelated probes excluded).

Changed private observer SHA-256:
`379ec5b7108abe6d501cbbd65372e5b34d2630f94429e51a85fc6e49cbd88e94`.
Original property/policy module SHA-256:
`0fea8a8058171d8c5d404f2eb1b9e60e21e28b4c20367cb704ed106683dd9d2e`.
Profile-delta receipt SHA-256:
`54b0a24695b20735d5ffccbb1a909b9cf16faa970d166c98107c914580b1c5e6`.
Current metadata coverage receipt SHA-256:
`eed295fb3b91ab0da91f3315ba16ff433d256c8e1020ea7ebcd8f1ef514c83bb`.

**OBSERVED:** The tracer started at 11:04:09.170610 UTC. The primary window was
recorded at 11:04:28.885632. By 11:04:30.094494, both the primary and splash
windows reported class/instance `marvelheroesomega.exe` and `_NET_WM_PID = 12`.
That PID's bounded command-line read begins with the exact owned
`/game/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`; process start-time
checks were stable around each ownership read. The missing-PID fallback was not
needed for these observations. Window properties are reported client metadata,
supplemented by this isolated runtime/process evidence, not cryptographic
proof of arbitrary X-client identity.

The loading artwork remained visible at the 11:04:54.027012 deadline. Nineteen
recorded XWD frames contain ten identical black-screen frames and nine
identical painted splash frames. The splash hash matches the preceding run:
`93ca5bf5c8ac77b3d0c7cb210a5acabaca82c0c4085bb6bc4d3f7889226a660d`.
No new prompt/error window or visible login/trust/approval controls were
observed. This is a loading-splash reference, not frontend completion, login,
timing or visual parity. Identical saved frames do not prove continuous
coverage between samples.

At 11:04:38.125509–11:04:38.129019, retail PID 12 made nine successful
`newfstatat` checks of the original 256-byte configuration. At
11:04:38.129478, owned Wine-server PID 15 successfully opened that exact path
with `O_RDONLY|O_NONBLOCK`, receiving fd 438. All ten selected calls and 2,178
bytes were retained; the collector reached clean EOF without cap, discard or
error. As before, this establishes metadata success and handle acquisition,
not XML consumption/deserialization or direct native callsite attribution.
The narrow configuration filter does not inventory general assets or network
attempts, so it cannot identify why loading had not completed at the deadline.
The cleanup process list also records owned `marvelheroes_webhelper.exe` PID
1283. Its file/network activity and role in loading are outside this capture;
its presence is not a failure diagnosis.

Owned cleanup finished at 11:04:54.657775. The tracer received cleanup SIGTERM,
Xvfb and REA exited zero, no owned live process remained, and no UI input was
sent. The final exceptional-window list is empty because no exceptional window
caused this stop; it does not contradict the recorded allowed splash windows.
REA's unverified host-PID metadata was checked without attachment or signaling:
nine kernel workers and one container-service process were observed, while one
already-gone PID's identity remains unknown.

The Wine stderr capture contains diagnostic prefixes including `err:ntlm`,
`err:ntoskrnl` and `err:winediag`; their presence is retained in private category
counts. No causal link from those prefixes to the loading duration was
established. No absence-of-errors claim is made from the empty exceptional
window list or from the bounded fatal-log checks.

REA Evidence:
`ev_f1f27260053118aa66d3e214ce312b0731d9a5654028445a9be5d626a0e1980d`.
Exported `receipts/verification/rea-splash-continuation.bundle.json` SHA-256:
`2b6e3fe91a0cf74e6a22ac95d6f6da7b6e5f0cc3419acae77d04d0a2fc995d6f`.
REA reports no terminal, filesystem or process-budget truncation. Before/after
runtime receipt hashing covers 38,647/42,088 bytes with no omissions. Sampled
process and executable image-binding limitations remain. The bundle was
exported and the investigation reset.
The independent metadata receipt, `receipts/verification/splash-continuation-facts.json`,
has SHA-256 `a7aa89440673ea055dbf7b562784d4340e39e3da8d62262d81c74f0286e6e094`.
It preserves the supervisor's generic `retail_execution_observed = null` field
and separately records the actual owned-process and window observations rather
than silently changing that generic field to success.

**VERIFIED after cleanup:** The complete second post-run pass checks all 16,490
input sizes and SHA-256 values, including the exact original configuration;
every value matches. Private hosts and supervisor hashes also match.
`Launch.log` remains empty. Post-run receipt SHA-256:
`8191f58ea9f8ab52b6870490d29738ac40983ca8028b7f785d9375161cdbd132`.
Aggregate `receipts/verification/finding-receipt.json` SHA-256:
`f3a7ce640c72be2a49fa9dd1e85aefd24e89b31fc61b5c50d8a8c75cfeb1d16a`.
Raw ten-call trace SHA-256:
`f9ef38ff9507ba318a719472e0ccfa80fb5f8aaf4e133d028f3c1b04a20fd6de`.

Sources and licences: the owner-selected game/configuration and saved captures
are proprietary/private compatibility evidence. Installed
`/usr/include/X11/Xlib.h` was read for the `XInternAtom`, `XGetWindowProperty`,
`XCreateSimpleWindow`, `XChangeProperty`, destruction and synchronization API
signatures, under its Open Group permissive licence; no implementation was
copied. Header SHA-256:
`7430c535c4e2b0a31d002d5f9d8f21c6b9f7232e9aad2e297662b90c2591f64d`.
Runtime and
strace sources/licences remain those recorded in the preceding preparation and
trace documents. All supervision, property reading and fixture code is
original. No package installation, native setting change, endpoint override,
network egress, existing-prefix access or native UI input was added.
