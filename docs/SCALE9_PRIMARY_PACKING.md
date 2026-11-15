# Conditional native primary record writer

`gfx::native_primary_packing::write_primary_records` implements the recovered
single-style store stage of `bff70`. It consumes already selected, normalized
`GeneratedVertex` values and explicit resolved native color words, then writes
the ten defined bytes of each 12-byte record. It preserves both padding bytes
and the destination tail. It does not enable a renderer or establish that an
actual movie selected the asserted format or native profile.

## Explicit inputs and admission

`PackingProfile` supplies the actual selected format fields, the forced-style
argument, and the morph-owner/factor state. The accepted format is exactly:

| Byte offset | Element code |
| --- | --- |
| 0 | `0x132` |
| 4 | `0x251` |
| 8 | `0x1211` |
| 9 | `0x2211` |
| 0 | `0`, terminator |

The stride must be 12. `EXACT_PRIMARY_FORMAT` describes those values; passing
that constant is an explicit caller assertion, not evidence that the current
native object has them. This matters because the native cache comparator can
accept different descriptor pointers and does not compare stride. The remaining
source-key dispatch gap and all format/carry conditions are documented in
[SCALE9_PRIMARY_SOURCE_FORMAT.md](SCALE9_PRIMARY_SOURCE_FORMAT.md).

The forced-style argument must be zero. The factor must be finite, and either
the morph owner is absent or the factor is numeric zero, including negative
zero. No morph interpolation, forced color, or native callback is executed.
An absent owner makes the factor unused on the recovered branch; a finite
nonzero factor is therefore admitted with an absent owner.

For each vertex:

- Bits `0x8000` and `0x10` must be clear; skipped-color and dual-style paths
  are unsupported.
- The selected style is `styles[(flags >> 5) & 1]`. It must be nonzero.
- `ResolvedColorWord.style_id` must match that style. Its `word` is the exact
  native dword **after** callback/cache resolution. No channel swizzle, white
  default, premultiplication, color transform, or interpolation is inferred.
- Adjacent records selecting the same style must supply the same word. This
  conservative consistency guard follows the retained last-style reuse within
  the supported branch; the module does not implement the native cache. Initial
  cache state and continuity across calls remain explicit caller obligations.

Style zero is deliberately excluded because the native last-style state begins
at zero and can bypass a fresh style callback. The helper must not invent its
initial result storage. Other vertex flag bits are retained inputs; only the
proved selection and factor-byte masks are consumed by this store stage.

The module accepts at most 256 vertices, matching the native per-transfer
request, and exactly one resolved color per vertex. The destination must hold
at least `12 * count` bytes. Mesh IDs, local ordinals, descriptor cursors,
triangle indices, source positions and colors are never changed.

## Exact byte stores

The caller supplies positions after native export selection and normalization.
The existing finite-nearest-rounding coordinate helper packs each coordinate
using the recovered separate arithmetic, `floor`, signed32 conversion and low
word storage. Its CRT/MXCSR and finite/subnormal limitations remain unchanged.
In particular, negative integral coordinates are not silently rounded toward
the nearest integer: `-32764` stores the low word of `-32765`.

| Bytes | Value |
| --- | --- |
| 0–1 | Packed X word, little endian |
| 2–3 | Packed Y word, little endian |
| 4–7 | Resolved native color dword, little endian |
| 8 | `[0,127,255,255][flags & 3]` |
| 9 | `[0,127,255,255][(flags >> 2) & 3]` |
| 10–11 | Existing destination bytes, unchanged |

The implementation validates and stages the entire chunk in a fixed stack
array before any destination write. Unsupported profiles, lengths, styles,
inconsistent colors, and arithmetic errors therefore leave every destination
byte unchanged. This atomic rejection is bounded Rust engineering policy; the
native routine writes progressively and is not claimed to provide the same
failure behavior. Safe Rust's disjoint borrowed inputs/output are the aliasing
profile. No heap allocation or mutable renderer/global state is involved.

## Evidence and verification

The original implementation is derived from the private owned executable,
SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and earlier original project arithmetic. No external implementation source was
copied. Canonical native evidence is the already sealed
`evidence/scale9-output/rea-output-stage.json`,
`ev_b4b937795178c5900fcf0c51650c0472acbee93992acdc3be5b1cac00444b1d7`:

- `c054c..c0559`: at most 256 exported vertices per request.
- `c0590..c05d9`: X/Y packing and word stores.
- `c05ce..c0601`, `c0728..c07ca`: forced/skipped/dual gates, style-bit selection,
  cached style/color path, callback receiver and morph bypass.
- `c07cc..c07f9`: exact flag-table indices and byte stores.

The table bytes are independently retained in
`scale9-primary-source-format/native/rea-fill-object-callers-and-pack-table.json`,
`ev_636cfd31cadc290e53e3d07f1528b53cfb595f4d83766ed51ec5298a8a868bc7`.
The complete format source/cached-comparison study remains separate.

Private checks are retained under `evidence/scale9-primary-packing/`. Five
focused module tests pass: hand-written exact bytes and style selection,
all 16 flag-factor combinations and negative zero, late-error atomicity,
exact layout/profile guards, and count/buffer boundaries through 256 records.
The standalone test harness compiles the original dependencies with `rustc`
and runs only this module's five tests. Its initial fixture omitted the `gfx`
namespace expected by existing dependency tests; that failed harness/log is
retained. The corrected fixture supplies the namespace. No product change was
needed for that harness error.

Nine independent private fixtures also pass against four frozen source files:
three successful byte-output cases and six atomic-rejection cases. Their oracle
was derived from the retained native contract without reading or running this
Rust helper. It uses exact rational binary32 rounding for the half-boundary
coordinates and independently checked literal record prefixes. The comparison
checks all 351 destination bytes and 500 serialized source-field bytes, plus
unchanged color inputs. It includes nonzero unused factor with absent morph
owner, negative-zero factor with present owner, bit5 style selection, exact
format rejection, and a failing second vertex after a valid first one.

Oracle SHA256:
`be065bca09c9ecb7c63a20517c6c2f338e85234987cad6d83789be5a52f6570d`.
`independent-comparison.json` retains all nine results and verifies that the
frozen and working source hashes remain equal. These synthetic witnesses test
the explicit bounded contract; they do not establish runtime format selection,
native callback outcomes, or wire/render parity.

This increment does not run the client, a GPU draw, live style callbacks,
automatic source-format selection, or retail parity checks. Root owns the
module hook and later composition with the descriptor-cursor exporter.
