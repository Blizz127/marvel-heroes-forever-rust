# Native World file payload

`upk::world::WorldObject` reads the native `Engine.World` payload for the pinned
UE3 **868/3** profile. It retains ordered package-local links, four exact
seven-word records, the reference-array count and spans, and any unknown trailing
bytes. It does not load linked objects or initialize a world.

Admission uses the existing `ObjectProperties` checks: a `Core.Class` import
directly beneath the root `Engine` package, with no state-frame prefix. This
reader additionally requires the exact `World` class and 868/3 version profile.
See [ENTRY_PACKAGE_OBJECTS.md](ENTRY_PACKAGE_OBJECTS.md) for the established
UObject/property boundary and native class identity evidence.

## Established file layout

**VERIFIED (static):** the complete native World serializer is
`0x140e474c0..0x140e47680`, 448 bytes. It first calls the shared UObject
serializer. The file-load branch then serializes the fields below, in order.
Offsets in the first column are relative to the start of the native payload,
after the terminating property `None`.

| Native payload offset | Serialized width | Native World field | Representation |
| ---: | ---: | --- | --- |
| 0 | 4 | `+0x80` | Signed object reference; persistent Level |
| 4 | 4 | `+0x88` | Signed object reference |
| 8 | 28 | `+0xa8` | Seven exact 32-bit words |
| 36 | 28 | `+0xc4` | Seven exact 32-bit words |
| 64 | 28 | `+0xe0` | Seven exact 32-bit words |
| 92 | 28 | `+0xfc` | Seven exact 32-bit words |
| 120 | 4 | `+0x118` | Signed object reference |
| 124 | 4 | Array at `+0x380` | Signed element count |
| 128 | `count × 4` | Array elements | Ordered signed object references |

The `+0x88` reference is included for archive versions at least 575. A separate
legacy reference is only serialized below version 496. The 868 profile therefore
uses exactly the layout above. A block of runtime-only fields is skipped when
archive `+0x14` indicates loading or `+0x18` indicates saving; those fields are
not inserted into the package reader.

**VERIFIED (static):** helper `0x140e360c0..0x140e3615f`, 159 bytes, serializes
seven successive four-byte fields. Their scalar types and meanings are not
established here. The reader preserves all bits, including patterns that could
represent NaNs or signed zero, without assigning camera, position or viewport
semantics.

**VERIFIED (static):** the reference-array helper starts at `0x140bae2b0` and
ends at `0x140bae432`; its three chained unwind ranges cover 386 bytes. The load
path reads a four-byte count and loops through archive object-reference
serialization once per element. The reader validates the complete encoded
array length before reserving storage, rejects negative counts and counts above
the explicit 65,536-reference inspection limit, and checks every nonnull
reference against the package import/export tables.

**OBSERVED:** native construction at `0x140e39ed0` creates a Level named
`PersistentLevel` and stores its pointer into World `+0x80` at `0x140e3a1f2`.
The complete 1,375-byte construction body was retained through REA and checked
against the owned PE. This supports the `WorldLinkSlot::PersistentLevel` role.
An additional bounded static study observes a FaceFXAnimSet reference stored at
`+0x88`; its exact member name remains unproved, so the API retains that slot's
native offset. Names for `+0x118`, array `+0x380`, and the four records remain
unresolved. Exhausting these bounded lookups is not proof that names or metadata
are absent elsewhere.

## Actual package observations

All private paths below are relative to
`/home/blizz/.local/share/marvel-heroes-forever-rust/`.
Both maps are under
`pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/`.

| Package | SHA-256 | One-based World export | Export logical offset |
| --- | --- | ---: | ---: |
| `MarvelEntry.upk` | `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` | 4 | 2,037 |
| `MarvelFrontEndMap.upk` | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` | 36 | 50,958 |

**OBSERVED:** both World exports are 140 bytes: a four-byte NetIndex, an empty
property list ending at export offset 12, and a 128-byte native payload. Their
persistent-Level references are respectively export 1 and export 11, both
resolving to `theworld.persistentlevel` with class `Engine.Level`. The other two
fixed references are null, the final arrays are empty, and no bytes remain after
the established native layout. Their raw records differ and are preserved.

The reader's link sequence preserves all three fixed slots followed by every
array element, including nulls and duplicates. Each link and record carries its
exact export-relative span. `unknown_tail` explicitly retains any later bytes;
nonempty unknown tails are not represented as decoded World fields.

## Focused validation

**VERIFIED:** three focused reader tests pass for class/profile admission,
state-frame rejection, every truncation of the synthetic layout, malformed
counts, extreme or out-of-range references, ordered nonempty arrays, exact raw
bits, and unknown-tail preservation.

**VERIFIED:** the actual two-package smoke agrees with the independent Python
oracle on all six fixed references, eight record spans and all 56 record words,
array counts and final boundaries. Both persistent-Level links resolve through
the existing project reference API. Both input hashes and 86 Rust
source/manifest hashes remain unchanged across the check. The product receipt
SHA-256 is
`aee1aca5efaa4642d67dd2c40562ba7a9b1a7f80490f745210940a8a7663622c`.

The private validation wrapper compiles the actual new module by path and
reexports the real project UPK API, leaving the integrator's module hook
untouched. The independent oracle has 27 additional focused synthetic checks.
It consumes Rust-extracted exports and the captured metadata dictionary, so
package decompression and dictionary extraction are shared provenance. Both
original arrays are empty: nonempty-array behavior is backed by the native
loop and synthetic checks, not an observed nonempty retail array.

Private evidence root: `evidence/entry-world-native/`.

- `rea-evidence-bundle.json`: five REA records exported, then investigation
  reset with no retained target. No active binary was opened.
- `native-verification.json`, `verify_native.py`: four complete native bodies,
  2,368 bytes, independently matched to PE bytes and chained unwind ranges.
  An initial count-based array-helper window emitted invalid decoder rows at a
  block boundary; it is retained but superseded by the complete explicit
  386-byte observation and supports no semantic claim.
- `independent/verification.json`, `independent/oracle/oracle.json`: paired
  original exports, exact source/package hashes, field oracle and synthetic
  checks. The frozen source is commit
  `52d2876ed88e7dfa8a83d54ddb4d0511f5b2f56c`.
- `product-check/verification.json`: focused build/test/run exits, exact source
  hashes, unchanged package hashes and oracle comparison.
- `field-identities/verification.json`: bounded optional field-role study,
  explicit unresolved lookups and source provenance.
- `source-ledger.json`, `MANIFEST.json`: exact source paths, ownership/licence,
  hashes and retained-artifact inventory.

No native client, script, gameplay, renderer or network execution was run.
Following a persistent-Level reference does not initialize that Level or prove
the entry-map transition to the frontend. This increment is a file reader.

## Provenance

All implementation and validation code is original. No new third-party
implementation source was read or copied for this increment.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The two private map pins above | Owner proprietary files; private interoperability study | Actual native payloads and reference identities |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Owner proprietary client; private interoperability study | Native serialization branches, widths, array loop and persistent-Level binding |
| Existing original UPK/object/reference APIs, entry-package evidence and PE helpers | Original project source and findings | Class admission, retained spans, independent byte mapping and reference resolution |
| REA 6.9.0 and its installed skill/tool schemas | Analysis tooling; REA MIT as already recorded in [PROVENANCE.md](PROVENANCE.md) | Bounded target-free native observations, export and cleanup |
| GNU objdump, Python and Rust standard tooling | Installed inspection/build tools; no implementation copied or linked | Independent disassembly, oracle and focused checks |
