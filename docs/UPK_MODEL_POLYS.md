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
| `+0xc8` container and later fields | Undecoded | `unparsed_native_tail` |

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
| Entry Model | 2 | 1,785 | 228 | 56 | 160 |
| Entry Polys | 3 | 2,013 | 24 | 12 | 0 |
| FrontEnd Model | 13 | 27,705 | 7,540 | 1,088 | 6,440 |
| FrontEnd Polys | 26 | 37,011 | 24 | 12 | 0 |

Both Model seven-word records are all zero. Entry's array counts are 0/0/0;
FrontEnd's are **6/16/12**, including actual nonempty raw bulk records. Their
`+0xd8` references are respectively 2 and 13. Both Polys counts and secondary
words are zero, with self references 3 and 26. No nonempty original Polys
record stream was observed in these two samples.

## Evidence and validation

**VERIFIED:** six focused product tests pass: all known-prefix truncations,
negative/over-limit counts, unsupported strides, invalid object references,
exact class/profile admission, state-frame rejection, raw-bit preservation,
record indexing and opaque-tail retention. Polys checks also preserve unequal
secondary words and accept a nonnegative maximum count without allocating or
claiming that polygon records exist.

**VERIFIED:** the actual four-export product comparison matches the independent
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
| Existing original project object/World/Level readers and private PE helpers/evidence | Original project source/findings | Admission, span API, byte mapping, prior helper proof and fixtures |
| REA 6.9.0 and installed skill, native/evidence/recovery guides and tool schemas | REA MIT; analysis tooling, as registered in [PROVENANCE.md](PROVENANCE.md) | Target-free observations, retained evidence and deadline recovery |
| Rizin, GNU objdump, Python and Rust tooling | Installed analysis/build tools; no implementation copied or linked | Native inspection, byte oracle and focused checks |
