# Bounded offline standalone-mode observation

Observation: 2026-11-15 UTC, task `a3d58c579542`.

**OBSERVED:** The freshly isolated retail process remained on an essentially
black primary window throughout 53.43 seconds of passive observation. The
previous launcher-mode message did not recur during this interval. No frontend,
new dialog, concrete log error, natural process exit, or missing-configuration
message was observed. This is an inconclusive startup observation, not proof
that initialization finished or that a specific failure was avoided.

The run ended through the observer's deadline stop, with zero UI input and no
remaining live children reported by its private PID-namespace supervisor.
The host's independent outer deadline also fired during final wrapper shutdown;
its REA result is explicitly a timeout, not an exit-zero retail result.

## Controlled inputs

Private run: `evidence/native-retail-reference/run-20261115T095946Z/`.
The [verified downloader-mode study](NATIVE_LAUNCHER_GATE.md) and existing owner's
normal standalone documentation motivated the separately authorized invocation:

```text
/game/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe -robocopy -nosteam -siteconfigurl=mh.bansheerealm.com:8443/SiteConfig.xml
```

The installed Wine loader executed this exact argument vector from
`/game/UnrealEngine3/Binaries/Win64`. The vector is recorded both at process
creation and in the owned native process command line. No Bifrost or existing
Wine prefix was used; no executable, Steam setting or launch check was patched.

The same original 453-file, 5,115,044,673-byte frozen game snapshot was mounted
read-only from `run-20261115T092318Z/game`, with no writable alias. Its earlier
partial-install limitations remain. A fresh independent clone of the clean
initialized prefix was verified against all 1,755 baseline regular files and
both symlink targets. Prefix-clone receipt SHA-256:
`acebbb010c9fc82b525baf8422b74a6803c0832205b5b4cc5d1f2c84c9b1f45a`.

The private hostname mapping was unchanged from the preceding probe:
`127.0.0.1 localhost devbox` and `::1 localhost`, SHA-256
`80b35dc4505c04b70c1aad49ee5bdf380a2fbaf0391f91fd3d99692cd6a831e0`.
Both preflight and runtime checks confirmed `devbox` resolved locally to
`127.0.0.1`, separate network/PID namespaces, only loopback and no routes,
hidden owner home, read-only game mount and fresh private X11 socket/auth.
The site's argument was retained exactly while external network egress remained
unavailable. No server, account, credential or host network change occurred.

`ClientConfig.xml` remained excluded. No concrete missing-ClientConfig branch
was observed, so the conditional original-file retrieval was not triggered.
No replacement configuration or further runtime arguments were invented.

## Observation and bounded cleanup

| Event | UTC |
| --- | --- |
| Retail process launched | 10:04:10.924 |
| Observer deadline stop acknowledged | 10:05:04.355 |
| Supervisor finished owned cleanup | 10:05:05.379 |

Nineteen periodic frames and a final stop capture retained the black primary
window. The final image was inspected visually. Its XWD pixel capture hash
matched the preceding known-black state; this is not a reference frontend image.
`Launch.log` remained zero bytes, and no minidump was generated. The known
window allowlist is structural, not complete prompt classification.

The observer stop request was written at elapsed 53.33 seconds to leave shutdown
margin before the independent outer timeout. The supervisor acknowledged it at
53.43 seconds and finished at 54.45 seconds, reporting Wine terminated by signal
(`-15`), Xvfb exit zero, no live children, and `native_input_sent = false`.
The final result's empty `visible_windows` field is the unpopulated new-window
list; the observer-stop event correctly records the still-visible primary
window. It must not be interpreted as proof that no window was present.

REA separately reports its 55-second outer timeout, SIGTERM finalization and
verified owned process group. Thus clean child shutdown is supported by the
inner receipt, but an orderly outer-wrapper exit is **not** claimed. Future
supervisor deadlines should reserve more setup/cleanup margin before the outer
limit. The additional inaccessible host PID records were checked: the surviving
entries were unrelated kernel workers, and one had exited. No unrelated process
was signaled.

Preparation recorded load averages near 180 with low measured I/O pressure;
hash and clone work ran at low priority. This is context, not a proven cause of
the black window. The bounded interval and partial snapshot do not distinguish
slow initialization, unresolved input dependencies, software-renderer delays,
or a later network prerequisite. No speculative success/failure attribution is
made from the black frames.

## Evidence, validation and provenance

REA Evidence:
`ev_c14c50f2f6ff95e91dc1b973d2a078cda2a1ee5aac993f68e0421985384ccdac`.
Bundle `receipts/rea-standalone-probe.bundle.json`, SHA-256
`aac3d93bd62855944885488806c06da9f737daf1c29633e0dfbd5bc99beb85cd`.
It retained all 26 terminal frames, sampled process records and requested
filesystem digests without truncation. Sampling can still miss brief processes,
and filesystem snapshots are not syscall traces. REA was exported and reset.

Final private image:
`captures/screen-retail-standalone-observer-stop.png`, SHA-256
`f1e4df0ca150728455917c057bb03f80ad3563b394216bf5ef191b611e020f59`.
Exact arguments, timings, controls, cleanup, generated log inventory and all
source-file hashes are retained in this run's `receipts/` and `logs/` directories.

**VERIFIED:** All 453 frozen files matched their pre-run hashes and lengths
after cleanup. The private hosts and supervisor hashes were unchanged.
`receipts/post-run-snapshot.json` SHA-256:
`a74f399f977ca87b2853795af50a9913fab8b15985ffe94c1443b798d9553a25`.
Final aggregate `receipts/finding-receipt.json` SHA-256:
`7c83a2c3b157cccb42c8cca8792bc6354135e00fa736381c4864046d5a0bb249`.

No new outside source was read. This reused the proprietary owner snapshot,
original project supervisor/hash scripts, installed runtime tools and mandatory
REA workflow already recorded in `NATIVE_REFERENCE_CAPTURE.md` and
`NATIVE_HOSTNAME_PROBE.md`. Their Wine, X.Org, bubblewrap and FFmpeg licence and
package-identity caveats remain applicable. The community launch documentation
and AGPL study provenance are recorded in `NATIVE_LAUNCHER_GATE.md`; no external
implementation was copied. All binaries, captures, logs and prefix files remain
private and excluded from source control/builds.
