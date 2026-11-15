# Native Level link and URL prefix

`upk::level::LevelObject` reads the established `Engine.Level` native prefix
through its actor, model-component and sequence links for UE3 **868/3**. It
preserves exact URL string framing and all later native bytes. It does not load
actors, follow URLs, initialize a Level or execute a sequence.

Admission requires the exact imported `Engine.Level` identity, the supported
version profile and the existing four-byte UObject prefix without a state
frame. [ENTRY_PACKAGE_OBJECTS.md](ENTRY_PACKAGE_OBJECTS.md) records that shared
prefix contract. [UPK_WORLD.md](UPK_WORLD.md) establishes the World link to the
persistent Level.

## Proven native prefix

**VERIFIED (static):** the complete native Level serializer is
`0x140bc0690..0x140bc0dba`, 1,834 bytes. After the shared UObject serializer, its
unconditional leading archive operations occur in this order:

| Native Level offset | Serialized representation | Retained role |
| --- | --- | --- |
| `+0x70` | Signed four-byte object reference | Actor-array owner/self-reference |
| `+0x60` | Signed count, then ordered four-byte references | Actor array |
| `+0x78` | FURL serialization below | Exact URL metadata |
| `+0xd0` | Signed four-byte object reference | Model |
| `+0xd8` | Signed count, then ordered four-byte references | ModelComponents |
| `+0xe8` | Signed count, then ordered four-byte references | Sequence array |
| `+0xf8` and subsequent fields | Not decoded by this increment | Entire remaining native span |

The prefix ends immediately after the sequence array. The remaining serializer
contains version-dependent native containers, bulk data and later object links.
Their boundaries have not been recovered here, so those later links are not
included in the returned dependency list. Even an empty retained suffix would
not prove that a complete Level payload was valid.

**OBSERVED:** bounded native bindings support the roles above:

- Base construction at `0x1408359b3` initializes `+0x70` with the Level's own
  pointer. This is an array-owner/self slot, not a recovered World link.
- Native `AActor::AllActors` registration leads to iterator `0x14083b0e0`,
  which reads Level count `+0x68` and actor storage `+0x60`. REA covers the
  complete 438-byte iterator body; no actor-iteration behavior is implemented.
- World construction stores a newly constructed Model into Level `+0xd0` at
  `0x140e3a2f7`. A native diagnostic explicitly names
  `Level->ModelComponents.Num()` and obtains its count from `+0xe0`, identifying
  the array beginning at `+0xd8`.
- The owned `AWorldInfoexecGetGameSequence` registration leads through its
  wrappers to `0x140bb6cd0`. Its complete 83-byte body returns the first pointer
  from Level `+0xe8` when the count at `+0xf0` is nonzero, otherwise null. This
  does not imply execution or traversal of that sequence. A null first element
  must not be replaced by searching for another nonnull element.

The no-explicit-Level getter path uses the runtime World's current-Level field.
Following a saved World→PersistentLevel link establishes a serialized package
relationship; it does not establish runtime current-Level initialization.

## FURL framing and preserved strings

**VERIFIED (static):** helper `0x140e36160..0x140e361f6`, 150 bytes, serializes
the following fields. The native FURL constructor and its default-setting code
bind Protocol, Host, Map, Portal and Port to these offsets. Names/meanings for
the string array and the final word remain unproved.

| FURL offset | Serialization order | API field |
| --- | ---: | --- |
| `+0x00` | 1 | `protocol` FString |
| `+0x10` | 2 | `host` FString |
| `+0x24` | 3 | `map` FString |
| `+0x44` | 4 | `portal` FString |
| `+0x34` | 5 | `extra_strings`, signed count followed by FStrings |
| `+0x20` | 6 | `port`, four raw bytes |
| `+0x54` | 7 | `word_54`, four raw bytes |

The complete 818-byte FString serializer at `0x140776fa0` reads a signed count.
Nonnegative counts consume that many bytes, which the native load path widens
individually to 16-bit units. Negative counts consume twice the absolute count
as 16-bit units. The complete 340-byte helper at `0x1406e51b0` reads a signed
string-array count and invokes the same serializer once per element.

`LevelString` retains the original signed count, storage encoding, count/payload
spans and every encoded unit. It does not require UTF-8, a terminating zero or
valid surrogate pairs. Native runtime normalization of one-unit strings is not
performed. `LevelWord` preserves raw bits without imposing a boolean meaning
or a port range. URL metadata is not interpreted as a travel request.

## API and bounds

`parse_export` takes a zero-based export index. Object references retain the
package convention: zero is null, positive is a one-based export and negative
is a one-based import. Every nonnull reference is checked against the package
tables. `links` preserves serialization order, nulls and duplicates; the
`actors`, `model_components` and `sequences` ranges select their slices of that
vector. Each link, array, string, word and unparsed suffix has an exact
export-relative span.

Negative array counts and truncated fields are rejected. Full reference-array
byte lengths and minimum string-array lengths are checked before reserving
storage. Explicit inspection limits are 65,536 references per array, 4,096
extra URL strings, and 1,048,576 units per string. These are reader resource
limits, not claimed native format limits. Invalid data/bounds return
`InvalidData`; unsupported class/profile/state-frame/cap cases return
`Unsupported`.

## Paired original-package observations

Private root: `/home/blizz/.local/share/marvel-heroes-forever-rust/`.
The files are under
`pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/`.

| Package | SHA-256 | One-based Level export | Export bytes | Decoded native prefix | Opaque native suffix |
| --- | --- | ---: | ---: | ---: | ---: |
| `MarvelEntry.upk` | `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` | 1 | 704 | 77 | 615 |
| `MarvelFrontEndMap.upk` | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` | 11 | 1,243 | 122 | 1,109 |

**OBSERVED:** both have an empty property list ending at export offset 12.
Entry's native known/opaque logical spans are `[1093,1170)` and `[1170,1785)`;
FrontEnd's are `[26394,26516)` and `[26516,27625)`.

Entry's actor array contains WorldInfo and a null slot. FrontEnd's contains
WorldInfo, a null slot and PlayerStart. The latter also retains ten
ModelComponents and a sequence reference to export 34, `MainSequence`.
Entry's sequence array has one null element. These saved links do not establish
arbitrary startup behavior, nor prove the initial-map transition to the frontend.
Both owner/self references identify the Level export itself.

## Validation, evidence and remaining scope

**VERIFIED:** three focused reader tests pass for exact class/profile admission,
prefix truncation, count/reference bounds, nonempty arrays, raw byte and wide
string units, absent terminators, scalar bits and opaque suffix retention.
The paired original-package smoke agrees with the independent oracle on all
21 links, six reference arrays, eight strings, two string-array counts, four raw
words and all 1,724 opaque bytes. Both package hashes and 89 Rust
source/manifest hashes remain unchanged across the check. Product receipt
SHA-256: `d9d1b0bdc23e5c015c356e3e4800b3ea24c4ae9a6a7bbcc3d3f777131ffcfb1a`.

Original strings in these two prefixes use one-byte units; their extra-string
arrays are empty. Wide/noncanonical strings and nonempty string arrays have
native-code and synthetic coverage only. The independent oracle's 45 focused
checks are separate from retail observations. It consumes Rust-extracted bytes
and captured metadata, so decompression and name extraction share provenance.
The private wrapper compiles the actual new module by path with the real UPK API
while leaving the integrator-owned module hook untouched.

Private evidence root: `evidence/entry-level-native/`.

- `rea-evidence-bundle.json`: seven records exported, then reset with no retained
  target. No active binary was opened.
- `native-verification.json`, `verify_native.py`: six complete native bodies,
  3,663 bytes, independently matched to the pinned PE and unwind ranges. The
  original reference-array helper's prior 386-byte REA evidence is reused and
  rechecked. The preliminary FString entry window is superseded by its full body.
- `field-identities/verification.json`: field-role/name chains and 10,282
  independently checked bytes across 13 selected ranges; unresolved meanings
  remain explicit.
- `independent/verification.json`, `independent/oracle/oracle.json`: source and
  package pins, all known-prefix atoms, exact opaque suffixes and oracle checks.
- `product-check/verification.json`: focused command exits, unchanged
  source/package hashes and actual reader/oracle comparison.
- `source-ledger.json`, `MANIFEST.json`: exact source provenance and retained
  artifact inventory.

No native client, actor, script, renderer, server or network execution was run.
This increment does not validate the remaining native containers or completeness
of the Level's dependencies.

## Provenance

All new implementation and validation code is original. No new third-party
implementation source was read or copied.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The two owner package pins above | Proprietary; private interoperability study | Exact Level prefixes, metadata and reference identities |
| Owner's pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary; private interoperability study | Native serializer, strings, array loop, field roles and getter |
| Existing original UPK/object APIs, World reader, private native/bounds evidence and PE helpers | Original project code and findings | Admission, spans, reference conventions and independent verification |
| Installed REA 6.9.0 skill/tool schemas | Analysis tooling; REA MIT as previously recorded | Bounded target-free observations and evidence lifecycle |
| GNU objdump, Python and Rust standard tooling | Installed inspection/build tools; no implementation copied or linked | Independent inspection, oracle and focused checks |
