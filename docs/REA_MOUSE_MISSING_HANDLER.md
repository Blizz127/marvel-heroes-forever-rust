# Missing Mouse handler diagnostic route

**VERIFIED static contract:** when native `Mouse.hide()` actually reads a null
MovieImpl state-4 handler, it enters diagnostic helper `0x140186d10`. That
helper can return without output or call a selected logging sink. Its normal
return is not an AS2 exception or a cursor operation. The earlier Undefined
result write is retained by the callback contract in
[REA_MOUSE_HOST.md](REA_MOUSE_HOST.md).

This does not establish that the current movie has no handler. The Rust
default-rejecting host remains unchanged. Unknown handler state, known missing
handler state, selected diagnostic context and selected logging sink are
different inputs; completing a diagnostic helper does not supply those inputs.

No original client, script, service, window or operating-system input was run.
All observations below are from the owner's pinned executable and retained REA
records. Normal external callback, allocation and import behavior is explicit.

## Diagnostic context and ordered calls

Let E be the invocation Environment, D its optional pointer at `+0x138`, and
L the pointer at `D+8` when D exists. Complete helper
`0x140186d10..0x140186e91` selects these paths:

| Actual input | Selected behavior |
| --- | --- |
| D nonnull, L null | Return through the normal epilogue. No target fallback or sink call occurs. |
| D nonnull, L nonnull, byte `D+0x12` zero | Call L virtual `+8` with a pointer to code `0x24000`, the original message and the native variadic-argument pointer. |
| D nonnull, L nonnull, byte `D+0x12` nonzero | Prepare the formatter described below, then call the same sink interface. |
| D null | Obtain the target's logging sink through virtual `+0x258`; if that first result is nonnull, obtain it a second time and call the second result's virtual `+8`. |

The last path has two actual getter calls, not one cached expression. Only the
first return is null-checked. The second result therefore requires the admitted
valid-lifetime/stable-selection contract before dereferencing it. Coalescing the
calls would discard the getter's reference and cache effects.

The missing-handler message is the existing fixed ASCII literal at
`0x142510c80`. It has no percent conversion and fits the selected native stack
formatting buffers. The variadic argument storage is nevertheless passed as
native code does; its unused words are not asserted initialized.

For the prefixed path, the helper computes the message length, removes one
terminal newline if present, initializes a formatter with a 256-byte backing
buffer, sets format `{0} : {1}\n`, and supplies the message view plus
`D+0x18` to separate formatting helpers. Calls are `082b60`, `082dd0`,
`184cd0`, `0ace40`, `081dc0` and `082820`, followed by the sink call.
Those formatter internals, argument binding, capacity behavior and cleanup are
not expanded here. This path must not be replaced with guessed concatenation.

The fresh Environment constructor's zero at `E+0x138`, documented in
[FRESH_AVM1_ENVIRONMENT.md](FRESH_AVM1_ENVIRONMENT.md), remains a constructor
checkpoint. It does not establish the value during this invocation. The
interpreter's retained save/restore of that field likewise does not prove
its value through nested calls or callbacks.

## Target getter and MovieImpl log cache

The selected source target table `0x1425069d0`, slot `+0x258`, points to
complete leaf `0x14009ae70..0x14009ae9d`. It walks the target and its
`+0x38` parents until low-byte `+0x6a` bit7 is set, then follows that
target's `+0x20` root and root `+0x10` movie M. It tail-calls `09a690(M)`.
The exhausted/null-chain branch passes null to that helper, whose first
field access requires nonnull M. No graceful null-chain behavior is invented;
the supported route requires the valid selected target/root/movie chain.

Complete `09a690..09a711` uses M's dword `+0x4310` bit1:

- If set, return the cached pointer `M+0x220`.
- If clear, call `052d50(M+0x10, localOutput)`, retain its resulting log
  pointer when nonnull, release the previous nonnull `M+0x220`, publish the new
  pointer there, then release the temporary reference and return the cache.

This body does not set bit1. Both diagnostic getter calls can therefore enter
the refresh path. `M+0x220` is the log cache, distinct from the state-4 handler
at `M+0x228`. Replacing the old cached log can reach virtual destruction;
successful reference-count/lifetime conditions or a concrete destructor proof
are required when claiming preservation of surrounding state.

Complete `052d50..052dac` requests ID2 from the supplied `M+0x10` interface's
virtual `+0x18`. The returned state must be nonnull on this route; its `+0x20`
is the log pointer. If that pointer is null, the helper uses `047e20`, whose
complete eight-byte body reads the global pointer at `0x142c950d0`.
It retains a selected nonnull log, writes the output pointer and releases the
state. This establishes the requested ID and field selection, not an actual
loaded interface/table or a current singleton value. The state-bag identity
and current cached state remain explicit profile inputs.

## Concrete engine logging sink

The complete retained engine setup `1412706b0` allocates a 16-byte log object,
initializes its count to one and installs table `0x1421c3358`. It passes that
object to `052eb0`, which constructs and registers a state-2 wrapper using
`052db0`; the wrapper stores the actual log pointer at `+0x20` and retains it.
This gives a concrete intended log implementation under successful allocation,
registration and lifetime conditions. It is not a state-4 implementation or a
current-cache observation.

That log table's `+8` is complete `0x1412785b0..0x14127869a`. The selected
body does not use the incoming `0x24000` code. It:

1. Formats the supplied bytes/arguments through `MSVCR120.dll!_vsnprintf_s`
   into a 1,024-byte stack buffer, with the native truncation request and
   explicit final-byte handling on result −1.
2. Builds a temporary engine string through complete `6e2730`.
3. Calls `77f590` using the output-device pointer stored at global
   `0x14292a018`, numeric category `0xd0`, a UTF-16 `%s` format and the
   temporary string.
4. Frees the temporary string through the selected engine allocator route,
   then returns through its normal epilogue.

The code does not establish the human-readable name of category `0xd0`.
Neither an output-device identity nor an actual console/file message was
observed. Unused stack-buffer bytes are not claimed to be zeroed.

Complete `6e2730` counts the nonempty byte string including its terminator,
allocates twice that count, invokes complete conversion helper `6e1800`, and
copies the resulting words. `6e1800` uses imported `lstrlenA` and
`MultiByteToWideChar` with code page **0** and flags **0**; this is not an
inferred UTF-8 conversion. For at most 128 bytes including the terminator,
its converter buffer is inline. The fixed missing-handler message fits that
branch. The conversion status is not checked before subsequent use, so valid
inputs, successful conversion/allocation and ordinary import behavior remain
conditions. Arbitrary code pages, conversion failure and allocator effects
are not emulated here.

Complete `77f590..77f707` first checks output-device `+8`. When nonzero it
calls `785f50(category)` and can return without output. Otherwise it formats
into a 256-word stack buffer through `7d2a80`, whose complete 13-byte leaf
dereferences the format argument and tail-calls `_vsnwprintf_s`. Large or
failed formatting takes a reallocating retry loop; successful bounded output
is terminated and passed to output-device virtual `+8` with category `0xd0`.
The filter, output-device callback and engine allocation/free bodies remain
external boundaries in this contract. Normal returns through them are not
proof of their transitive effects on arbitrary native state.

## Consequences for the Rust host

A host with coherently established missing state-4 handler can follow the
selected diagnostic route and return normally when the required logger profile
and callback behavior are supported. No OS cursor action follows from this
branch. A host with unknown handler state must continue reporting that missing
binding; it cannot select this branch merely to advance the original frame.

This study does not change the Rust Host implementation or rerun the frame.
It closes the diagnostic dispatcher, target/log selection and concrete engine
log entry while leaving exact current identities, prefixed formatting and final
output-device effects explicit.

## Evidence and provenance

Private root: `evidence/mouse-missing-handler/`. `verification.json` SHA-256:
`6504574a80354113a5f2744c299e2d1834f14104fb6ff7a8653adfa26c04b6c0`.
The original validator matches 614 distinct instruction rows / 2,429 bytes,
thirteen selected complete bodies, exact literals/table slots and six selected
PE import identities to the pinned executable. Counts include reused loader
and state-wrapper bodies, not just new observations.

Seven new target-free REA records completed with exit zero, untruncated output
and unchanged executable hash. `final.bundle.json` retains them; `reset.json`
records zero retained target/records afterward. Initial 12-instruction getter
and 64-byte output-routine captures were prefixes. Later complete captures
close their tails while preserving the original bounded observations.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's Win64 `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner installation; private compatibility study | Exact diagnostic, getter, log sink, conversion and output bodies |
| Retained `frontend-image-context/complete.bundle.json`, `mouse-host/installation/complete.bundle.json`, `scale9-rendering/rea-pe-imports.json`; cited project docs and saved interpreter navigation | Original project evidence from that owned artifact | Reuse exact constructor, wrapper, import and Environment facts; no runtime inference from stored globals |
| REA 6.9.0 / Rizin, installed REA skill/native guidance; GNU objdump | Previously registered MIT / LGPL-3.0 / GPL tooling | Bounded static reads and navigation, canonical export/reset |
| Python standard library | PSF tooling | Original PE mapping, instruction, literal, import and source-hash validation |

No outside game/SDK implementation was copied. Raw native evidence, executable,
data extracts and captures remain private and are excluded from git/builds.
