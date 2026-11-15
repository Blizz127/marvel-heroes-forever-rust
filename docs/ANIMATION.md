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
W from `1 - (X*X + Y*Y + Z*Z)`, clamping a nonpositive remainder to zero before
the square root. The operation order follows the studied reference format. It
does not conjugate, normalize, convert axes, compose parents or apply a bind
transform. Retail-native instruction-level floating-point parity is unverified.

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

**VERIFIED:** the independent comparison matches 9,094 keys (2,092 translation,
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

The final `validation.json` SHA-256 is
`fc34f9912dc601ba9d32222c465211dd5f6600d45856e6af0d9f5727a737788b`.

REA's prior read-only session status is preserved at private
`evidence/skeletal-mesh-study/rea-session-status.json`. That process reported no
open target and unavailable Ghidra Java configuration. No native animation
serializer/codec entry point was identified in this increment; no shared target
or configuration was changed. Package parsing and source study are labeled as
such, not as REA decompilation evidence.

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
