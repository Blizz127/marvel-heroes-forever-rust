# Frontend GFX movie structure

`src/gfx.rs` is original, dependency-free Rust for bounded static SWF/GFX
inspection. It does not execute ActionScript, create UI controls, render shapes
or claim frontend parity. Retail files remain in the private evidence directory.

## Observed client profile

**OBSERVED:** All 20 `SwfMovie` RawData arrays in the pinned
`MarvelFrontEnd_SF.upk` start with uncompressed `GFX`. Their declared file sizes
exactly match the property-array lengths. Versions 10, 11 and 15 occur. Adobe's documented SWF framing parses
these GFX headers and tag envelopes consistently. Compressed SWF/GFX and versions
outside 1–19 return explicit unsupported errors; no decompression dependency is
required for these 20 files.

The source UPK SHA-256 is
`af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190`.
Its private path is
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFrontEnd_SF.upk`.
The initial independent extraction used the private liblzo logical package and
observed UE3 property ranges. The original Rust package-reader extractions of
login/character-select RawData match those independently extracted bytes.

| Movie | RawData bytes | Stage pixels | Tags including sprites | Dictionary IDs | Sprite timelines | Root first-frame placements |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| `marvel_login_dialog` | 168,573 | 1920 × 1200 | 1,654 | 268 | 87 | 13 |
| `marvel_character_select` | 193,385 | 1920 × 1200 | 7,613 | 355 | 122 | 6 |

These are file-declared/static facts. Scripts can alter placement, visibility,
text and scene choice. In particular, the frontend map references
`marvelfrontend.frontend_manager`, not directly the login movie; that startup
composition is being investigated separately. This module does not select an
initial application screen from names alone.

## API and semantics

`Movie::parse(&[u8])` and `parse_with_limits` return an owned movie with:

- `header`: signature family, version, exact length, signed stage twips, raw 8.8
  frame rate, declared frame count and tag-stream offset.
- `tags`: flattened records with source offsets, body spans, timeline IDs and
  zero-based frame indices. Sprite children retain their own timeline.
- `dictionary`: local definition/import IDs, character kind, tag index and
  available bounds. Duplicate or zero definition IDs are rejected.
- `timelines`: root ID 0 and each sprite's ordered direct tag indices.
- `bytes()` and `Span::bytes`: checked access to retained original payload ranges.
- `tag_counts()`: complete parsed-envelope counts, including nested timelines.

Typed tags cover timeline placement/removal, labels, background color,
file attributes, asset exports/imports, symbol classes, scaling grids, sprites
and the observed external-image profile. `PlaceObject`/`2`/`3` preserve signed
16.16 matrices, signed color-transform components, names, depths, ratios,
clipping, visibility, blend/cache values, filter records and clip-action ranges.
The eight documented filter envelopes are bounded and retained, not rendered.
Strings are bounded UTF-8; unknown encoding is rejected rather than guessed.

`static_frame(timeline_id, frame_index)` applies direct placements/removals
through a recorded ShowFrame. It preserves depth ordering, move/replacement
fields, and default transforms; invalid move/remove assumptions return errors.
It does **not** instantiate nested sprites or execute actions. The returned
`ignored_action_tags` and each object's `has_clip_actions` keep that limitation
visible. This is a structural snapshot, not the runtime screen.

Shape, morph, text, font, button, bitmap, sound and other definition bodies remain
opaque after their ID and applicable bounds. AVM1 actions are retained as ranges;
no instruction interpretation or action validation is claimed. Unknown tags
retain exact code/offset/length. Placement references are not fully resolved
across imported movies by this module.

Subsequent original readers now decode selected retained bodies separately:
[shapes](GFX_SHAPES.md), [fonts/text](GFX_TEXT.md) and [AVM1 actions](AVM1_FORMAT.md).
The envelope/static-frame API above still does not execute those instructions or
compose their geometry. [Frontend binding](FRONTEND_READER.md) resolves saved
external-image references against explicitly supplied package archives.

## Inferred GFX external image encoding

**INFERRED from the owner's files:** All 716 tag-1009 payloads fit one exact
encoding: a little-endian 32-bit identifier (within 16-bit dictionary range in
these samples), three 16-bit fields, then two single-byte-length UTF-8 strings.
The fields behave as image format, target width/height, export linkage name and
filename. Every observed format value is 13; Autodesk documents that enum value
as TGA. A direct case-insensitive export-name lookup finds only 669/716 records. The
UPK lane's separately retained `frontend-image-mapping.json` finds unique
reference-array-constrained candidates for all 716 (677 local textures and 39
imports), including inferred filename-stem normalization. This is corroborated
static association, not a verified native binding rule. No path is opened by
this parser, and no public specification of tag 1009 was located.

The parser exposes this as `TagData::ExternalImage`, while preserving the source
span. This measured profile must not be generalized to every Scaleform version.
Autodesk's public control-tag table names older external-image tags but provides
no serialization grammar. It identifies `ExporterInfo` as **1000**, not 1023.
ExporterInfo is retained opaque; C++ API field order is not used as a disk layout.

## Limits and verification

Default limits are 64 MiB input, 250,000 total tags, 64 nested sprite levels and
65,536 bytes per zero-terminated string. Configurable nesting is capped at 256.
Declared movie/tag lengths, nested ranges, End termination, ShowFrame totals,
bit-field bounds, strings, dictionary IDs, filters and clip-action sizes are
checked. Public spans use checked arithmetic. Opaque payload internals are not
validated by successful envelope parsing.

**VERIFIED:** All 20 actual movie streams (3,278,987 bytes, 33,817 tag records,
1,413 sprites) parse. Their complete tag inventories
match an independent Python envelope scan exactly. Root first-frame snapshots
succeed for all 20; this establishes structural handling only. Eight focused
synthetic tests pass, covering placement/move/removal, independent sprite
frames, signed bit fields, image/import IDs, clip-action bounds, filter envelopes,
truncation/length/count budgets and explicit unsupported compression.

The retained unknown-work inventory contains 493 opaque tags (codes 74, 77,
1000), 1,191 opaque definition bodies and 2,063 unexecuted action tags. Placement
filter/clip-action ranges remain available through their typed records.

Private evidence, relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `gfx-samples/`, `gfx-tag-inventory.json`: independently extracted file hashes,
  complete tag counts and extension ranges.
- `gfx-external-images.json`: all 716 inferred external-image records, with movie
  hashes and exact payload offsets.
- `gfx-smoke.tsv`, `gfx-smoke.rs`: Rust parser results and private original harness.
- `gfx-unit-tests.txt`: focused test results.
- `gfx-coverage.json`: exact source ranges for opaque tags, unparsed definition
  bodies and unexecuted action tags; private follow-up work inventory.
- `upk-object-samples/manifest.json`, `upk-rust-samples/`: independent and Rust
  extraction identities supplied by the UPK lane.

These are ordinary static-parser observations. No REA or native client execution
was performed in this helper; the integrator maintains the separate REA evidence.

## Sources and licences

All implementation was written here. No reference parser or runtime was copied.

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), chapters 1–4 and sprite structure | ©2006–2012 Adobe, all rights reserved; document study only | Standard header, bit fields, tag framing, transforms, placements, filter/clip-action envelopes, symbols and timelines. Adobe's original hosted PDF URL returned 404, so the authored document was read through this mirror. |
| Autodesk [control tags](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/flash_support/controltags.html) and [file formats](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/00556.html) | Autodesk documentation, all rights reserved | Extension-tag names/IDs and image-format enum semantics; no on-disk grammar inferred from declarations. |
| Autodesk [ExporterInfo](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/00526.html), [ImageFileInfo](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/00743.html), [ImageCreateExportInfo](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/00699.html) | Autodesk documentation, all rights reserved | Naming/semantic cross-checks only; C++ layout not treated as serialization. |
| Autodesk [GFx namespace](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/00216.html), [structures](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/cpp_ref/07408.html), [GFxExport](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/gfxexport_reference/usage.html), [GFxExport2](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/gfxexport2_reference/usage.html), [source access](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/source_access.html), [legal notice](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/welcome/legal.html) | ©2015 Autodesk, all rights reserved | Discovery of public format guidance and licence/access boundary. No proprietary SDK source accessed. |
| Epic [Scaleform technical guide](https://docs.unrealengine.com/udk/Three/ScaleformTechnicalGuide.html) | ©2001–2012 Epic, separate Scaleform notice; no open-source licence established | UE3 external-texture linkage semantics only; no runtime binding implementation copied. |
| Owner's pinned frontend package and derived private samples/UPK-lane manifests | Proprietary game assets; owner-authorized interoperability study | Actual profile, inferred extension grammar and parser validation; no asset bytes in git/builds. |

Search-result discovery also returned Ruffle type names, third-party format
summaries and unofficial SDK mirrors. Their implementation source was not used.
No packages were acquired, no credentials/access settings changed, and no
unofficial SDK was opened.
