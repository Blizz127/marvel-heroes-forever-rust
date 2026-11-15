# Saved UClass metadata

`upk::class::ClassObject` is an original bounded reader for the observed UE3
868/3 UClass file profile. It exposes the actual terminal default-object
reference and preserves serialized fields, counts and export-relative spans.
It does not construct classes, load the referenced default, apply inheritance,
resolve effective flags, execute bytecode or initialize a frontend.

The generic `ObjectProperties` reader intentionally excludes UClass. No prior
product reader duplicated this grammar. This module promotes the retained
independently corroborated class-profile study while adding checked package
identity, resource budgets and typed access.

## API and representations

```rust
ClassObject::parse_export(&package, export_index, &serialized_export)?;
ClassObject::parse_export_with_limits(
    &package, export_index, &serialized_export, Limits::default(),
)?;
```

Both return `std::io::Result<ClassObject>`. `Field<T>` contains an exact value
and its `Span`; `Array<T>` retains the count field, complete array span and
ordered entries. Duplicate names and references are preserved. Nothing is
deduplicated into a runtime lookup table.

The reader retains:

- UObject NetIndex and UField Next; UStruct Super, ScriptText, Children,
  CppText, line and text-position words; separate loaded/serialized script
  sizes and an opaque serialized-script span.
- UState probe mask, unaligned two-byte label-table offset, flags and ordered
  name/reference function-map entries.
- UClass flags, Within, configuration FName, component default map, interface
  word pairs, four category arrays, ForceScriptOrder, class groups, narrow
  native-class-name framing, DLLBindName and terminal default reference.

No alignment padding is inserted after LabelTableOffset. Only the serialized
script size advances the cursor. The loaded size remains separate: earlier
owned Engine.Actor evidence has 970 loaded bytes versus 722 serialized bytes.
The four frontend classes tested here have both sizes zero.

`Interface` retains two **raw signed integer words**. Their target tables and
runtime interpretation are not established. This deliberately avoids the
older private checker's assumption that both words are object references.
Component defaults retain the corroborated name/object pair representation.
The four frontend classes and nine prior class profiles have empty component
and interface collections; nonempty layouts have source/synthetic coverage
only, not validation against an owned nonempty instance.

NativeClassName keeps count and raw payload span, including its trailing NUL.
Zero-length and one-byte-NUL strings remain distinguishable. Only the observed
narrow ASCII profile is admitted. Wide counts and non-ASCII content are
unsupported; a missing terminator is malformed. No text normalization is
performed. Other unknown flag bits and scalar words remain unchanged.

## Admission, identity and budgets

The reader requires package version 868/licensee 3, an in-range export whose
`class_index == 0`, exact metadata/payload length, no state-frame flag and the
observed NetIndex of -1. Matching a resolved short class name is insufficient
to establish intrinsic Class identity. The embedded Super must exactly equal
the export-table super reference.

Every field that is established as an object reference is range checked against
the actual import/export tables, including negative `i32::MIN` without signed
overflow. Every FName requires an in-range index and a nonnegative number.
For a positive terminal default reference, the target export's `class_index`
must equal this exact source class reference. A target with the same printed
name but another class fails. Null and imported defaults remain explicit
signed references and are not followed or converted into local defaults.
No CDO flag normalization or Default__ name search is used.

The complete payload must be consumed. Extra bytes are an unsupported layout;
truncation, invalid references/names, negative counts, mismatched metadata and
wrong local default identity return `InvalidData`. Unsupported versions,
preambles, native-name encodings, ForceScriptOrder outside 0/1, extra layout or
inspection-budget exhaustion return `Unsupported`.

Default inspection budgets are 64 MiB per export, 1 MiB for each script size,
10,000 entries per collection, 20,000 entries in aggregate and 1,024 native-name
bytes. These are resource limits, not format maxima. Export input additionally
respects the package reader's hard `MAX_EXPORT_BYTES`. Count multiplication,
offset additions and aggregate entry additions are checked; remaining bytes
are checked before vector allocation. A caller may tighten limits without
changing the file grammar.

## Actual package verification

The four selected class records from the
[movie-default study](FRONTEND_MOVIE_DEFAULTS.md) are parsed from original
package exports. References are one-based; indices are zero-based.

| Package / class index | Class | Bytes | Function-map entries | Terminal default reference / export-relative span |
| --- | --- | ---: | ---: | --- |
| GFxUI / 36 | GFxMoviePlayer | 1,018 | 75 | +38 / 1014..1018 |
| GFxUI / 145 | GFxAction_OpenMovie | 149 | 1 | +147 / 145..149 |
| MarvelGame / 8538 | MarvelGFxMoviePlayer | 2,026 | 159 | +8540 / 2022..2026 |
| MarvelGame / 8764 | MarvelGFxFrontEnd | 1,174 | 88 | +8766 / 1170..1174 |

**OBSERVED** complete saved metadata; **VERIFIED** byte/span correspondence.
The profile's broader field semantics remain independently corroborated
analysis, not a complete retail serializer implementation. The retained native
UClass serializer separately grounds the flags, configuration-name and
default-reference destinations. It does not establish effective default values.

Five focused synthetic tests pass: all truncations of the original fixture,
metadata/profile mismatches, reference and name bounds, exact local default
identity, negative counts, oversized/truncated arrays, aggregate budgets,
separate script lengths, duplicate entries, native-name framing and raw
interface-word preservation. These fixtures contain no game data. Only the
five new tests ran; unrelated compiled tests were filtered out.

The frozen original-package comparison checks 767 scalar/payload field records
and 32 array aggregates against the retained independent profile, covering all
4,367 class bytes without gaps or overlaps. It checks the actual package
metadata and exact local default class identities. Forty-eight nonnegative
count/ForceScriptOrder fields have an explicit representation normalization:
the older Python profile called them signed integers, while the product stores
their validated nonnegative values as u32. Values, widths and raw bytes agree;
this is not a native signedness claim. The initial strict-kind comparison is
retained separately and did not represent a product parser failure.

Private receipts are under
`~/.local/share/marvel-heroes-forever-rust/evidence/upk-class-reader/`.
`focused-check/receipt.json`, SHA256
`07e1caba11027c86311e9e22ad351130ec1e188f4d9d0564fb19224c1a3ad627`,
records the warning-free std-only compile and five passing tests. Frozen source
copies receive the module hook privately for validation; the integrator owns
the repository hook. The independent original-package comparison receipt is
`retail-check/receipt.json`, SHA256
`ec3d9c93bce4ae68ac9fcdb7fd14836762eea085a642e564343a8a889db4bee1`;
its complete comparison is `retail-check/frozen-run/comparison-normalized.json`,
SHA256 `ddbd9c5c3ae5032d2a1c20009ce33fa3ae8216bda8e0401dafc00b1b217111f6`.
It also checks 332 FNames and 351 object references against package identity.
All package/module hashes remain unchanged. A provisional earlier run whose
source changed during a test-only fixture edit remains retained separately;
the final frozen source hash is
`0c8173d757721d61d1df5b41fc4c158ad5efef6bd79a756abcece8457c68133c`.
No GPU, native client, handler, service or network test was run.

## Sources and provenance

All raw assets, extracts and reference output remain private. All new Rust code
and validation code are original. No implementation was copied or translated
from the external library.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner GFxUI.upk, SHA256 `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199`; MarvelGame.upk, SHA256 `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` | Proprietary owner-supplied build 1.52.0.1700, private compatibility study | Four actual class records and exact package identities |
| [UELib UClass.cs](https://github.com/EliotVU/Unreal-Library/blob/8236c39b79e30887a487ecf9b018204e430a5ebc/src/Core/Classes/UClass.cs), commit `8236c39b79e30887a487ecf9b018204e430a5ebc` | MIT, Eliot van Uytfanghe | Prior class-profile corroboration; narrow retained-source reread confirms component map type, raw interface integers and native-name field. No runtime/default implementation copied |
| `evidence/world-transform/defaults/class-default-link/check_profile.py`, `profile.json`, `sources.json`; `evidence/current-movie-defaults/package-audit/class-link/apply-profile.py`, `profile.json` | Original project analysis; extracted bytes proprietary | Existing independent profile and nine-plus-four observed class records |
| `evidence/current-movie-defaults/native.bundle.json`, record `ev_faca4d7535519ce6911cb773e40c8730974e1d201237fed857388740a282eae6` | Original REA evidence over owner executable SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Reused complete 640-byte UClass serializer proof; no new REA session needed for promotion |
| Original `upk.rs`, `upk/objects.rs`, `upk/world.rs`, `upk/level.rs`, FRONTEND_MOVIE_DEFAULTS and retained source ledgers | Original project work | Existing metadata, error/span conventions and documented limits |
| Rust compiler/standard library; Python standard library | Rust MIT/Apache-2.0; Python PSF | Compilation, original fixtures and byte comparisons; no dependency added |

The audit's exact source paths, hashes and narrow source excerpts are indexed
in `audit/product-and-native-audit.json`, SHA256
`9242ba05b65d869f5ebb2c4b2e0fe9fec914425eb07ab8ea601954807ef55ee2`.
The retained UELib source hash is
`237d5e196f3cf65a70bf2f4471ffdd55487f6ffe173a7306f46f242d2abc1764`.
