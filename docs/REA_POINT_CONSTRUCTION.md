# Native zero-argument flash.geom.Point construction

**VERIFIED static native contract:** the original Point deferred factory and
zero-argument construction path allocate a distinct native Point instance with
own `x` and `y` values of **Integer 0**, property flags 0. The callback preserves
those initial values for zero arguments. This is an allocation and metadata
contract, not a Point method implementation or a retail runtime observation.

**OBSERVED controlled frontier:** frozen `c680c59` completes 452 initializers
and reaches 217 manager-frame instructions before the minimal global model
stops at `GetMember 168730` for `flash.geom`. The following original
`NewMethod 168736` is `Point` with argument count 0; its result is assigned to
`movieSize` at 168737. Construction has not run in that receipt. The enclosing
`MarvelUIManager` constructor is definition 168589, body `168600+392`, flags
`0x19`, in stream `163775+17120`, tag 475. Later Rectangle/Point constructions
remain unexecuted and are outside this proof.

The private probe is
`evidence/avm1-attached-manager-frame-action-increment8/receipt.json`, SHA-256
`aa8ab18c6c6160a992df64afe625bf39ea7f3fa3ac40bfa725364666ca2c7b20`.
The owned manager movie SHA-256 is
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Its controlled root name and supplied raw state remain explicit assumptions.

## Namespace and deferred identity

The bootstrap contract is grounded in [AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md)
and the private `rea-point-construction/namespace-deferred-review.json`.
Fresh successful namespace creation uses ordinary objects with the supplied
Object prototype. Their initial own metadata is `__proto__`, marker tag 10,
flags 3; the helper does not add own `constructor` or `__constructor__`.

The bootstrap calls `0x1401d0850` for the `flash.geom` path at
`0x14020fd70`, stores the returned geom identity in **global context +0x38**
at `0x14020fd80`, and supplies it to Point registration `0x1401f9580` at
`0x14020fda6`. Fresh member flags are GLOBAL.flash 1 after the final global
visitor, flash.geom 0 and geom.Point 0. The visitor does not recursively change
namespace flags. Existing namespace components are reused after raw lookup
and object conversion; a found nonobject must not be repaired as a miss.

Point registration uses builtin name offset `0x1a8`, index 14, and factory
**`0x1401f5210`**. The private 334-byte finite wrapper equivalence binds it to
the retained canonical registration body. The registered namespace value is
**tag 11, a deferred name token**, not a callable and not a built graph.
An existing factory-registry entry skips registration and namespace writing.

Resolver `0x140197860` uses the **active Environment +0xe8 global context**.
The registered factory runs once per successful cached registry entry; repeated
resolution returns that callable. It produces a separate FunctionRef and leaves
a previously retained token alias unchanged. A token read alone does not call
the factory. Missing registration and null factory return are separate failure
paths; the latter does not fabricate a completed cached graph.

Named NewMethod reads the **current receiver member** at `0x14020c0d2`, converts
that value with the current Environment at `0x14020c151`, then calls shared
construction at `0x14020c16c`. Thus the current GLOBAL.flash, flash.geom and
geom.Point lookup results cannot be bypassed with a remembered factory name.
A retained token alias instead resolves its name against the active context's
registry without rereading a namespace property.

## Factory and graph

Complete `0x1401f5210..0x1401f54ff` takes an output FunctionRef and a global
context. It allocates the callable C, installs callback **`0x1401d5930`**,
uses cached **Function.prototype, index 7**, and creates three static native
method descriptors with flags **7** through `0x1401e9bd0`. It allocates P and
passes cached **Object.prototype, index 1**, and the C reference to
`0x1401f50b0`. P is registered as cached Point.prototype, **index 14**, at
`0x1401f5447`.

The final publication at **`0x1401f54c1`** writes tag-8 C to the namespace
stored in **context +0x38**, key Point, supplied flags 0. This is the bootstrap's
cached namespace identity, not a fresh traversal of current GLOBAL.flash.geom.
Those identities can diverge after mutation. The generic raw setter preserves
existing flags on replacement; supplied flags 0 are not proof of resetting
changed descriptor flags. The initial bounded profile therefore requires
consistent current and cached namespaces and untouched Point registration.

P constructor `0x1401f50b0` calls base `0x1401e6ea0` and shared metadata helper
`0x1401b80f0`. It creates seven native method descriptors with flags **6** via
`0x1401e4110`, then an own **length = Integer 0, flags 2** at `0x1401f5168`.
These method objects are real native graph members. An implementation omitting
them must keep reads/calls/enumeration that encounter the omitted graph explicitly
unsupported. A missing Rust entry is not native Undefined or absence. The raw
method inventory remains private; this document does not distribute its table.

The metadata join stores C's FunctionRef in auxiliary P+0x68, specifically
public reference P+0x70, frame P+0x78 and reference-flags byte P+0x80. It installs
P's own public `constructor` as marker tag 10, flags 3; the raw getter exposes the
saved C reference. C's own `prototype` is an actual tag-6 P value, flags 3.
Hidden reference fields P+0x88/P+0x90/P+0x98 start null, and no own
`__constructor__` is installed on P. These metadata flags differ from method
flags and FunctionRef flags.

The fresh successful slice therefore has five own keys on C (`__proto__`,
`prototype` and three static methods), ten on P (`__proto__`, `constructor`,
`length` and seven methods), and four on a constructed instance (`__proto__`,
`__constructor__`, `x`, `y`). The private checker binds table pointers and decodes
their owned literal names to verify distinctness and lack of metadata collisions.
These are structural own-key counts, not full visible-property or allocation
counts. Ancestor members and special raw getters remain separate.

The concrete interfaces distinguish the prototype from an instance:

| Role | Native category | Normal getter | Raw getter | Prototype predicate, interface +0x80 |
| --- | --- | --- | --- | --- |
| Fresh Point instance | `0x10` | `0x1401c83e0` | `0x140197000` | false |
| Point.prototype | `0x10` | `0x1401e8670` | `0x1401e7c40` | true |

The predicate is a virtual function result, **not an object byte at +0x80**.
Both roles carry ordinary object references; the role must remain distinguishable.

## Current public metadata selects construction

Shared New helper `0x1401a74c0` reads the selected callable's current public
`prototype` at `0x1401a76ac`. If absent it uses cached Object.prototype. It
then reads the selected prototype's public `constructor` at `0x1401a7774` and
may use that callable's allocator. The bounded canonical Point profile requires
all these public links to agree with the recorded factory graph; it must reject
accessors, resolvers, replacement constructors or changed prototypes that it
cannot execute accurately.

The **argc == 1** path at `0x1401a7513` performs the special comparison with
current GLOBAL.Object. **Zero arguments bypass that comparison.** The String
boxing rule must not be presented as a native requirement for this zero-argument
Point route. Cached Object/Function/Point prototype identities and normal context
admission still matter to the actual allocation and metadata operations.

The selected allocator is invoked at `0x1401a79ae`, with the same Environment.
After allocation, the helper applies the originally selected public prototype
through interface +0x68, installs the **original called constructor** as hidden
`__constructor__` through `0x14018ec90`, and invokes that original constructor
at `0x1401a7a48`. Its temporary callback result starts Undefined and is discarded;
normal success returns the allocated receiver at `0x1401a7b42`. A callback return
value cannot replace this instance in the supported NewMethod path.

## Fresh instance and zero-argument callback

C's primary table `0x142516130`, slot +0x78, points to allocator
**`0x1401d5b60..0x1401d5b9c`**. It allocates 0x68 bytes from the active
Environment's context heap, then enters **`0x1401d4cf0`** with that same
Environment. Allocation failure returns null and is not a successful Point.

The base `0x1401ccf40..0x1401cd009` starts an empty own hash, clears resolver,
watch pointer and flags, and sets cached Object.prototype. The concrete Point
constructor replaces its interface/category and uses cached Point.prototype,
index 14, through `0x14018edf0`. This helper creates marker-tag-10
`__proto__`, flags 3, only when the prior parent is null, and maintains the
internal parent link. Rebinding an existing parent does not add another key.

At `0x1401d4d75` and `0x1401d4dac`, the constructor writes builtin keys `x`
and `y` through the raw setter with flags 0. Their native constants at
`0x142add7e0` and `0x142add800` are each a 32-byte Value with **tag 4 and zero
payload**. They are Integer 0, not floating-point Number 0. The key slots and
literal bytes are independently bound to the owned PE. Subsequent shared New
installs actual tag-8 `__constructor__`, flags 3; it is not a marker value.
No own public `constructor` is added to the instance by this chain.

The callable's primary +0x50 dispatch is native wrapper `0x14018c5b0`.
The instance interface +0x78 returns false, so that wrapper forwards the original
FnCall, including receiver and Environment, directly to C's callback at
`0x14018c6c3`. No Super receiver substitution occurs on this path.

Callback **`0x1401d5930..0x1401d5b58`** reuses a nonnull receiver only when
its interface category is `0x10` and its prototype predicate is false. Common
New supplies exactly that fresh Point instance. The callback stores an ordinary
object result, then the argc <= 0 branch at `0x1401d59d1` jumps to cleanup:
**no x/y write occurs**. Therefore successful zero-argument NewMethod returns
the fresh initialized receiver, with x and y unchanged and a distinct identity
for each successful allocation.

The callback's direct-call or invalid/prototype-receiver fallback allocates its
own object, and positive argc takes separate value-copy branches. These are
observed branches but remain outside the initial zero-argument construction
profile. Do not implement direct Point(), conversion, or nonzero arguments by
assuming this profile covers them.

## Member access and conservative implementation boundary

Instance normal getter `0x1401c83e0..0x1401c8478` recognizes builtin `length`
and computes it through another helper plus square root. Every other name tail
calls its raw getter. Thus fresh ordinary `x` and `y` reads return their stored
Values; they are not virtual numeric payload fields. Computed length and method
execution remain unsupported until their own input/coercion contracts are
implemented. Prototype normal getter `0x1401e8670` directly forwards to its raw
prototype getter, so its own length descriptor is a separate path.

Prototype raw getter `0x1401e7c40` handles constructor metadata: an own ordinary
non-marker value takes precedence; otherwise public constructor uses the saved
FunctionRef. The separate hidden constructor reference starts null and can
inherit through the actual parent. Do not substitute public and hidden constructor
slots, or invent an own hidden slot on P.

Both concrete interfaces bind normal setter `0x1401a1020` and raw setter
`0x1401e3b20`. These preserve existing flag/accessor/watch/prototype semantics.
Fresh x/y are ordinary own data, with no installed watch. A bounded implementation
may support those exact ordinary reads/writes using its already verified generic
object machinery, while gating metadata mutation, special members, changed
ancestors, watches/resolvers/accessors and omitted methods. Other properties must
not receive automatic Undefined from an incomplete native prototype.

Construction must preflight its actual Rust object/property/string budgets and
roll back unpublished state after failure. The native graph contains more objects
than a partial implementation; no native allocation-count parity is claimed.
Keep deferred registry publication, canonical graph caching and namespace write
atomic at the supported boundary. Treat failed allocations and partial factory
construction as failures, not successful calls with fabricated objects.

## Evidence, validation and provenance

All evidence paths are private relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`.
The owned binary SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
New REA records are retained in `rea-point-construction/complete.bundle.json`.
The original private checker `verify-evidence.py` binds selected disassembly,
tables and literals back to the PE sections and requires completed, exit-zero,
untruncated, unchanged-artifact records for positive proof. Validation checked **3,193 contiguous instruction spans**, **944 selected table
and Value bytes**, and selected literal bindings against the owned PE. It checked
the original probe/source hashes and private method-name noncollision assertions.
This is a static check; it does not execute the constructor.

- `rea-point-construction/verification.json`: SHA-256
  `091b90e5310925301d80847204cb74a5cd87b23ce74233bdaec2ca839d533100`.
- `rea-point-construction/namespace-deferred-review.json`: SHA-256
  `92c7b7aec3e5df6f9bd5c0fdff37ae99cff0d812e1f7f47e86341ea096dd25fe`.
- `rea-point-construction/shared-graph-helper-review.json`: SHA-256
  `640fae34530f5be9473794cd6176907ea34a009b7e3d63278b1c8a63e26ff794`.

The verification receipt lists complete evidence IDs, ranges, bundle hashes and
component hashes. New factory, allocator, callback and Point getter records join
retained namespace, NewMethod, shared graph and ordinary member evidence.

The timed-out `ev_6fac95ce...` has unknown completion and is not positive proof;
complete `ev_e8a3b130...` and `ev_c4c43a65...` supersede its selected ranges.
The short base-constructor read in `ev_47f7ae9f...` ends inside an instruction and
is not used for that body; the complete retained `ev_8123b7df...` supplies it.
The two predicate leaves in `ev_47f7ae9f...` are complete and used independently.
No native executable, original frontend script, network or host callback was run
by this study. REA was exported and its target-free ledger cleared afterward.

Sources: Blizz's private build-1.52.0.1700 executable and manager movie
(proprietary, compatibility study only); original repository analyses
AS2_GLOBAL_BASELINE, DEFERRED_BUILTIN_VALUES, REA_MOVIECLIP_CONSTRUCTOR and
STRING_NATIVE_METHODS, with their private retained REA records (original project
analysis; underlying artifact remains private). REA/Rizin are analysis tools,
not implementation sources. No outside implementation code was copied.
