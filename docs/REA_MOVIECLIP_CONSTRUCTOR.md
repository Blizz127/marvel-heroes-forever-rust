# Native MovieClip constructor callback

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

Surrounding function-object invocation and the earlier `NewMethod` allocation
remain distinct operations. In particular, native wrapper `0x14018c5b0` can
unwrap a special receiver before invoking a callback. The callback alone does
not prove the category of every script subclass instance, general direct
`MovieClip()` calls, `new MovieClip()` allocation, or arbitrary super-method
dispatch. Constructor-only support must retain its receiver and call-context
guards until the corresponding caller path is grounded.

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

| Source read | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned Win64 executable and existing version receipt | Proprietary Gazillion client; owner-authorized compatibility study | Exact bounded callback instructions and target identity. No native execution or recovered implementation copied into source. |
| Existing private manager movie extraction and AVM1 lane's original-bytecode observations | Proprietary Gazillion assets; private study only | Actual zero-argument call, immediate result pop, offsets and body digest. |
| `docs/REA_FUNCTION_CALLS.md` and private `evidence/rea-function-calls/function-binding.bundle.json` | Original project analysis and REA evidence; source ledgers retained there | Callback address handoff and existing return conversion, record `ev_ceb0b6b4241d0b1b4625bb26bb29866a83de58568e4eac93b099ca54166e5092`. External reference pages cited by that document were not independently reopened here. |
| `evidence/rea-asflags-followup-return/final.bundle.json` and previous native proof notes | Original project analysis and private REA evidence | Invocation wrapper and independent Push tag identities; record `ev_baa1bc0e27483ab8df8eee94de03d3795c651dc840a70de0902a005289939e72`. |
| Mandatory REA skill and connection/native guides, installed REA/Rizin tool schemas | Previously read installed REA guidance; REA 6.9.0 MIT, Rizin 0.9.1 LGPL-3.0-only tooling | Supported bounded static analysis and evidence preservation. No shared configuration/toolkit changes. |

All added prose is original analysis. Client bytes, decoded bytecode and native
disassembly remain private and are excluded from the repository and builds.
