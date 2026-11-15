# Frontend hardware-cursor visibility call

This study follows the saved `FrontEnd.Start` call to
`MarvelPlayerController.SetHardwareMouseCursorVisibility`. It is a distinct
host API from AS2 `Mouse.hide`: the latter emits event `0x16` through selected
state 4, while this route uses an Unreal player-controller function. See
[REA_MOUSE_HOST.md](REA_MOUSE_HOST.md) for the still unresolved state-4
provider; no installation or cursor effect is inferred across those APIs.

The source is the owner's private Win64 build 1.52.0.1700 executable,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and the saved original MarvelGame package. All native evidence is static.
No original client, service, network, debugger, account or current scene was
run or probed; existing native/live/access holds remain unchanged.

## Saved call and Boolean input

The owned FrontEnd.Start token tree makes a virtual function-name call,
name index **10765**, to `SetHardwareMouseCursorVisibility` on local `mpc`.
Its argument is the result of
`mpc.m_gameSettings.GetUseHardwareMouse()`. The local is declared as
MarvelPlayerController, and the saved GetMPC body dynamically casts the result
of GetPC to that class. These are saved declarations and control flow, not
proof of a current controller, successful cast, runtime function selection or
absence of subclass overrides.

Under the retained token/operator profile, the guard is
`mpc != None && mpc.m_gameSettings != None`; its false destination is the
existing `return true`, not a cursor operation. In ordinary serialized order,
the caller first invokes the parent Start (without testing its result here),
obtains viewport size, calls OnStart, sets alignment, obtains `mpc`, then
checks this guard and makes the cursor call. Earlier calls and runtime
overrides may have effects. The exact cursor call is at serialized script
offset 137 / loaded offset 201 / package logical address 2367667.

MarvelGame export **5724**, logical `2126362+70`, declares the setter. Its
serialized seven-byte body is a NativeParm reference to `+5724` (parameter
export 5723, `bUseHardwareMouse`), then Nothing and EndOfScript; the declared
loaded length is 11. Function flags are `0x00020400`, native index 0 and
friendly name 10765/0. The parameter is a BoolProperty with flags `0x80`;
there is no optional/default argument token in this selected declaration.

The saved settings getter returns the instance's `UseHardwareMouse` field.
The owned Default__MarvelGameSettings explicitly serializes this property as
true, which establishes a stored class default only. Runtime settings,
configuration application and later changes are not observed, so the caller's
argument must not be replaced with unconditional true.

## Named native wrapper

**VERIFIED native registration:** ASCII name
`AMarvelPlayerControllerexecSetHardwareMouseCursorVisibility` is at
`0x14225db28`; registration row `0x1429e7480` pairs that name with wrapper
`0x1413673a0`. The `A` prefix is part of the actual name.

Complete wrapper `0x1413673a0..0x141367434` parses one expression into a
DWORD and normalizes its nonzero value to a Boolean at
`0x1413673e2..0x1413673e6`. After the bytecode terminator/optional-`0x41`
handling, it calls the actual receiver's **primary virtual `+0xca8` at
`0x14136741e`**, passing the normalized Boolean in EDX. This establishes the
native wrapper's input and dispatch contract, without treating the declared
base class as proof of every runtime receiver's concrete method table.

## Concrete controller method and ordering

The named MarvelPlayerController class constructor installs primary table
`0x142298728`; its `+0xca8` slot at `0x1422993d0` selects complete
**`0x1413e2f80..0x1413e30e1`**. This is a concrete class binding, not a
universal devirtualization of every possible subclass.

Let `P` be the controller and `b` the normalized Boolean. The method first
casts `P+0x42c` through `0x1409c7670` to LocalPlayer and `P+0x4a4` through
`0x141382f60` to MarvelHUD. These complete helpers use the corresponding
named native class builders and follow the object's class/superclass chain.
Null input or a nonmatching chain gives null; class resolution and its
failure branches remain actual prerequisites. Both casts occur before the
LocalPlayer-null test. If that first selected pointer is null, the method
returns without the subsequent cursor callbacks.

With a selected LocalPlayer `LP`, the concrete ordering is:

1. Construct the MarvelGFxMoviePlayer object iterator and send
   `SetGFXMouseVisibility(!b)` to every selected object, using reflected
   function lookup `0x140799ae0` and actual receiver virtual `+0x210`.
2. If the selected MarvelHUD exists, call its reflected `SetMouseIcon()`
   with no argument buffer. This branch runs for either Boolean value.
3. Otherwise, only when `b==1`, call `0x140cd30f0(LP[+0x98], P[+0x820],
   0, 0)`.
4. Send reflected `SetHardwareMouseCursorVisibility(b)` to `LP[+0x98]`.

The final native receiver is literally `V=LP[+0x98]`; a valid coherent
receiver is required on this selected path. The saved LocalPlayer.ViewportClient
declaration has type GameViewportClient, but its serialized metadata does not
establish native offset `+0x98`. That field-name linkage and the actual current
receiver class must not be treated as independently proved by the cast.
The GameViewportClient script contract below is conditional on selecting that
receiver/function contract. No null-viewport fallback is invented.
Lookup/dispatch side effects and subclass/script overrides are not flattened
into direct property writes without their own contract.

The reflected FName globals are independently tied to their initialization
literals: `0x142c4bc78` is `SetGFXMouseVisibility`, `0x142c4bcf8` is
`SetMouseIcon`, and `0x142c32128` is `SetHardwareMouseCursorVisibility`.
They are actual runtime names used by this method, not names guessed from the
method's intent.

### Iterator scope

Complete iterator constructor `0x1413cfae0..0x1413cfbd0` resolves the named
MarvelGFxMoviePlayer class. With the caller's EDX=0 it starts at index -1,
sets its initial exclusion mask to `0x200000000` or `0x40200000000` according
to global `0x142afd228`, advances once, then adds mask `0x200` and skips a
first candidate carrying that bit. Complete advance helper
`0x1406e6540..0x1406e66ae` scans global object-array entries in increasing
index order, skips null entries and objects whose `+0x10` flags intersect the
mask, and follows `+0x50` class / `+0x78` superclass links for the selected
class predicate. The normal profile assumes successful concrete class
resolution; its null/base-class alternatives are not hidden.

This is not merely the current frontend movie or the controller's own movie.
The callback-bearing loop repeatedly consults the actual iterator/global
collection; it is not a proved immutable snapshot. Arbitrary mutation of the
global collection during reflected calls is outside a simple fixed-list
implementation profile.

### Optional native cursor setup helper

Complete `0x140cd30f0..0x140cd311c` loads receiver `+0x70`. If nonnull it
calls that object's primary virtual `+0xa8`, retaining the supplied pointer
and two integer arguments. A nonzero return causes the single byte store
**receiver `+0x11c = 0x12` at `0x140cd310f`**. Missing backend or a zero
return skips that store. The helper does not establish the supplied pointer's
current asset, the backend's concrete type, or the OS effect of `+0xa8`.
Its byte store must not be assigned a property name solely from an expected
cursor enum layout.

## Selected movie-player and HUD scripts

**OBSERVED bytes; INFERRED token-profile semantics:** complete saved
MarvelGFxMoviePlayer.SetGFXMouseVisibility (export 8452, code 49/69 serialized
/ loaded bytes) checks its own `MouseMC` reference against None. When present,
it calls `GFxUI.GFxObject.SetBool("_visible", bEnable)` on that object.
`MouseMC` is declared GFxObject. The native caller supplies `!b`, but this
saved body contains no additional inversion or hardware API call. A current
MouseMC, its actual display identity, native SetBool dispatch and rendered
result remain separate facts; it must not become a fabricated global cursor
object.

Complete saved MarvelHUD.SetMouseIcon (export 5514, code 295/447 bytes) uses
**the HUD's own stored `mpc`**. It does not receive the outer Boolean or call
GetMPC/GetLP. It first obtains local LP by dynamically casting
`HUD.mpc.Player` to LocalPlayer. There is no preceding explicit JumpIfNot
guard for HUD.mpc or Player; the profile's ordinary Context/cast semantics
remain relevant. Neither this stored `mpc` nor its local LP is proved equal
to the native method's retained P/LP.

The script then rereads its own `mpc.m_gameSettings.GetUseHardwareMouse()`.
On the selected profile:

- If that getter equals true and its local LP is present, it checks the
  integer-converted `m_currentMouseCursorMode` against `mouseCursors.Length`.
  A valid index selects cursorData, then calls the final native
  `LP.ViewportClient.SetMouseCursorTexture(cursorData.Texture,
  int(registrationPoint.X), int(registrationPoint.Y))`.
- An invalid index inside this selected hardware branch exits without the
  software fallback.
- Only when the outer hardware/LP condition is false does it test its own
  `m_gameUI`; if present, it invokes that object's virtual
  `SetMouseIcon(int(m_currentMouseCursorMode))`. The declared game-UI type is
  MarvelGFxGameUI; its runtime override and downstream icon effect are not
  followed here.

Thus the HUD's settings read is distinct from the value already supplied to
the controller method. Reflected movie-player callbacks precede this reread;
no single shared snapshot or immunity from intervening mutations is implied.

The saved native texture declaration has required Texture2D plus optional
HotSpotX and HotSpotY integers with serialized zero defaults. The HUD supplies
explicit converted registration coordinates, while the no-HUD controller
fallback supplies explicit zeroes. These cases must remain distinct.
The named `UGameViewportClientexecSetMouseCursorTexture` registration
`0x14294b530` binds complete wrapper `0x140ae7560..0x140ae764f`. Its two
optional integer locals start at zero, and at `0x140ae763a` it **directly
calls `0x140cd30f0`**, passing the original receiver, parsed texture pointer
and both integers. Thus the HUD's selected final native call and the
controller's no-HUD fallback reach the same captured helper; this particular
wrapper edge does not require a guessed virtual slot.
Engine.GameViewportClient.EScriptMouseCursor ordinal **18 (`0x12`) is
SMC_Texture**; the separate MarvelHUD.ECursorMode ordinal 18 is
CM_TransferToStash. Neither their integer coincidence nor the native byte
store proves a physical property-offset identity.

## Saved viewport visibility transition

**OBSERVED saved bytes; INFERRED original-token semantics:** complete
Engine.GameViewportClient.SetHardwareMouseCursorVisibility is a script body,
not another native stub. Export 15060 has flags `0x00820902` (native bit
`0x400` clear), 118 serialized code bytes and 166 declared loaded bytes.
On the selected ordinary script profile it performs this ordered transition:

1. If requested `bIsVisible` is true and the prior
   `bDisplayHardwareMouseCursor` is false, obtain the viewport size and call
   `SetMouse` with each component divided by float 2 and converted to an
   integer. The exact midpoint token profile is `int(float_component / 2)`;
   no extra clamp or rounding rule is added.
2. Assign `bDisplayHardwareMouseCursor = bIsVisible`.
3. Call `ForceUpdateMouseCursor(true)` and return void.

The assignment and force-update call occur on ordinary completion even when
the requested value equals the old value. Recentering belongs only to the
false-to-true branch. Thus an implementation that merely sets a cursor-visible
Boolean would omit observable calls and ordering. This is saved code, not an
observed window movement or current flag value.

### Named native forwarding methods

The exact `UGameViewportClientexecSetMouse` wrapper is
`0x140ae7de0..0x140ae7e94`. It parses two integer arguments and calls receiver
virtual `+0x288` at `0x140ae7e7e`. On the separately evidenced concrete
MarvelGameViewportClient table `0x142279028`, slot `0x1422792b0` selects
**`0x140cd30d0..0x140cd30e4`**. This complete leaf reads `V+0x70`; if null,
it returns, otherwise it tail-calls that backend's primary virtual `+0xc0`
with the original X/Y integer arguments.

The exact `UGameViewportClientexecForceUpdateMouseCursor` wrapper is
`0x140ae7c50..0x140ae7ce4`. It parses and normalizes one Boolean and calls
receiver virtual `+0x280` at `0x140ae7cce`. On that same concrete table,
slot `0x1422792a8` selects **`0x140ccd180..0x140ccd191`**. This complete
leaf returns if `V+0x70` is null; otherwise it tail-calls backend primary
virtual `+0x70` with the original normalized Boolean. The selected script
supplies true.

Neither leaf calls USER32 directly. The backend's concrete class/table and
its `+0xc0`/`+0x70` effects are not established here; imported cursor API names
alone cannot fill that gap. These exact forwarding contracts support a
bounded host interface, not a claim of full platform cursor parity or proof
of the current viewport's concrete type.

## Implementation boundary

**INFERRED bounded implementation recommendation:** preserve the controller's
ordered reflected calls and each script's own receiver/state reads. For the
selected viewport script, retain the old visibility value for its branch,
preserve the viewport-size and coordinate-conversion calls, then perform the
assignment and unconditional force-update call. Repeated calls are observable
at the native forwarding interface even when the flag value is unchanged.

Known missing backend pointers make the two native forwarding leaves return;
they do not undo the script's prior visibility assignment. Unknown current
receivers, script overrides, property offsets or backend tables must not be
replaced with invented defaults. This is useful source/native-interface
evidence for a future host implementation, not a native cursor execution
capture, an implemented platform backend, or proof that the current frontend
ran the path. No VM, frontend, graphics or input code is changed here.

## Evidence and provenance

Private artifacts are under
`~/.local/share/marvel-heroes-forever-rust/evidence/hardware-cursor/`.
The package and native-binding lanes retain original bytes, exact source
hashes, canonical REA records and checks outside git. Initial import-table
navigation in `effect/import-navigation.json` only inventories cursor-related
imports; the presence of an imported name alone establishes no selected call
or cursor effect.

| Private evidence | Bounded verification |
| --- | --- |
| `package/receipt-method-edges.json` | Supersedes the initial package intake while preserving its files; exact saved declarations, complete selected token profiles, property references and two separate enums. |
| `native-binding/receipt.json` | Native controller name, wrapper, concrete class table and method slot: 88 instruction spans / 361 bytes plus 174 name/table bytes. |
| `native-binding/extended-receipt.json` | Three reflected FName identities and two cast class names: 114 instruction spans / 569 bytes plus 180 literal bytes. |
| `native-binding/viewport-edges/receipt.json` | SetMouse/ForceUpdate wrappers and supported concrete subtype slots: 136 instruction spans / 549 bytes plus 176 name/table bytes. |
| `native-binding/texture-edge/receipt.json` | Exact native texture wrapper's direct call to the captured optional setup helper: 58 instructions / 239 bytes plus 61 name/table bytes. |
| `effect/complete.bundle.json` and `effect/verification.json` | Four new records: controller method, casts, iterator, texture helper and forwarding leaves; 328 instructions / 1,244 bytes across eight exact ranges. Earlier session records in the bundle are outside these claims. |
| `receipt.json` | Joined document/source hashes and explicit remaining receiver/property/backend boundaries. |

The four new effect records are
`ev_490b630e5c4f5e2628c37d3efd82e4952afab53ca96461473810e1ff090546da`,
`ev_59331b645a50de012c466260c36142cebe24ef43f0d402ea1dce6573f8762c51`,
`ev_43d36d59c51fdfaa66b52efd911de80096c275835ed605de83323c72210e8333`,
and `ev_d2b0174db8fe2eb4f3a6e130889a126558e3fd900f8a288c35588dd2f3668b83`.
All completed with exit zero, no truncation and unchanged executable hash.
Independent PE mapping matches every selected instruction byte. Six nonleaf
ranges match complete chained unwind extents; the two small forwarding leaves
end at their observed null-return/interface-tail-jump boundaries, excluding
following `int3` padding. No Rust product test or native runtime smoke is
claimed for this documentation-only work.

Sources used: the owner's proprietary Gazillion executable, MarvelGame and
Engine packages (private interoperability study, no redistribution licence;
Engine SHA-256 `66dd5dca80f275a8cc2de5bf4b03042949cfccd0afc0edc6b7ba9b577cde4f51`);
original retained frontend package/token/default evidence and
[FRONTEND_EXTERNAL_RECEIVER.md](FRONTEND_EXTERNAL_RECEIVER.md),
[FRONTEND_INITIAL_INSTANCE.md](FRONTEND_INITIAL_INSTANCE.md); installed REA
skill/native-artifact guidance, REA 6.9.0 (MIT tooling), Rizin (LGPL-3.0
tooling), and Python standard library (PSF) for original navigation and byte
verification. The original bounded package reader and token parser were
reused as extraction/validation tools; their frozen sources and all retained
input paths/hashes are listed in the private package receipts. No outside
game implementation is copied. The source ledger is
for integration into [PROVENANCE.md](PROVENANCE.md).
