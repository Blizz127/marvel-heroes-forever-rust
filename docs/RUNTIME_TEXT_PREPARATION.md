# Conditional TextField document preparation

`frontend::runtime_text` bridges an admitted `TextFieldHost` document to the
existing per-instance `scene_content::RuntimeTextOverride` path. It performs no
native format discovery, attachment, script execution, glyph layout or GPU work.
Opaque native-derived format handles remain caller-supplied identities; their
numeric values do not encode an inferred renderer style.

## Public inputs and outputs

```text
FieldBinding {
    field: AttachedTextFieldId,
    identity: AttachedIdentity, // domain, scene_node, avm_owner
    expected_definition: DefinitionRef,
    empty_style: EmptyDocumentStyle,
}
FormatBinding {
    domain: NonZeroU64,
    profile: FormatProfile, // both TextFormat and ParagraphFormat
    style: RuntimeTextStyle,
    provenance: String,
}
snapshot_overrides(host, scene, fields, formats, Limits) -> Result<TextBatch>
```

The output owns strings and snapshot metadata while borrowing the exact immutable
scene. Accessors expose `scene()`, `overrides()`, `snapshots()` and `statistics()`.
The overrides and snapshots align in caller-supplied field order. Pass the
associated scene and overrides together:

```rust,ignore
let batch = runtime_text::snapshot_overrides(host, scene, fields, formats, limits)?;
let prepared = scene_content::prepare_with_text_overrides(
    assets, library, batch.scene(), batch.overrides(), preparation_limits,
)?;
```

The caller must preserve the host's domain association with this VM/scene
lifetime, as required by `TextFieldHost`. Binding checks compare the full
attached identity, exact node/definition, original dictionary record and actual
DefineEditText tag. They cannot prove a native association solely from numeric
IDs. Keep the batch's snapshots alongside any diagnostic preparation receipt;
the renderer's override records alone do not retain all host metadata.

## Document and format selection

For a nonempty document, the bridge selects `snapshot.document.run_format`.
It copies `snapshot.document.text` exactly. `last_input`, the `htmlText`
serializer, authored initial strings and current defaults are not substitute
text/run sources. In particular, current defaults may differ from the current
run after a replacement or native snapshot adoption.

An empty document has no character run. Every field binding therefore requires
one of two policies, with no implicit default:

- `Reject` returns a typed empty-document error.
- `UseSuppliedCurrentDefaults` selects the exact current default handles as
  explicit conditional renderer inputs. The receipt labels this
  `ExplicitEmptyCurrentDefaults`; native empty-paragraph formatting remains
  unproved. The downstream layout still checks the supplied font and bounds.

The format registry must match both selected handles in the host domain.
Missing text/paragraph combinations and duplicate keys fail. Every mapping
supplies the complete `RuntimeTextStyle`, including exact local/resolved font
identities, RGBA, height, layout and flags. Its bounded nonblank provenance
identifies the caller's mapping evidence or conditional test assertion; this
module does not authenticate that evidence. Native target/view flags retained
in the receipt are not translated into EditFlags or effective style.

Kerning, letter spacing, underline and block indent must explicitly be inactive
under the current downstream plain-layout subset. `None` is unresolved and
fails. The host's six admitted absent side paths are also checked and retained:
watches, translation service, stylesheet, editor, variable binding and image
callbacks. The bridge does not silently disable an active effect or service.

## Instance isolation and snapshot meaning

The output names one structural node and exact definition per field. Duplicate
fields or nodes fail; shared definition identity never expands an override to
other placements. The scene-content adapter retains authored fields and source
spans, uses its separate runtime-instance path and leaves all unlisted instances
on their existing saved path. It validates the actual source font route and
reported resolved identity before producing glyph geometry.

Snapshot metadata retains selected and current profiles, attached identity,
original definition, conditional side-path profile, target/view flags and
pending invalidation. These records are a point-in-time observation of the
admitted host state. Later host mutations require another snapshot. Reading or
preparing a batch never acknowledges dirty flags, clears invalidation, reports
render completion or modifies host statistics/traces. No invalidation consumer
or acknowledgement API is introduced.

Successful snapshot construction establishes only a structurally admitted batch.
Actual preparation can still fail on a source font, glyph, bound, unsupported
layout or its separate budgets. There is no saved-string/font fallback on such
failure. No GPU/native layout or pixel-parity claim follows from CPU preparation.

## Bounds and failure behavior

Limits cover fields, format entries, total document bytes and provenance bytes.
Default ceilings are 4,096 fields/formats and 16 MiB for each byte budget;
configured maxima are 100,000 bindings and 64 MiB. Zero is a valid budget for
an intentionally empty selection. The provenance ceiling independently bounds
the supplied registry and emitted per-instance strings; each string is also
bounded by scene-content's 4,096-byte provenance ceiling. Document content must
fit the existing plain-layout bounds and admitted printable-ASCII/no-markup
subset. These are semantic/resource restrictions, not an exact RSS bound.

Identity, missing/duplicate bindings, unresolved effects, unsupported content,
empty policy and budgets return typed errors. No partially successful batch is
returned. Output vectors/strings use checked reservation; ordinary collection
allocation remains subject to Rust's process allocation behavior. Any failure
leaves the immutable host and source scene unchanged.

## Verification and provenance

Focused validation uses private original-asset fixtures under
`~/.local/share/marvel-heroes-forever-rust/evidence/runtime-text-preparation/actual/`.
The selected original login nodes are 89 and 136, both using EditText 1034 through
their distinct saved parent chains. Native attachment identities, format
handles, absent services and effective styles remain explicit diagnostic
fixture assertions; they are not inferred from these assets.

Code is original. Inputs are existing original `textfield_state`,
`textfield_host`, `scene_content`, scene/VM association code and the private
original-asset harnesses documented in `TEXTFIELD_RUNTIME.md` and
`FRONTEND_SCENE_CONTENT.md`. This increment reads no new external implementation
source and adds no dependency. Original assets and evidence stay private.
The integrator supplies the `frontend::runtime_text` module hook separately.
