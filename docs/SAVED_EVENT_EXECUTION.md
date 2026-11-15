# Saved Construct/onLoad execution state

**VERIFIED, static native evidence:** saved clip-action invocation is not the
fresh top-level frame currently created by `Vm::execute`. Saved handlers use
the display owner's Environment, its existing operand stack and its persistent
four-register bank. They push a null activation entry, copy an empty captured
scope vector and reuse a constant pool owned by the installed source wrapper.
Raw modes 3 and 4 also have different target-state gates. Running the original
bytes through `execute` remains a useful diagnostic, not proof of native event
execution.

This study covers the previously identified concrete AS2 display owner,
zero-argument saved Construct (`0x40000`) and onLoad (`1`) delivery, original
tag-8 ScriptFunctions installed by `0x140200570`, successful allocations and
normal returns. Dispatch order, onLoad eligibility, the separate dynamic
event-member call and retained constructor records remain governed by
[FRONTEND_EVENT_DISPATCH.md](FRONTEND_EVENT_DISPATCH.md). This document adds
no VM code, event firing or native client execution.

## Source and mode are properties of the installed callable

The private build `1.52.0.1700` executable has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All native addresses below refer to that pinned Win64 image.

Installer `0x140200570` creates a fresh source wrapper and ScriptFunction for
an admitted saved action. The wrapper retains the original source object;
the function retains the wrapper, start 0 and the original source length. It
passes an **explicit null captured-scope argument** to `0x1401e14c0`.
The resulting function has empty scope and formal-argument vectors, zero
Function2 flags/register count and null auxiliary frame reference. Modes 3/4
also skip captured MovieImpl/target creation, leaving function `+0x68/+0x70`
zero. Mode alone would not prove that the scope vector is empty; the installer
argument closes that dependency.

The raw mode is 4 only when the original aggregate mask equals `1`, `4`,
`0x200` or `0x40000`; otherwise it is 3. Splitting a combined mask into separate
map keys happens afterward and copies the **same callable** into those keys.
Thus a pure Construct or onLoad record uses mode 4, while a combined mask
containing either can use mode 3 and share source/pool state across its keys.
Do not derive the mode from the currently delivered single-bit event alone.
These are native mask values, not a general serialized SWF-mask conversion.

The saved dispatcher supplies owner `+0x08` as `this`, owner Env as caller,
zero arguments, null result destination and null method-name hint to
`0x14019aed0`. The helper preserves the function/frame reference and builds
the native FnCall. It creates no operand stack or new Environment. The
ScriptFunction's uncaptured environment selector falls back to that caller
Env; the original source remains independent of the receiver and environment.

## Activation, scope, `this` and locals

Invoker `0x14020e190` calls setup `0x140205090` before bytecode entry
`0x140208ef0`, then cleanup `0x1401f2480` on normal return. For the selected
zero-formal/zero-argument modes 3/4 path:

- Setup saves the old activation-vector depth and appends a **null activation
  slot**. Modes 1/2 allocate an activation object; these saved modes do not.
- The current function is pushed on the Environment function stack. No
  synthetic `this`, `arguments` or `super` local is installed in a new object.
- The zero-parameter path skips Function2 register allocation/preload logic.
- Cleanup removes the function-stack entry and restores the old activation
  count through `0x140193b50`. Its `this`/`super` cleanup blocks are skipped
  because the top activation is null. Mode-2 register cleanup is also skipped.

The null slot is a scope fence. Retained helper `0x14018f070` examines the
top activation; null returns no activation and does not continue into an
outer activation below it. It must not be modeled by simply leaving an outer
script activation visible.

The interpreter receives `&function.scope_vector` as its sixth argument.
`0x1401882a0` allocates an empty local vector and copies through
`0x140181320`. The supplied pointer is nonnull even though its data/count are
zero; the latter helper's count-zero branch performs no element copy. This is
an empty captured/With-scope vector, not an absence of target/global lookup.

For the supported version-above-6, unchanged-target profile, retained
`0x1401faf10` establishes the effective variable reads:

1. The empty captured/With vector supplies no candidate.
2. The null activation supplies no local and blocks outer activations.
3. Reserved `this` returns the **current Env target** through display-Value
   conversion `0x14018ddf0`. It is not a newly stored local property.
4. Ordinary names use the target's actual raw member getter, then the current
   context's global object's normal getter if the target lookup misses.

Synthetic `arguments`/`super` lookup requires a nonnull activation and is
skipped here. These names can still encounter whatever later lookup branches
their supported native profile permits; no empty arguments Array is invented.
The ordinary target getter retains its real storage/child/prototype ordering.
An own found Undefined remains different from a lookup miss. Existing special
identifier, path, accessor and native member restrictions still apply.

**VERIFIED local-opcode distinction:**

| Opcode | Native saved modes 3/4 |
| --- | --- |
| DefineLocal `0x3c` | Converts the name, then uses ordinary variable assignment helper `0x140203170`. For a simple name with empty scope and null activation, `0x140194f20` calls the current target's normal SetMember interface. It does not create an activation-local binding or bypass the attached object's setter. |
| DefineLocal2 `0x41` | Converts the name and pops it, but skips the declaration helper. No property is created or overwritten by this opcode in these modes. Name conversion effects are not skipped. |

Mode 1/2 branches take the separate local-declaration helpers. A Rust frame
with `locals=target` does not capture the distinctions above, especially
DefineLocal2 and the attached target's normal write semantics. This study does
not generalize the simple-name proof to paths, With scopes, accessors or all
SetVariable cases.

The interpreter saves/restores the current target and cached target version on
normal exit. Therefore initial `this=current target` agrees with the supplied
owner in this profile, but arbitrary SetTarget behavior cannot be replaced
with an immutable initial receiver. A first bounded API can reject unsupported
target changes. Native Env `+0x138` is also saved/restored; its public meaning
was not established here and is not assigned an invented Rust semantic role.

## Registers and operand stack belong to the Environment

The native Environment constructor initializes the four fixed register Value
tags to Undefined **once**, at Env `+0x48`, `+0x68`, `+0x88` and `+0xa8`.
The saved-handler setup, interpreter prologue and normal cleanup do not clear,
allocate, save or restore this bank.

For modes 3/4, StoreRegister (`0x87`, native branch `0x14020c5e8`) writes the
current operand-stack top into `Env+0x48+32*index` for indices 0–3. An index
above 3 is ignored on this branch. Push-register (`0x96` literal tag 4,
`0x14020c859`) copies those same persistent entries; an index above 3 pushes
Undefined. Mode 2 uses its separate register helper instead. An implementation
may initially gate unsupported indices, but must not claim that its error is
the native invalid-index behavior.

The actual operand stack is the selected Environment's stack throughout
execution. Invocation helper `0x14019aed0`, setup, interpreter entry/normal
epilogue and cleanup establish no fresh-empty stack or saved stack-depth
boundary for this selected path. They also do not discard residual operands
when a handler ends. Existing values and later handler values therefore cannot
be modeled by resetting a new vector at each delivery. Bytecode operations
still perform their own pushes/pops and called functions have their own
established contracts.

A proof of a newly created Env can seed Undefined registers and its initial
stack. Installing another saved action into an existing owner does not create
another Env. Nor does successful execution of an earlier handler prove that
the bank is Undefined or that its stack is empty afterward. Reentrant use,
unknown prior executions and untracked state require explicit admission or
an unsupported boundary.

## Constant pools belong to the retained source wrapper

Wrapper constructor `0x1401850a0` initializes pool data/count/capacity
`+0x18/+0x20/+0x28` to zero and marker DWORD `+0x38` to -1. This occurs once
per installation, not once per call. No caller pool is copied. Invoker
`0x14020e190` passes this same retained wrapper into each execution; the
interpreter prologue and epilogue do not reset its pool.

Complete ActionConstantPool helper `0x140186800` implements the following:

- If the current opcode offset equals stored `+0x38`, return without parsing.
- If `+0x38` is -1, record the current offset and populate the retained pool
  from that ConstantPool record, interning strings in the supplied context.
- If a different pool offset was already recorded, optionally diagnose and
  return without replacing the pool.

This is the **first executed** pool record, not necessarily the first one in
file order. The helper has no mode argument or mode branch. Push-constant8/16
reads wrapper count `+0x20` and data `+0x18` directly; out-of-range indices push
Undefined. The interpreter does not use a freshly empty per-call pool.

The current Rust opcode implementation instead replaces `frame.pool` whenever
it executes a ConstantPool and errors on an invalid index. Those semantics
must not be silently reused for saved events. Immutable source with a single
pool dominating all uses is a possible narrower profile, but its dominance
and reuse conditions need actual source validation. Do not assume pool state
is keyed only by CodeId: separate installations allocate separate wrappers,
while mask splitting retains one wrapper for all keys of that installation.
Native offsets are relative to the retained action source; file-absolute Rust
spans require an explicit consistent mapping.

## Mode gates and normal-return scope

At bytecode entry, modes 0/3 reject target word `+0x6a` bit `0x1000` or signed
target DWORD `+0x2c < -1`; mode 4 skips those two checks. Both modes reject
target `+0x6a` bit 4. Complete helper `0x14017b840` implements the same raw-mode
test at the interpreter's documented call sites. No public name is inferred
for the additional fields. This is distinct from the outer onLoad eligibility
bits and event descriptor admission described in FRONTEND_EVENT_DISPATCH.

The ScriptFunction invoker also applies its selected-target fallback and
per-Env depth limit before the body. Its normal-return cleanup restores the
activation depth, function stack and execution bookkeeping, but preserves the
fixed register bank and pool. The bytecode entry restores its saved target,
cached target version and scratch field noted above. The shared operand stack
is not replaced by this restoration.

The saved call's result destination is null. The dispatch helper's Boolean
indicates a callable invocation path, not body completion; depth/target gates
can skip the body. Native exception flags, unsupported coercions, allocation
failure, native unwinding, target retirement during calls and reentrant handler
map mutation are not established as successful execution. A Rust executor
must surface its own unsupported/error result, not translate a visited record
or native-looking helper return into successful delivery.

## Minimal honest VM interface

**INFERRED design from the verified contracts:** keep the existing generic
`execute` diagnostic and add an explicit saved-action mode/state seam. Names
below describe required ownership, not an implemented public API.

- `SavedActionState`: immutable original source/span identity, original
  aggregate mask, derived raw mode, empty-capture metadata, first-pool-offset
  marker and persistent pool. Split delivery keys share this state; distinct
  installations do not silently share it.
- `SavedEnvironmentState`: stable owner/current target and context identity,
  persistent operand stack and four fixed registers, with known initialization
  or tracked prior execution. Invocation pushes/pops the null activation fence
  and function entry, preserving relevant outer bookkeeping.
- `execute_saved_action(handle, environment, host)`: validates source/mode and
  caller context, uses the actual typed target/getter/setter paths, enforces
  the saved-mode local opcodes and pool rules, and reports completion, a known
  skip or an explicit error separately. Dynamic `onConstruct`/`onLoad` member
  invocation remains a subsequent, independently resolved callable call.

A first implementation may admit only pure Construct/onLoad mode 4, a known
live concrete owner, version above 6, no target changes or With, well-formed
retained bytes, zero arguments and no reentrant dispatch. Such restrictions
must be explicit. An empty stack/Undefined bank may be required only with a
real initialization/state receipt, not manufactured for each handler. The API
must also retain source and current Environment separately; this study does
not select a source-origin Environment for these uncaptured functions.

The observed `Vm::execute` currently resets execution counters/context, creates
an empty operand vector, four Undefined registers, `[target, global]` scopes,
an empty pool, visible typed-target `this` and `locals=target`. Its source hash
and these observations are retained privately. Some supported ordinary reads
can agree, but the state and opcode differences above prevent a blanket
equivalence claim. No normal DoAction/initializer-mode equivalence is inferred
from the saved-event modes studied here.

## Evidence and provenance

Evidence stays under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-saved-event-execution/combined-proof-receipt.json`, SHA-256
  `605e6cb215c169ad5bb6ab1773644591ee78e7ae6cfeb3d6fb1e87b8db92bc96`:
  binds all component receipts and independently checks eight new REA records,
  1,266 decoded spans and 20 dispatch-table bytes against the pinned PE.
- `rea-saved-event-execution/semantics.bundle.json`: two new parent REA records,
  including prologue, mode gates, selected local/register/pool branches,
  normal epilogue and complete ordinary-name dispatch `0x140203170`.
- `rea-saved-event-frame/finding-summary.json`: setup/cleanup, activation fence,
  variable lookup, simple-name normal setter and Environment register lifetime.
  SHA-256 `53d3e7d1d344cf8eaf5e873dfbee2d738ec2981716006f735293a4449296c0e5`.
- `rea-saved-event-source/finding-summary.json`: installation, retained source,
  original mode, empty metadata/scope and complete empty-scope-copy helpers.
  SHA-256 `b0fcf67e3e11db3e912d775b2e95a15f9317ea90f74cbfc25efd3bada8762c46`.
- `rea-event-dispatch/complete.bundle.json`, record
  `ev_6dfab21ba5f9d8c474ad1c640ca8ade9679527eb868f4bb9b52c0fcd4fe3e63d`:
  retained complete `0x14019aed0` and exact zero-argument caller profile;
  companion `ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54`
  binds source installation and ScriptFunction invocation.
- `rea-enumeration-callback/constructor-and-pool-final.bundle.json`, record
  `ev_aab6a42276a45efe8236d3bdbf7e1df13827f7e052fed26d812cc12c9271dccf`:
  retained complete ConstantPool helper and its opcode binding.
- `rea-saved-event-execution/current-vm-observed.json`: original Rust entrypoint
  source hashes and diagnostic-frame comparison. Candidate objdump listings
  remain private; positive claims above use retained or new REA observations.

New observations completed with exit 0, untruncated output and unchanged target
digest. Decoded bytes are checked against PE section mapping. Initial partial
unwind-fragment reads were superseded by complete helper reads; trailing padding
and partial neighboring instructions supply no semantic claims. All study
sessions were exported/reset. No original client execution, VM implementation,
runtime event smoke or retail comparison was performed.

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned executable above | Proprietary owner-installed artifact, private interoperability study | Selected invocation, interpreter, constructors, lookup and state-lifetime evidence |
| REA skill and native/connection guides under `~/.agents/skills/reverse-engineer-anything/` | REA MIT documentation | Supported target-free evidence workflow |
| REA 6.9.0 / Rizin 0.9.1 | MIT / LGPL-3.0-only tooling | Selected native observations and exported receipts |
| GNU objdump / Python standard library | GPL-3.0 tooling / PSF | Bounded candidate disassembly and original byte/hash validation |
| Original project `src/avm1/vm.rs`, FRONTEND_EVENT_DISPATCH and existing qualified-variable, MovieClip, Environment, property and source receipts listed in component ledgers | Original project analysis/code; underlying native artifacts remain proprietary | Compare current diagnostic API and reuse known native contracts |

No external VM or SDK implementation was read or copied. Binary bytes, native
tables and disassembly remain private; only this original research document
is committed.
