# Fresh AS2 level-zero root name

**VERIFIED static fresh-path result:** the selected AS2 level-zero setup
assigns the seven-byte name **`_level0`** to its newly created root target.
This closes the formatter gap in
[REA_ATTACHED_NAME.md](REA_ATTACHED_NAME.md#follow-up-concrete-level-root-name-assignment).
The checkpoint is the successful native name-field assignment reached from
`0x140210180` through `0x1401871a0(owner, 0)`. It is not a native execution
capture, the name of an adopted instance, or proof that the name survives all
later setup, host calls and original scripts.

The proof uses the owned build 1.52.0.1700 Win64 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Imported CRT calls are identified in this executable's PE import tables;
their ordinary ASCII classification, decimal conversion and byte-copy
contracts are assumed. The imported DLL was not loaded, inspected or run.
No service, original client, account, network or current-instance probe was
performed. All existing live and access holds remain in force.

## Exact producer and storage

Retained complete root setup `0x140210180..0x14021034d` receives an explicit
MovieDef and creates its case-3 target through support factory
`0x140178440`. Its CreateInfo contains the source obtained from
`[MovieDef+0x38]+0x18`, that same MovieDef, and zero; the parent argument is
null. After `0x1400b5150(target, 1)`, setup derives the target's actual AVM
owner and calls **`0x1401871a0(owner, 0)` at `0x1402102e8`**. The later
`0x14005fdb0(MovieImpl, 0, target)` level publication has not occurred yet.
See [REA_DISPLAY_LIFECYCLE.md](REA_DISPLAY_LIFECYCLE.md) and the retained
root-name evidence for these concrete bindings.

The complete level-name producer `0x1401871a0..0x140187407` retains the
integer in ESI, stores it at owner `+0x1d0` at `0x14018726e`, and supplies
literal **`_level{0}\0` at `0x14250fa28`** to the parser. For formatter base
`F = RSP+0x40`, the producer's fixed 64-byte output buffer is at `F+0x400`.
Its prologue establishes `RBP=RSP+0x100`; consequently the buffer passed as
`RBP+0x340` is the same storage, not an alias of an item or entry array.

Formatter constructor `0x140082b60..0x140082be5` initializes argument byte
`F+9=0`, unresolved-argument count `F+0xa=0`, cursor `F+0xc=0xffff`, entry
count `F+0x40=0`, and empty dynamic-entry fields. The producer's descriptor
selects mode 2, output `F+0x400`, capacity 64. These are specific consumed
fields; neither the descriptor's unused high bits nor all stack storage is
asserted to be zero.

The constructor also supplies a 512-byte inline item arena starting at
aligned `F+0x1e8`. For this aligned stack formatter its initial capacity is
`0x200`; the sole `0x60`-byte numeric item fits. The producer selects the
inline allocation branch, so no fallback allocator call is needed for this
item. Its storage lies below `F+0x3e8`, distinct from the final output buffer.
This is a concrete placement argument, not arbitrary-aliasing support.

## Template, zero conversion and output

The complete parser `0x140082dd0..0x1400830da` handles the concrete template
as follows. These are branches derived from native instructions and the
literal bytes, not a replacement formatter executed as an oracle.

| Stage | Exact result on this input |
| --- | --- |
| Literal before `{` | `0x140082c20` appends kind-0 entry 0, pointer to `_level`, length 6. |
| Placeholder `{0}` | Imported `isspace('0')` is false, `isdigit('0')` is true, and `atoi("0}")` is zero. |
| Placeholder record | Kind-1 entry 1 selects argument 0. Its format-spec length is zero because the cursor reaches `}` immediately after the index. |
| End of template | Entry count is 2; there is no trailing literal. Explicit-index parsing leaves unresolved count `F+0xa=0`. |
| Argument selection | `0x140080bc0` matches entry 1 against current argument byte 0, records index 1 and advances its cursor to 2. |
| Item binding | `0x140080c60` changes that entry to kind 2 with the actual item and ownership byte 1. Zero format length skips item virtual `+0x10`, so no format override is applied. |
| Completion | The next selection fails at entry count 2. The producer increments the argument byte and calls `0x140081dc0`, whose zero unresolved count tail-calls `0x140081450`. |

The item's complete constructor `0x1400804c0..0x14008053b` receives
**R8D=0** at `0x140187347`. It sign-extends this to the item's integer field,
clears its formatted byte and terminator, and initializes the fields consumed
by the concrete path: decimal radix 10, minimum digit count 1, width 1,
disabled grouping, and the signed-32 selector. Optional sign/prefix/padding
bits consumed by this path are clear. Unused upper flag bits remain
unspecified.

Finalizer `0x140081450..0x14008160c` visits entries in order. For entry 1 it
calls complete preparation helper `0x140080d00..0x140081449`. The item's
primary table `0x142504250` binds `+0x30` to leaf `0x1405b93b0`, whose exact
three bytes are `xor al,al; ret`. Thus preparation jumps directly to
`0x140081427`; with the freshly cleared formatted byte, it invokes item
virtual `+0x18`, **`0x1400805c0`**, at `0x140081433`. No context-search or
neighbor-item callbacks in the other preparation branches run on this path.

Complete numeric formatter `0x1400805c0..0x14008083d` calls complete digit
converter `0x14007fd80..0x14007fe2f` with magnitude zero and radix ten. The
converter executes its loop once, takes remainder zero from `div 10`, and
selects byte `0x30` from verified table `0123456789ABCDEF\0` at
`0x1425040a8`. It writes `0` at item `+0x5b`, leaving the constructor's NUL
at `+0x5c`. The selected numeric branches add no sign, space, radix prefix,
grouping or padding; the formatted byte becomes 1.

The concrete item output span is therefore `{pointer: item+0x5b, length: 1}`.
Virtual `+0x28` (`0x14007fe30`) computes that length; virtual `+0x20`
(`0x140080080`) returns the span. The finalizer obtains total length
**6+1=7**, then selects the mode-2 fixed-buffer branch. Formatter virtual
`+8`, table slot `0x142504480`, resolves to complete writer
`0x140081610..0x140081703`. It copies entry 0's six literal bytes, then entry
1's one-byte span, using imported `MSVCR120.dll!memcpy`. At
`0x140081542`, the finalizer writes NUL at
`buffer[min(total_length, capacity-1)] = buffer[7]`. The resulting first eight
bytes are exactly:

```text
5f 6c 65 76 65 6c 30 00    _level0\0
```

## Cleanup, interning and the name-field checkpoint

The producer calls formatter destructor `0x140082820..0x140082904` before
interning the output. For its owned kind-2 entry, this invokes the numeric
destructor `0x140080300..0x140080331` with EDX=0. That branch only rewrites
the item's primary and secondary vtable pointers; it does not read the
formatter pointer, invoke a callback, or touch the separate output buffer.
The formatter recognizes the inline item and skips heap deletion. Its
dynamic-entry pointer is still null. Thus this concrete cleanup preserves
the eight output bytes.

At `0x1401873b3`, the producer passes this exact buffer to `0x140097410`
through the owner's Environment/String-manager chain. Retained complete
`0x140097410..0x14009744d` computes the NUL-terminated byte length and
tail-calls `0x1400972e0` with the original bytes; see
[REA_ENUMERATION.md](REA_ENUMERATION.md) and private
`rea-enumeration-callback/constant-pool-finding-summary.json`.

The returned StringNode is passed at `0x1401873cb` to target virtual
`+0x3f0`, concretely `0x1400b56f0`, then `0x14009e930`. The retained complete
setter has two successful branches:

- Existing handle: `0x14009a410` retains the supplied node and stores it at
  **handle `+0x10` at `0x14009a441`**. Its parent-path field is separate.
- Missing handle: `0x14009a2c0` stores this node at **handle `+0x10` at
  `0x14009a2d8`**; the setter publishes the handle at target `+0x70` at
  `0x14009ea06`.

Allocation/reference failures are real branches, not a success guarantee.
The explicit level-name path does not substitute the lazy `instance{0}`
generator or its MovieImpl `+0x3b90` counter. This establishes the successful
fresh name-field checkpoint while preserving both handle branches.

## Movie definition association and later effects

The retained native host path is
`0x14127fdf0 → 0x141276a60 → 0x141277340 → 0x141277840`.
The last routine obtains a loader result from `0x140031710` and returns it
unchanged. The lower loader retains that result in R15 at `0x1412773fd`,
stores it at wrapper `+0x34` at `0x14127740e`, reloads that same field at
`0x14127771d`, and invokes its virtual `+0xc0` at `0x14127778b`. These are
identity-preserving stores, not a filename-to-root inference.

For the concrete MovieDef table `0x1425048b0`, `+0xc0` resolves to
`0x140086ad0`, whose path reaches virtual `+0xb8`, `0x1400882c0`.
That routine retains the original MovieDef in RSI, selects support state 23
when the DataDef-state `+0xd0` bit 3 is clear (24 otherwise), and calls
support virtual `+8` at `0x140088349`. **On the selected AS2 support binding**
this is `0x140178340`, constructing a MovieImpl and its AS2 root. It then
obtains the root from returned MovieImpl `+0x18`, passes the same retained
MovieDef in RDX at `0x140088370`, and calls root `+0x88` at `0x140088376`,
concretely the already established `0x140210180` setup.

This root setup occurs before the retained boolean's later test. The false
boolean supplied through the retained Start path skips a subsequent optional
MovieImpl `+0xc0` call; it does not skip fresh root setup. This observation
does not imply that all callbacks, allocations, scripts or other effects
throughout movie creation are absent.

The concrete MovieDef table and AS2 support are necessary path conditions.
This document does not infer those conditions for an arbitrary loader return,
nor claim that the current frontend manager executed this path. The owned
frontend default-movie reference and host source path discussed in
[FRONTEND_EXTERNAL_RECEIVER.md](FRONTEND_EXTERNAL_RECEIVER.md) remain static
configuration/dispatch evidence, not current-instance evidence.

The narrower loader-type join follows `0x140031710 → 0x14004ffc0 →
0x14004ef00`. Its input to complete identity-or-null filter `0x14004e340`
comes from `0x14004e4b0`. Retained complete `0x14004e4b0` has a fresh branch
when its lookup returns status 3: it allocates `0x40` bytes, calls constructor
`0x14008d520` at `0x14004e555` with the original DataDef in RDX, retains the
constructed pointer in RBX, and returns it at `0x14004e6df`. That constructor
installs the concrete table `0x1425048b0`. Subject to the selected successful
return/filter branches, this supplies the concrete fresh MovieDef needed by
the association above.

Lookup results other than 3 instead use the cached-pointer helper
`0x14004c2e0`; their concrete category has not been established here. The
fresh allocation proof does not silently cover cached returns, null/error
outcomes or arbitrary MovieDef subclasses. It is sufficient to document the
selected fresh path, and insufficient to promote a current manager fixture.

After naming, level publication calls target `+0x3e0` and stores the root;
root setup then invokes further virtuals including `+0x118`, `0x140196a30`,
which can process pending ordinary member assignments. The native `_name`
setter case at `0x1401a926c..0x1401a929d` reaches the same name setter.
Therefore later rename is a concrete available mechanism. No complete
name-preservation proof through these later phases, host work or original
script execution is asserted.

## Minimal construction contract

**INFERRED implementation recommendation:** a future explicit fresh AS2
level-zero constructor may derive `_level0` as part of the evidenced
name-assignment transition, carrying the actual source MovieDef/owner/target
identity and selected AS2 support conditions. It should not accept an
arbitrary diagnostic name for that transition. Keep it distinct from adopting
an existing instance, which still requires an honest current-name witness.

This is a narrowly justified fresh-path input, not an implementation of the
whole native loader or permission to assert other target fields, inherited
state, pending actions, publication order, or current-scene identity. A
synthetic fixture should not be relabeled as the actual manager simply by
changing its name to `_level0`.

## Private evidence and provenance

All paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. No binary bytes,
disassembly, captured assets or private bundle are committed with this doc.

| Evidence | Contribution |
| --- | --- |
| `fresh-root-name/formatter.bundle.json` | Four new formatter/parser/output/import records; also retains two earlier Environment records outside this task's claims. |
| `fresh-root-name/verification.json` | Exact PE mapping for 16 selected code ranges, 1,274 instructions / 4,678 bytes; complete chained unwind coverage where available; table/literal/import bindings. |
| `fresh-root-name/import-bindings.json` | Executable PE bindings to `MSVCR120.dll` `isspace`, `isdigit`, `atoi`, `memcpy`; no DLL execution. |
| `fresh-root-name/integer-zero/complete.bundle.json` and `receipt.json` | Four numeric-method records, 12 selected code/data ranges / 1,142 bytes, four complete unwind groups, exact zero output and cleanup. |
| `fresh-root-name/movie-association/` | Bounded MovieDef creation/AS2 root association captures and retained-source joins. |
| `fresh-root-name/receipt.json` | Joined task receipt, original document hash, source hashes and explicit remaining boundaries. |
| `rea-attached-root-name/verification.json` and `checkpoint.bundle.json` | Existing root setup, exact literal, producer, name setter/handle stores and later rename boundaries. |
| `rea-enumeration-callback/constructor-and-pool-final.bundle.json` | Reused complete StringManager byte-length/interning wrapper, record `ev_aab6a42276a45efe8236d3bdbf7e1df13827f7e052fed26d812cc12c9271dccf`. |

The selected new formatter records are
`ev_4e49228e232ef1e11d013fb3bf8a290005d37d5c526dd9fe1ef04e11c1203a3e`,
`ev_5fe95d28434f2caace9b1dcafcc709cdf2687cb8f0704bdfc1c35de25765cf5d`,
`ev_3de35da6c99f37ecf022281fd0f710089b904d20ebe521da7823b142a3977751`,
and `ev_f9befd5164f90704f0b9e0f260c868da9350bcb3623aeaaf5a1ccfec72fe698c`.
All completed with exit zero, no truncation, and unchanged executable hash.
Initial partial unwind fragments and unrelated neighboring decoder tails
remain retained but are excluded from complete-function claims. Multi-entry
chained unwind metadata was followed rather than treating its first fragment
as the entire function. Validation is static; no Rust/core change, gameplay
test, current-state capture or native execution was performed for this doc.

Sources used: the owner's proprietary Gazillion retail executable (private
compatibility study; no redistribution licence); the original project docs
and retained evidence named here, including the preceding fresh Environment
and attached-target studies; installed REA skill/native-artifact guidance and
REA 6.9.0 (MIT tooling) with Rizin (LGPL-3.0 tooling); Python standard library
(PSF) for original byte/extent checks. No Scaleform SDK/source or outside game
implementation was copied. This is the task's source ledger for integration
into [PROVENANCE.md](PROVENANCE.md).
