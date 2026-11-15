# Reduced startup metadata trace

Task `a3d58c579542`, 2026-11-15 UTC.

The first trace exhausted its 20,000 selected-syscall cap during early startup.
Its original source and receipt remain unchanged. Root authorized one fresh
45-second observation with the same 55-second outer bound, 20,000-call cap,
8-MiB collector cap, original standalone arguments and immutable 453-file
snapshot. Only the metadata allowlist changes:

```text
open,openat,openat2,access,faccessat,faccessat2,socket,connect,getsockname,getpeername
```

**OBSERVED:** Filtering the prior retained trace to this list leaves 7,773
completed calls and 1,463,710 original emitted bytes, versus 20,000 calls and
3,443,729 bytes. That is a retrospective count, not a prediction of runtime
or next-run event coverage. All 69 game-root missing-path names in the prior
capture appeared only in omitted status probes. These include two configuration
candidates and 67 DLL/DRV search candidates. Therefore a missing-input request
can disappear entirely from the reduced trace; absence cannot establish that
an input was unused or unnecessary.

The 100 retained relative `AT_FDCWD` opens in the prior capture have explicit
`-y` directory annotations. Omitting cwd changes does not erase those recorded
bases; one nonempty relative `access` request remains unresolved. Any future
unannotated relative path also remains unresolved. No referenced
file is opened merely to normalize a path.

The same private authenticated Xvfb, fresh clean prefix clone, generated loopback
hostname mapping, network/PID isolation, read-only game mount, native prompt
stop and owned-process cleanup apply. No UI input, source install changes,
existing-prefix access, config retrieval, endpoint changes, payload tracing or
network egress is authorized by the narrowed observation.

## Evidence status

Private run: `evidence/native-retail-reference/run-20261115T102012Z/`.
Preceding trace reduction receipt:
`run-20261115T101116Z/receipts/trace-analysis/reduction-assessment.json`, SHA-256
`29dc021d28f3e82e8e7fd715f4eb88275a4a3f7dd9751b7df3bd700ec5477600`.

**VERIFIED:** The 453 frozen distribution files and 1,755 independent clean
prefix copies matched before launch. Read-only namespace preflight verified
loopback-only interfaces, no route to external networks, hidden owner home,
and a single read-only game mount. The only supervisor code change was the
approved syscall allowlist substitution; the collector stayed byte-identical.

**OBSERVED:** The one attempt reached the time deadline, with 12,914 selected
calls and 2,505,200 retained bytes. It did not hit the syscall or byte cap, and
discarded no bytes. Pairing found 12,609 single-line calls plus 305 exact
unfinished/resumed pairs, with no orphan or pending records.

| Event | UTC |
| --- | --- |
| Supervisor namespace record | 10:22:59.825 |
| Tracer launch | 10:23:00.066 |
| First recorded primary game window | 10:23:21.865 |
| Deadline stop | 10:23:45.436 |
| Owned cleanup complete | 10:23:46.081 |

The 18 periodic frames show no new prompt or error window. The known primary
954-by-508 game window appears and remains essentially black. The captured
cleanup process inventory identifies `MarvelHeroesOmega.exe` in the owned
namespace. Neither that process observation nor black frames establish
successful frontend initialization or visual parity.

The supervisor stopped its tracer with SIGTERM at the deadline and completed
owned cleanup; no live owned process remained. Xvfb and the REA wrapper exited
zero. The collector reached clean EOF with no startup/read/write/finish error.
REA's inaccessible host-PID entries were seven kernel workers and two processes
with unrelated SSH-service ancestry; no unrelated process was signaled or
attached. The receipt's `visible_windows` field records its exceptional-window
stop list, not all prior windows; periodic records supply the actual primary
window observations.

Successful `openat` calls now include `Data/Game/mu_cdata.sip` and
`Data/Game/Calligraphy.sip`. These are positive handle-acquisition observations;
content consumption, decoding and later loading remain unproven. No
`ClientConfig` token appears in the retained trace. This profile omits existence
probes, so it does not establish that the file was not queried. The original
configuration retrieval condition remains untriggered.

The paired inventory contains 781 successful game-path events, including 766
opens over 211 distinct game paths. It includes 16 INI-named paths, shader/cache
BIN paths and `TextureFileCacheManifest.bin`. No status-derived file kind is
assigned: a successful path open is not automatically a regular file. There is
no recorded game-path `ENOENT` in this reduced profile; omitted existence checks
make a complete missing-input inventory impossible.

The 45 recorded connection attempts remain local: 28 missing nscd paths, two
missing D-Bus paths, two missing Pulse paths, one successful private Wine socket,
ten successful X97 sockets and two loopback port-631 attempts returning
`EINPROGRESS`. No remote connect destination appears in this retained trace.
Unconnected UDP/DNS sends and payloads remain outside the profile, and loopback
`EINPROGRESS` is not established connection success.

The fresh prefix contains a zero-byte `Launch.log` and no observed minidump.
The generated Wine/CEF diagnostics include unavailable NTLM support, audio-device
lookups, `WSALookupServiceBeginW` and WinSAT assessment failures. Their causal
role in the black window is unestablished; no package, host service or runtime
configuration was changed in response. No concrete `Launch.log` error marker
triggered a stop. After cleanup, all 453 frozen
input files still match their pre-run lengths and hashes; private hosts and
supervisor hashes also match. No input file or settings was added between runs.

REA Evidence:
`ev_55d93a6adadb92d7eb02938d92479ed5d2dee9576e4f28c5fc2ddf3b8199cdee`.
Untruncated bundle `receipts/rea-reduced-metadata-trace.bundle.json` SHA-256:
`7986cf5e8777fa8ed77f5d90fe19f97ae12e465660a83ceeb03ce3ef205d0454`.
REA was exported and reset. Its ordinary process-sampling and executable-binding
limitations remain. Independent raw trace `trace/events.log` SHA-256:
`b95e4cb34f9fd3d9af7583d121217a684891c5c7c69fd0ed9d2d829d04593d3c`.
Post-run 453-file hash receipt SHA-256:
`80da771cf81036946408d68fa7bdae3b6e77c5e3cd999baea5465ebdf4e64967`.
Fresh-prefix clone receipt SHA-256:
`200d8913cefec92d5f27f6d574b784feee5afa53b354b9b98a9c0bc9bab996f9`.

Paired metadata receipt `receipts/trace-analysis-v2/trace-facts.json` SHA-256:
`a0d0ebb14dcd13eae7b9fabe489f3f6a53f3d668d5502ffcdbc3e026d1a8c0ac`.
Original adapted parser SHA-256:
`1d5df7842d089688fb3eee395e94b760ff4541bb31437fa85e0d3ac65a3219f4`.
Aggregate `receipts/finding-receipt.json` SHA-256:
`d1a346e9104b942f90a3f489e291562cdefcaadb4a47354e66ea527b570874f4`.
Final detailed path/connection/log analysis receipt
`receipts/trace-analysis-v2/analysis-receipt.json` SHA-256:
`4989584b0207f2785c80a380e4354fb2ea162cad4d54e81064550f496331a41d`.
Generated-log findings SHA-256:
`65cb2dc63b7aaa5718db2c4319edd2d32497b70f9f2ba17ca009286dd6abfac3`.
The source trace, original broader-trace receipt, full metadata offsets and
mapping proof remain unchanged and private.

## Sources

Reuses the owner-private trace and original parser/supervisor from
`run-20261115T101116Z`, the installed strace 6.19 manual and LGPL-2.1-or-later
package notices, and runtime/REA sources recorded in
`NATIVE_PASSIVE_TRACE_PLAN.md` and `NATIVE_REFERENCE_CAPTURE.md`.
No external implementation is copied. The installed manual's status filter
applies globally; no untested per-syscall status rule is assumed. Its syscall
limit excludes filtered calls, and `-s` does not abbreviate filenames.
All raw metadata, binaries, captures and prefix files remain private.
