# Frontend scene content preparation

`frontend::scene_content::prepare(&assets, &library, &scene, limits)` prepares
owned CPU data for the supplied structural scene. It uses original shape edges,
existing tessellation and declared texture bindings. It makes no GPU calls,
executes no scripts and invents no replacement or bounding-box geometry.

## Owned caches and instances

`PreparedScene` owns `nodes`, `meshes`, `images`, `textures`, `actions`,
`opaque_nodes`, `unbound_action_tags`, `stats` and the tessellation options.
Each prepared node retains its complete cloned `SceneNode` as `instance`,
including raw placement, composed transforms, visibility, ordered masks and
source identity. `content` identifies a group, shape cache index, bitmap cache
index, original text definition or explicitly unrendered content. `limitations`
retains effects or behaviors not prepared by this stage.

| Cache | Identity | Retained data |
| --- | --- | --- |
| `meshes` | Resolved `(movie_id, character_id)` | `DefinitionRef`, optional original `Shape`, optional `tessellate::Mesh`, per-fill image indices and preparation issues |
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
silently called implemented. Text nodes retain their real `DefinitionRef` and
`TextNotPrepared`; the independent glyph/layout modules can extend this bridge.
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
at 256. Existing shape/texture/tessellation input and work bounds also apply.
Triangle counts cover unique cached meshes; decoded bytes cover unique cached
textures. The total RGBA budget is checked before inline pixel allocation.

Source identity conflicts, invalid tessellation options and budget exhaustion
return an error without a successful partial result. Recoverable asset-specific
parse, tessellation, binding or decoding failures remain explicit
`PreparationIssue` records with a stage and error kind. The existing tessellator's
budget diagnostics are treated as fatal limits, rather than skipped geometry.
The caches do not mutate the supplied AssetSet, MovieLibrary or Scene.

## Actual-file smoke

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
stroke edges in these snapshots. Login retains 33 unprepared text nodes and
character-select retains 17; scripts and other per-node limitations remain
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

## Sources and provenance

This is original glue around the project's original modules; no third-party
implementation was studied or copied for this increment.

| Input read | Ownership / licence | Use |
| --- | --- | --- |
| Project `src/frontend.rs`, `src/frontend/library.rs`, `src/gfx/{scene,shapes,tessellate}.rs`, `src/texture.rs`, `src/texture_cache.rs`, `src/upk.rs` and module declarations | Original project code; underlying source studies retain their existing PROVENANCE entries | Existing APIs, explicit identity, metadata, decode and budget behavior |
| Private `evidence/frontend-import-library-smoke.json` and original `evidence/frontend-movie-dependencies.rs` | Original project findings/harness | Select the already established private package inputs and lookup identities |
| Private pins `alienware-candidate/.../MarvelFrontEnd_SF.upk`, `frontend-startup/.../MarvelGame.upk`, `frontend-fonts/.../MarvelFonts_SF.upk` | Proprietary game files, owner-authorized compatibility study | Actual scene/asset preparation and limits checks only |

The full private paths and input hashes are recorded in the verification receipt.
No new external documentation, SDK, network source or proprietary implementation
was accessed. These are static-parser observations, not REA/native execution;
the integrator maintains the separate native evidence.
