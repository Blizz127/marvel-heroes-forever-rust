# Static frontend shape geometry

`gfx::shapes` decodes the owner's actual DefineShape/2/3/4 bodies into an
original Rust semantic model. It does not draw an approximate layout or execute
Flash. This increment supplies geometry for later contour assembly,
tessellation, rasterization and external-image binding.

## API

- `Shape::parse(tag_code, body, body_offset)` and `parse_with_limits` accept tags
  2, 22, 32 or 83. `parse_movie_tag(&Movie, tag_index)` retrieves the original span.
- `Shape.edges` retains every directed straight or quadratic segment, with signed
  twip endpoints/control points, selected left/right fill and line, and absolute
  source bit range. No curves are flattened and no edge is reversed or discarded.
- `Shape.styles` holds immutable banks of fill and line definitions. A `StyleRef`
  has a bank and one-based index; `Shape::fill` and `Shape::line` resolve it.
- `Fill` preserves solid RGBA, gradient matrix/stops/modes/focal value, or bitmap
  ID/matrix/repeat/smoothing. Matrices use signed fixed-point values without
  floating-point conversion. Bitmap IDs require the movie's own dictionary and
  reference-constrained external-image mapping, not a global filename guess.
- `LineStyle` preserves width, cap/join/scaling/pixel-hint/closure flags, miter
  limit, and color or fill paint. Style and style-bank byte spans are absolute in
  the movie. `Shape.records` preserves edge ordering, moves, style changes,
  replacement banks and EndShape; moves are absolute shape coordinates.
- `Shape.bounds`, optional `edge_bounds`, raw Shape4 flags and `fill_rule` retain
  shape header information. Reserved/invalid semantic values remain explicit
  `unsupported` entries with exact bit ranges; renderers must inspect them.

No ownership of texture, renderer, Cargo or application files changed.

## Semantics and limits

**OBSERVED in the primary specification:** Fill0 belongs to an edge's left side;
Fill1 belongs to its right. Selectors in a style-change record precede replacement
arrays. The decoder resolves those selectors against the preceding bank.

**INFERRED:** Already selected styles retain their definitions after replacement
until another selector changes them. This is represented explicitly by immutable
bank references; native behavior has not yet verified the inference. Shape4
replacement banks are accepted as the later shape version; the older spec table
names only Shape2/3. Mapping Shape4's winding bit to nonzero filling is also
inferred. Its raw bit remains available; no winding-dependent pixels are claimed.

Unknown fill layouts fail explicitly because their lengths cannot safely be
inferred. Reserved caps/joins/flags, gradient modes/order/count/profile violations
and focal range violations are preserved and reported. Input truncation, invalid
style indices, coordinate/source arithmetic overflow, forbidden Shape1 new
styles, missing EndShape and trailing bytes fail. Padding is skipped for alignment
without assigning it a rendering meaning. Default budgets are 16 MiB, one million
records and 131,072 aggregate styles; callers can reduce these.

Morph shapes, font glyph SHAPE records, contour reconstruction, self-intersection
handling, fill-rule rasterization, stroke expansion, scale9, filters, clipping,
blend modes, fonts and scripts are outside this increment. Successful decoding is
not a claim of rendered or native-client parity.

## Validation and evidence

**VERIFIED:** Nine focused synthetic checks cover signed straight/curved edges,
left/right selection, absolute moves, old/new style-bank selection, complete
matrix fields, gradient and bitmap modes, LineStyle2 fields, extended counts,
reserved/legacy semantics, every byte-truncated prefix of a representative shape,
trailing data, malformed references and resource limits.

**OBSERVED:** Every one of the 673 shape tags from all 20 privately extracted
`MarvelFrontEnd_SF` movies consumes its exact body, with no unsupported semantic
entries. The inventory comprises 462 DefineShape, 57 DefineShape2, 75 DefineShape3
and 79 DefineShape4 tags. It contains 10,643 edges, including 3,543 quadratic
curves; 247 solid, 977 bitmap and 114 gradient fill definitions; 147 line styles;
and 311 replacement style banks. These are definitions, not on-screen counts.

| Movie | Shapes | Edges | Quadratic curves | Bitmap fills | Gradient fills | Replacement banks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `marvel_login_dialog` | 50 | 371 | 4 | 93 | 3 | 26 |
| `marvel_character_select` | 84 | 386 | 24 | 78 | 0 | 3 |
| `MarvelGame-frontend_manager` | 3 | 12 | 0 | 1 | 0 | 0 |

The startup manager is a separately extracted 2,055,748-byte GFX movie from
`MarvelGame.upk`; its three shapes also consume exactly, making 676 parsed shapes
across the 21 tested movies. This does not establish which script-controlled
objects are visible at startup.

Private evidence under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `gfx-shape-smoke.tsv` and original `gfx-shape-smoke.rs`: per-movie counts.
- `gfx-shape-detail.tsv`: every shape's tag index, code, ID, offset, length,
  geometry counts, bank/record counts and raw flags.
- `gfx-shape-evidence.json`: movie and individual body SHA-256 identities with
  the complete 676-shape receipt. Extracted data remains private.
- `gfx-shape-tests.txt`: focused test output.
- `gfx-samples/`, `frontend-startup/MarvelGame-frontend_manager.gfx`: private
  samples, cross-referenced to the package/extraction receipts in GFX_FORMAT.md
  and the UPK lane's startup manifests.

The input identities and parsing results are static observations. This helper
performed no REA or native execution; those project-wide receipts are maintained
by the integrator.

## Source and licence ledger

All code and synthetic inputs were authored here; no implementation was copied.
The integrator mirrors these rows into `docs/PROVENANCE.md`.

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), shape/gradient chapters, printed pp. 118–136 and basic types | ©2006–2012 Adobe, all rights reserved; document study only | Binary field grammar, coordinate/fill-side semantics and validation constraints. Official Adobe PDF URL is unavailable; this is the authored document mirror already recorded in GFX_FORMAT.md. |
| Adobe [ActionScript 3 Developer Guide](https://help.adobe.com/en_US/as3/dev/as3_devguide.pdf), winding-rules discussion, read by the format-reference helper | Adobe documentation; no open-source licence established | Background meaning of winding rules only; does not prove the SWF Shape4 bit mapping. |
| Owner's private GFX samples and UPK-lane extraction manifests, paths above | Proprietary game data, owner-authorized interoperability study | Real-profile decoding and identity receipts; no sample bytes committed or packaged. |

No proprietary SDK source was accessed.
