# Selected Model Node geometry records

`upk::model_node_geometry::ModelNodeGeometry::parse_export` reads one exact
868/3 Model export through the existing `ModelObject` parser and follows only
the caller's selected zero-based Node indices. It retains the selected Nodes,
their ordered corner records, saved position triples and corresponding `+0xc8`
records. It does not triangulate, resolve materials, transform coordinates,
create render resources or execute a world.

## Native evidence and interpretation boundary

**VERIFIED (static):** the established Model serializer uses raw bulk loading
for the admitted 868/3 profile, with 64-byte Nodes, 24-byte records at Model
`+0x80` and 12-byte triples at Model `+0xb0`. These bytes map directly to the
native record offsets in this profile. The existing reader checks the stored
strides and complete payload bounds. Different widths, legacy per-element load
paths and byte-swapping remain outside its admission.

New complete REA observations retain the fallback array helpers and their
per-element dependencies:

| Function range | Role established by its caller and operations |
| --- | --- |
| `0x140beff00..0x140bf0080` | Nodes fallback array loop, calling `0x140bef990` |
| `0x140bef990..0x140befb27` | Node element fields, covering all 64 native bytes |
| `0x140bf0d70..0x140bf0f04` | 24-byte pool fallback array loop, calling `0x1408035a0` |
| `0x1408035a0..0x140803678` | Four mandatory raw words, then two conditional words |
| `0x1410af230..0x1410af542` | Triple-array fallback loop, preserving three four-byte fields |

Those fallback paths support record field widths; they are not asserted to run
during the admitted modern raw load. In particular, the Node's `+0x36` is one
byte, while `+0x1c` and `+0x1e` are separate two-byte fields. The fallback helper
masks Node byte `+0x37` with `0x1f` on load; this reader preserves the original
byte without applying that mutation.

**VERIFIED (static):** known Level vtable `0x141f90e50` slot `+0xb0` selects
`0x140bbedd0..0x140bbee01`. This complete 49-byte wrapper preserves `this` and
tail-calls `0x140bc4760..0x140bc66e4`, the complete 8,068-byte Level consumer.
The typed Level's `+0xd0` is its Model. Its following operations establish the
joins without relying on a generic UE3 layout or a matching anonymous receiver:

| Source field | Grounded use |
| --- | --- |
| Model Nodes record `+0x10` | Signed 32-bit start index into Model `+0x80` |
| Node `+0x36` | Unsigned-byte corner count |
| Consumed `+0x80` record `+0` | Signed 32-bit index into Model `+0xb0` |
| Selected `+0xb0` record | Three single-precision position coordinates |
| Node `+0x14` | Signed 32-bit index into Model `+0xc8`, native stride 96 |
| Node `+0x1c` | Previously proved unsigned 16-bit ComponentIndex |

The consumer checks the `+0x14` record index before testing corner count. A zero
count skips the pool and position dereferences, so an unused negative pool start
is retained without an invented lookup. No negative-sentinel branch for the
selected `+0x14` index was established in this typed path.

The API names `surface_index` and `surface` identify the `+0xc8` record selected
by the Node. They do not claim recovery of original member/type spelling.
`ModelC8Record` already distinguishes its **60-byte wire record** from the
**96-byte native record** and retains two object references and thirteen raw
words. This increment does not reinterpret the other words or require either
reference to resolve to a particular class. The owned samples' leading
references resolve to Material imports; no import is loaded or evaluated.

The typed Level consumer also exposes additional vector/basis relationships.
They remain private supporting findings; Model `+0x98`, other pool words,
other Node fields and the rest of the `+0xc8` words keep their existing raw
representation here. A record's name or semantic role is not inferred merely
from numerical agreement in the selected assets.

## Explicit selection and retained data

```rust,ignore
let geometry = ModelNodeGeometry::parse_export(
    archive.package(),
    model_export_index,
    &model_bytes,
    &selected_node_indices,
    Limits::default(),
)?;
```

`selected_node_indices` can come from the already checked component bindings,
but this API does not discover a Level or select components. Caller order and
duplicate Node indices are preserved. An empty selection validates the original
Model grammar and returns no joined Nodes.

The result borrows exactly the same bytes used by `ModelObject::parse_export`
and retains that complete Model reader result, including all untouched arrays,
raw words, references and opaque suffixes. Each `NodeGeometry` contains:

- The complete original 64-byte record and its export-relative span.
- The raw signed pool start and `+0x14` index, unsigned corner count and
  ComponentIndex.
- The selected original `ModelC8Record`, with both raw reference values.
- Ordered `Corner` entries, each retaining its pool index, 24-byte record span,
  all six raw words, signed point index and position record.

`Position` preserves all three coordinate bit patterns and its 12-byte span.
`coordinates()` converts those exact bits to `f32` without rejecting NaNs,
infinities, signed zeros or applying normalization. No coordinate-system or
world-transform convention is imposed. Spans refer to the complete Model export;
add its saved serial offset for a logical package position.

## Checked bounds

Default inspection limits are 32 MiB serialized bytes, 65,536 selected Node
occurrences and 262,144 total corners. Repeated selections and point indices
consume these budgets again. Existing Model parser limits also apply. Request
and byte limits are checked before parsing; the cumulative corner budget is
checked before each corner vector allocation.

Each selected Node and `+0xc8` record index must be in range. A nonzero corner
count requires a nonnegative pool start and a checked `start + count` range
inside the admitted pool. Each consumed pool record must have a nonnegative,
in-range position index. Complete record spans are checked by the existing
bulk descriptor. Zero-corner nodes preserve the unused start as described above.

Only dereferenced joins are checked. Unselected Nodes and unused pool records
are not subjected to new semantic validation. This matters for the actual
front-end pool: it has 144 records, while the twelve selected Nodes reference
only records 0–47. Treating every pool first word as a dereferenced position
index would incorrectly reject unused records.

Invalid selected links/ranges return `InvalidData`; work-budget exhaustion
returns `Unsupported`; existing class/profile/stride errors propagate. These
are bounded join-admission rules, not claims that retail safely rejects every
malformed package. There is no sorting, deduplication, partition check, forced
minimum corner count, triangle generation or material-equality rule.

## Focused validation and original observations

**VERIFIED:** exactly four new module tests passed, followed by one focused
offline build and one actual-package run. Tests cover stored field/coordinate
bits and spans, repeated selections/corners, unused invalid pool words,
selected Node/pool/point/record bounds, zero-corner behavior, work budgets and
the existing Model class/profile/stride boundary. No full suite or native game
runtime was run.

The two original Models match **106 output rows** against independent raw-byte
expectations. Entry Model 2 has an empty explicit Node selection. FrontEnd
Model 13 Nodes 0–11 yield **48 ordered corners** using all sixteen position
indices. Its selected `+0xc8` record indices, in Node order, are
`[0,1,2,3,10,11,6,7,8,9,5,4]`. This is an observation, not an enforced ordering
or partition invariant.

The comparison verifies all 24 selected `+0xc8` reference occurrences, 288 pool
words, 144 position-coordinate words and 156 `+0xc8` raw words, with exact
record spans and both complete base Model results preserved. Independent
Python derivations agree on 696 stored field views. Borrowed-byte identity and
coordinate bit roundtrips also pass. This establishes saved-record joins only;
rendered geometry and retail behavior remain unverified.

New source SHA-256:
`7a1d87d55a8231125bd107b137d49be406057e1c56287a81aa793f025cf07232`.
Private joined receipt `independent/verification.json`, SHA-256
`92ecbaefd166e5ea0b12b61932f35fb05584dd3b7a10a4efceac49f24718447d`,
binds the original inputs, native support and focused product cycle.
`independent/product-check/comparison.json`, SHA-256
`55de7e42078633cc7c3937197d773a5c549ab09e11672b03e18579ce2717e1fb`,
retains the exact output comparison. Source and used input identities remained
unchanged throughout validation.

## Evidence and provenance

Private root:
`/home/blizz/.local/share/marvel-heroes-forever-rust/evidence/model-node-geometry/`.
The REA bundle retains three new successful records for the seven complete
functions above. Native support verifies the exact owned PE bytes, unwind
extents, typed receiver chain and selected consumer operations. Independent
expectations retain original-byte joins separately from native semantic claims.

`native-support/receipt.json`, SHA-256
`0d77d68b303b10b1d0157ac6480adcbd504abe43cccfac8c197045c27e5c9f44`,
verifies all three REA records and **10,314 bytes** across seven complete chained
bodies against the unchanged owned executable. `rea-bundle.json`, SHA-256
`3303bc3f8c99055ef561cdc99fe24e3e5da946b3d38a3c1c4ed15b53571196d2`,
retains those observations. The target-free session was exported and reset;
there was no native execution or shared configuration change.

Existing serialization proof is documented in
[UPK_MODEL_POLYS.md](UPK_MODEL_POLYS.md), with component/node relationships in
[UPK_MODEL_COMPONENT.md](UPK_MODEL_COMPONENT.md) and
[MODEL_COMPONENT_BINDING.md](MODEL_COMPONENT_BINDING.md).

| Source | Licence/ownership | Use |
| --- | --- | --- |
| Owned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Owner proprietary; private interoperability study | Exact serializer dependencies and typed Level consumers |
| Owned `MarvelFrontEndMap.upk`, SHA-256 `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` | Owner proprietary assets; private interoperability study | Model 13 Nodes, selected pool/position records and `+0xc8` references |
| Owned `MarvelEntry.upk`, SHA-256 `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` | Owner proprietary assets; private interoperability study | Model 2 empty-node selection |
| Existing original Model/component readers, private PE helper and prior receipts | Original project code/findings | Layout admission, checked spans and retained source identity |
| REA 6.9.0 skill/tool contract and native/evidence guides | REA MIT, registered in [PROVENANCE.md](PROVENANCE.md) | Bounded target-free Rizin observations and evidence retention |
| GNU objdump, Python and Rust tools | Installed tools; no implementation copied or linked | Independent byte verification, raw-byte expectations and focused checks |

All code and validation glue is original. No additional third-party geometry
implementation was read or copied. Raw assets, disassembly and captures remain
private; `source-ledger.json` records the exact sources used.
