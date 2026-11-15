# Matrix consumption after packed geometry

This study follows the matrix retained by the ordinary output receiver toward
the engine's draw-constant path. The selected 2D consumer computes
`(environment × instance) × inverseNormalization`, in that exact binary32
association, and writes eight words to the selected constant-buffer locations.
It preserves the distinction between that constant matrix, packed vertex words,
native vertex-format conversion and final pixel positions.

The draw record's second pointer is not yet joined to the exact M passed through
the earlier upload collection. Applying this consumer to that M's retained
inverse normalization therefore remains conditional on that pointer association;
matching the `+c0` field offset alone is insufficient.

[SCALE9_OUTPUT_RECEIVER.md](SCALE9_OUTPUT_RECEIVER.md) remains the sealed earlier
receiver, staging and upload contract. The evidence here is separate under
`evidence/scale9-position-reconstruction/`; its predecessor's evidence was not
rewritten to include later findings. No renderer enablement is part of this
increment.

## Evidence and provenance

**OBSERVED selected instructions, VERIFIED byte identity:** all native research
uses the owner's private x64 1.52.0.1700 executable, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No native implementation, asset bytes or third-party SDK source enters git.
Addresses below omit preferred base `0x140000000`.

| Record under `native/` | Evidence ID | Selected dependency |
| --- | --- | --- |
| `rea-cache-owner-constructor.json` | `ev_fa5b6d09c9b713a6fa9fdbf4224e950aaa38dc14a911b1458ec0e020193237b6` | Cache-owner table installation and upload slot |
| `rea-position-reader.json` | `ev_4f205b71cb62910cb53b2740c9022f017b85b5e088512d9eff38263fe8ba388e` | Draw record's second pointer +c0, selected position and other matrix paths |
| `rea-reader-caller-and-composition.json` | `ev_bc5e54b6707a8da4123918aae47241a7a03d6cf8c86320f73059985d89425042` | Draw collection caller, 2D/3D selection, constant-location adapter |
| `rea-two-dimensional-composition.json` | `ev_f0f580e0441cd5ee5491ac5a183f12967f59f77767f3398760e909e92ebf8303` | Complete 032880 arithmetic plus six padding bytes |
| `rea-constant-write.json` | `ev_7116b25c2e4f6c9074d8b9ed7cbaac3be11501dfc44dbfcea73325c9e56db71e` | Exact float-word copies and dirty marking |
| `rea-source-mesh-getter-binding.json` | `ev_217f73624f29aaa4d3ad6afb944adccf88ceedde7af9f841552fbf2c77668492` | Source cache object table and mesh getter slot |
| `rea-primitive-insertion-candidates.json` | `ev_1a2a1a0b91a50b27b5ac577d17fc364e90357c08d2560915f11009f530cc5563` | Two rejected slot-70 candidate bodies |
| `rea-rejected-route-tails.json` | `ev_1b4d6f6488c12f0205ecd0c5643f6bdb15057933fa6719bf5f4654cb75b382c0` | Both candidates' shared epilogues |
| `rea-upload-collection-join.json` | `ev_f2a15ceebc482f2f18321c4ae6fc0ebddc195d5e0f47b9594d851c3cadfaaec7` | Two rejected adjacent slot-48 candidates |
| `rea-rejected-hal-tail.json` | `ev_1873a57f926585d8401f297a0d73961fade21e2c8ba1c84ac3ed9335d5e8bc5b` | 042600's separate shared epilogue |

The earlier matrix-mask record is reused directly:
`scale9-rendering/rea-matrix-data-masks.json`, Evidence
`ev_da3f6575c4f3b34fbffaf185dc9d5265cac2d0d00741d046e98f4786f88c4753`,
SHA256 `49c4fb0eefdbcdb973fa551b073d59d5e2ef8899860b81d7b5602232668842db`.
Its separate raw mapping base is `0x140001800`; query `142adafb0` maps to the
same preferred `.data` address. It must not be translated using the new code
captures' `0x140000c00` raw base.

All ten new commands completed with exit 0, no truncation and unchanged target
hash. Independent PE mapping checked 1,559 decoded rows, 6,347 distinct code
bytes and 272 dumped bytes across the new records plus the reused mask record;
zero invalid rows. The three rejected candidate bodies with separately captured
shared epilogues form complete contiguous unions of their `.pdata` fragments.
The 032880 selection includes its complete 234-byte body and six `int3` bytes.
Final audit: `verification/final-receipt.json`, SHA256
`c56ffbe5349ce89d9a248a68bd26a0e200705c32ed3d2e46ec057091c3c4f0ba`.
All prior receiver receipts, frozen source inputs and the executable remained
unchanged. The ten new REA records were exported to `native/final.bundle.json`
(SHA256 `8dae5e96af2fe21f896414724fb6c5685dd559c39af8f2ea2691977fa8ccc42b`)
and reset with no retained target.

## Exact draw-record path

**OBSERVED:** cache-owner constructor `1203560` installs primary table
`1421ba140`. Its `+48` entry is the previously recovered upload `1255030`.
This installation binds the upload slot; no neighboring table is assumed to
have the same class or field layout.

Draw-side function `1246580` takes a collection object P in entry RDX. For each
eligible batch B, it calls `1257710` with:

| Input | Source |
| --- | --- |
| 16-byte record-array start | `[P+40] + 16 * [B+4c].u32` |
| record count | `[B+48].u32` |
| format pointer | `[B+30]` |
| matrix-environment pointer | `[renderer+28]` |
| constant storage | `renderer+33c0` |

`1257710` retains the record-array input and increments it by 16 for each
iteration. At `1257d00..1257d30` it loads `record.second`, adds `c0`, and passes
that matrix to `1257320` with numeric selector 5. The exact matrix-bearing
record is therefore preserved across the draw loop; the pointer is not selected
by a filename or a reconstructed scene index.

Separately, constructor/factory `0af6e0` installs source-cache table `1425063b0`
whose `+70` is the existing `0b1570` mesh getter. The retained `0b0f10` evidence
stores the ordinary `03bf20` result M at cache-entry `+18`, then that entry at
source S's `+b8`. The getter returns `[[S+b8]+18]`, or null when the entry is
null. These construction/storage/getter links are grounded, but the caller that
inserts their return into P's 16-byte record array has not been identified.

`reuse/cache-selector-link.json` retains ten exact instruction spans and source
hashes, SHA256
`0847e23b65c636e72c1974c338c34a1429f1f285df1bdad0e43f2c9434353753`.
It also checks that the source path's main-matrix selector and the draw consumer
address the same sixteen static table cells. This does not prove identical live
handles or flags at two different calls.

## 2D versus 3D selection

`1257320` dereferences the first word of `record.first` to get D. It computes
the main-matrix offset from table `142adb5c0[(D.byte0f & 15)*5 + 4] * 16`.
The selected instance matrix starts at `D+10+offset`.

If `D.byte0f & 0x10` is clear, the function calls:

`032880(destination, record.second+c0, instanceMatrix, environment+1e0)`.

It then sends exactly eight float words through `1258bd0`, retaining selector 5
and the current record ordinal. This is the admitted 2D route. A set bit instead
calls `033190`, `03f8d0` and `033420` and transfers sixteen words; those 3D
operations are not covered by the 2D arithmetic result.

The role name *environment* denotes the concrete incoming pointer and `+1e0`
field. Its actual producer, viewport conventions and relation to final screen
coordinates are not inferred from its use in this calculation.

## Binary32 operation order

**OBSERVED:** `032880` loads eight-word matrices N (RDX), I (R8) and E (R9).
Rows use the existing native layout `[a,c,unused,tx]`, `[b,d,unused,ty]`.
Its first stage computes W for each row r and lane j:

```text
W[r][j] = round32(round32(E[r][1] * I[1][j])
                 + round32(E[r][0] * I[0][j]))
W[r][j] = round32(W[r][j] + (j == 3 ? E[r][3] : +0))
```

Its second stage computes O:

```text
O[r][j] = round32(round32(W[r][1] * N[1][j])
                 + round32(W[r][0] * N[0][j]))
O[r][j] = round32(O[r][j] + (j == 3 ? W[r][3] : +0))
O[r][2] = positive_zero
```

The masks at `142adafb0` and `142adafc0` are respectively
`[0,0,0,ffffffff]` and `[ffffffff,ffffffff,0,ffffffff]` in u32 lanes.
They preserve translation bits and clear the final unused lanes. This is not
an algebraic license to reassociate the products or use fused multiply-add.
The native intermediate W's unused lanes are computed, even though they do not
contribute to final meaningful lanes.

The existing original `Matrix2x4` API can express the admitted output as:

```rust
inverse_normalization.left_compose(instance.left_compose(environment)?)
```

Its first composition clears W's unused lanes earlier than this native helper;
those lanes do not enter the second stage's meaningful lanes. Both stages still
check all performed products/additions against the existing bounded finite,
non-subnormal arithmetic profile. No duplicate math module was needed.

**VERIFIED conditional arithmetic:** a private standalone harness compiled the
unchanged `native_scale9.rs` from commit `ca3c033` with `rustc` and compared
against an independent exact-rational binary32 oracle derived from the native
instructions. All 88 output words match across eleven admitted cases, including
noncommuting matrices, signed zeros, nonzero unused lanes and three rounding-order
witnesses. A twelfth gradual-underflow input correctly receives the existing
`SubnormalSensitive` bounded rejection. This does not establish the retail
process's MXCSR settings or select any live runtime matrices.

Private receipts are `oracle/oracle.json` (SHA256
`322a676bb9f4c4686ab8506175a89ecfa28fdbaa771181271b780a899055ed3c`) and
`comparison/receipt.json` (SHA256
`67817c3f38eee3b08fa9b59c3325411304fde338836eeaaf18f0e8d50adda2c7`).
The unchanged Rust source SHA256 is
`29e70b57343a5ff08bfdbb53f61d2ff345739270475864bcce019b17ef2193de`.

## Constant-buffer persistence

`1258bd0` resolves the numeric selector through each selected layout's `+2c`
metadata and computes record-relative offsets, then calls `1258b00`.
The latter checks each of the two selected layouts' `+24` ten-byte descriptor
records. Where the descriptor's signed word at `+6` is nonzero, it copies
`wordCount * 4` bytes to constant storage at:

`4 * (descriptor.signedWord(+2) + descriptor.byte(+4) * resolvedIndex)`.

It finally sets `constantStorage+1100+resolvedSelector` to 1. These concrete
copies retain the computed words; shader metadata decides their exact address.
The numeric selector is not replaced with an assumed shader-name binding.

A separate loop in `1257710` copies the same `record.second+c0` matrix,
left-composes a selected additional matrix from `record.first` through `032970`
and transfers eight words with selector 10. That route is relevant to later
fill-coordinate association, but its selected additional matrix's producer
must be bound separately before calling it a particular paint/UV transform.

## Supported use and remaining joins

For a caller that supplies the exact selected N, I and E matrices and explicitly
admits the non-3D, finite non-subnormal profile, the existing Rust expression
above reproduces the observed eight output words. It does not choose those
inputs, bind a live draw record to M, or interpret the packed position words.
Unresolved dependencies are:

- P's record construction and its identity relation to the upload collection.
- The actual selected matrix-environment field `+1e0` producer and state.
- Source/destination vertex format identity, signedness and normalization.
- Shader consumption of the selected constants and conversion to final pixels.
- Additional per-fill matrix producer/slot association; the observed selector-10
  calculation alone does not supply that mapping.

The bounded attempts to close the first join retain their negative results:
`0738e0` and `0778e0` call unrelated `object+10` interfaces at slot 70 and do not
use the return as M; `03fd30` and `042600` are unrelated slot-48 calls, not the
upload ABI. They were rejected instead of treating equal slot offsets as type
identity. No global absence conclusion follows from those searches.

No retail process, native GPU execution or final pixel comparison was run.
