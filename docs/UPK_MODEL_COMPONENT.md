# Saved ModelComponent records

`upk::model_component::ModelComponentObject` reads the selected ordinary
`Engine.ModelComponent` map-instance layout in UE3 **868/3**. It retains the
original saved scene properties, Model link, component index, ordered element
records, node indices, references, raw words and any undecoded suffix. It does
not load references, initialize components, prepare render resources or draw BSP.

The actual scope is the ten components reached by the saved
`MarvelFrontEndMap` Level's ModelComponents links, references **14 through 23**.
No unrelated package scan or Asgard bridge is part of this increment.

The diagnostic command `mhf upk model-component <package.upk>
<export-name|1-based-index>` prints the retained Model link, component index,
ordered nodes and element links. Node range validation still requires the
linked Model; the command performs no geometry or material evaluation.

## Admission and component preamble

The reader requires an exact `Core.Class` import named `ModelComponent` directly
under root `Core.Package Engine`, and a direct local export outer with the
equivalent exact `Engine.Level` identity. Custom classes, exported class
lookalikes and different outer profiles are not admitted.

**VERIFIED (static):** the ModelComponent class factory at `0x140c02850` leads
to callback/constructor `0x140c08ab0`, which installs vtable `0x141f9b2a8`.
Its Serialize slot `+0x68` selects `0x140c0e6c0..0x140c0e737` (119 bytes).
That body calls PrimitiveComponent Serialize
`0x14094eb50..0x14094ebe3` (147 bytes), which calls the established UObject
serializer. Class metadata separately links ModelComponent to PrimitiveComponent,
ActorComponent, Core.Component and Core.Object. The actual Serialize call chain
does not insert an extra call for every metadata base.

PrimitiveComponent's additional `+0x1b0` reference is skipped when the archive
is loading or saving; its pre-560/pre-569 runtime adjustments are excluded at
868. It contributes no additional ordinary-file tail before ModelComponent.

**VERIFIED (static):** ModelComponent's vtable `+0x168` predicate resolves to the
six-byte true leaf at `0x140210e00`. UObject therefore calls component helper
`0x14074cfb0..0x14074d359` before its four-byte NetIndex. That helper always
serializes the reference at component `+0x60`. It conditionally adds an FName
from `+0x68` when an object/outer has flag bit 9, or under other archive modes
(`archive +0x28 == 0` or `archive +0x70 & 0x1000`). Thus a component's preamble
is **not universally eight bytes**.

This reader admits the observed ordinary persistent file/no-name branch. It
rejects the state-frame flag, object/outer flag `0x200`, imported or unresolved
outer chains, cycles and chains exceeding 64 objects. It reuses
`world_scene::SavedSceneObject` for the established eight-byte prefix and tagged
properties. The leading reference is checked against package tables; NetIndex
is preserved as a signed word. Named/template and other archive modes remain
unsupported. Later native repair side effects are not executed or claimed.

## Native field sequence

All spans below are relative to the complete decompressed export. Native memory
offsets identify fields, without implying equal wire offsets.

| Native field | Ordinary file representation | Reader field |
| --- | --- | --- |
| `+0x238` | Signed four-byte Model reference | `model` |
| `+0x240` | Four raw bytes | `word_240` |
| `+0x258` | Signed element count, then ordered records below | `elements` and count/span |
| `+0x244` | Two-byte unsigned ComponentIndex | `component_index` and span |
| `+0x248` | Signed count, then unsigned two-byte Model node indices | `nodes` |
| Remaining bytes | Retained without interpretation | `unparsed_native_tail` |

**VERIFIED (static):** known Level construction supplies its Model to component
`+0x238`. Its ModelComponents array count supplies both the component's `+0x244`
index and the corresponding Model Node.ComponentIndex field. Component
`+0x248` stores indices used to select 64-byte records from Model's Nodes array.
The `+0x240` word's member name and meaning remain unproved and are not inferred
from its observed values.

**VERIFIED (static):** element-array helper `0x140bf0f10..0x140bf1124`
allocates native records of size `0x7c` and calls
`0x140bf1130..0x140bf11d4` for each. Native memory size is not a wire stride.
Each admitted element serializes:

| Native element field | Stored representation | Reader field |
| --- | --- | --- |
| Pointer `+0x30` | Four-byte discriminator, **zero only** admitted | `header_30` |
| `+0`, `+8` | Two signed object references | `references[0..2]` |
| `+0x10` | Signed count and unsigned two-byte node indices | `nodes` |
| `+0x20` | Signed count and signed four-byte object references | `references_20` |
| `+0x38` | Signed count and four raw 32-bit words per record | `records_38` |

Helper `0x140bd7380..0x140bd7531` handles the first discriminator. Values 1 and
2 construct different native types and dispatch payload serializers that this
increment has not recovered. The selected array-load path initializes the
record's `+0x30` pointer to null. With discriminator zero, no polymorphic payload
follows. Both conditions are established; zero alone would not prove that no
payload is consumed from an arbitrary existing native pointer. The reader
rejects every nonzero discriminator as `Unsupported`, without skipping bytes
or treating unknown variants as null.

Unsigned-two-byte arrays use helper `0x140df6c90..0x140df6e72`. The reference
array uses the previously established helper `0x140bae2b0..0x140bae432`.
Helper `0x140d67500..0x140d67882` establishes the final array's four raw words
per record. Those records remain unnamed; no UUID, lighting or geometry meaning
is imposed. The element's first object slot is bound to its owning component
by native construction. The second slot's exact native member/type spelling is
unresolved; its actual referents are recorded below.

## Bounds and API

`parse_export(package, export_index, bytes)` accepts a **zero-based** export
index. Signed references use zero for null, positive one-based exports and
negative one-based imports. Every reference is checked against package table
ranges. Nulls, duplicates and order are preserved; native typed referent classes
or identity equalities are not imposed without their separate context.

The element count has an explicit 4,096-element inspection cap. Nested arrays
and the final node array share a cumulative **65,536-item** budget per export.
These are reader limits, not asserted native rejection limits. Negative counts
and incomplete known fields return `InvalidData`; exceeded inspection limits
and unsupported profile/discriminator cases return `Unsupported`. The parser
preflights `element_count × 24 + 6` minimum remaining bytes before reserving the
element vector, and each complete fixed-width array payload before allocation
or iteration. All arithmetic is checked.

`scene` preserves the eight-byte prefix, all original property tags and exact
object/outer/archetype identities. `ModelComponentNodes` preserves its count and
payload spans and raw `u16` values. A node's range requires the linked Model;
this single-export parser does not load that Model or claim an executable scene
from a valid wire index. Reference arrays keep a span for every link. Raw
four-word records retain their full record spans and bit patterns.

## Original FrontEndMap observations

Private root: `/home/blizz/.local/share/marvel-heroes-forever-rust/`.
Input: `pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFrontEndMap.upk`,
SHA-256 `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2`.

**OBSERVED:** all ten exports use class import -12 (`engine.modelcomponent`),
outer Level reference 11, null super/archetype, and object flags
`0x0007000100000000`. Their contiguous logical export range is
35,245..36,685, totaling 1,440 bytes. All have leading reference 0, NetIndex -1
and three saved property tags. The native payload starts at export offset 94
for references 14–17 and 22–23, and 102 for references 18–21.

All ten native Model references are 13. Each has one element with discriminator
zero; its first reference equals the component itself. Its second reference is
-33 (`RemoveSurfaceMaterial`) for components 14–17 and 22–23, or -32
(`DefaultMaterial`) for 18–21. Both are `Engine.Material` imports. Both later
element arrays are empty in these samples. Nonempty reference/four-word-array
handling is grounded in native loops and focused synthetic checks, not observed
nonempty retail arrays in this selection.

| Component reference | ComponentIndex | Element and component node lists |
| ---: | ---: | --- |
| 14 | 0 | `[0]` |
| 15 | 1 | `[1]` |
| 16 | 2 | `[2]` |
| 17 | 3 | `[3]` |
| 18 | 4 | `[4,7,8]` |
| 19 | 5 | `[5]` |
| 20 | 6 | `[6]` |
| 21 | 7 | `[9]` |
| 22 | 8 | `[10]` |
| 23 | 9 | `[11]` |

These lists select all twelve of Model 13's 64-byte Nodes records exactly once.
The raw unsigned word at each selected Node's `+0x1c` agrees with the component
index, consistent with the separately established native Node.ComponentIndex
role. The ten components occur in exactly this order in Level 11's saved
ModelComponents array. These are original saved relationships, not evidence of
runtime loading, visibility, material evaluation or drawing.

Component 18 has a 54-byte native payload; the other nine have 46 bytes. The
known layout accounts for all 468 native bytes with no suffix observed. Extra
bytes remain supported as opaque suffixes rather than being required to vanish.

## Focused validation

**VERIFIED:** the four new reader tests passed in a private wrapper using frozen
baseline `7871c9efa6b63a968b2cd34faeb050dbfff10030`. They cover raw fields and
nonempty arrays, preserved references and suffixes, truncated known fields,
invalid counts/references, unsupported discriminators, exact class/profile and
outer-prefix admission, and the cumulative item limit. The focused test, build
and actual-export run each exited zero; no baseline or full suite was run.

The actual ten components match **214 output rows** from a separate raw-byte
oracle: 30 property tags, 40 prefix/native references, 24 ordered node values,
ten ComponentIndex and ten NetIndex values, 20 raw native words and twelve
existing Model Node record spans. All 261 frozen source files and the new reader
remained unchanged. The reader SHA-256 is
`90e62001c5c11e89435967f390f8dc165ea1406cb4324faa35ebfad97744106c`.

The independent oracle also passed 105 focused fixture checks, including the
same minimum-payload and count policies. Its first attempt failed an
instrumentation assertion for an early unsupported-profile rejection; the
metrics assertion was corrected and both attempts are retained. This was not a
product discrepancy. Oracle fixture success and original-asset agreement do
not establish retail runtime, material evaluation or geometry parity.

Private receipts:

- `independent/verification.json`, SHA-256
  `3e201165decb807aa338018dc2642145f0d6ed5efa5343b1d971eec6b290b88a`,
  joins the focused product checks, separate oracle, exact original captures,
  Model node relationships and native support by hash.
- `independent/product-check/comparison.json`, SHA-256
  `26b969d16fe58fd1a4a62dd0e4778250da21d867ccfa8e01ab4a4c7ccab89855`.
- `independent/oracle/verification.json`, SHA-256
  `e7f5e7b164d817f739de1bc83dba0b6a36676a980929e005b15e47e539ac2302`.

## Evidence and provenance

Private evidence: `evidence/model-component-native/`.

- `native-support/receipt.json`, SHA-256
  `3d229036294fd069abdc411ce49c133aaf204f1d0b68de7ef39ce921c10e8c67`,
  verifies all four new REA records, 3,718 bytes across nine selected bodies or
  the true leaf, complete unwind ranges, class/base identity and prefix/record
  conditions. Prior UObject, NetIndex, scalar and reference-array findings are
  reused with explicit source identities. Native and runtime limitations remain
  separate from the checked file grammar.
- `native-support/field-identities/` grounds Model, ComponentIndex, record-owner
  and node-index relationships; unresolved names remain unresolved.
- `independent/` retains the frozen baseline
  `7871c9efa6b63a968b2cd34faeb050dbfff10030`, selective original extraction,
  exact source/input hashes, a separate raw-byte oracle and product validation.
- `independent/model-node-links/findings.json`, SHA-256
  `65dcf1628ef419a4d1ef65d98665474d76dedce39648884d48f0b308931fff02`,
  records the twelve original Model Nodes selections and component relationships.
- `source-ledger.json` and `MANIFEST.json` register private sources and evidence.

All implementation and validation glue is original. No additional third-party
implementation source was read or copied.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The private FrontEndMap pin above | Owner proprietary assets; private interoperability study | Ten exact component exports, saved Level links and Model 13 bytes |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Owner proprietary executable; private interoperability study | Exact factories, class chain, serializer branches and field relationships |
| Existing original object/world-scene/Model readers and named private prior receipts | Original project source and findings | Saved property/preamble handling, reference identities and bounded original-byte comparisons |
| REA 6.9.0, installed skill/tool schemas and evidence/native guides | REA MIT, as registered in [PROVENANCE.md](PROVENANCE.md) | Bounded target-free native observations and evidence retention |
| Original PE helper, GNU objdump, Python and Rust tools | Original glue and installed tools; no implementation copied or linked | Independent byte/range verification, oracle and focused build checks |
