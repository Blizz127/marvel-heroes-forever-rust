# Explicit saved ModelComponent binding

`frontend::model_component_binding::prepare` composes one explicitly selected
Level, Model and complete ordered component list from an `AssetSet`. It checks
their saved links and maps each component/element node index to its retained
Model Nodes record span. This prepares saved relationships; it does not
initialize components, resolve materials, build geometry or draw a scene.

The diagnostic command `mhf upk model-binding <package.upk> <1-based-level>
<1-based-model> [1-based-component ...]` exposes this exact selection contract.
An empty component list is explicit; omission is rejected when the saved Level
contains components. The command prints retained counts without discovering
missing selections or evaluating materials/geometry.

## Request and archive identity

```rust,ignore
use marvel_heroes_forever::frontend::{AssetLocation, model_component_binding};

let components: Vec<_> = (13..23)
    .map(|export| AssetLocation { package: 0, export })
    .collect();
let binding = model_component_binding::prepare(
    &assets,
    model_component_binding::Request {
        level: AssetLocation { package: 0, export: 10 },
        model: AssetLocation { package: 0, export: 12 },
        components: &components,
    },
    model_component_binding::Limits::default(),
)?;
```

This example explicitly selects the owned FrontEndMap's references 11, 13 and
14–23. `AssetLocation` uses **zero-based** package and export indices; serialized
positive references use **one-based** export indices. The function neither
discovers this selection nor chooses the runtime current Level. Existing
`startup_map::prepare_world` can report the saved World/PersistentLevel route;
it is not implicitly invoked and does not prove a runtime map transition.

Every selected location must use the same archive slot. All bytes are read
through that slot's `PackageArchive`, which already binds its tables to the
stored bytes. The result retains the `AssetSet`'s private `Arc` identity.
`binding.matches_assets(&assets)` remains true when that set is moved and false
for a separately created set containing identical package bytes. Names, paths
and GUID equality cannot substitute for this namespace identity.

`Binding` exposes immutable getters. `level()` and `model()` return
`RetainedExport<T>` values containing their locations, logical serial offsets,
complete original export bytes and parsed objects. `components()` returns the
ordered `ComponentBinding` slice, including each original Level array link,
retained component export and node mappings. The owned result preserves its
bytes and namespace token if the original set is dropped; later archive lookup
still requires a matching live set.

## Checked relationships

**VERIFIED (composition contract):** all selected exports pass their existing
868/3 exact Engine class readers. Then:

1. The Level's `LevelLinkSlot::Model` reference equals the explicitly selected
   Model reference.
2. The complete `Level.model_components` list equals the explicit component
   locations in length and order, including repeated references.
3. Each component's export outer equals the selected Level, and its saved
   `model` reference equals the selected Model.
4. Every unsigned index in both the component's node list and every element's
   node list is less than the selected Model Nodes count. Its mapped record span
   comes from `ModelBulkArray::record_span`, which also checks record bounds.

These are coherent-selection checks. A failed selection is not evidence that
retail rejects the entire source package. Null or imported Level Model/component
links cannot match this complete local-export request. They are not skipped,
substituted or loaded automatically. Empty component lists are supported when
the Level's saved list is empty and its selected Model link is valid.

The existing component reader independently rejects unsupported preamble modes
and nonzero polymorphic element headers. The binding preserves that frontier.
It introduces no condition requiring:

- `Model.outer` to equal the selected Level: the inspected map has this shape,
  but the reviewed native proof does not establish it as a universal invariant.
- A component's `ComponentIndex` to equal each occurrence's array ordinal.
- Element owner references to equal the component itself.
- Component and element node lists to be equal, unique, sorted or a partition
  of the Model's Nodes array.

All saved order, duplicate references/indices, material import references, raw
words, unknown fields and opaque tails remain in their original reader objects
and bytes. No material class, renderer use or default value is inferred from a
reference. Reference-to-record mapping does not interpret the 64-byte Nodes
records as finished vertices, surfaces or draw commands.

## Bounds and errors

Default aggregate inspection limits are 4,096 component slots, 32 MiB of selected
serialized exports, 65,536 top-level saved property tags, 65,536 elements and
262,144 node-index entries. Repeated selections and duplicate indices consume
these budgets again. Per-export limits in the existing readers also apply.

The complete selected-byte total and all export locations are checked before
payload reads. Element and node mapping budgets are checked before their output
vectors are reserved. The node budget counts both component and element lists.
Budget exhaustion returns `Unsupported`; inconsistent selections and node
bounds return `InvalidData`. Existing archive and class-reader errors propagate.

`usage()` reports the checked totals. `model_link()` retains the original Level
link and span; `model_nodes()` retains the selected Model bulk descriptor.
Each `NodeBinding` retains its raw index, the two-byte source span relative to
its component export, and the target record span relative to the Model export.
Add the respective retained export's serial offset to obtain logical package
positions. Those spans are never offsets into the other object's bytes.

## Focused validation

**VERIFIED:** all four new binding tests passed in a private composite preserving
the original `AssetSet` implementation and its private identity semantics.
Existing ModelComponent tests were filtered out. The focused build and both
actual-map runs exited zero; no full suite or retail runtime was run.

The tests exercise repeated component links and node indices, unequal node
lists, a non-self element reference, raw words/import references/opaque tails,
wrong selected links and outers, both node-list bounds, cross-archive requests,
missing exports, exact aggregate budgets and an empty component list. Moving
the AssetSet preserves its identity; independently constructing one from the
same bytes does not.

**OBSERVED and cross-checked:** the owned Entry Level 1/Model 2 selection has no
components. The FrontEnd Level 11/Model 13 selection has references 14–23 in
saved order and 24 separately preserved node mappings: twelve component-list
entries and twelve element-list entries. Together the selections retain all
14 exports and 11,155 original bytes. The product matches 81 independent output
rows, including exact source/target spans and usage totals. The actual FrontEnd
selection with a 29-property budget returns `Unsupported`.

The product cycle also checks binding clone/move identity and rejects a distinct
AssetSet namespace built from identical bytes after the original set is dropped.
Eleven named baseline source files, the generated composite and the new binding
source remained unchanged. This verifies the explicit composition, not native
initialization, material evaluation, geometry preparation or runtime parity.

The binding source SHA-256 is
`ec056ddb312d05d7266a662a003bad1913c764335999153809d94b9a5a39c8f1`.
Private joined receipt `independent/verification.json`, SHA-256
`fe2e96b42a389af704ea4fb236e40d8e9be5a4b08b84c088d72f973c3c948e04`,
links the used inputs, contract review, independent expectations and focused
product cycle. `independent/product-check/comparison.json`, SHA-256
`f319689ce5fbecd6407544333571d63a705405ba5bf779a4df775fc2d264dbe8`,
retains the exact actual-export comparison.

## Evidence and provenance

Native grammar and field-role proof are reused from
[UPK_LEVEL.md](UPK_LEVEL.md), [UPK_MODEL_POLYS.md](UPK_MODEL_POLYS.md) and
[UPK_MODEL_COMPONENT.md](UPK_MODEL_COMPONENT.md). No new native format claim or
REA invocation is necessary for this original-source composition.

Private evidence root:
`/home/blizz/.local/share/marvel-heroes-forever-rust/evidence/model-component-binding/`.
The contract review separates native field roles, observed relationships and
explicit composition policy. The independent validation records retain original
input identities, selected spans and focused check results. Raw packages,
captures and private harness output stay outside git.

`contract/receipt.json`, SHA-256
`f2394f70e719120b471b9f746de054f4e9fd91c8185ed37f09b1abba43abe350`,
records the relationship review and selected FrontEnd observations.
`contract/native-constraints/receipt.json`, SHA-256
`c8700a34f2c7d53c3e0ec0dbe856a3b67ac7df6a0638b689c217626b841df149`,
separately distinguishes the prior native evidence from sample-only patterns.

All code and validation glue is original. This increment reads the project's
existing `AssetSet`, startup-map, Level, Model and ModelComponent APIs and their
private prior evidence. It reads no new third-party implementation source.
The underlying selected package and executable provenance remains the owner's
private interoperability study, already registered in
[PROVENANCE.md](PROVENANCE.md); source names and hashes are recorded in the
private `source-ledger.json`.
