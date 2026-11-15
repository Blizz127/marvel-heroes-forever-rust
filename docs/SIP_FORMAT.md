# SIP package reader

`src/sip.rs` is original, dependency-free Rust for read-only GPAK version 1
packages. It indexes metadata once, retains the source reader, and decompresses
requested members into memory. It never extracts files onto the host filesystem.

## Evidence and format

**OBSERVED:** The two available server-side data packages begin with the bytes
`4b 41 50 47` (`KAPG`), followed by little-endian version `1` and a signed 32-bit
entry count. They have the following identities:

| Package | File bytes | Members | Data section offset | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| `Calligraphy.sip` | 47,001,699 | 94,788 | 10,385,751 | `992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147` |
| `mu_cdata.sip` | 27,831,934 | 7,928 | 766,540 | `694f8af3f1950bac8b154e4e1ac354516f70bd3d895d6e9472eabd2df635d425` |

The inspected source files are at
`/home/blizz/dev/marvel-heroes/Data/Game/`. They were read without modification.
These observations alone do not identify a complete client installation, prove
their build number, or establish retail execution behavior. REA evidence is
tracked separately by the project lead; the measurements here are ordinary file
inspection and original-parser results, not REA results.

**OBSERVED (reference documentation and matching package metadata):** Each table
record contains a 64-bit hash, a signed 32-bit UTF-8 name byte count, the name,
then four signed 32-bit integers: modification time, offset, compressed length,
and decoded length. The offset is relative to the data section immediately after
the entire table. Integer fields are little-endian. Each payload is a raw LZ4
block; it is not an LZ4 frame or a size-prefixed LZ4 block.

Names are exact, case-sensitive identifiers. Stored hashes and timestamps are
exposed as metadata, without inferring their meaning or recomputing the hashes.
The reader does not depend on table/hash ordering, contiguous payloads, or the
last entry owning the last bytes. Every payload range is checked independently.

## API and limits

```rust,ignore
let mut package = SipArchive::open(path)?;
let entries: &[SipEntry] = package.entries();
let bytes: Vec<u8> = package.read("Calligraphy/Type.directory")?;
```

`read_index` uses the table's zero-based order. `entry` returns metadata for an
exact name. `data_offset` and `file_len` expose the validated file layout.
`from_reader` supports a complete `Read + Seek` source such as an in-memory
`Cursor`. The default file reader is buffered. Both entry points also have
`*_with_limits` variants.

Default budgets are one million entries, 4,096 bytes per name, 128 MiB of encoded
index metadata, and 256 MiB per compressed or decoded member. Negative sizes and
offsets, unsupported signatures/versions, invalid UTF-8, duplicate names,
truncation, out-of-file ranges, arithmetic overflow and allocation failures are
reported as I/O errors. Name validation rejects absolute/drive paths, backslashes,
empty path components, `.`/`..` components and control characters. There is no
filesystem extraction API.

Raw LZ4 decoding checks literal/input boundaries, extension lengths, nonzero
back-reference distances, overlap semantics, the output budget, and exact final
decoded length. It does not enforce canonical encoder layout or authenticate
content. The format contains no member checksum. Callers should use a private,
hash-pinned stable copy when reproducibility matters: retaining an open file does
not prevent another process from modifying its bytes.

## Validation

**VERIFIED:** Five focused tests pass: valid lookup and rejection of malformed
metadata/truncation/path names; duplicate names and configured budget rejection;
payload offsets independent of table order; LZ4 overlapping matches and extended
lengths; and malformed LZ4/output-size rejection. These tests use original
synthetic fixtures and prove those checked properties only.

Real-data validation is recorded privately in
`~/.local/share/marvel-heroes-forever-rust/evidence/sip-real-smoke-20261115.txt`.
The private harness at `evidence/sip-smoke/src/main.rs` reads both original
packages, decodes each member through this implementation, compares the complete
decoded bytes with `lz4_flex` 0.14.0, and writes only the five Calligraphy directory
samples under `evidence/calligraphy-samples/`. The comparison tool is a validation
dependency of that private harness only; it is not part of the client.

**VERIFIED:** The comparison passed for all 94,788 Calligraphy members
(60,411,527 decoded bytes) and all 7,928 resource members (114,346,226 decoded
bytes). This establishes matching decoded bytes for these two package identities,
not full client asset or gameplay parity.

## Study provenance

All implementation code in this module was written for this project. No reference
implementation code was copied. The complete outside-source reading list for
this work is:

| Source | Location and licence | Use |
| --- | --- | --- |
| MHServerEmu package documentation | `https://github.com/Crypto137/MHServerEmu`, local `/home/blizz/dev/marvel-heroes/MHServerEmu/docs/GameData/PakFile.md`; AGPL-3.0, local commit `8fedb1a35922adfc9bff77e54f5fc3b6956073c9` | Binary field layout, relative offsets, raw LZ4 designation. |
| MHServerEmu runtime package reader | Same repository, `src/MHServerEmu.Games/GameData/PakFile.cs`; AGPL-3.0 | Cross-check signature/version, exact-name behavior and metadata interpretation. |
| MHPakTool reader/writer and entry metadata | Same repository, `src/Tools/MHPakTool/PakFile.cs` and `PakEntry.cs`; AGPL-3.0 | Cross-check little-endian entry layout and raw block compression. |
| MHServerEmu licence | Same repository, `LICENSE`; AGPL-3.0 | Verify licence identity. |
| Owner's local package bytes | `/home/blizz/dev/marvel-heroes/Data/Game/{Calligraphy,mu_cdata}.sip`; proprietary Gazillion game data, owner-authorized private interoperability study | Metadata observations and decoder validation. No game data included in source control or builds. |
| `lz4_flex` 0.14.0 manifest/licence and validation binary | `https://github.com/pseitz/lz4_flex`, cached `/home/blizz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lz4_flex-0.14.0/{Cargo.toml,LICENSE}`; MIT | Manifest/licence inspection and independent decoder oracle in private offline harness. Implementation source was not used to write the module. |

**Unverified:** Other GPAK versions, SQLite-era packages, file-hash semantics,
retail loading order, corruption behavior of the original client, UE3 assets,
and gameplay/wire parity.
