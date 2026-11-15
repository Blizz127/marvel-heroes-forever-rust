# First bounded retail startup metadata trace

Observation: 2026-11-15 UTC, task `a3d58c579542`.

**OBSERVED:** The first offline metadata trace exhausted its approved 20,000
selected-syscall limit during early startup, before any visible game window or
`ClientConfig.xml` request. It retained every emitted metadata byte and completed
owned cleanup. The tracer's zero exit is cap completion, not natural retail
completion or proof of a working frontend. No configuration was retrieved or
changed, and no UI input was sent.

## Instrumentation and actual execution

Private run: `evidence/native-retail-reference/run-20261115T101116Z/`.
The exact [metadata profile](NATIVE_PASSIVE_TRACE_PLAN.md) wrapped the unchanged
Wine loader and existing owner's standalone arguments. The 45-second inner and
55-second outer bounds, 20,000 selected-syscall limit and 8-MiB retained-byte cap
were used without increasing them after startup. No syscall result, target
memory, credentials, host policy or network configuration was changed.

A harmless synthetic child first verified ptrace and FIFO operation inside the
same isolation. It captured an intentional missing path and loopback connection
metadata, retained 30,786 bytes, exited zero, and left no live children. The
collector's separate synthetic checks exercised bounded retention/draining,
ordinary EOF and a timed-out finish. The original collector module SHA-256 is
`c08a7087822b728aa130cdbd3327f7dd42de6e4e9d262f12a6b23abb430d153c`.
No game or Wine process ran during that preflight.

Both the original 453-file frozen snapshot and the fresh 1,755-file clean prefix
clone were hash-verified before the one retail trace. The private `devbox`
loopback mapping, isolated network/PID namespaces, read-only `/game` mount,
hidden owner home and authenticated Xvfb constraints were preserved.
`ClientConfig.xml` remained absent from the frozen snapshot.

REA separately sampled owned PID 7 first as the Wine loader with the exact
standalone arguments and later as `MarvelHeroesOmega.exe` with those arguments.
This supplies process evidence beyond the tracer wrapper's creation alone.
It is not proof of completed initialization. No primary game window appeared in
the retained X11 observations, and no game log or minidump was generated.

| Event | UTC |
| --- | --- |
| Tracer started | 10:13:42.163 |
| Tracer exit observed | 10:13:55.240 |
| Supervisor cleanup finished | 10:13:55.887 |

The collector retained 3,443,729 bytes, discarded zero, and reached clean EOF
without startup/read/write/finish errors. No live owned child remained when the
supervisor began final cleanup; Xvfb also exited zero. REA reported an ordinary
exit-zero wrapper result and verified owned process group. Its additional
inaccessible host PIDs were checked and were kernel workers or already gone.
No unrelated process was signaled.

## Captured scope and interpretation

The trace contains 21,415 physical lines: 20,000 syscall starts plus 1,415
resumed records. Its selected-syscall limit explains the terminal observation;
its 8-MiB byte limit was not reached. This is deliberately bounded coverage of
early startup, with no later-call absence claim.

No `ClientConfig`, `/Data/`, or `Launch.log` token appears in the retained trace.
The final calls inspect the fresh prefix's generated Marvel configuration
directory and `MarvelSystemSettings.ini`. Therefore the condition for inspecting
the original `ClientConfig.xml` is not established by this capture. A failed
optional Wine/library search does not establish a fatal game dependency.

**VERIFIED:** Pairing found 18,585 single-line calls and 1,415 complete
unfinished/resumed pairs, with zero orphan or unresolved records. The inventory
contains 753 game-path metadata events over 91 paths. Nineteen successful
regular-file opens cover the executable, ten bundled DLLs and three shipped
configuration INIs: `DefaultEngine.ini`, `DefaultSystemSettings.ini` and
`BaseSystemSettings.ini`. Opens establish handle acquisition, not contents
consumed or decoded.

The only missing distribution paths outside binary search candidates are
`MarvelGame/Config/localSettingsDirectoryName.txt` and `MarvelSystemSettings.ini`,
with four failed status probes each. No fatality is established. All 67 missing
DLL/DRV candidate basenames also have a successful case-insensitive basename
match elsewhere in this trace; this establishes search alternatives, not binary
identity or version equivalence.

All 34 connection attempts target local paths or loopback: 24 failed nscd
lookups, two failed system-D-Bus lookups, one successful private Wine socket,
five successful authenticated-display sockets, and two loopback port-631
attempts returning `EINPROGRESS`. The latter is not confirmed connection
success. No remote/server connection endpoint appears before the cap.

The exact paired syscall/path/connection inventory is retained privately with
source byte/line references. Missing-file conclusions must be qualified by a
later successful search of the same logical input where observed. Connection
coverage is limited to the approved metadata calls; send/receive buffers and
unconnected UDP/DNS sends were excluded. No network payload or game-file content
was captured. Trace overhead and the early syscall cap prevent timing or visual
parity conclusions.

## Integrity and evidence

**VERIFIED:** After cleanup, all 453 frozen game files still matched their
pre-run lengths and SHA-256 values. Private hosts and supervisor hashes were
unchanged. `receipts/post-run-snapshot.json` SHA-256:
`3111b772e4f6bcd7cea58ca29986617ad9ec226a689daa91777412a44954abef`.
The prefix-clone receipt SHA-256 is
`bf9918ce440e29f3f75b9b580769390e32e417260f63898f3e6235717cb95e6e`.

REA Evidence:
`ev_9a1f03c9bce871f8bfca4f82431c074b16dd733c1c4cadac70a23abdd97958b4`.
Bundle `receipts/rea-metadata-trace.bundle.json` SHA-256:
`01af45d6f212952610e7316ee56a4db46d1c11a0a3ec095bac90c950ef46e86c`.
No requested terminal/process/filesystem output was truncated; normal process
sampling and artifact-binding limitations remain. REA was exported and reset.
The independently retained raw trace `trace/events.log` has SHA-256
`97f8b0aa578c534e3633bef5a511de625e15079166033c5c66a226aa850f7bed`.

`receipts/trace-analysis/trace-facts.json` SHA-256:
`28f188e0f2d3ff03a67733fc75d029121a0bd6c1d2987d4a04244f44ca6d8e88`.
Original bounded parser SHA-256:
`873494fa4d4936f571fcb763cccf747409a0c9f6770c26b5f220589fe3b409ab`.
Aggregate `receipts/finding-receipt.json` SHA-256:
`8c20c6d226ad0f649db2b906092167035744635a008ce7e4b37543eef7adc8f3`.
The aggregate preserves and corrects one inventory wording error: filenames
are not abbreviated by strace's `-s` limit; the separate collector byte cap
provides the bound. No pathname truncation was observed.

No new external implementation source was read. The proprietary owner snapshot,
original supervisor and collector, installed strace 6.19 and its LGPL-2.1-or-later
package notices, existing runtime dependencies and mandatory REA workflow are
recorded in `NATIVE_PASSIVE_TRACE_PLAN.md` and `NATIVE_REFERENCE_CAPTURE.md`.
All traces, raw paths, binaries, captures and prefix files remain private and
excluded from source control/builds.
