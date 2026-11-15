# Marvel texture-cache manifest

`src/texture_cache.rs` is an original bounded metadata reader and resolver for
the observed 1.52-client `TextureFileCacheManifest.bin`. Its `payload.rs` child
reads explicitly selected cache files and decodes the observed external LZO
profile through the project's original UPK LZO decoder. The `binding.rs` child
connects a saved Texture2D mip to that reader explicitly. It does not search for
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
that a complete local cache copy exists. Only the three selected ranges were
transfer-verified by that initial evidence.

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
full-cache `TextureCacheFile` reads had only synthetic range/name coverage in
that initial increment. The later complete-file check below closes that gap.

## Complete cache files and explicit Texture2D binding

**VERIFIED (later local full-file hashing):** these existing files under
`pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/` now match the
complete saved source inventory. File device/inode, size, modification time and
change time remained identical before opening, throughout hashing and after the
actual payload smoke. No rsync destination was modified and no huge file was
copied again.

| Existing cache | Bytes hashed | Complete SHA-256 |
| --- | ---: | --- |
| Icons.tfc | 143,472,250 | `23c9b90cd88f99888cbc71332251daeac1e494866c27eab80c6cf78e6f2c7777` |
| Lighting.tfc | 131,317,986 | `3a9a5a560cb02cb1a0497043d2b6559e70cc9647c2fc98b9b83bcf8aef8e93e8` |
| CharTextures.tfc | 1,373,627,963 | `40742a138af46b6dcc6b2bf8e802c6b849247cfccda6213054f54b6854b204e5` |

This establishes these three local file identities at the time of the checks,
not completion of the full distribution transfer. Only Icons payloads were
decoded in this increment. The complete receipt is
`evidence/texture-cache-binding/full-cache-hash-receipt.json`, SHA-256
`e10c4c0171104b0a671ddc6047b1d1b9e5a7c7e12e8a2834eeeb336a3fc0cd89`.

The original new bridge is reexported from `texture_cache`:

```rust,ignore
let texture = Texture2D::from_export(package, export_index, &export_bytes)?;
let qualified = qualified_texture_name(package, export_index)?;
let bound = BoundExternalMip::resolve(&texture, mip_index, &manifest, &qualified)?;
let rgba = bound.decode_rgba(&mut explicitly_selected_cache_file)?;
```

`qualified_texture_name` follows only the supplied package's saved outer
references. It never substitutes the `.upk` basename. For these actual exports,
the outer path starts with `marveluiicons_characterselect`, while the containing
filename is `ICO__MarvelUIIcons_CharacterSelect_SF.upk`. Flattening uses periods;
a period inside a single object-name component is rejected to avoid losing
boundaries. Missing names, invalid signed references, outer cycles, more than
64 object-path components, control characters and names over 4,096 bytes fail.

`BoundExternalMip::resolve` requires:

- The explicitly selected primary mip exists, is External, and has flags exactly
  `0x11`. There is no inline, absent, or separate-payload fallback.
- Its dimensions and pixel format give exactly the saved element count, and its
  dimensions fit the texture descriptor.
- A nonempty saved cache name agrees with the manifest record selected by exact
  raw GUID, qualified object name and the same explicit mip index. Name matching
  uses the existing ASCII-insensitive policy; ambiguous named matches fail.
- The external offset/size pair is the observed `(-1, -1)` placeholder pair, or
  an explicit nonnegative offset/positive size pair exactly agreeing with the
  manifest. Explicit positive pairs have synthetic validation only; all 110
  actual CharacterSelect mips use `(-1, -1)`. Mixed/other negative placeholders
  and conflicting explicit ranges fail.

`read_encoded` returns the selected decompressed mip bytes; `decode_rgba`
additionally calls the existing pixel decoder. Both have `_with_limits` forms
accepting `PayloadLimits`. The supplied `TextureCacheFile` still checks its own
basename and actual file bounds. `resolved()`, `mip_index()` and `mip()` expose
binding metadata without enabling reassignment. No file search, first-candidate
selection, lower-mip fallback, mip-tail remapping, gamma conversion or alpha
premultiplication is added. Pin authenticity remains the caller's responsibility.

**OBSERVED (fresh independent descriptor/manifest scan):** all 110 CharacterSelect
textures have one primary mip, index 0, and one exact GUID/name/mip binding. All
have cache-name and qualified-path agreement after ASCII case folding. Formats
are 83 BGRA8, 26 BC3/DXT5 and one BC1/DXT1. Fifty textures save MipTailBaseIdx=7;
60 omit it, but all manifest indices are still 0. The bridge therefore does not
add MipTailBaseIdx to the requested index. Other actual mip-chain layouts are not
established by this sample. Sparse explicit indices have focused synthetic
coverage, with absent indices returning NotFound.

**VERIFIED (actual full-file loader):** the existing `TextureCacheFile::read_mip`
and the new binding both read exports 3, 4 and 5 from the complete Icons file.
All 731,056 decoded pixel-format bytes equal the earlier independently decoded
liblzo2 oracle files byte for byte. The new RGBA path also reproduces all
2,727,616 bytes of the earlier private RGBA outputs. This RGBA comparison is a
regression check against the project's earlier decoder result, not a separate
retail image oracle. All 110 actual descriptors also resolve through the new
bridge; only the selected three payloads were decoded in this focused smoke.

Three new focused tests pass: exact qualified-name/GUID/cache/index matching,
ambiguous named records, sparse-index misses without fallback, invalid cache
names/absence, bulk kind/flags/counts/dimensions, partial placeholders and explicit
range conflicts. The actual harness also checks cyclic and extreme invalid outer
references. Thirty-seven unrelated tests were filtered out; no full suite or
rendered client check was run.

Private evidence is under `evidence/texture-cache-binding/`: `full-file-check.rs`
and `full-file-check.tsv`, `tests.txt`, `post-smoke-sources.json`,
`independent/binding-receipt.json`, `source-ledger.json`, and `validation.json`.
The original CharacterSelect package is 10,351 bytes with full SHA-256
`367a51bd755b95b192c7b2f3a6fd70a0150433afb79d3e8835d9c225606619b9`; it and the
manifest were rechecked against the source inventory. All asset bytes and
receipts remain private. No root Texture2D or renderer hook was changed.

No new third-party source was read for this bridge. The inputs were the owner's
existing files, saved independent range/oracle receipts, and original project
`src/texture.rs`, `src/upk.rs`, `src/texture_cache.rs`, `payload.rs`, and prior
private harnesses. Existing BC/LZO decoders are reused without new dependencies.
Exact paths/hashes are recorded in the private source ledger. The earlier outside
source register below continues to apply to the underlying format readers.

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
conversion is delegated to `texture::decode_pixels`. Unobserved compression
profiles and retail renderer/runtime behavior remain outside the verified scope.
