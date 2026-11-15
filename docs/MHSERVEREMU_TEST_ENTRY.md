# MHServerEmu test-entry prerequisites

Task `a3d58c579542`, read-only investigation on 2026-11-15 UTC.

**OBSERVED retained evidence:** The normal launcher control route is the shared
authenticated stackd API with game ID `marvel-heroes`. The later DIT handback
identifies a retained Nov14 r63-successor binding; Marvel remains `native_hold`
without health, human-count or lifecycle command pins. A usable current native
Banshee contract is **not established**. The original r62 investigation below
is historical and is supplemented by the later handback section. No wake,
status request, server command, credential-store read, Account.db access or
native client launch was performed for either read-only reconciliation.

The owner's standing permission for normal wake does not turn an unresolved
held binding into permission to start individual containers or activate a
different controller release. Root retained that boundary explicitly during
this investigation. No alternate Docker/systemd start is proposed.

## Current source contract, not current service state

Inspected launcher checkout:
`/home/blizz/dev/banshee-launcher-mh-forever`, clean relevant tracked files at
commit `869e9dbcc5ef83cff43f2aec65bdd069f83c062c`.
`OnDemandClient.cs` supplies these routes:

| Purpose | Method and endpoint | Prerequisite / interpretation |
| --- | --- | --- |
| Normal wake | `POST https://login.bansheerealm.com:8450/v1/games/marvel-heroes/wake` | Existing authorized Banshee session; JSON `{}`. This is the launcher client's `WakeAsync`, not a direct container start. |
| Game status | `GET https://login.bansheerealm.com:8450/v1/games/marvel-heroes/status` | Same authenticated session; does not itself establish readiness. |
| Binding inventory | `GET https://login.bansheerealm.com:8450/v1/games` | Same session; retain actual mode/held state rather than assuming a managed row. |
| Player counts | `GET https://login.bansheerealm.com:8450/v1/status/players` | Same session; human/bot split is admin-only. Missing or refused data stays unknown. |

The source passes an existing session as a Bearer header, uses a ten-second
request bound and limits wake/status bodies to 64 KiB. No token value was read
or copied, and no unauthenticated workaround was identified. The source maps
the retail Marvel provider to `marvel-heroes`; this does not establish that a
particular installed Forever tile already has the same binding.

Wake responses distinguish ready, starting, forbidden, signed-out, rate-limited,
failed and unreachable states. A response naming another game cannot establish
Marvel readiness. The later scoped DIT source review confirms that mapped
Marvel requires explicit `ready` after normal authenticated wake/status.
The generic 404/not-on-demand continuation previously noted here is not a
Marvel readiness exemption. Held/error responses are neither asleep nor ready.
An HTTP error, unreachable service or historical `asleep` note is not permission
to restart an already-running game.

The complete PLAY flow also contains provisioning/login behavior outside this
prelogin study. It must not be invoked merely as a convenient wake test. The
documented request above identifies the normal control operation, not authority
to extract credentials, accept sign-in prompts or run the rest of PLAY.

## Exact held-work evidence

The coordinator ledger was opened with SQLite URI `mode=ro`. Only the two
specified task rows and their three latest events were queried; no task was
updated and no game/account database was opened.

| Task | Exact saved state read on 2026-11-15 | Meaning and limit |
| --- | --- | --- |
| `5d1c59545860` | `blocked`; last task update 2026-11-11 06:09:49 UTC, latest event 46058 | Queue cleanup labels the previous report stale. Its substantive report is Nov10: r62 staged, not active; MH/WAR held. No newer activation/native-contract receipt occurs in these exact-task results. |
| `af7322ea1487` | `queued`; task update 2026-11-01 04:56:26 UTC, latest event 43293 | Existing Marvel lifecycle assignment recorded, no worker resumed; latest event is a watcher alert, not native bring-up evidence. |

The substantive r62 producer report says Marvel had no working Banshee stack
and no verified start/stop/humans contract. It reports the game container exited
on Oct29 and the launcher/edge-forward containers also exited. These are
**historical producer observations**, not current process/container checks by
this lane. The report calls for the native owner to establish the actual
working stack and graceful-stop contract before adding its managed sleep row.
No new deployment or owner assignment follows merely from the saved report.

The corresponding local source mirror exists at:

`/home/blizz/dev/dit-ffxi-bridge-integration-20261029/tools/stackd_migration/live_r62/`

Checkout commit `beb684c9c3588f656dfb01f12958e8e43d7f6e77` contains
`bindings.stage.json`, whose Marvel row is `mode: held`,
`held_reason: native_hold`, with retained native-activation-hold evidence.
Its README explicitly says staged/not activated. The binding has no Marvel
start/stop/humans commands to reuse. No activation script was executed.

The Nov14 coordinator review independently retains that same unresolved
lifecycle prerequisite. Older Nov10/11 notes also mention a discretionary
development deferral, but the Nov14 review reconciles the Nov7 general resume:
the old policy pause alone is not the present blocker. The concrete unresolved
native binding and player-protection contract remain distinct.

`~/dev/marvel-heroes/run` is a directory containing a local MHServerEmu build,
not a wake script. Only its directory/type and filenames were inspected; no
binary, configuration, database, boot log or live transport file was opened.
`research/STATUS-marvel-heroes.md` is dated Oct10 and describes Oct9 deployments
and a then-asleep server. It cannot establish present availability or justify
launching that local build as a replacement service.

## SiteConfig port correction and public read-only checks

The current local launcher source differs from its older prose document:

| Source | Stored contract |
| --- | --- |
| `MarvelHeroesLaunchPlan.cs:43–57,106–129` | Native SiteConfig is plain HTTP, default port 80. The builder emits `-siteconfigurl=mh.bansheerealm.com/SiteConfig.xml`, without scheme or `:80`. HTTPS 8443 is retained for auth/account APIs and the public status probe. |
| `docs/MARVEL_HEROES_TABULA_RASA.md` | Still shows the older `:8443/SiteConfig.xml` native argument and describes that SiteConfig URL as HTTPS. This conflicts with the source builder. |

The HTTP builder was introduced by commit
`8f40ed7893ad422b683c5c12e9eaf9f64b30d60a` on 2026-09-10. Its comment records a
prior failed `host:8443` behavior; that comment is a historical developer report,
not a native-parser or current endpoint verification performed here.
The earlier isolated reference runs deliberately used the older documented
`:8443` argument. This finding corrects their **argument provenance**, without
changing their immutable receipts or attributing their deadline to that choice.

Concrete public candidate URLs supported by current local source are:

- `http://mh.bansheerealm.com/SiteConfig.xml` — native SiteConfig form.
- `https://mh.bansheerealm.com:8443/ServerStatus` — public count/status JSON.
- `https://mh.bansheerealm.com:8443/SiteConfig.xml` — historical documented URL;
  not the current source's native startup argument.

For a future authorized read-only check, the following bounded commands are
reviewable examples. `receipt_dir` must already designate a new private evidence
directory. Neither command was run by this lane. They send no authentication,
do not follow redirects and do not disable TLS verification:

```sh
curl --silent --show-error --connect-timeout 5 --max-time 10 \
  --max-filesize 65536 --proto '=http' \
  --dump-header "$receipt_dir/siteconfig.headers" \
  --output "$receipt_dir/siteconfig.xml" \
  --write-out '%{http_code} %{size_download} %{content_type}\n' \
  'http://mh.bansheerealm.com/SiteConfig.xml'

curl --silent --show-error --connect-timeout 5 --max-time 10 \
  --max-filesize 65536 --proto '=https' \
  --dump-header "$receipt_dir/server-status.headers" \
  --output "$receipt_dir/server-status.json" \
  --write-out '%{http_code} %{size_download} %{content_type}\n' \
  'https://mh.bansheerealm.com:8443/ServerStatus'
```

Keep bodies private; report status, size, type and SHA-256. A returned SiteConfig
file proves retrieval of those bytes, not current auth/world readiness. A
redirect or error remains evidence to inspect, not a reason to change endpoints
or add credentials. No native login/account request belongs in this check.

The launcher status parser requires `PlayerManagerPlayers`; it optionally reads
`DitBotHeroes`, `StartupTime` and `CurrentTime`. Its displayed humans count is
`max(0, players - bots)` when bots is present, otherwise it conservatively uses
all reported players. A missing/invalid player counter fails parsing. Bot-only
totals, unavailable counters or a timestamp-free response do not supply fresh
zero-human proof for a restart. No current human count is asserted here.

## Historical operator safeguards and their limits

The Sept28 launcher contract and the older AX41 source at
`~/dev/dit-wow-stackd` commit
`1053f0c46f526e7fc8b883f66ccfd351cb87a674` describe idempotent wake, one lock per
game, a short wake lease, separate launcher heartbeats and status reads that do
not renew a lease. The local operator interface was `stackd.py ctl status
marvel-heroes` / `ctl wake marvel-heroes`, through a private `ctl.sock` under
`STACKD_STATE_DIR` or `~/.local/state/stackd`. These are **historical AX41
interfaces**, not an established current Banshee entrypoint; no such command or
socket was invoked or substituted for the held route.

That old source starts the existing container and checks the container's own
game/web listeners, rather than assuming a host Docker proxy means ready. Its
human probe counts established frontend peers, while in-process bots do not
hold those sockets. Probe failure yields unknown, and stop logic rechecks under
the lifecycle lock and refuses an unknown or positive human count. Its optional
last-human mode changes idle timing, so the original contract's 25/30-minute
defaults must not be assumed to describe the deployment. None of these old
source safeguards establishes the missing Banshee native contract.

Before any eventual normal wake, root needs a current verified binding/hold
disposition from the existing service owner or an actual current read-only
controller result. If it remains held, preserve it. If already ready, use that
state without a restart. An unknown player count never becomes zero. This
investigation supplies no permission or plan to stop, restart, recreate,
deploy, migrate or alter accounts/player data.

## Evidence and provenance

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/mhserveremu-test-entry/`.

| Receipt | SHA-256 |
| --- | --- |
| `exact-task-receipt.json` | `0d20f4429f39acd7ea0926378916cf84d953db95423c80d110db6a3650b2ba3d` |
| `selected-source-excerpts.json` | `02269e2327ebb2ea9f9e0820daa3da957c296fb4451b6ecc07a7f713b9874b74` |
| `source-ledger.json` | `5f83d96c7bb005ddf2f656e8e3c3b9bf1dda619ad5ab0fe8e0ea2c4dbe9da920` |

The ledger lists all 28 inspected source/document files with paths, hashes,
repository commits where available, usage and licence limits. These are the
owner's local launcher/DIT source and operational documentation; no top-level
open-source licence was established for that code, and no implementation was
copied. The source scope includes the launcher README/integration/audit docs,
`OnDemandClient`, `OnDemandWakeFlow`, `MarvelHeroesProvider`,
`MarvelHeroesLaunchPlan` and `MarvelHeroesServerStatus`; the old DIT
`stackd.py`/`probes.py`; the r62 README, binding and narrow searched source
files; the named Marvel status; the Sept28 contract; and the dated coordinator
notes/Nov14 review. Exact task-ledger reads are separately preserved with query
time and status/event timestamps. No remote service, installed launcher build
or live controller identity was verified, and no source repository was edited.

## Root public route observation

**OBSERVED 2026-11-15 11:44:25 UTC:** root made one bounded unauthenticated GET
to `http://mh.bansheerealm.com/SiteConfig.xml`, the route selected by the current
local launcher source. Curl completed with HTTP **502**, `text/html`, 150 bytes.
This does not identify the backend process or establish whether the game server
is asleep, stopped, held or serving players. No redirects were followed and no
wake, login or native-client request was made.

The private receipt, response headers and body are under
`evidence/siteconfig-route-check-20261115T1145Z/`. The response body SHA-256 is
`61b30d408583991fd69f3dec694e154cb652471e663328ad9c8482c9021ab5db`.
The concrete route failure and unresolved hold were queued to the coordinator;
the report does not establish acknowledgement or release of that hold.

## Later scoped DIT handback: retained r63 successor

**VERIFIED document identity, 2026-11-15:** the supplied
`/home/blizz/scratch/dit-marvel-retained-contract-20261115/HANDBACK.txt` matches
SHA-256 `008746dd02eea46c6e657543ba4d89d53b7e4f90ef96cff4968e0ab82ecd60d8`.
Its exact private copy and root read-only receipt are retained under
`evidence/dit-contract-handback-20261115/`. The facts below are attributed to
that scoped DIT review, not new live observations by this client lane.

The newer retained release is
`release-20261114-forever-oplock-a71c-candidate`, still using unit name
`dit-stackd-r60.service`. The handback pins `bindings.stage.json` to SHA-256
`7887fecae5169e5249a2eb4061677e485c7208a39d0cee93d8f85b1d37aadb9b`
and `banshee_service.py` to
`e32231cb0371cbdbe3f6ea2462ef195fb655881a853a0750de8f9f83b04673eb`.
Its saved Nov14 paired-cutover and boundary-check receipts concern controller
identity. They do not establish Marvel readiness, stopped state, completed save
or zero humans. This lane has not independently re-read those underlying files
or queried their current runtime.

The Marvel row remains `mode=held`, `held_reason=native_hold`, identical to the
retained r62/r63 rows, without health, human, start, stop, wake or sleep pins.
The endpoint hint is metadata, not a probe. Source-derived controller behavior
skips probes for held rows, keeps counts unknown, reports held status and refuses
sleep. No endpoint was called to reconfirm those source-derived results.

The owner's Nov9 **sleep** choice is settled. `native_hold` records the missing
native lifecycle implementation; it does not reopen sleep versus off. Existing
native task `af7322ea1487` remains queued with its current lead unverified. Its
missing deliverable is a current Banshee contract with exact service/artifact/
command pins, reliable health and fresh human/in-flight counts, graceful
drain/save and recoverable stop, normal demand wake, and observed
progress-preserving stop/wake/reconnect evidence. Historical AX41 acceptance
and the saved HTTP80 502 do not supply that contract.

No alternate route, access change, hold release, controller activation or worker
redispatch follows from this handback. All live/access and FFXI no-restart holds
remain in force. Independent Rust work continues without an acknowledgement
gate. Publication bookkeeping in the handback concerns an earlier intake;
dev15's separate staged artifact and queued intake remain as recorded in
`FINDINGS.md`.
