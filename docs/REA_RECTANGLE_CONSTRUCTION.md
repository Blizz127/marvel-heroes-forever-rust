# Native zero-argument flash.geom.Rectangle construction

**VERIFIED static native contract:** the original Rectangle factory and shared
NewMethod path construct a distinct native Rectangle with own `x`, `y`, `width`
and `height` values of **Integer 0**, flags **0**. Its zero-argument callback
preserves all four initialized fields. Rectangle's native category, prototype
members and normal setter differ from Point and are proved separately here.
This study implements no geometry operations and runs no client or original script.

## Exact original source and current execution limit

The unchanged owned manager movie contains the following source sequence in
`MarvelUIManager`, definition 168589, body `168600+392`, flags `0x19`:

- Push 168738: register 1, `screenRect`, Double +0 argument count, `flash`.
- GetVariable 168756 resolves `flash` through the actual scope chain.
- GetMember 168762 reads `geom` from that result.
- **NewMethod 168768**, named `Rectangle`, has argument count 0.
- SetMember 168769 assigns the result to the receiver's `screenRect`.

Pool 163775 contains 293 entries; indices 10, 11, 13 and 14 are `flash`,
`geom`, `screenRect` and `Rectangle`. The sequence expresses
`this.screenRect = new flash.geom.Rectangle()`; `flash` is not explicitly
qualified with `_global`. Shadowing and scoped variable resolution cannot be
bypassed by forcing a global namespace.

**OBSERVED controlled execution limit:** the existing manager increment 8
receipt, frozen `c680c59`, completes 452 initializers and stops after 217 frame
instructions at the earlier `flash.geom` GetMember 168730. Neither the earlier
Point construction nor this Rectangle call was executed by that receipt.
Its supplied root name and raw state remain explicit diagnostic assumptions.

Private sources:

- `frontend-startup/MarvelGame-frontend_manager.gfx`, SHA-256
  `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
- `avm1-attached-manager-frame-action-increment8/receipt.json`, SHA-256
  `aa8ab18c6c6160a992df64afe625bf39ea7f3fa3ac40bfa725364666ca2c7b20`.

## Namespace, token and current context

The shared namespace and deferred-resolution contracts are established in
[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md) and
[REA_POINT_CONSTRUCTION.md](REA_POINT_CONSTRUCTION.md). Rectangle has its own
registration: bootstrap **`0x14020fdb7`** calls wrapper **`0x1401f96d0`**, with
factory **`0x1401f58b0`**, builtin offset **0x1b0**, index **15**. The checked
finite wrapper equivalence binds the registry insertion and initial namespace
write; ordinary registration does not invoke the factory.

Fresh successful bootstrap creates ordinary `flash` and `geom` namespace
objects with the actual Object prototype and marker-tag-10 `__proto__`, flags 3.
It adds no own public or hidden constructor metadata to those namespaces. Final
fresh flags are GLOBAL.flash 1, flash.geom 0 and geom.Rectangle 0. The Rectangle
member initially holds **native Value tag 11**, the deferred name token; its
factory and cached callable live separately in the context registry.

Actual scoped lookup must first resolve `flash`, then the current `geom` member,
then current `Rectangle`. Named NewMethod reads the current member at
`0x14020c0d2`, converts it with the active Environment at `0x14020c151`, and
calls shared New at `0x14020c16c`. A missing, replaced or intercepted member must
not silently select the canonical Rectangle factory.

Token resolution uses the active **Environment +0xe8 global context** and that
context's registry. It produces a separate FunctionRef, leaving a retained token
alias unchanged. Successful resolution caches its callable; subsequent resolution
reuses it. An ordinary token read does not materialize the graph. A previously
retained token alias resolves through the registry without rereading the current
namespace member. Missing registration and allocation failure stay distinct from
a successfully published graph.

## Rectangle factory and public graph

Complete factory **`0x1401f58b0..0x1401f5b88`** allocates callable C, installs
callback **`0x1401d6dc0`**, and uses cached Function.prototype, index 7. Its final
primary table is **`0x142514600`** and script interface **`0x142514430`**.
**No static method installer or static method table is present on this factory
path.** Point's three static methods cannot be transferred to Rectangle.

The factory allocates P, fetches cached Object.prototype, index 1, and calls
**`0x1401f5500`**. It registers P as cached Rectangle.prototype, **index 15**,
at `0x1401f5ad0`. Publication at **`0x1401f5b4a`** writes actual tag-8 C,
supplied flags 0, into the namespace saved at **context +0x38**, key Rectangle.
This cached geom identity comes from bootstrap. It is not a fresh traversal of
current GLOBAL.flash.geom and can diverge from visible namespaces after mutation.
An existing descriptor's flags are preserved by raw replacement; supplied flags
0 do not reset arbitrary prior flag changes.

P base **`0x1401e6600`** clears its auxiliary constructor references and joins
shared metadata at `0x1401e669e -> 0x1401b80f0`. Its concrete final primary table
is **`0x1425138e0`**, script interface **`0x1425161c0`**, auxiliary table
**`0x1425161b8`** at P+0x68. The primary table is not the script interface.

The public metadata is:

| Object | Own entry | Native representation | Flags |
| --- | --- | --- | --- |
| C | `__proto__` | marker tag 10, backed by internal Function prototype parent | 3 |
| C | `prototype` | actual object tag 6, P | 3 |
| P | `__proto__` | marker tag 10, backed by internal Object prototype parent | 3 |
| P | `constructor` | marker tag 10, backed by saved C FunctionRef | 3 |

The saved public reference is P+0x70, frame P+0x78, reference-flags byte P+0x80.
The separate hidden reference P+0x88/P+0x90/P+0x98 starts null; this graph does
not install an own P.`__constructor__`. An own non-marker public constructor
value takes precedence over the saved reference, so current metadata mutation
cannot be ignored.

Complete P constructor **`0x1401f5500..0x1401f589c`** installs **15 native
method descriptors, flags 6**, through `0x1401e4110`. It also installs seven
own descriptors: **left, top, right, bottom, topLeft, bottomRight and size**,
each **Integer 0, flags 2**, through explicit raw writes. These are stored
prototype descriptors, not the results of the instance's computed getters.
There is no Point-style single `length` installation.

The private checked member table terminates after row 15. Literal names are
read only from the owned PE and checked for distinctness and metadata collisions.
Successful fresh structural own-key counts are **C: 2, P: 24**. Ancestor members
and native special lookups are separate. No native allocation-count parity is
claimed, and the raw method table is not distributed in source or packages.

## Shared New and fresh four-field allocation

Shared New **`0x1401a74c0`** reads the called constructor's current public
prototype, then that prototype's current public constructor to select the
allocator. The canonical supported profile requires these live links to agree
with the recorded Rectangle graph and cached prototype identities. Accessors,
resolvers, altered prototypes or alternate allocators require their own support.
Zero arguments bypass the helper's argc==1 current GLOBAL.Object comparison;
that String-boxing special case is not a Rectangle New0 prerequisite.

C's primary +0x78 entry is **`0x1401d7110..0x1401d714c`**. It allocates
0x68 bytes from the active Environment context heap and invokes
**`0x1401d5c60..0x1401d5d78`** with the same Environment. Null allocation
returns null. The shared base `0x1401ccf40` clears the own hash, resolver, watch
pointer and flags, and links cached Object.prototype. Rectangle then installs
its concrete interface **`0x142511550`** and cached Rectangle.prototype,
index 15, through `0x14018edf0`.

The constructor writes four own ordinary data fields:

| Field | Constant | Value | Flags | Write path |
| --- | --- | --- | --- | --- |
| x | `0x142add820` | Integer 0 | 0 | raw interface +0x50 at `0x1401d5d05` |
| y | `0x142add840` | Integer 0 | 0 | literal-key raw helper `0x140182770` |
| width | `0x142add860` | Integer 0 | 0 | same helper |
| height | `0x142add880` | Integer 0 | 0 | same helper |

Each constant is a 32-byte native Value with tag **4** and zero payload.
The helper explicitly supplies flags 0, interns the original literal, forwards
the supplied Value to raw interface +0x50, and returns that call's result.
These are stored Values, not floating-point payload fields or virtual defaults.

After allocation shared New rebinds the originally selected public prototype,
installs the originally called constructor as own **tag-8 `__constructor__`,
flags 3**, and invokes that constructor. The instance retains its own
marker-tag-10 `__proto__`, flags 3. Thus the successful fresh instance has six
own keys: those two metadata entries and the four data fields. It has no own
public `constructor` installed by this chain.

## Callback and receiver preservation

Rectangle category leaf **`0x1400726e0`** returns **0x11 (decimal 17)**.
This category is distinct from native Value tags and from Point category 0x10.
Both Rectangle prototype and instance have category 0x11; their interface
+0x80 predicates distinguish prototype=true from instance=false.

The instance interface +0x78 Super predicate is false, so native invocation
wrapper `0x14018c5b0` forwards the original FnCall, receiver and Environment
directly to callback **`0x1401d6dc0..0x1401d710b`**. That callback reuses a
nonnull receiver only if its category is 0x11 and its prototype predicate is
false. Shared New supplies the fresh concrete Rectangle instance.

The callback stores that receiver as an ordinary object result. At
**`0x1401d6e72..0x1401d6e76`**, argc <= 0 jumps directly to cleanup at
`0x1401d70c6`; none of the four field writes executes. Shared New discards the
callback's temporary result and returns its allocated receiver, with all four
Integer-zero fields unchanged. Each successful construction has a distinct
instance identity; repeated deferred resolution reuses C/P instead of rebuilding
them.

The direct-call, mismatched/prototype receiver and positive-argument branches
remain outside this initial support profile. Do not implement direct Rectangle(),
argument conversion or constructor return overrides by extrapolation from New0.

## Native lookup and mutation boundaries

The concrete interfaces bind different normal lookup behavior:

| Role | Normal getter | Raw getter | Normal setter |
| --- | --- | --- | --- |
| Instance | `0x1401d5d80` | `0x140197000` | `0x1401c8480` |
| Prototype | `0x1401e8670` | `0x1401e7c40` | `0x1401c8480` |

Complete instance getter **`0x1401d5d80..0x1401d6145`** compares exact literal
bytes, including terminators, for the seven named computed properties above.
Their branches perform numeric work or construct other objects. Those behaviors
remain unsupported here. The all-miss route at `0x1401d611b..0x1401d6128`
passes the original name and active context to raw +0x58. Fresh `x`, `y`,
`width` and `height` therefore read their ordinary own stored Values.
Prototype normal getter is the shared raw-get thunk; its seven own Integer-zero
descriptors take that separate route. Do not return prototype placeholders from
an instance's computed getters.

Complete normal setter **`0x1401c8480..0x1401c87cb`**, used by both roles,
first rejects incoming flag bit **4**. It then compares the same seven exact
literal names and takes specialized mutation/coercion paths when matched.
Only the all-miss route at `0x1401c879f..0x1401c87aa` forwards the original
name, Value and flags to generic normal setter `0x1401a1020`. Fresh four-field
writes can use that proved ordinary route under existing version, property-flag,
no-watch, accessor/resolver and prototype guards. Copying Point's generic setter
without this Rectangle classification would admit incorrect behavior.

The initial construction implementation may keep all geometry methods and the
seven computed-property operations unsupported. Omitted method descriptors are
real native members, not evidence of absence. An incomplete prototype cannot
justify Undefined, unrestricted enumeration or a generic write to a native
special member. Mutation of public constructor/prototype links, metadata,
ancestors or namespaces remains gated unless fully implemented.

The Rust implementation must account for its real object/property/string
budgets, roll back unpublished graph state on failure, and publish registry,
namespace and canonical graph state consistently within its admitted profile.
It must not fabricate a successful object after allocation failure or claim
native allocation-count parity for a partial graph.

## Evidence and provenance

All paths below are private relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. The owned native binary
SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Six completed, exit-zero, untruncated, unchanged-artifact REA records are retained
in **`rea-rectangle-construction/complete.bundle.json`**. Selected retained
Point/shared-object records are joined by their full evidence IDs and bundle
hashes in the private verification receipt.

The original private checker validates selected instruction bytes and contiguous
spans against PE sections, raw tables and all four constant Values, native
interface/category joins, private literal-name distinctness, source bytes/pool
entries and the existing probe hash. It checks **3,798 instruction spans** and
**1,248 data bytes**. These are static validations, not runtime constructor tests.

The short first capture of `0x140182770` ends inside an instruction; only its
valid prefix through `0x1401827f1` is used. A separately complete 24-byte tail
joins it through `0x140182809`. No clipped instruction is positive proof.
REA was exported and the target-free investigation ledger cleared afterward.

Private receipts:

- `rea-rectangle-construction/namespace-source-review.json`: SHA-256
  `20b3ae287526ffb9ccb9da57016fbffa96e8096015dd533de58f52a4f2211f0d`.
- `rea-rectangle-construction/shared-graph-review.json`: SHA-256
  `cf95f28641055d07ba8a0d102a667bf75e598f112a86ecf80e9968793ac22191`.
- `rea-rectangle-construction/verification.json`: SHA-256
  `85102e75375831dc691e1a78aec6c5c79b4f7ca3996846d0f68479408c89d35e`.
  It includes the two reviews and private member profile.

Sources: Blizz's private build-1.52.0.1700 executable and manager movie
(proprietary, compatibility/interoperability study); original project
AS2_GLOBAL_BASELINE and REA_POINT_CONSTRUCTION analyses and their cited retained
REA records (original analysis; underlying bytes remain private). REA/Rizin are
analysis tools. No outside implementation code, geometry library or inferred
public API inventory was used.
