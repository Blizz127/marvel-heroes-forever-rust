# Current frontend frame preparation

`frontend::current_scene::prepare` composes one existing `FrontendHost` display
and its admitted TextField state into an owned `PreparedScene`. It does not
create a native frontend, run scripts, acknowledge invalidation or alter the
saved-frame preparation path.

```rust
prepare(
    host: &FrontendHost,
    movies: BTreeMap<MovieId, &Movie>,
    assets: &AssetSet,
    library: &MovieLibrary,
    fields: &[runtime_text::FieldBinding],
    formats: &[runtime_text::FormatBinding],
    limits: current_scene::Limits,
) -> current_scene::Result<CurrentScene>
```

The caller supplies the source movies from the same library and VM/Scene
association as the attached host. `DisplayState::snapshot_scene` checks the
exact retained movie key set, versions, lengths and source-byte hashes. It
projects current transforms and effective visibility while retaining original
source placement, action, opaque and unbound-action metadata. The content
preparer independently checks the library entry's movie pointer identity.
Matching bytes alone therefore do not admit a different library association.
The supplied AssetSet must be the one used to construct that library; the
existing library API does not retain an AssetSet identity token, so this remains
a caller association contract rather than an additional check in this adapter.

For an attached TextFieldHost, the existing
`runtime_text::snapshot_overrides` validates supplied field identities,
definitions, domains, current documents, explicit format mappings and supported
style conditions. This adapter additionally requires coverage of **every bound
field**, including unchanged or hidden fields. Missing a field rejects the
whole preparation. Duplicate, unknown, mismatched or unsupported fields also
reject; they cannot fall back to authored initial text. Unbound scene text
continues through the existing saved-text path.

If there is no TextFieldHost, both field and format slices must be empty. All
text then follows saved preparation. An attached empty TextFieldHost remains
distinguishable from an absent host. Format registries and empty-document policy
are explicit caller inputs; this adapter supplies no effective style, fallback
font, native default or automatic format-handle conversion. Existing text
profile restrictions remain unchanged, including the admitted uniform plain
ASCII document and explicitly inactive side paths/effects.

The returned `CurrentScene` owns:

- `prepared`: the current CPU content preparation, retaining asset-specific
  limitations from the existing content stage;
- `display_revision` and `display_statistics`: captured display metadata;
- optional `text`: domain, every `FieldSnapshot`, text projection statistics and
  TextFieldHost statistics. Pending invalidation is retained unchanged.

Field snapshot order follows the caller's binding slice; use its explicit node
identity rather than treating it as a PreparedScene text-cache index.

The immutable host borrow covers the complete projection. Later host writes do
not update an already returned frame. The display revision alone is not a text
revision; text snapshots/statistics are separate metadata, not an automatic
cache invalidation protocol. No invalidation flag is cleared on success or
failure. A caller must establish a new domain when replacing the VM/Scene
association; numeric object IDs cannot prove that lifetime relationship.

`Limits` contains the existing runtime-text and content budgets. All bound
fields count against the text field limit. Existing hard ceilings, document and
provenance budgets apply during text projection; the content stage checks node,
geometry, glyph, bitmap and decoded-byte budgets. A missing display, disallowed
inputs without a TextFieldHost, incomplete coverage, display-source mismatch,
text error or content error returns a typed error without a partial frame.

## Focused validation

**VERIFIED:** four focused composition tests pass through real public
UPK→MovieLibrary→Scene→TextFieldHost→glyph preparation APIs. The test package,
Font2 glyph, two DefineEditText instances and VM associations are constructed
original test data. They are not retail assets or recovered native defaults.

The checks cover current display translation and per-instance current strings,
complete bound-field coverage, duplicate/unknown/foreign-domain fields, missing
format mappings, text limits, a wrong resolved font, immutable earlier frames,
pending invalidation and unchanged host traces/statistics during preparation.
They also exercise explicit empty-document policy, bound fields under a hidden
synthetic root, the no-TextFieldHost saved path, missing display/source movies,
and an identical-byte Movie from the wrong library association. The synthetic
visibility change uses the existing attached visibility transaction. No
original panel or original script is changed or executed by these tests.

The first run retained one failed test that attempted the intentionally
unsupported numeric Visible setter. Correcting that test trigger required no
adapter change. The retained failure is separate from the passing run; neither
a full suite nor GPU checks were run for this increment.

**OBSERVED original saved-login comparison:** with the retained explicit English
inspection alias, the original login scene has 147 nodes. A diagnostic VM root
association and unchanged DisplayState, with no TextFieldHost, produce the same
complete PreparedScene representation as `scene_content::prepare`: 25 mesh
entries, 21 cached text entries, two fonts, 56 glyph entries, 29 images and 29
textures. Both contain 123 actions, 48 opaque records and 889 unbound action
tags, 32,456 triangles, 207 glyph placements and 4,290,240 decoded RGBA bytes.

The comparison checks every public retained field through complete Debug
representation equality, including source nodes, geometry/index arrays,
glyph records and texture pixel payloads. That 23,831,069-byte representation
has SHA-256
`b3dad84dfb45a73ce9b8800a5fff2abe57a8abf57e6b9db19d830b1699cd49e7`.
Only its digest and counts are retained in the receipt. Display revision stays
zero, display statistics/traces are unchanged, and the optional text snapshot
is absent. All 95 frozen source/manifest/harness/input files have unchanged
hashes. The explicit alias is diagnostic input, not a selected runtime locale.
No original handler, native client, original visibility change, renderer or
network operation ran. This is composition correspondence, not an initialized
frontend or native/visual parity claim.

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/current-scene/`.
`check2/verification.json` records command exits, input/source hashes, full
comparison counts and scope; SHA-256
`6a4f098f38aa2949b090f3b59406b21415da6ed34803bb69361a6cda966404b5`.
`check2/focused-tests.log` and `original-login.log` retain the successful
commands without compiler warnings. `check1/failure-receipt.json` retains the
earlier test-trigger failure. The private composite adds only the new module
declaration and CPU comparison binary; the integrator owns production hooks.

## Evidence and provenance

This is original composition code. Its inputs are the existing original
`host`, `display_state`, `textfield_host`, `runtime_text`, `scene_content`,
`library`, UPK and GFX modules, and original AVM1 `vm`, `attached` and
`attached_textfield` APIs/tests. The domain, attachment and format limitations are
documented in [RUNTIME_TEXT_PREPARATION.md](RUNTIME_TEXT_PREPARATION.md) and
[TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md). No new external implementation,
web source or dependency was used.

The source baseline is `1952767b54a6bfbe2f787cf529dd2c312309015d` plus the
integrator's projection API commits `1987432` and `23c08bf` (the latter corrects
two synthetic test field spellings). The existing project implementation and
private diagnostic fixtures are original project work. Any original game
packages used for a separate CPU composition check remain proprietary and
private; no game bytes, captures or extracted source enter the repository.

The synthetic helper was written from this project's original UPK, object,
text, glyph and edit-layout readers/tests. Its private precursor is
`evidence/current-scene/synthetic-fixture.rs`, SHA-256
`7e529e841bf83ac45a92e0aa3c3f11c97f4cc4d31b2ec5146735c87c6ce6fce6`;
the source ledger is `synthetic-fixture-receipt.json`, SHA-256
`b30f1101eea6c58911846a44ff0577b6f50ce180a3da66926b009774c07e4175`.
The integrated test module omits unused helper entry points and formats this
original encoder with the surrounding tests.

The original CPU comparison reuses the package selection and explicit English
alias from the original private
`evidence/runtime-text-preparation/actual/src/scene_recipe.rs`; no former
runtime-field adoption or handler fixture was executed. Source inputs below
are proprietary owner assets for private interoperability study, all under
`~/.local/share/marvel-heroes-forever-rust/pins/`:

| Pin | SHA-256 | Use |
| --- | --- | --- |
| `alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFrontEnd_SF.upk` | `af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190` | Original login definitions and assets |
| `frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelGame.upk` | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` | Declared shared UI movie dependency |
| `frontend-fonts/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFonts_SF.upk` | `43b7f8576a487601e812607e1c18f3ced659629de22b0cf7c7ae97b601adf76e` | Original explicitly routed English font resources |
