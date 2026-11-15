# Stored material properties and local dependencies

`src/material.rs` inspects saved Material and MaterialInstanceConstant objects
for the observed UE3 **868/3** profile. It preserves exact parameter values and
reference origins, and follows local material parents with explicit limits.
It does not evaluate shaders or choose effective inherited parameter values.

## API

`Material::from_export(&Package, export_index, bytes)` uses the established
NetIndex/property prefix and returns:

- Optional saved Parent, ReferencedTextures and Expressions fields. Missing
  arrays remain `None`; explicitly empty arrays remain `Some(empty)`; null
  references and duplicate slots retain their original positions.
- Ordered scalar, vector and texture parameter records. Each retains optional
  ParameterName, exact ParameterValue, raw ExpressionGUID, all nested property
  tags and any uninterpreted fields. Missing fields remain absent. Repeated
  parameter names across records are preserved without choosing a winner.
- Only explicitly saved surface flags, blend/lighting enum names or bytes, and
  opacity clip float bits. No engine defaults or enum-to-render-state mapping
  is synthesized.
- The full property list, uninterpreted top-level property indices and opaque
  native-tail span, all relative to the supplied serialized export.

Scalar and LinearColor values retain IEEE-754 bits, including nonfinite values
and signed zero. LinearColor component order is R,G,B,A; values are not clamped
or converted to colors. Names and GUIDs retain their serialized identities.
Native material data is not decoded. Standalone callers retain the source bytes
if they need to inspect opaque spans later.

`Material::dependencies()` enumerates saved Parent, ReferencedTextures,
Expressions, texture-parameter references and other directly tagged object
properties. Every edge retains its array item or property index. It does not
discover object references hidden inside unknown structures/native data.

`MaterialGraph::inspect(&PackageArchive, &[signed_object_refs])` takes actual
package-local object references as roots. `inspect_with_limits` accepts tighter
budgets. The generic `MaterialSource` interface supports controlled fixtures.
The graph:

- Loads supported local root/parent exports once, retaining both parsed fields
  and exact serialized bytes in `materials`.
- Resolves every enumerated reference into a deduplicated `ObjectIdentity`, with
  null/local export/import classification, declared class and bounded outer-chain
  object label. Labels are not filesystem paths or cross-package identity proof.
- Keeps all dependency edges and their origins. It follows only Parent edges;
  texture and expression targets are metadata identities, not loaded shaders or
  decoded texture bodies.
- Reports null, external-import and unsupported-class root/parent targets as
  explicit traversal boundaries. It does not fetch another package or substitute
  a default material. Imported texture/expression targets remain explicit import
  identities even though they are not parent-traversal boundaries.

MaterialInstanceTimeVarying, expression bodies, material functions, static
switch evaluation, inherited parameter selection, native shader resources,
lighting behavior and scene component overrides remain unimplemented here.

## Established encoding and limits

**OBSERVED in primary reference source and private owner data:** UE3 arrays of
parameter structures contain individually terminated tagged-property lists.
Their records are not fixed-width native structs. The bounded reader permits
supported fields in any order, preserves unknown fields, and rejects duplicate
known fields within a record. It requires all records to consume the declared
array span exactly; a missing nested terminator cannot consume later object
properties. It does not reproduce the reference reader's permissive one-byte
array-size repair.

ParameterName is an eight-byte FName. Scalar ParameterValue is four-byte float
bits, texture ParameterValue is a signed four-byte object reference, and vector
ParameterValue is the 16-byte native LinearColor structure. ExpressionGUID is a
16-byte Guid structure. ReferencedTextures and Expressions arrays have exact
four-byte reference elements. The existing property reader establishes scalar
widths, tag structure, table indices and the 64 MiB per-export ceiling.

Material arrays are capped at 16,384 entries; nested parameter lists at 65,536
properties across one material. Object labels have a maximum of 128 outer links
and 16,384 bytes. Invalid indices, outer cycles, parent cycles, unsupported
profiles and malformed known field types fail explicitly. Unknown field payloads
and native tails remain opaque; their existence is not considered successful
semantic decoding.

Default graph limits are 256 loaded materials, 8,192 distinct reference
identities, 32,768 edges, 32 MiB of serialized material bytes, 1 MiB of combined
identity label/class strings, and 64 local parent levels. Declared export sizes
are charged before loading. Limits may only be tightened by callers. These are
work/storage bounds, not a total process RSS guarantee; decoded properties and
container overhead additionally consume memory within per-object/count limits.

## Owner Vista evidence

The fixed private pin is `pins/mesh-samples/MarvelVista_AsgardHub.upk`, SHA-256
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`, previously
independently matched to the owner's source inventory. No in-progress transfer
candidate was accessed. Pin provenance is recorded in [STATIC_MESH.md](STATIC_MESH.md).
All evidence paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/`.

**OBSERVED:** The package contains 24 base Material exports, 15
MaterialInstanceConstant exports and no TimeVarying instances. Three material
assets are imports: engine default material, engine fog-volume material and a
shared smoke material. All **36 native mesh-section material references are
zero**. This does not establish a default material or any scene assignment;
component material overrides are separate work.

**OBSERVED:** All 39 material exports parse, totaling **55,737 serialized bytes**.
The 15 instances contain **83 scalar, 13 vector and 41 texture parameter records**.
Every observed record saves Name, Value, ExpressionGUID and None, in that order;
scalar/texture records happen to be 116 bytes and vector records 136 bytes in
this sample. The implementation follows tags rather than hardcoding those sizes.
Some vector values are negative or greater than one, including location values
and alpha values of eight/ten; their names alone do not establish shader meaning.

**OBSERVED:** Fourteen instance parents point to six local base materials. One
points to imported `enginematerials.fogvolumematerial` (reference -111). The
41 texture parameter slots contain 32 distinct references, including TextureCube
and an imported reflection texture. None of these 39 objects saves a
ReferencedTextures property. This absence does not prove absence of native
texture dependencies.

**OBSERVED:** The base material Expressions arrays have **1,005 slots**:
**736 null** and **269 local parameter-expression objects**. Their classes are
105 scalar parameters, 77 static switches, 50 texture-2D samples, 32 vectors,
three texture-cube samples and two static booleans. Their bodies and shader graph
are not decoded by this module. Null expression slots remain null.

**OBSERVED (Rust graph):** Using all 39 material exports as roots yields
**345 distinct object identities**, **1,070 saved dependency edges** and one
external-parent boundary. This is a saved-reference graph, not a complete shader
graph. Texture/expression bytes were not loaded by graph traversal.

## Focused verification

**VERIFIED (synthetic):** Nine focused tests pass. They cover exact float/name/
GUID/opaque preservation, absent fields and repeated parameter names, nested
array boundaries, reference widths/indices, duplicate known fields, local parent
deduplication, explicit external/null/unsupported boundaries, parent and outer
cycles, depth and all graph budget categories, and charging bytes before loads.
Fixtures contain newly constructed bytes, not retail assets.

**VERIFIED (independent private byte comparison):** A separate original Python
tagged-property reader checked all **735 top-level/nested tags** across the 39
exports. The original Rust output matches **117/117 files**, **34,850 bytes**, with
zero differences: all 137 parameter records, all 1,070 ordered dependency records
and all 39 opaque native tails. The flattened comparison dump does not encode
missing-versus-empty parameter arrays; the independent source inventory retains
that distinction and focused tests check absence handling.

Comparison receipt SHA-256:
`d931b172383afdd0584160cafe5f6aef25d3803c3194039c96f057bad7215c2c`.
Independent summary SHA-256:
`84e51fd935731718d076aafa729619dfe55f25553ded96718952147b1b798fc8`.

| Private evidence under `evidence/material-study/` | Purpose |
| --- | --- |
| `independent-inventory.json`, `object-index.json` | Raw property spans, nested values and package object identities. |
| `independent-material-summary.json`, `OBSERVATIONS.md` | Independent byte-reader results and concrete dependency observations. |
| `rust_material_check.rs`, `rust-material-check.txt` | Original Rust graph/value smoke and result. |
| `compare_rust_material.py`, `rust-material-comparison.json` | Independent exact typed-value/reference/native comparisons. |
| `tests.txt` | Nine focused test results. |
| `sources.json`, `helper-sources.json` | Exact read-only source identities, hashes and line ranges. |

No renderer, game server or native client runtime was exercised. These are local
source/data-reader checks, not REA output or retail shading/behavior parity. The
project integrator owns the separate mandatory REA session and binary evidence.

## Sources and licences

The external format references below are **MIT** UEViewer, copyright Konstantin
Nosov, pinned commit `a0bfb468d42be831b126632fd8a0ae6b3614f981`, read 2026-11-15.
Implementation and tests are original Rust; no external implementation is copied
or linked. Previously pinned files were reused read-only.

| Source | Use |
| --- | --- |
| [UEViewer `UnMaterial3.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMaterial/UnMaterial3.h) | Material/interface/instance property declarations, saved reference arrays, Parent, scalar/vector/texture parameter fields and native-tail limitations. Private snapshot `evidence/upk-reference/Unreal__UnrealMaterial__UnMaterial3.h`, SHA-256 `d48e4dfd99782f95caf8548a3b2b8be774cda09fa2cebae7643a4f67514ea3cd`. |
| [UEViewer `UnObject.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnObject.cpp) | UE3 structure-array tagged lists and native LinearColor RGBA order; permissive reference size repairs were not adopted. Private snapshot `evidence/upk-reference/Unreal__UnObject.cpp`, SHA-256 `9c8447f35a3fab058879e2547f0cf206a1242826cd727e30994d35b6783a44b2`. |
| Owner's fixed private `MarvelVista_AsgardHub.upk`, identity above | Proprietary game data; bounded compatibility inspection and verification only. |

The existing original project `upk.rs`, `upk/objects.rs` and StaticMesh evidence
supplied package/property APIs and mesh section offsets. Their use does not
introduce another external implementation or runtime dependency.
