# Marvel texture-cache manifest

`src/texture_cache.rs` is an original bounded metadata reader and resolver for
the observed 1.52-client `TextureFileCacheManifest.bin`. Its `payload.rs` child
reads explicitly selected cache files and decodes the observed external LZO
profile through the project's original UPK LZO decoder. It does not search for
cache files automatically, write extracted assets or claim retail rendering.

## Actual layout and reference discrepancy

**OBSERVED (private manifest bytes):** The little-endian file starts with an i32
entry count. Each record contains:

1. A UE3 FString qualified texture name.
2. Sixteen raw FGuid bytes.
3. A second FString containing the cache basename.
4. An i32 mip count.
5. For each mip: i32 serialized mip index, i32 stored offset and i32 stored size.

The cache filename is its stored basename plus `.tfc`. Mip indices are explicit;
they are not necessarily contiguous or equal to positions in a record's array.
The manifest supplies stored ranges, not decoded image dimensions, pixel formats,
compression codecs or decoded lengths. Those must come from the matching texture
and bulk-data formats.

**OBSERVED (reference source):** The studied UEViewer Marvel reader has only one
FString before the GUID, then immediately reads mips. It also selects the first
GUID match and the first mip. That older layout is insufficient for this file.
Our parser uses the additional cache-name field confirmed by consuming all
6,605,396 bytes of the owner's manifest; it never substitutes the old layout.

Positive FString lengths include a terminating narrow NUL; negative lengths
count UTF-16LE units including the terminator. All 95,538 observed strings in this
manifest are narrow ASCII. The reader supports validated UTF-16LE as a tested
format case, but rejects non-ASCII narrow strings until a code page is established.
Interior control characters, malformed termination and invalid UTF-16 fail.

## Lookup and limits

`TextureCacheManifest::open(path)` or `parse(bytes)` builds a private GUID index
without discarding duplicate records. `entries()`, `mip_count()` and
`unique_guid_count()` expose the parsed metadata.

`candidates(guid, mip_index)` returns every matching record. `resolve_unique`
returns one match or none; two or more matches are an error. `resolve_named`
additionally selects a qualified texture identifier with explicit ASCII-insensitive
matching and still rejects ambiguity. This matching policy is supported by the
observed lowercase cooked object paths versus mixed-case manifest names; it is
not claimed to establish all retail name-resolution behavior. GUID input is the
raw serialized 16-byte representation, not a network-order UUID conversion.

A `ResolvedMip` borrows its `CacheEntry` and `MipLocation`. `cache_file_name()`
returns a validated basename, preserving the manifest's case. Cache names cannot
contain path separators, drive delimiters, control or reserved Windows filename
characters, or end in a space/dot. The reader performs no directory traversal or
filesystem extraction. `MipLocation::checked_range(actual_cache_file_length)`
checks addition and the caller-supplied file boundary. It does not authenticate
that the supplied length belongs to the correct cache identity.

Default limits: 64 MiB file, 500,000 records, 4,096 encoded bytes per string,
64 mips per record, and 2,000,000 total mip records. Counts must also fit physical
input bytes before allocation; allocation failures are returned as errors.
Negative counts/indexes/offsets/sizes, zero stored sizes, duplicate mip indices
within one record, malformed/truncated records and trailing bytes fail. Duplicate
GUIDs across records are preserved because they occur in the actual client data.

## Private evidence

The copied manifest is at
`~/.local/share/marvel-heroes-forever-rust/pins/frontend-cache/UnrealEngine3/MarvelGame/CookedPCConsole/TextureFileCacheManifest.bin`.
Its **6,605,396 bytes** have SHA-256
`f20698f0dae941037bc697bab338f15e2e444647c03533f2c72c326477e6a903`,
matching the read-only Alienware source inventory. The pin is a selected local
manifest copy; it does not establish full distribution-copy completion.

**OBSERVED (independent original Python scan):**

- 47,769 records and 143,422 mip records; exact end-of-file consumption.
- 46,878 unique GUIDs. There are 891 extra records across 375 repeated-GUID groups.
- 3,283 repeated GUID/mip records across 1,315 groups. In 1,061 groups, even the
  cache basename does not remove ambiguity. Different stored ranges are present;
  equal sizes do not prove equal payloads.
- Qualified texture names are unique, including ASCII-insensitive comparison.
- 311 cache names, all ASCII letters/digits/underscores, lengths 5–70.
- All ranges fit the corresponding source-inventory cache sizes. No missing
  named cache, overlapping range, negative field, zero stored size or duplicate
  mip index within one entry was found.
- All 110 external CharacterSelect texture mip references resolve uniquely.
  Their cache names and qualified object paths match case-insensitively.

The complete independent receipt, original scanner and source list remain
private under `evidence/texture-cache-study/`. Receipt `manifest-scan.json` has
SHA-256 `ee4281d43c060da94be56ccb018822094d159fe3aa06696c7971b9cbf7d7b198`.
These are ordinary file/parser observations, not REA or runtime observations.

**VERIFIED (synthetic):** Five focused Rust tests cover sparse mip indices,
GUID collisions and name disambiguation, all byte-truncation positions, trailing
data, file/name/count budgets, malformed names and signed fields, exact file
bounds, UTF-16 decoding and malformed string encodings. The private transcript is
`evidence/texture-cache-tests.txt`.

**VERIFIED (Rust, source-pinned manifest):** All 47,769 records and 143,422 mip
records match an independent Python parse field for field. Every named lookup and
source-inventory-size range check passes. There are 138,824 unique lookup results
and 4,598 mip-record queries correctly rejected as ambiguous (counting every
record participating in a duplicate key). All 110 CharacterSelect cross-references
match the independent receipt.

The private Rust harness is `evidence/texture-cache-smoke.rs`; complete parsed
metadata is emitted only to `evidence/texture-cache-rust-records.tsv`. Summary and
comparison receipts are `evidence/texture-cache-rust-smoke.txt` and
`evidence/texture-cache-validation.json`. No actual TFC payload was read or
decoded by that initial manifest validation; the separate payload validation
below covers three selected ranges.

## Named cache reads and payload decoding

`TextureCacheFile::open(explicit_path)` opens a regular `.tfc` file read-only.
It records the supplied path's basename without constructing any path from
manifest strings. `read_mip(resolved, texture_cache_name, bulk_flags,
decoded_size)` requires agreement between the file basename, resolved manifest
cache name and Texture2D cache name, using ASCII-insensitive comparison. A wrong
cache name fails before reading. The caller must first resolve the texture's
GUID, serialized mip index and qualified name using the manifest APIs.

The range must fit the currently opened file's metadata length; the reader seeks
to that offset and reads exactly the bounded stored length. A concurrent
truncation fails the read. Filename and length agreement do **not** authenticate
a cache's contents: callers select their private verified pin, and pin hashing
remains a separate responsibility. There is no fallback to another cache, the
first GUID candidate, or a Texture2D's negative placeholder offset.

`decode_lzo_mip(stored_bytes, bulk_flags, decoded_size)` supports privately
pinned ranges or other already-located bytes. It does not establish their cache
identity. Both APIs have `_with_limits` variants accepting `PayloadLimits`:
64 MiB stored bytes, 128 MiB decoded bytes and 1,024 blocks by default. The
decoded length is the Texture2D mip's byte element count, independently checked
against its pixel format/dimensions by the texture decoder.

**OBSERVED (three owner cache ranges):** Bulk flags are exactly `0x11`; source
Texture2D stored offsets and sizes are `-1`, so the manifest supplies the actual
ranges. Each selected range contains a little-endian envelope:

1. u32 tag `0x9e2a83c1`.
2. i32 block size `131072` (128 KiB).
3. i32 total compressed payload bytes and i32 total decoded bytes.
4. `ceil(decoded_bytes / 131072)` pairs of i32 compressed/decoded block sizes.
5. Consecutive raw LZO1X version-0 streams, one per block.

The manifest's stored size includes the 16-byte header and 8-byte entries. The
header's compressed total excludes them. All blocks except the last decode to
128 KiB; the final block accounts for the exact remainder. Every table total,
decoded texture length and stored range length must agree before decompression.
The raw LZO decoder additionally rejects invalid references, overruns, truncated
streams and trailing bytes. Other bulk-flag combinations and block sizes return
`Unsupported`; this increment does not claim to cover every cache profile.

## Actual payload evidence

Three read-only source ranges were selected from `Icons.tfc` through the existing
`alienware-bazzite` SSH alias with strict host checking and batch mode. Remote and
local SHA-256 checks agree for each range, and source file size, modification time
and inode stayed stable during the read. Exact offsets, hashes, source metadata
and local filenames are in
`evidence/texture-cache-ranges/range-copy-receipt.json`, SHA-256
`d2ebc690926e4e6c1649cf34eeb60689026bc8a0cfce623e421deed6f6214954`.
The original private script is `pin_ranges.py` in that directory.

**VERIFIED (range transfer and independent decoding):**

| CharacterSelect export | Stored offset | Stored bytes | LZO blocks | Decoded bytes |
| --- | ---: | ---: | ---: | ---: |
| 3, full portrait | 6,289,438 | 130,164 | 5 | 641,200 |
| 4, small portrait | 18,897,908 | 13,111 | 1 | 24,320 |
| 5, power icon | 50,869,458 | 58,694 | 1 | 65,536 |

The independent original Python wrapper parser and separately installed
`liblzo2` 2.10 validation oracle consumed all seven blocks successfully. No
library code was copied or linked into the client. Receipt
`evidence/texture-cache-study/wrapper-scan.json` has SHA-256
`5723c0df6f053850661a8db974d742ad1c49140f1bdcc68c8e1d0b35ad9f1543`;
the original scanner and complete source ledger are `scan_wrappers.py` and
`wrapper-sources.json` alongside it.

The source inventory's historical whole-file record is 143,472,250 bytes with
SHA-256 `23c9b90cd88f99888cbc71332251daeac1e494866c27eab80c6cf78e6f2c7777`.
This range-copy operation did **not** recompute the whole-cache hash or establish
that a complete local cache copy exists. Only the three selected ranges are
currently transfer-verified by this evidence.

**VERIFIED (synthetic Rust checks):** Three added tests cover every truncation of
a small compressed envelope, trailing bytes, invalid tag/blocksize/totals and
negative fields, corrupt LZO termination, unsupported bulk flags, resource caps,
wrong cache names, exact range selection, out-of-bounds reads and short reads.
Together with the five manifest tests, all eight pass. The focused transcript is
`evidence/texture-cache-ranges/rust-tests.txt`.

**VERIFIED (Rust, three actual selected ranges):** A private harness parses each
actual Texture2D export, resolves its GUID/mip/qualified name against the actual
manifest, verifies cache name and range metadata, and decodes all 731,056 encoded
pixel bytes. Those bytes match the independent `liblzo2` outputs byte for byte.
Passing them to `texture::decode_pixels` succeeds for 700×916 BC3, 152×160 BC3 and
128×128 BGRA8 textures, producing 2,564,800, 97,280 and 65,536 RGBA bytes. The
RGBA files stay private; successful conversion is not a rendered-image or retail
parity check.

The harness and transcript are `evidence/texture-cache-ranges/rust-smoke.rs` and
`rust-smoke.txt`. `rust-validation.json` records output and implementation hashes;
its SHA-256 is
`39a052aaff26a53bbadea8c7964e0ea9fe1567f118742a11dbbf1f70557c07c7`.
The actual smoke uses the lower-level decoder on independently pinned ranges;
full-cache `TextureCacheFile` reads have only synthetic range/name coverage in
this increment.

## Source and licence register

The following outside sources were read for this increment. All module code is
newly written here; no reference implementation was copied.

| Source | Licence / identity | Used for |
| --- | --- | --- |
| [UEViewer `Unreal/UnrealMaterial/UnTexture3.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnTexture3.cpp), lines 566–640; private snapshot `evidence/upk-reference/Unreal__UnrealMaterial__UnTexture3.cpp` | MIT; commit `a0bfb468d42be831b126632fd8a0ae6b3614f981` | Reference manifest fields and GUID/mip relationship; actual extra FString and collision behavior were independently established from the owner's bytes. |
| [UEViewer `Unreal/UnCoreSerialize.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp), lines 359–436, 1036–1168 and 1531–1570; private matching snapshot | MIT, same commit | FString encoding, UE3 chunk framing and bulk codec selection. Actual block size/totals are independently checked, rather than copying the reference's permissive behavior. |
| [UEViewer `LICENSE.txt`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt), private matching snapshot | MIT, copyright 2022 Konstantin Nosov | Confirm licence text. |
| Owner's copied manifest and `evidence/alienware-distribution-inventory.json` | Proprietary Gazillion game data; owner-authorized private interoperability study | Identity, actual format, candidate collisions and source-cache bounds. No data included in git or builds. |
| Original project `evidence/upk-object-research.json` and its private logical package observations | Project-generated analysis of proprietary owner data; the report's reference-source ledger is retained privately | CharacterSelect texture GUID/mip/cache references and qualified object paths; no runtime assertion. |
| Owner's three exact `Icons.tfc` ranges; `evidence/texture-cache-ranges/range-copy-receipt.json` | Proprietary Gazillion game data; owner-authorized private interoperability study | Actual compressed wrapper grammar and sampled decoded pixel payloads. No game bytes enter git or builds. |
| System `liblzo2` 2.10, `/usr/lib/x86_64-linux-gnu/liblzo2.so.2.0.0`; licence `/usr/share/doc/liblzo2-2/copyright`; [upstream LZO](https://www.oberhumer.com/opensource/lzo/) | GPL-2.0-or-later; separately installed private validation oracle | Independent decoded-byte comparison only. No source copied, runtime dependency or linkage added to the Rust client. |

The implementation reuses the project's original `src/upk/lzo.rs` via
`upk::decompress_lzo`; its existing source/provenance record still applies. Pixel
conversion is delegated to `texture::decode_pixels`. Full-cache validation,
unobserved compression profiles and retail renderer/runtime behavior remain
outside this increment's verified scope.
