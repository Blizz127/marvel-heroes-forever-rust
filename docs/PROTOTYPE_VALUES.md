# Lazy prototype parent-value views

`src/prototype_values.rs` provides a bounded, lazy view of values encoded in a
prototype and its explicit header-parent chain. It preserves the origin of every
stored assignment. It does not instantiate game classes or claim retail runtime
assignment parity.

## API

Create a catalog reader as usual, then call
`PrototypeValues::resolve(&mut reader, prototype_id)` or
`resolve_with_limits(&mut reader, prototype_id, ValueLimits { ... })`.
The generic `PrototypeSource` interface also supports controlled fixtures without
opening a package. The `PrototypeReader` adapter uses its directory, blueprint,
member-size and lazy body-read accessors.

- `layers()` returns the explicit chain, default-first and requested prototype
  last. Each layer includes the exact prototype ID, blueprint ID, path, encoded
  parent ID and decoded body.
- `parent_links()` records whether the parent blueprint is identical, an ancestor
  or unrelated, and whether the two runtime-binding **names** match. Name equality
  is metadata evidence, not proof of runtime type identity.
- `fields()` and `field(FieldKey)` expose structural slots keyed by declaring
  blueprint, copy number and field ID. Each slot retains every stored assignment
  in chain/file order.
- `latest_stored()` identifies the last encoded assignment even when its runtime
  meaning is unresolved. `stored(origin)` and `origin_layer(origin)` retrieve its
  exact value and source.
- `selected()` returns a conservative **serialized override** selection only
  when the available metadata is unambiguous. It is not a runtime-class field.
  `unresolved()` explains why a slot has no selection.

No parent value is deep-cloned or eagerly resolved through referenced assets or
prototypes. Only the requested prototype's header chain is loaded. Returned
origins point into immutable owned layers; subsequent lookups do not load files.
Separate resolution calls have separate bounded snapshots rather than an
unbounded process cache.

## Established behavior and deliberate gaps

**OBSERVED (reference source):** Deserialization copies the explicitly referenced
parent before applying the child's fields. A reference-only child adds no stored
assignments. Blueprint-parent relationships supply schema information; they are
not an implicit value-default chain. Terminal missing values must remain absent.

**INFERRED (conservative storage-level implementation):** For a stable structural
slot containing a scalar or plain ID list, the nearest stored assignment wins
when group/schema binding names match the containing blueprint and lineage is
unambiguous. Both ancestor and child assignments remain inspectable. An explicit
empty ordinary ID list is distinguishable from an omitted list. Field IDs missing
from the entire encoded chain remain missing; no zeros, empty arrays or host-class
defaults are synthesized.

**OBSERVED (reference source limitations):** Runtime copying requires exact class
type equality. Ordinary lists replace inherited arrays, but property lists update
an inherited property collection and an empty property list preserves it.
Ordinary embedded null clears a pointer, while a similarly encoded property ID
may perform no assignment. Mixins need runtime element type and may load separate
element defaults. Therefore this reader leaves all `RHStruct` fields/lists,
including empty lists, explicitly unresolved. It also withholds selections for
runtime-binding context differences, nonzero mixin copy numbers, duplicate slots
within a layer, conflicting schema identities, cross-group field aliases and
questionable blueprint lineage.

The key `(declaring blueprint, copy number, field ID)` is an exact structural
identity, not a complete runtime identity. No property merge, mixin allocation,
nested-object inheritance, numeric coercion, enum binding, post-processing or
runtime type resolution is implemented here. Unrelated blueprint lineages remain
inspectable with unresolved selections rather than being silently accepted as
compatible runtime classes.

## Bounds and identity checks

Default limits cover 64 parent layers, 32 MiB of accumulated uncompressed member
bytes, 128 MiB of accounted decoded storage, 100,000 nodes, 200,000 groups,
500,000 fields, 1,000,000 values, embedded depth 64 and 1,000,000 schema steps.
Member sizes are charged **before** loading each body. Decoded vectors, boxes and
paths are accounted after each bounded catalog decode; node/field/group/value
counts also bound the provenance index. Accounted storage excludes allocator and
index-container overhead, existing catalog metadata and the catalog decoder's
temporary per-file buffers. This is not an exact RSS ceiling.

Prototype and traversed blueprint cycles fail explicitly. Missing directory IDs,
members, blueprints or matched field schemas fail. Selected directory IDs must
match the requested ID, blueprint IDs must match their default-prototype IDs, and
a reference-zero terminal must be its blueprint's declared default ID. The reader
does not repair a broken chain by inserting an inferred default. Different binding
names and unrelated blueprint ancestry are retained as unresolved metadata checks.

## Validation

**VERIFIED (synthetic):** Eight focused tests cover lazy body-loading counts,
parent overrides with preserved origins, reference-only inheritance, absent fields,
ordinary empty-list replacement versus unresolved empty embedded lists, prototype
and blueprint cycles, missing IDs, nondefault terminals, every budget category,
duplicate slots, mixin copies, cross-group aliases and mismatched lineage.

**OBSERVED (independent private sampling):** A separate original Python scanner
examined 612 candidate paths, reading 707 prototype files and six blueprints, to
identify five distinct two-node chains: a reference-only child, inherited scalar,
scalar override, inherited list and list override. It used explicit encoded
parents and field bytes rather than emulating runtime inheritance. Exact IDs,
paths, schemas and raw values remain private in
`~/.local/share/marvel-heroes-forever-rust/evidence/prototype-inheritance-samples-20261115.json`.
Evidence SHA-256:
`cca8d82fbd4987224da18585b8acc430b815bf372d9b1437679bf4d54692936c`.
The scanner is `prototype-inheritance-samples.py` beside that evidence.

**OBSERVED (Rust reader against those independent samples):** All five chain
checks pass, with **ten prototype body loads** total. Exact chain IDs, blueprint
IDs, header parents, raw selected-field bits and latest assignment origins agree
with the independent scanner. Scalar and plain-list overrides produce selections;
the sampled inherited scalar/list slots retain their values and origins while
remaining unresolved because their declaring contexts have different binding
names. The reference-only child exposes 19 inherited structural slots, all from
the default layer. No full package rescan was needed for this increment.

Private reproducible harness and output:
`evidence/prototype-values-smoke.rs` and `evidence/prototype-values-smoke.txt`.
Focused test harness/output:
`evidence/prototype-values-tests.rs` and `evidence/prototype-values-tests.txt`.
All paths are relative to `~/.local/share/marvel-heroes-forever-rust/`.

The selected package is the private Alienware pin, Calligraphy SHA-256
`1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c`.
These checks are local data-reader evidence, not REA or running-client evidence.

## Sources and licences

MHServerEmu, <https://github.com/Crypto137/MHServerEmu>, **AGPL-3.0**, read-only local
root `/home/blizz/dev/marvel-heroes/MHServerEmu`, commit
`8fedb1a35922adfc9bff77e54f5fc3b6956073c9`. No reference implementation code is
copied into this module.

| File under `src/MHServerEmu.Games/GameData/` | Facts studied |
| --- | --- |
| `Calligraphy/CalligraphySerializer.cs` | Lines 42–114: explicit parent before child; 717–879: runtime copying distinctions; 965–1023: mixin identity/defaults; 1241–1278 and 1302–1354: embedded/property differences; 1443–1453 and 1514–1528: ordinary list replacement; 1555–1601 and 347–419: property list updates. |
| `DataDirectory.cs` | Lines 170–189 and 420–447: directory-to-blueprint/runtime mapping; 552–557: blueprint defaults; 666–716: schema ancestry versus prototype parents. |
| `GameDatabase.cs` | Line 158: lazy prototype lookup delegation. |
| `Calligraphy/Blueprint.cs` | Lines 171–190: ordered schema-parent lookup. |

The original Python validation used system `liblz4.so.1` from `liblz4-1 1.10.0-8`,
**BSD-2-Clause** library code; see the licence provenance in
`docs/PROTOTYPE_FORMAT.md`. Owner-provided Calligraphy data is proprietary
Gazillion material studied privately for interoperability. Package files,
extracted data and exact sampled values are excluded from git and build archives.
