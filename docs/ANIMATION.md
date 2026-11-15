# Saved Loki animation data

`src/animation.rs` is an original bounded reader for the AnimSet and AnimSequence
exports observed in the owned PC 868/3 Asgard vista package. It decodes stored
translation and rotation keys and preserves their frame metadata. It does not
evaluate an animation or produce a rendered pose.

## Actual source and scope

**OBSERVED:** the private pin is:

`~/.local/share/marvel-heroes-forever-rust/pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelVista_AsgardHub.upk`

The complete file is 8,475,579 bytes, SHA-256
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.
It matches the source inventory under private
`evidence/alienware-distribution-inventory.json`. The fixed pin is read-only;
the ongoing full-copy destination is not used or modified by this increment.

Export indices below are zero-based. Signed object references in the reader keep
UE's saved convention: positive references are export index plus one; negative
references are imports; zero is null.

| Export | Observed role |
| --- | --- |
| 4–75 | 72 AnimSequence exports belonging to the local Loki AnimSet |
| 76 | AnimSet, 84 track names, 72 sequence references, 16 translation-policy names |
| 4 | SequenceName `idle`, 120 frames, saved length 4 seconds |
| 53 | Small single-key representative, 84 translation and 84 rotation keys |
| 70–73 | Four sequences with explicitly saved variable-key encoding |
| 0 | AnimNodeSequence whose saved AnimSeqName is `idle` |
| 162 | InterpTrackAnimControl with one saved key naming `idle` |
| 1316 | Vista SkeletalMesh with 73 reference bones; see `SKELETAL_MESH.md` |

**OBSERVED:** export 0 and the InterpTrack key independently name the unique
sequence at export 4. The InterpTrack key saves start time and animation offsets
zero, play rate one, looping false and reverse false. These are serialized
references, not evidence that the client has executed or activated either node.

**OBSERVED:** all 73 vista bone names have exact resolved-name matches among the
84 animation tracks. Eleven animation tracks have no counterpart in that mesh.
The AnimSet's saved PreviewSkelMeshName is `loki_traveling.loki_traveling`, while
the inspected skeletal export is `loki_vista.lokiandstaff_vista`. Name matching
does not establish compatible reference poses, retargeting, or the live binding.
The mismatch remains explicit.

## Observed storage

Each inspected animation object begins with a four-byte UObject net index,
followed by bounded tagged properties. The observed net index is -1. The AnimSet
ends after its property terminator. An AnimSequence then contains a signed raw
track count, zero in all 72 samples, followed by a signed compressed-byte count
and exactly that many bytes. No native trailer remains in these samples.
Nonempty raw tracks are currently unsupported; unclaimed native bytes retain an
explicit span instead of being reinterpreted.

Saved sequence properties include SequenceName, SequenceLength, NumFrames,
RotationCompressionFormat and CompressedTrackOffsets. Notifies occur in 28
sequences and remain ordinary retained property spans. Optional rate, looping,
additive and codec properties retain their presence or absence.

**OBSERVED:** every rotation format is `ACF_Float96NoW`. Every offset table
contains 336 signed integers: 84 ordered groups of translation offset,
translation key count, rotation offset and rotation key count. Offsets address
the compressed byte stream. Translation keys contain three little-endian
binary32 values. Observed rotation keys also contain three binary32 values.

The original decoder preserves X/Y/Z and reconstructs the nonnegative quaternion
W from `((1 - X*X) - Y*Y) - Z*Z`, clamping a nonpositive remainder to zero before
the square root. Each operation rounds to binary32. This order follows the owned
native decoder, superseding the reference viewer's initially studied grouped
sum (see the native correction below). It
does not conjugate, normalize, convert axes, compose parents or apply a bind
transform. Retail execution and its floating-point environment remain unverified.

**OBSERVED:** four sequences explicitly save `AKF_VariableKeyLerp`; the other 68
omit KeyEncodingFormat. All 72 omit TranslationCompressionFormat.
**INFERRED, caller-selected:** the validation profile interprets those omissions
as constant-key encoding and `ACF_None` translation storage. These choices are
consistent with the complete observed byte ranges and reference source, but
the retail class defaults have not been recovered from the client binary.

For variable-key channels with more than one key, frame indices follow the keys
at four-byte alignment. All inspected arrays use one byte per frame index
because their sequence frame counts are below 256. The source-described
two-byte branch at 256 frames is implemented and tested synthetically only.
Single-key channels have no frame array. Padding is included in the occupied
source span and its contents are not constrained: the actual variable-time
samples contain `0x55` padding. Explicit times must be strictly increasing and
less than NumFrames. Constant-key tracks retain uniform timing metadata;
endpoint, loop and interpolation semantics are not evaluated.

## API and boundaries

```rust,ignore
let set = AnimSet::from_export(package, set_index, &set_bytes)?;
let sequence = AnimSequence::from_export(package, sequence_index, &sequence_bytes)?;
let decoded = sequence.decode(&set, &DecodeDefaults {
    translation: Some(Compression::None),
    rotation: None,
    key_encoding: Some(KeyEncoding::ConstantKeyLerp),
})?;
let names = mesh.bones.iter().map(|bone| bone.name_text.as_str()).collect::<Vec<_>>();
let correspondence = set.bind_exact_names(&names)?;
```

There is deliberately no implicit default configuration. Every omitted format
needs an explicit caller value. A saved unsupported format remains unsupported;
a fallback never overrides it. The decoded result labels each effective format
as `Serialized` or `CallerSupplied`. Unknown property types/values remain in
the shared original `PropertyList`, with byte spans for later investigation.

Decode verifies package GUID, saved outer reference and AnimSet membership.
Callers must supply objects from the same opened package; a GUID is an identity
check, not a cryptographic integrity check. Track order follows TrackBoneNames.
The exact-name binding method rejects duplicate names, retains missing mesh
bones as `None`, and reports unused animation tracks. It does not apply
UseTranslationBoneNames, ForceMeshTranslationBoneNames or bAnimRotationOnly.

All spans are relative to their serialized export. Add `source.serial_offset`
using `Span::logical_offset` to locate a span in the logical decompressed UPK.
Each channel retains its original signed offset, key count, key-byte span,
complete occupied span and explicit frame-array span when present. The stream
bytes are retained with the metadata, avoiding an accidental later decode from
a different caller-supplied export buffer. Exact shared ranges are allowed;
partial overlap is rejected. Gaps remain reported as unclaimed spans.

Bounds include 64 MiB exports/streams, 4,096 tracks, 65,536 sequence references,
one million frames and two million decoded channel keys per sequence. Counts,
source ranges, float finiteness, alignment, object/name references, typed array
widths, membership, optional-property duplicates and time indices are checked
before use. Unknown codecs, per-track compression and nonempty raw tracks return
explicit errors. No package dependency is fetched.

No sequence time sampling, key interpolation, animation-tree evaluation,
notifies, skeletal pose composition, translation policies, retargeting, blending,
skinning or renderer integration is included. Additional codecs and the saved
defaults need further evidence.

## Focused validation and private evidence

**VERIFIED:** the original reader parses all 72 sequences and accounts for all
1,585,384 compressed bytes, with no stream gaps or native tails. It decodes
27,880 translation keys and 104,032 rotation keys under the explicit profile
above. This is asset-reader coverage, not gameplay or visual parity.

An independent Python reader uses the owned extracted bytes and separately
inventoried property metadata. Its six representatives are actual selected
`idle`, the smallest single-key example and all four explicitly variable-time
sequences. Exact XYZ bit patterns, decoded frame arrays, source spans and bone
name correspondences are compared with the Rust reader. Reconstructed W is
checked with separately rounded binary32 operations; W is not stored evidence.

**VERIFIED, initial grouped-W version:** the independent comparison matches 9,094 keys (2,092 translation,
7,002 rotation), all 27,282 saved XYZ bit fields, all 7,002 reconstructed W bit
fields, 1,008 channel spans / 6,048 span fields, 159 explicit frame arrays /
2,176 frame values, all 72 metadata profiles and all 73 mesh-name bindings.
Comparison receipt SHA-256:
`1e8004a07ba668e173973c723d788146567befef6beb19c81a899cffe253d1a8`.

Four focused tests cover single-key rotation storage, exact frame/source ranges,
every truncation of a small variable-time channel, the synthetic 255/256 frame
width boundary, invalid time order/ranges, bad offsets/counts, allocation budget,
nonfinite floats, missing/unknown codec fallback behavior and overlapping spans.
No full repository suite or live-client/runtime test is claimed.

All extracted exports, streams, key values, test binaries and detailed receipts
remain private under `~/.local/share/marvel-heroes-forever-rust/evidence/animation-study/`:

- `validation.json`: full pin hash/stability, code hash, smoke/test results and
  hashes of supporting receipts.
- `independent/metadata.json` and `validation.json`: 72-sequence metadata and
  independently decoded representative streams, including saved activation refs.
- `independent/tracks-canonical.json` and `rust-comparison.json`: raw-field
  comparison and exact outcome; private scripts retain the repeatable procedure.
- `rust-profiles.tsv`, `rust-keys.tsv`, `rust-spans.tsv`, `rust-binding.tsv`:
  original-reader results, kept outside git.
- `source-ledger.json`: exact external reference paths, hashes and scope.

The initial `validation.json` SHA-256 is
`fc34f9912dc601ba9d32222c465211dd5f6600d45856e6af0d9f5727a737788b`.

REA's prior read-only session status is preserved at private
`evidence/skeletal-mesh-study/rea-session-status.json`. That process reported no
open target and unavailable Ghidra Java configuration. The initial reader increment
identified no native animation serializer/codec entry point; no shared target
or configuration was changed. Package parsing and source study are labeled as
such, not as REA decompilation evidence.

## Native Float96 W correction

**OBSERVED with REA:** the owned `MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
uses three sequential binary32 subtractions, then a positive-remainder check and
`SQRTSS`. The Float96 rotation function begins at `0x1407e27e0`; the single-key
subtractions are at `0x1407e281f`, `0x1407e2835`, `0x1407e283f`. Both endpoints
in its multi-key branch use the same order. The variable-key Float96 function
at `0x1407e5070` independently has the
same subtraction order in its single-key and both multi-key reconstructions.
This is a read-only compatibility
study of the owner's proprietary Gazillion client; no client code was copied
or executed. No shared REA target or configuration was changed.

The Rust change only replaces grouped-sum W reconstruction with the observed
ordered subtractions. It adds a synthetic rounding regression and leaves saved
formats, explicit caller defaults, time metadata and binding behavior intact.
No interpolation or class-default inference is added.

**VERIFIED:** sequential and grouped arithmetic differ for 1,468 of 7,002 checked
rotation keys: 91 of 344 single-key-channel keys and 1,377 of 6,658 multi-key-channel
keys. Maximum absolute difference is approximately `1.9548519e-5`; no key changes
between zero and nonzero W. All 7,002 corrected Rust values match independent
Python and an original scalar SSE instruction harness bit-for-bit. The harness
uses MXCSR `0x1f80`; it does not execute the retail client. All 27,282 XYZ bit
fields, 1,008 spans, 159 frame arrays / 2,176 frame indices remain unchanged;
profile, span and bone-binding TSVs are byte-identical. All 72 sequences still
decode, and all five focused decoder tests pass.

Private evidence is under `evidence/animation-runtime/`:

- `native/rea-constant-float-keys.json` and
  `native/rea-quaternion-tail-and-default-copy.json`: retained REA instructions;
  only the Float96 instruction ranges support this arithmetic correction.
- `native/rea-variable-float96.json`: the corresponding variable-key native
  instruction ranges, confirming the same W arithmetic for all observed profiles.
- `native/w-rounding-comparison.json`: before/after arithmetic counts, SHA-256
  `9ce8d4f3824d0c93cecdc57c1028932166ff68a26e4f95dc3f1e42b845e25a21`.
- `native/w-rounding-sse/comparison.json`: independent instruction-contract check,
  SHA-256 `fc89acf0251e15b3df4c63cb5d567fa941955b1fa8fe2f04cb5e5954f2ede148`.
- `w-fix/rust-comparison.json`: corrected Rust comparison, SHA-256
  `938d06458d5d7e754121f0526c6bcbc80ee8c354318ef7811d8fbd78a27b7129`.
- `w-fix/validation.json`: code/source hashes, focused test result and combined
  evidence ledger, SHA-256
  `17c8e38adc8cbf900f2274ca0beb652c54bf5c27f3431a295c2ae8e1c92fed61`.

Earlier grouped-W receipts remain unchanged as historical evidence. Broader
native timing, default initialization, skinning and runtime binding research is
separate from this correction.

## Outside source ledger

Read-only format study used **UE Viewer**, Konstantin Nosov, **MIT**, pinned commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981`. New Rust and independent validation
code are original; no reference implementation was copied.

| Primary source | Study use | Private snapshot SHA-256 |
| --- | --- | --- |
| [UnAnim3.cpp](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnAnim3.cpp) | Lines 135–187 native arrays; 229–263 frame arrays; 754–789 offset grouping; 1017–1287 standard track layouts; 818–1014 per-track layout studied, not implemented | `9a8f49f69b0dc7b56958d7e270861fc069dc79ae9bd5be48f1268f511551288b` |
| [UnMesh3.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh3.h) | Lines 360–600 animation enums, fields, viewer defaults; defaults are not retail-native evidence | `c3694376cbc0d0f69b233bf666da00c90f80f4892c3e64e84eb0826056061d47` |
| [UnMeshTypes.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMeshTypes.h) | Lines 114–165 Float96NoW/W reconstruction; 166–264 other encodings studied only | `a6ef0dda3ab7d333ab57a77e3230cb7acf2006d065c78341856043942424ccae` |

License snapshot: private `evidence/upk-reference/LICENSE.txt`, SHA-256
`23831a13a99f726e62388efade7f0b8a778f1b84e40dec8a4bf4a1e481ccbdfe`.
The source snapshots reside under private `evidence/animation-study/reference/`
and `evidence/static-mesh-reference/`. No MHServerEmu or other implementation was
read for this increment. Existing original `upk`, `upk::objects` and
`skeletal_mesh` project APIs supply package/property decoding and saved bone names.
