# Marvel texture-cache manifest

`src/texture_cache.rs` is an original, std-only bounded metadata reader and
resolver for the observed 1.52-client `TextureFileCacheManifest.bin`. It does not
open texture caches automatically, extract files, decode stored blocks, or claim
that an image has been rendered.

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
decoded by this validation.

## Source and licence register

The following outside sources were read for this increment. All module code is
newly written here; no reference implementation was copied.

| Source | Licence / identity | Used for |
| --- | --- | --- |
| [UEViewer `Unreal/UnrealMaterial/UnTexture3.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnTexture3.cpp), lines 566–640; private snapshot `evidence/upk-reference/Unreal__UnrealMaterial__UnTexture3.cpp` | MIT; commit `a0bfb468d42be831b126632fd8a0ae6b3614f981` | Reference manifest fields and GUID/mip relationship; actual extra FString and collision behavior were independently established from the owner's bytes. |
| [UEViewer `Unreal/UnCoreSerialize.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp), FString section lines 359–436; private matching snapshot | MIT, same commit | FString count/termination encoding; Unicode is validated here rather than reproducing the reference's lossy conversion. |
| [UEViewer `LICENSE.txt`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt), private matching snapshot | MIT, copyright 2022 Konstantin Nosov | Confirm licence text. |
| Owner's copied manifest and `evidence/alienware-distribution-inventory.json` | Proprietary Gazillion game data; owner-authorized private interoperability study | Identity, actual format, candidate collisions and source-cache bounds. No data included in git or builds. |
| Original project `evidence/upk-object-research.json` and its private logical package observations | Project-generated analysis of proprietary owner data; the report's reference-source ledger is retained privately | CharacterSelect texture GUID/mip/cache references and qualified object paths; no runtime assertion. |

Cache decompression, actual cache-byte reads, texture pixel reconstruction and
retail renderer behavior are outside this module's implemented/verified scope.
