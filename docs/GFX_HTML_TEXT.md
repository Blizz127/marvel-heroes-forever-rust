# Source-preserving HTML text declarations

`src/gfx/html_text.rs` reads saved HTML EditText content into bounded tokens,
element scopes, Unicode atoms, runs and formatting declarations. It does not
strip markup into a replacement plain-text field or perform text layout.

## API and source mapping

`HtmlText::parse_initial(&EditText)` borrows the exact initial string and retains
its saved field flags, FontID/font class, height in twips, RGBA color and paragraph
layout fields. The initial source span must account for its terminating NUL.
`HtmlText::parse(&str)` inspects an explicit string without manufacturing field
defaults or movie offsets.

- `tokens` cover every source byte, including tag syntax and opaque comments.
- `elements` retain opening/closing spans, nesting, attributes and separate
  character/paragraph declaration patches. Attributes retain exact name/value
  spans and decoded values where their entities are understood.
- `atoms` retain Unicode scalars or explicit unresolved entities. Each has exact
  source UTF-8 bytes, source UTF-16 units and optional absolute movie byte spans.
- `runs` associate atom ranges with their element scope and explicit paragraph.
  `scopes(run_index)` walks inner-to-outer declaration scopes, bounded at 64.
  It does not select a font, apply a style sheet or calculate relative sizes.
- `issues` identify unknown tags/attributes, invalid attribute values, opaque
  markup, unresolved entities, nested paragraphs and observed font extensions.

Paragraph boundaries and BR breaks are structural tokens, not injected newline
characters. Literal whitespace, trailing spaces and line-ending code points are
preserved. Entity-decoded UTF-16 positions exclude structural P/BR separators;
they are not a claim about the eventual TextField.text index. An unresolved
entity makes that decoded position unknown for itself and all following atoms.
Original markup UTF-16/source-byte mappings remain exact throughout.

The document also explicitly lists unresolved runtime behavior: whitespace and
style sheets, paragraph/break-to-text mapping, font alias/typeface resolution,
relative-size/default application, and numeric units/runtime rounding. Consumers
must not interpret a successful parse as permission to ignore those boundaries.

## Declaration subset and source basis

**OBSERVED in primary Adobe documentation:** htmlText accepts a limited HTML
subset. P supplies paragraph alignment; FONT supplies face, hexadecimal RGB and
absolute/relative size declarations. B/I/U mark character styles. TEXTFORMAT
carries paragraph margins, indentation, leading and tab stops. P/BR behavior
depends on multiline mode. Named entities documented there are lt, gt, amp,
quot and apos; decimal and hexadecimal character references are also described.
See [Adobe TextField](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html#htmlText).

These declarations are represented separately rather than converted into a
single resolved style. Decimal values retain integer coefficients and decimal
precision; no float rounding, twip conversion, font fallback or relative-size
calculation occurs. Font-face strings retain their original spelling and `$`
prefixes. RGB declarations do not replace saved RGBA alpha here. Native family
names, imported aliases and movie-local FontIDs remain distinct identities.

**OBSERVED extension, not documented HTML attribute mapping:** Every sampled
FONT stores `letterSpacing="0.000000"` and `kerning="1"`. Both are retained as
typed declarations with `Evidence::ObservedExtension` and an explicit issue.
Adobe documents the corresponding TextFormat properties, but the inspected
htmlText attribute list does not establish this FONT spelling's native mapping.
The parser does not infer or apply that behavior. [Adobe TextFormat](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextFormat.html)
also distinguishes paragraph from character properties and requires available
embedded typefaces for bold/italic; this module substitutes none.

Unknown balanced tags remain element scopes with issues; unknown attributes and
their exact source bytes remain inspectable. Comments are opaque tokens. Syntax
requiring repair is rejected: unclosed/mismatched tags, duplicate attributes,
unquoted values and unclosed quotes/comments. Other Adobe-supported tags such as
anchors, images, lists and CSS spans are not implemented as rendering features.
Undocumented entities, invalid Unicode references and numeric values outside the
accepted bounded syntax remain explicit issues, not replacement characters.

Bounds are 1 MiB of source, 262,144 source scalars/decoded atoms, 65,536 tokens,
16,384 elements, 65,536 attributes, 64 attributes per element, 128 bytes per
tag/attribute name and 64 nested elements. Decimal syntax permits up to nine
integer digits and six fractional digits. Tab-stop lists are capped at 4,096
entries. Those limits bound representation work and storage; they are not a
process RSS ceiling. Parsing allocates no GPU resources and opens no font files,
URLs or embedded media.

## Actual saved frontend fields

**OBSERVED:** The two private movies contain **36 HTML definitions**, 16 login
and 20 character-select. They have **39 complete P/FONT paragraphs**, **234
attributes**, and **1,660 literal text scalars**. Source totals are 6,218 UTF-8
bytes and 6,208 Unicode scalars; five U+2022 bullets account for the difference.
No entity, literal CR/LF/TAB, other tag or self-closing tag occurs in this sample.
All source strings plus their NUL match their original movie spans exactly.

The observed FONT aliases are `$Strat2Bold` and `$Strat2Medium`. Paragraph
alignments are left/center/right. All saved FONT sizes, RGB declarations and
paragraph alignments agree numerically with their field defaults; FONT size
times 20 agrees with saved height in twips. This is a comparison of stored data,
not a proof of native layout/default application.

**OBSERVED:** Character-select field **311** saves alpha **127**, while its
HTML FONT declares only `#ffffff`. The other 35 fields save alpha 255. The
representation preserves that distinction. Nine login Medium fields save
leading -40 twips; remaining fields save +40. No HTML textformat leading tag is
present, so that saved field property remains available independently.

**OBSERVED:** Login fields **75**, **198** and **206** contain two paragraphs.
Field 75 has multiline **false** (flags `0x8d23`), while 198/206 have multiline
**true** (`0xed33`). Fourteen of the 36 HTML fields have multiline false. Their
source structure alone therefore does not justify inserting a newline for every
P boundary. No paragraph is empty; a trailing space in field 75 is preserved.

**VERIFIED (explicit English library route):** Each of the 39 FONT FACE values
equals its field FontID's saved import symbol and the target English font export
symbol **exactly**, including case and `$`. Independent identity checks found:

| Exact alias | Paragraphs | Login FontID | Character FontID | English font ID | Raw native font name |
| --- | ---: | ---: | ---: | ---: | --- |
| `$Strat2Medium` | 14 | 74 | 162 | 4 | `Stratum2 Medium` followed by NUL |
| `$Strat2Bold` | 25 | 77 | 138 | 5 | `Stratum2 Bold` followed by NUL |

The original MovieLibrary was tested with and without the explicit
`MarvelFonts.gfxfontlib` → `MarvelFonts.fonts_en` host route. Without it, all 36
field resolutions report `MissingNamedReference` for `marvelfonts.gfxfontlib`.
With it, all resolve to `marvelfonts.fonts_en`, UPK movie export index 18, and the
font IDs above. Source and English movie bytes match the independently extracted
copies. Alias equality and configured metadata resolution do not establish the
retail locale selection, native FACE-application behavior or rendered metrics.
No `$` stripping, alias rewriting or font substitution was performed.

Font comparison receipt SHA-256:
`c826cd29cdb7b0d699f5385de3c31cb2973e321f45b0f6f3cbc65a5aa3179d46`.

Private movie identities:

| Movie | SHA-256 |
| --- | --- |
| `marvel_login_dialog.gfx` | `56a39b5546161f3b96e96def43b81fba27103f20d8337079f778924d4904738f` |
| `marvel_character_select.gfx` | `a12781e8109d682a17dcfb11e622700f6ce33522a989e17b42b2a01137eb2034` |

These are the fixed extracted owner copies under
`~/.local/share/marvel-heroes-forever-rust/evidence/upk-rust-samples/`.
The corresponding `gfx-samples` copies used by earlier EditText research have
the same content identity. No client text or asset bytes enter source control.

## Focused validation

**VERIFIED (synthetic):** Eight focused tests cover UTF-8/UTF-16/entity origins,
supplementary Unicode, exact source coverage, structural paragraph/break tokens,
literal whitespace, nested scope restoration, separate RGB/RGBA/default units,
exact decimals and relative declarations, explicit unknowns, strict malformed
input, resource bounds and invalid saved spans. Entity and B/I/U/TEXTFORMAT/BR
branches are source-established and synthetic-tested; they were absent from the
selected owner HTML strings.

**VERIFIED (actual source comparison):** All 36 definitions parse through their
original DefineEditText body and `parse_initial` entry point. They yield **195
tokens**, **78 elements**, **39 runs**, and **78 observed-extension issues**.
An independent original Python comparison matches **180/180 files**,
**130,811 bytes**, including source strings, every scalar's UTF-8/source-UTF-16/
decoded-UTF-16/movie spans, all 234 attribute/declaration records, and exact
run/scope/paragraph indices.

Inventory SHA-256:
`aeccebd5a14fffde057da768151e82a2ede8eb0a6bf2c39e85d5ceaccd7cbb8b`.
Comparison receipt SHA-256:
`27fa96e3a2bf23a75a33ba099f6ca8d940d2734096a6a335174d71809ec77dae`.

Private evidence under `evidence/html-text-study/`:

| File | Purpose |
| --- | --- |
| `html-text-inventory.json`, `helper-observations.md` | Exact owner definitions, source spans/hashes, defaults and paragraph grammar. |
| `rust_html_check.rs`, `rust-html-check.txt` | Original Rust saved-field/token/declaration smoke. |
| `rust-html-comparison.json` | Independent source/declaration/mapping comparison receipt. |
| `font-face-comparison.json`, `library-font-identities.json`, `check_font_identities.rs` | Exact per-paragraph FACE/import/export/native-name identities and explicit-route resolution check. |
| `tests.txt` | Focused parser test results. |
| `sources.json`, `helper-sources.json` | Read-only source URLs/paths, licence descriptions and hashes. |

No HTML glyph layout, font selection, GPU draw, editing/input behavior or retail
pixel parity has been run by this increment. Existing plain layout continues
to reject HTML. The parser is a distinct representation for a later grounded
paragraph/run layout adapter, not a plain-text bypass.

## Provenance

All code and fixtures are original Rust. External documentation was read on
2026-11-15 for interface/format facts; no reference implementation was copied.

| Source | Licence/status and use |
| --- | --- |
| [Adobe AS3 TextField](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html) | Adobe copyrighted documentation; supported htmlText tags/attributes/entities and multiline/whitespace boundaries. Private HTML SHA-256 `f1c35956775fe317c6b651edd22cdb793e3f3d51ca7cda081ca1dc77d64ea482`. |
| [Adobe AS3 TextFormat](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextFormat.html) | Adobe copyrighted documentation; distinct character/paragraph properties, font/typeface availability and numeric units. Private HTML SHA-256 `429039adc3537fe4a6c92a8ee9f6947e0ce1cacbbacce864a0e25d5a86b9789e`. |
| [HARMAN AIR SDK TextField](https://airsdk.dev/reference/actionscript/3.0/flash/text/TextField.html) | Adobe/HARMAN documentation; version cross-check only. Its modern `htmlParagraphBehavior` property is not projected into the 2017 target. Read through the web tool; direct private snapshot returned HTTP 403 and was not retried. |
| Owner's two movies and English font-library movie | Proprietary assets; source/default/import/export identity comparison only, retained privately. |
| Existing original `gfx/text.rs`, `gfx/edit_layout.rs`, `gfx/glyphs.rs` and [REA_NATIVE_ALTERNATIVE.md](REA_NATIVE_ALTERNATIVE.md) | Internal API and earlier project evidence; source spans, font identity interfaces and separately recovered native alias/locale observations. No new REA session was started by this lane. |
