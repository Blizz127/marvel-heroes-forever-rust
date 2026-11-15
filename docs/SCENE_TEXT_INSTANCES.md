# Explicit text content for individual scene instances

`frontend::scene_content::prepare_with_text_overrides` prepares caller-supplied
runtime text for exact structural instances. Two placements of one saved
DefineEditText can now have different strings and layout profiles. The original
`prepare()` API delegates an empty override list and retains its saved-definition
cache behavior.

This is an **original preparation API**, not native attachment or text execution.
It does not resolve `TextFieldState` format handles, run setters or AVM1, select
native defaults, or establish retail typography. The caller must supply the
effective style and identify how it was resolved. That provenance is retained as
a caller assertion; its text is not interpreted as proof.

## Input and identity

```text
prepare_with_text_overrides(assets, library, scene, overrides, limits)
    -> io::Result<PreparedScene>

RuntimeTextOverride {
    node, expected_definition, text, style, provenance
}
```

`node` is an exact `NodeId` in the supplied structural scene.
`expected_definition` is the complete `DefinitionRef`, including movie,
character, tag, kind, and bounds. Before resource preparation, admission rejects
missing nodes, duplicate node overrides, a differing definition, non-text
targets, dictionary disagreements, and definitions whose actual tag is not
DefineEditText (37). A source definition alone cannot select an instance.

`RuntimeTextStyle` supplies every supported layout input explicitly:

| Field | Contract |
| --- | --- |
| `flags: EditFlags` | Effective layout flags in the existing layout API's representation; **not** native TextField target bits. Font, color, and layout presence must be explicit. |
| `font: RuntimeFont` | Exact source movie/local font ID plus expected resolved `FontIdentity`. The existing declared library route must resolve to that exact identity. |
| `height_twips`, `color_rgba`, `layout_twips: EditLayout` | Explicit size, paint, alignment, margins, indentation, and leading. No absent-value defaults are selected. |
| `effects: RuntimeTextEffects` | Known inactive kerning, underline, letter spacing, and block indent. `None` or an active/nonzero value returns an error. |

The current effects profile requires `Some(false)` for kerning/underline and
`Some(0)` for letter spacing/block indent because `layout_plain` does not
implement those effects. Exact font identity supplies the selected glyph face;
the adapter does not synthesize bold/italic glyphs or select substitute fonts.
The remaining layout engine restrictions still apply, including unsupported
device fonts, FontClass, password, auto-size, justification, absent glyphs,
unsupported metrics, and lines requiring automatic wrapping.

The text profile is empty or printable ASCII without `<` or `&`. Markup,
entities, CR/LF, NUL, tabs, other controls, and non-ASCII remain errors. This
restriction deliberately matches the current conditional text-state increment;
there is no HTML stripping or string coercion here. An empty string is valid,
retains its exact instance/font/style identity, and yields no character glyphs.

`provenance` is a nonempty caller string, at most 4096 bytes. It should identify
the evidence or explicit diagnostic policy used to resolve the style. There is
no implicit mapping from opaque native format handles to saved SWF style values.
See [TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md) for that separate gate.

## Saved source, caches, and runtime ranges

Each override creates a distinct `TextAsset` with `runtime: Some(input)`. It is
never entered in the saved-definition text cache. Non-overridden nodes continue
to share their saved text asset by movie/character definition. Font definitions
and glyph outlines/meshes retain the existing caches, so instance strings do not
duplicate those resource assets unnecessarily.

`TextAsset.field` always holds the parsed original DefineEditText. Its initial
text, source spans, source flags, body identity, and bounds are unchanged. A
temporary layout input receives only the supplied effective style; the original
initial text and its spans are never overwritten, including in that temporary
input. `layout_plain` consumes the runtime string separately.

The resulting layout has `TextContentKind::PlainUpdate`. Text glyph and line
ranges index the runtime string, and their `movie_bytes` are `None`; they never
claim that new letters occupy the placeholder's original movie bytes. Original
font glyph shape spans remain valid source evidence. Field identity and bounds
remain tied to the original definition, so the existing preview adapter can
retain its field/layout identity checks and clipping source.

Runtime nodes report `SuppliedRuntimeTextState` instead of `SavedTextStateOnly`.
This is not evidence of executed native scripts. Existing placement, transforms,
visibility, masks, filters, border limitations, and glyph-rasterization
limitations remain present. The API does not alter the structural scene.

## Errors and bounds

An invalid override returns `Err`; preparation never silently uses the saved
placeholder, a replacement glyph, or another font. Unresolved font routes,
different resolved font identity, font decode failure, unsupported layout, and
glyph preparation failure all reject the requested preparation. Saved assets
keep their previous explicit issue-cache behavior when there is no override.

The existing node/text/font/glyph/triangle budgets still apply. Admission bounds
override count, per-string layout byte/scalar limits, provenance length,
aggregate runtime bytes, and potential glyph placements before layout. Actual
preparation charges the saved definition bytes plus retained runtime text and
provenance, and continues the existing font/glyph/triangle accounting. Distinct
instance text assets consume distinct text slots; shared font/glyph caches charge
their source assets only when first prepared. Budget failure returns no partial
`PreparedScene`.

## Sources and validation scope

All implementation is original. Existing project sources studied for the API
were `src/frontend/scene_content.rs`, `library.rs`, `preview.rs`, `display_state.rs`,
`src/gfx/edit_layout.rs`, `html_layout.rs`, `html_text.rs`, `text.rs`, `glyphs.rs`,
and `scene.rs`, together with [EDIT_TEXT_LAYOUT.md](EDIT_TEXT_LAYOUT.md).
They supply original resource identity, cache, layout, and range contracts.
No SDK or outside renderer implementation was copied or used to supply native
format conversion. The original owned UI/font assets used for the focused
check remain proprietary and private under
`~/.local/share/marvel-heroes-forever-rust/`; no asset or extracted payload enters
git.

The focused fixture uses the saved login scene's two real placements of UISource
EditText 1034: node 89 at
`loginContainer_mc/loginButton_mc/localizedText_tf`, and node 136 at
`compatLevelWarning_mc/m_yesButton/localizedText_tf`. Both resolve to movie 19,
character 1034, body `224143+41`. Font reference `19:270` follows the declared
`$Strat2Bold` route to movie 22, character 5. The fixture supplies the existing
explicit English diagnostic alias for `gfxfontlib`; that alias remains a caller
choice, not newly established native locale policy.

Private input inventory `evidence/scene-text-instances/source-and-inputs.json`
records full paths, sizes, licences, current hashes, and frozen original-code
inputs. Selected owned inputs are:

| Input | SHA256 |
| --- | --- |
| `MarvelFrontEnd_SF.upk` | `af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190` |
| `MarvelGame.upk` | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` |
| `MarvelFonts_SF.upk` | `43b7f8576a487601e812607e1c18f3ced659629de22b0cf7c7ae97b601adf76e` |
| `!uisource.gfx` | `49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5` |
| `MarvelFonts_SF-fonts_en.gfx` | `330b919bcbabe8612db9c2140a8f327a663362fd7e1aaa28a215c702cae7032a` |

No extra placement or font glyph is invented for this fixture. Any explicit
metric effects chosen for its runtime style are labeled diagnostic policy,
separately from actual saved font/color/layout values.

**VERIFIED focused CPU preparation on those actual files:**

- One override at node 89 leaves all 32 other text instances unchanged,
  including saved sibling 136. Its saved field, layout, and full glyph geometry
  compare equal by resource identity rather than incidental cache index.
- Two overrides (`LABEL` and an empty string) create separate runtime text
  assets with five and zero glyphs. The remaining 31 text instances are unchanged.
- Both cases retain exactly the baseline font/glyph key sets: two fonts and 56
  glyph cache entries. Original scene-instance metadata, saved fields, movie
  bytes, and all five input file hashes remain unchanged.
- All 15 narrow rejection cases pass: duplicate/missing/wrong/non-text targets,
  wrong/missing font identities, password style, NUL, absent provenance,
  unresolved/active effects, and byte/glyph/node budgets.

The private frozen-module harness compiles and runs successfully with `rustc`.
Focused crate checks also pass with and without the `renderer` feature
(`nice -n 19 cargo check --lib --features renderer -j2` for the latter).
`evidence/scene-text-instances/verification.json` has SHA256
`bd1ee3cf6166b976180bd232eaef1434721a9c730c164ce347a7f24cc5cce68d`;
it pins the unchanged source-under-test hash
`8941aca5bc9246318735058de29351f7199586912b736529adb02e5bce27aefa`,
input hashes before/after, commands, exact font route, source snapshots, and the
independent placement receipt. Runtime glyph/line ranges are explicitly checked
to have no fabricated movie-byte spans. Invalid API inputs are synthetic tests
over actual assets, not evidence of native error semantics.

No native/AVM1 execution, live initialization, state-frame transition, or GPU
image comparison was run. The focused harness is CPU-only; a separate executable
offscreen renderer harness was not built or invoked for this increment. Actual
saved placements and source geometry are established, not native runtime
visibility, kerning defaults, or rendered parity.
