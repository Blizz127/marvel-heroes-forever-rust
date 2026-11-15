# SetMouse masked coordinate preparation

**VERIFIED static arithmetic and focused standalone witnesses:**
`src/frontend/cursor_coordinates.rs` implements the complete masked numerical
conversion of the selected retail SetMouse backend, with a separate finite
preflight. It performs no OS calls,
window queries, backend writes or cursor movement. The module is deliberately
not added to the frontend module tree in this increment.

The adapter takes explicit requested coordinates, sampled viewport dimensions,
and a sampled **GetWindowRect** rectangle. Its scaled variant explicitly admits
nearest-even arithmetic with floating exceptions masked; the adapter does not
inspect or write the floating control environment. The masked API returns
native integer-indefinite for invalid conversions; the finite API can decline
them. No actual window, rectangle, viewport size or
control state is inferred from settings or a saved scene.

## Exact native sequence

The owned 1.52.0.1700 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
contains complete method `0x14132a5a0..0x14132a6b8`. The selected backend
table binds it at `+0xc0`, as established in
[REA_CURSOR_BACKEND.md](REA_CURSOR_BACKEND.md). `B` below is that backend
interface, not the whole platform allocation or GameViewportClient.

**VERIFIED:** the method calls backend virtual `+0xd0` first. Its selected
leaf returns `(DWORD[B+0xac] >> 1) & 1`. A zero result bypasses all rectangle,
dimension and floating arithmetic. Requested X/Y remain their original signed
32-bit words, including `INT_MAX` and `INT_MIN`.

For a nonzero result, the method:

1. Reads `[B+0xb8]` and calls the exact imported `GetWindowRect`. Its return
   status is ignored. No zero-initialization of the local RECT is shown, so a
   failed call does not supply a fabricated zero rectangle to this adapter.
2. Computes `right-left` and `bottom-top` using 32-bit `SUB`. The adapter uses
   wrapping subtraction and then treats each resulting word as signed.
3. Converts requested X to binary32 with `CVTDQ2PS`, obtains the viewport width
   through virtual `+0x10`, zero-extends EAX into RAX, and uses `CVTSI2SS` on
   that signed 64-bit register. The resulting value is the conversion of a
   **u32** dimension, including values above `INT_MAX`.
4. Converts the signed rectangle width to binary32, performs `DIVSS` followed
   by `MULSS`, then truncates with `CVTTSS2SI` to the X integer register.
5. Converts requested Y, samples height through virtual `+0x18`, and repeats
   the corresponding single-precision divide/multiply/truncate sequence.

The selected dimension getters simply return DWORD `B+0xa0` and `B+0xa4`.
They are distinct reads at their actual positions in the sequence. A supplied
pair is not evidence of an atomic or current viewport snapshot.

For each axis the admitted numerical formula is:

```
span = signed32(wrapping_sub(rect_end, rect_start))
quotient = f32(requested_i32) / f32(viewport_u32)
product = quotient * f32(span)
result = masked_cvttss2si(product)
```

Each conversion and arithmetic step retains binary32 rounding under the
admitted profile. The multiplication must not move before division, and the
intermediates must not be widened to binary64. Rectangle left/top are used to
calculate spans; this local arithmetic does not add the window origin.
`GetWindowRect` is the observed import; substituting `GetClientRect` is not
supported by this evidence.

## Complete masked values and finite preflight

**OBSERVED standalone instruction behavior:** an original private C harness
uses the same conversion, `DIVSS`, `MULSS` and `CVTTSS2SI` operations, with
MXCSR explicitly set to `0x1f80` during each witness and restored on exit.
This retained original arithmetic test code neither loads nor executes the
original game. Its disassembly confirms the intended instruction sequence.
The masked extension reuses these witnesses; it does not rerun that harness
or add product control-register writes.

| Witness | Product / native integer observation |
|---|---|
| `867 / 1280 * 1920` | `1300.5`, truncates to `1300` |
| `-867 / 1280 * 1920` | `-1300.5`, truncates to `-1300` |
| `16777217 / 3 * 3` | Initial f32 rounding gives result `16777216` |
| `INT_MAX / UINT_MAX * INT_MAX` | Unsigned dimension conversion gives `1073741824` |
| `123456789 / 1234567 * 7654321` | Result `765432704`; multiply-first yields `765432640` |
| Largest f32 below `+2^31` | `2147483520`, valid integer conversion |
| Exactly `-2^31` | `INT_MIN`, valid conversion; invalid flag clear |
| `INT_MAX / 1 * 1` | Input rounds to `+2^31`; conversion gives `INT_MIN`, invalid flag set |
| `INT_MIN / 1 * 2` | Out-of-range negative product; `INT_MIN`, invalid flag set |
| `0/0`, positive/zero, negative/zero, infinity times zero | Observed NaN/infinity products; conversion gives `INT_MIN`, invalid flag set |

The native method has no checks for zero dimensions, nonfinite products or
integer overflow and no shown control-register setup. Its actual ambient
rounding/masking state remains unknown. The masked instruction witnesses do
not establish unmasked exception delivery or current retail floating state.
NaN bit patterns in the private receipt are observations of that controlled
harness, not an adapter promise.

**Implemented complete numerical path:** `prepare_set_mouse_masked` accepts
every supplied signed coordinate/rectangle word and unsigned viewport dimension
under `NearestEvenMasked`. It truncates finite products in `[-2^31, +2^31)`
toward zero. Nonfinite products and finite conversion overflow return
`i32::MIN`, matching masked CVTTSS2SI's integer-indefinite value. The upper
bound is exclusive; converting `INT_MAX` to f32 cannot supply an inclusive
maximum. Exactly `-2^31` is a valid conversion with the same integer value
as integer-indefinite. The numeric result does not distinguish those cases.

A zero denominator is now included: zero/zero produces NaN, a nonzero numerator
produces signed infinity, and multiplying infinity by a zero span produces
NaN. All of those products convert to integer-indefinite in the masked profile.
No Rust saturating-cast behavior is substituted. Both axes are calculated and
returned even when the first conversion is invalid. Unscaled mode still
preserves the original integer words without a floating round trip.

`prepare_set_mouse` remains a separate **finite preflight**. It rejects a zero
dimension or a product outside the representable finite range with an explicit
`CoordinateError`, returning no prepared payload. This is not native partial
execution: an invalid X in the native masked path continues to the height
getter, Y conversion and both backend stores. Earlier native calls may already
have effects; this pure preflight does not emulate, undo or roll them back.

Neither API reports, reproduces or preserves MXCSR status flags. For example,
the Rust branch returning MIN for overflow need not raise the native conversion's
invalid flag. Regular arithmetic may affect hardware status flags, but this
module contains no explicit MXCSR inspection or control write. Unmasked
exceptions, other rounding modes and actual control state remain outside the
claimed profile. This is complete numerical behavior for the stated input and
arithmetic profile, not a complete host-effect or processor-status emulator.

## Why DAZ and FTZ do not change this profile

**VERIFIED conditional operand bound:** a nonzero
signed input or signed wrapping span converts to a normal f32 with magnitude
between `1` and `2^31`. A nonzero u32 dimension converts to a normal f32 between
`1` and `2^32`; `UINT_MAX` rounds to the latter endpoint under nearest-even.
Therefore any nonzero quotient has magnitude from `2^-32` through `2^31`.
Multiplication by a nonzero converted span yields magnitude from `2^-32`
through `2^62`. These endpoints are representable and all lie in the normal
binary32 range. A zero numerator or span with nonzero dimension gives zero;
a zero dimension produces NaN or infinity, never a subnormal intermediate.

Consequently no nonzero subnormal input or intermediate reaches these SSE
operations for any supplied i32/u32 words. DAZ and FTZ therefore cannot alter
the numerical result in this profile. Their actual bit values need not be
invented or changed. The nearest-even and masked-exception requirements still
apply; the bound does not establish current retail control state.

## Pure handoff and actual effect ordering

**VERIFIED:** after the optional arithmetic, the native method reloads
`[B+0xb8]`, stores X at `B+0x55c`, stores Y at `B+0x560`, then calls
`ClientToScreen(handle, &backend_point)` at `0x14132a682`. This handle read
occurs after the earlier rectangle query; their identity/preservation is not
inferred.

The exact ordered anchors are X conversion `0x14132a630`, Y input conversion
`0x14132a639`, height getter `0x14132a63c`, Y integer conversion `0x14132a65d`,
handle reload `0x14132a667`, X store `0x14132a675`, then Y store
`0x14132a67b`. The width getter was at `0x14132a60e`, after conversion of the
requested X input. There are no backend coordinate stores between the two
axis conversions. An invalid conversion under the masked profile does not
take a native rejection branch.

On normal return it ignores the imported status, reads backend Y and then X,
and calls `SetCursorPos(X,Y)` at `0x14132a694`. The final imported status is
also unchecked. The method contains no bounds clamp, focus check or implied
successful cursor movement on this path.

The pure API separates those stages:

- `prepare_set_mouse_masked` returns the complete masked coordinate pair to
  store, including integer-indefinite values. `prepare_set_mouse` separately
  offers finite preflight. Neither performs getters, stores or partial effects.
- `client_to_screen_request` attaches an explicitly supplied window token to
  that pair. The caller remains responsible for the actual X-then-Y stores
  and host call; constructing these values performs no effects.
- `after_observed_return` requires the actual post-call backend coordinate
  reads. It ignores the supplied reported-success Boolean, matching the
  native decision, and returns a `SetCursorPosRequest` containing those reads.

`WindowToken` is a non-owning integer identity with no default, resolution,
destructor or native-pointer access in this module. It cannot own or close a
host window. There are no caller-supplied callbacks in the adapter.

Failure of ClientToScreen does not imply that its input point was unchanged.
Nor may its output be fabricated by adding GetWindowRect's left/top: that
rectangle does not establish the conversion result. The caller must supply
the actual observed result or leave this handoff unavailable. These data types
do not execute or verify host integration, OS success or current window binding.

The previously recovered named visibility field remains conditional on its
native metadata profile and V0/V1 identity. This arithmetic adapter does not
clear those gates or connect the separate MovieImpl state-4 `Mouse.hide` path.

## Verification and provenance

Private evidence roots are `evidence/cursor-coordinates/` and
`evidence/cursor-coordinates-masked/` under
`~/.local/share/marvel-heroes-forever-rust/`. The latter receipt supersedes the
initial finite-only intake and retains immutable old/new source snapshots.
The earlier receipt and controlled oracle remain unchanged.
The replacement `cursor-coordinates-masked/receipt.json` SHA-256 is
`854a10476b071a1ad0b08c9bf538f43647f3860ada4e156b800262aa6eeb5ec2`.

`native-audit/verify.py` independently rechecks the retained REA source bytes,
complete main function extent, three selected leaves, table pointers and exact
import identities: **77 instructions, 306 code bytes and 90 data bytes**. Its
retained records are `ev_d9d47a10ccac3f1592debcffa905296dcb699f725c4e2848e50b67c21c7171d1`,
`ev_0e022be4f409dd1b9e7829e1ef6c393aaa6d392f739262ee88d9b0938aa4e583`
and `ev_0c65fd63d51f6a7caa81063f12ce28352e17393c23360dd3f96c69373144602a`.
No duplicate original-binary capture was required.

The original `cursor-coordinates/oracle/` retains standalone C source, compiler versions,
disassembly and **23 controlled witnesses** including six exceptional cases.
All 23 integer outputs are now checked by the complete masked path, while the
17 finite product bit patterns remain checked independently. **Seven focused
tests pass** in standalone debug and optimized builds. They also cover unscaled
integer extremes, rectangle wrapping/origin handling, finite preflight errors,
continuation after invalid X or Y, and the supplied post-call point for either
imported status. The extension adds a focused canonical sequence audit of the
getter, conversion and X/Y store ordering without recapturing the original.

The original game was not started, no window/cursor API was invoked, no full
suite ran, and no module hook or surrounding product behavior changed. These
are focused static and controlled arithmetic checks, not a retail playtest or
wire/visual parity claim.

| Source | Licence / use |
|---|---|
| Owner's pinned executable, exact hash above | Proprietary owned artifact; retained private compatibility evidence, never redistributed |
| Existing original `REA_CURSOR_BACKEND`, `REA_HARDWARE_CURSOR`, `REA_CURSOR_VISIBILITY_PROPERTY` and private cursor-backend receipts | Original project evidence and inherited provenance; native arithmetic, input and effect boundaries |
| Retained REA 6.9.0 / Rizin output and installed REA skill | Existing ledger records MIT / LGPL-3.0-only tooling; no implementation copied |
| Original private C/SSE harness, original Rust adapter/tests and Python PE checks | Newly written work; system compiler/runtime used only for private analysis, no new client dependency or linked external implementation |

Raw native evidence and compiled test artifacts remain outside git. Only the
original Rust module and this document are included in the change.
