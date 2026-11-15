# Native MovieClip subclass construction and `super()`

Task `a3d58c579542`, private interoperability study, 2026-11-15 UTC.

This investigation follows the original manager's zero-argument `super()` call
inside the Tween constructor. It changes no VM implementation and does not run
the retail executable.

## Original call and current scope

**OBSERVED:** The manager's Tween initializer is at movie offset 321,958 with
length 1,774. The AVM1 lane identifies constructor definition 322,404, body
322,415 + 188 bytes, flags `0x119`, and zero-argument undefined-name
`CallMethod` at 322,430. The call reuses the newly constructed Tween receiver
through `super`; it does not request a separate display object.

An independent private-byte check confirms `CallMethod` byte `0x52` at 322,430
followed immediately by `Pop` byte `0x17`. The saved body hash and offset receipt
is `evidence/rea-movieclip-constructor/actual-call-receipt.json`. The movie is
the existing private `MarvelGame-frontend_manager.gfx` extraction, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.

## Complete native callback behavior

**VERIFIED (static instructions):** Callback `0x1401955c0..0x14019561f`
contains 30 instructions and these two result paths:

| Receiver supplied in call-structure field `+0x10` | Behavior |
| --- | --- |
| Null, or virtual category method `+0x10` returns a value other than `0x0b` | Clean up the previous result value and set its tag to zero, independently established as `Undefined`. |
| Non-null, category `0x0b` | Return a reference to the same underlying receiver as an object value. |

The callback reads the result pointer at call-structure offset `+0x08` and the
receiver at `+0x10`. It does not read argument count, argument indices or a
constructor-mode flag. Its only virtual call queries the receiver category.
There is no allocation or write to receiver properties or display state in
this body. Reference-count maintenance associated with the result is separate
from script-visible receiver mutation.

For the category-`0x0b` path, `0x1401955f5` tailcalls `0x1401936f0` with the
result pointer and the receiver interface minus `0x20`. Existing REA evidence
shows that helper's non-function path at `0x1401937aa..0x1401937e6` writes
internal object tag `6` and retains the same object pointer. It does not create
a replacement receiver. The other callback branch writes tag zero at
`0x140195616`; null/undefined internal tag identities are independently bound
to the raw Push opcodes in the previous native Function study.

The allocation and dispatch paths below establish which branch this particular
Tween construction reaches. They remain separate from the callback: omitting
the native allocation category because the callback has no allocation would
change the object being constructed.

## Global binding and actual MovieClip prototype

**VERIFIED (static data and instructions):** Global setup constructs a callable
object whose callback is `0x1401955c0` and whose primary vtable is
`0x142514280`. The object remains in `RSI` through registration. At
`0x14020fb9d` it is stored as a function value under builtin name offset
`0x168`, independently tied to literal `MovieClip` at builtin index 6.
Its primary invoke slot `+0x50` is the native wrapper `0x14018c5b0`.

The actual prototype is allocated at `0x14020f608..0x14020f62f` and initialized
by `0x1401eb640`, which calls `0x1401e5470`. Setup passes the saved MovieClip
function reference and the Object prototype parent. The result is inserted
in the builtin prototype map under index 6 at `0x14020f63c..0x14020f689`.
The constructor installs interface vtable `0x142514db0` at object offset
`0x20`. Its relevant methods are:

| Interface slot | Method | Established behavior |
| --- | --- | --- |
| `+0x10` | `0x1401da7b0` | Returns native category `0x0b`. |
| `+0x28` | `0x1401a1540` | Existing own-hash raw lookup. |
| `+0x38` | `0x1401e3ef0` | Existing own-property flag setter. |
| `+0x50` | `0x1401eb5e0` | MovieClip hook followed by generic raw assignment. |
| `+0x58` | `0x1401e6020` | Constructor-aware member lookup. |
| `+0x78` | `0x1405b93b0` | False; this prototype is not a Super wrapper. |

Helpers `0x1401b80f0` and `0x1401a1560` establish both directions of the link.
They store the native MovieClip function reference at prototype offset `0x80`,
create its own public `constructor` entry as special tag `0x0a` with flags
`3`, and assign the prototype object to the constructor's `prototype` member,
also with flags `3`. The special tag is a raw marker; interpreting it as a
user accessor is not established.

The native prototype's member getter `0x1401e6020` handles public
`constructor` through `0x1401a16c0` with auxiliary pointer `prototype+0x78`.
For that special entry the helper reads the stored function reference at
auxiliary offset `+8`, producing the same native MovieClip function value.

## Allocation uses public `constructor`

**VERIFIED (static control flow):** Native `NewObject` and `NewMethod` call
shared helper `0x1401a74c0`. For the motivating zero-argument construction,
the successful path does the following:

1. Read the original constructor's `prototype` member, builtin offset `0x3a0`.
2. Read that prototype's public `constructor`, builtin offset `0x3b0`, and
   use a usable function reference to select an allocator.
3. Invoke the selected function's primary virtual slot `+0x78` to allocate
   the receiver, then bind its prototype and hidden `__constructor__` to
   the original script constructor, using the established flags-`3` setter.
4. Invoke the original script constructor's slot `+0x50` with that receiver.
   After successful allocation, discard the script constructor's result and
   return the allocated receiver.

At `0x1401a79e1`, the hidden-constructor setter's argument comes from
`[RBP+0x48]`, the prologue's saved incoming original function reference. The
same saved reference supplies the later script invocation at
`0x1401a79f4..0x1401a7a48`; the selected allocation function is separate.

The original manager executes `Extends` at 322,632 with superclass MovieClip.
Its fresh prototype uses interface `0x142513cd0`. Constructor `0x1401e4310`
initializes the internal public constructor reference to null. Assigning this
object to the script function's `prototype` through `0x1401911c0` and raw
setter `0x1401e3b20` preserves that null reference; the setter only updates
the function's own property. `Extends` separately installs hidden
`__constructor__`, builtin offset `0x3b8`, as the superclass reference.

The fresh prototype's constructor-aware getter `0x1401e7c40` therefore follows
its parent for public `constructor`. That parent is the actual MovieClip
prototype established above. Its stored public constructor resolves to the
native MovieClip function and selects allocator `0x1401de5e0` from primary
vtable `0x142514280`, slot `+0x78`.

That allocator requests `0x78` bytes and calls instance constructor
`0x1401da7d0`. The new instance's interface is `0x142511d60`, whose category
method is the same `0x1401da7b0`, returning `0x0b`. Thus the Tween receiver
reaches the callback's same-object return branch. This is an AS2 object
allocation; creation of a rendered display object or a frame queue is not
established by this path.

Public `constructor` controls this allocation selection. Hidden
`__constructor__` supplies the superclass callable for the subsequent
`super()` dispatch. These two native name slots must remain distinct.

## Function2 and the fresh Super wrapper

**VERIFIED (static control flow):** The Function2 creator stores its parsed
register count and flags in the native script function. For Function2 mode,
frame helper `0x140205090` preserves the incoming call's receiver; the
preload-super flag `0x10`, present in original flags `0x119`, constructs a
Super wrapper around that receiver. The prototype's virtual method `+0x70`,
`0x14019ad30`, obtains the function reference from hidden `__constructor__`.

Super constructor `0x1401da160` stores the receiver at object offset `0x78`
and the superclass function reference at `0x80`. Its interface is
`0x142511cb0`; virtual `+0x78` returns true, and virtual `+0x70` returns that
stored function reference. The raw `CallMethod` handler's undefined-name
branch uses these methods to invoke the superclass constructor through its
ordinary function invoke slot.

Native wrapper `0x14018c5b0` recognizes this Super interface and builds a
temporary call context using its stored actual receiver at interface offset
`+0x58` (object offset `0x78`). It preserves the result destination,
environment, argument count and argument index, then calls the callback.
Therefore the MovieClip callback receives the allocated Tween instance.

Afterward, wrapper cleanup `0x140189230` checks Super object offset `0x70`.
The fresh wrapper initializes this field to null, and this undefined-name
constructor path does not replace it, so cleanup returns without changing
wrapper or receiver state. The separate non-null branch restores only
wrapper bookkeeping. The callback itself has no argument-count or
constructor-mode requirement; the implementation scope here remains the
observed zero-argument, unchanged-parent, constructor-only Super context.

## The following prototype writes and flag selector

**VERIFIED (static instructions):** MovieClip.prototype constructor
`0x1401e5470` initializes its link field `+0x68` to null. Raw assignment
method `0x1401eb5e0` calls `0x1401ad440` before generic setter `0x1401e3b20`.
For this unbound object, the hook skips linked-object resolution and exits without
writes when a name does not start with ASCII `on`. Public `constructor` and
the subsequent `tween__*` property names take that path. Event-name writes
and attached display objects require separate evidence.

The own-property raw lookup and flag setter are the same methods already
proved for selected-name `ASSetPropFlags`; the receiver's actual native
prototype category is now bound to those methods. On existing own Tween
method entries, they preserve stored values and modify flags without invoking
getters or traversing parents.

**VERIFIED native path; derived exact input segmentation:** The literal
`tween__start,tween__to,tween__run` at the next call, movie offset 322,601,
produces these selectors in source order: `tween__start`, `tween__to`,
`tween__run`. String-tag conversion retains the same string node. Split
helper `0x1401c8f30` recognizes the comma delimiter, interns each exact
pointer/length slice, and appends the resulting string values in order.
The ASCII decoder consumes each byte unchanged; no whitespace conversion is
entered. This proves the exact nonempty ASCII selector with two internal
commas. Empty components, surrounding whitespace, non-ASCII text,
non-string coercion and allocation failure remain outside this result.

## Evidence and provenance

The target is the owner's private Win64 build 1.52.0.1700 executable,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

Callback REA record
`ev_c3e031f37a7c9e8d5a56ef3da74939f9d25c2b9c9c688ced4e6955e2c5f5debc`
is preserved in `evidence/rea-movieclip-constructor/callback-final.bundle.json`.
The private `callback-finding.json` records complete branches and limits.
The call completed with exit zero, no truncation and an unchanged target hash.
All 47 decoded instruction spans, including adjacent padding outside the
claimed body, matched the original PE bytes. Code-only REA decoding used raw
base `0x140000c00` from PE image base plus `.text` RVA minus file offset.
The session was exported/reset.

The joined receipt is
`evidence/rea-movieclip-constructor/combined-proof-receipt.json`, SHA-256
`059cc06b26888c1c67ab156b417d88587a8301e6c931a5cd83ad002225200253`.
It lists 30 new completed REA records across the following private bundles.
All calls exited zero without truncation or target changes; an independent
section-mapped byte check matched all 6,692 decoded instruction spans.
Earlier partial handler windows are retained but superseded by the complete
Function2 handler record. No claim relies on an incomplete instruction.

| Private evidence bundle or receipt | Proof component |
| --- | --- |
| `rea-movieclip-constructor/prototype-final.bundle.json` | Actual prototype constructors, setup call, public-constructor linkage, prototype assignment and the unbound property hook. Six records; byte checks in `raw-map-validation.json`. |
| `rea-movieclip-binding/final.bundle.json` and `finding-summary.json` | Global name/callable binding, Function2 creation and header flags, script invoke/frame preparation, hidden superclass reference, Super wrapper and undefined-name invocation. Ten records. |
| `rea-script-allocation/final.bundle.json` and `finding-summary.json` | Shared allocation helper, native allocator/category, Extends public-reference inheritance and script-function prototype raw assignment. Nine records. |
| `rea-script-allocation/movieclip-public-getter.bundle.json` | The actual MovieClip prototype's public constructor getter; one additional record. |
| `rea-asflags-comma-selector/final.bundle.json` and `finding-summary.json` | Exact ASCII string identity, comma splitting, ordered append and supplied substring boundaries. Three records. |

Paths in this table are relative to the private `evidence/` directory. Prior
Function, Extends and selected-name flag evidence is reused by the joined
receipt. The global object's `RSI` continuity between bounded REA windows was
also checked in existing private GNU objdump output, under the Win64
nonvolatile-register convention; it is a static relationship, not an observed
runtime trace.

The result is limited to successful zero-argument construction with the
original prototype/constructor links unchanged. It does not establish
arbitrary named-super calls, constructor replacement, primitive coercion,
allocation failure, every direct `MovieClip()` use, display attachment,
frame scheduling or gameplay parity. Native instance interface
`0x142511d60` and prototype interface `0x142514db0` share the category getter
but must not be assumed to share every write method.

| Source read | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned Win64 executable and existing version receipt | Proprietary Gazillion client; owner-authorized compatibility study | Exact bounded instructions, vtables, builtin names and target identity. No native execution or recovered implementation copied into source. |
| Existing private manager movie extraction and AVM1 lane's original-bytecode observations | Proprietary Gazillion assets; private study only | Actual zero-argument call, immediate result pop, offsets and body digest. |
| `docs/REA_FUNCTION_CALLS.md`, `function-global.bundle.json` and `function-binding.bundle.json` under private `evidence/rea-function-calls/` | Original project analysis and REA evidence; source ledgers retained there | Shared global setup, Object prototype parent and return conversion, including record `ev_ceb0b6b4241d0b1b4625bb26bb29866a83de58568e4eac93b099ca54166e5092`. External reference pages cited by that document were not independently reopened here. |
| `docs/REA_NATIVE_ALTERNATIVE.md`, prior Extends/selected-name receipts and the helper receipts listed above | Original project analysis; underlying client output remains private/proprietary | Reuse previously established property flags, raw Value handling, Extends metadata and exact per-component REA records. |
| `evidence/rea-asflags-followup-return/final.bundle.json` and previous native proof notes | Original project analysis and private REA evidence | Invocation wrapper and independent Push tag identities; record `ev_baa1bc0e27483ab8df8eee94de03d3795c651dc840a70de0902a005289939e72`. |
| Existing private `evidence/rea-class-registry/owner-init-14020ea40.objdump-candidate.txt`; GNU binutils objdump | GPL-3.0-or-later analysis tool; output is private owner-client study | Bounded address discovery and register continuity checks; not linked or shipped. |
| Python 3 standard library | PSF tooling | Original PE candidate scanners, digest receipts and byte-mapping checks; no implementation code copied. |
| Mandatory REA skill and connection/native guides, installed REA/Rizin tool schemas | Previously read installed REA guidance; REA 6.9.0 MIT, Rizin 0.9.1 LGPL-3.0-only tooling | Supported bounded static analysis and evidence preservation. No shared configuration/toolkit changes. |

All added prose is original analysis. Client bytes, decoded bytecode and native
disassembly remain private and are excluded from the repository and builds.
