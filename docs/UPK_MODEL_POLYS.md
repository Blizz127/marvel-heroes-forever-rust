# Native Model and Polys prefixes

`upk::model::ModelObject` and `upk::polys::PolysObject` read bounded native
prefixes from exact `Engine.Model` and `Engine.Polys` exports in the pinned
UE3 **868/3** profile. They retain raw bits, record spans, package-local object
references and the undecoded remainder. Neither reader produces geometry or
loads an object. A successful prefix read does not validate its opaque tail.

Both use the existing `ObjectProperties` admission: an exact `Core.Class`
import directly under root `Core.Package Engine`, with no state-frame prefix.
The readers additionally require their own class and file/licensee versions.
They begin after the parsed property terminator, without assuming that every
property list is empty. See [ENTRY_PACKAGE_OBJECTS.md](ENTRY_PACKAGE_OBJECTS.md).

## Model

**VERIFIED (static):** the complete serializer at
`0x140c0e010..0x140c0e6b2` (1,698 bytes) first calls the established UObject
serializer, then emits the following native prefix in this order:

| Native member | Stored representation | API |
| --- | --- | --- |
| `+0xc5c` | Seven successive 32-bit words | `record_c5c` |
| `+0x98` | Encoded stride, signed count, raw records | `arrays[0]` |
| `+0xb0` | Encoded stride, signed count, raw records | `arrays[1]` |
| `+0x68` (Nodes) | Encoded stride, signed count, raw records | `arrays[2]` |
| `+0xd8` | Signed object reference | `link_d8` |
| `+0xc8` through `+0x19c` containers/fields | Extended prefix below | `native_fields` |
| `+0x4d4..+0x4e0`, then `+0x110` | Final words and counted records below | `final_fields` |
| Bytes beyond the established file layout | Undecoded, if present | `unparsed_native_tail` |

Each stride and count occupies four little-endian bytes. The established
strides are respectively **12, 12 and 64**. The vector-array helper is
`0x140dfdc90..0x140dfdde5` (341 bytes); the Nodes-array helper is
`0x140bf6e80..0x140bf6fc5` (325 bytes). Both compare the archive version and
licensee version against initialized global words at `0x14292cd70` and
`0x14292cd78`, whose pinned values are 868 and 3. In the ordinary file-load
branch at this profile, each helper reads a signed count and then reads
`count × encoded_stride` bytes. Saving or older versions take a different
element-serialization path, which these readers do not admit. Byte-swapping
and other archive modes are not modeled.

The checked reader requires the established encoded stride. This is an explicit
support boundary; the native routine itself uses the stored stride in its raw
byte count. Negative counts are invalid; counts above **1,048,576** are rejected
as an inspection limit, not a claimed native format limit. Multiplication,
offset addition and complete record payload bounds are checked before advancing.
No per-record allocation occurs. `record_span(index)` returns a checked,
zero-based export-relative record span.

The shared seven-word helper at `0x140e360c0..0x140e3615f` was established by
the [World reader](UPK_WORLD.md). It does not by itself establish scalar types.
All seven word patterns are retained, including possible NaNs and signed zero.
The optional bounds arithmetic study observes center/half-extents/radius math
written at receiver `+0xc5c`, but its receiver-to-Model identity remains
**INFERRED**. The reader therefore keeps this record opaque. Names for arrays
`+0x98` and `+0xb0` remain unresolved.

**VERIFIED (static):** the owned diagnostic
`Level->Model->Nodes.Num()` binds Model `+0x68` and its `+0x70` count to Nodes;
the associated code indexes native records at stride 64. The Model serializer
masks byte `+0x37` of each loaded Node with `0x1f`. The reader preserves the
stored byte without applying this runtime normalization. Native member names,
record field interpretation and usable BSP geometry remain separate work.

`link_d8` is validated against package tables and retained in place. It is the
owner slot of the following native array, initialized to self by construction;
both samples contain self references. The parser does not impose identity or
invent an outer-Level relationship. Later references remain in the opaque tail,
so this one link is not a complete Model dependency inventory.

### Extended Model containers

The extended reader continues from `link_d8` through the following native
fields. Native offsets identify slots; they do not invent member names.

| Native field | Ordinary 868/3 file representation | `native_fields` API |
| --- | --- | --- |
| `+0xc8` | Signed count, then 60-byte records | `records_c8` |
| `+0x80` | Bulk stride 24, count, raw records | `arrays[0]` |
| `+0x554` | Raw 32-bit word | `words[0]` |
| Count at `+0x558`, records at `+0x55c` | Signed count, then 24-byte records | `records_55c` |
| `+0x60` | Polys object reference | `polys_link` |
| `+0xe0`, `+0xf0` | Two bulk containers, stride 4 each | `arrays[1..3]` |
| `+0x548`, `+0x54c` | Two raw 32-bit words | `words[1..3]` |
| `+0x100` | Bulk stride 4 | `arrays[3]` |
| `+0x4d0` | Raw 32-bit word | `words[3]` |
| `+0x19c` | Bulk stride 36 | `arrays[4]` |

The `native_fields` substructure ends immediately after the `+0x19c` container,
before the conditional call at `0x140c0e49b` and final serialized
fields described below. The additional legacy container below archive version 686 is absent
from this admitted profile. These raw arrays are not asserted to contain object
references, float vectors, vertices, indices or renderer-ready geometry.

**VERIFIED (static):** helper `0x140bf0080..0x140bf0214` reads the `+0xc8`
count and calls record serializer `0x140befb30..0x140befcdc` once per element.
Each record occupies 96 bytes in native memory but **60 bytes on this file
path**: a four-byte object reference, six four-byte words, another object
reference, then seven four-byte words. Their native offsets are respectively
`+0`, `+0x08..+0x1c`, `+0x20`, and `+0x30..+0x48`. The `+0x48` word is present
from version 600. The extra `+0x4c/+0x50` words belong to the reference/archive
mode at archive `+0x1c`; they are not appended to the ordinary file record.

**VERIFIED (static):** scalar helper `0x1406d66b0..0x1406d6729` passes exactly
the caller's byte count to the ordinary archive operation, or reverses those
bytes in the swapping path. It does not add a field header. This closes the
60-byte record width without equating it to native memory size. The record API
retains both reference spans, all 13 raw words and its complete wire span.
Names such as Material and Actor remain unproved for those two native slots.

**VERIFIED (static):** the `+0x80` helper at `0x140bf7130..0x140bf7285`,
four-byte bulk helper at `0x140d6ad90..0x140d6aed5` and 36-byte bulk helper at
`0x140bf6fd0..0x140bf7125` use the same stored-width/count/raw-load framing
and 868/3 threshold words as the earlier containers. The Model caller has a
conditional 16-versus-24 width selection for `+0x80`; the checked reader admits
the observed 24-byte form only. A stored width of 16 returns `Unsupported`;
this is a declared support boundary, not a claim that native 16-byte records
cannot exist. Bulk counts retain the 1,048,576 inspection limit.

**VERIFIED (static):** the `+0x558` count loop uses native records at `+0x55c`
with stride 28. The wire order is an object reference, eight raw bytes from
record `+0x0c`, eight from `+0x14`, then four from `+0x08`: **24 bytes**.
The API keeps both 64-bit patterns and the final 32-bit pattern without boolean,
bitset, index or float normalization. All record payload bounds are checked
before allocating the retained record vectors.

The registered Model constructor initializes **64** inline records, advancing
28 bytes from the `+0x55c` region. The final record ends at `+0xc5c`, the next
independently serialized field; record 64 would overlap it. The reader rejects
counts above 64 as `InvalidData`. This is defensive admission based on the
native storage layout, not a claim that the native serializer clamps or rejects
malformed counts. The dynamic `+0xc8` array instead uses an explicit 65,536
record inspection limit, with `Unsupported` above that limit. Negative counts
and truncated known records are `InvalidData`.

**VERIFIED (static):** native Model initialization constructs Polys through its
typed allocator and registered constructor, then stores it at Model `+0x60`
at `0x140bfc5c2`; the alternate path stores null. This supports the Polys link
role. The reader checks the serialized reference range without imposing an
unproved nonnull or referent-class invariant.

### Final Model fields

`final_fields` retains four `ModelWord` values at native offsets
`+0x4d4/+0x4d8/+0x4dc/+0x4e0`, followed by `records_110`, a counted array of
`Model110Record`. The four words are serialized for archive version 600 and
newer. The older initialization path is excluded by the admitted 868 profile.

**VERIFIED (static):** array helper `0x140bf0220..0x140bf03a0` reads a signed
four-byte count and calls `0x140b41560..0x140b41762` for each record. Native
records occupy 28 bytes, but the 868 file path stores **36 bytes**, comprising
nine four-byte words in this exact order:

| Wire word | Native role |
| ---: | --- |
| 0 | Input to bit 0 of the native record's first word |
| 1 | Input to bit 1 of that word |
| 2 | Raw field at record `+0x18` |
| 3 | Input to bit 2 of the first word |
| 4–8 | Raw fields at `+0x04/+0x08/+0x0c/+0x10/+0x14` |

The first three stored words are included from version 652; bit 2 and the
`+0x04` field from version 600; the `+0x08` field from version 636. All apply
at 868. The last three scalar words are unconditional. There are no object
references in this record serializer. It uses the established exact-width
scalar helper, without additional field framing.

Native loading masks each flag input with `& 1` before inserting its respective
bit. The reader preserves all nine raw word patterns in `Model110Record.words`
and does not normalize or reject nonbinary flag inputs. It retains count and
record spans, rejects negative counts, applies the 65,536-record inspection
limit, and validates the entire `count × 36` payload before allocating records.
Zero records are allowed. Any bytes after the last known record remain in
`unparsed_native_tail`; an empty suffix is not required for admission.

**VERIFIED (bounded static):** the intervening conditional helper at
`0x140bf7290..0x140bf762f` modifies the in-memory Model `+0x4d0` value and
temporary storage. Its body has no direct archive or indirect calls. Its direct
dependencies include diagnostic and allocator wrappers; those wrappers reach
global logging/allocation backends. Whole-transitive-graph absence of I/O was
not proved. The reader does not execute that helper or replace the saved
`+0x4d0` word with a derived runtime value. The known serialization sequence
resumes with the four words above and ends after the `+0x110` array.

## Polys

**VERIFIED (static):** the complete serializer at
`0x140b61d00..0x140b61eb0` (432 bytes) calls UObject serialization and uses this
ordinary file header:

| Native payload offset | Width | Representation | API |
| ---: | ---: | --- | --- |
| 0 | 4 | Signed record count | `count`, `count_span` |
| 4 | 4 | Secondary stored word | `secondary_word`, `secondary_span` |
| 8 | 4 | Signed array-owner reference at native `+0x70` | `owner_reference`, `owner_span` |
| 12 | Remaining bytes | Undecoded polygon serialization | `unparsed_native_tail` |

On save, both initial words come from the same current array count. On load,
the first count drives array initialization and iteration; the second word is
not subsequently used. It is not named capacity, required to equal the first
word, or interpreted as a separate record count.

Each native record occupies `0x154` bytes in memory and is serialized through
helper `0x140bf11e0`. This does **not** establish a fixed wire stride. The
polygon serializer has not been recovered by this increment. The reader checks
a nonnegative count and the owner-reference range, retaining all remaining
bytes without count-to-length or record-EOF validation. Because it performs no
count-driven iteration or allocation, it admits every nonnegative signed count,
including `i32::MAX`. This is header admission only.

**VERIFIED (static):** constructor `0x140800e60` stores the object itself into
the array-owner slot at Polys `+0x70`. The API retains the serialized reference
without requiring it to equal the selected export.

## API and actual package observations

Both entry points are `parse_export(package, export_index, bytes)` with a
**zero-based** export index. Object references use 0 for null, positive
one-based exports and negative one-based imports. All spans are relative to the
complete decompressed export; `Span::logical_offset` provides the package
logical address. Invalid counts, bad references and truncated known fields
return `InvalidData`; unsupported classes/profiles/strides/state-frame prefixes
or inspection limits return `Unsupported` through the relevant reader.

All private paths below are relative to
`/home/blizz/.local/share/marvel-heroes-forever-rust/`.
The original maps are under
`pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/`.

| Package | SHA-256 |
| --- | --- |
| `MarvelEntry.upk` | `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` |
| `MarvelFrontEndMap.upk` | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` |

**OBSERVED:** exactly one Model and one Polys export occur in each map. Each
has NetIndex -1 and an empty property stream terminating at export offset 12.

| Package/object | One-based export | Logical offset | Export bytes | Native prefix bytes | Opaque bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Entry Model | 2 | 1,785 | 228 | 216 | 0 |
| Entry Polys | 3 | 2,013 | 24 | 12 | 0 |
| FrontEnd Model | 13 | 27,705 | 7,540 | 7,528 | 0 |
| FrontEnd Polys | 26 | 37,011 | 24 | 12 | 0 |

Both Model seven-word records are all zero. Entry's array counts are 0/0/0;
FrontEnd's are **6/16/12**, including actual nonempty raw bulk records. Their
`+0xd8` references are respectively 2 and 13. Both Polys counts and secondary
words are zero, with self references 3 and 26. No nonempty original Polys
record stream was observed in these two samples.

**OBSERVED:** Entry has no `+0xc8` or `+0x55c` records and all five added bulk
arrays are empty. FrontEnd has 12 `+0xc8` records: six first references resolve
to `RemoveSurfaceMaterial` and six to `DefaultMaterial`, both `Engine.Material`
imports. All second references are null. Its added bulk counts, in native field
order `+0x80/+0xe0/+0xf0/+0x100/+0x19c`, are **144/78/7/0/48**. Its three
`+0x55c` records also have null references. The Polys links are respectively 3
and 26. These concrete referents do not establish universal record member names.

The extension through `+0x19c` accounts for export `[12,136)` in Entry and
`[12,7484)` in FrontEnd. The final fields consume the remaining 92 and 56 bytes:
Entry's final array count is 2 at export offset 152, followed by 72 record
bytes; FrontEnd's count is 1 at offset 7,500, followed by 36 record bytes.
Their known native spans now end at 228 and 7,540 respectively, exactly at the
export boundaries. No bytes in these two Models remain opaque or are treated
as padding. This is serialized coverage of these samples, not geometry or
runtime behavior parity.

### Bounded StaticMesh dependency lookup

**OBSERVED:** complete captured Entry tables (20 names, 6 imports, 5 exports)
and FrontEnd tables (199 names, 45 imports, 37 exports) have no StaticMesh names.
The separate lookup of 29 already selected saved native references reaches no
StaticMesh target. This is scoped to these captured tables and links; it does
not establish runtime absence or cover arbitrary transitive assets.

An existing, separately captured Asgard collection at object reference 1341
saves 45 component references whose byte-checked `StaticMesh` properties resolve
to 18 distinct explicit mesh exports: references 1322 through 1338 and 1340.
They include `asgard.asgard_boatbridge`, `limbo_rocks.limbo_rocks_e` (15 uses),
and `vfx_water.meshes.msh_asgard_waterfall_top` (14 uses). No Entry/FrontEnd
bridge to this collection was established. Exact asset paths, all ordered uses,
property spans and source hashes are retained in
`evidence/model-tail-native/independent/static-mesh-links/SUMMARY.md` and
`findings.json` (SHA-256
`b7aef1b0f2a16d1e3cc16458b9ff56c660ecd2db0c4d5077d88774f58d3e80a1`).
This lookup reused named prior receipts; it performed no arbitrary package scan
or new mesh payload read.

## Evidence and validation

**VERIFIED (final Model fields):** five focused Model tests pass, including every
known-prefix truncation, negative/oversized final counts, full-payload preflight,
nonbinary raw flag inputs, zero-count arrays and arbitrary trailing bytes.
The paired product check matches all 424 output rows, retaining the prior prefix
comparison and adding all 35 final raw words, two counts and three final record
spans. Both original exports end exactly at their observed boundaries. The
independent final-array oracle passes 29 focused checks. The new Model source
and all 254 frozen baseline files remain unchanged; no baseline or Polys suite
ran. These are static package-reader results, not retail-client execution.

Private final-field evidence is under `evidence/model-final-tail-native/`.
`native-support/receipt.json` has SHA-256
`f46a289fcec20b5b55e4c3a48c60b64ee8fcd9b0468373165bdfe5f42d80b26e`.
It verifies all 1,825 bytes of the three new REA bodies, complete chained
boundaries, the reused Model/scalar helper proofs, version gates, record widths
and the conditional helper's explicitly limited side-effect coverage.
`rea-bundle.json` has SHA-256
`6ab6c2490a2a0ca83b699138dc6e9929d133d15fd9217b993be1490b51780a24`;
all three records were exported before reset with no target or cleanup pending.
The final-array oracle at `independent/final-oracle/oracle.json` has SHA-256
`bc84de23d9278d769dd6293b097ce7cfc6a8e6e2e166fab1339990034d0462d8`.
The product receipt at `independent/product-check/verification.json` has
SHA-256 `dd2691908cd0ac6377afa58cbe980c270cda03fac96d4469bc3ac3784e8398bc`;
the joined native/oracle/product receipt at `independent/verification.json` has
SHA-256 `063f0bce42a78c74c4ae36d6bf3bca1d9982f056bc7c181b19f184c3c16538d5`.
The private frozen baseline is `2cb32808ad84aee344cd444703af095bbbe31773`.

**VERIFIED (extension through `+0x19c`):** five focused Model tests pass for that known
prefix, every truncation, each reference slot, raw-field retention, all bulk
headers, dynamic-count limits and the inline table's 64/65-record boundary.
The 16-byte `+0x80` variant rejects explicitly. The original-byte product check
agrees with the independent oracle on both Models across 411 output rows:
16 bulk descriptors, 311 actual record spans, 32 record-index rejections,
12 `+0xc8` records, 3 inline records, 31 references, 181 raw 32-bit words and
6 raw 64-bit words. Both opaque suffixes match exactly.

The focused test, build and actual-package run all exit zero. The new Model
source hash and all 243 frozen baseline files remain unchanged; no Polys or
baseline suite ran. The independent oracle passes 56 focused checks. Product
receipt: `evidence/model-tail-native/independent/product-check/verification.json`,
SHA-256 `f3a0cd90edc3a684a559cbb238a066368b9ca0c1095bcbc272f84d83fda0fbb6`.
These checks establish file-reader behavior against the two owned samples,
not native execution, geometry preparation or runtime parity.

The extension's private evidence root is `evidence/model-tail-native/`.
Its `rea-bundle.json` retains six complete successful observations, independently
checked against 1,960 owned PE bytes and all chained unwind fragments. The
complete 1,698-byte Model serializer is reused by exact hash. The native audit
receipt at `native-verification/receipt.json` has SHA-256
`437d4c9e18d151adad2d6705459eff3fa200597d949e4b1408e2a09060aba8b8`.
It verifies the wire sequence, fixed-table extent, ordinary-file record widths,
version gates and the explicit stop boundary. All six REA records were exported
before investigation reset, with no target open or cleanup pending.

`field-identities/` retains the direct Polys construction proof and unresolved
member names. Candidate geometry indexing relationships from other functions
remain **INFERRED** as Model class relationships; they are not applied by the
reader. `independent/` retains the unchanged original captures, frozen baseline
`40d8a705ef07488b46148fa8cdb4f2ce2d7ea0dd`, the extended byte oracle, product
comparison and bounded StaticMesh lookup. Its source ledger registers the named
prior Asgard/component receipts used without reading additional mesh payloads.

**VERIFIED (initial prefix increment):** six focused product tests pass: all known-prefix truncations,
negative/over-limit counts, unsupported strides, invalid object references,
exact class/profile admission, state-frame rejection, raw-bit preservation,
record indexing and opaque-tail retention. Polys checks also preserve unequal
secondary words and accept a nonnegative maximum count without allocating or
claiming that polygon records exist.

**VERIFIED (initial prefix increment):** the actual four-export product comparison matches the independent
oracles on all 68 output rows, six array descriptors, 34 actual record spans,
12 record-index boundary results, all fixed words/references and exact opaque
tails. Both original package hashes, all 237 baseline source files and both
new reader source hashes remain unchanged. The private wrapper compiles these
actual modules by path against frozen baseline `6650da5`; it leaves the
integrator's module hooks untouched. The product receipt SHA-256 is
`2ac14980bd52c67f93edf983984b989914cce3dac0b064cc5338cdc8cc9cff3b`.
The separate Python oracles pass 39 Model and 18 Polys focused checks. Only the
six new product tests ran; no baseline/full suite or retail execution ran.

Private evidence root: `evidence/entry-model-polys-native/`.

- `rea-bundle.json` retains five records: complete Model, Polys and Nodes-helper
  observations, a successful 512-byte vector-helper window, and a later
  341-byte vector request that reached its 30-second provider deadline.
  The bundle was exported before clearing all five investigation records;
  no binary target was open or retained. Bundle SHA-256:
  `22687ffefd9aa9552543ad7c9a3759f84264c730edbbe4d91f19759620413420`.
- The vector proof uses only the complete 341-byte function within the earlier
  successful window. Rows outside that function, including incomplete rows at
  the end of the larger window, support no claim. The deadline result retains
  complete captured output but has unknown process completion; it is not
  reported as an exit-0 observation.
- After that deadline, the error reported no retained cleanup owner; REA
  reported no pending cleanup, and process inspection found no outstanding
  Rizin command against this target. The following Nodes call used the
  recommended increased 60-second per-call budget and completed successfully.
  No shared settings, provider registrations or other sessions were changed.
- `native-verification/` verifies all 2,796 instruction bytes against the
  unchanged PE, complete chained unwind ranges and version constants. The
  previous World seven-word proof and entry class-vtable receipts are reused by
  exact hash. Its receipt SHA-256 is
  `3f1b583cf133e27ec4ce4d50e924aa2a321fa8aa848704c784893cab60f37cf8`.
- `field-identities/` contains the bounded Nodes/self-owner study and the
  explicitly unresolved bounds/member-name questions.
- `independent/` contains the frozen baseline `6650da5bd8567e9b4dbc5980d3b59624b7e5da0e`,
  original extracted exports, separate raw-byte oracles and original-input
  hashes. The oracle shares package extraction and dictionary provenance with
  the existing Rust package reader; it independently decodes the new native
  prefixes.
- `independent/product-check/` records focused validation of the actual new modules against
  these oracles. `source-ledger.json` and `MANIFEST.json` inventory source use
  and private evidence.

No native client, renderer, scripts, gameplay, network connection or runtime
geometry was executed. These are serialized prefix readers; full Model/Polys
loading and retail geometry parity remain unverified.

## Provenance

All implementation, oracle and validation glue is original. No additional
third-party implementation source was read or copied for this increment.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The two map pins above | Owner proprietary files; private interoperability study | Actual headers, raw array records, tails and object references |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Owner proprietary client; private interoperability study | Native serializers, branch constants, class construction and diagnostic identities |
| Existing original project object/World/Level/Model readers and private PE helpers/evidence | Original project source/findings | Admission, span API, byte mapping, prior helper proof and fixtures |
| Named prior Asgard collection/component receipts under the private evidence root | Original project findings over owner proprietary bytes; no redistribution | Bounded saved StaticMesh dependency identities; exact 70-file ledger in `model-tail-native/independent/static-mesh-links/source-ledger.json` |
| REA 6.9.0 and installed skill, native/evidence/recovery guides and tool schemas | REA MIT; analysis tooling, as registered in [PROVENANCE.md](PROVENANCE.md) | Target-free observations, retained evidence and deadline recovery |
| Rizin, GNU objdump, Python and Rust tooling | Installed analysis/build tools; no implementation copied or linked | Native inspection, byte oracle and focused checks |

## Diagnostic CLI integration

`mhf upk model <package.upk> <export-name|1-based-index>` reports raw array
counts/strides/spans, seven retained words, the native +0xd8 reference and opaque
tail length. `mhf upk polys` uses the same selector form and reports only the
validated header and remaining bytes. It does not validate polygon records
against the stored count or prepare geometry.

**VERIFIED dev16 CLI smoke:** Model exports 2/13 and Polys exports 3/26 from
Entry/FrontEnd pass through their respective public commands. Selecting the
opposite class rejects with the correct error. Receipt:
`evidence/build-0.1.0-dev.16-smoke.json`. Original inputs remain private; no object
execution, service access or geometry interpretation was performed.
