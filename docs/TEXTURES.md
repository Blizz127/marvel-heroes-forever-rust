# UE3 Texture2D readers and inline pixel decoding

Task `a3d58c579542`, 2026-11-15. Original std-only Rust implementation;
proprietary packages and all derived pixel data remain private. This increment
provides asset pixels, not a reconstructed login UI or verified retail renderer.

## Public API

- `Texture2D::from_export(&upk::Package, export_index, &serialized_export)` uses
  the checked generic property reader, requires Texture2D and scalar SizeX,
  SizeY and EPixelFormat fields, then consumes the complete native texture tail.
- `Texture2D::parse_native(Descriptor, bytes, logical_offset)` accepts explicitly
  supplied property metadata and the tail's absolute logical package offset.
- `Texture2D::decode_mip(index)` decodes an inline, uncompressed-bulk mip to
  `RgbaImage { width, height, pixels }`. Package-level LZO is handled upstream.
- `decode_pixels(PixelFormat, width, height, bytes)` handles already resolved and
  bulk-decompressed pixel bytes. Cache identity, lookup and decompression are
  separate responsibilities; this function neither opens files nor guesses names.

`PixelFormat` supports `Bgra8` (`PF_A8R8G8B8`), `Bc1` (`PF_DXT1`), `Bc2`
(`PF_DXT3`) and `Bc3` (`PF_DXT5`). BC2 is covered by synthetic/reference checks,
not observed in the currently inspected frontend textures. Other formats fail
explicitly. The [BC reader ledger](BC_FORMAT.md) is part of this source register.

`Descriptor` retains width/height, format, optional SRGB/NeverStream/cache-name
properties and the optional mip-tail base index. Absence is not substituted with
a guessed default. No gamma conversion or alpha premultiplication is applied:
RGBA output is row-major tightly packed channel data. The renderer must preserve
an explicit color-transfer and blend-state decision.

## Native layout and scope

**OBSERVED:** All 692 Texture2D native tails in the two selected packages are
consumed exactly by this observed 868/3 layout:

1. SourceArt bulk header and any inline bytes.
2. Primary mip array; each mip has a bulk header/payload followed by width/height.
3. Sixteen-byte texture-cache GUID.
4. Cached PVRTC mip array.
5. Cached Flash maximum resolution, cached ATI mip array and Flash bulk.
6. Cached ETC mip array.

Bulk headers carry flags, element count, signed stored byte count and signed
stored offset. The signed fields retain the observed `-1` external placeholders.
Unused or zero-element bulks have no inline payload. The external and separately
located flags produce `BulkPayload::External` and `Separate`; inline bytes remain
in `Inline(Vec<u8>)`. Unknown flags, conflicting storage/codecs, bad ranges or
unexpected trailing data fail instead of being guessed. Cached alternate-format
mips and source-art data are retained as metadata/payloads, not decoded as primary
pixels.

**OBSERVED:** Every one of the 582 frontend primary mip bulks has flags zero and
inline bytes, including the 63 textures carrying cache name `textures`. Cache
name alone therefore does not select an external read. Every one of the 110
character-icon mips uses flags `0x11`, cache name `icons`, and `-1` size/offset;
these require separately implemented manifest/TFC resolution. `decode_mip`
returns Unsupported for them and for individually compressed bulk data. No
placeholder pixels or implicit fallback mip are produced.

Limits are 64 MiB native/individual bulk bytes, 32 mips per serialized array,
16,384 per dimension and 256 MiB RGBA output. Encoded sizes use complete 4×4 BC
block grids, including cropped edge blocks. Input/output bounds, scalar-property
duplicates, count/format consistency and allocation failures are checked.

## Actual package observations

The frontend source is the owner's copied `MarvelFrontEnd_SF.upk`, SHA-256
`af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190`.
The icon source is `ICO__MarvelUIIcons_CharacterSelect_SF.upk`, SHA-256
`367a51bd755b95b192c7b2f3a6fd70a0150433afb79d3e8835d9c225606619b9`.
Their private source/copy identity is documented in `UPK_FORMAT.md`.

**VERIFIED:** The new reader parsed and decoded all 582 inline frontend textures
without failure, yielding **176,983,936 RGBA bytes** in total. There are 23 BC1,
558 BC3 and one BGRA8 textures. Each has one primary mip with dimensions equal
to its serialized SizeX/SizeY; each explicitly serializes SRGB=false. All 110
external icon textures parsed with preserved placeholders and refused pixel
readout as expected. Counts do not establish visual or whole-client parity.

**VERIFIED:** Four native-reader tests cover BGRA channel order with unchanged
alpha/transfer bytes, external-placeholder preservation, every truncated prefix,
trailing data, invalid flags/counts/offsets/dimensions and allocation budgets.
Six BC tests are described in `BC_FORMAT.md`. The original BC expansion uses the
Microsoft Direct3D 9 rounded interpolation convention. Independent Pillow checks
show at most one unit per channel difference, so byte-identical hardware or retail
pixel output is explicitly unverified.

**VERIFIED:** Five real decoded samples were independently compared with Pillow:
four BC1 samples differed by at most one channel unit; the BGRA sample matched
exactly. The receipt retains both hashes and differing-channel counts. These
comparisons do not establish the original shader, sampler or display behavior.

Private evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `texture-validation/retail-smoke.tsv` and `retail-smoke-final.tsv`: per-texture
  export index/name/dimensions/format/count, final totals and explicit external
  refusal checks through the Rust parser.
- `texture-validation/unit-tests.txt`, `native-tests-final.txt`.
- `texture-validation/samples.json`: native-size decoded sample metadata/hashes.
- `texture-validation/real-pillow-comparison.json`: independent sample comparison.
- `upk-object-research.json`: independent property/native-layout inventory.
- `upk-object-samples/manifest.json`: original serialized export sample identities.

These are static file and decoder observations, not REA native execution evidence.
REA and actual retail runtime comparisons remain the project lead's separate work.

## Outside source and licence register

All UEViewer references were read from pinned commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981`, MIT (copyright Konstantin Nosov).
They were studied for format facts; no reference implementation was copied.
Their private source hashes are retained in `evidence/upk-reference/`.

| Outside source | Licence/status | Use |
| --- | --- | --- |
| [UEViewer UnTexture3.cpp](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnTexture3.cpp) | MIT | Texture2D native field order, mip width/height placement and version gates. |
| [UEViewer UnMaterial3.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnMaterial3.h) | MIT | Property names, pixel format designations, cache GUID/name and mip metadata. |
| [UEViewer UnCoreSerialize.cpp](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp) | MIT | Bulk header signedness and inline/external/separate/unused storage meaning. |
| [UEViewer UnCore.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCore.h) | MIT | Bulk flag values; targeted definitions only. |
| [UEViewer LICENSE.txt](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt) | MIT | Licence verification. |
| Owner's copied packages, private logical exports and sample manifests listed above | Proprietary Gazillion/Marvel data; owner-authorized private interoperability study | Actual native layout/format/pixel evidence; nothing included in source control or builds. |
| Microsoft BC/DXT/DDS primary format documents | MicrosoftDocs CC-BY-4.0; full URL list in `BC_FORMAT.md` | Original BC decoder grammar and independent DDS validation wrapper. |
| Installed Pillow 13.0.0, via its image loading API | MIT-CMU, separately installed validation only; full register in `BC_FORMAT.md` | Independent DDS and BGRA output comparison; no decoder source read/copied or runtime dependency added. |

The other task helpers' original parsers and private receipts were used as
internal integration inputs. Package/object code remains in its existing module;
this increment adds no Cargo dependency and no filesystem extraction interface.
