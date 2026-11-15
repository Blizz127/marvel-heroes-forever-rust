# Retail Windows cursor backend

## Scope and evidence status

**VERIFIED, static artifact:** the owned 1.52.0.1700 executable contains a
Windows viewport interface whose cursor methods reach the named USER32 imports
`SetCursorPos` and `SetCursor`. This is a conditional native host contract. It
does not establish a current viewport, current window handle, successful OS
operation, or the installed state-4 handler used by `Mouse.hide`.

This extends [the hardware visibility call](REA_HARDWARE_CURSOR.md), which
already binds the reflected `GameViewportClient` calls to the `V+0x70` backend
slot. [Mouse.hide](REA_MOUSE_HOST.md) uses a separate MovieImpl state-4 route;
these APIs must not be joined without an actual provider witness.

All addresses below are preferred image VAs in the private executable
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`
under `~/.local/share/marvel-heroes-forever-rust/`. Its SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No original process, cursor API, input injection, window creation, service or
network endpoint was exercised.

## Interface and dispatch

Let `V` be the whole GameViewportClient receiver, `N` a selected platform
allocation, `B=N+8` its viewport interface, and `C` its client interface.
Offsets on `B` must not be applied directly to `N` or `V`.

The named MarvelGameEngine initialization path reaches base initializer
`0x140b54060`. Its selected viewport factory result `V0` is stored at
`engine+0x6dc` and checked for null. At `0x140b56398` it supplies `V0+0x60`
as the client argument to platform-client virtual `+0x270`, retaining the
returned frame `N`. Later it reloads `V1=[engine+0x6dc]` and calls `V1+0x2a0`
with that frame at `0x140b564aa`. These are explicit selected-class conditions,
not proof that current engine configuration chose those implementations.
Intervening calls also mean `V0==V1` requires preservation; the reload alone
does not prove equality.

**VERIFIED:** complete constructor `0x14131f790..0x14131fd41` installs primary
table `0x1422277e0` at `N`, cursor-interface table `0x1422277f8` at `N+8`,
and another table at `N+0x18`. It returns `N`. Its original third argument is
passed to base constructor `0x140a789e0` as the client `C`; that base stores it
at `B+0x44` at `0x140a78a39`. The constructor publishes `N` in a global array
before its later window/resize setup. A returned allocation/interface does not
alone prove a valid current OS window.

The selected factories allocate `0x5dc` bytes and call that constructor:
`0x141321c50` returns `N`, while `0x141321d00` returns `B` on success.
Their containing WindowsClient table is `0x142227528`, at offsets `+0x270`
and `+0x278`. The frame interface's first virtual method `0x140de0460`
also converts nonnull `N` to `N+8`.

**VERIFIED:** for the selected Marvel viewport table, `V1+0x2a0` dispatches
to complete `0x140cd4c00..0x140cd4c41`. It stores the supplied frame at
`V1+0x78`; for nonnull frames it calls frame virtual `+0` to obtain `B`,
then tail-dispatches viewport virtual `+0x2a8`. A null frame passes null to
that same setter. There is no allocation-null guard before this attachment
on the selected initializer route. The factory's retained client remains
`C=V0+0x60`, even though the attachment receiver is the reloaded `V1`.

**VERIFIED:** viewport setter `0x140cd4b60` stores its incoming interface
argument at `V+0x70` at `0x140cd4b75`. Its later reflected/virtual callbacks
are separate mutation boundaries; this store does not prove that the field
retains the value after all callbacks, or that a particular current receiver
traversed this path.

The selected cursor table binds the following exact slots:

| `B` virtual slot | Function | Contract |
|---|---|---|
| `+0x10` | `0x1400f6830` | Return DWORD `B+0xa0` |
| `+0x18` | `0x140a8ea00` | Return DWORD `B+0xa4` |
| `+0x38` | `0x140074030` | Return QWORD `B+0xb8` |
| `+0x48` | `0x1413242c0` | Compare `GetFocus()` with `[B+0xb8]` |
| `+0x70` | `0x14132b5a0` | Cursor refresh described below |
| `+0x90` | `0x141323560` | Sample and convert OS cursor X |
| `+0x98` | `0x141323630` | Sample and convert OS cursor Y |
| `+0xa8` | `0x14132a6c0` | Selected texture-cursor path |
| `+0xc0` | `0x14132a5a0` | Selected SetMouse path |
| `+0xc8` | `0x140a92820` | `DWORD[B+0xac] & 1` |
| `+0xd0` | `0x140a927b0` | `(DWORD[B+0xac] >> 1) & 1` |
| `+0x160` | `0x1413252d0` | Return DWORD `B+0x59c` |

The dimensions, scaling bit, override word and window-handle argument remain
actual native inputs. This study does not invent their fresh or current values.

## Selected client callback and visibility-layout boundary

**VERIFIED:** the named MarvelGameViewportClient constructor installs table
`0x142279308` at `V0+0x60`; its slot `+0xa0` selects complete consumer
`0x140ccd270`. Thus the typed factory path supplies `C=V0+0x60` to this
consumer. The function's executable code ends at `0x140ccd43c`, followed by
a separately checked 76-byte switch table through `0x140ccd488`.

For supplied `(C,B,X,Y)` it preserves the following ordered predicates:

1. An optional manager at `0x142c48b20` supplies Boolean predicate `D` via
   virtual `+0x40`; absent manager means false. The service/result meaning
   is unresolved. Its addressed argument at `0x141fb8ca4` begins with a zero
   UTF16 code unit; that does not identify the service.
2. It calls `B+0x38` twice, once before `ClientToScreen` of the supplied
   point and once before `GetWindowRect`. Under the selected accessors these
   read `[B+0xb8]` twice. Imported statuses are ignored. It computes signed
   inclusive containment of the converted point in the rectangle, then
   `A = (original Y < 0) && containment`. This uses the original Y sign.
   No valid rectangle values are supplied by this study when an import fails.
3. If DWORD global `0x142aee944` is nonzero and low-byte masks `0x08` and
   `0x10` in DWORD `[C+0xa8]` are both set, it returns native code `2`.
   This path bypasses the following visibility checks.
4. Otherwise it rereads byte `[C+0xa8]`. Clear mask `0x02`, or true `D`,
   returns `0`. If both gates pass, it calls `B+0xc8`. A zero result with
   true `A` also returns `0`; other combinations proceed.
5. Finally it reads byte `[C+0xbc]`: values `0..18` return `value+2`, and
   values `19..255` return `2`. No path in this consumer returns `1`.

The raw visibility gate is therefore **`V0+0x108`, mask `0x02`**, and the
raw selector is **`V0+0x11c`**. The named SetMouseCursorType API below writes
the same selector storage when its receiver is that `V0`; equality with an
attachment receiver `V1` is an explicit condition. API enum ordinal `18`
(`SMC_Texture`) maps through this consumer to native code `20`, explaining
the two separate numeric domains.

**UNRESOLVED:** the saved BoolProperty `bDisplayHardwareMouseCursor`
(Engine reference `30407`) has not been bound to native offset `0x108` and
mask `0x02` by a concrete linked UProperty or named layout registration.
The equivalent reflection-layout link for saved `MouseCursorType` reference
`30665` is also unresolved. Exact named class construction/link helpers end
at a class virtual `+0x118` boundary without assigning these named offsets.
Behavioral similarity and the API's matching enum do not close that gap.
Consequently the source-script Boolean assignment is not yet proved to set
this raw gate, and the raw gate alone is not a complete visibility decision.

## SetMouse reaches SetCursorPos

**VERIFIED:** the complete `0x14132a5a0..0x14132a6b8` path receives signed
integer X/Y. If the `+0xd0` accessor is nonzero, it calls the exact imported
`GetWindowRect([B+0xb8], &rect)` and transforms each coordinate as follows,
preserving the observed single-precision operation order:

```
X = cvttss2si((f32(X) / f32(u32(B.width))) * f32(rect.right - rect.left))
Y = cvttss2si((f32(Y) / f32(u32(B.height))) * f32(rect.bottom - rect.top))
```

The imported function is **GetWindowRect**, not GetClientRect. No zero-size,
range, API-success or conversion-exception guard appears on this path. The
notation records the x86 instruction sequence; it does not substitute Rust
integer-cast behavior for `cvttss2si` edge cases.

It writes the resulting point to `B+0x55c` and `B+0x560`, calls
`ClientToScreen([B+0xb8], &point)` at `0x14132a682`, then passes the resulting
point to `SetCursorPos` at `0x14132a694`. The imported return values are not
checked. Without the scaling bit, the original integer pair goes directly to
that point/conversion/cursor-position sequence.

**INFERRED interoperability requirement:** a replacement host implementation
needs explicit viewport dimensions, coordinate transforms and host-window
binding. A test that merely updates a Rust coordinate variable cannot verify
this native effect, and this static evidence does not authorize moving the
owner's current cursor.

## ForceUpdateMouseCursor selects an effect

**VERIFIED:** `0x14132b5a0` receives a Boolean-like integer argument `apply`.
Its exact complete code ends after RET at `0x14132b7d2`; the same PE unwind
extent also contains padding and a 19-entry switch table. Those data bytes
are excluded from executable-code claims.

If `DWORD[B+0x59c]` is nonzero, it immediately calls
`LoadCursorW(NULL, 0x7f00)` and `SetCursor`, irrespective of `apply`, cursor
position or the client callback.

Otherwise it obtains X and Y from the two separate virtual accessors. Each
accessor independently calls `GetCursorPos`, then
`ScreenToClient([B+0xb8], &point)`. Thus the two samples are not a proved
atomic point. With the scaling bit set, it obtains `GetWindowRect` and uses
the inverse single-precision transform:

```
X = cvttss2si(f32(u32(B.width)) * (f32(point.x) / f32(rect.width)))
Y = cvttss2si(f32(u32(B.height)) * (f32(point.y) / f32(rect.height)))
```

Without that bit, the corresponding converted point component is returned
directly. The API results and zero rectangle dimensions are not checked.

The refresh initializes DWORD `B+0x4e8` to `2`. For nonnull `C=[B+0x44]`,
it invokes virtual `C+0xa0` with `(C, B, X, Y)` and stores the returned cursor
code at `B+0x4e8`. The precise generated interface-null adjustment passes null
instead of `B` when `B-8==0`; ordinary valid nonnull allocations take `B`.
This is a callback boundary, not a saved property or immutable snapshot.

It then requires signed `X>=0`, `Y>=0`, `X<width` and `Y<height`. Failure
returns `1` without calling `SetCursor` on this normal path. In-bounds cursor
codes select the following effects:

| Native cursor code | Selection |
|---|---|
| `0`, with `GetFocus()==[B+0xb8]` | Null cursor handle; `SetCursor(NULL)` only if `apply!=0` |
| `0`, without that equality | Fall through to default resource `0x7f00` |
| `1` | Return without changing the cursor |
| `2` or other out-of-range code | `LoadCursorW(NULL, 0x7f00)` |
| `3..9` | Null module, resource IDs `0x7f03, 0x7f86, 0x7f83, 0x7f82, 0x7f84, 0x7f85, 0x7f89` respectively |
| `10` | Module value at `0x142c48b28`, resource `0x7d1` |
| `11..19` | Same module value, resources `0x7db..0x7e3` respectively |
| `20`, nonnull `[B+0x508]` | Direct `SetCursor([B+0x508])`, irrespective of `apply` |
| `20`, null `[B+0x508]` | Default resource `0x7f00` |

After ordinary resource loading, it calls `SetCursor` only if `apply!=0`.
The nonzero override and nonnull custom-handle branches bypass that test.
All normal exits return `1`; this is not a report that the cursor changed or
that the operating system accepted a call. No ShowCursor call occurs in this
complete refresh body. No claim about a process-wide show-counter follows.

These are **native** cursor codes. In particular, native code `20` must not
be confused with saved Engine enum value `18` (`SMC_Texture`) or a HUD mode
enum that also happens to contain value `18`.

## Texture-cursor edge

**VERIFIED:** `0x14132a6c0..0x14132a757` passes `B-8`, the supplied Texture
and hotspots, plus `&B+0x508`, to helper `0x141325780`. This helper's texture
conversion, allocation and prior-handle lifetime are outside the present
closure. A false return makes the interface method return false.

After a true helper result, the method takes fresh separate X/Y samples and
checks viewport bounds. An outside point returns false even though the helper
has already run. An inside point calls `SetCursor([B+0x508])` and returns
true. The higher-level `0x140cd30f0` sets `V+0x11c` to `0x12` only on that
true result, as recorded in the hardware-cursor study. Neither result proves
that a particular texture or handle currently exists.

**VERIFIED, named native setter:** the registration pair at `0x14294b540`
binds `UGameViewportClientexecSetMouseCursorType` to complete wrapper
`0x140ae7490..0x140ae7507`. It decodes one byte and directly calls leaf
`0x140cd3120..0x140cd313f`. If `[V+0x70]` is null, that leaf skips both
storage and refresh. Otherwise it writes the supplied byte to `V+0x11c`
at `0x140cd312c`, then tail-calls `B+0x70` with integer argument `1`.
The saved native function has a required ByteProperty parameter typed by
`EScriptMouseCursor`; the wrapper's initially zero local does not make that
parameter optional. This binds the named API to raw byte storage. It does
not prove the reflected `MouseCursorType` property's native offset.

## Construction API consequence

**INFERRED implementation recommendation:** keep the host cursor interface
attached to an explicitly supplied viewport/window binding. The API needs
distinct operations for coordinate placement, cursor-resource selection and
texture-handle selection, plus a focus query and the actual dimension/scaling
inputs. Preserve the observed callback and sampling order: the two position
queries, client cursor selection, bounds checks and eventual OS call are not
interchangeable with one unconditional cursor update.

The source script's visibility assignment and its native client callback must
be joined through a proved property layout or an explicitly supplied layout
profile. The raw bit test alone does not justify silently mapping a saved
property name. A Rust implementation can expose these unresolved inputs and
verify finite supplied-input behavior; it cannot call that current retail
cursor parity. The operating-system effect remains a separate integration
check after an authorized actual window binding exists.

## Evidence, validation and remaining boundary

The private evidence root is `evidence/cursor-backend/`. Complete REA replies,
exported bundles, original navigation/checking scripts and byte checks remain
there, outside git. The selected new effect records are:

| Record | Content |
|---|---|
| `ev_d9d47a10ccac3f1592debcffa905296dcb699f725c4e2848e50b67c21c7171d1` | Three complete cursor method ranges and selected table slots |
| `ev_0e022be4f409dd1b9e7829e1ef6c393aaa6d392f739262ee88d9b0938aa4e583` | Complete accessors, focus predicate, switch table and slots |
| `ev_0c65fd63d51f6a7caa81063f12ce28352e17393c23360dd3f96c69373144602a` | Exact USER32 import strings and IAT words |
| `ev_854c48d15fd1f0c70defb64ff3b114f87214c38f750da193197e400bfc4f1676` | Complete backend handle/low-bit accessors and selected slots |

`effect/verify.py` independently PE-maps every selected byte, checks complete
chained unwind ranges or explicitly bounded leaf returns, internal direct
branch boundaries, exact switch targets and import-thunk/name identity.
It passes **4 effect REA records, 40 ranges and 372 selected instructions**.
The original ForceUpdate disassembly also rendered its inline switch data as
instructions; the verifier and this document explicitly exclude those bytes.
No product code changed and no native startup, visual or input test ran.

The producer receipt checks 8 new canonical records, 5,030 instructions,
21,592 code bytes and 68 data bytes; its separately checked named engine-class
receipt adds 102 instructions, 480 code bytes and 74 data bytes. This includes
the selected complete engine initialization and frame attachment, not a scan
of arbitrary field-offset matches.

The field-binding receipt checks 8 new REA records, 31 ranges, 2,259 bytes,
461 instructions and 14 complete unwind groups, including retained named
class evidence. The disjoint cursor-type receipt independently verifies its
150 code bytes, 41 instructions, 58 literal/table bytes and two package
declarations. Its initial mismapped registration chunk was excluded and
replaced by a canonically captured, independently PE-checked mapping; the
private findings retain that correction.

The producer joins are conditional on selected classes, ordinary ABI and
normal callee return. Current main-window identity, current visibility,
`V0==V1` preservation, setter callback preservation, exact reflected-property
layout and actual OS success remain unverified.

## Source and licence ledger

| Source | Licence / use |
|---|---|
| Owner's pinned 1.52.0.1700 MarvelHeroesOmega.exe, hash above | Proprietary owned artifact; static compatibility/interoperability study only; never redistributed |
| Owner's pinned `pins/animation-runtime-class-defaults/UnrealEngine3/MarvelGame/CookedPCConsole/Engine.upk`, SHA-256 `66dd5dca80f275a8cc2de5bf4b03042949cfccd0afc0edc6b7ba9b577cde4f51` | Proprietary owned package; selected SetMouseCursorType function/parameter declaration, private only |
| REA 6.9.0, local Rizin provider, `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md` and native reference | REA MIT licence recorded in the main ledger; Rizin licence not newly inspected; analysis tooling/evidence only, no client implementation copied |
| Existing project `REA_HARDWARE_CURSOR.md`, `REA_MOUSE_HOST.md`, retained `frontend-external-receiver` evidence | Original project findings reused for named receivers and earlier dispatch edges; same owned-artifact restrictions |
| Private `cursor-backend/` original scripts and retained evidence, including the field-binding/cursor-type source ledger | Original analysis/checking code and private results; original project UPK readers reused, Python standard library under PSF licence; no outside source implementation or runtime state substituted |

No SDK, decompiled third-party source, external cursor implementation or new
dependency was imported into the Rust client.
