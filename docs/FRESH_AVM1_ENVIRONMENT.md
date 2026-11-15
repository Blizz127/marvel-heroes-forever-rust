# Fresh AS2 Environment construction

**VERIFIED static checkpoint:** the complete native Environment constructor
`0x1401c2fe0..0x1401c30a9` initializes an empty logical operand stack, four
Undefined fixed registers, an empty dynamic-register vector, and empty
activation storage. Its pending-exception Value has sentinel tag `0x0a`, and
execution/invocation depths are zero. This is the successful fresh-allocation
checkpoint before a target, string context or loaded movie version is bound.

This is not the current state of an adopted login instance. The source
MovieClip owner binds those identities afterward and then performs prototype
selection. Publication and actions occur later still. No original client,
service, VM or renderer was executed for this study.

## Exact constructor state

Let `E` be the Environment address. The complete constructor's two calls are
`0x140187780(E+0x08)` and `0x1401878a0(E+0xf8)`. Both complete helper bodies
were newly retained through REA and matched to the owned executable. Under
ordinary successful allocation with valid, distinct storage, the constructor
return establishes:

| Environment field | Constructor result | Meaning and limits |
| --- | --- | --- |
| `+0x08` operand-stack structure | One allocated page; current slot equals current page base; sentinel tag 0 | Logical depth zero, not a null backing allocation |
| `+0x48`, `+0x68`, `+0x88`, `+0xa8` | Each Value's tag byte is 0 | Four Undefined Values; inactive payload and metadata bytes are not zeroed by these stores |
| `+0xc8`, `+0xd0`, `+0xd8` | Three zero QWORDs | Dynamic Function2 register-vector data/count/capacity start empty |
| `+0xe0` | Null | Current target is not bound yet |
| `+0xe8`, `+0xf0` | Null context pointer and zero version byte | Not a valid loaded movie context or an inferred SWF version |
| `+0xf8` function-stack structure | One allocated page; current slot equals page base; sentinel pointer null | No selected function in the sentinel slot; backing allocation still exists |
| `+0x138` | Zero QWORD | Scratch field; its public semantic name remains unassigned |
| `+0x140`, `+0x148`, `+0x150` | Three zero QWORDs | Empty auxiliary vector state; no complete Try implementation follows from this initialization |
| `+0x158` | Value tag `0x0a` | Empty pending-exception sentinel, distinct from Undefined tag 0 |
| `+0x178` | Zero DWORD | Interpreter execution-depth field |
| `+0x17c` | Zero WORD | ScriptFunction invocation-depth field |
| `+0x17e` | AND with `0xfc` | Bits 0 and 1 clear; upper six bits are not initialized by this instruction |
| `+0x180`, `+0x188`, `+0x190` | Three zero QWORDs | Activation-vector data/count/capacity start empty |

The initial fixed-register stores are at `0x1401c3001`, `...3005`,
`...3009` and `...3010`. The constructor writes the pending sentinel at
`0x1401c306b`, clears the two control bits at `...3072`, and initializes the
depth fields at `...3079`/`...307f`. This does not justify serializing the whole
Environment or its Values as zero-filled native bytes.

## Physical sentinels and logical depth

The complete operand-stack constructor is `0x140187780..0x140187894`.
Its successful fresh path requests `0x408` bytes for the first page and `0x80`
bytes for the page-pointer array. After the initial count/capacity stores and
ordinary successful allocator returns, the page count becomes 1 and the array
capacity becomes 16. The growth helper is not reached on this fresh count-0,
capacity-16 path. For a nonnull page `P`, it stores:

```text
E+08 = P             current slot
E+10 = P             current page base
E+18 = P + 0x400     current page limit
E+20 = P             retained first-page pointer
E+30 = 1             page count
tag[P] = 0          physical bottom sentinel
```

The previously retained action-loop expression at
`0x140209ce6..0x140209d03` is:

```text
logical_depth = (page_count << 5) + ((top - page_base) >> 5) - 32
```

Substituting count 1 and equal pointers gives zero. An original Rust logical
empty vector therefore need not expose the physical bottom sentinel as an
operand. Native empty-Pop behavior remains the separate contract in
[REA_NATIVE_ALTERNATIVE.md](REA_NATIVE_ALTERNATIVE.md#avm1-empty-pop-follow-up),
not permission to generalize underflow handling to every opcode.

The complete function-stack constructor is `0x1401878a0..0x1401879b1`.
It uses the corresponding pointer-stack layout, requests `0x108` bytes for its
first page and `0x80` for the pointer array, records one page/capacity 16, and
sets the current slot and page base to the new page. The limit is `P+0x100` and
the sentinel QWORD is null. Retained ordinary setup increments `E+0xf8` by 8
at `0x1402050c2` before storing the executing function at `...5104`.
That first push is above the sentinel; it does not overwrite a preexisting
active function. See [SAVED_EVENT_EXECUTION.md](SAVED_EVENT_EXECUTION.md).

The allocator's failure policy, page-growth semantics and runtime heap hooks
are not implemented or proved equivalent here. The initialized fields and
branch conditions above describe the supported successful ordinary allocation
profile, not arbitrary allocation reentry or aliasing.

## Pending exception and depth identities

The short-opcode table binds ActionThrow `0x2a` through entry
`0x14020d3f4` to `0x140209e73`. Its complete selected opcode branch ends at the
jump at `0x140209eeb`, exclusive end `0x140209ef0`. It passes the current
operand to ordinary Value assignment `0x14018d9d0` with destination `E+0x158`,
pops the source operand and calls the exception-processing path. The Value
assignment contract is separately retained in
[REA_NATIVE_ALTERNATIVE.md](REA_NATIVE_ALTERNATIVE.md). This identifies the
constructor's `+0x158` slot as pending-exception storage; it does not implement
Throw, Try, catch/finally or their cleanup.

The retained normal interpreter epilogue decrements `E+0x178`. When that depth
reaches zero and the pending Value is not tag `0x0a`, it releases the Value,
restores tag `0x0a` and clears `E+0x17e` bit 0. Bit 1 is separately preserved
and restored with target-operation state; the whole byte must not be called
an exception Boolean. The complete ScriptFunction invoker
`0x14020e190..0x14020e307` increments/checks/decrements WORD `E+0x17c`, confirming
the distinct invocation-depth field. The fresh constructor begins outside both
execution and invocation.

## Owner binding is a later checkpoint

The complete source MovieClip owner constructor
`0x140203560..0x140203716` places the Environment at `E=O+0x38` and calls its
constructor at `0x1402035d6`. Afterward it performs these later bindings:

| Site | Store or operation |
| --- | --- |
| `0x140203607` | Clears `E+0x17e` bit 1 again |
| `0x140203611` | Stores the concrete target into `O+0x118 = E+0xe0` |
| `0x14020361d` | Stores the target-derived cached version into `O+0x128 = E+0xf0` |
| `0x140203646` | Stores the returned context into `O+0x120 = E+0xe8` |
| `0x1402036b6` | Calls prototype selector `0x14019b570` with this Environment |

The loaded-version producer chain is separately established in
[ATTACHED_TARGET_INITIALIZATION.md](ATTACHED_TARGET_INITIALIZATION.md) and the
version follow-up in `REA_NATIVE_ALTERNATIVE.md`. The zero constructor version
must not become an accepted loaded-header default. Prototype factory/getter
work, publication, initializer/saved actions, and later callbacks are separate
state transitions; this document does not assert they preserve all fresh state.
EditText has a different owner and ancestor-selected Environment, so this
case-3 embedded constructor must not be copied into each text field.

## Construction API implication

**INFERRED faithful implementation boundary:** a fresh-construction operation
may initialize logical operands empty, fixed registers Undefined, dynamic
registers and activations empty, both depths zero, and pending exception absent
at this exact native checkpoint. Model the two known control bits separately
or preserve an explicit known-bit mask. Do not invent values for inactive Value
payloads, upper flag bits or unknown scratch semantics.

Keep that fresh state unbound until the actual owner/target/context/version
steps run. Subsequent prototype and lifecycle work must use the same persistent
Environment, so their mutations survive. An adoption API for an already-live
owner must still require supplied current state; it cannot use this constructor
profile to reset registers, clear exceptions or replace retained operands.

This supplies native initialization evidence for a future constructor path and
for the empty-bank/no-pending-exception shared-call profile. It is not an
assertion that any previously adopted scene Environment is at that checkpoint.

## Evidence, verification and provenance

Private root: `evidence/fresh-avm1-environment/` under the project data root.
`final.bundle.json` contains two new REA records:

- `ev_55b420dee9021cfda6c267bd4cfc9f274d2056ad1113610ed54ce13d522d6926`:
  both complete stack constructors.
- `ev_f57fc3e499363f39c35f1ebb3dd1183454f41b70157b4194be0e4fca8c7d76d4`:
  Throw dispatch entry and selected opcode branch.

Both completed with exit 0, untruncated output and unchanged target hash.
`verification.json` records 472 selected instructions / 1,936 bytes across
eight exact ranges. Five complete functions also match their PE `.pdata`
extents. The new captures contain 180 valid instructions / 674 instruction
bytes and four dispatch bytes, all matched to the PE. Three trailing bytes
decoded as invalid after the Throw branch are retained and explicitly unused.
The mathematical fresh-depth witness yields zero; it is not a native replay.

The first local verifier incorrectly included an unrelated partial epilogue
fragment from the constructor's older evidence record. Its failure is retained;
the final verifier binds each selected range to its exact complete record.
No native call was repeated for that correction. Complete raw tool replies,
the final exported bundle, source hashes, and the original verifier remain
private. No product code or runtime tests were changed or run.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's `MarvelHeroesOmega.exe`, build `1.52.0.1700`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Exact constructors, dispatch and owner/depth checkpoints |
| Existing project documents cited above and their retained REA bundles | Original project investigation; underlying native bytes remain proprietary | Reuse complete constructor, owner, depth, assignment and epilogue evidence |
| Installed REA skill and native/artifact guide; REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling; no implementation copied | Bounded target-free static capture and complete Evidence export |
| Python standard library | PSF tooling; checker is original project code | PE mapping, exact byte/extent checks and immutable receipts |

No external SDK or reference implementation, live service, credentials, native
client process, shared configuration or other project's source was modified.
