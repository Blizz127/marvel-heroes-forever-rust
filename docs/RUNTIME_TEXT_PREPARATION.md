# Conditional TextField document preparation

`frontend::runtime_text` bridges an admitted `TextFieldHost` document to the
existing per-instance `scene_content::RuntimeTextOverride` path. It performs no
native format discovery, attachment, script execution, glyph layout or GPU work.
Opaque format handles remain caller-supplied identities; their
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

**VERIFIED focused CPU fixture:** the selected node's admitted document differs
from its last input, and its run profile differs from current defaults. The
bridge uses the document/run pair. All 32 other text instances, including node
136 sharing its definition, retain their saved fields/layout/glyph results.
The source scene/movie bytes, host snapshots/statistics, trace count and VM
storage identities remain unchanged by the snapshot calls and 20 typed
rejections. A later host replacement prepares `BTN` with three glyphs while
the previous batch stays frozen. Empty text first rejects without the explicit
policy, then prepares zero glyphs with pending invalidation still retained.

This controlled check compiled the exact module SHA256
`2d52cd193826a7be09723531a6a993163acb800ab95203dec4780b31f8b7f3f7`
on a frozen core source snapshot with only the private module hook. Its build
and the single test process both exited zero. The later committed integration
contains the same module. The sealed `actual/controlled-receipt.json` SHA256 is
`f9b8fafa0132f2cf1e9820ae49a05a08002486de3442ab86344d98b96ecdfb96`.
The receipt retains source/input hash checks and confirmed worker exit statuses.

**VERIFIED original saved-action continuation:** a separate source freeze at
root commit `7041d197cc63a002075aa239c04985954b73158f` executes the original
true-uppercase path: 142 instructions and three host calls. The String call
at source offset 94251 produces `LOG IN`, and the typed setter at 94252 stores
that live document. Backend/controller cache remains `Log In`. The bridge reads
the live host after this successful execution and prepares six glyph placements
for node 89. The other 32 text instances, source scene/movie bytes and host
snapshot metadata remain unchanged by preparation; invalidation remains pending.
The fixture does not construct replacement text from an expected-result string.
The effective style/attachment assertions remain conditional throughout.

Both the post-action build and its single worker exited zero. Its sealed
`actual/post-vm/receipt.json` SHA256 is
`c8241a887c4eca684f141c72ee6edfe1487e2ed07f0c6babbb876e20adf49c59`.
The receipt verifies 77 source files and 32 input files unchanged. No product
fix or additional broad suite was needed; neither fixture ran a GPU or the
native retail client.

The combined private `actual/HANDOFF.md` identifies the frozen CPU manifests,
binaries and source/input inventories. Its `handoff-receipt.json` SHA256 is
`39ba359fb2edf6f875cf3b6ea724e2902f79e0fa997e0c2a6376bd4f5ade14c3`.
For a separate GPU experiment, copy the harness into a new private directory
and retain the `PreparedScene` returned by `src/vm_bridge.rs::check`, called
immediately after successful execution in `src/bin/post_vm.rs`. Redirect that
copy's output paths before running it. Completed CPU sources, manifests,
receipts and binaries remain immutable.

Code is original. Inputs are existing original `textfield_state`,
`textfield_host`, `scene_content`, scene/VM association code and the private
original-asset harnesses documented in `TEXTFIELD_RUNTIME.md` and
`FRONTEND_SCENE_CONTENT.md`. This increment reads no new external implementation
source and adds no dependency. Original assets and evidence stay private.
The integrated library exposes `frontend::runtime_text`. The root window-feature
compile and the two scoped CPU document-to-glyph checks passed. Rendering and
native effective-format parity remain separate work.

## First whole-scene GPU observation

**OBSERVED diagnostic GPU execution, not a successful visibility check:** a new
private fixture reused committed source `7041d197cc63a002075aa239c04985954b73158f`
and the sealed post-VM CPU case. It rendered the returned PreparedScene and saved
baseline through one software Vulkan renderer with the existing explicit policy.
Product/source assets were unchanged; effective format/font mapping stayed supplied.

Both outputs are 1920 by 1200 with all 33 text instances prepared. Saved baseline
pixels match the retained diagnostic exactly. Node 89 changes from eleven fills /
2,266 triangles to five fills / 1,062 triangles; the space in six glyph placements
has no painted fill. Total glyph fills change from 303 to 297. Nevertheless, final
RGBA pixels are identical. The assertion requiring visible change fails with exit
101 and remains recorded as a failure.

The selected field's transformed bounds are pixels `[835, 612, 1095, 637]`.
Visual inspection shows overlapping authored modal layers. Occlusion is an
explanation under investigation, not yet proof of the exact responsible draw.
No visibility flag, asset, scene order or native state was changed to pass the
check. Actual startup and visibility/lifecycle execution remain required.

The private fixture selected `ordered-float` 5.5.1 instead of the product's
5.3.0; all other registry identities match the root lock. This first run remains
sealed with that difference. A later comparison must use a new scope and the
product dependency versions. The product build retains its own lock/notices.

Artifacts: `evidence/runtime-text-gpu/`. `run-receipt.json` SHA256:
`ab73f2d50e14544a048c670da7319c57083ce0960556f216dd0b13d67e6c7dd7`.
All 77 frozen source files, 32 prior inputs and ten copied harness files were
checked unchanged before/after execution. The harness manifest was sealed after
compilation began but before execution; it is not a pre-compilation snapshot.
Product sources were separately verified before compilation. Installed Pillow
13.0.0 (metadata licence MIT-CMU) encoded unchanged RGBA to PNG for inspection;
it is a private tool, neither linked nor packaged with the client.
