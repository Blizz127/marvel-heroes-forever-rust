# Isolated hostname-corrected retail observation

Observation: 2026-11-15 UTC, task `a3d58c579542`.

**OBSERVED:** One new offline retail probe reached a native `Message` dialog
stating “Please run the game through the launcher.” It was captured and stopped
without clicking its OK button or sending any input. The primary game window
remained essentially black; no login frontend or renderer parity was observed.
The required-file branch for `ClientConfig.xml` was not observed, so conditional
authorization to retrieve that file was not exercised.

This follows the [read-only postmortem](NATIVE_STARTUP_SETTINGS.md), which found
an unchecked null `gethostbyname(devbox)` result in the prior run. The present
probe changed only the isolated hostname mapping among the game's environment
inputs. Different scheduling, fresh process identities and a fresh prefix clone
also distinguish the runs. The changed observation supports the usefulness of
the mapping correction, but is not direct measurement of the native Winsock
return or proof that every later startup dependency is satisfied.

## Exact private preparation

The new run is private:
`evidence/native-retail-reference/run-20261115T094721Z/`.
The clean initialized prefix from
`evidence/native-runtime-prep/run-20261115T091948Z/prefix` was copied into its
new `prefix` directory. All 1,755 regular files were hash-matched and checked for
independent inodes; both source symlink targets were preserved without following
them. No existing owner prefix or previously executed retail prefix was used.
The clone receipt SHA-256 is
`69b10e9ceac560971a668a9a14017e16a4ab2774ec542a6d1af55c708ddcf7df`.

The generated private `/etc/hosts` contained exactly:

```text
127.0.0.1 localhost devbox
::1 localhost
```

Its SHA-256 before and after execution is
`80b35dc4505c04b70c1aad49ee5bdf380a2fbaf0391f91fd3d99692cd6a831e0`.
Only the `devbox` alias was added relative to the previous private hosts file.
The host machine's resolver configuration was not changed. Separate namespace
preflight and the runtime supervisor both confirmed hostname `devbox`, files-based
resolution to `127.0.0.1`, a different network/PID namespace, only loopback,
no routes, hidden owner home, and no shared X11 socket. This Linux resolver check
is distinct from the game's Wine Winsock behavior.

The original frozen 453-file, 5,115,044,673-byte game snapshot at
`run-20261115T092318Z/game` was reused through a read-only `/game` mount, with
no writable alternate alias. Its incompleteness and exclusions remain as listed
in [the capture record](NATIVE_REFERENCE_CAPTURE.md). `ClientConfig.xml` remains
absent and its contents were not read. The source install and active transfer
were neither changed nor used as writable runtime storage.

The same argument-free executable ran from `/game/UnrealEngine3/Binaries/Win64`
through the same direct installed Wine loader. No new game arguments, Steam
configuration, endpoints, credentials, accounts, devices or host service access
were introduced. Authenticated Xvfb and new private auth/home/log/capture
directories were created inside the isolated process environment. No-cookie
X11 access failed as expected. Every runtime stdin was closed or reserved for
the private setup command; no game input events were supplied.

## Terminal result and cleanup

| Event | UTC |
| --- | --- |
| Retail executable launched | 09:50:06.368 |
| Additional `Message` window detected; stop initiated | 09:50:46.489 |
| Painted dialog captured | 09:50:46.846 |
| Owned cleanup finished | 09:50:47.721 |

The new dialog appeared about 40.1 seconds after launch, within the independent
55-second outer execution bound. Fifteen periodic frames recorded the known
primary window; the observer also retained immediate and painted dialog frames.
The supervisor permitted the single known game window, stopped additional
windows, and had a passive fatal-log stop plus the wall-clock bound. Window-title
classification is not a complete semantic prompt detector; the captured dialog
was independently inspected visually after stopping.

**VERIFIED:** The private PID-namespace cleanup reported no remaining live
children. Wine received the supervisor's termination signal (loader result
`-15`), and Xvfb exited zero. The supervisor's exit zero is cleanup success, not
retail-game success. REA reported its owned process group verified. Its additional
inaccessible host PID observations were cross-checked: the surviving entries were
unrelated kernel workers or a containerd shim; another had already exited. No
unrelated process was signaled. No native debugger command or UI action occurred.

`Launch.log` remained zero bytes and no new minidump was produced. The dialog
therefore establishes a launcher prerequisite message, not a specific activation,
account, Steam or missing-settings cause. Read-only identification of its exact
native condition and the normal supported launch path is separate work. There
will be no attempt to dismiss, patch or bypass this dialog from this receipt.

## Retained evidence and limits

REA runtime Evidence:
`ev_d9a6f6d564453cc2d15e45f62bffa6b02a006456d1bec9303c2a1c1684e51473`.
The bundle `receipts/rea-hostname-probe.bundle.json` has SHA-256
`c2989650b21959bb6f48cb237913a27029fce81810812715c68ab69a44bf272f`.
REA was exported and reset after the terminal result.

The runtime record's truncation flag is **true** for one specific reason: its
131,072-byte filesystem hash budget omitted the 483,621-byte `prefix-clone.json`
digest in both before/after snapshots. Enumeration, all 26 raw/rendered terminal
frames, and the sampled process records were not truncated. The omitted receipt
has a separately recorded whole-file hash above. Sampling still may omit brief
processes; filesystem observations are not syscall traces.

The dialog image remains private at
`captures/screen-retail-hostname-new-window-painted.png`, SHA-256
`c8d0fe17b91d40f4cdee201ec5834ba882b2fd821914eb55bafa146c537e196d`.
`receipts/generated-hosts.json`, `namespace-preflight-output.json`,
`prefix-clone.json`, `result-retail-hostname.json`, and
`host-cleanup-crosscheck.json` retain the exact controls, checks and outcomes.
**VERIFIED:** All 453 frozen game files still matched their pre-run hashes and
lengths after execution. `receipts/post-run-snapshot.json` has SHA-256
`b4c812b27c90fae7daad00c456d062ed231bd9fcc9dfac94922b4c4bbc7dc4af`.
The private hosts and supervisor hashes also remained unchanged. The final
aggregate `receipts/finding-receipt.json` records every subordinate receipt,
exact timing and the untriggered ClientConfig condition. Raw bytes and captures
are never committed.

## Sources and provenance

No new external implementation or web source was used. This increment reused:

- The owner's proprietary executable and original frozen game files, plus
  self-generated runtime observations: private compatibility study only.
- The installed Wine 10.0 runtime, LGPL-2.1-or-later; installed Xvfb and xauth,
  permissive X.Org licences; local bubblewrap, LGPL-2.0-or-later; and the installed
  FFmpeg 7.0.2-static build, GPL-3.0-or-later. Exact paths, package notices and
  provenance limitations remain in `NATIVE_REFERENCE_CAPTURE.md`. No packages
  were downloaded or installed and no implementation source was copied.
- `~/.agents/skills/reverse-engineer-anything/SKILL.md` and its
  `references/runtime-observation.md`: mandatory REA 6.9.0 capture/evidence
  workflow. The original private supervisor and snapshot scripts were adapted
  from this project's earlier isolated observations.
