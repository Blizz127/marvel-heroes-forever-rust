# Explicit camera math from the pinned native client

**VERIFIED static contract:** the pinned client provides a quantized integer
rotator, a row-vector view transform and a perspective projection. The formulas
below can support original Rust math with explicitly supplied camera values.
They do not choose the active Vista camera, its effective FOV, animation time or
output surface. Those remain the separate boundaries in
[FRONTEND_VISTA.md](FRONTEND_VISTA.md).

This document describes preferred Win64 image addresses in the owner's pinned
`MarvelHeroesOmega.exe` (SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`).
No client/native code was executed. **OBSERVED** means selected artifact bytes;
**VERIFIED** means REA instruction evidence or a stated reproducible check;
**INFERRED** means interpretation beyond those observations. Static numeric
replay is not a retail frame, runtime dispatch proof or native floating-point
capture.

## Caller-owned values and the saved route

Supply position as three finite binary32 values; rotation as three signed
32-bit words; FOV in degrees; aspect policy; a positive viewport rectangle;
and near/far distances. Do not obtain missing values from similarly named UI
textures. Normal application validation of finite positive viewport dimensions,
positive near distance and an FOV strictly between zero and 180 degrees is a
proposed input boundary, not a recovered native validation policy.

**INFERRED bounded saved script route:** `Actor.GetActorEyesViewPoint` copies
Location/Rotation; `CameraActor.GetCameraView` calls that virtual method and
copies FOVAngle into TPOV. The CameraActor branch of `Camera.UpdateViewTarget`
also propagates aspect and constraint state. `DoUpdateCamera` updates view
targets, optionally blends, then calls `FillCameraCache`. `GetCameraViewPoint`
reads cached pose; `GetFOVAngle` reads LockedFOV when locked, otherwise cached
POV.FOV. This establishes a route for supplied values, not an active receiver,
override, camera modifier or blend result.

The selected nine Engine bodies cover 3,052 serialized bytes and 4,700 inferred
loaded bytes, with 43 checked branch destinations and no unresolved Context
skip warnings. Declaration/serialization offsets are not native member offsets.
The source default TPOV FOV90 and default-object camera values are retained as
saved records only. In particular, MarvelPlayerCamera declares Engine.Camera
as its superclass: the separately present GamePlayerCamera FOV-adjust helper
is not proof that Marvel uses it.

## Integer rotation and its table

**VERIFIED:** `UObjectexecMakeRotationMatrix` at `0x14072b8a0`, the translation
variant at `0x14072b950`, and `GetAxes` at `0x140737750` all call the same
414-byte constructor at `0x14073a690`. GetAxes extracts the first three values
of matrix rows zero, one and two. The constructor reads words at offsets
0, 4 and 8; call them `p`, `y`, `r` below. Their physical effects are elevation,
azimuth and roll as shown by the explicit matrices. The saved Core field chain
and tagged defaults are Pitch → Yaw → Roll. Mapping those names to native word
offsets is **INFERRED** from that order and the numeric behavior; the three
40-byte saved field records do not independently encode native offsets.

For a word `a`, use wrapping unsigned arithmetic:

```text
index(a) = (u32(a) >> 2) & 0x3fff
S(a)     = T[index(a)]
C(a)     = T[index(a + 0x4000)]
```

The table has 16,384 binary32 entries. A full turn is 65,536 integer units;
the low two bits are discarded, including for negative words. Adding a full
turn changes no index. Do not replace this with an unquantized radians sine.
The exact native formula also does not force cardinal-angle outputs to zero.

**OBSERVED initialization:** the leaf at `0x1403d6d10` supplies table address
`0x142afd480` to `0x140759b30`. That 182-byte body constructs two angles per
iteration and calls `0x141c6f090`, which jumps to `0x141c6fa50`. The table's
stored image bytes are zero; this is an initializer reconstruction, not a live
table dump or proof of startup execution.

Under IEEE754 round-to-nearest, ties-to-even, with a separate rounding after
each operation and no fused multiply/add, the angle for entry `n` is:

```text
a = f32((f64(f32(2*n)) * PI64) * (1.0 / 16384.0))
PI64 bits = 0x400921fb54442d18
```

The full selected sine routine is 2,240 bytes. All 16,384 table arguments fall
below its absolute-input threshold 10,000, so only its first fast branch is
needed for table reconstruction. That branch is lane-independent. Let every
arithmetic operator below round to binary32; `nearest_int` uses the stated
rounding mode, and `F(bits)` decodes binary32:

```text
k = nearest_int(a * F(0x3ea2f983))
b = ((a - k*F(0x40490000)) - k*F(0x3a7da000)) - k*F(0x34222000)
t = b - k*F(0x2cb4611a)
z = b*b
q = F(0x362f0519)*z
q = (q + F(0xb94fbaf1))*z
q = (q + F(0x3c088773))*z
q = (q + F(0xbe2aaaa5))*z
T[n] = sign_xor(q*t + t, (k & 1))
```

The last operation flips the sign bit; it is not multiplication by a sign.
Live MXCSR/rounding state was not captured. The private static reconstruction
has SHA256 `4add0a9aa4af1a52c67058e38d74f26f1c6384f55c04c3e69df59c6c1c8b9cc2`.
Its quarter-turn entries are `+0`, `1`, `-8.742277657347586e-8`, `-1` at
indices 0, 4096, 8192 and 12288. A library sine substitution is therefore a
different implementation requiring a separately measured error policy.

## Forward rotation and inverse view rotation

Set `sp=S(p)`, `cp=C(p)` and similarly for `y,r`. The constructor writes these
four rows in memory, with binary32 rounding after each multiply/add/subtract:

```text
R[0] = [cy*cp,                  cp*sy,                  sp,       +0]
R[1] = [(sp*sr)*cy - cr*sy,      (sp*sr)*sy + cy*cr,     -(cp*sr), +0]
R[2] = [-((cr*sp)*cy + sy*sr),   cy*sr - (cr*sp)*sy,     cp*cr,    +0]
R[3] = [origin.x,               origin.y,               origin.z,  1]
```

Negation is sign-bit XOR. Parentheses preserve the observed product grouping.
For row vectors, the real-number factorization is roll, then pitch, then yaw:

```text
Roll  = [[1,0,0], [0,cr,-sr], [0,sr,cr]]
Pitch = [[cp,0,sp], [0,1,0], [-sp,0,cp]]
Yaw   = [[cy,sy,0], [-sy,cy,0], [0,0,1]]
R3x3  = Roll * Pitch * Yaw
```

This factorization explains the axes; it is not a bit-identical replacement
for the constructor's grouped scalar formula. At zero rotation, local +X is
forward, +Y is right and +Z is up in the recovered camera basis. Positive yaw
moves forward toward +Y, positive elevation toward +Z, and positive roll moves
the local right axis toward −Z. These explicit directions avoid importing an
unverified graphics-library handedness convention.

**VERIFIED inverse path:** `0x140834c50` constructs inverse elementary matrices
and evaluates `Yinv * Pinv`, then that result times `Rinv`. Its nontrivial rows
are the transposes of the three elementary matrices above, with fourth row
and column `[0,0,0,1]`. Each matrix-product output lane uses this exact order:

```text
(((A[i,1]*B[1,j] + A[i,0]*B[0,j]) + A[i,2]*B[2,j]) + A[i,3]*B[3,j])
```

Each multiplication/addition rounds to binary32. Do not substitute a transpose
of the already-rounded forward matrix and claim identical words.

## View transform

**VERIFIED:** named LocalPlayer Project and DeProject wrappers call
`0x140cd1e70` and `0x140cc48f0`; both use the shared view constructor
`0x140cc1050`. The selected constructor builds negative camera translation,
multiplies by the inverse rotation above, then multiplies by the fixed basis:

```text
V = T(-camera_position) * InverseRotation * B
B = [[0,0,1,0], [1,0,0,0], [0,1,0,0], [0,0,0,1]]
view_point = [world.x,world.y,world.z,1] * V
```

The basis sends `(forward,right,up)` to `(right,up,forward)`. For the two view
products, rows zero through two accumulate `((p0+p1)+p2)+p3`; row three's final
add is `p3+((p0+p1)+p2)`, with `pk=A[i,k]*B[k,j]` and separate binary32 rounding.
The basis permutation is a useful algebraic description; retaining the native
multiply/add sequence matters for an eventual word-level comparison.

## Perspective projection and aspect selection

**VERIFIED:** helper `0x140cbf430` takes half-angle inputs `hx,hy`, multipliers
`xm,ym` and explicit near/far distances `n,f`. For finite inputs it writes:

```text
Q = (n == f) ? F(0x3f7fbe77) : f32(f / f32(f-n))
P = [[f32(xm/tanf(hx)), 0,                  0,                  0],
     [0,                 f32(ym/tanf(hy)), 0,                  0],
     [0,                 0,                  Q,                  1],
     [0,                 0,                  sign_xor(f32(Q*n)), 0]]
```

Here `F(0x3f7fbe77)=0.9990000128746033`. The depth division is evaluated
separately for the two depth entries. `tanf(hy)` is called before `tanf(hx)`.
The selected thunk resolves through the image's import table to
`MSVCR120.dll!tanf`; the loaded CRT and exact tangent results were not executed
or recovered. Supplying a different platform tangent therefore requires a
stated precision policy. The unordered/NaN branch is outside this finite-input
contract.

For an explicit view-space point, these rows give `clip.w=view.z` and
`clip.z=Q*view.z−Q*n` in real arithmetic. This is positive-forward perspective
with zero depth at the near plane. With finite `f>n`, depth at `f` is one;
with equal near/far arguments, depth approaches Q. Preserve actual binary32
operations for comparison rather than these simplified real equations.
This does not establish a GPU backend's complete viewport/depth state.

In the unconstrained branch of `0x140cc1050`, the caller computes:

```text
h = f32(f32(FOV * F(0x40490fdb)) * F(0x3b360b61))
if mode == 1 or (mode == 2 and W > H): xm=1,       ym=f32(W/H)
else:                                xm=f32(H/W), ym=1
hx = hy = h
```

The mode byte is read at LocalPlayer+`0x3e8`. **OBSERVED saved enum:** Core's
EAspectRatioAxisConstraint orders MaintainYFOV=0, MaintainXFOV=1,
MajorAxisFOV=2, MAX=3. Its association with this native byte is **INFERRED**
from the branches and property type, not a recovered reflected member offset.
The saved LocalPlayer default names MaintainXFOV; runtime inheritance was not
observed. The explicit W/H branch itself does not depend on that name mapping.

The caller passes the same writable global at `0x14292a6e0` as both near and
far. Its stored image float is 10.0; that is not an effective runtime near
distance. Original Rust math should accept an explicit near value instead of
treating the stored image as a selected camera setting.

The constrained branch uses a supplied aspect `A` and one tangent call:

```text
k = f32(1 / tanf(h))
P00 = k; P11 = f32(A*k); P22 = F(0x3f7fbe77); P23 = 1
P32 = f32(n * F(0xbf7fbe77)); all other entries = +0
```

The reciprocal-then-multiply and direct negative-constant product are observed
operations; substituting `A/tanf(h)` changes rounding. The native constraint
branch follows pointer/flag fields whose reflected member names were not
independently recovered. Expose this as an explicit selected policy.

## Viewport and normalized output boundaries

**VERIFIED selected wrapper:** GameViewportClient.GetViewportSize reaches
`0x140cce530`; virtual slots `+0x10/+0x18` return viewport width/height, converted
to binary32. Raw LocalPlayer fields `+0xa0/+0xa4` multiply those values for
origin and `+0xa8/+0xac` for dimensions; conversion uses truncating
`CVTTSS2SI`. Saved Origin/Size names do not independently prove those live
offsets or effective fractions. Passing an explicit rectangle avoids choosing
them.

The constrained rectangle helper `0x140a80510` additionally calls viewport
virtual slot `+0x130` for a scalar `s`. Its meaning and runtime value remain
unresolved. Its numeric contract can still be stated with explicit positive
finite inputs: viewport size `Vx,Vy`, requested aspect `A`, rectangle `W,H`,
and that extra scalar:

```text
E = f32(A / f32(s / f32(Vx/Vy)))
d = f32(E - f32(W/H))
if abs(d) <= F(0x3c23d70a): leave rectangle unchanged
if d > 0: newH=Round(f32(W/E)); originY=Round(f32(f32(H-newH)*0.5)); H=newH
else:     newW=Round(f32(H*E)); originX=Round(f32(f32(W-newW)*0.5)); W=newW
Round(v) = CVTSS2SI(f32(f32(v*2.0)+0.5)) arithmetic-shift-right 1
```

The integer dimension subtraction in the retained code is unsigned32; the
clean equations above require nonnegative representable differences. The
affected origin is overwritten, not added to the prior origin. CVTSS2SI
observes MXCSR rounding, unlike the earlier truncating size conversion. The
tolerance is approximately 0.01. Do not substitute square pixels, `s=1`, or
`s=Vx/Vy` without separate evidence. An initial Rust camera interface can keep
this crop outside its supported path while accepting a caller-resolved viewport.

The native point-transform helper at `0x140fa3700` consumes sixteen floats at
viewstate+`0x2b0` as a homogeneous row-vector transform. `0x140f92ee0` computes
`rw=f32(1/clip.w)` then multiplies xyz by rw, retaining clip.w; no zero guard
occurs in that selected body. Project's outer method returns normalized X as
`f32(f32(ndc.x+1)*0.5)` and Y as `f32(1−f32(f32(ndc.y+1)*0.5))`. Missing object
prerequisites instead return two zero floats. This is not a visibility test,
pixel-coordinate API or behind-camera rejection. The full viewstate matrix
constructor, downstream GPU installation and DeProject inverse path were not
derived. `point * V * P` is the proposed explicit mathematical interface using
the recovered matrices, not a claim that those omitted integration paths ran.

## Evidence, checks and implementation boundary

All extracted data, instruction records, reconstructed table bytes and numeric
fixtures remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/camera-math/`.
The combined `receipt.json` SHA256 is
`0f5b42a9d55bb03a1cc2fbcf6dd663dced9d434b0d172e27db592ce1fad446ca`;
the native canonical bundle SHA256 is
`195d7326c5b0e9f32e59ab4bfb3f8f7442fe832bec96a86ff8de74d7407ca500`.

| Evidence | What it establishes |
| --- | --- |
| `native/math-wrappers.json`, `matrix-helper.json` | Named registration/wrapper route, GetAxes rows and full rotation constructor. |
| `native/trig-body.json`, `trig-constants-rea.json` | Exact initializer/leaf, angle scaling and called sine entry. |
| `native/vector-sine.json`, `sine-fast-constants.json` | Full selected sine body, all fast-path constants and sign mask. |
| `native/source-byte-check.json` | 809 selected decoded instructions and 336 constant/table bytes match the pinned file exactly. |
| `native/numeric-check.json`, `verify_math.py` | All 16,384 table entries agree word-for-word between original instruction replay and derived formula; 266 matrix cases check factorization, full-turn wrap and low-bit quantization. |
| `projection/projection-contract.json` | Exact view/projection/crop formulas, native spans, import names, constant bits and per-record Evidence IDs. |
| `projection/inverse-replay-check.json` | All 2,144 output words agree across 134 synthetic inverse-rotation cases between original instruction replay and ordered products. |
| `projection/metadata/view-basis.json` | Independent constant-byte and multiplication-order check of translation and axis permutation. |
| `script/receipt.json`, `script/function-profiles.json` | Nine bounded saved camera bodies, typed references and checked branch destinations. |
| `script/fields/rotator-receipt.json` | Exact Pitch→Yaw→Roll declaration chain, with native offsets explicitly unresolved. |

The table replay assumes round-to-nearest/ties-even and uses an original
instruction interpreter; it never loads or calls the executable's machine code.
The rotation factorization maximum absolute difference is below `7.84e-8`
when comparing the grouped binary32 constructor with independently composed
elementary matrices in binary64. The maximum tested orthogonality residual is
below `5.76e-7`; exact orthogonality is not assumed after table approximation and
rounding. These checks establish transcription consistency, not native execution
or rendered parity. No repository build was needed for this documentation-only
increment.

REA 6.9.0 was used through selected, target-free Rizin reads of the explicit
owner-owned binary. The parent lane exported/checkpointed its eight Evidence
records without retaining a target; the projection helper used isolated CLI
reads. Two earlier broad discovery reads contain incomplete/invalid instruction
tails and remain excluded from final numeric proof, as does one malformed
argument error. Exact bounded replacements are cited above. Rizin's warning
about skipping large string-search regions is retained; no absence conclusion
depends on those skipped regions. Function unwind ranges can be fragments;
the FOV-adjust routine required an explicit continuation beyond its first range.

Outside-source provenance for this increment: the owner's pinned retail EXE,
Core/Engine/GameFramework packages and previously retained Vista evidence are
proprietary interoperability inputs; only facts and original analysis are used.
The local REA skill and its native/evidence workflow references are tooling
instructions, with no implementation copied. Existing original project readers
and earlier private extraction profiles supplied bounded package access. No
external engine source, third-party math implementation or web source was used.
Exact paths, tool versions, hashes and private source-ledger receipts accompany
the evidence and can be added to the integrator-owned provenance index.

## Original explicit-input module

`src/camera.rs` implements the supported contract without dependencies. It owns
an originally generated sine table and exposes raw `RotatorWords([i32;3])`,
`Pose`, row-major `Matrix4`, `DepthRange`, `FovAxis`, `ProjectionProfile` and
`ProjectionInput`. No camera position, rotation, FOV, viewport or depth is
selected by the module. `CameraMath::new()` constructs only the math context;
its Default implementation does the same.

```text
CameraMath::sin_cos(word)                         -> (f32, f32)
CameraMath::rotation_translation(pose)            -> Result<Matrix4, CameraError>
CameraMath::inverse_rotation(raw_words)           -> Result<Matrix4, CameraError>
CameraMath::view(pose)                            -> Result<Matrix4, CameraError>
CameraMath::projection(input, caller_tangent)     -> Result<Matrix4, CameraError>
half_fov_radians(explicit_fov_degrees)             -> Result<f32, CameraError>
```

The projection callback is `FnMut(f32) -> f32`, called for Y then X, even when
the angles agree. This makes the tangent implementation an explicit caller
choice. Passing Rust `f32::tan` uses platform math and carries no native CRT
precision claim. The supported profile is Unconstrained with explicit X, Y or
Major axis. ConstrainedAspect preserves its supplied aspect but returns
`UnresolvedConstrainedViewport`; it never substitutes a guessed crop or silently
uses unconstrained math.

The implementation validates finite position and FOV, strictly positive viewport
dimensions and near depth, far≥near, finite positive tangent results, and every
returned matrix entry. Equal near/far selects the recovered Q constant. It
rejects arithmetic overflow rather than returning a nonfinite matrix. These are
deliberate finite-input API boundaries; the original native forward constructor
would copy nonfinite position words, which this API rejects. No claim is made
about native error handling outside the supported input domain.

**VERIFIED focused implementation checks:** the standalone module compiled with
rustc 1.98.1 on x86_64 Linux, and the private harness compiled at optimization
level 2. Its generated table agrees in all 16,384 words with the frozen static
replay. Independently authored instruction interpreters, which did not read the
Rust module, supplied forward/inverse/view/projection oracles. The Rust harness
matches 77 checks and 1,061 words: 24 finite forward matrices, 16 inverse matrices,
16 view matrices, nine caller projection matrices, nine half-angle values and
three direct-helper depth subsets. The last three compare only the four depth
entries, because the direct helper fixtures have independent half-angles/scales
outside the public caller profile. Deterministic supplied tangent values make
the arithmetic comparison independent of the host's math library.

Eight native forward fixtures with nonfinite translations are correctly rejected
by the finite API. Thirteen further boundary checks cover equal-depth success,
invalid position/FOV/viewport/depth/aspect/tangent, unresolved constrained profile
and computed view/projection overflow. No routine suite, renderer, native camera
call or live client was run. `rust-oracle-check.json`, `rust-table-check.json`,
`validation-boundaries.txt`, the private original harness and frozen oracle
receipts retain the exact inputs, source hashes and limitations. These checks
were performed before the integrator's module-export hook; no full application
build is asserted by this lane.
The private `implementation-receipt.json` SHA256 is
`00e3ffcf2895d97a3695c582d714448e680e70435a0a0561f19fbd01074cc0e3`.

**Still unresolved:** live camera selection and modifiers; effective FOV/aspect/
near settings; camera animation and missing move-axis171; viewport scalar `s`;
the output target; loaded CRT tangent precision; runtime floating-point state;
and rendered behavior. The original module implements the explicit finite-input
rotation/view/projection contract with tangent precision and unsupported crop
policy stated. It does not establish the active retail Vista or a completed 3D
frontend.
