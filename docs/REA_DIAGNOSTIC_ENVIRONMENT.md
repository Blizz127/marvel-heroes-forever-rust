# Diagnostic Environment context: producer and preservation checkpoints

**VERIFIED static contract:** the selected AS2 interpreter saves the incoming
Environment `E+0x138` pointer after constructing a separate stack-local diagnostic
record. The local virtual getter returns a movie-policy byte; it does **not**
install that record into `E+0x138`. Two explicit exit paths restore the saved
pointer. Neither the selected frame nor `Mouse.hide` supplies a new context by a
direct field assignment in the checked paths.

The fresh Environment constructor establishes null at its own checkpoint.
That does not establish null at the selected callback, an adopted scene, or
the return from arbitrary callbacks. The proof below distinguishes those points.
No original client, service, native input or window operation was executed.
The diagnostic formatter's `0x140186d10` body was deliberately left to its
separate study.

## Field identity and fresh state

[The missing-handler consumer](REA_MOUSE_MISSING_HANDLER.md) supplies the
supported meaning: `E+0x138` is an optional diagnostic-context pointer `D`.
The consumer uses `D+0x08` as a selected log pointer, byte `D+0x12` to choose
raw versus context-prefixed formatting, and `D+0x18` as a C-string pointer for
the context format. A null `D` selects the target-log route. No public SDK
structure name or current context value follows from that consumer evidence.

The complete fresh constructor `0x1401c2fe0..0x1401c30a9` stores zero to
`E+0x138` at **0x1401c304f**. Its successful ordinary allocation conditions
remain those in [FRESH_AVM1_ENVIRONMENT.md](FRESH_AVM1_ENVIRONMENT.md).
This gives the earlier “scratch field” label a supported consumer role; it
does not extend the constructor's lifetime checkpoint past owner setup,
publication, action execution or callbacks.

The complete ScriptFunction invoker `0x14020e190..0x14020e307` selects an
**existing** Environment through virtual `+0x48`, may substitute the calling
Environment under its target-state condition, then calls setup `0x140205090`,
interpreter `0x140208ef0` and cleanup `0x1401f2480`. Its checked body does not
directly assign `E+0x138`. The selected provider
`0x14017f710..0x14017f7d2` returns the target-derived Environment or the supplied
caller's Environment. Selection is not fresh construction; its virtual calls,
reference releases, setup and cleanup are not an inferred zeroing operation.

## The local record built before the snapshot

At **0x140208fbe**, the interpreter retains its selected target in a local.
Scope copying through `0x1401882a0` occurs at **0x140208fd8**. It then calls
the complete helper `0x14017e680..0x14017e7aa` at **0x140208fe8**, with:

```text
RCX = address of stack local L, [rbp-0x30]
RDX = saved selected target T
R8  = null optional C-string S
```

The helper's first target query, complete leaf
`0x140099680..0x1400996a5`, walks `T+0x38` until target byte `+0x6a` has bit
`0x80`, then follows target `+0x20` and that object's `+0x10` to MovieImpl `M`.
It returns null if the chain ends. The constructor subsequently dereferences
the result, so its successful profile requires the valid target/movie chain;
it does not supply a fallback MovieImpl.

The helper constructs this local shape:

| Local field | Native producer | Selected null-S result |
| --- | --- | --- |
| `L+0x00` | Table **0x14250f7f0** | Exact local table |
| `L+0x08` | Return of `0x14009a690(M)` | Actual selected log pointer, possibly null |
| byte `L+0x10` | `(DWORD[M+0x4310] >> 2) & 1` | Movie-policy flag |
| byte `L+0x11` | `~(DWORD[M+0x4310] >> 6) & 1` | Movie-policy flag |
| byte `L+0x12` | Optional string comparison path, or zero when S is null | **Zero** at **0x14017e6d3** |
| `L+0x18` | Supplied S, potentially adjusted by a later basename branch | **Null** |

These offsets match the diagnostic consumer's record shape. **That layout
match is not an identity proof that `E+0x138 == &L`.** The checked selected
prologue makes no such assignment.

The complete MovieImpl log getter `0x14009a690..0x14009a711` returns cached
`M+0x220` directly when `M+0x4310` bit 1 is set. Otherwise it calls
`0x140052d50` on `M+0x10`, performs conditional reference operations, replaces
`M+0x220`, and returns the resulting cache. Those provider/reference calls
occur **before** the interpreter snapshots `E+0x138`; the local constructor
cannot by itself prove preservation from frame entry to that snapshot. The
local `+0x08` store adds no separate direct retain operation to the returned
pointer. Actual provider identity, provider reentry and pointer lifetime remain
explicit inputs/conditions.

With nonnull S the helper takes a different branch: it calls MovieImpl virtual
`+0x08`, then the returned object's virtual `+0x60`, compares the resulting
C-string with S, and selects MovieImpl flag bit 3 for equality or bit 4 for
inequality. When the resulting `L+0x12` is nonzero and movie flag bit 5 is clear,
it can adjust `L+0x18` after the last slash or backslash. These virtual/string
paths are skipped for the selected null-S interpreter call. They establish
neither the current movie filename nor any active Environment installation.

## Snapshot and concrete virtual route

After helper return, the interpreter executes:

```text
0x140208ff1  saved = QWORD[E+0x138]
0x140208ff8  [rbp-0x58] = saved
```

It queries the target version and fills adjacent interpreter-local state.
At **0x14020903c** it invokes `L`'s table `+0x08`. The exact table entry is
**0x14017e7b0**; its complete five-byte body is only:

```text
return zero_extend(BYTE[L+0x11]);
```

It neither writes `E+0x138` nor returns or publishes a context pointer. The
returned byte contributes to local interpreter flags. The local record's null
S and zero prefix-selector byte therefore cannot be used to select the active
Environment's raw-message branch: the active pointer is independently saved,
and may refer to a different record or be null.

The interpreter's early target gates branch to **0x14020d32f** before this
local construction and snapshot. No save/restore contract is implied for a
frame that never passed those gates. Normal bytecode processing follows the
snapshot; an initially empty body reaches the ordinary epilogue directly.

## Two exact restore checkpoints

| Path | Operations before the context restore | Exact store | Operations still after the store |
| --- | --- | --- | --- |
| Ordinary interpreter epilogue, **0x14020d271** | Execution-depth/pending-Value cleanup, target and cached-version restoration, conditional reference cleanup | **0x14020d2ec:** `E+0x138 = [rbp-0x58]` | Optional two heap virtual `+0x60` frees for local scope storage; shared return/security-cookie path |
| Out-of-buffer ActionJump branch | Target helper `0x14017a000`, conditional reference cleanup | **0x14020cd3b:** `E+0x138 = [rbp-0x58]` | Local scope destructor `0x140186a40`; shared return/security-cookie path |

For either executed store, with valid distinct frame/Environment storage,
**the field immediately after the store equals the value sampled at
0x140208ff1**. This statement does not require assuming that intervening
actions left the field unchanged. It also does not claim that the sampled
pointer's pointee is still alive or that later allocator/destructor callbacks
preserve it. Arbitrary aliasing, reentry, native unwinding and alternate exits
are outside that checkpoint contract.

The second path is precisely bound to **ActionJump `0x99`** by long-action
table entry **0x14020d580 → 0x14020cce1**. The interpreter decodes the signed
16-bit jump displacement, adds it to its already computed next PC with 32-bit
arithmetic, and compares that unsigned destination against the source buffer's
DWORD byte count. A destination below the count returns to **0x140209602**.
A destination at or above the count takes the special restore path. This
comparison is against the source buffer count, not a substituted Rust body
slice length. The special path must not be treated as the entire ordinary
epilogue: the preceding control/pending-state operations differ.

Both paths subsequently replace the local record table with
**0x1424fe0f8**. That stack-local teardown is distinct from restoring the
Environment's saved pointer. No blanket “restore means null” rule is valid.

## Direct access inventory and its limits

A bounded inventory of the known interpreter executable interval
**0x140208ef0..0x14020d34a**, 17,498 bytes, finds nine exact encodings of the
32-bit displacement `0x138`. Each was bound to canonical REA instruction bytes:

- Three are the Environment snapshot and two restore instructions above.
- Five are DWORD stores to **stack frame `rbp+0x138`**, not Environment fields.
- One is **0x14020b0a7**, `lea rdx,[rax+rdx*8+0x138]`, in the `TypeOf` handler.
  Its RAX comes through `E+0xe8 → +0x28 → +0x18`; it forms a selected string
  address for Value assignment. It is not an `E+0x138` context store.

This closes the tempting textual false matches without asserting a whole
program write analysis. The inventory does not detect every possible aliased,
computed-address or transitive callee write, and cannot prove current absence
of a context. No whole-binary search was used. An initial bounded text-disassembly
query timed out without output; it is retained as an excluded observation,
not negative evidence. Smaller canonical captures and the independently mapped
PE bytes support the claims here.

## The selected Mouse.hide boundary

The complete retained callback `0x1401b6fe0..0x1401b70bd` begins by releasing
the supplied result Value through `0x1401887a0`, then writes Undefined. It
loads its actual invocation Environment from `FnCall+0x30` and selects the
target/root/MovieImpl handler route described in
[REA_MOUSE_HIDE.md](REA_MOUSE_HIDE.md).

The callback contains no direct assignment to `E+0x138` and constructs no
diagnostic context record. With a missing handler it passes that Environment
unchanged as RCX and the fixed message as RDX, tail-jumping to **0x140186d10**
at **0x1401b70b8**. It does not substitute `&L`, a source filename or a zero
context. The provider path uses handler virtual `+0x08`; a nonzero argument
count can also invoke numeric conversion. Those calls, the initial Value
release and the diagnostic formatter's effects require their own closure.
The checked callback instructions alone do not prove preservation through all
of them or identify a current handler/context/log.

## Construction and execution contract for original code

The supported model is an optional, explicitly tracked diagnostic context:

1. A genuinely fresh Environment starts with null under the existing constructor
   profile. An adopted Environment requires its supplied context state.
2. Selected interpreter entry saves the actual incoming context at the proved
   snapshot. Local diagnostic-record construction and its virtual byte query
   do not replace it.
3. Each proved cleanup checkpoint restores the saved value. Effects during
   actions and after that store remain separate from the checkpoint itself.
4. Mouse.hide's missing-handler path consumes the callback Environment's actual
   context. It may use the consumer's null route only when null is established
   at that invocation point.

No VM implementation change is made by this study. The residual gates are
the selected pre-snapshot providers, actual incoming context and pointee
lifetime, intervening indirect/callee effects, post-restore cleanup effects,
and unproved alternate/native-unwind paths. The current manager fixture must
not supply those answers merely because its fresh ancestor began with null.

## Evidence, validation and provenance

All private paths below are under
`~/.local/share/marvel-heroes-forever-rust/evidence/`. The new original verifier
`diagnostic-environment/verify.py` independently maps PE sections, matches every
selected instruction to the owned file, follows chained unwind entries for
complete-function boundaries, and checks the local virtual and opcode tables.
It verifies **702 selected instructions / 2,674 instruction bytes, six complete
unwind-bounded functions and 24 table bytes**. Two additional leaf extents are
bounded by their complete instruction sequence, RET and following INT3. This
is static evidence, not a native execution or behavioral-parity test.

New canonical records are retained in
`diagnostic-environment/complete.bundle.json`:

| Evidence | Selected claim |
| --- | --- |
| `ev_32e41446edcb3589f5aec36ac71720f5cfb25db206bce45498fa50f4f6ad9965` | Complete local constructor `17e680` |
| `ev_1a9c810fb4bb8562054925dfc8c093a864c549688ebeeaae0e6127cd791e354a` | Local table, five-byte virtual getter, complete MovieImpl log getter |
| `ev_a125a11e2045836f3d9d5247bf8de45c60c73b1b905622a9c1ddc334ce37f679` | Complete 37-byte target/movie leaf and selected special restore branch |
| `ev_cb3d2bee4706701c3ed3edb2b4a48b41ddfa30c3d5fa3885cac7d426dc07a82a` | Exact stack-displacement candidates and ActionJump table entry |
| `ev_e2cdfada4f35935bc6d21db3b7a26617bb8813def22899b7d56dc8ab1ab2c011` | Complete selected TypeOf branch and its opcode entry |

The receipt additionally binds reused canonical records from
`rea-saved-event-execution/final.bundle.json` (entry and ordinary epilogue),
`rea-saved-script-calls/final.bundle.json` (long-action dispatch),
`rea-event-dispatch/complete.bundle.json` (invoker and selected Environment
provider), `rea-mouse-hide/hide.bundle.json` (complete callback) and
`rea-asflags-followup-version/final.bundle.json` (fresh constructor).
Trailing unrelated or incomplete decoded instructions are explicitly excluded.

| Source | Licence/ownership and use |
| --- | --- |
| `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | Blizz-owned proprietary retail 1.52.0.1700 artifact, compatibility study only; SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` |
| Existing project docs and retained evidence named above | Original private project analysis; checkpoint and byte-evidence reuse, not outside implementation |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and `references/native-and-artifacts.md` | Installed toolkit instructions; REA capture/evidence handling only |
| `diagnostic-environment/pe.py`, `verify.py`, source ledger and receipt | Original project validation tools and records; no game/SDK implementation copied |

The source ledger records exact source hashes. These reuse the owned-artifact
and toolkit provenance already recorded in [PROVENANCE.md](PROVENANCE.md).
No new external implementation, dependency or network source was consulted.
