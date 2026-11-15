# Saved HTML EditText metric layout

`gfx::html_layout` implements an original bounded layout path for the saved
P/FONT definitions in the two pinned frontend movies. It keeps source HTML and
declarations beside exact embedded-font geometry. It does not call the plain
layout API with stripped markup.

**VERIFIED:** the isolated implementation compiles, seven focused tests pass,
and 31 of the 36 original HTML definitions produce 945 glyph placements in 33
paragraphs. Independent original Python arithmetic matches 10,626 checked
identity/source/scalar values and 3,039 rational values, with zero mismatches.
These checks establish source decoding and the stated metric model. **Native
GFx line placement, rounding, hinting, and pixel parity remain unverified.**

## API and renderer boundary

`layout_initial(&EditText, resolver) -> Result<HtmlLayout, HtmlLayoutError>`
accepts a resolver from the field's local FontID to `HtmlFont`:

- `resolved: glyphs::ResolvedFont`, including the actual movie/font identity;
- `source_import_symbol: &str` from that local import;
- `target_export_symbol: &str` belonging to the resolved font character.

The caller must establish all three through the **same** library route. Matching
names alone does not resolve an import. Each FONT FACE must equal both symbols
exactly, including case and `$`; there is no prefix removal, font-name guess,
replacement face, or synthetic bold/italic style.

`HtmlLayout.geometry` is the existing `EditTextLayout` geometry representation.
It retains the HTML flag and `SavedInitial` content kind; this does not turn the
HTML into a plain update. Glyphs contain the actual font identity, glyph index,
shape span, EM units, size, RGBA, and rational x/baseline/advance. Its existing
`position` method scales original glyph coordinates. Font rasterization,
display transforms, clipping, visibility, and GPU integration remain callers'
responsibility; this module neither creates a window nor draws pixels.

The result owns the original markup, tokens, elements, atoms, runs, saved field
defaults, and source issues. Paragraph records identify the P/FONT elements,
atom ranges, line indices, size, alignment, and original signed pair
adjustments. Glyph source byte/UTF-16/movie spans refer to the **original
markup**, including an entity's complete source spelling. Atom decoded UTF-16
indices exclude structural P boundaries. No source newline, break span, or
movie byte is fabricated for a closing paragraph tag. Geometry `utf16_len`
counts the original markup; it is not an ActionScript text-length result.

## Supported profile and arithmetic

Each root is one balanced P containing exactly one balanced FONT and nonempty
simple left-to-right text. P explicitly declares left/right/center alignment.
FONT explicitly declares FACE, positive integral pixel SIZE, RGB COLOR,
zero LETTERSPACING, and enabled KERNING. All declarations remain available at
their original precision. Fractional pixel sizes are rejected even if they
could be represented as integer twips: the native integer conversion path has
not established their interpretation.

The exact field FontID resolves to a DefineFont2/3 Unicode font with original
advances and kerning pairs. Duplicate glyph codes or pair keys, missing metrics,
inconsistent identities/units, and absent glyphs fail. Font sizes are bounded
by the existing u16 twip representation; source-parser limits are inherited
(1 MiB markup, 262,144 atoms, 16,384 elements, depth 64). A font has at most
65,535 glyphs and kerning pairs. Element/run indexing is linear; code/pair maps
use bounded ordered maps. Arithmetic uses i128 with a common denominator of
twice the font EM units, preserving exact center alignment without float
rounding. No unbounded recursion or input-dependent external access occurs.

The placement model shares the documented Flash metrics used by the original
plain layout: 40-twip side gutters, saved margins/indent, first baseline at top
plus scaled ascent, and line height equal to scaled ascent plus descent. The
next paragraph top adds that height and the saved signed paragraph leading.
The separate font-leading field is retained in the source font and is not
added again. Each internal adjacent pair contributes its serialized signed
kerning adjustment; no pair crosses a paragraph boundary. Output glyph advance
includes the adjustment **after** that glyph. Paragraph width and alignment use
the same adjusted advances. Original negative field coordinates are retained.

Saved multiline fields with multiple P elements produce separate paragraph
lines under this documented model. A single-line field with multiple paragraphs
is rejected. A word-wrapped paragraph exceeding available width returns
`NeedsWrapping` with exact required/available values; no break policy is
invented. Fixed-width nonwrapping text retains horizontal overflow. These line
boxes are metric boxes, not measured ink bounds or native clipping evidence.

Current unknowns fail explicitly: automatic sizing, password fields, device
fonts, FontClass, missing saved layout/color/font defaults, justified text,
mixed/nested formats, unknown or malformed declarations/entities, nonzero HTML
letter spacing, disabled-kerning inheritance, control characters, complex
shaping, empty paragraph metrics, and nonopaque field-alpha composition. The
module does not execute style sheets, script mutation, locale selection,
selection/caret behavior, input editing, or text filtering. Runtime whitespace
configuration is not established by a saved initial definition.

## Actual corpus and exact font route

**OBSERVED:** login has 16 HTML definitions; character select has 20. They
contain 39 P/FONT paragraphs and only explicit letterSpacing `0.000000` and
kerning `1`. All FACE values equal their saved field FontID import symbols and
the English font library export symbols exactly:

| Face | Source local IDs | English font ID |
|---|---|---|
| `$Strat2Medium` | login 74; character 162 | 4 |
| `$Strat2Bold` | login 77; character 138 | 5 |

The private check loads the actual UPKs through `MovieLibrary` and explicitly
routes `MarvelFonts.gfxfontlib` to `MarvelFonts.fonts_en`. It checks exact source
movie bytes, target font movie bytes, and target export membership. Without
that route the existing library reports the missing logical movie; this module
does not silently install a language alias. These are verified saved metadata
and configured-resolution facts, not proof of native locale initialization.

The fonts each contain 378 glyphs, 20,480 EM units, ascent 18,440, descent 6,140,
and leading 4,100. Medium has 3,090 pairs; Bold has 2,871. Independent raw-byte
decoding verified codes, glyph/shape offsets, advances, bounds, and pair tables.
Both original bold/italic bits are clear, including the font whose name contains
Bold. All source characters, including U+2022, have exact glyphs.

Five definitions are deliberately excluded:

| Definition | Reason |
|---|---|
| login 75 | Two P tags with the saved multiline flag clear |
| character 163, 246, 262 | Saved wrapping enabled and measured paragraph wider than available width |
| character 311 | Saved alpha 127; initial alpha inheritance has not been traced |

The 31 successful definitions contain 94 nonzero pair adjustments. All six
positive-alpha login HTML instances in the prior preflight snapshot use
supported definitions (95, 213, 206, 200, and 198 twice); the two positive-alpha
character instances use 307 and 281, also supported. This is a snapshot-to-
definition coverage check, **not** evidence that retail currently shows those
instances or that they have been rendered by this module.

## Native REA observations

The `reverse-engineer-anything` skill was read before use. Only target-free,
bounded Rizin commands inspected Blizz's pinned executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No executable was launched, shared Ghidra target bound, configuration changed,
or native prompt accepted. The initial session-status query encountered an
unrelated apktool version-probe timeout and was not repeated. Other native
lanes confirmed that no shared engine binding was active.

**VERIFIED static observations:** the exact uppercase HTML table maps FONT
to tag ID 3. Its dispatch table leads to `0x1401777e3`. The FONT attribute
dispatch maps COLOR 17 to `0x140177873`, KERNING 24 to `0x14017790f`,
LETTERSPACING 27 to `0x140177956`, and SIZE 30 to `0x1401778d7`.

- COLOR parses a prefixed hex value, retains the current format's high alpha
  byte, and replaces only RGB. This does **not** yet establish which alpha the
  format inherits from a saved field; character 311 therefore stays rejected.
- A successful nonnegative KERNING integer sets the enabled bit for a positive
  value through `0x1400b9e10`. The zero path also changes a format-presence bit,
  so disabled-kerning inheritance is left unsupported.
- LETTERSPACING uses a float conversion, multiplies by the byte-verified
  constant 20.0, truncates to an integer, and writes a word. Only the actual
  zero value is implemented; signed overflow, rounding and nonzero placement
  semantics are not inferred from that storage operation.
- SIZE passes the nonnegative integer conversion to a formatting helper. Only
  the observed positive integral sizes are supported by this increment.

The three REA calls completed without truncation and reported the executable
unchanged. Parser disassembly comprises 1,601 instructions whose returned
bytes were independently matched to the original PE offsets. Jump tables,
attribute tables, and the scale constant were checked against the same file.
This establishes native declaration handlers; it does **not** recover native
glyph positioning, kerning arithmetic/rounding, paragraph spacing, or raster
output. Those layout relationships remain the explicitly documented metric
model, with raw-file arithmetic checked independently.

## Validation receipts and provenance

All paths below are relative to the private
`~/.local/share/marvel-heroes-forever-rust/evidence/` directory. No original
strings, font tables, binaries, or extracted geometry enter git.

- `html-layout-study/helper-expected-geometry.json`: independent raw-font/source
  model, SHA-256 `a1fdacc5d4ad842951f15ea3083b840bfe9e3c715a26f030d6b441560c1e5bc0`.
- `html-layout-study/rust-actual.json`: original Rust actual-package output,
  with `actual.rs` reproducer and explicit host font route.
- `html-layout-study/helper-rust-layout-comparison.json`: zero-mismatch receipt,
  SHA-256 `fb5a8f0284ede9673ff739aa5299f96e19b90b25b29a608c73c7e585366fda72`;
  `helper_compare_actual.py` reproduces comparisons. It scopes out metadata
  that the producer did not dump, including paragraph source boundaries,
  decoded UTF-16, local source FontIDs, and combined colors. Source preservation
  and entity mappings also have focused synthetic tests; those are not retail
  parity tests.
- `html-layout-native/rea-evidence.json`: deterministic bundle, three records,
  1,157,817 bytes; complete inline `native1.json`, `native2.json`, `native3.json`
  and independent discovery/decoding files accompany it. Provider identity
  probing was not performed; the tool explicitly reports that limitation.

Focused validation compiled the isolated module against this project's existing
library, then ran only `html_layout::tests::` (seven tests) and the actual 36-field
smoke. No full test suite, native client comparison, window/GPU smoke, or server
login was performed by this increment.

Outside sources read or reused for this increment (study only; all Rust and
independent checkers are original):

| Source | Licence and use |
|---|---|
| [Adobe TextField](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html) | Adobe copyrighted documentation; htmlText/P/FONT and multiline semantics, saved-text/input distinction. Private snapshot SHA `f1c35956775fe317c6b651edd22cdb793e3f3d51ca7cda081ca1dc77d64ea482`. |
| [Adobe TextFormat](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextFormat.html) | Adobe copyrighted documentation; kerning, character and paragraph properties, size and spacing units. Snapshot SHA `429039adc3537fe4a6c92a8ee9f6947e0ce1cacbbacce864a0e25d5a86b9789e`. |
| [Adobe TextLineMetrics](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextLineMetrics.html) | Adobe copyrighted documentation; gutter, ascent/descent/line-leading metric model. Snapshot SHA `03f4b1e0053e33cea8d23f4ced9995cafb36e0eb405514406fe6527280c9f548`. Pages carry copyright and linked CC BY-NC-SA 3.0 notices; no implementation reuse licence is assumed. |
| Owner's pinned `MarvelHeroesOmega.exe` under `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/` | Proprietary, owner-provided compatibility study. REA evidence and SHA above; no executable code copied. |
| Owner's `MarvelFrontEnd_SF.upk` under that pin and `MarvelFonts_SF.upk` under `pins/frontend-fonts/UnrealEngine3/MarvelGame/CookedPCConsole/` | Proprietary source assets; exact original field/font binding and geometry inputs. Existing package provenance in `GFX_HTML_TEXT.md` and `FRONTEND_FONTS.md`. |
| Private `upk-rust-samples/marvel_login_dialog.gfx`, `marvel_character_select.gfx`, and `frontend-fonts/MarvelFonts_SF-fonts_en.gfx` | Proprietary extracted originals. Movie SHA values respectively `56a39b5546161f3b96e96def43b81fba27103f20d8337079f778924d4904738f`, `a12781e8109d682a17dcfb11e622700f6ce33522a989e17b42b2a01137eb2034`, and `330b919bcbabe8612db9c2140a8f327a663362fd7e1aaa28a215c702cae7032a`. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/native-and-artifacts.md`, `references/connection-and-recovery.md` | Installed tooling instructions; licence not independently determined. Used for bounded static calls, timeout handling and Evidence retention; no implementation code reused. |

This increment also reads this project's original `html_text`, `edit_layout`,
font/glyph/library APIs and prior private receipts. It reads no outside text
engine implementation and changes only `src/gfx/html_layout.rs` and this file.
