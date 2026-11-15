# Targeted configuration-path metadata observation

Task `a3d58c579542`, 2026-11-15 UTC.

The reduced trace reached real SIP package opens but omitted existence-only
status calls. Root authorized one further fresh offline observation, contingent
on proving the exact-path filter with harmless missing-path probes first.
The same 45-second inner and 55-second outer bounds, 20,000-call limit,
8-MiB collector cap, original standalone arguments, clean prefix and immutable
453-file snapshot remain in force. No configuration contents are read or added
by this plan. A synthetic missing-path probe cannot trigger authorization to
retrieve the owner's original configuration.

**OBSERVED:** Read-only metadata confirms `/game/Data` is present while
`/game/Data/Configs` and its `ClientConfig.xml` child are absent from this frozen
snapshot. This does not inspect the original install or the active full pin.
The native path contract remains the one recovered in
`NATIVE_STARTUP_SETTINGS.md`; the ordinary source/launch controls remain those
in `NATIVE_REFERENCE_CAPTURE.md` and `NATIVE_LAUNCHER_GATE.md`.

Candidate metadata calls:

```text
open,openat,openat2,access,faccessat,faccessat2,newfstatat,stat,lstat,statx
```

The first candidate repeated `-P` for the normalized target, its absent parent
and existing `Data` ancestor, under both `/game` and the verified Wine z-drive
alias. The harmless preflight rejected that candidate: canonical and Wine-alias
absolute probes were captured, but all three cwd-relative target probes were
missed. Selecting the existing ancestor also captured all three unrelated
directory-fd sibling probes. The fixture itself passed all 18 expected-absence
operations, and collector/process cleanup passed. This was a filter-coverage
failure, not a permission denial or native execution failure. Failed-only
filtering would not repair either defect.

The revised profile removes the existing `Data` ancestor and names the missing
`Configs` parent and `ClientConfig.xml` in these observed/tested forms:

- Canonical absolute path under `/game/Data`.
- Absolute path through `/capture/prefix/dosdevices/z:/game/Data`.
- Raw `../../../Data/Configs` path from the actual game working directory.
- Raw `Configs` path relative to a directory fd for `/game/Data`.
- Uncollapsed absolute paths containing the actual
  `/UnrealEngine3/Binaries/Win64/../../../Data/Configs` sequence, both canonical
  and through the Wine alias.

Each parent and final-file spelling is an explicit `-P` argument; matching is
not assumed recursive. Preflight two issued 42 operations across those target
and parent forms plus unrelated absolute/dirfd negative controls. Exactly 36
intended ENOENT calls were captured and all six negative calls were excluded.
It exercised `newfstatat`, `access`, `faccessat2` and `openat`; legacy stat/lstat,
statx, open and openat2 remained allowed but were not exercised by that fixture.
The trace retained 6,634 bytes, with clean EOF and no live owned process.

Raw-relative filters can match the same spelling under a different runtime
base. Actual findings therefore require the captured `-y` cwd/dirfd annotations
or remain unresolved. Bare relative `access` lacks such an annotation and must
not be assigned an invented cwd. No filtered negative result establishes that
all possible pathname spellings or syscall variants were observed.

Preflight one coverage receipt SHA-256:
`d4bf18e6a89ccffb0fe9c05a3096021e995a76bc752ddc3ca12003068587382c`.
Preflight two coverage receipt SHA-256:
`10e9d9ae6561903b7a9af01e467deed2cf6a9eb7fe6a45c1238ec2b96cc25606`.
Original first fixture SHA-256:
`f37a940f712a44de1a6ef586ca2643a8abcb3f9a2cbad7446e1b53305560486e`.
Expanded original fixture SHA-256:
`4e4c982f2b4894089fa8bcc060e9ef5ae57ca147559b0c2aec8d1a30c525b05d`.
Both probes used metadata-only `O_PATH` handles, never read target bytes and
created no source/configuration file. Neither synthetic result is native
missing-configuration evidence.

Private preparation: `evidence/native-retail-reference/run-20261115T102749Z/`.
**OBSERVED:** The native attempt selected two calls (421 bytes), both by the
owned retail PID 12. The verified Wine z-drive mapping normalizes them to:

| UTC | Request | Result |
| --- | --- | --- |
| 10:32:49.641464 | `/game/Data/Configs/ClientConfig.xml` | `newfstatat`: `ENOENT` |
| 10:32:49.642284 | `/game/Data/Configs` | `newfstatat`: `ENOENT` |

The final-file request precedes the parent-directory check. Raw source spans
are 0+219 and 219+202 bytes. These are actual native-runtime requests, separate
from the earlier harmless fixture. Their correspondence with the recovered
native XML loader is **INFERRED**: this capture contains no call stack or
instruction-address attribution. Root accepted the positive exact-path failure
as meeting the standing condition for original-file inspection; that follow-up
is documented separately in `NATIVE_CLIENT_CONFIG.md`.

The tracer started at 10:32:32.316 UTC, reached the deadline at 10:33:17.114,
and completed owned cleanup at 10:33:17.775. The collector retained all 421
bytes, reached clean EOF and reported no cap or read/write/finish error.
The supervisor stopped its tracer with SIGTERM; Xvfb and REA wrapper exited zero.
No owned live process remained. Additional inaccessible host entries were five
kernel workers and an unrelated container-service process with parent PID 1;
no unrelated process was signaled or attached.

Nineteen periodic frames include thirteen observations of the known primary
game window. Retained visual checks show it essentially black, with no new
prompt or error window. No input was sent. The terminal `visible_windows=[]`
field describes exceptional-window stops, not the prior permitted primary
window. `Launch.log` is empty and no explicit fatal marker was found.
This does not establish frontend completion, rendering or login parity.

**VERIFIED:** All 453 frozen inputs still match after cleanup, as do the private
hosts and supervisor. Post-run receipt SHA-256:
`58d5f836f369f1568f67ff0f2b951f8300b3083d41568b1d75ca4e17c91a13fd`.
Independent targeted findings SHA-256:
`909784384bc2c9de392dc7d3b295e98c8b96257ba6a46e6e0d8643e5c7220784`.
Raw trace SHA-256:
`c520aa9d1b91b86e32bcdf8d1d5e8829267e15be9e48b7f856e66d4d43f61869`.
REA Evidence:
`ev_47f89d7edf317791644ac954cf36b336001959d0343ab4f562ace663ddf880e1`.
Untruncated exported bundle SHA-256:
`729b3e5f75f0120824b80eddcf75091fd31eef97fb547b1d5dfb3485bdfaa849`.
REA was exported and reset; normal process-sampling and image-binding limits
remain. Aggregate private finding receipt SHA-256:
`9b6c5b09d848102329b613262771fcba9fd9f967dbd343da33b3faccc8346567`.

Sources: installed strace 6.19 help/manual and LGPL-2.1-or-later package notices;
original private namespace/supervisor/collector; owner-private frozen input
metadata and clean-prefix clone receipts. Runtime, REA and proprietary-source
provenance is inherited from the cited observation docs. No new external
implementation is copied or installed. All raw data remains private.
