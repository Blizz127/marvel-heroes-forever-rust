# Saved plain EditText layout foundation

`src/gfx/edit_layout.rs` lays out a bounded plain-text subset using the exact
caller-resolved original font. It preserves source characters, glyph identities,
UTF-16 indices, saved flags, metrics, colors, and field bounds. It does not
execute movie code, replace missing fonts/glyphs, or claim retail typography
parity.

## Interface and coordinate contract

- `layout_initial(&EditText, resolver)` reads the original initial string.
- `layout_plain(&EditText, &str, resolver)` accepts an explicit unformatted
  update for a future runtime setter. It does not strip or parse HTML.
- The callback returns `Result<glyphs::ResolvedFont, EditLayoutError>` for the
  field's local font ID. Import/locale resolution remains with the caller.
- The result is `EditTextLayout`, with glyph placements and line boxes in
  rational field-local twips. Each `Twips` value retains an integer numerator
  and positive denominator. `.as_f64()` is an explicit rendering conversion.
- `.position(glyph_index, [em_x, em_y])` scales an original integer glyph point
  and adds the saved metric baseline. `GlyphOutline` supplies the original
  outline/meshes; the caller then applies scene/display transforms and clipping.

Each glyph preserves its local font ID, resolved movie/character identity,
code/glyph index, original shape span, UTF-8 byte range, and UTF-16 code-unit
range. Saved strings also retain their absolute movie-byte spans. Runtime
updates have no invented movie source span. `content_kind` distinguishes saved
initial content from an explicit plain update, while `flags` retains the source
field flags.

## Supported subset

The current profile supports printable ASCII, selected precomposed Latin, Greek and
Cyrillic codepoint blocks, and selected punctuation/currency blocks. Every character
must occur exactly once in the resolved font's code table. Other script/shaping
requirements return `CharacterProfile`; a non-BMP character or absent glyph
returns a typed failure with the original UTF-16 index. No replacement box,
fallback font, normalization, or lossy text conversion is applied.

DefineFont2/3 layout metrics supply advances, ascent, and descent. Their
Unicode wide-code tables are required; legacy ANSI/Shift-JIS/narrow codepages
are rejected rather than interpreted as Unicode. Saved left,
right, and center alignment, margins, paragraph indentation, and signed leading
are supported. Kerning is disabled in this saved plain profile: DefineEditText
has no kerning switch, and Adobe's documented TextFormat default disables it.
Font kerning tables remain intact in the font reader for later explicit formats.

The placement model uses the documented two-pixel (40-twip) field gutter. The
first baseline is field top plus gutter plus scaled font ascent. Line height is
scaled ascent plus descent; subsequent baselines add the saved paragraph leading
in twips. FontLeading is not added again to that explicit spacing. These choices
are documented Flash semantics applied to GFx, **not verified native GFx pixel
behavior**. Native CSM, rounding, hinting, and glyph clipping may affect pixels.

CR, LF, and CRLF are represented as explicit line separators in multiline
fields; CRLF consumes two source UTF-16 units. The source string is not rewritten.
Only CR occurs in the inspected original plain text. CRLF/LF normalization,
multiline placement and nonzero margins/indent currently have synthetic coverage.
An explicit separator in a single-line field fails rather than becoming a space.

WordWrap fields are accepted when each explicit line already fits. A line that
would require automatic breaking returns `NeedsWrapping`, including source
UTF-16 range and required/available widths. No undocumented word-breaking rule
is approximated. AutoSize, justification, device fonts, FontClass, tabs, control
characters, and unsupported character profiles also return typed errors.

HTML initial fields return `Unsupported(Html)` even when their bytes happen to
look plain. Password fields return `Unsupported(Password)` through both entry
points; no plaintext glyph placements or guessed masking glyph are returned.
Read-only/selection/border flags and maximum input length are retained. Rendering
a border, editing, selection, scrolling, style sheets, and enforcing interactive
input limits are outside this module. Saved/script strings are not truncated to
MaxLength; Adobe documents that restriction as an input-entry limit.

## Original-asset coverage

**OBSERVED:** Across the original login movie, character-select movie, and
English font-library preview, all 88 EditText source definitions have explicit
FontID, height, and layout and select glyph outlines. None requests AutoSize,
FontClass, password masking, or justification. There are 52 plain definitions,
36 HTML definitions, and 45 nonempty plain initial strings. Plain content is
ASCII; character-select field 332 contains the only two carriage returns.

**OBSERVED:** Original-reader layout smoke succeeds for:

| Source definitions | Supported | Explicit gaps |
| --- | ---: | --- |
| Login | 32 | 16 HTML |
| Character select | 14 | 20 HTML; field 332 requires wrapping |
| English font preview | 5 | None |

These 51 supported definitions produce 553 original glyph placements and 51
line boxes. Representative actual glyphs also pass the existing outline/fill
tessellator, yielding 10,404 triangles with explicit EvenOdd fill and a
4-EM-unit tolerance. This is **source-definition coverage**, not proof that all
fields are instantiated or visible in a native login frame. Scene visibility
and runtime text updates are separate work.

The unsupported wrapped paragraph is field 332, UTF-16 range 18–150. The API
returns its exact required and available widths without discarding content or
laying out a partial field. Its actual multiline placement remains unrun.

All five English fonts serialize ascent+descent equal to 1.2001953125 EM.
Thirty-four supported definitions therefore have metric line boxes exceeding
the saved field height. `overflow_y` records this; no font size is reduced or
bound enlarged. This does not prove visible glyph pixels are clipped: metric
line boxes and drawn glyph extents are distinct. Callers retain the actual
TextBounds for their clipping implementation. Horizontal overflow is also
recorded for fixed-width, non-wrapping fields.

## Focused checks and private evidence

**VERIFIED:** Five focused tests pass for exact metric scale and source spans,
UTF-8/UTF-16 correspondence, alignment and line separators, explicit unsupported
content/password behavior, missing glyphs/fonts, wrapping-required errors,
inconsistent metrics, empty strings, zero height, and preserving saved strings
past an input-length limit.

Limits are one MiB of supplied text, 262,144 Unicode scalars, 65,536 lines, and
the font format's 16-bit glyph count. Source arithmetic, source ranges, fields,
font identities, units and advance counts are checked. Rational arithmetic
retains exact supported layout coordinates; float conversion is deferred.

Private evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `glyph-geometry/edit-research/research.json` and `saved-fields.jsonl`: original
  field inventory, flags, actual strings, Unicode/source statistics and hashes.
- `edit-layout/retail.tsv` and `.stderr`: each definition result and glyph
  identity/source/position, including every typed rejection.
- `edit-layout/english-metrics.json`: the actual font metrics used.
- `edit-layout/focused-tests.txt` and `validation.json`: focused checks, input
  and implementation hashes, scope and unrun native checks.
- `edit-layout/adobe-sources.json`: private documentation snapshot hashes.

No original strings, glyph data, font bytes, or movie data enter git or build
packages. This increment changes only the reader module and this document.

## Source ledger

1. Adobe, *SWF File Format Specification*, version 19,
   <https://open-flash.github.io/mirrors/swf-spec-19.pdf>, printed pages 171–173.
   © 2006–2012 Adobe, all rights reserved; field/flag/unit study only. Existing
   private PDF identity is recorded in `GFX_TEXT.md`.
2. Adobe ActionScript 3 API reference, [TextField](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html),
   [TextFormat](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextFormat.html),
   and [TextLineMetrics](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextLineMetrics.html),
   including its [metrics diagram](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/images/text-metrics.jpg),
   read 2026-11-15. Pages carry © 2015 Adobe, all rights reserved and a linked
   CC BY-NC-SA 3.0 notice; no source-code reuse licence is assumed. Used only
   for gutters, line metrics, default kerning/leading, text updates and input
   limits. No example implementation was copied. Native Scaleform equivalence
   is not established by these Flash documentation sources.
3. Blizz's private pinned original client/font movies and this project's
   original font/text/glyph readers. Proprietary assets remain private;
   paths/hashes are in the evidence above and `FRONTEND_FONTS.md`. Used for
   actual code-table resolution, fields, metrics and placement validation.

No other font or text-engine implementation was read or linked.
