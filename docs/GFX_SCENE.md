# Static GFX scene graph

`src/gfx/scene.rs` builds an original, bounded structural scene from existing
`Movie::static_frame` snapshots. It recursively instantiates each placed sprite
at frame zero. It does not execute ActionScript, select a frontend screen, decode
geometry, render pixels or establish retail behavior.

## Host API

```rust
let scene = gfx::scene::Scene::first_frame(
    gfx::scene::MovieRef::new(movie_index, &movie),
    &gfx::scene::NoImports,
    gfx::scene::Limits::default(),
)?;
```

`MovieId` and `NodeId` are `usize`. The host supplies stable movie IDs; reusing
an ID for a different `Movie` reference fails. The returned scene borrows all
registered movie data. Its nodes form a preorder arena, with a parent link and
children in ascending local display depth. IDs stay stable within this snapshot;
no identity persistence across rebuilds or mutable VM update protocol is claimed.

`Scene::path(node_id)` returns structured depth/name segments from root to
instance. Paths do not interpret names as filesystem paths, and unnamed or
identically named instances remain distinguishable by ancestry and depth.
Every node retains the original `DisplayObject`, placement movie ID and exact
local definition or import binding. Definitions retain character ID, kind,
tag index and declared bounds. The borrowed movie supplies the original spans.

`NodeKind` distinguishes roots, definitions, explicit import chains, missing
characters and class-only placements. `Unsupported` records mark recursive
sprites, empty timelines, uninstantiated buttons, non-display definitions,
invalid mask ranges and class bindings requiring execution. A missing character,
cyclic import or unsupported definition does not produce substitute geometry.

## Explicit import resolver

```rust
pub trait ImportResolver {
    fn resolve(
        &self,
        request: ImportRequest<'_>,
    ) -> std::io::Result<Option<ResolvedCharacter<'_>>>;
}
```

The request contains `from_movie`, `local_character_id`, the exact serialized
ImportAssets `url`, and the exact symbol name. A response contains a `MovieRef`
and character ID borrowed from the resolver. `NoImports` returns no binding.
The scene code performs no file lookup, URL retrieval, case folding, extension
substitution or global symbol search. Those decisions belong to an explicit host
library using its observed package references.

A resolved import remains an import node. Its complete chain and final definition
are retained. A missing target, unresolved binding or alias cycle remains explicit.
The scene expands a resolved sprite using the target movie's dictionary/timeline;
its child definitions and script spans retain that movie's identity.

## Transforms and presentation metadata

**OBSERVED (documented SWF fields):** matrix coefficients are signed 16.16,
translations are twips, and color multipliers use a 256 denominator. ClipDepth
covers greater display depths through its inclusive endpoint; zero disables
clipping. These field meanings are documented in Adobe's specification cited
below.

`Affine` stores `a,b,c,d,tx,ty` as `f64`, using
`x'=a*x+c*y+tx`, `y'=b*x+d*y+ty`. `compose(child)` applies the child first,
then the parent. The world transform stays in twips; viewport/stage-origin
mapping and division by 20 belong to the renderer.

`ColorTransformF64` composes multiplication and addition without clamping.
Addends retain 0..255 channel units, not normalized shader units. Each node
also preserves its raw local transform, so a renderer can retain the original
chain. Native intermediate clamping, quantization and shader color space are
unverified; the composed float value is not a pixel-parity assertion.

Effective visibility is the conjunction of serialized ancestor/local visibility.
Scripts, alpha and future animation can change what is actually visible. Raw
ratio, blend mode, filter spans, bitmap-cache mode, opaque-background values and
clip-action records remain attached. Filters, blend groups, morph interpolation,
bitmap caching and scale-nine transformation are not executed by this module.

Scaling grids retain their splitter rectangle and source tag. Grids attached to
local import aliases and the final target are all preserved; runtime precedence
between multiple grids is deliberately unresolved.

## Masks

Each node has `is_mask` and an ordered `masks` list naming ancestor and active
sibling mask nodes. Sibling intervals are independent: for A at depth 1 ending
at 4, and B at depth 2 ending at 6, depth 3 has `[A,B]`, depth 5 has `[B]`, and
depth 7 has neither. A mask's own geometry is not ordinary display geometry.
The lists describe structural applicability; no stencil/alpha coverage is made.

A renderer must rebuild/replay its active mask stack as these lists change.
It must not permanently bake A into B merely because B occurred while A was
active. This matters when intervals cross. Visibility and special compositing
behavior of masking layers still need retail runtime evidence.

## Unexecuted actions and opaque content

`ActionNode` records bind byte spans to an owner node. Frame actions,
serialized initialization declarations, candidate sprite initializers and clip
event handlers have distinct `ActionKind` variants. Their source movie IDs and
tag references (where present) are retained. An initializer association is a
structural relation to a definition, not proof of its runtime execution order;
an initialization declaration can appear separately from each instance binding.
No action is executed, interpreted as a scene command or silently treated as
completed.

`unbound_action_tags` retains action-tag references in registered movies which
have no binding in this snapshot. `opaque_nodes` records uninterpreted tags in
each visited first-frame prefix, including opaque script-bearing tags if present.
Clip handlers from uninstantiated definitions are not separately expanded; their
original placement records remain in the borrowed movie. Button definitions stay
explicitly uninstantiated pending integration with the independent button reader.

## Limits

Defaults: 100,000 nodes (including root), depth 64, 100,000 bound action records,
2,000,000 tag visits, 32 import hops and 64 MiB of accounted copied metadata.
Depth/import limits are capped at 256. Registration scans and first-frame walks
consume the tag budget; copying names, arrays and reference records consumes
metadata accounting. This is not an allocator-level peak-memory measurement:
the existing static-frame parser also has temporary allocations under its movie
input limits.

Sprite ancestry and import-chain identities detect cycles explicitly. Invalid
source spans, conflicting movie IDs, out-of-bounds timeline indexes, nonzero
frame indexes in a first-frame prefix and non-finite composed transforms fail.
Exceeding a resource budget returns an error without a successful partial scene.
No source movie or external file is changed.

## Focused verification

**VERIFIED:** five focused scene checks passed: nested skew/translation and color
composition, visibility/raw metadata/path retention; crossing/ancestor masks;
unexecuted actions/opaque tags/buttons; explicit import binding, alias grids,
cycles and conflicting IDs; sprite recursion, missing definitions and resource
budgets. Synthetic data covers these behaviors; it is not retail pixel evidence.
The private harness compiled the original module against a frozen copy of the
integrator's `gfx.rs` containing `DisplayObject::clip_actions`. The repository's
`gfx.rs`, `lib.rs` and Cargo files were not modified by this lane. The first
standalone test invocation needed a crate-root `gfx` wrapper for the existing
text module imports; that failed invocation is not counted as a test pass.

**VERIFIED (actual files, static only):** all 20 frontend movie samples plus
`MarvelGame-frontend_manager.gfx` produced first-frame scenes with `NoImports`.
The 21 streams total 5,334,735 bytes. Their snapshots contain 587 nodes, 515
resolved local definitions, 51 explicitly unresolved import instances, zero
missing local definitions, 1,615 unexecuted action bindings, 1,177 unbound action
tag references and 498 opaque-tag nodes. These first frames contain zero active
mask links; actual mask rendering remains unobserved.

| Movie | Scene nodes | Unresolved imports | Unexecuted action bindings |
| --- | ---: | ---: | ---: |
| frontend_manager | 6 | 0 | 461 |
| marvel_login_dialog | 109 | 15 | 110 |
| marvel_character_select | 205 | 23 | 122 |

No structural unsupported markers occurred in those snapshots. That does not
mean scripts, filters, button interactions or other runtime features were
implemented. No cross-movie import resolution was exercised on the actual files
in this lane; that belongs to the host library integration. No renderer, login,
mouse/keyboard control, clock progression, native capture or whole-client parity
check was run.

Private evidence under
`~/.local/share/marvel-heroes-forever-rust/evidence/gfx-scene/`:
`inputs.json` (all input paths/lengths/hashes), `source-snapshot.json`,
`tests.rs`, `tests.txt`, `smoke.rs`, and `smoke.tsv`. Samples remain under
`evidence/gfx-samples/` and `evidence/frontend-startup/`. No game bytes were added
to source or a distributable build.

## Sources and licences

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), matrix/color and display-list/clipping sections | ©2006–2012 Adobe, all rights reserved; document study only | Check field units, matrix orientation, color terms and clipping endpoints; no implementation copied |
| This project's `src/gfx.rs`, `src/gfx/text.rs`, `docs/GFX_FORMAT.md`, and original private `evidence/gfx-smoke.rs` | Original project code/documentation | Existing decoded types/static-frame interface and actual-file harness integration |
| Owner's private 20 frontend movie samples and frontend-manager sample, identified by `inputs.json` | Proprietary game data; authorized private interoperability study | Structural first-frame validation only; no payload redistribution |

No third-party parser/runtime implementation was read or copied. The source
and tests are original. These are static source/document/parser observations;
REA/native runtime evidence remains separately maintained by the integrator.
