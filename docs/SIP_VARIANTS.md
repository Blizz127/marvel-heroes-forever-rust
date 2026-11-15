# Calligraphy package variants

Investigation date: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED:** The server-data and Alienware candidate packages differ in actual
decoded content. Their complete header and entry table, including every recorded
timestamp, are identical. Exactly one of 94,788 members differs:
`Calligraphy/Missions/Prototypes/BonusMissions/WeeklyEventMissions/CarnivalMissionDay1.prototype`.
This finding does not identify who changed either package, when it changed, or
which copy represents untouched retail content.

## Compared identities

Both complete files contain 47,001,699 bytes, GPAK version 1, and a data-section
offset of 10,385,751.

| Source | Local path | Whole-file SHA-256 |
| --- | --- | --- |
| Server data | `/home/blizz/dev/marvel-heroes/Data/Game/Calligraphy.sip` | `992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147` |
| Alienware candidate private copy | `~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/Data/Game/Calligraphy.sip` | `1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c` |

The candidate's final filename was present before inspection; no rsync temporary
file or remote source was read by this investigation. Both computed hashes match
the identities supplied for comparison. These local hashes do not independently
establish build number or publisher authenticity.

**OBSERVED (project lead's read-only remote hash report):** The Alienware backup
`Data/_mhclientpatch-backup/Calligraphy.sip.20261009T031312Z.orig` also contains
47,001,699 bytes and hashes to
`992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147`, matching the
server-data package exactly. The lead retained this observation in
`~/.local/share/marvel-heroes-forever-rust/evidence/alienware-calligraphy-backup-hash.json`.
The backup was neither restored nor copied by that check. Its directory and
filename describe a backup relationship, but do not establish untouched retail
provenance, a particular patch author, or modification chronology.

## Difference boundaries

**VERIFIED:** All entry names, hashes, timestamps, payload offsets, compressed
sizes and decoded sizes match. Every other member's compressed payload is
byte-identical between the files. A complete aligned byte comparison finds only
seven changed bytes in the whole archive, all inside member index 13,574:

| Absolute archive offset | Changed bytes |
| ---: | ---: |
| 14,669,367 | 1 |
| 14,669,369 | 6 |

The changed member occupies 845 compressed bytes and declares 1,586 decoded bytes
in both files. The original Rust SIP reader successfully decompresses both to
that exact length. Decoded comparison finds 21 changed bytes in six runs:

| Offset within decoded member | Changed bytes |
| ---: | ---: |
| 52 | 1 |
| 54 | 6 |
| 137 | 1 |
| 139 | 6 |
| 188 | 1 |
| 190 | 6 |

The same seven-byte difference repeats three times in the decoded output. Exact
byte values and both decoded payloads remain private; none are stored in this
repository. Decoded identities are:

| Source | Decoded member SHA-256 |
| --- | --- |
| Server data | `ae921a446eeb468a93f2083a63dcb13f8b8ef2f93b7e12d4458664989c7d8378` |
| Alienware candidate | `70d56dccb4d8f4b2f9018090461b41045934d5f1a48ccf7c8488d51d42ea9d34` |

**INFERRED:** The difference is a localized content substitution in one mission
prototype. The field types and gameplay consequences have not been decoded or
tested, and no author, patch tool, intent, modification time, or direction of
change can be assigned from these bytes alone.

## Private evidence and validation

Evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/sip-variants/`.

- `metadata-comparison.json`: both file identities, complete table comparison
  result and identity of the single changed compressed member.
- `archive-byte-differences.json`: exact absolute offsets and byte values for
  both sides of every archive difference.
- `decoded-byte-differences.json`: exact decoded offsets, byte values and hashes.
- `13574-server-decoded.bin` and `13574-alienware-decoded.bin`: private complete
  decoded members.
- `decode-variant.rs`, `decode-build.txt`, `decode-result.txt`: focused Rust
  harness, compiler output and successful decoded-length comparison. The harness
  uses this project's `src/sip.rs`, asserts identical metadata for the selected
  entry, and reads only the changed member from each package.
- `comparison-receipt.json`: timestamp, source-module identity and evidence-file
  hashes.

**VERIFIED:** Whole-file hash checks, all 94,788 metadata comparisons, all 94,788
compressed payload comparisons, complete aligned raw-byte comparison, and the
two focused member decodes completed successfully. This is static compatibility
evidence from ordinary file tools and the original Rust reader. No REA result,
retail execution, server connection or gameplay-parity result is claimed here.

## Provenance

The only new external file inputs read by this helper are the two owner-provided
package files named above; the remote-backup observation is attributed to the
project lead's separate read-only check and private evidence receipt. These are
proprietary Gazillion game data, used for owner-authorized private
interoperability study. The decoder is this project's
original `src/sip.rs` from commit `c9e30ab466964ade59b1cbcf37a1fa755b507e22`;
its earlier study sources and their licences are recorded in `SIP_FORMAT.md`.
No additional reference implementation was consulted and no game payload was
added to source control.
