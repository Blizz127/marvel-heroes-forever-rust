# Ordinary script calls from saved handlers

**VERIFIED, bounded static native evidence:** an ordinary ScriptFunction
selected onto the same Environment as a saved handler uses that Environment's
existing operand stack. It does not get a fresh private operand vector.
Raw type 1 also uses the same persistent four-register bank as saved modes
3/4. Raw type 2, Function2, appends exactly its serialized register count to
a separate Environment register vector and removes that extent on return.

The caller does not restore its pre-call operand depth. After the callee
returns, CallFunction and CallMethod drop fixed operand counts from the
**current** top and overwrite the remaining current top with the result.
Consequently a callee's residual stack change can remain observable. A private
per-function vector that is discarded at return is not generally equivalent.

This supplements [SAVED_EVENT_EXECUTION.md](SAVED_EVENT_EXECUTION.md), without
implementing VM calls or running original code. The supported profile is an
original type-1/type-2 function, a known same selected Environment, valid
operand/register references, successful allocation and normal return without
Try/Throw, pending exception cleanup or untracked reentrant state changes.
It does not establish every captured-target, activation or method-resolution
case. The private build `1.52.0.1700` image SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

## Selected Environment and entry bookkeeping

Complete invoker `0x14020e190..0x14020e307` obtains an existing Environment
through the ScriptFunction's virtual `+0x48`, applies its target/depth gates,
then invokes setup `0x140205090`, interpreter `0x140208ef0` and cleanup
`0x1401f2480` with that selected Environment. It supplies the function's
source wrapper, body start/length, captured scope vector and raw function type.
It creates no new Environment or operand-stack object.

The selected Environment can differ from FnCall's caller Environment. The
latter still supplies the argument positions used by setup. This study admits
only their proved identity for the particular call; matching a receiver name,
source movie or method spelling is not sufficient. Do not assume that every
function reached from a saved handler selects its display owner's Environment.

Unlike a saved handler's null activation fence, ordinary types 1/2 allocate
an activation and bind the applicable parameters/locals. They push the executing
function and retain required references. Setup copies argument Values from
FnCall's caller stack into activation members, arguments storage or registers;
those copies do not remove the argument operands from the stack.
An ordinary activation can retain an explicit captured frame; the saved
caller's null fence does not erase that lexical capture. Normal cleanup restores
the prior activation depth and exposes the same saved fence. Escaped activation
lifetime remains reference-driven.

No direct operand cursor/depth checkpoint is saved by the selected invocation
bookkeeping, ordinary setup branches or normal cleanup. The interpreter's
entry retains the selected Environment, current target, scope state and scratch
field `+0x138`; its normal epilogue restores target/cached version/scratch and
execution bookkeeping. Neither creates, clears or restores a private operand
stack. Helpers invoked while creating locals/preloads can have their own
effects; this observation does not authorize arbitrary getter/reentrancy cases.

## Call operands, Return and caller cleanup

The native short-opcode table binds CallFunction `0x3d` to `0x14020a596`,
Return `0x3e` to `0x14020a94c`, and CallMethod `0x52` to `0x14020b6ef`.
For the supported ordinary-call branches, the caller retains its arguments,
argument-count operand and name/receiver operands on the shared Environment
stack during invocation. The native FnCall records the appropriate argument
index from that stack; setup reads those argument Values by index.
Both caller branches initialize their separate result destination to Undefined
before invocation.

On normal return:

| Caller opcode | Native post-call operation |
| --- | --- |
| CallFunction | `0x14020a89d` drops `argc + 1`; `0x14020a8a8` writes the result into the remaining current top. |
| CallMethod | `0x14020becb` drops `argc + 2`; `0x14020bedb` writes the result into the remaining current top. |

The complete drop helper `0x140193910` repeatedly releases the current Value,
decrements the current stack pointer by one 32-byte slot and applies its page
transition helper when needed. It receives a count, not a saved pre-call depth.
The result write uses Value assignment, not a new independent stack push.

Return copies the current top to the supplied result destination when present,
then pops one operand and selects the source-end continuation. The normal
continuation advances the program counter and reaches the interpreter
epilogue. Return does not truncate the stack to a function-entry boundary.
Source fallthrough/ActionEnd likewise supplies no such boundary on this profile.

**INFERRED consequence:** when a well-formed callee pushes its return Value
and Return pops it, its net stack change can be zero and normal call cleanup
leaves one result above the caller's older operands. If a callee leaves another
operand, removes an existing operand or otherwise has a nonzero net change,
the caller's fixed-count cleanup acts on that changed stack. It can overwrite
a different remaining slot. A VM must not silently discard that difference
by dropping a private callee vector.
With a valid caller prefix of depth `P` and callee net change `Delta` (including
Return's pop), both opcodes finish at `P + 1 + Delta`, provided no physical
bottom clamp or exceptional cleanup intervenes. This does not mean that the
callee's leftover Values are preserved verbatim above an otherwise cleaned
caller frame.

This does not require unsafe Rust indexing. Unsupported underflow, invalid
argument counts, unknown previous state or unmodeled native bottom/page behavior
can remain explicit errors. Such restrictions are admission limits, not claims
that the original establishes a private frame boundary.

There is a separate pending-block cleanup route before the common epilogue.
The interpreter initializes its local cleanup counter to zero; raw opcode
`0x8f` increments it. A nonzero cleanup branch can compare the current stack
index with a retained block depth and drop excess operands. It is not an
ordinary function-entry checkpoint. Full Try/Throw/catch/finally and pending
exception handling remain excluded from this normal-call profile.

## Type 1 shares the fixed register bank

Raw type 1 does not enter Function2 register setup or cleanup. Like saved
modes 3/4, its register opcodes select the fixed Environment Values at
`+0x48`, `+0x68`, `+0x88` and `+0xa8`. These are initialized once by the
Environment constructor. Ordinary type-1 entry does not reset them to
Undefined or save a private copy; normal return does not restore their old
contents. Writes by the callee therefore remain visible to the saved caller.

For this fixed-bank branch, valid indices are 0–3. The previously recovered
native opcodes ignore an out-of-range StoreRegister and push Undefined for an
out-of-range Push-register. A first implementation can retain an explicit
unsupported boundary for these cases, but must not claim its error is the
native behavior. Named parameter binding is independent of these register
Values.

## Function2 appends exactly N registers

The original DefineFunction2 producer copies the serialized register-count
byte unchanged into function `+0xcb`. Setup passes that exact byte to
`0x1401c3160`; there is no added register. Let `M` be the existing Environment
register-vector count and `N` that byte:

```text
before call:     count M
during call:    count T = M + N
new local area: physical slots M .. M + N - 1, initially Undefined
after return:  count M
```

The vector is rooted at Env `+0xc8`, with count at `+0xd0`. Cleanup
`0x1401c31d0` subtracts exactly `N`; removed Values requiring cleanup are
released. The fixed four-register bank is not used as the normal local storage
for valid Function2 references.

Complete register helper `0x14017e8d0..0x14017e91d` and the inlined setup
references use this native rule:

```text
if unsigned index < total count T:
    physical slot = T - index - 1
else:
    diagnose and use fixed register 0 at Env + 0x48
```

Thus indices `0 <= i < N` address only the new local extent. With nested
Function2 storage, `N <= i < T` reaches an older extent; the native check is
against the accumulated count, not the current function's declared count.
Indices at least `T` fall back to fixed register 0. Granting an extra local
slot by allocating `N + 1` would change these relationships.

The first faithful isolated profile should require every executed register
index, nonzero parameter-register target and enabled preload target to be
strictly below `N`. It can reject references outside that profile instead of
guessing how to emulate access to an outer extent or fixed-register fallback.
A serialized `N = 0` supplies no isolated local register.

Parameter writes occur **before** preloads. A parameter descriptor with
register number zero creates a named activation binding; it does not write
logical register zero. A supplied argument with a nonzero parameter register
is copied into that register. A missing named parameter gets Undefined;
a missing register parameter skips the write and stays Undefined when it
addresses the freshly initialized local extent.

Enabled preloads are assigned consecutively starting at logical register 1:
`this`, `arguments`, `super`, root, parent, then global, in that order. They can
overwrite parameter Values assigned to the same logical register. Their flag
bits are respectively `0x001`, `0x004`, `0x010`, `0x040`, `0x080` and
`0x100`. Suppression bits `0x002`, `0x008` and `0x020` govern the separate named
`this`, `arguments` and `super` bindings. This receipt grounds storage/index
behavior; it does not broaden all receiver, parent/root lookup or accessor
semantics used to produce those preload Values.

The resize helper establishes count changes, new-slot initialization and
removed-tail cleanup. Capacity changes delegate to the heap's virtual
reallocation interface; this study does not recover that concrete allocator's
prefix-preservation implementation. Entry with `M = 0` has no outer register
prefix to preserve. Nested cases that depend on an unproved allocator path
must keep their scope explicit rather than turning this limitation into a
claim of complete nested-register parity.

## Integration boundary

**INFERRED design requirement:** the selected Environment should own the
operand stack and fixed bank. Ordinary function frames carry control flow,
activation, scope and return destination while their operand operations refer
to that selected shared state. A Function2 call additionally records its
declared register extent, initializes it and removes exactly that extent on
normal return. The caller's native cleanup applies to the resulting current
operand stack.

A dedicated saved-event entrypoint alone is insufficient if its nested
ordinary calls still allocate private operand vectors or reset type-1
registers. Until integration implements the supported shared-state contract,
ordinary script calls from saved execution should remain explicitly gated.
Direct native callbacks, different selected Environments, Super/captured-frame
variants, arbitrary recursive getters, exception unwinding and full native
allocation failure behavior require their own admission contracts.

No VM code or original bytecode was run for this investigation. Source/register
audits and synthetic Rust checks can validate later integration, but neither
alone proves native execution parity.

## Evidence and provenance

Evidence stays under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-saved-script-calls/parent-check.json` binds the retained complete
  invoker, selected Environment helper, interpreter entry/normal epilogue and
  new bounded continuation/cleanup evidence. It matches 442 instruction spans
  plus eight dispatch bytes to the private image.
- `rea-saved-script-calls/final.bundle.json` contains new Evidence
  `ev_f164812d1eeb3af351b44d0fa6b5c48809bf52186ad86d25f70548a5a3a1ef2e`
  and `ev_6e66ca77218f4c3817834a829a3d61f25f62a4295c2347f973b8f60de9896b99`.
  Exact corrected 17-byte ActionEnd and 12-byte cleanup-count selections
  supersede adjacent partial tails from the first selection; those tails
  supply no claim.
- `rea-saved-script-registers/register-index-contract.json`, SHA-256
  `fde9c4b4b4138a98dfdae12628d124510b7a32780533e6d2a15ba8dc4a2be6c2`,
  binds serialized `N`, logical-to-physical indexing and preload/parameter order.
- `rea-saved-script-registers/finding-summary.json` and
  `rea-saved-script-callers/finding-summary.json` bind the full component
  selections, cleanup helpers and opcode-table joins.
  Their SHA-256 values are respectively
  `12e726847eb957ab8de9a0065388198a2b3d4a216f9b76ee452d1de9fe4147d9`
  and `8e04413807b8197987c112bbc77ddb5f5373cd86ac15e7a1b75ed480133d07a6`.
- `rea-saved-script-calls/combined-proof-receipt.json` binds those immutable
  receipts and the original verification scripts; SHA-256
  `d308e57e2b1844022e7d9b05418ffa592d963191097696713a5138013dcb1203`.

Seven new REA records are complete, exit 0, untruncated and bound to an
unchanged target hash. Component checks cover 442 parent, 2,544 register/frame
and 884 caller/Return instruction spans, with overlap between retained
selections; those counts are not a unique combined total. The three scoped
investigations exported and reset their ledgers without an active target.

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Owner-private `MarvelHeroesOmega.exe`, build 1.52.0.1700 | Proprietary; compatibility/interoperability study | Bounded native instruction and dispatch-table evidence |
| Previously read REA skill and connection/native guides; REA 6.9.0; Rizin 0.9.1 | MIT documentation/toolkit; LGPL-3.0-only disassembler | Supported static Evidence workflow; not linked or shipped |
| GNU objdump; Python standard library | GPL-3.0 tooling; PSF | Candidate navigation and original byte/receipt checks |
| Original project saved-event, MovieClip-binding and event-dispatch documents/retained receipts named above | Original project work; native outputs remain private | Reuse complete function setup/cleanup, source and register-opcode observations |

No native implementation or proprietary payload is added to git. The study
changes documentation only and does not claim that the current VM already
implements these contracts.
