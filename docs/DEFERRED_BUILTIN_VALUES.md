# Native AS2 deferred builtin Values

**VERIFIED — bounded static evidence:** native Value tag 11 is a distinct
retained builtin-name token. It is neither Undefined nor an already-created
callable. Several operations preserve or consume it without resolving its
factory. Object/callable conversions resolve it using the **current
Environment's context**, not an Environment captured inside the token.

These distinctions matter when installing the actual global baseline. Replacing
every registered token with a callable changes conversions and equality;
resolving every property read adds factory side effects at the wrong time.
The verified stored inventory and factory registration remain in
[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md); the specific ExternalInterface
graph and cache mechanism remain in
[EXTERNAL_INTERFACE_BINDING.md](EXTERNAL_INTERFACE_BINDING.md).

All findings use the owner's private Win64 build `1.52.0.1700`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses are preferred-image virtual addresses. No original process was run.
These are native control-flow facts, not a completed Rust implementation or
retail startup result.

## Small value contract

The Value contains tag 11 and a retained interned-name node at payload `+8`.
The factory pointer and cached callable belong to a separate name-keyed map
in the global context. The token contains neither a factory pointer nor a
creation Environment.

Value copy construction `0x14018d770` preserves tag 11 and retains the same
name node: its tag-11 table entry at `0x14018d9c4` selects `0x14018d7c0`.
Assignment `0x14018d9d0` likewise selects the name-node retain branch
`0x14018da25` through the entry at `0x14018dc08`. Ordinary member flags are
separate from this Value tag and payload.

**INFERRED implementation design:** represent this as a distinct
`DeferredBuiltinName(InternedName)` Value. Keep each context's registry and
cached callable separate. A retained token stays a token when another access
materializes the same name; a subsequent namespace read may instead see the
callable that the factory published there. Unsupported factories should stop
at an actual resolution boundary, without inventing a missing registry entry
or executing the factory during every read.

## Scalar operations that do not resolve the token

| Operation on a retained tag-11 Value | Native result | Factory lookup |
| --- | --- | --- |
| Boolean conversion / `If` | true | none |
| `TypeOf` | String `"undefined"` | none |
| `ToString` | String `"[type Function]"` | none |
| Numeric conversion / `ToNumber` | positive zero, `+0.0` | none |

These results depend on the Value still being tag 11. They do not inspect
whether the token's name already has a cached callable. A resolved tag-8
function is a different Value state.

### Boolean and If

The complete Boolean helper is `0x14017d730..0x14017d87a`. After excluding the
other handled tags, `0x14017d871..0x14017d87a` compares the tag with 11 and
returns true. It does not inspect the payload, Environment, registry, or
factory on this branch.

The high-opcode dispatch indexes its table by `opcode - 0x81`. The `If`
opcode `0x9d` entry at `0x14020d590` selects `0x14020cf9d`. That case reads the
signed branch displacement, calls the Boolean helper at `0x14020cfc0`,
releases/pops the operand, and adds the displacement when the saved Boolean is
true. This joins the helper's tag-11 truthiness to actual `If` behavior.

### TypeOf

Raw opcode `0x44` selects `0x14020b014`. The case initializes builtin string
index `0x33`, dispatches only input tags at most 10, and sends tag 11 directly
to the default result. Static slot `0x142add478` binds that index to the literal
`undefined` at `0x1424fe918`. Helper `0x140188fb0` replaces the operand with
that String Value. This is the literal type-name string, not a lookup of a
script variable named `undefined`.

### String conversion

The string helper `0x1401a2920` indexes a tag table at `0x1401a2ea8`. Tag 11's
entry at `0x1401a2ed4` selects `0x1401a2dbb`, the same branch as tag 8. It reads
the current context's builtin string slot `+0x3d0`, bound by static slot
`0x142add578` to `"[type Function]"` at `0x14250bf08`, and retains that string.
There is no token resolution, `toString` member read, or user callback on this
branch. The helper still requires the current Environment to obtain its
builtin strings.

Raw `ToString` opcode `0x4b` selects `0x14020b1a8`, which calls wrapper
`0x1401a34a0`. Tag 11 takes the helper call above and the wrapper replaces the
operand with tag 5 and that string. Its older-version special case applies to
other input tags; it does not change this token branch.

### Numeric conversion

Numeric helper `0x1401a2f50` sends tag 11 through
`0x1401a3014 -> 0x1401a329b`. The tag is nonzero and not 10, so this reaches
`0x1401a2fca`, whose `xorpd` returns positive zero. No factory, token-name
inspection, or primitive callback occurs.

Raw `ToNumber` opcode `0x4a` selects `0x14020b197` and wrapper
`0x1401a3450`. It computes the number, releases the old token reference, and
replaces the operand with Double tag 3 containing that result.

## Primitive conversion can resolve without producing a primitive

The separate helper `0x1401c1f20` must not be replaced with the scalar
conversions above. For input tag 11, its branch at `0x1401c1f57` calls
`0x140197860` with the supplied current Environment.

- A non-null resolved FunctionRef is copied into the output as tag 8, including
  the reference's auxiliary frame ownership. The helper returns at
  `0x1401c22ed`; it does not continue into another conversion to obtain a
  string or number.
- If resolution returns no function object, the branch at `0x1401c2029` copies
  the original token to the output. It does not manufacture Undefined.

The input Value itself is not rewritten by this helper. Successful factory
effects on the context cache and published namespace are separate. Thus a
caller that accepts only primitive output tags can reject the resulting
tag 8 or retained tag 11 after resolution has already occurred.

## Equality and operand order

Raw `Equals2` (`0x49`) selects `0x14020b134` and helper `0x1401ca3a0`.
The case supplies the lower stack operand as the helper's left argument and
the top operand as its right argument, together with the current Environment.
It replaces the lower operand with the Boolean result and pops the top.
This operand mapping is directly checked; the native helper's asymmetric
token branches must not be silently reordered.

For valid supported operands, the token-specific contract is:

| Left operand | Right operand | Equals2 result | Token materialization |
| --- | --- | --- | --- |
| retained token | retained token, including a different name | true | none |
| retained token | any non-token | false | none |
| Undefined / Null / internal null-like tag 10 | retained token | false | none |
| ordinary object / display value / tag-8 callable | retained token | false | none |
| Boolean / Number / String | retained token | false on the normal return path | attempts resolution of the right token |

The first two rows follow the left-tag default at `0x1401ca832`, which compares
only the two raw tag bytes. The helper first handles the null-like set
`{0, 1, 10}`; a token does not belong to it. For the final row, String and
numeric left operands invoke the separate primitive helper on the right
token. Boolean left operands first become a temporary number and then take
that same comparison path. The recursive comparison is admitted only if the
converted right result has a primitive tag 1..5. The token conversion above
returns tag 8 or tag 11, so that test fails and comparison returns false.
Factory invocation and namespace/cache mutation can still have happened.
Exceptions or interrupted factories are outside this normal-return claim.

Raw `StrictEquals` (`0x66`) selects `0x14020c301`. Helper `0x140179e20`
accepts equal raw tags, plus the Int/Double numeric pair. On acceptance it
joins the same Equals2 implementation; otherwise it writes false. Therefore
two retained tokens are strictly equal regardless of their token names,
while a retained token and any non-token are strictly unequal without
materialization.

For two already-materialized tag-8 Function Values, the shared equality branch
at `0x1401ca788` compares their function-object pointers at Value `+8`.
Auxiliary frame pointers and reference ownership flags are not compared.
Neither equality operator first normalizes retained tokens to these objects.

## Read and invocation triggers

Ordinary returned Values and receiver conversions are separate operations:

| Operation | Token treatment |
| --- | --- |
| Ordinary stored-member copy | preserve tag 11 and retain its name |
| Successful unqualified GetVariable returning a token | preserve token |
| Terminal segment of qualified GetVariable | preserve token |
| Intermediate qualified segment used as an object receiver | resolve through current Env before the next member read |
| GetMember receiver is a token | resolve receiver |
| GetMember returns an ordinary stored token | preserve returned token |
| CallFunction name resolves to a token | resolve at callable conversion |
| Direct non-string token callee in CallFunction | preserve through copying, then resolve at callable conversion |
| Named CallMethod receiver is a token | resolve receiver, then perform the actual method lookup/call |
| Named CallMethod finds a token as the method Value | resolve that method at callable conversion |
| Undefined-name or empty-name CallMethod receiver is a token | resolve at callable conversion; this-binding remains a separate contract |
| NewObject constructor lookup returns a token | resolve callable before shared construction |
| NewMethod selected constructor is a token | resolve at its explicit callable conversion |

GetVariable wrapper `0x140202930` and normal member helper `0x140194950`
special-handle returned accessor tag 9 and resolver tag 12, not tag 11.
Generic getter `0x140197000` copies a found stored token unchanged. Qualified
resolver `0x1401fb6a0` copies terminal results, but when another segment requires
an object it calls `0x140199640` at `0x1401fbb7f`.

GetMember calls receiver converter `0x1401997f0` at `0x14020b270`. Its
non-display path uses `0x140199640`; tag 11 reaches `0x140197860`. An ordinary
returned property then remains subject to the separate normal-member helper.

Callable converter `0x140199880` receives the current Environment as its third
argument. CallFunction reaches it at `0x14020a7c3`; the supported unnamed
CallMethod branch reaches it at `0x14020bdff`. Receiver guard `0x14018e020`
copies a token unchanged and only has additional retention for display tag 7,
so it is not an earlier lazy-factory trigger. The named CallMethod branch
already documented for ExternalInterface resolves its receiver at
`0x14020bbc8`, then converts a found method Value through `0x140199880` at
`0x14020bc22`, again with the active Environment. A returned method token is
therefore resolved at this call boundary, rather than by ordinary member-read
machinery.

NewObject (`0x40`) first converts its constructor-name operand to a string,
then performs variable lookup. A **looked-up result** of tag 8 or 11 is
accepted and converted through `0x140199880` at `0x14020abb6`, before common
construction helper `0x1401a74c0`. This is not evidence that a bare token used
as the name operand is directly invoked. NewMethod and the shared allocator
also have explicit conversions when obtaining the selected callable,
prototype, and public constructor; the private constructor receipt records
these individually. Their allocation/prototype behavior remains governed by
the separate constructor studies.

## Environment and cache boundary

Interpreter entry `0x140208ef0` receives Env in RDX, saves it at `[rsp+0x48]`,
and the cited instruction routes explicitly reload/pass that active pointer.
The token itself cannot substitute its origin movie or factory-creation Env.

The corrected complete lazy helper is `0x140197860..0x14019795d`. With a
nonnull Environment and tag-11 input it obtains the token name and calls
`0x140194670` through Env `+0xe8`'s context. That resolver uses the context's
name-keyed registry at `+0x20`, returns an existing nonnull cached callable, or
calls the registered factory, reacquires the entry, and caches its returned
function object. Missing entries return null. A null factory result is not a
successful once-only cache and may be retried later.

The conversion returns a separate reference without changing the input token.
A specific factory can publish a callable into the namespace property, as
proved for ExternalInterface; a retained earlier token still uses the current
context's registry on its next explicit resolution.

Do not substitute an unimplemented known factory with a missing-entry result,
capture a fixed host root in every token, memoize null as permanent success,
or manufacture a callable for scalar conversion. Reentrant/exceptional
factories, unsupported contexts, arbitrary string-manager combinations,
missing host state, unsupported native methods, and generic accessor/resolver
execution remain explicit implementation boundaries.

## Evidence, validation, and provenance

All evidence remains private under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-deferred-builtin-values/final.bundle.json` and
  `combined-proof-receipt.json`: scalar conversions, actual If/ToNumber/
  ToString dispatch, primitive helper's tag-11 branch, source hashes and limits.
- `rea-deferred-builtin-equality/final.bundle.json` and
  `finding-summary.json`: complete equality core, operand order, StrictEquals,
  TypeOf, dispatch/string data and byte verification.
- `rea-deferred-builtin-materialization/finding-summary.json`, with embedded
  `constructor/finding-summary.json`: retained copy/read paths, active-Env
  call conversions, receiver guard, and constructor conversions. Reused
  records and new bounded records are individually identified there.
- `rea-external-interface/final.bundle.json`: existing registry/cache/object
  conversions. Use corrected complete lazy helper record
  `ev_13a35134d29edfd62e7aecc19583c957c3182e870d48cd99b5cf85505ec9f4ba`;
  the earlier 302-byte query also included unrelated trailing bytes and is not
  a 302-byte helper.

Selected native code, opcode/table bytes and string bindings were independently
checked against the pinned executable. Embedded jump tables were interpreted
as data; trailing padding or partial unrelated functions do not expand the
claimed semantic scope. New REA results completed without output truncation or
target change, and sessions were exported/reset. `git diff --check` was the
focused repository check. No Cargo test, native process, server, renderer or
gameplay test applies to this documentation-only increment.

| Source | Licence / status | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe` | Gazillion/Marvel proprietary; private owner installation | Exact tagged-value behavior, instruction dispatch and conversion paths for interoperability; binary output stays private. |
| REA 6.9.0 and local reverse-engineer-anything skill/native/connection guides | MIT toolkit and local workflow | Required supported evidence capture and session lifecycle. |
| Rizin provider | LGPL-3.0-only; analysis tool | Bounded instruction/data decoding. |
| Existing project global-baseline, ExternalInterface, qualified-variable, equality, constructor and value-copy receipts cited above | Original project analysis | Reuse established inventory/identity and resolve exact operation joins without importing outside implementation. |

No SDK source, third-party VM implementation, or language-convention guess was
used to supply the token's unusual scalar or equality behavior.
