# Native downloader mode and the launcher message

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED, static native control flow:** The exact message observed in the
[hostname-corrected probe](NATIVE_HOSTNAME_PROBE.md) is produced when the
post-core settings downloader-mode selector is zero after client initialization.
The selected condition is a launch-mode configuration check. No activation,
account authentication, entitlement or DRM check appears in this bounded branch.
This does not establish the absence of such checks elsewhere in the client.
The runtime remains stopped; no alternate launch or dialog input was performed.

## Exact producer and selector

The literal message is UTF-16 at `0x1422d4ea0`. Function
`0x1414486a0..0x141448940` calls client initialization `0x141756270` at
`0x1414487a9` and continues only on a true result. It then checks the post-core
settings pointer at owner `+0x48`. When that pointer is nonnull and its DWORD
field `+0x3cc` equals zero, it reaches the message at `0x1414487ee`.
The adjacent diagnostic explicitly identifies an invalid set of client
downloader flags. The field's original C++ enum/type declaration was not recovered;
“downloader-mode selector” describes its verified use and diagnostic context.

Settings override function `0x1418efc60` calculates and overwrites `+0x3cc` from
five argument queries at `0x1418f03bf..0x1418f0465`. The observed literal names
and masks are:

| Query | Contribution |
| --- | --- |
| `nosteam` | disabled-integration mask bit `0x08` |
| `nosolidstate` | disabled-integration mask bit `0x04` |
| `steam` | selected-mode mask bit `0x08` |
| `solidstate` | selected-mode mask bit `0x04` |
| `robocopy` | selected-mode mask bit `0x02` |

Let `D` be the first two contributions ORed together and `S` the last three.
The native code stores `S` if `(D | S) == 0x0c`, `S` is positive and a power
of two, and `D != 0x0c`. Otherwise it stores `2` when `S == 2`, and zero
for other values. This is a transcription of the bounded arithmetic/branches,
not a runtime test of every argument combination. In particular, absent all
five flags yields `S = D = 0` and a zero selector. The standalone selection
named `robocopy` yields selector `2`; integration controls remain distinct
from the selected mode.

The query wrapper `0x1405b8620` delegates to `0x1405b8740`. The latter scans
entries at its input object's `+0x2008`, bounded by count `+0x2408`, strips
leading hyphens, handles an equals-sign value boundary, and returns a Boolean
match result. Its zero-count path returns false. The underlying string comparator's
complete case/encoding semantics and the upstream tokenizer were not investigated.
No complete command-line parser or argument-behavior parity is claimed.

The client's own post-core initialization sequence first calls its XML file
loader and then this override. It does **not** check the file loader's Boolean
return immediately before the override. Consequently, the captured dialog and
its zero selector do not by themselves establish that the excluded XML file was
read successfully or that a specific missing-file branch was observed. No
`ClientConfig.xml` contents were retrieved for this study.

## Message and requested exit

The producer calls `0x1407d2ef0` with message kind zero and the exact literal.
Its ordinary desktop branch formats the message and calls imported
`USER32!MessageBoxW` at `0x1407d3281`, using caption `Message` and flags
`0x1000`. The pointer at IAT slot `0x141dbc420` is identified from the original
PE import table. This matches the captured title and OK dialog. Other wrapper
branches exist for different global states; the bounded read does not identify
every UI mode.

After the message call returns, the producer calls `0x1407d3b70` with zero.
That branch calls imported `USER32!PostQuitMessage` via IAT `0x141dbc1e0` and
sets an exit-request flag. This is static continuation evidence only. The probe
never dismissed the dialog; its supervisor terminated the owned process while
the message was visible, so this continuation was not observed executing.

## Existing owner's normal launch documentation

**OBSERVED documentation, not a new execution:** The owner's Marvel integration
document pins the same nested Win64 executable and gives this standalone
community invocation:

```text
MarvelHeroesOmega.exe -robocopy -nosteam -siteconfigurl=mh.bansheerealm.com:8443/SiteConfig.xml
```

The same document prefers `Bifrost.exe` with those arguments on Windows 11 when
present. Its relevant source is
`/home/blizz/dev/banshee-marvel-pack/docs/MARVEL_HEROES_TABULA_RASA.md:14–17`,
SHA-256 `9912b849e324f479d1b6fbfa333692fc4e3139b4bed7c769a89f3bad7bcd6bed`.
The quoted invocation is the existing owner-community contract, not an invented
argument set or a verified original Gazillion launcher command.

MHServerEmu's `docs/Client/ClientLaunchParameters.md` at commit
`8fedb1a35922adfc9bff77e54f5fc3b6956073c9` names standalone, Gazillion-launcher
and Steam modes, with separate integration controls. That documentation is
consistent with the names and masks recovered above. Its description of
launcher-free community use remains a documentation claim; no launcher,
Steam client, network endpoint, login, or configuration change was tested.
The normal original Gazillion/Steam launcher invocation is still unresolved.

Existing owner launcher documents disagree about historical implementation
status, and the old launcher worktree's gitdir is stale. Exact file hashes
therefore identify the studied text; it is not proof of today's launcher tile
implementation. Any next native probe must be separately sequenced by the root
lead, preserving the offline namespace, private prefix, immutable snapshot,
no-input and prompt-stop constraints. This receipt authorizes no new execution.

## Evidence and focused checks

The owner executable remains SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
48,544,080 bytes, version 1.52.0.1700. Only target-free bounded REA disassembly
and original PE metadata/literal reads were performed. Raw disassembly, strings,
command documentation and captures remain private under
`evidence/native-launcher-gate/` and the earlier probe directory.

**VERIFIED:** Four REA calls completed with exit zero, unchanged target digests
and untruncated output. All 1,141 returned instruction records were compared
with the original PE bytes. The message helper's trailing embedded jump table
at `0x1407d32ac..0x1407d32c4` is data and excluded from instruction-level semantic
claims. The final bundle SHA-256 is
`d45654abbf252b0e5be209d2113e445021560ccf5fc392479dc8e41653b05985`;
`finding-receipt.json` SHA-256 is
`2e0f4cb0f4b182baec5492a59076eeadd796c62102f6ec9e1abfa45e3de5e1d3`.
REA was exported and reset.

Evidence IDs:
`ev_2a4b26edfabb71a5f03b1c6850326bd5d27250ea88811493d4041698a09219a0`,
`ev_ba5a103316a7ea942ff852e3828318a96d0ace9a21e48ea67cb54f699805aa08`,
`ev_12cf2fe639a04c414e109baf3434325df867f098d0fd1caf48e2a98ee9f4e1be`,
`ev_ebd9964eaf79f7559905a89c009b50307a492f3f61e4377d04ac4621374f7edc`.

`launch-doc-study/launch-path-facts.json` records the exact local source hashes,
repository pin, scope limits and documentation disagreements; SHA-256
`0e551a89e0bd1ca00d0f3a563d0b6d1122b1051364276105fb26d7ef8857c52a`.
The initial documentation search treated the requested `run` path as a directory
and returned unrelated Protocol Buffers/asset-name matches; those are retained
as provenance-only false positives. Subsequent reading was restricted to named
documents. No settings/account contents were returned or retained by that search;
unrelated example-account prose in a combined document was neither used nor
retained in the fact receipt.

## Source ledger

The proprietary owner binary was studied solely for compatibility and the exact
observed message. Original project evidence from `native-startup-settings/` and
`native-retail-reference/` supplied earlier call paths and the stopped observation.
Mandatory workflow came from
`~/.agents/skills/reverse-engineer-anything/SKILL.md` and its runtime guide;
no toolkit licence is assumed from availability and no toolkit code was copied.
No web or external implementation source was newly fetched. Local documentation
sources actually read or returned by the bounded search are listed below; full
hashes and exact line scopes are in the private ledger. AGPL material was study
only; no implementation source was copied.

| Source path | Licence / provenance | Use |
| --- | --- | --- |
| `/home/blizz/dev/mh-retail-accuracy-research.md` | Owner research; no explicit licence identified | Selected text and keyword search; identifies owner launcher worktree; no official invocation found |
| `/home/blizz/dev/marvel-heroes/research/STATUS-marvel-heroes.md` | Owner research; no explicit licence identified | Status identifies previous owner launcher login and Bifrost use; run15 launch references are server validation, not retail launch |
| `/home/blizz/dev/mh-omega-study/activity.txt` | Owner private research; no explicit licence identified | Negative launcher/Steam/Bifrost keyword search; no relevant result |
| `/home/blizz/dev/banshee-marvel-pack/README.md` | Private owner launcher documentation; no licence file found in scoped filename discovery | Keyword excerpts; stale implementation-status statements and pointer to Marvel integration document |
| `/home/blizz/dev/banshee-marvel-pack/docs/MARVEL_HEROES_TABULA_RASA.md` | Private owner launcher documentation; no explicit licence identified | Marvel-only facts retained; community version pin, executable selection and documented invocation |
| `/home/blizz/dev/banshee-marvel-pack/docs/LAUNCH_PROFILES.md` | Private owner launcher documentation; no explicit licence identified | Keyword excerpts only; managed-launch declaration caveat conflicts with channel status |
| `/home/blizz/dev/banshee-marvel-pack/docs/channels/README.md` | Private owner launcher documentation; no explicit licence identified | Keyword excerpt identifies Marvel Windows client through umu/Proton on Linux |
| `/home/blizz/dev/marvel-heroes/MHServerEmu/README.md` | AGPL-3.0 repository | Target 1.52.0.1700 full support and Steam-library distribution; does not supply original vendor launch contract |
| `/home/blizz/dev/marvel-heroes/MHServerEmu/LICENSE` | GNU AGPL version 3 licence text | Repository licence identification |
| `/home/blizz/dev/marvel-heroes/MHServerEmu/docs/Client/ClientLaunchParameters.md` | AGPL-3.0 repository | Community launcher-free parameters and named mutually exclusive launch modes; no implementation copied |
| `/home/blizz/dev/marvel-heroes/MHServerEmu/docs/Setup/InitialSetup.md` | AGPL-3.0 repository | Keyword excerpts; Steam distribution and community Setup tool path, no official vendor launch instruction |
| `/home/blizz/dev/marvel-heroes/MHServerEmu/docs/Setup/ManualSetup.md` | AGPL-3.0 repository | Keyword excerpts; community SiteConfig selection and launcher-free note |
| `/home/blizz/dev/marvel-heroes/run/Google.ProtocolBuffers.xml` | Third-party Protocol Buffers documentation; licence not investigated because irrelevant false positive | Initial run-directory search returned a protogen command-line false positive; not used for launch claims |
| `/home/blizz/dev/marvel-heroes/run/DitTransport/knowledge/marvel-heroes-atlas.json` | Owner generated game-data knowledge; proprietary source data, private study only | Initial run-directory search returned BifrostBridgeRegion asset-name false positives; not used for launch claims |
