# Frontend scene content preparation

`frontend::scene_content::prepare(&assets, &library, &scene, limits)` prepares
owned CPU data for the supplied structural scene. It uses original shape edges,
existing tessellation, declared texture bindings and supported saved plain/HTML
EditText glyphs. It makes no GPU calls,
executes no scripts and invents no replacement or bounding-box geometry.

## Owned caches and instances

`PreparedScene` owns `nodes`, `meshes`, `texts`, `fonts`, `glyphs`, `images`, `textures`, `actions`,
`opaque_nodes`, `unbound_action_tags`, `stats` and the tessellation options.
Each prepared node retains its complete cloned `SceneNode` as `instance`,
including raw placement, composed transforms, visibility, ordered masks and
source identity. `content` identifies a group, shape cache index, bitmap cache
index, original text definition or explicitly unrendered content. `limitations`
retains effects or behaviors not prepared by this stage. `PreparedNode.text`
indexes a text-cache record while the existing `NodeContent::Text { definition }`
retains its original identity.

| Cache | Identity | Retained data |
| --- | --- | --- |
| `meshes` | Resolved `(movie_id, character_id)` | `DefinitionRef`, optional original `Shape`, optional `tessellate::Mesh`, per-fill image indices and preparation issues |
| `texts` | Resolved `(movie_id, character_id)` | Original EditText, exact font binding, optional saved geometry, full HTML source/layout and route proof, typed plain/HTML failures and glyph placements |
| `fonts` | Exact resolved font `(movie_id, character_id)` | Original FontDefinition and parse issues |
| `glyphs` | Exact FontIdentity plus glyph index | Original GlyphOutline and optional neutral-white mesh in font EM units |
| `images` | Local movie bitmap character ID, before import resolution | Exact import chain, resolved definition, existing `ImageBinding`, logical dimensions, texture index, explicit unverified padding policy and issues |
| `textures` | `AssetLocation { package, export }` | Original descriptor including optional SRGB property, mip index, physical mip dimensions, optional decoded RGBA and issues |

Indices are stable within one preparation and repeated instances reuse their
cache entries. A failed asset occupies a cache entry with an explicit issue,
preventing repeated attempts and preserving the reason. No cached error becomes
substitute pixels or geometry. The returned data owns its meshes, paints and
pixels; action byte spans and source IDs still refer to the host's movie library
if later inspection or execution is needed.

`MeshAsset.fill_images` aligns exactly with `mesh.fills`. A bitmap paint has an
image-cache index; solid and gradient paints have `None`. Fill matrices, stops,
repeat/smoothing flags and directed-edge provenance remain in the original
shape/mesh types. Stroke definitions remain in `Shape`; unexpanded stroke counts
and inferred winding are explicit issues/statistics. This bridge does not convert
world transforms to screen pixels or normalize fill coordinates.

## Saved plain text and geometry

`TextAsset` preserves the original `EditText` body, bounds, flags, saved color,
variable name and initial text. For plain fields, `layout` is the original
`EditTextLayout` from the unchanged `edit_layout::layout_initial` path; it is
always `SavedInitial`, never a localization,
script update, inferred runtime string or populated input control. Its source
spans, UTF-8/UTF-16 indices, rational pens, baselines, advances, line boxes and
original font/glyph identity remain available to the caller. Repeated placements
of one text definition reuse one text cache record.

Font imports follow only `MovieLibrary::binding`. Each `FontBinding` keeps
its local ID, complete import chain and resolved font-cache index. Missing
aliases, symbols, definitions, cycles and unsupported font formats remain
explicit issues; there is no global symbol search, filename guess or fallback.
An explicit caller-selected English inspection alias can resolve the saved
`gfxfontlib` imports, but this bridge never selects that alias. Native locale
routing evidence and its remaining limits belong to `REA_NATIVE_ALTERNATIVE.md`
and `MOVIE_LIBRARY.md`.

`TextAsset.layout_error` retains the typed `EditLayoutError`, separately from
`issues`. Password, automatic sizing, device fonts, wrapping-needed cases
and the layout module's other unsupported features produce no glyph placements.
Even an empty password field is rejected before content layout. Static DefineText
records are still explicit unsupported text entries in this bridge; their
separate glyph/layout implementation is not implicitly invoked.

For each accepted field, `TextAsset.glyphs` aligns with `layout.glyphs`:

- `layout_index` selects the exact rational placement and source metadata.
- `glyph` indexes the shared `PreparedScene.glyphs` cache.
- `local_matrix` scales original font EM coordinates by `height_twips / units_per_em`
  and adds the field-local rational pen/baseline converted to f64.
- `color: Option<[u8; 4]>` keeps the saved paint. Missing color remains unresolved.

The adapter composes `node.world_matrix.compose(local_matrix)`, then an explicit
stage-to-pixel transform. No `/20` conversion, bounds-origin shift, hinting,
kerning guess or vertical flip is inserted by the bridge. The exact rational
layout remains available even though tessellated curve points and the final
matrix use floating point.

Cached glyph meshes use neutral white fill so that a glyph can be shared by
fields of different colors and sizes. The adapter must replace that neutral
paint with each placement's saved RGBA **before** applying the scene's composed
color transform. It must skip and report an absent color. Neutral white is an
internal cache representation, not an inferred saved text color.

`Limits.glyph_fill_rule` explicitly selects the glyph tessellation policy
(default EvenOdd); `PreparedScene.glyph_fill_rule` records it. The shape winding
override does not silently change this policy. The shared curve tolerance is in
EM units when used for glyphs, and in shape twips when used for shape meshes.
Native fill/rasterization, hinting and antialiasing parity remain unverified.

This stage does not apply field-bounds clipping or draw an optional EditText
border. It preserves `TextBoundsClippingNotApplied`, `TextBorderNotPrepared`,
`FontRasterizationUnverified` and `SavedTextStateOnly` limitations for the adapter.
A laid-out empty string is a successful zero-glyph layout. A failed glyph remains
a referenced cache entry with decode/tessellation issues; it does not become a
rectangle, replacement character or substitute font. Consumers must inspect
text layout errors, font-binding issues, font issues and glyph issues as well as
ordinary shape/texture preparation issues.

## Source-preserving saved HTML

HTML fields use `gfx::html_layout::layout_initial`, with its documented and
observed P/FONT subset, rather than the plain-text routine. `TextAsset.layout`
receives an exact clone of `HtmlLayout.geometry`, preserving the existing GPU
contract. `TextAsset.html_layout` separately owns the complete markup, defaults,
tokens, elements, atoms, runs, issues and paragraphs, including source ranges,
alignment, exact font sizes, explicit kerning declarations and adjustments.
Glyph source byte/UTF-16 coordinates address the original markup; no stripped
string, inserted paragraph newline or invented movie span is created.

`TextAsset.html_font_route` retains `source_import_symbol`,
`target_export_symbol` and `resolved_font`. The proof follows the same
`FontBinding.import_chain` that supplied the decoded font. Every hop must match
the library binding at its exact `(movie, local character)` identity, resolve
through an existing target movie, and select that symbol's single exact
ExportAssets character. The final pair must equal the decoded FontIdentity.
The source and final target symbols are passed unchanged to `HtmlFont`; FACE
equality preserves case and `$`. A local embedded font without a declared
import/export route is explicitly unsupported here. A matching name elsewhere
in the library is never a fallback or a font-resolution proof.

The HTML module owns paragraph layout, explicit kerning and paint semantics.
The bridge does not apply kerning a second time: each glyph's rational X is
already positioned, and its advance already includes the adjustment after that
glyph. The existing glyph cache remains keyed by exact font identity/index.
`TextGlyph.local_matrix` uses that glyph's own height, pen and baseline;
`TextGlyph.color` receives its resolved source RGBA unchanged. Plain fields keep
their previous routine and paint path, including their previous kerning policy.

The current HTML subset requires explicit supported declarations and opaque
saved field alpha; ambiguous nonopaque-alpha composition, required wrapping,
unsupported paragraph/field combinations, font routes or glyphs produce no
replacement geometry. The authoritative restrictions are in
`docs/HTML_TEXT_LAYOUT.md`. Those rules are not broadened by the bridge.

`TextAsset.html_layout_error` retains the typed `HtmlLayoutError` separately
from the plain `layout_error`. Each recoverable HTML failure also produces a
`PreparationIssue` at `PreparationStage::HtmlLayout`, so the unchanged GPU
adapter reports the exact reason through its existing SourceText diagnostic.
The original field and font-binding issues remain available even when layout
fails. Parser `Limit` failures are fatal preparation limits, rather than
successful scenes with silently skipped excessive input. All HTML glyphs use
the existing shared placement, font, outline and triangle budgets.

Accepted HTML fields retain `SavedTextStateOnly`, field-bound clipping and
font-rasterization limitations. Retaining source markup/metric geometry does
not establish runtime localization, script updates, native glyph pixels or
complete HTML support. No GPU adapter changes or dependencies are required.

## Image resolution and sampling boundaries

Imported bitmap IDs follow `MovieLibrary::binding`, which uses the host's saved
References and exact ExportAssets symbols. The bridge follows explicit resolved
IDs, detects cycles, and preserves each `SymbolBinding`. It does not guess a
movie filename, search all symbols globally, fetch a URL or open a path from
movie data.

At the final ExternalImage definition, the existing `AssetSet::bind_images`
provides the declared Texture2D association. The bridge retains its `NameBasis`
and complete `ImageResolution`; it does not upgrade the existing empirical
image-name association to a native runtime fact. Embedded SWF bitmaps without
an ExternalImage/Texture2D bridge remain explicit unsupported content.

Logical image width/height are distinct from descriptor size and the selected
mip's physical width/height. Mip zero is selected explicitly. No lower-mip
fallback, resize, crop or padding guess occurs. `UvPaddingPolicy::Unverified`
remains attached even when logical and physical dimensions happen to match.
The GPU adapter must choose and report its diagnostic sampling policy separately.

Texture decoding retains raw channel values. It applies no gamma transfer or
alpha premultiplication. `Descriptor::srgb` stays `Option<bool>`: an absent
property is not an assumed default. External TFC, separate-range or individually
compressed mips unsupported by `decode_mip` remain texture issues; this bridge
does not probe cache paths or substitute another mip.

## Explicit incomplete work

Preparation preserves filters, non-normal blend modes, scaling grids, bitmap
cache requests, opaque backgrounds, morph ratios, scene unsupported markers,
bound actions and all unbound/opaque script records. Those behaviors are not
silently called implemented. Unsupported text nodes retain their real
`DefinitionRef`, typed layout error or decode issue, and `TextNotPrepared`.
Prepared saved plain/HTML text retains the limitations described above.
Direct bitmap instances retain image bindings but have
`DirectBitmapGeometryNotPrepared`, because this module does not manufacture a
quad or use a definition's bounds as a visual substitute.

Hidden instances are preserved and prepared from their actual definitions.
Visibility, clipping and draw order remain the renderer's responsibility. A
successful preparation means that its bounded traversal completed, not that the
scene is fully rendered, interactive, native-equivalent or safe to call a retail
login screen. There is no overall `complete=true` claim.

## Budgets and errors

Defaults: 100,000 nodes, 4,096 mesh records, 16,384 bitmap bindings, 4,096 texture
records, 2,000,000 triangles, 256 MiB decoded RGBA, 64 MiB selected shape bodies,
2,000,000 shape records/edges and 32 bitmap import hops. Import hops are capped
at 256. Text adds 4,096 text records, 128 fonts, 16,384 glyph meshes, 16 MiB
of text tag bodies, 64 MiB of font tag bodies, 2,000,000 glyph records/edges and
1,000,000 cached glyph placements. Before layout, source scalar count is a
conservative placement-budget preflight (line separators and original HTML
syntax/attributes also count there).
Existing shape/texture/text/font/tessellation input and work bounds also apply.
Triangle counts cover both unique shape and glyph meshes; placement counts cover
unique cached layouts, not repeated display instances. Decoded bytes cover unique
cached textures. The total RGBA budget is checked before inline pixel allocation.

Source identity conflicts, invalid tessellation options and budget exhaustion
return an error without a successful partial result. Recoverable asset-specific
parse, tessellation, binding or decoding failures remain explicit
`PreparationIssue` records with a stage and error kind. The existing tessellator's
budget diagnostics are treated as fatal limits, rather than skipped geometry.
The caches do not mutate the supplied AssetSet, MovieLibrary or Scene.

## Initial shape/image actual-file smoke

**VERIFIED (CPU preparation only):** the private smoke loaded the selected
frontend, manager and font archives, created a `MovieLibrary` without logical
aliases, constructed each requested first-frame Scene using that library, and
prepared the actual login, character-select and manager content.

| Actual movie | Scene nodes | Unique ready meshes | Triangles | Decoded textures | RGBA bytes | Retained node limitations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| marvel_login_dialog | 147 | 25 | 132 | 29 | 4,290,240 | 114 |
| marvel_character_select | 264 | 29 | 72 | 22 | 45,544,960 | 103 |
| frontend_manager | 6 | 1 | 2 | 1 | 16,384 | 5 |

All three preparations had zero asset preparation issues. No selected shape had
stroke edges in these snapshots. That earlier bridge retained 33 unprepared text nodes in login and
17 in character-select; scripts and other per-node limitations remain
explicit. Login's used imported bitmap `movie 13 / character 71` resolved through
the exact library binding to `!UISource`, `movie 19 / character 44`, and decoded
its referenced texture. These movie numbers are local indices of the retained
library load, not global game IDs.

**VERIFIED (binding-only probes):** both declared imported bitmap characters
were separately prepared from their actual library bindings: movie 4 / ID 60
resolved to movie 19 / ID 162 with logical 30×17 and physical 32×20 dimensions
(2,560 decoded bytes); movie 13 / ID 71 resolved to movie 19 / ID 44 at 64×64
(16,384 decoded bytes). The probe constructs a minimal reference-only scene
instance to exercise binding, without geometry or a runtime-placement claim.
The dimension mismatch survives unchanged and the padding policy stays
unverified. Its source and transcript are `bitmap-probe.rs` and
`bitmap-probe.tsv` in the same evidence directory.

The smoke verified unique cache keys, every mesh/fill/image/texture index,
matching retained script/opaque records and preservation of every scene node.
Zero-node, zero-mesh, zero-triangle and zero-RGBA limits rejected the actual scenes
without producing successful partial preparations. The module compiled against
a frozen original-project source snapshot, with a private module hook only.
No root `frontend.rs`, `library.rs`, Cargo or renderer source was edited here.
No GPU output, UI interaction, native execution or retail parity check was run.

Private evidence under
`~/.local/share/marvel-heroes-forever-rust/evidence/scene-content/`:
`smoke.rs`, `smoke.tsv`, `issues.txt`, `source-snapshot.json` and
`verification.json`. Original client packages and decoded image bytes stay
private; neither is a repository or distributable asset.

## Saved plain EditText checkpoint

**VERIFIED (CPU preparation only):** the extension smoke used the same three
private packages and first-frame snapshots, first without logical aliases and
then with caller-supplied `MarvelFonts.gfxfontlib -> MarvelFonts.fonts_en`.
This is an explicit English diagnostic selection, not native locale detection.

| Actual movie | Text instances / definitions | Ready saved layouts | Unique ready glyph meshes | Cached / instanced glyph placements | Combined shape + glyph triangles | Typed layout failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| marvel_login_dialog | 33 / 21 | 16 | 51 | 142 / 243 | 30,406 | 5 HTML |
| marvel_character_select | 17 / 17 | 10 | 39 | 101 / 101 | 23,874 | 6 HTML, 1 wrapping required |
| frontend_manager | 0 / 0 | 0 | 0 | 0 / 0 | 2 | 0 |

Each populated snapshot used two exact resolved fonts. There were no glyph,
font, or text asset-preparation issues beyond the separately retained typed
layout failures. Every cached glyph mesh was ready; empty/space geometry remains
an empty original glyph, without substitute quads. Without a caller font route,
no text layout succeeded: login retained 16 unresolved font bindings plus its
five HTML failures; character select retained 11 unresolved bindings plus its
six HTML failures. The route was never guessed.

The smoke checked exact cache identities, every text/glyph index, original saved
color and font identity, preserved action/opaque records, and affine placement
against `EditTextLayout::position` at an integer EM probe point for every placed
glyph. The focused unit test independently checked fractional baselines and EM
scaling before a rotated/scaled world transform. Its one selected test passed;
no routine full suite was run.

Zero text/font/glyph-cache, text-byte/font-byte, glyph-record and
placement budgets rejected the actual login scene. Zero font import hops and a
combined triangle ceiling equal to the earlier shape-only total also rejected
preparation; glyph work cannot escape the shared triangle budget. All smoke
commands exited successfully. Password rejection remains covered by the existing
layout module's typed policy, not an observed password flag in these selected
first-frame fields.

All 33 login and 17 character-select text instances retained
`effective_visible=true` from the serialized structural snapshot. This is not a
claim about initialized or script-controlled visibility. No VM update,
localization, input, native render comparison, GPU result or retail UI behavior
was tested in this extension.

Private evidence: `evidence/scene-edittext/{smoke.rs,smoke.tsv,details.txt,`
`focused-test.txt,source-snapshot.json,verification.json}`. The receipt pins the
actual input hashes, exact original-project source snapshot, compiled smoke and
focused test. The snapshot supplies existing glyph/layout module hooks privately;
this increment changes only this document and `src/frontend/scene_content.rs`.

## Saved HTML bridge validation

**VERIFIED (original files, CPU preparation and offscreen GPU):** a focused
private fixture compared the preceding plain-text bridge with this extension,
using the same original login/character scenes, three pinned packages and explicit
English alias above. The renderer-feature project build passed with offline
dependencies, two compiler jobs and lowered priority. No GPU source changed.

| Saved scene | HTML definitions / instances | Supported HTML layouts | New unique / instanced glyph placements | Submitted nonempty text glyphs, before → after | Changed full-frame pixels |
| --- | ---: | ---: | ---: | ---: | ---: |
| Login | 5 / 6 | 5 | 65 / 75 | 232 → 303 | 482 |
| Character select | 6 / 6 | 4 | 51 / 51 | 92 → 139 | 0 |

An independent exact-arithmetic expectation model matched all 116 supported HTML
glyph records from nine definitions, including rational placement, original
markup spans, exact font identity and saved color. Both character fields requiring
wrapping retained typed errors. Omitting the explicit font alias produced
unresolved bindings without fallback. A separately selected original field 311
retained its unsupported nonopaque-alpha error. Text source-byte and glyph-placement
ceilings rejected the actual login scene rather than returning partial success.

The frozen comparison preserved plain layouts and errors for 16 login definitions
(142 cached placements) and 11 character definitions (101 placements), including
font bindings and glyph mesh geometry. Cache ordinals were compared by their exact
font/glyph identities because new HTML entries legitimately change cache ordering.
For the isolated plain-pixel check only, HTML instances were hidden explicitly;
both GPU frames were byte-identical to the earlier plain-text captures. Full-frame
checks retained all saved visibility, color and placement unchanged.

**VERIFIED (saved character state):** HTML fields 307 and 281 draw before node 77,
whose two original triangles cover the stage. Its original bitmap has alpha 255
throughout and its composed RGB multiplier is zero with alpha multiplier one.
The later supported HTML fields 260 and 248 have composed alpha zero. The other
two HTML fields require wrapping. Thus the saved full frame remains opaque black
despite the additional submitted glyphs. This is a check of the saved snapshot;
initialized/script-controlled visibility and intended retail presentation remain
unverified. No colors or visibility were changed to make text appear.

**OBSERVED (private capture inspection):** the login capture contains the newly
prepared source text alongside the existing saved placeholder labels and overlapping
panels. The lossless PNG is only a container for the original RGBA readback. This
does not establish startup execution, usable login or retail rasterization parity.

Private evidence is under `evidence/scene-html-checks/`:
`validation.json` combines the exact source hashes and completed checks;
`result.json`, `expected/comparison.json`, `actual-html-glyphs.tsv`,
`actual-html-errors.tsv` and `character-source-visibility.txt` retain the details.
The first private fixture attempt incorrectly required new HTML to change every
scene's visible pixels. Its failed character assertion is preserved in `attempt1/`;
the later source-backed visibility check corrected that assertion without changing
product code or saved scene state. The passed login GPU result was retained rather
than repeated. A prior private fixture compile issue involved relocated module
paths only and is retained in its build log. The actual project build receipt is
`evidence/scene-html-build.log`.

At 1920×1200 the raw login HTML frame has SHA-256
`67ca5b733220f7a71992ee8f024f75f0b35ec6da1d83804f404a15757dee6ae9`;
the unchanged character frame has SHA-256
`42355e4ebb230ebc5c3522380404efa7d0cd6e4aaec13e9f5cc44cf381239b35`.
Captures and extracted source data remain private. No full suite, native window
test, server connection, VM localization or retail pixel comparison was run for
this bridge increment.

## Sources and provenance

The saved text extensions also read original project `docs/HTML_TEXT_LAYOUT.md`,
`docs/MOVIE_LIBRARY.md` and
`docs/REA_NATIVE_ALTERNATIVE.md` for the explicit font-route boundary. It does not
claim new native observations from those records.

This is original glue around the project's original modules; no third-party
implementation was studied or copied for this increment.

| Input read | Ownership / licence | Use |
| --- | --- | --- |
| Project `src/frontend.rs`, `src/frontend/library.rs`, `src/gfx/{scene,shapes,tessellate,text,glyphs,edit_layout,html_text,html_layout}.rs`, `src/texture.rs`, `src/texture_cache.rs`, `src/upk.rs` and module declarations | Original project code; underlying source studies retain their existing PROVENANCE entries | Existing APIs, explicit identity, metadata, decode and budget behavior |
| Private `evidence/frontend-import-library-smoke.json` and original `evidence/frontend-movie-dependencies.rs` | Original project findings/harness | Select the already established private package inputs and lookup identities |
| Private pins `alienware-candidate/.../MarvelFrontEnd_SF.upk`, `frontend-startup/.../MarvelGame.upk`, `frontend-fonts/.../MarvelFonts_SF.upk` | Proprietary game files, owner-authorized compatibility study | Actual scene/asset preparation and limits checks only |

The full private paths and input hashes are recorded in the verification receipt.
No new external documentation, SDK, network source or proprietary implementation
was accessed. Source findings come from static parsing, with the explicitly scoped
offscreen checks above; no new REA/native-client execution is claimed. The
integrator maintains the separate native evidence.
