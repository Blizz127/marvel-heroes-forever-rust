# Directed frontend fill tessellation

`gfx::tessellate` turns the decoded game shapes into triangles from their actual
edge geometry. The implementation is original, dependency-free Rust. It does not
replace shapes with bounds or assemble a guessed screen.

## API and rendering contract

`tessellate(&Shape, Options) -> io::Result<Mesh>` returns one `FillMesh` per used
style-bank/index pair. Each contains the original `FillStyle`, exact source-edge
identities and directions, closed flattened contours, `f64` positions in shape
twips and `u32` triangle indices. The fill's bitmap ID, gradient stops/modes and
raw matrix remain unchanged. Triangles have positive algebraic area; consumers
must account for their own screen-coordinate convention and transforms.

`fill_coordinates(&Fill, Position)` inverts the preserved SWF matrix. Bitmap
results are bitmap pixel coordinates; gradient results remain in the gradient's
native coordinate system. These are not normalized UVs. Singular matrices return
an error. Placement transforms, color transforms, texture binding/sampling and
gradient shading are renderer responsibilities.

A mesh reports the applied fill rule, whether its winding interpretation is
inferred, the count of unexpanded stroke edges, canceled internal edges and edges
without a fill. Original stroke data remains in `Shape`. Eight actual shapes have
only strokes and therefore intentionally produce no fill triangles. They must
not be represented as completed rendered shapes.

## Construction and bounds

Each fill graph uses immutable `(style bank, index)` identity. Left-side edges
keep their direction; right-side edges reverse. An edge with the same fill on
both sides cancels as an internal boundary. Exact integer endpoints must have
balanced incoming/outgoing degree. Deterministic closed walks consume every
remaining directed edge once; disconnected components remain separate, with no
endpoint welding or invented connecting edges. Source spans are recovered through
each contour's original edge indices.

Quadratics use bounded de Casteljau subdivision. A quadratic minus its endpoint
chord is `t(1-t)(2B-A-C)`; the norm of `2B-A-C` divided by four bounds that segment's
approximation error. Default tolerance is 0.25 shape twip, or 0.0125 unscaled
pixel. Placement scaling can magnify it, so the renderer should select tolerance
for its transformed scale when needed. Curve depth is limited and failure is
explicit instead of silently accepting an under-resolved segment.

The original triangulator divides the flattened paths into horizontal slabs at
vertices and segment intersections. Directed crossing counts select intervals
using even-odd or nonzero rules; each selected interval becomes at most two
triangles. Active Y intervals limit intersection candidates. This supports
holes, disjoint components, touching contours and self-intersections without
bridging holes. Geometry arithmetic uses `f64`, not exact predicates; numerical
behavior for adversarial near-degenerate input is not a native-parity claim.

Default budgets per shape are 250,000 flattened segments, 8,000,000 intersection
checks/crossing visits/curve or input visits, and 500,000 triangles. Curves allow
24 subdivision levels. Invalid tolerances, unresolved styles, unsupported shape
semantics, open graphs, budget overflow and nonfinite output fail explicitly.
The mesh is returned only after complete success.

The Shape4 nonzero-bit interpretation remains **INFERRED**, as documented in
GFX_SHAPES.md. `Options.fill_rule_override` permits an explicit caller selection;
otherwise the mesh exposes `inferred_winding`. All 79 observed Shape4 tags have
raw flags `1`, so nonzero behavior has synthetic coverage only.

## Focused verification

**VERIFIED:** Seven focused synthetic tests pass, covering same/opposite-oriented
holes under both fill rules, disjoint and concave components, a crossing bow-tie,
coincident edges, a touching branch, curved area convergence and reversal,
separate style banks, matrix inversion, strict closure and resource budgets.
Known triangle areas and representative hole/interior points are checked.

**OBSERVED:** All 676 shapes from the 20 frontend movies plus the startup manager
complete with default options. They produce 1,117 fill groups, 1,864 closed
contours, 29,097 flattened segments and 43,599 positive-area triangles. Total
measured work is 463,621 visits; the largest shape uses 366,427. This validates
static geometry production, not native pixels or script-controlled visibility.

| Movie | Shapes | Fill groups | Contours | Triangles | Unexpanded stroke edges |
| --- | ---: | ---: | ---: | ---: | ---: |
| `marvel_login_dialog` | 50 | 83 | 88 | 320 | 0 |
| `marvel_character_select` | 84 | 83 | 84 | 1,142 | 22 |
| `MarvelGame-frontend_manager` | 3 | 3 | 3 | 6 | 0 |

An independent original topology probe agrees that all graphs close exactly. It
records 423 same-fill internal edges, 272 edges without fills and 96 graphs with
multiple connected components. One graph branches at a vertex; closed-walk
extraction preserves it as two contours. The largest disconnected graph has 627
components. This is why reconstructing only one polygon per style is insufficient.

Private evidence under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `gfx-tessellation-smoke.rs` and `.tsv`: original harness and all 676 per-shape
  geometry/area/work receipts.
- `gfx-tessellation-evidence.json`: input identities, options and summarized
  receipts; hashes link to the existing shape/package extraction evidence.
- `gfx-tessellation-tests.txt`: focused test output.
- `gfx-contour-research.json`, `gfx-contour-research-details.tsv`,
  `gfx-contour-research-issues.txt`, `gfx-contour-research-summary.txt` and the
  adjacent original `.rs`: independent exact graph topology, including the
  branching vertex's source bit spans.

No native render comparison, stroke expansion, shader output, antialiasing,
scale9/clipping, font rendering or scripting was validated by this increment.

## Provenance

All algorithms and tests were authored here. No geometry library or game/runtime
implementation was copied. A cache lookup found no Lyon/Earcut crate, and no
package was downloaded. The integrator maintains the global PROVENANCE.md ledger.

| Source read | Licence / notice | Use |
| --- | --- | --- |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), MATRIX definition and shape/fill sections | ©2006–2012 Adobe, all rights reserved; document study only | Matrix coefficient convention and native fill coordinate semantics; directed fill-side interpretation already recorded in GFX_SHAPES.md. |
| Owner's private decoded GFX samples and extraction receipts | Proprietary game data; owner-authorized interoperability study | Real topology and complete static tessellation checks. All derived data remains private. |

The scanline implementation and quadratic error calculation were developed from
geometry directly, without reading a third-party implementation.
