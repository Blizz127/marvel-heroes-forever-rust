# Conditional constant-key animation timing

`src/animation/timing.rs` selects channel key indices and an alpha using the
observed constant-key codec contract. All inputs and cache knowledge are
explicit. It does not advance a clock, interpolate vectors/quaternions, choose
inherited defaults, apply transforms or establish live Loki playback.

## API and supported domain

`sample_constant(Inputs, &mut CacheState)` returns a `Sample` containing the two
indices, alpha, normalized time and whether the cache was bypassed, hit or
missed. `Inputs` supplies key count `K`, frame count `N`, duration, time and the
full signed 32-bit codec flag. The branch distinguishes zero/nonzero; cache
matching distinguishes every flag value. No boolean is inferred from Matinee
or AnimNode properties.

Counts must be positive and no greater than `2^24`, so integer-to-binary32
conversions remain exact and the supported nonnegative index operations stay
representable. This is an API arithmetic bound, not an observed retail limit.
Time must be finite, duration finite and positive, and their binary32 quotient
finite. Invalid inputs return errors without mutating the cache. Existing asset
decoder limits may be stricter.

Multiplication, division and subtraction remain separate `f32` operations in
native instruction order. There is no algebraic reassociation or fused
multiply-add. The reviewed arithmetic assumes ordinary binary32 behavior;
retail MXCSR, rounding/denormal state and live caller scheduling were not
captured. Unsupported NaNs, infinities and invalid native pointer/count behavior
are not simulated.

## Explicit shared cache

**OBSERVED with REA:** both selected native channel readers share these fields:

| Native address | Field |
| --- | --- |
| `0x142c24ea0` | Key count, signed 32-bit |
| `0x142c24ea4` | Normalized time, binary32 |
| `0x142c24ea8` | Duration, binary32 |
| `0x142c24eac` | Full integer codec flag |
| `0x142c24eb0` / `0x142c24eb4` | First / next indices |
| `0x142c24eb8` | Alpha, binary32 |

A hit compares key count, full flag, duration and normalized time. It does
**not** compare NumFrames, sequence identity, channel identity or data pointer.
Float equality is numeric, so positive and negative zero match. The API
preserves the snapshot on a hit, including its stored signed-zero bits. Native
unordered float comparisons are outside the supported domain: nonfinite known
cache keys return `Unsupported` for a multi-key call.

The reader has no observed validity bit. Its initial runtime contents remain
unknown. The Rust API therefore has no implicit default and requires one of:

- `Unknown`: a multi-key call returns `Unsupported` without choosing a hit/miss.
- `KnownMiss`: the caller explicitly asserts that the next multi-key call must
  miss. This is a conditional fresh calculation, **not** a native initialization
  claim. The completed calculation replaces it with a `Known` record.
- `Known(record)`: the seven supplied fields participate in the native key
  comparison. A matching result must have in-range indices and finite alpha in
  `[0,1)` before it is returned. Invalid unused result fields are replaced on a
  miss; they are not read as a selection.

For `K=1`, both native readers bypass cache comparisons and writes. The API
returns key zero and reports `Bypassed`; its conceptual `(0,0,+0)` selection
does not assert that native code computed or stored alpha. Cache state remains
unchanged, including `Unknown` or an otherwise unsupported snapshot.

On a completed native miss, all seven fields are written. Native writes first
publish the key, then compute/update result fields, sometimes through
intermediate frame indices. This primitive exposes only a completed call via
an exclusive mutable reference. Errors are transactional; interleaved native
writes, concurrent races and cache initialization are not emulated. Callers
seeking the observed cross-channel cache behavior must share this cache rather
than silently create one per track or add `N` to its key.

## Fresh computation

The wrapper computes `r = f32(time / duration)` before calling a channel reader.
For `K>=2`, on an established miss:

| Condition | Selection |
| --- | --- |
| `r <= 0` | First and next are 0, alpha 0 |
| Flag zero, `r >= 1` | First and next are `K-1`, alpha 0 |
| Flag nonzero, `r >= 1` | First and next are 0, alpha 0 |
| Flag zero, interior | Position `f32(f32(K-1) * r)`; floor, subtract floor for alpha, clamp selected indices to `K-1` |
| Flag nonzero, interior | Start with position `u = f32(f32(N) * r)` and its floor/alpha |

In the nonzero interior branch, if the provisional next frame reaches `N`,
select `(K-1, 0)` and retain the frame alpha. This check comes before comparing
`K` and `N`, so `N=1` takes this branch without dividing by zero. Otherwise,
equal counts retain the provisional pair. Unequal counts recompute position
as `f32(f32(u / f32(N-1)) * f32(K-1))`, with division before multiplication,
then apply the same floor/alpha and bounded-index selection as the zero branch.

Native floor uses truncating conversion plus a negative-fraction correction.
Supported interior positions are finite, nonnegative and representable, so
truncation has the same result. Alpha is computed before clamping indices.
Beyond-end time is an endpoint selection; this helper performs no modulo wrap.

These cases apply to a miss. A hit returns the retained pair/alpha even if a
different frame count would produce another fresh result. The difference is
preserved intentionally and tested, without claiming that live callers actually
violate a frame-count invariant.

## Evidence and provenance

**OBSERVED primary source:** the owner's proprietary build 1.52.0.1700
`MarvelHeroesOmega.exe`, pinned under
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/`.
Full SHA-256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No executable bytes or raw native instruction listings enter git.

This increment studies retained REA 6.9.0 target-free Rizin output, without new
provider calls or shared-configuration changes. Relevant addresses are:

| Native function/range | Use |
| --- | --- |
| `0x1407e26e0`, division at `0x1407e275d` | Time/duration quotient and unchanged channel arguments |
| `0x1407e8450` | Constant None translation reader: cache and index branches |
| `0x1407e27e0` through `0x1407e2b48` | Constant Float96 rotation reader: matching cache and index branches |
| `0x1407e2904` through `0x1407e2972` | Rotation cache comparisons, hit loads and miss key writes |
| `0x1407e2a3c` through `0x1407e2b40` | Frame-count branch, final-to-first pair and unequal-count remapping |

The retained records are under private `evidence/animation-runtime/native/`:
`rea-init-and-constant-dispatch.json`, `rea-constant-float-keys.json` and
`rea-quaternion-tail-and-default-copy.json`. Their source-bound independent
contract is `constant-policy-independent/constant-policy-receipt.json`, SHA-256
`283c7b4cf88e6f1ca7404dfddebe8b38ebba9e451b9e88e60e96dda9b8badaa0`.
The exact threshold 1.0 is verified in the existing file-backed constants
receipt; no unknown vector mask, quaternion coefficient cache or wrapper
initialization value is needed for this index-only primitive.

The owned Asgard package supplies idle dimensions for private checks:
`MarvelVista_AsgardHub.upk`, SHA-256
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.
Saved scene/default and binding limits remain in [ANIMATION_RUNTIME.md](ANIMATION_RUNTIME.md).
No outside parser, animation library or implementation source was read for this
increment; earlier UE Viewer MIT study remains separately recorded in
[ANIMATION.md](ANIMATION.md). All code and validation harnesses are original.

## Focused validation

The standalone module's five tests cover endpoint/reset behavior, the
single-key bypass, final-to-first selection, unequal counts including `N=1`,
frame-count cache aliasing, full integer flags, signed zero, duration-key
changes and malformed inputs/cache results. Only this module is compiled for
its focused test executable; no full project suite or retail executable runs.

Independent native-derived cases and actual idle dimensions are checked in
private `evidence/animation-timing/`. All **33** cases passed: 28 successful
selections match index, alpha and normalized-time bits; five excluded/unknown
cases return errors. All resulting cache states and seven-field snapshots match,
including untouched error/bypass/hit states. The oracle was authored from the
retained native instructions before reading this implementation.

Idle export 4 saves 120 frames and duration 4 seconds; both channel types have
key counts 1 or 120. Its key encoding and translation compression are omitted,
so these dimension checks remain conditional on the explicit constant profile.
They do not establish the missing defaults. The comparison includes endpoints,
the last representable normalized time below one, the last-to-first interval,
unequal dimensions, `N=1`, frame-count cache aliasing, signed-zero retention and
the exact-count bound.

These receipts distinguish conditional arithmetic/cache checks from live
animation or whole-client parity. Root module exposure is left to the integrator.

Final private receipt: `evidence/animation-timing/validation.json`, SHA-256
`5d6b054fdcb0ce07094ca208b75ea23c11bca4f416a03d5caac972a32b7d2775`.
It binds the original implementation, focused test output, optimized comparison
driver, source ledger and independent oracle/cache review. The separate
`independent/rust-comparison.json` records 396 exact field comparisons; its
SHA-256 is `c4a2fb1341da38d2b5ea6d6e80c83bcaf6c4dc19183ab253d9b6eff66926d4d7`.
