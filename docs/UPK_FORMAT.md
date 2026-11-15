# UE3 package metadata

Status: **OBSERVED** on five privately pinned packages from the owner's Alienware
installation. This implements package metadata, not UE3 object deserialization,
rendering, or gameplay parity. Source is original Rust; reference implementations
were studied for format facts only.

## Supported profile and API

`upk::Summary::parse(bytes)` and `Summary::open(path)` inspect the stored package
summary and compression map. `upk::Package::parse(bytes)` and `Package::open(path)`
also decode the name, import, and export tables. Both return `std::io::Result`.
`Package::resolve_name(NameRef)` interprets the serialized name suffix.

The reader accepts the little-endian UE3 signature `0x9e2a83c1`, file version
`868`, and licensee version `3`. Other profiles return `Unsupported`. The five
samples report engine version `10897`, cooker version `136`, compression flag
`2`, and 128 KiB compression blocks. Flag `2` payloads successfully decode as
LZO1X version 0. Uncompressed profile-compatible packages are supported by the
synthetic fixture; none of the five retail samples is uncompressed.

The summary retains table counts and logical offsets, package flags, GUID,
generation records, engine/cooker values, compression chunks, and package source.
Fields after package source remain opaque. Name entries retain text and flags;
imports retain class/outer/name references; exports retain class, superclass,
outer, archetype, object name, flags, serialized range, generation counts and GUID.

## Compression and bounds

Stored and logical offsets have different address spaces. Each compression-map
row records both. The reader validates the chunk envelope and its block totals,
then decodes only the blocks covering the metadata header. Export data ranges are
checked against the declared logical file size, but export payloads are not
decoded or verified. A successful metadata parse therefore does not establish
integrity of the whole package; use the independent private pin hashes for that.

The raw LZO1X decoder checks every input/output range and dictionary distance,
supports overlapping copies, requires the canonical end marker and exact output
size, and rejects trailing data and versioned LZO-RLE streams. It is an original
implementation from the public format description, with no external codec linked.

Inspection limits are 512 MiB stored/logical package size, one million table or
chunk records, 4,096 generations, and 1 MiB per string. Table allocation also
requires enough bytes for the minimum encoded record sizes. Metadata tables may
not overlap by sharing an offset or consume the next table. References must fall
inside the declared name/import/export table counts. String termination and UTF-16
are checked; non-ASCII narrow strings are unsupported until their code page is
established. Sizes and offsets use checked arithmetic and negative values fail.

## Private sample evidence

Source installation:
`alienware-bazzite:/var/home/blizz/Games/banshee-library/games/marvel-heroes/`.
Private sample directory:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/`.
The candidate name does not establish executable build identity; that is tracked
by the integrator's separate retail pin evidence.

**VERIFIED:** SHA-256 of each copied sample equals its entry in
`~/.local/share/marvel-heroes-forever-rust/evidence/alienware-distribution-inventory.json`.

| Sample | Stored bytes | Names | Imports | Exports | SHA-256 |
| --- | ---: | ---: | ---: | ---: | --- |
| `music__mrhyde_area_SF.upk` | 436 | 8 | 4 | 2 | `a63b2240adb98c4d410c51b298776f3cd6a994390b8a8302db70081e1361ba86` |
| `UC__MarvelProjectile_Empty_SF.upk` | 733 | 18 | 9 | 5 | `088fd70969d531c237349543c1741adcd6c03553dbb7d736c1f603e3a487bd01` |
| `ICO__MarvelUIIcons_CharacterSelect_SF.upk` | 10,351 | 150 | 5 | 113 | `367a51bd755b95b192c7b2f3a6fd70a0150433afb79d3e8835d9c225606619b9` |
| `MarvelFrontEndMap.upk` | 12,038 | 199 | 45 | 37 | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` |
| `MarvelFrontEnd_SF.upk` | 8,905,511 | 613 | 20 | 606 | `af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190` |

**OBSERVED:** An independent private Python/`liblzo2` 2.10 probe decoded the sample
chunks and confirmed name-table and export-table endpoints. Its receipts are in
`~/.local/share/marvel-heroes-forever-rust/evidence/upk-validation/liblzo-baseline.json`.
No copied package bytes or extracted data are stored in the repository. These
shell observations are not REA Evidence; the integrator owns the REA session.

**VERIFIED:** Thirteen focused Rust tests pass (six package/string/envelope tests
and seven raw-codec tests). The Rust package reader loads all five sample tables
with the counts above. The independent LZO check compared all 322 compressed
blocks, totaling 38,845,307 decoded bytes, against installed `liblzo2` byte for
byte. Private receipts: `evidence/upk-validation/checks.json`,
`evidence/upk-lzo-retail-validation.json`, and `evidence/upk-lzo-checks.json` under
the project private data directory. Synthetic tests establish checked failure
behavior; the five retail reads establish only the covered profile and metadata.

## Provenance for the central ledger

All UEViewer source references below are pinned to
`a0bfb468d42be831b126632fd8a0ae6b3614f981` (MIT, copyright Konstantin Nosov),
read on 2026-11-15. Public reference snapshots and the exact source list remain
private under `evidence/upk-reference/`; only format knowledge was used here.

| Outside source | License | Use |
| --- | --- | --- |
| [UEViewer `UnPackage3.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealPackage/UnPackage3.cpp) | MIT | UE3 summary, name-table and export-table field order; version boundaries. |
| [UEViewer `UnPackage.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealPackage/UnPackage.h) | MIT | Generation and compression-map fields. |
| [UEViewer `UnPackage.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealPackage/UnPackage.cpp) | MIT | Signature/version encoding, import fields and name suffix interpretation. |
| [UEViewer `UnCoreSerialize.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp) | MIT | FString and compressed-block envelope encodings. |
| [UEViewer `UnPackageUE3Reader.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealPackage/UnPackageUE3Reader.h) | MIT | Stored/logical offset distinction and block-selective reads. |
| [UEViewer `LICENSE.txt`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt) | MIT | License verification. |
| [Linux kernel LZO format documentation](https://docs.kernel.org/staging/lzo.html) | GPL-2.0, documentation studied only | LZO1X instruction grammar for an original Rust decoder. |
| Installed `/usr/lib/x86_64-linux-gnu/liblzo2.so.2` | GPL-2.0-or-later; separately installed system library | Private differential validation only; neither linked nor distributed. |
| `/usr/share/doc/liblzo2-2/copyright` | Copyright and license notice | Verification of the private validation oracle's license. |
| Owner's five packages listed above | Proprietary Gazillion/Marvel game data | Private interoperability study and format validation. |
| `/home/blizz/dev/mh-omega-study` | License not assessed | Directory inventory only; no source or footage read or reused. |

## Remaining work

Object properties, dependencies, package-level import resolution, external bulk
data, textures, materials, meshes, animations, audio and Scaleform payloads remain
unimplemented. Other profiles and codecs are unsupported. These metadata checks
do not establish retail runtime behavior or visual parity.
