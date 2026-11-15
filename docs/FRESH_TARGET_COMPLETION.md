# Conditional source-owner constructor completion

**VERIFIED static conditional result:** the four target fields established at
`0x14020366e` survive the remaining source-owner constructor through its normal
return `0x140203715`, and through the case-3 factory return, under the exact
allocation, binding and cleanup profile below:

| Field | Owner/factory return value |
| --- | --- |
| DWORD target `T+0x6a` | `0x52004f80` |
| WORD target `T+0x88` | `0x000a` |
| WORD source target `T+0x140` | `0x0001` |
| QWORD target `T+0xb8` | `0x0000000000000010` |

This extends [FRESH_TARGET_STATE.md](FRESH_TARGET_STATE.md), not live-instance
defaults. The supported profile requires positive frame capacity, successful
disjoint allocations, effective original MovieClip cache/global/prototype data
bindings at selector entry, and the positive-reference cleanup route. Earlier
bootstrap registration alone does not prove those bindings survive every
intervening startup call. No native client, original script, VM, all34 replay,
renderer, server or service was executed.

## Frame capacity and ledger

Let `O=T+0x148`, `E=O+0x38`, and `V=O+0x1e0`. Owner `0x140203560` has
already zeroed the ledger pointer, count and capacity at `V`, `V+8` and
`V+0x10`. These owner-relative locations are disjoint from all four target
fields; their initialization is not a zero-filled-target assumption.

At `0x14020367c`, source DataDef table `0x142504680` virtual `+0x50`
selects complete getter `0x1400863d0`. It follows DataDef `+0x40` to state
and returns DWORD state `+0x74`, frame capacity `N`. This is a different
getter and field from the version read at state `+0x54`. It makes no calls
or stores.

The caller zero-extends `N` and invokes resize `0x140148320(V,N)`. Complete
resize and reserve `0x1401481d0` establish the following fresh branches:

| Fresh input | Operation |
| --- | --- |
| `N=0`, existing count/capacity zero | Store count zero; do not allocate. The owner still subsequently calls `memset(NULL,0,0)`. That unobserved CRT edge is outside the positive-capacity completion profile. |
| `N>0`, existing data/count/capacity zero | Request growth `G=N+(N>>2)`, round capacity to `C=(G+3)&~3`, allocate, store returned data pointer and `C`, then store count `N`. |

On the positive branch, global heap `H=*[0x142c947f0]` virtual `+0x70`
receives `RCX=H`, `RDX=V`, `R8=C`, and `R9` pointing to a stack DWORD equal
to 2. Fresh null data excludes the realloc and free branches. The reserve
body does **not** check the allocation result for null before publishing it;
ordinary successful allocation of at least `C` disjoint bytes is a real
precondition. The `u32` input's growth arithmetic fits the observed 64-bit
operations; this is not a promise that arbitrarily large allocation succeeds.

The owner then calls `0x141c6ed8c` with the returned pointer, zero, and `N`.
The exact PE thunk/IAT/import chain identifies `MSVCR120.dll!memset`.
Under its ordinary byte-fill contract, the first `N` ledger bytes become zero;
padding through capacity `C` is not asserted to be initialized. The imported
DLL was neither inspected nor executed. Heap callback/reentry, allocation
failure, overlapping storage and corrupted metadata remain outside this profile.

**Next independent checkpoint:** at `0x1402036a0`, before prototype selection,
the four target fields retain the opening values and the positive-capacity
ledger is initialized. This checkpoint does not need a claim about the current
global `MovieClip` binding.

## Exact effective bindings for the completed profile

The selector call at `0x1402036b6` is `0x14019b570(C,E,6)`, where `C` is
the actual target/root context. This completion profile requires the following
state **at that call**, after earlier startup work:

| Identity / data | Required effective state |
| --- | --- |
| Context numeric prototype cache `C+0x10` | Key 6 exists and names retained nonnull original MovieClip prototype `P`. |
| Current global `C+0x60` | Concrete interface `0x142512aa0`; its own `MovieClip` data member contains original native function `F`, native tag 8, auxiliary frame pointer null and reference flags zero. |
| Native function `F` | Primary table `0x142514280`, interface `0x142514430`; own `prototype` data member contains `P`, native object tag 6. |
| Prototype `P` | Concrete MovieClip prototype interface `0x142514db0`; category getter returns `0x0b`, not special function-conversion category `0x17`. |
| Lookup context | Correct actual context and interned names; loaded version greater than 6 selects the retained exact-name route. |
| Storage and lifetime | Target/owner/Environment, Values, function, prototype, collector and ledger are valid distinct storage where required; rooted function/prototype counts remain positive without low-26-bit overflow during the balanced retains/releases. |

Retained bootstrap evidence grounds these identities and links at their
registration checkpoints. It inserts prototype `P` into numeric key 6 at
`0x14020f689` and writes function `F` to global `MovieClip` at
`0x14020fb9d`. Complete prototype/link constructors establish `F.prototype=P`
with descriptor flags 3. Function interface assignment is visible at
`0x14020f362..0x14020f369`. The saved tag-8 auxiliary field receives zero
and its reference flags are explicitly zero; they are not descriptor flags.
See [AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md) and
[REA_MOVIECLIP_CONSTRUCTOR.md](REA_MOVIECLIP_CONSTRUCTOR.md).

These are usable current-entry guards. They are **not** an assertion that
every fresh context automatically reaches the selector with those values.

## Selected lookup and conversion have no script callback

Numeric-cache lookup `0x1401862d0` is now captured completely: it contains
only table reads, local register/stack work and returns, with no calls or
non-stack stores. With key 6 present, fallback helper `0x140194830` avoids
factory resolver `0x140194670`. Numeric cache presence is the condition; it
must not be confused with the separate lazy function-cache shortcut.

The actual global raw slot `+0x58` is `0x1401db080`. The name `MovieClip`
is distinct from its special `gfxExtensions` builtin, so it tail-calls ordinary
getter `0x140197000`. Function interface `0x142514430` slot `+0x58` points
directly to that same getter. Both required members are present in the first
object's own map, so neither read falls through to an inherited special marker,
resolver or alternate-name path. The getter copies the retained data Value
through `0x14018d9d0`; it does not invoke the function merely because the
value is callable.

Value conversion `0x140199640` takes direct tag-8 function and tag-6 object
branches. Accessor tag 9 and deferred-factory tag 11 are not reached. Copying
the prototype object also consults its category; the exact `0x0b` getter
excludes category `0x17`'s additional conversion virtual. The null auxiliary
function frame excludes `0x1401883c0` in the selected copy route.

The selected result is therefore the same `P` as both cached fallback and
current `F.prototype`. This evidence does not broaden the profile to arbitrary
native getters, accessors, deferred tokens, modified prototype data or absent
members. The wider selection decision table remains in
[FRESH_MOVIECLIP_PROTOTYPE.md](FRESH_MOVIECLIP_PROTOTYPE.md).

## Reference cleanup is stateful; its successful branch is bounded

Data lookup is not enough to establish a pure selector. Complete Value copy
`0x14018d9d0` and cleanup `0x1401887a0`, together with the selector's explicit
retains/releases, call `0x1401860e0` even when references remain rooted. The
caller checks the count before decrement; it does not call the helper only
when the resulting count is zero.

The complete 482-byte helper spans **three chained unwind entries**. Its
positive-count branch updates object header mark bits and may call
`0x140184ba0` to enter the object in its collector's root buffer. Retain
operations apply `(header+1)&0x8fffffff`, so an earlier mark state alone
cannot be presumed to bypass this work.

The selected positive-count cleanup is bounded as follows:

| Root-buffer path | Effects and condition |
| --- | --- |
| Already admitted header state / buffer membership | Header work and return; no destruction dispatch. |
| Reusable free slot | Update the object's buffer index/header and the collector's slot/free-list metadata. No helper call. |
| Existing page capacity | Publish the object pointer into a valid slot and increment the buffer count. No allocation call. |
| Successful page growth | Complete `0x14017b3a0` may grow the page-pointer array through heap virtual `+0x58` or `+0x70`, then allocate a `0x2000`-byte page through `+0x70`. Successful ordinary disjoint allocations return true and permit the slot write. |

All these writes address the separate function/prototype headers or collector
storage, not the target flag fields, owner prototype slot or Environment.
Allocator implementations and arbitrary hooks are explicit ordinary-success
contracts, as for the frame ledger; their bodies were not generalized here.

**The exact excluded callback boundary is known:** failure to grow/obtain a
valid root-buffer slot reaches `0x1401817f0` at `0x140184c77`, invoking
collector processing. Zero-count release instead reaches virtual destruction
and recursive processing in `0x1401860e0`. This study does not claim either
path preserves the selected target or other runtime state. Nonnull captured
frames, category `0x17`, refcount overflow, malformed collector state and
allocation/reallocation failure also remain excluded.

## Final direct store and factory return

On the bounded path above, no selector callback changes fresh `O+0x18`, which
was initialized null. After selection, the owner retains returned `P` and
stores it directly to `O+0x18` at `0x1402036f4`. The old-prototype release
branch is therefore excluded. This is not the later attached-prototype setter:
it creates no bound storage and does not run that setter's hook.

The owner epilogue contains no further calls. The selected case-3 factory
jumps from `0x14017864c` to `0x140178505`, returning the same target at
`0x14017851f` without an intervening call or target mutation. Thus the four
opening values survive through the owner return and
factory return under the complete stated profile. Fresh optional bound storage
`O+0x1d8` remains null; the ledger and selected prototype are the explicit
new owner state. No initializer, saved handler, registered constructor, dynamic
event or scheduler is completed by these stores.

The next root call is `0x1400b5150(T,1)`, followed by resource-context setup
and naming/publication. Those are separate transitions. This document makes
no survival claim beyond the factory return.

## Remaining automatic fresh-root dependency

Complete root `0x140210180` calls global bootstrap before publishing its new
context and invoking the source factory. The retained registration points prove
the eager bindings above, but the prior bootstrap inventory is a prefix, not
a complete return proof. In that prefix, `0x14020feeb` calls
`0x140183fa0` with **`C+0x70`**, then `0x14020fef3` calls
`0x1401954c0(C)`. Root setup also calls `0x1400583e0`, `0x1400499f0` and
`0x1400a9ae0` before the factory, with conditional reference cleanup.

Those exact unclosed intervals prevent promoting eager registration into an
unconditional selector-entry snapshot. This is not a claim that those calls
invoke user code or mutate `MovieClip`; their preservation closure is simply
not established by this increment. Two small initial fragments read during
navigation (`183fa0`, 17 bytes; `0499f0`, 38 bytes) are explicitly unused
for a return claim. The next automatic-startup proof should close this specific
state interval, rather than substitute the all34 controlled fixture.

The existing VM initial-prototype API in `src/avm1/initial_prototype.rs`
already requires an available cache and supported data reads. This research
supports a later fresh-construction transition with explicit effective bindings;
it changes neither that API nor live adoption defaults.

## Evidence and provenance

Private root: `evidence/fresh-target-completion/` beneath the project data root.
The original verifier matched **18 selected ranges, 1,466 instructions /
5,348 instruction bytes, four pointer witnesses / 32 bytes, and 76 dispatch
bytes** to the pinned PE. It independently checked the `memset` thunk and PE
import identity, source hashes, range continuity and chained unwind metadata.
The two release fragments are combined before making a complete-body claim;
old raw-getter padding/overreads and the two new startup fragments are excluded.

Six target-free REA results completed with exit zero, untruncated output and
unchanged source hash. `new-reads.bundle.json` retains their canonical IDs;
`reset.rea.json` records export/reset with `retained_target=false`.
The first local checker had a Python keyword syntax error before analysis;
its source is retained, the corrected focused check passed, and no native
capture was repeated for it.

Independent ledgers are `ledger-audit/closure.json`,
`context-audit/receipt.json` and `refcount-audit/FINDINGS.md`. They preserve
earlier incomplete findings and the later explicit supplements. Final
`verification.json`, `receipt.json` and `MANIFEST.json` bind the selected
bytes, source identities, conditions and retained results. No recovered code,
binary data or extracted game assets enter git.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's `MarvelHeroesOmega.exe`, build `1.52.0.1700`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Exact owner/factory, ledger, cache, lookup, conversion, cleanup and import evidence |
| Existing project documents cited above; retained REA records individually named and hashed in private `source-ledger.json` and helper ledgers | Original project investigation; underlying native artifacts remain proprietary | Reuse established constructor, global/prototype binding and Value semantics without repeated captures |
| Current original `src/avm1/initial_prototype.rs` | Original project code | Describe the existing bounded API; no source modification or runtime oracle |
| Installed REA skill and evidence/native guides; REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling | Bounded static reads, complete Evidence and reset |
| Python standard library | PSF tooling; verifier and receipts are original project code | Independent PE/import/byte checks and private manifests |

No external VM or SDK implementation, shared configuration, native process,
live data, account, launcher or other project's source was changed.
