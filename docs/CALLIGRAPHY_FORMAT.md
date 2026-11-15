# Calligraphy metadata reader

`src/calligraphy.rs` contains original, dependency-free Rust readers for the
version 11 metadata used by the selected client generation. Parsing operates on
byte slices supplied by the package loader; this module does not write files.

## Implemented scope

| API | Data decoded |
| --- | --- |
| `Header::parse` | Four-byte signature and export version; does not parse the body. |
| `Directory::parse` | Curve, type, blueprint, prototype and replacement directory records. |
| `Curve::parse` | Signed start/end positions and the corresponding little-endian doubles. |
| `AssetType::parse` | Asset IDs, GUIDs, flags and names. |
| `Blueprint::parse` | Runtime binding name, default prototype ID, parents, contributors and member schemas. |
| `PrototypeDataHeader::parse` | Flags and optional reference ID, returning the number of bytes consumed. |

Directory, curve, asset type and blueprint readers require complete consumption
of their input. Prototype field groups, prototype inheritance, runtime class
bindings, semantic ID resolution and replacement-chain behavior are not
implemented by this module. A successful metadata parse does not establish any
of those behaviors or retail gameplay parity.

**OBSERVED (reference source):** The four-byte prefix consists of three signature
bytes followed by export version 11. The source directory table expects `CDR`,
`TDR`, `BDR`, `PDR` and `RDR`, respectively. The reference documentation calls the
type-directory signature `CDR`; the implementation uses `TDR` to match the source
table, with private package validation recorded below.

**INFERRED (reader design):** Record counts must fit the remaining bytes using
the smallest valid record size before allocation. Every scalar, string and
optional field is range checked. Strings are strict UTF-8; NULs and unsafe
archive-relative paths are rejected. Paths normalize `\\` to `/`. Unknown
versions, flag bits and type tags fail explicitly. The parser preserves physical
record ordering and does not silently deduplicate IDs or resolve references.
These defensive policies are our design, not observations of retail error
handling. Sixteen-bit lengths/counts are decoded as unsigned format values;
the reference documentation uses unsigned fields while the source reader uses
signed `short` for some of them. Behavior above the signed range is not established
by the reference source.

## Validation

**VERIFIED (synthetic, 2026-11-15):** Eight focused module tests pass with
`rustc --edition 2024 --test src/calligraphy.rs`. They cover complete directory
decoding, all truncation boundaries of a representative record, impossible
counts, trailing data, invalid UTF-8, unsafe paths, unknown flags/versions,
prototype/replacement record layouts, extreme curve ranges, blueprint subtype
alignment, asset names and optional prototype reference headers. These tests
validate our implementation, not wire or runtime compatibility.

**OBSERVED (private package, 2026-11-15):** The original Rust SIP loader and these
readers successfully decoded every directory, blueprint, curve and type file in
`/home/blizz/dev/marvel-heroes/Data/Game/Calligraphy.sip`: **9,602 metadata files,
zero failures**. This confirms `TDR` in the actual type directory and the layouts
supported by these readers for this available package.

| Metadata file kind | Files parsed | Directory records |
| --- | ---: | ---: |
| Directory | 5 | 103,035 total |
| Blueprint | 5,659 | 5,659 blueprint references |
| Curve | 3,680 | 3,680 curve references |
| Asset type | 258 | 258 type references |
| Prototype directory | Included above | 85,186 prototype references |
| Replacement directory | Included above | 8,252 GUID replacements |

The private harness and summary are at
`~/.local/share/marvel-heroes-forever-rust/evidence/calligraphy-metadata-smoke.rs`
and `calligraphy-metadata-smoke.txt`; synthetic test output is in
`calligraphy-tests.txt` alongside them. This is ordinary local parser evidence,
not REA output. The package's attribution to client build 1.52.0.1700 requires the
separate executable/pin evidence maintained by the project lead. No retail client
has been launched as part of this reader's validation.

SHA-256 of the tested package:
`992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147`.
SHA-256 of the metadata summary:
`ffb066718462e289b485f74ee9cc462098b3b83dd2211a9c6ec9f1cf4f28ce32`.

## Sources studied and licence provenance

All MHServerEmu references below were read from
`/home/blizz/dev/marvel-heroes/MHServerEmu` at commit
`8fedb1a35922adfc9bff77e54f5fc3b6956073c9`. Project:
<https://github.com/Crypto137/MHServerEmu>, licence **AGPL-3.0** (local `LICENSE`
inspected). They were studied for data layout facts; no source implementation was
copied or translated into this module.

| Relative source path | Purpose |
| --- | --- |
| `docs/GameData/Calligraphy.md` | Signatures, field layouts, base/structure tags and file relationships. |
| `docs/GameData/PakFile.md` | Package relationship and how metadata is carried inside SIP archives. |
| `src/MHServerEmu.Games/GameData/DataDirectory.cs` | Directory signature table and directory record field order. |
| `src/MHServerEmu.Games/GameData/Calligraphy/CalligraphyReader.cs` | Export version checks, string/path handling, prototype data-header flags. |
| `src/MHServerEmu.Games/GameData/Calligraphy/Blueprint.cs` | Optional subtype presence, blueprint reference ordering and member metadata. |
| `src/MHServerEmu.Games/GameData/Calligraphy/AssetType.cs` | Asset value layout and distinction between asset names and file paths. |
| `LICENSE` | Licence identification. |

The owner's `Data/Game/Calligraphy.sip` is proprietary Gazillion game data,
studied read-only for interoperability. It and all extracted samples remain
private under the owner's local storage; none is included in git or releases.
