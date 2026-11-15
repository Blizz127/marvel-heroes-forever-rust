# Object prototype primitive callbacks

**VERIFIED — bounded static evidence from the owner's pinned executable.**
The actual Object prototype installs `toString` at `0x140192760` and
`valueOf` at `0x140197410`. These are distinct callbacks and distinct function
objects from the similarly named Function prototype methods. Their behavior
depends on the native receiver interface category; neither callback supports
an unconditional JavaScript-style conversion of every VM value.

The narrow implementation-ready profile is the existing concrete ordinary
native/function identities and cached Object/Function prototypes. A separate
allocation witness can admit the proved ordinary Object instances. Display
receivers, arbitrary `Plain` allocations and primitive-to-interface conversion
remain outside that profile. No native invocation, original script replay or
VM implementation is part of this study.

## Installed identity and invocation boundary

The finite Object prototype method table `0x14250e750` places these callbacks
at entries 6 and 7. The fresh bootstrap installs each as a separate tag-8
native function with descriptor flags **1**, null auxiliary reference and
reference flags zero. Their actual native interface is `0x142514430`; their
initial linked parent is null. These facts reuse the sealed
[fresh global bootstrap](FRESH_GLOBAL_BOOTSTRAP.md) producer/own-member ledger.
This study rechecks both table entries against the pinned PE.

The corresponding Function callbacks are `0x14018c7c0` and `0x140196710`,
as documented in [Function primitives](REA_FUNCTION_PRIMITIVES.md). The
earlier distinction remains correct. Sharing the text `toString` or `valueOf`
does not make two installed functions interchangeable.

The common native invocation wrapper's ordinary, non-Super branch forwards
the original `FnCall` unchanged. The concrete ordinary Native, ScriptFunction,
Object prototype, Function prototype and ordinary Object instance interfaces
all bind the false `+0x78` predicate. The prior Function receipt already
closes named CallMethod's fresh tag-0 result slot and its `FnCall+8` mapping.
Use that ordinary direct-method boundary, with no getter/resolver invocation
or reentry before the callback. Arbitrary Super interfaces are excluded.

Both callbacks require a valid nonnull `FnCall.this` interface. `toString`
dereferences it immediately; `valueOf` forwards it to a converter that does
the same. Unlike Function's `valueOf`, there is **no null-interface guard**
at this entry. This is a compatibility precondition, not a VM crash policy:
unrepresented or invalid receivers should return explicit unsupported errors.

## Object.prototype.toString

The complete callback occupies `[0x140192760, 0x1401928d6)`, 374 bytes.
It does not read argc or argument values. It selects the following branches
through the receiver interface's category getter at virtual slot `+0x10`:

| Native receiver category | Selected behavior |
| --- | --- |
| `0x17` (23) | Return the builtin String `[type Function]`. |
| `2..5` inclusive | Follow the display owner/handle path described below; excluded from the initial profile. |
| Every other category | Return the builtin String `[object Object]`. |

The category-23 test occurs at `0x14019277c`. Its return path reads the
current `FnCall.Env` at `+0x30`, then `Env+0xe8`, context `+0x28`, selected
object `+0x18`, and builtin-root slot **`+0x3d0`**. The ordinary fallback
starts at `0x140192891` and follows the same context chain to slot
**`+0x3e0`**. Both return native tag **5**, store the existing string node,
and increment its reference count at node `+0x18`.

The complete retained builtin initializer/binder connects root `+0x138`
to the static name table at `0x142add2e0` in entry order. Consequently:

- Index 83 gives root `+0x3d0`; static pointer `0x142add578` selects
  `[type Function]` at `0x14250bf08`.
- Index 85 gives root `+0x3e0`; static pointer `0x142add588` selects
  `[object Object]` at `0x14250bee8`.

Each result is exactly **15 ASCII/UTF-16 units**, excluding the native NUL
terminator. The nearby `[type Object]` literal is not this fallback result.
No function name, source code, public `constructor` property, public prototype
chain or movie-version byte selects these strings. The callback performs no
member lookup or user coercion on the admitted categories. It queries the
concrete native category getter; admitting an arbitrary virtual implementation
would require its own proof.

Before writing a result whose prior tag is at least 5, the callback invokes
`0x1401887a0` for old-value cleanup. A fresh tag-0 CallMethod result skips
that cleanup. The string-node retain still occurs. The Rust profile may use
its existing string budget, but that is an engineering limit; the native
operation returns an already interned node rather than allocating this text.

**Context requirement:** a real active Environment with the associated
builtin context must be supplied. The receiver does not provide a substitute
Environment. An explicit VM binding to its one builtin context is a suitable
bounded witness; direct logical calls with no active target should remain
unsupported. Extra otherwise-valid arguments are ignored without conversion.

### Excluded display branch

The unsigned category-minus-2 test at `0x1401927be..0x1401927ca` selects
categories 2 through 5. After another category check, this path reads the
owner at interface `+0x18`, then its handle at owner `+0x70`, calling
`0x14009e770` if the owner has no existing handle. It constructs and retains
a temporary native tag-7 Value. With a nonnull handle it passes a context
derived from the current `Env+0xe0` chain to `0x14009a540`; success selects
the string node at handle `+0x18`. Failure or no handle selects builtin-root
`+0x138`. It writes tag 5, retains the selected node and cleans the temporary
tag-7 Value at `0x14019287c`.

This establishes that display `toString` has owner, handle creation/resolution
and cleanup effects. It does not establish a VM path-string algorithm or
authorize replacing that branch with `[object Object]`. Concrete display
identity, handle retirement, the resolution helper, empty-node selection and
allocator/cleanup effects remain separate dependencies. Neither MovieClip
nor TextField is admitted by this study's initial implementation profile.

## Object.prototype.valueOf

The complete 13-byte leaf `[0x140197410, 0x14019741d)` loads
`FnCall.this` into RDX, loads the result pointer from `FnCall+8` into RCX,
and tail-jumps to **`0x1401937f0`**. It does not read argc, argument values,
or Environment. Adjacent `int3` bytes are padding, not callback instructions.

The complete retained converter `[0x1401937f0, 0x140193895)` first queries
the supplied interface category:

1. Categories **2..5** read interface `+0x18`, obtain or create the owner's
   `+0x70` handle, and write a native **tag-7** result. An existing identical
   tag-7 destination is preserved. Otherwise it cleans the old destination,
   writes the handle and retains it. This branch remains excluded.
2. Other categories are checked again. Categories **6..44** inclusive
   subtract `0x20` from the interface to recover the object base. Every
   category outside that range supplies a null object base instead.
3. The recovered base, or null, is passed to **`0x1401936f0`**.

The latter converter is the same complete body already proved for Function's
`valueOf`, reached here through a different category/owner-aware entry:

- Category **23** invokes the actual function-reference getter at `+0x60`.
  The known ordinary Native and ScriptFunction tables both select
  `0x14018c580`, which returns the same underlying function identity with
  auxiliary association null and reference flags zero. Into a fresh tag-0
  result, `0x14018de50` writes a fresh **tag-8** reference carrying those
  values.
- Every other object category, including the proved category-6 receivers,
  returns **tag 6** containing that same object base. A null base returns
  tag 6 with a null payload, not tag-1 Null or tag-0 Undefined.

The category-23 result is not a copy of the original caller's function Value
wrapper. Existing same-function tag-8 destinations can take a shortcut that
retains their prior auxiliary fields; the initial profile therefore relies
on the proved fresh result producer. Top-level ScriptFunction scopes
`[target, global]` remain a conservative admission guard where the VM does
not represent auxiliary FunctionRef associations. The callback does not
modify the underlying script's code or captured target.

No Environment or argument conversion is performed on the admitted
category-6/category-23 branches. Reference retention/release and the
existing collector helper remain real native effects. Valid ownership and
the previously stated no destructive cleanup/reentry profile still apply;
this is not a blanket claim that the converter has no side effects.

## Concrete receiver admission

The retained producer/table joins establish:

| Concrete interface | Category | Initial result profile |
| --- | --- | --- |
| Ordinary native callbacks `0x142514430` | 23 | Function string; fresh function reference for valueOf. |
| Actual ScriptFunction `0x142513090` | 23 | Function string; valueOf needs the auxiliary-association guard. |
| Cached Object prototype `0x142513cd0` | 6 | Object string; same Object identity. |
| Cached Function prototype `0x142515be0` | 6 | Object string; same Object identity. |
| Proved ordinary Object allocation `0x1425185c0` | 6 | Same category-6 behavior, if allocation provenance is preserved. |

The ordinary Object allocation is separately grounded: original Object's
primary table slot `+0x78` calls `0x1401cd330`, which allocates `0x68` bytes
and invokes `0x1401ccf40`. That constructor installs interface
`0x1425185c0` at base `+0x20`; its category slot selects the complete
read-only `0x1403cecd0` leaf returning 6. The existing InitObject and
zero-argument New Object receipts bind the source operations to that producer.
The precise retained joins are
`rea-enumeration-callback/initobject-finding-summary.json` and
`rea-deferred-builtin-materialization/constructor/object-newobject-zero-arg.json`;
their hashes are dependencies in this study's verifier receipt.

This does **not** establish a native category for every Rust `Kind::Plain`.
The current kind also represents global/root objects, activation frames and
other allocation paths. An immutable origin witness on proved literal/New0
allocations would support broader admission. Writable `__proto__` or
`constructor` data cannot establish the witness; the enumeration hash profile
is also not durable allocation provenance. Moreover, Object's argc-0 callback
can reuse a supplied receiver: passing through that callback alone does not
certify an arbitrary borrowed receiver as category 6.

A private immutable category witness set only after completed
`object_literal()` construction can express this allocation profile, including
the existing zero-argument New Object and ordinary host-result paths that
use that same constructor. It should default false for other allocation
paths and survive ordinary property mutation. That witness is an explicit
Rust representation invariant, not an observation of a native heap instance.

An initial implementation can reuse the already proved Function primitive
native-kind whitelist, add the two distinct Object primitive method objects,
and admit exact cached OP/FP identities. For `toString`, an actual ScriptFunction
category is sufficient even when it has captured auxiliary state, because no
FunctionRef conversion occurs on that branch. For `valueOf`, preserve the
existing top-level-scope/fresh-result guards. Specialized constructors/methods,
deferred values, arbitrary Plain, primitives, display values, invalid handles
and null interfaces should reject until their particular bridges are proved.

## Evidence, verification and provenance

All native material remains under the private directory
`~/.local/share/marvel-heroes-forever-rust/evidence/rea-object-primitives/`.
The only repository output of this study is this document. The source is the
owner's proprietary retail `MarvelHeroesOmega.exe`, used for compatibility
study; no external VM implementation or borrowed implementation code was read.

Pinned executable SHA-256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

Three bounded target-free REA calls are exported in `complete.bundle.json`
and an identical `reset.bundle.json`. The native executable was not launched;
REA reported unchanged input, complete commands and untruncated output. The
session was reset after export with three records cleared and no retained
active target. A navigation read of adjacent unrelated strings was excluded
from the semantic proof; the actual fallback pointer was followed explicitly.

The private verifier checks **427 native instruction spans / 1,527 bytes**,
**600 retained REA data bytes**, and independent concrete table-pointer joins
against the pinned PE. It retains selected instruction records and dependency
hashes. Existing Function receipts supply the fresh named-call destination,
ordinary wrapper forwarding and reference-ownership limits without alteration.

Primary retained records:

- New `ev_8ce112d0fd47fc454f39d1a9c0ea0261aa806988451e38cbe3a42cc228baecc3`:
  complete Object callbacks.
- New `ev_148db9508a68b50b97dcba5e7a4f5a517920b3c13967fc1d36e02501bf87b4da`
  and `ev_8cc6f72c6e4499209ab12e409f3e2632a27e9c8cc01c37962bf5b00c5cef8ad4`:
  selected builtin pointer and exact Object string bytes.
- `rea-function-calls/function-binding.bundle.json`, `ev_ceb0...`:
  complete interface-to-Value and object-to-Value converters, category leaves
  and builtin initializer.
- `rea-script-function-conversion/getter.bundle.json`, `ev_d9be...`, and
  `rea-function-calls/function-context.bundle.json`, `ev_9f5b...`:
  exact function-reference getter and destination setter.
- `rea-enumeration-callback/initobject-final.bundle.json`, `ev_8123...`
  and `ev_e82e...`: ordinary Object constructor, allocator and concrete tables.
- `fresh-global-bootstrap/prototype-producers/`: finite callback table,
  original descriptor flags, installed identities and OP/FP producer joins.

`verification.json` and `finding-summary.json` seal the selected proof and
its limits. These static results do not show that an original manager script
has reached or executed either Object callback, nor establish general object
coercion or full VM parity.

Verification SHA-256:
`197718d1b02fdd79e1407e2c8fc6b2672f4b3ee69448015ecfcea8ec20552fa6`.
