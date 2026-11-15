# Original glyph geometry and saved static text placement

`src/gfx/glyphs.rs` turns retained font SHAPE bytes into integer outlines and
provides saved static-text placement metadata. It uses the actual caller-selected
font definition and original movie bytes. It does not substitute a system font,
execute movie code, or implement editable-text/HTML layout.

## Outline and mesh interface

`GlyphOutline::from_font(&FontDefinition, glyph_index, movie_bytes)` decodes the
selected glyph. The result preserves font ID/tag, glyph index, raw character
code, source span, EM scale, ordered style/edge records, and absolute bit spans.
Edges reuse `gfx::shapes::{Edge,Geometry,Point}`. Their coordinates here are
**font EM units**, not the twips normally carried by those shared shape types.

Straight edges and quadratic controls retain checked signed integer positions.
Moves establish absolute glyph positions. Fill selections retain their original
left/right roles; only the font's implicit fill slot 1 and the empty slot 0 are
supported. New style arrays and nonzero stroke selections fail explicitly.
Saved font-layout bounds are retained separately from the conservative hull of
decoded anchors/control points. The hull is not claimed to be a tight curve box.

`outline.tessellate(Color, FillRule, tessellate::Options)` returns `GlyphMesh`,
retaining identity and the existing `tessellate::Mesh`. Its `mesh.fills` contain
the renderer's existing `FillMesh` type. Positions remain in EM units, so the
curve tolerance is also in EM units. The caller explicitly selects fill policy;
conflicting policy overrides are rejected. A private geometric adapter supplies
one implicit solid fill to the existing tessellator. Its zero-length style spans
mean that no fill-style bytes were serialized in the glyph; no DefineShape tag
is invented or exposed as an original resource.

The renderer can borrow these fill meshes directly. Its draw transform must
compose EM-to-twip scale, saved glyph pen position and TextMatrix, display-list
placement, and target pixel scaling. No implicit division by 20 occurs in this
module or in the mesh renderer.

## Saved static text

`layout_static(&StaticText, resolver)` returns `StaticLayout`. Its callback
returns `ResolvedFont { identity: FontIdentity { movie_id, character_id },
definition: &FontDefinition }`, resolving a text-local font ID to
the caller's exact movie/character identity and `FontDefinition`. This keeps
import resolution and locale routing with `MovieLibrary`; it does not search
by a font's display name. Glyph indices select their original font table entries.

Each placement retains its original record/entry identity, referenced local font
ID, resolved font identity, glyph index/code, glyph source span, signed pen and
advance, text height, color, EM units, and the saved TextMatrix. Serialized
advances determine static spacing; font advance/kerning tables are not applied
again. A new explicit X/Y value updates the pen; inherited style fields retain
their previous values.

The position helper requires an explicit text-origin policy. Adobe's prose
describes X/Y offsets relative to TextBounds, while the supplied original text
objects strongly support text-local coordinates. No origin default hides that
distinction. Both policies preserve the original metadata; native pixel parity
has not been tested. TextMatrix uses signed 16.16 coefficients and twip
translations. The helper applies scale and pen position before that matrix;
`StaticLayout::position(placement_index, [em_x, em_y], TextOrigin)` returns
`ExactPosition` with signed `i128` rational twip coordinates. Its `.twips()`
conversion supplies floats only when a consumer requests them. The layout
resolver caches each local ID; aliases sharing one resolved identity must
return the same cached definition instance. Unknown color remains `None`.

## Actual asset evidence

**OBSERVED:** All 3,197 glyphs in the original English font library and
`!uisource.gfx` decode into 69,781 exact line/quadratic edges. This includes the
actual Stratum2 and Roboto resources identified in `FRONTEND_FONTS.md`.
Twenty-eight representative original glyphs tessellate into 10,228 triangles
with an explicit EvenOdd policy and a 4-EM-unit curve tolerance. The selected
characters include spaces, A/B/O/g, and available symbol characters; these
results establish structural and geometric processing, not native antialiasing
or rasterization parity.

**OBSERVED:** The 13 saved DefineText objects occur in `!uisource.gfx` and refer
to its local font ID 846, Segoe UI Symbol. They contain 26 glyph entries; these
static objects do not require an imported font alias. All 26 saved placements
also tessellate into 7,466 triangles with their original colors and the same
explicit fill/tolerance policy. Their TextMatrix X translations are 1,333–1,358 twips, while TextBounds X minima are 1,348–1,374.
Adding those bounds minima would introduce another approximately 1,350-twip
shift. **OBSERVED:** all 26 decoded glyph control hulls horizontally overlap
their saved TextBounds with TextLocal positioning. All 26 BoundsRelative hulls
lie to the right, separated by 1,157.77–1,289.13 twips. **INFERRED:** TextLocal
fits these saved objects; this does not establish every movie or native pixel
behavior. Exact saved data and both coordinate evaluations remain private.
No `$Strat2Regular` substitution is introduced for the unresolved options-movie
imports reported by the library integrator.

## Bounds, validation, and limitations

Outline limits default to 16 MiB per glyph and one million records. Source
ranges must lie within the font definition; checked arithmetic covers source
bit positions and accumulated coordinates. The decoder consumes the glyph's
exact terminator/padding and cross-checks decoded counts against the earlier
font metadata. Tessellation keeps its independent work/segment/triangle limits.

**VERIFIED:** Ten focused tests pass. They cover signed lines and quadratic
deltas, absolute source spans,
implicit paint, a known square area, empty glyphs, stale metadata, truncation,
invalid indices, source/record limits, and arithmetic overflow. Saved-text tests
and actual placement checks are recorded with the private receipt.

Private evidence directory:
`~/.local/share/marvel-heroes-forever-rust/evidence/glyph-geometry/`.
`outline-smoke.tsv` and `.stderr` retain actual outline/mesh results;
`static-smoke.tsv` and `.stderr` retain all 26 saved glyph placements and
their mesh results. `focused-tests.txt` retains the combined test result.
`layout/` retains independent saved-placement observations and focused checks.
Input source hashes are already recorded in `frontend-fonts/font-movies.json` and
`gfx-fonts/validation.json`; `glyph-geometry/validation.json` identifies its exact
input subset and parser source identity.

Not implemented or verified here: editable-text shaping and wrapping, HTML,
font alias policy, native hinting/CSM, subpixel antialiasing, device fonts,
interactive editing, or pixel comparison with the retail renderer. Explicit
fill and text-origin policies remain visible to callers.

## Source ledger

1. Adobe Systems Incorporated, *SWF File Format Specification*, version 19,
   © 2006–2012 Adobe, all rights reserved; documentation study only. Source:
   <https://open-flash.github.io/mirrors/swf-spec-19.pdf>, read 2026-11-15.
   Printed shape pages 123–130 and font/text pages 153–170 informed record
   deltas, EM scale, and saved text semantics. The private document snapshot and
   SHA-256 are recorded in `GFX_TEXT.md`. No outside implementation was copied.
2. Blizz's original proprietary Marvel Heroes font/frontend movie bytes,
   retained only under the private evidence/pin directory. Paths and hashes are
   recorded in `FRONTEND_FONTS.md`, `GFX_TEXT.md`, and the receipts above. Used
   for actual glyph geometry and saved-coordinate validation. No font/movie
   bytes enter git or build packages.

The project's original shape types, fill tessellator, font/text readers, and
renderer interface were consulted and reused. No third-party font engine or
font substitution source was read or linked.

The isolated original static-layout helper was integrated from commit
`42cc8bb2f832a5ee7cacd8cfbd72d4e401c84a72`; its source receipt is retained in
the private layout evidence directory.
