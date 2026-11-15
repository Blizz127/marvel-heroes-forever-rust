# Original GFX font and text readers

`src/gfx/text.rs` reads serialized font and text definitions from original movie
tag bodies. All returned spans are absolute offsets into the original movie.
The implementation uses Rust's standard library and contains no outside source
code or game data.

## Public interface

- `FontDefinition::parse(tag_code, body, body_offset)` supports DefineFont (10),
  DefineFont2 (48), and DefineFont3 (75). It preserves identifiers, flags, raw
  name bytes, character codes, glyph SHAPE spans, advances, bounds, metrics, and
  kerning pairs. Glyph records are scanned through their exact terminators;
  paths are not tessellated or rasterized. `units_per_em` distinguishes the
  coordinate scale. Font bounds contain raw EM coordinates, so the common
  `Rect::width_pixels()` helper, which assumes twips, does not apply to them.
- `StaticText::parse(tag_code, body, body_offset)` supports DefineText (11) and
  DefineText2 (33). It preserves bounds, transform, explicit style changes,
  packed glyph spans, glyph indices, and signed advances. Absent style fields
  remain `None`; the reader does not resolve inherited style state.
- `EditText::parse(body, body_offset)` supports DefineEditText (37), retaining
  font references, flags, layout fields, variable name, and initial text.
  String spans include their terminating NUL. Strings require valid UTF-8.

`EditFlags`, `EditLayout`, `Glyph`, and `TextRecord` are also re-exported from
the module. Consumers obtain a tag body with `tag.body.bytes(movie.bytes())`
and pass `tag.body.offset` as `body_offset`. The root integrator adds the module
declaration in `gfx.rs`; this increment does not edit that file.

## Observed original assets

**OBSERVED:** The combined reader consumes every supported definition in the
20 pinned frontend movies, `Startup-fonts.gfx`, and
`MarvelGame-frontend_manager.gfx` without trailing bytes:

| Definition | Actual count | Additional retained data |
| --- | ---: | --- |
| DefineFont3 | 12 | 3,494 glyph streams; 107,711 edge records |
| DefineText | 13 | 26 glyph entries |
| DefineEditText | 479 | All flags and initial string values |

Startup supplies seven embedded fonts with recorded names Times New Roman,
CCGibbonsGazette, Impact, Arial (two style variants), Slate Mobile, and Vivaldi.
Five other DefineFont3 definitions occur in `!uisource`,
`confirmationdialog_frontend`, and `emptypanel_frontend`. Names retain their
serialized NUL bytes; an empty name is not substituted. One zero-glyph
definition in `emptypanel_frontend` ends immediately after its count and has no
CodeTableOffset; that observed omission is explicitly supported. Nonempty
offset tables must describe adjacent bounded glyph slices beginning directly
after the table.

Startup's seven ExportAssets symbols were recorded privately. None establishes
the `$Strat2Medium`, `$Strat2Bold`, or `$RobotoBold` runtime aliases imported by
the frontend. No alias relationship is inferred from font appearance or names.

Actual samples contain no DefineFont, DefineFont2, DefineText2, or EditText
FontClass branch; those supported branches have **synthetic coverage only**.
No proprietary font-extension tag appeared in this sample set. Unsupported
font tags return `Unsupported`; their source body remains available through the
parent movie reader.

## Limits and unresolved behavior

Font bodies are limited to 64 MiB and each glyph to one million records; table
counts are bounded by their serialized 16-bit count and available bytes. Glyph
ranges, shape termination, offset overflow, bit widths, layout tables, and
trailing data are checked. Glyph streams introducing new style arrays are
unsupported. Text bodies are limited to 8 MiB, 65,536 records, one million
glyphs, and 65,536 bytes per string. Invalid flags, truncation, incompatible
font selections, invalid UTF-8, and trailing bytes fail closed.

The Adobe specification's FontClass prose and FontHeight table disagree. The
reader follows the formal table: FontHeight is present only with HasFont.
FontID and FontClass together are rejected. No supplied original movie takes
the FontClass branch, so compatibility there remains unverified.

This increment does not implement font substitution, text layout, HTML,
ActionScript, advanced antialiasing, DefineFontAlignZones, CSMTextSettings,
DefineFont4/CFF, or proprietary font extensions. Existing movie tags retain
their opaque bodies. There is no text-rendering or retail visual-parity claim.

## Validation and private evidence

**VERIFIED:** Ten focused tests passed, covering exact metadata/spans, signed
values, inherited-style preservation, every truncation of representative
records, malformed offsets/flags, unterminated streams/strings, trailing data,
unsupported glyph styles, narrow/wide character codes, and source overflow.

**OBSERVED:** The real-asset smoke above returned success. Evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `gfx-fonts/validation.json`: exact 22 input paths, lengths, SHA-256 values,
  parser SHA-256, totals, and explicit unrun checks.
- `gfx-fonts/retail-smoke.tsv` and `.stderr`: per-font counts and combined totals.
- `gfx-fonts/font-exports.json`: Startup's actual exported font symbols.
- `gfx-fonts/focused-tests.txt`: combined focused test result.
- `gfx-text-records/actual-text-tags.json` and `sources-and-validation.json`:
  independently scoped helper text-tag observations and source receipt.

Native-client rendering comparison, interactive input, localization behavior,
and all runtime execution remain unrun. These structural tests do not establish
behavioral parity.

## Outside-source ledger

1. Adobe Systems Incorporated, *SWF File Format Specification*, version 19,
   © 2006–2012 Adobe, all rights reserved; documentation study only, no outside
   implementation copied. Adobe-authored document obtained from
   <https://open-flash.github.io/mirrors/swf-spec-19.pdf>, read 2026-11-15.
   Chapter 10 (printed pages 151–175), with bit/shape serialization from the
   primitive and shape chapters, informed field ordering and units. Private
   snapshot `gfx-fonts/adobe-swf-spec-19.pdf`, 1,724,258 bytes, SHA-256
   `6a1625b0929a86329021d365ba079c7e1f51502dfb4ea7d92ec9947841625cdd`.
   This mirror is a document source, not a source-code dependency.
2. Blizz's private pinned Marvel Heroes 1.52.0.1700 movie bytes, proprietary
   Gazillion/Marvel assets, compatibility study only. Exact files/hashes are in
   the private receipts above; their UPK provenance is recorded in
   `docs/UPK_OBJECTS.md` and `docs/FRONTEND_COMPOSITION.md`. Used to verify actual
   tags, offsets, fields, zero-glyph omission, and complete consumption. No
   extracted font, text, or movie bytes enter git or distributable builds.

Internal code dependencies are the project's original `gfx::{Span,Rect,Matrix}`
types and original movie reader. The record helper's original implementation
was integrated from isolated commit `30c63f41aa02ccaeff112dccf94b395a88d00dd6`;
that helper used only the Adobe document and these private movies.
