# Prototype body reader

`src/calligraphy/prototype.rs` adds original Rust structural decoding for complete
Calligraphy version 11 `PTP` files, including `.prototype` and `.defaults` bodies.
It consumes the existing blueprint and prototype-directory metadata readers.

## API and supported data

Construct `SchemaSet::new(&blueprints, &prototype_directory)` once. For a directory
record, call `Prototype::parse(&bytes, record.blueprint_id, &schemas)`. The optional
`parse_with_limits` takes an explicit `PrototypeLimits` value. Integration requires
`pub mod prototype;` in `src/calligraphy.rs`.

The result preserves the prototype reference/data/polymorphic header, ordered
field groups, declaring blueprint IDs, copy numbers, field IDs, matched schema
declaration IDs, scalar values and list values. Embedded `RHStruct` values contain
the same complete structure recursively. `Value` distinguishes asset, curve,
prototype, localized-string and type IDs from integers, doubles and booleans.
Boolean values retain their full serialized 64 bits; doubles retain their decoded
IEEE 754 representation. No conversion to narrower runtime fields is performed.

**OBSERVED (reference source):** Ordinary groups search the containing blueprint
and its ordered parent hierarchy for field definitions; property groups search
the declaring group blueprint instead. Property status is set through additional
runtime metadata, not established by the blueprint binding-name string alone.

**INFERRED (structural validation design):** The reader checks both available
contexts, rejects unknown fields and conflicting type/structure/subtype matches,
and preserves the declaring group rather than assigning runtime properties. A
matched schema ID identifies the schema used for validation, not a proven runtime
field owner. Nested data bodies obtain their containing blueprint through the
referenced prototype's directory entry (or the blueprint/default ID). Parent
schema lookup preserves declared parent order and detects traversal cycles.

The supported list tags are `A`, `P`, `R` and `T`, matching both the reference
blueprint admission logic and the observed package. Unknown tags/flag bits,
schema mismatches, truncated data and trailing bytes fail explicitly. Top-level
files require a reference header and cannot be polymorphic. Nested flags remain
represented as data; ordinary-pointer and property-pointer runtime semantics are
not inferred from the serialized tag alone.

## Limits and remaining scope

Default limits apply to one complete input file, including all nested bodies:
64 MiB input, depth 64, 100,000 prototype nodes, 200,000 groups, 1,000,000 fields,
1,000,000 values and 1,000,000 schema traversal steps. Configured nesting depth
cannot exceed 256. Counts must also fit the physically remaining bytes before
allocation. Limits are reader safeguards, not observations of retail limits.

This reader does **not** merge inherited prototype values, instantiate runtime
classes, bind symbolic enums or properties, validate every referenced data ID,
resolve deprecated GUIDs, infer client behavior or establish gameplay parity.
Reference-only embedded values stay reference-only. Runtime-specific handling of
polymorphic values and mixin copies remains separate work. Embedded data whose
containing blueprint cannot be resolved fails rather than guessing.

## Validation evidence

**VERIFIED (synthetic):** Nine focused tests cover nested list and reference-only
values, signed and raw boolean values, all byte-truncation boundaries, trailing
data, unknown fields/tags, schema type and structure mismatches, conflicting
lookup contexts, parent traversal cycles, missing schemas, ordered inherited
schema lookup, invalid top-level headers and every configured budget category.
Output is private in
`~/.local/share/marvel-heroes-forever-rust/evidence/prototype-tests.txt`.

**OBSERVED (independent package shape scan):** A separate original Python scanner
using system `liblz4.so.1` parsed 85,186 prototype/default files and 5,659 blueprints
from the server-data Calligraphy package without shape errors. It counted
731,294 groups, 1,475,896 simple fields, 247,819 list fields and 1,947,742 values;
maximum embedded depth was 18. Top-level flags were only `1`/`3`; simple embedded
flags `0`/`1`/`3`; list embedded flags `1`/`3`/`5`/`7`. It observed no embedded zero
reference or reference-absent data body. This scan is independent structural
evidence; it does not itself validate Rust schema matching.

Private scan evidence:
`~/.local/share/marvel-heroes-forever-rust/evidence/prototype-format-scan-20261115.json`,
SHA-256 `d3fc9b30dba9a5f22d8cfdc6947fef308bbbe8568eff7e2d7f05388c4d7b3184`.
Source package SHA-256:
`992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147`.
These are local parser observations, not REA or retail runtime evidence.

**OBSERVED (Rust decoder, both pinned package variants):** The final reader passes
all **85,186** prototype/default files in each package using **5,659** decoded
blueprints, with **zero failures**. Both runs count **709,824** prototype nodes
including embedded values, **731,294** groups, **1,475,896** simple fields,
**247,819** list fields, **471,846** list elements and **1,947,742** total values;
maximum depth is **18**. These totals match the independent shape scanner.

| Private pin relative to project data root | SHA-256 of `Calligraphy.sip` | Evidence file under `evidence/` |
| --- | --- | --- |
| `pins/server-data-unverified-build/` | `992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147` | `prototype-smoke-server.txt` |
| `pins/alienware-candidate/Data/Game/` | `1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c` | `prototype-smoke-alienware.txt` |

The private Rust harness is `evidence/prototype-smoke.rs`. A focused structural
comparison of the sole differing prototype reports **three scalar asset-value
changes** and no other typed/header/group differences. Field IDs, values and
names stay in private `evidence/prototype-carnival-diff.txt`; no asset data is
embedded in this source tree. A changed design-state value does not by itself
prove how the running client or server handles that mission.

## Source and licence ledger

MHServerEmu, <https://github.com/Crypto137/MHServerEmu>, local root
`/home/blizz/dev/marvel-heroes/MHServerEmu`, commit
`8fedb1a35922adfc9bff77e54f5fc3b6956073c9`, **AGPL-3.0**:

| Source | Facts studied |
| --- | --- |
| `docs/GameData/Calligraphy.md` | Prototype/group/value physical layouts and tags. |
| `src/MHServerEmu.Games/GameData/Calligraphy/CalligraphyReader.cs:133` | Header flags and optional reference encoding. |
| `src/MHServerEmu.Games/GameData/Calligraphy/CalligraphySerializer.cs:24` | Top-level requirements, group ordering and distinct ordinary/property schema contexts. |
| `src/MHServerEmu.Games/GameData/Calligraphy/CalligraphySerializer.cs:1134` | Scalar representation and differences between serialized values and runtime coercion. |
| `src/MHServerEmu.Games/GameData/Calligraphy/CalligraphySerializer.cs:1251` | Embedded-reference handling; compared with property and list paths at lines 1302, 1521 and 1570. |
| `src/MHServerEmu.Games/GameData/Calligraphy/Blueprint.cs:171` | Ordered parent lookup; mutable property designation at lines 28 and 280; admitted list tags at line 353. |

All code in this project module is newly written. Reference source was studied
for format facts and bounds of supported claims, not copied into this module.

The independent scanner used Debian `liblz4-1 1.10.0-8`; its library code is
**BSD-2-Clause**, as recorded in `/usr/share/doc/liblz4-1/copyright` (inspected).
Upstream: <https://github.com/lz4/lz4>. It is a validation tool, not a Rust runtime
dependency. The owner's Calligraphy packages are proprietary Gazillion data,
read privately for interoperability and excluded from git and build archives.
