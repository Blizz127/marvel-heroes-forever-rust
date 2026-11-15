# Asset-bound Model index construction

`frontend::model_index_binding::prepare` connects the checked saved
[Level/component binding](MODEL_COMPONENT_BINDING.md) to the explicit
[integer index builder](MODEL_INDEX_RESOURCE.md). The result keeps the source
binding and its exact AssetSet identity alongside the constructed resource.
It does not choose the live Level, resolve materials, execute callbacks or
enable drawing.

## Selection and retained identity

The caller supplies the original `AssetSet`, an immutable `Binding`, explicit
zero-based component ordinals, the post-callback snapshot phase, resolved
equality keys and initial buffers/map. All builder inputs remain explicit.
The invocation selection can be empty, reordered or repeated; it does not have
to traverse every saved component. The underlying binding already retains and
checks the complete saved Level component list.

The bridge first rejects a different AssetSet identity with `InvalidInput`,
even if it contains identical package files. It then checks the invocation
count before allocating the selection vector. Too many invocations return
`Unsupported`; out-of-range ordinals return `InvalidInput`. Core reader and
index-builder errors propagate without substituting state.

The selected exports borrow original bytes directly from the binding. Package
tables come from that binding's matching archive slot. No files are reread or
references resolved. The core `Request<'a, 'r>` separates the original byte
lifetime from temporary request-slice lifetimes; this is the only core change.
The index algorithm and admitted native branch are unchanged.

`Prepared::source()` exposes the borrowed immutable binding;
`Prepared::resource()` exposes the constructed result immutably;
`Prepared::matches_assets()` preserves the same namespace check. Temporary
selection vectors, reference mappings and initial buffer storage can be dropped
after preparation. The result still borrows the binding's original export
bytes. Moving or dropping the open AssetSet does not invalidate those bytes;
subsequent archive-dependent work still needs a matching set.

The builder copies snapshot metadata and generated indices, but it preserves
the retained Model/component bytes and complete parsed objects. Repeated
invocations preserve separate results that point to the same original source
bytes. Component order is caller invocation order, not an assertion about
native scheduling, sparse-map enumeration or GPU draw order.

## Focused verification

**VERIFIED:** a private harness builds against the production library with
window features. Its three actual-package cases match all **147 index values
and 20 element metadata results** from the previously sealed independent
expectations. They use Entry Level 1/Model 2 and FrontEnd Level 11/Model 13 with
components 14–23. The identity tokens and empty/preseeded map states remain
explicit hypothetical stable snapshots, not observed native runtime state.

The same harness checks exact borrowed slice identity and parsed-object equality,
temporary-request lifetime independence, cloned binding/moved archive identity,
continued byte access after the archive set is dropped, and rejection of an
independently created set with identical files. Empty selections remain empty.
Selection `[4,0,4]` preserves order, repeated source backing and appended indices.
Bounds, selection/index budgets, missing identity mappings, invalid initial
indices and the unsupported earlier snapshot phase are rejected.

All four focused core index tests also pass after the lifetime change. An exact
source-replacement audit confirms that only lifetime declarations and signature
layout differ from the earlier tested core. Both builds and the actual-package
run exit zero without compiler warnings. No full suite, native execution,
service request, material evaluation or renderer parity check was run.

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/model-index-binding/`.
The receipt SHA-256 is
`28b3255e60ebebd8e06c1998e3481d7b70c7693936568e7b1b55d95c75be5bff`;
the 25-file manifest is
`f4e7716565008f9c81ece3156baeb4c850c20b54252e8acc5d81c59dc193d9d0`.
Frozen bridge source SHA-256:
`6ffcb5d3dfcf050fa0a39388379e49668f8261154ae3937f3f2e59993f60721b`.
Updated core source SHA-256:
`156a3728b9332bbee40dae89d4bbc804865eba84e59c9f9582fbfddf25c67b98`.
The earlier core source hash and its original checks remain historical evidence.

This composition uses original project code and the previously documented owned
packages, native evidence and independent expectations. It adds no external
game implementation or new claim about native lifecycle behavior.
