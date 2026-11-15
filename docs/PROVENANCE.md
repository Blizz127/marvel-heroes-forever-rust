# Provenance and source register

Task `a3d58c579542`. Implementation is newly written Rust/Python for private
compatibility study. No retail binaries, assets, extracted data, captures or
reference implementation source are shipped in this repository or its builds.
A source being readable is not treated as permission to republish it.
No project-wide redistribution licence is selected in this initial private build.

## Format implementation and validation sources

The detailed ledgers linked here are part of this register and enumerate exact
files, licences, usage and limits for each component.

| Source | Licence / identity | Used for | Detailed file register |
| --- | --- | --- | --- |
| [MHServerEmu](https://github.com/Crypto137/MHServerEmu), read-only `/home/blizz/dev/marvel-heroes/MHServerEmu` | AGPL-3.0; observed HEAD `8fedb1a35922adfc9bff77e54f5fc3b6956073c9` | SIP fields, raw LZ4 designation, Calligraphy v11 metadata layout; source/docs studied, implementation written here | [SIP](SIP_FORMAT.md), [Calligraphy](CALLIGRAPHY_FORMAT.md) |
| Owner's server-data packages `/home/blizz/dev/marvel-heroes/Data/Game/{Calligraphy,mu_cdata}.sip` | Proprietary Gazillion data; owner-authorized private study, build unverified | Decoder comparison and metadata observations; separately hash-pinned | [Findings](../FINDINGS.md) |
| Owner's Alienware installation `/var/home/blizz/Games/banshee-library/games/marvel-heroes` | Proprietary Gazillion client and bundled dependencies; no redistribution permission inferred | Static build identity, private selected distribution pin, UPK profile and future behavior evidence | [Findings](../FINDINGS.md); private source inventory |
| [lz4_flex](https://github.com/pseitz/lz4_flex), cached 0.14.0 manifest/licence and compiled decoder | MIT | Independent byte-for-byte LZ4 validation in a private harness; not a client dependency; implementation source not used | [SIP](SIP_FORMAT.md) |

## Graphics and peer-project design references

| Source | Licence observed | Used for |
| --- | --- | --- |
| `/home/blizz/dev/benilla`: graphics status, root/app/world Cargo metadata, MIT/Apache licence files | `MIT OR Apache-2.0` | Module separation and requested/active graphics capability distinctions |
| `/home/blizz/dev/forever-client`: graphics status, enhanced-graphics, modern-ui, Cargo metadata and licence | MIT | Preset/apply behavior and optional UI boundary; no FFXI mechanics or implementation copied |
| `/home/blizz/dev/wotlk-forever/docs/{RUST_GRAPHICS_PORT_PLAN_20261109,FOREVER_GRAPHICS_REFERENCES}.md` | No blanket top-level licence found; source notes explicitly limit classic-code reuse | Shared settings versus host renderer boundary, matched-scene comparison criteria |

[Graphics plan](GRAPHICS_PLAN.md) contains the exact file-by-file ledger,
observed heads and evidence limits. The private reference hash manifest is
`evidence/options-reference-files-20261115.json` under the private project root.

## Discovery, preservation and tooling sources

| Source | Licence / status | Used for |
| --- | --- | --- |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/connection-and-recovery.md`, `references/native-and-artifacts.md`; installed `rea-agents` 6.9.0 CLI help and MCP schemas | Installed owner tooling; licence not inspected, no code copied | Required REA routing, connection recovery, evidence retention, CLI invocation |
| `/home/blizz/dev/dcuo-rust/FINDINGS.md`; names/structure under `/home/blizz/.local/share/dcuo-rust/` | Local owner-project documentation; licence not inspected | Private pins/evidence/build directory pattern and existing Alienware SSH route |
| Devbox Steam `steamapps/libraryfolders.vdf` and matching config path | Owner configuration metadata | Bounded search for installed client; no secrets or values copied into source |
| `/home/blizz/dev/mh-retail-accuracy-research.md` and targeted search results under `/home/blizz/dev/marvel-heroes/research/` | Local research; no separate licence inspected | Locate prior client/version and support notes; no implementation copied |
| `/home/blizz/dev/banshee-marvel-pack/docs/MARVEL_HEROES_TABULA_RASA.md`, `docs/channels/marvel-heroes.json`, targeted search results from `scripts/publish-marvel-heroes-channel.sh`, `docs/{ARCHITECTURE,ENB_INTEGRATION,SETTINGS,HALO_INTEGRATION}.md` and `docs/channels/README.md` | Local owner launcher project; licence not inspected | Client executable/install location and existing channel identification only; no downloads, publishing, launcher edits or launcher source copying |
| `/home/blizz/dev/mh-omega-study` filename discovery | Local owner research; licence not inspected | Look for existing UE3 study material; no implementation source found/copied |
| `/home/blizz/.local/share/devbox-coordinator/coordinator.py` report interface | Owner local operational tool; implementation not read | Authorized task status/handoff reporting only |

Search hits are discovery context, not proof of current runtime behavior. The
original-client executables were inspected as data, not executed. No server data,
accounts, credentials, access settings or other project sources were changed.

## Named references not yet studied

[MHDataParser](https://github.com/Crypto137/MHDataParser) is identified by the
owner as MIT but has not been read in this increment. OpenCalligraphy and
MHPatcher/Bifrost are named future study inputs; their licences have not yet been
verified and no implementation is taken from them. UE Viewer and subsequent
compression study are recorded in the UPK component ledger when integrated.

## Integrated UE3/LZO reader

[UPK format ledger](UPK_FORMAT.md) enumerates exact primary-source paths and
versions. [UEViewer](https://github.com/gildor2/UEViewer) at commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981` is MIT and was studied for UE3 summary,
name/import/export and compression-envelope fields. The Linux kernel's
[LZO format documentation](https://docs.kernel.org/staging/lzo.html) (GPL-2.0
prose) was studied for the original LZO1X decoder's grammar; no codec code was
copied. Installed `liblzo2.so.2` (GPL-2.0-or-later) was used only in a private
independent validation harness and is neither linked nor distributed. Its
copyright notice was read to verify the licence. The five proprietary sample
packages remain private. [SIP variant findings](SIP_VARIANTS.md) record the two
owner-provided package variants and attributed read-only backup hash observation.
