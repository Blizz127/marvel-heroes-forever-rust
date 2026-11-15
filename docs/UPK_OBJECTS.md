# Serialized exports and tagged properties

This increment reads original stored exports through the existing bounded LZO
reader and decodes the supported UE3 asset property layout. It does not execute
GFX movies or render textures. All raw assets and extracted samples stay under
`~/.local/share/marvel-heroes-forever-rust/`.

## API and scope

`upk::PackageArchive::open(path)` binds immutable metadata to its original stored
bytes. `package()` returns that metadata; `read_export(index)` reads one serialized
export; `read_range(logical_offset, length)` reads a checked logical range. Reads
may cross compression blocks or chunks, with a 64 MiB per-read limit. Compressed
ranges must be fully covered by the chunk map. The reader does not fabricate a
logical package-summary prefix from the longer stored compression summary.

`objects::ObjectProperties::parse_export(package, index, bytes)` requires the
declared export length, then reads a standard `i32` NetIndex followed by tagged
properties. It accepts the supported asset class names case-insensitively:
Texture2D, SwfMovie, Package, ObjectReferencer/GazCookedObjectReferencer, Material,
MaterialInstanceConstant and MaterialInstanceTimeVarying. Material preambles are
reference-based; material exports were absent from the five selected samples.
UClass, actors and custom gameplay class preambles remain unsupported.

`parse_tagged_properties(package, bytes, start_offset)` is the lower-level entry
point when the caller has independently established a property's starting offset.
It makes no claim about preceding or trailing bytes.

Each property retains name/type references, array index, tag span, payload span,
type-specific tag details and a decoded value where supported. Integers, exact
float bits, booleans, bytes, names, object references and strings are typed.
Structures retain their struct name and opaque payload. Arrays retain count and
element-byte span; the generic tag does not establish the element schema. Unknown
value types remain opaque. `native_payload` identifies the class-specific bytes
following the terminating None name. All spans are relative to the serialized
export, with a checked helper for translating to logical package offsets.

Bounds include 16,384 properties per list, 64 MiB per serialized input/read and
the existing string limits. Scalar widths, terminating names, indices, object
references and canonical boolean tags are checked. A successful property parse
does not validate a native class payload or establish its runtime meaning.

## Observed front-end assets

**OBSERVED:** Independent private inspection of the five packages listed in
[UPK_FORMAT.md](UPK_FORMAT.md) found 725 supported asset property streams with
NetIndex `-1`. Asset property names, types and class names are lowercase in these
samples. The property streams terminate cleanly. They contain:

- 582 Texture2D exports and 20 SwfMovie exports in `MarvelFrontEnd_SF.upk`.
- 110 Texture2D exports in `ICO__MarvelUIIcons_CharacterSelect_SF.upk`.
- No Material or MaterialInstance exports. The front-end map imports engine
  default/removal materials; that is a dependency observation, not a loaded
  material implementation.

**OBSERVED:** All 582 front-end texture mip payloads are inline with bulk flag
zero. Some also carry a texture cache property, so that property alone does not
identify external data. Useful examples include `frontendbg_2560x1440`,
`playscreen_bg`, and `characterselect_bg_png`. Their Texture2D native tails start
294 bytes into each serialized export.

**OBSERVED:** The 20 movie RawData arrays begin with `GFX`; their declared lengths
equal the byte-array payload lengths. `marvel_login_dialog` is version 15 with
168,573 bytes; `marvel_character_select` is version 10 with 193,385 bytes. Movie
execution and UI behavior are unverified. Their actual assets are supplied to the
separate GFX parser work, rather than replaced with a mock login screen.

**OBSERVED:** The 110 separate character-select texture mips carry bulk flags
`0x11`, cache name `icons`, and stored-size/offset placeholders `-1`. Do not cast
those placeholders to unsigned offsets.

**INFERRED from the Marvel-specific reference reader:** External placeholders
resolve through `TextureFileCacheManifest.bin`, using texture cache GUID and mip
index. The owner inventory contains that manifest (6,605,396 bytes), `Icons.tfc`
(143,472,250 bytes), and `Textures.tfc` (883,956,008 bytes). The manifest itself was
not parsed in this increment. Inline front-end assets need no external TFC read.

Private research and extraction records:

- `evidence/upk-object-research.json`: complete class/property/native-tail study,
  source identities, observed counts and explicit inference boundaries.
- `evidence/upk-object-samples/manifest.json`: selected export and movie hashes,
  serialized offsets, property spans and texture bulk records from an independent
  Python/system-LZO probe.
- `evidence/upk-validation/object-smoke.txt`: original Rust archive/property smoke.
- `evidence/upk-rust-samples/`: original Rust export/GFX extractions for comparison.

These are shell and parser observations, not REA output. REA binary evidence is
maintained separately by the project integrator.

**VERIFIED:** Nineteen focused package/archive/LZO/property tests pass. The Rust
archive and property reader parsed 725 supported exports containing 6,442
properties across the explicit five-file sample set; 38 exports with unsupported
class preambles were skipped. Seven selected Rust extractions (five serialized
exports and two GFX arrays) match the independent probe byte for byte. The receipt
is `evidence/upk-validation/object-checks.json`. This establishes the tested
property/range behavior, not native payload or gameplay parity.

## Provenance additions

The sources below were studied at UEViewer commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981`, MIT (Konstantin Nosov), read 2026-11-15.
Code here is original Rust; no external implementation is copied or linked.
The previous package/LZO sources and oracle remain listed in
[UPK_FORMAT.md](UPK_FORMAT.md). Source SHA-256 values are retained in the private
research record.

| Source | Use |
| --- | --- |
| [UEViewer `UnObject.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnObject.cpp) | UObject NetIndex prefix, property tags and scalar encoding. |
| [UEViewer `UnObject.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnObject.h) | UObject representation context. |
| [UEViewer `UnCore.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCore.h) | Bulk-data flags and core format declarations. |
| [UEViewer `UnTexture3.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnTexture3.cpp) | Independent texture-tail research and Marvel cache-manifest mapping. |
| [UEViewer `UnMaterial3.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnMaterial3.h) | Texture/material property declarations; material behavior remains unverified. |
| Owner's private five-package pin | Proprietary game data; interoperability observations and focused reads only. |
