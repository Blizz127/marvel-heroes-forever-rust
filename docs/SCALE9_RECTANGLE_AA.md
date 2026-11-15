# Conditional native antialias rectangle geometry

`src/gfx/native_scale9_antialias.rs` implements the selected native mode-1 AA
route for one exact rectangular contour. It returns **12 generated records and
10 triangles**, preserving four unused source records and eight alternating
interior/exterior records. It does not enable rendering, choose current options,
bind a provider or infer the current state from constructor constants.

The implementation is original Rust derived from the owner's retained REA
evidence. **VERIFIED** below means byte/hash or local Rust checks; **OBSERVED**
means static native instructions; **INFERRED** means conditional execution of
those instructions. No retail process, native geometry call or visual comparison
was run for this increment.

## Explicit supplied profile and limits

`RectangleAaProfile` has three required values and no default:

| Field | Actual native role | Admission |
| --- | --- | --- |
| `relative_tolerance` | primary `T+0x1c` | finite, nonnegative; native Y coalescing must leave both exact source rows unchanged |
| `half_width` | primary `T+0x3ac` | finite and strictly positive, so the native `T+0x3b0` flag is set |
| `corner_tolerance` | primary `T+0x3b4` | separate finite nonnegative corner determinant parameter; the descriptive Rust name is not a recovered SDK name |

The other conditions are the fresh/reset mode-1 source contract in
[SCALE9_TESSELLATION.md](SCALE9_TESSELLATION.md), with positive relative tolerance
handled as described in [SCALE9_TOLERANCE.md](SCALE9_TOLERANCE.md): one five-point
rectangle in low-left, low-right, high-right, high-left, low-left order; bitwise
matching edges; styles `[0,1]`; resource-present style 1; valid coordinates above
the native sentinel; one monotone; and explicitly disabled generator postprocess.

[SCALE9_OPTIONS.md](SCALE9_OPTIONS.md) separately establishes that constructor
word `0x3f733333` would become half-width `0x3ef33333` only when the actual
incoming flags satisfy `(flags & 3) == 1`. It also records constructor words
`0x3727c5ac` and `0x3a83126f` for the two separate tolerance fields. The saved
diagnostic supplies those three values; it does not establish their current
runtime values or select them in this API.

The arithmetic comparison assumes IEEE binary32 nearest rounding, separate
scalar operations, ordinary correctly rounded finite `sqrtf`, and successful
valid nonaliasing native storage. Nonfinite inputs/results, nonzero subnormals
and operations with subnormal-sensitive exact magnitudes are rejected. The PE
import binds the helper's thunk to `MSVCR120.dll!sqrtf`; its loaded implementation
and native floating-point control state remain unobserved. Rust error names
describe this bounded implementation, not native error codes.

## Selected AA dispatch and incidence order

**OBSERVED, bytes VERIFIED:** `fed90(T,1)` sets mode 1 and clears `T+0x21`.
With positive width, `1115b0` selects `10e4c0`, then `10bb30`. The distinct mode-2
route through `10dfa0` is excluded. Addresses omit preferred base `0x140000000`.

The common scan preserves the generated order `0=LL, 1=LR, 2=HL, 3=HR` and
linked words `[0x80000000,1,0x80000002,3]`. The module reuses that proved source
scan and original triangle order; it discards the non-AA metadata. Original
coordinates and linked word 0 remain unchanged during AA corner construction.

`10ad90` counts two incidences per original vertex. `10af00 → 100260` writes
24-byte incidence records, with prefix offsets `[0,2,4,6]`. Each record holds
the center node pointer, neighbor node pointer, a style word and a rank word.
`10e4c0` computes angle keys; `102380` sorts each two-record group using `fedb0`.

For admitted finite axis vectors the keys are `+X=0`, `+Y=0.5`, `-X=-1`,
`-Y=-0.5`. The implementation still performs the native squared-length arithmetic
and rejects its unsupported numerical cases. Equal-angle style tie branches are
not reached. Only original vertex 3 swaps its initial two incidences.

| Original vertex | Sorted neighbor/style pairs | High-style neighbor, low-style neighbor |
| --- | --- | --- |
| 0 | `(1,0x8001), (2,1)` | 1,2 |
| 1 | `(0,1), (3,0x8001)` | 3,0 |
| 2 | `(0,0x8001), (3,1)` | 0,3 |
| 3 | `(2,0x8001), (1,1)` | 2,1 |

`10b050` finds one style boundary per group. Its first `1003d0` call receives
`(high neighbor, center, low neighbor)` and appends the interior vertex. It
replaces node `+4` with that index. The second call reverses the neighbors and
appends the exterior vertex when the selected ordinary intersection returns
false without writing its optional secondary output. New indices are therefore
`4+2*v` and `5+2*v`, with node mappings `[4,6,8,10]`.

Two distinct neighbors exclude the paired-join append to `T+0x518`. The temporary
two-entry corner group is below the native three-entry requirement and rolls
back, leaving `T+0x540` empty. Only the two directed-edge entries per corner
survive in `T+0x4a0`; their low words are exterior index and exterior index OR
`0x40000000`. Unused native bytes are not asserted.

## Exact corner arithmetic and rejection gates

`1003d0` computes two segment lengths with separate scalar products/additions
and `sqrtf`, then calls `f6060` for its signed turn value. The normal offsets
are multiplied by width before division by the corresponding segment length.
It builds displaced line endpoints and intersects those lines using the exact
captured subtraction, multiplication, subtraction, division and final addition
order. The Rust module preserves this order; it does not substitute center
coordinates plus or minus the width.

For example, the supplied 1280-square's first interior point has bits
`[0x3ef33000,0x3ef33333]`, while the supplied width has bits `0x3ef33333`.
Cancellation during the native sequence explains the unequal coordinates.
Two independent scalar oracles agree on all eight generated XY bit pairs.

Only the ordinary intersection path is admitted. In native order the checks are:

1. Both segment lengths are positive, with all selected intermediates supported.
2. `abs(turn) >= 0.125` skips the small-turn fallback.
3. The line determinant is nonzero, and
   `(length1 + length0) * corner_tolerance <= abs(determinant)` skips the
   near-parallel fallback. Equality is admitted; the native comparison is strict.
4. Distance from the center to the intersection does not exceed the native
   limit: `width * 4` for negative turn, otherwise
   `min(length0,length1) / abs(turn)`. This excludes clamp and split alternatives.

A native false return by itself would be insufficient because fallback and
clamp branches also return false. These specific branch predicates, including
the untouched secondary output, close the selected path.

For each original triangle `(a,b,c)` and mapped interiors `(ia,ib,ic)`, the
module computes the three `10c8e0` correction predicates in separate binary32
operations:

```text
g0 = (ia.x-c.x)*(c.y-b.y) - (ia.y-c.y)*(c.x-b.x)
g1 = (a.y-c.y)*(ib.x-a.x) - (ib.y-a.y)*(a.x-c.x)
g2 = (ic.x-b.x)*(b.y-a.y) - (ic.y-b.y)*(b.x-a.x)
```

All must be strictly negative to skip every `1007f0` correction. Subsequently,
`1009d0` examines each mapped interior triangle with
`q=(c.x-b.x)*(b.y-a.y)-(b.x-a.x)*(c.y-b.y)`. Strictly negative `q` skips all
repair writes. Zero or positive results are rejected. The supplied square has
six negative first-stage predicates and final `q` bits `0xc9c7b408` for both
triangles. Narrow rectangles can fail these gates even when their corners were
ordinary intersections; they are not silently replaced by another triangulation.

## Final generated records and exact fringe order

`10bb30` first binds the two interior triangles through `10a110`. It then sorts
the eight directed edges by unsigned center/neighbor node pointers. The four
nodes occupy successive 16-byte slots in one fresh page, so their logical
indices preserve unsigned pointer order under valid nonwrapping storage.
The untagged processed keys are `01,13,20,32`. For each `A→B` it appends
`[innerA,innerB,outerA]`, then `[outerB,outerA,innerB]`. Empty `T+0x518` and
`T+0x540` exclude the other join and fan phases.

The exact global triangle order is:

```text
[4,6,8], [10,8,6],
[4,6,5], [7,5,6], [6,10,7], [11,7,10],
[8,4,9], [5,9,4], [10,8,11], [9,11,8]
```

| Generated record | Final ordinal | Styles | Flags | Mesh |
| --- | --- | --- | --- | --- |
| original 0..3 | `0xffffffff` | `0xffff,0xffff` | `2` | `0xffff` |
| interior 4,6,8,10 | 0,2,4,6 | 1,1 | `0x0a` | 1 |
| exterior 5,7,9,11 | 1,3,5,7 | 1,1 | `0x08` | 1 |

The final generator scan skips original records with mesh `0xffff`, assigns
ordinals in generated order and counts eight vertices for descriptor 1.
Descriptor 0 is seven zero words; descriptor 1 is
`[1,1,1,0x8008,0x8000,0,8]`. The four source records remain in the global index
domain. The existing separate index exporter yields 30 packed indices from
these ordinals. The implementation models logical fields, not native padding,
unused qword halves, allocation capacities or failure behavior.

## Focused verification and evidence coverage

**VERIFIED local checks:** the frozen module compiled and all four new AA tests
passed. They cover exact cancellation-sensitive position bits, full record and
fringe order, source immutability, correction/fallback rejection, invalid width,
the separate corner option, Y-row merge rejection, overflow and subnormal limits.
The isolated wrapper produced expected unused-code warnings from dependencies.
No combined product build or routine full suite was run in this lane.

An optimized standalone invocation of the same frozen source was compared with
the independently retained instruction-derived Shape 72 oracle. All 12 full
vertex records, 10 triangle triples, two descriptors and 30 packed indices
matched in 25 comparison groups. Identity transform, resource-present style 1
and all three options were explicit supplied inputs. This proves local agreement
with that conditional oracle, not a native invocation or runtime AA parity.

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/scale9-rectangle-aa/`.

| Artifact | SHA-256 |
| --- | --- |
| frozen `native_scale9_antialias.rs` | `a63ef6b19b27811611491609bd85d1d349c34c472bb5d67c70d1938740d92660` |
| `focused-tests.json` | `9d3267c0b546da3bdda9abddffbbbac4c569905d1cd8074c6d5e9529e696b7bd` |
| `saved-oracle-check.json` | `21a5ee5a2f373630f394bfe2b62541f57884af68d5cfb2dfaf113b47edc08c78` |
| `aa-incidence/receipt.json` | `a63e6517fd80b7d26546063495fcad4578e098b727829a33d0b3938227a3913c` |
| `corner-math/receipt.json` | `b01aefdec9a7fe89ad7002a8d3a7359c9439248a9e91869c75b4d8904ae797d0` |
| `final-emission/verification.json` | `86563fe0e863f7148dd586e07525eebe4489b9142ea6848f7e2ecdf890c1a43e` |
| `final-emission/oracle.json` | `172e3765116831b0cabe5e6f4b99e8ed7db382c8db29dbedcf2643334b098e16` |
| `native/coverage-correction.json` | `413b5b6a725ab4bc3ddb2b55d1f9e7965385391586db28f644311c81e0a9be35` |

The parent native byte audit checks 2,856 instruction records / 12,002 bytes
plus 36 constant bytes. Incidence, corner and final-emission audits independently
check their selected dependencies; their totals overlap and are not added.
The corner audit includes complete `1003d0`, `f6060` and the explicit sqrt thunk.
The emission audit checks seven complete functions, with a separate complete
`1009d0` audit. All new REA records were exported and their sessions reset.

**Coverage correction retained:** the initial parent receipt called
`102380..1028db` a function. It is a 1,371-byte prefix; a later independent
coverage audit found its 28-byte stack-cookie epilogue. The additive correction
joins that prefix with Evidence
`ev_4e5540def464af0e5bb861f0069ec8b9baabd683a23400dfead88afe7b8ee461`,
closing the full 1,399-byte / 338-instruction range through `1028f7`.
Historical receipts and initial verifier failures remain unchanged for audit.
Ordinary successful stack-cookie validation remains a storage assumption.

## Provenance

All Rust and checking code is original. Proprietary bytes, instruction listings,
saved data, REA bundles and test executables remain outside git.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's private `MarvelHeroesOmega.exe`, build 1.52.0.1700, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner installation; interoperability study | Exact instruction/constant/import evidence and conditional AA behavior |
| Existing project Scale9 geometry/tessellation/tolerance/options documents and modules; private saved Shape 72 oracle | Original project investigation; underlying assets proprietary | Common source/scan/index contract, supplied diagnostic inputs and separate option roles |
| Retained REA evidence under `scale9-geometry`, `scale9-tessellation`, `scale9-rendering`, plus this lane's records identified by the private audit manifests | Private owner-artifact evidence | Complete selected native caller/callee dependencies; no third-party renderer implementation copied |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/evidence-workflows.md`, `references/native-and-artifacts.md`; REA 6.9.0 / Rizin 0.9.1 | MIT / LGPL-3.0 tooling as previously registered | Bounded target-free reads, retained-evidence reuse, exact-byte validation, export/reset |
| Installed Rust compiler and Python standard library | MIT or Apache-2.0 / PSF tooling | Original focused compile, scalar oracles, exact PE/hash checks and frozen comparison |

Principal new parent Evidence IDs are
`ev_96646fe297889494f19320d49b7c7e61457f041010e7fc45ea89bea156ea08fb`,
`ev_52fc4a414c30b98db33148200eeb63394a07231dcbc43b046b560db0275322c1`,
`ev_618fb49d1dac107fcfd7d803a00b65c390717d841ec0da3850e3dc7c9afa6221`,
and `ev_6087da4afce47df6a47aa523aa5608ca9c515b969a8efe2ebb80c7d39cef05c6`.
The corner helper records are
`ev_99f29430e57647a3331d575c9d453f418aff1cb461605b4e21592baddc2d28ce`
and `ev_4bd89ec5f69775bbc3ac3e3c1240b232dc0e42c847eb47ee8cd22bf998305f02`;
the edge-sort record is
`ev_93eecc3c37087462d629540dc10631f6be5a38d6ceb0f3231f01f4f08762c99a`.
Incidence helper records and all reused record hashes are individually listed
in the private manifests. No service, access, launcher or live-state action
occurred. General contours, alternate AA mode, corner splits, repair branches,
resource choice, UVs, GPU formats and native allocation failures remain outside
this implementation.
