# Native String boxing and ASCII `toUpperCase`

**VERIFIED, static evidence:** the original `LocalizedObject.updateLabelText`
call at movie offset `94251` can be supported for a NUL-free ASCII primitive
string through the native String constructor, a temporary typed wrapper, and
the actual prototype method. The supported mapping is `a`–`z` to `A`–`Z`;
other ASCII values remain unchanged. This is established from this client's
mapping table, not a modern Unicode library. No original client or native
callback was executed, and this document does not claim retail UI parity.

The call route depends on mutable public globals and prototype properties.
Recognizing the spelling `toUpperCase` alone is insufficient. Unsupported
mutations, coercions, receiver categories and Unicode input must stay explicit
boundaries until their native paths are implemented.

## Artifact and original call

The private executable is
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
build `1.52.0.1700`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses below are virtual addresses in that image.

**OBSERVED, original asset:**
`evidence/frontend-startup/MarvelGame-frontend_manager.gfx`, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`,
contains definition `94103+189`, body `94114+178`, body SHA-256
`0165651ddc0d9d9358fb2e44c590e599dab0b9a51bd967ef552ccc98c4233e76`.
The original branch tests `m_ForceAllCaps` at `94219–94220`. Its true branch
pushes zero arguments, register 2 and constant-pool entry 79 (`toUpperCase`)
at `94235`, calls at `94251`, then stores the result into `htmlText` at `94252`.
The private source receipts are
`evidence/avm1-reference/textfield-null-equality.json` and
`evidence/rea-string-primitive-call/finding-summary.json`.
Source bytes do not by themselves prove the runtime register's type, its text,
the current environment, or the saved instance's flag. Those remain inputs of
the actual attached-label probe; a controlled false-flag test skips this branch.

## Binding and object graph

**VERIFIED:** existing native bootstrap registration associates builtin index
3 (`String`, context name slot `+0x150`) with deferred wrapper `0x1401f8470`
and factory `0x1401f6ce0`. Raw reads preserve the initial tag-11 token.
Resolution uses the active environment's context and existing factory/cache
route documented in [DEFERRED_BUILTIN_VALUES.md](DEFERRED_BUILTIN_VALUES.md).
It is not a process-global or code-origin cache.

| Native object | Identity and linked parent | Relevant metadata |
| --- | --- | --- |
| String callable | Allocation `0x70`; primary `0x142516630`, interface `0x142514430`; native category 23; parent cached Function.prototype (index 7) | Callback `0x1401bf2d0`; returned FunctionRef has null auxiliary frame and reference flags 0 |
| String.prototype | Allocation `0xb0`; constructor `0x1401e7050`; primary `0x142516530`, interface `0x142516580`; category 8; parent cached Object.prototype (index 1) | Public constructor metadata references the String callable; reciprocal callable `.prototype` and prototype `.constructor` descriptors have flags 3 |
| `toUpperCase` method | Fresh native function stored on String.prototype; primary `0x1425150c0`, interface `0x142514430`; category 23; parent cached Function.prototype | Table row `0x142add960` binds the exact name to callback `0x14018d460`; stored Value tag 8, descriptor flags 1, null auxiliary frame and reference flags 0 |
| Temporary boxed instance | Allocation `0x70`; constructor `0x1401d79a0`; primary `0x142511860`, interface `0x1425117b0`; category 8 | Payload StringNode at base `+0x68`; Super and prototype predicates are false; final parent follows the selected public String callable `.prototype` |

Descriptor flags and FunctionRef flags are different fields. A method's
descriptor flags 1 mean DontEnum under the separately verified property-flags
contract. They do not change the FunctionRef flags to 1.

Factory method installation uses `0x1401e4110`, iterating 16-byte name/callback
rows and creating actual function objects. There are 14 prototype method rows
before the terminator, all supplied descriptor flags 1. This study grounds
`toUpperCase`; the other callbacks are present native members whose behavior
is unsupported, not absent properties. Their complete extracted inventory
stays in private evidence rather than a packaged native table.

The callable also has a separate static method table. Its single entry is
`fromCharCode` (callback `0x1401bf1c0`), followed by a null row. Factory call
`0x1401f6e89` invokes the distinct installer `0x1401e9bd0` with flags 7;
that helper passes those descriptor flags through the raw setter. On the
fresh callable, the own-key sequence is `__proto__` (flags 3), `fromCharCode`
(flags 7), then `prototype` (flags 3). `fromCharCode` behavior is unsupported;
its presence and flags are established. This method must not vanish from the
native baseline merely because the initial UI branch does not call it.

String.prototype starts with an empty string payload, but the factory replaces
it with the native literal `[type Object]`. That prototype payload must not be
used as a freshly boxed argument's contents. The prototype's own metadata is
`__proto__` and public `constructor`, both marker tag 10 / flags 3; it has no
own `length` marker and no own `__constructor__` marker. Its 14 methods bring
the fresh own-key count to 16. A special getter can expose constructor
metadata without making an absent hash entry present.

At factory completion the current global's `String` descriptor receives the
materialized tag-8 callable through a raw setter supplied flags 0. This proves
the supplied flags, not that an existing descriptor's flags are reset; value
replacement must preserve the verified ordinary setter semantics.

## Actual primitive-string named call

**VERIFIED:** the named CallMethod path first finds that ordinary object
conversion of a primitive tag-5 String returns no object. At `0x14020ba38`
it calls `0x14018f270` with the active Environment and receiver stack index 1.
That helper selects builtin index 3, takes **current** context `+0x60` GLOBAL,
and calls named construction `0x1401a7c50` with one argument borrowed from the
original operand stack. It does not directly return a cached String prototype.

Named construction invokes the current global's normal member getter for
`String`. A returned tag-8 function is accepted; tag 11 resolves through
`0x140197860` with the active Environment. Not-found and other tags fail this
construction route. Common construction `0x1401a74c0`, for one argument,
also retrieves **current** GLOBAL.Object through `0x14019c050` and compares
its function identity with the selected constructor. The original distinct
String and Object callables take ordinary allocation. Replacing Object with
String changes that branch; it cannot be ignored.

The common allocator reads mutable callable `.prototype` and public
prototype `.constructor` metadata. Native String's allocator
`0x1401d7a80` creates the category-8 wrapper. Its initial cached prototype
links are subsequently rebound to the public prototype selected by common
construction. A bounded implementation may require the original cached and
public graph to agree; it must not silently replace the actual public lookups.

The wrapper's fresh own-entry insertion sequence is:

1. `__proto__`, marker tag 10, flags 3.
2. `length`, marker tag 10, flags 0.
3. `__constructor__`, original constructor FunctionRef tag 8, flags 3.

There is no own `toUpperCase` and no own public `constructor`. The constructor
callback receives argument zero from the current Environment's stack via
`0x1401811b0`. For the supported string argument, it passes the original Value
to payload setter `0x1401be0e0`; that uses native ToString `0x1401a2920`, then
`0x1400979a0` scans to the first NUL and interns the prefix into the wrapper's
payload. NUL-free input preserves character bytes. Original StringNode
pointer identity is not required. The callback's primitive return value is
discarded by common construction, which returns the allocated wrapper.

`0x14018f270` retains that wrapper in a temporary tag-6 object Value. The
interpreter requires that tag at `0x14020ba55`, then performs normal member
lookup for the selected method through `0x140194950`. The concrete wrapper's
getter `0x140197790` only special-cases `length`; `toUpperCase` follows generic
own/prototype lookup and finds the stored inherited native function.

This primitive branch admits a returned method **only as tag 8** at
`0x14020baa3`. It does not materialize a returned tag-11 method. Different
CallMethod receiver branches have different admission rules. At
`0x14020bb05–0x14020bb43`, the native FnCall contains the wrapper interface
as `this`, the active Environment, actual method argument count and argument
index. Invocation at `0x14020bb61` enters native wrapper `0x14018c5b0`; the
proved false Super predicate leaves this FnCall's receiver unchanged.
The method result, function reference and temporary wrapper have separate
retains/releases. No instruction replaces the original primitive with the box.

## Constructor callback and `length` scope

The callback `0x1401bf2d0` selects its mutable-wrapper branch only when
FnCall.this is nonnull, its category is 8 and its interface predicate `+0x80`
is false. The concrete fresh wrapper satisfies these checks. String.prototype
is also category 8, but its predicate slot at `0x142516600` points to
`0x14058fa60` (`true`), so it takes the fallback branch. The following are
**VERIFIED callback profiles**, not proof of every language-level `String()`
or `.apply` call's receiver binding:

| Receiver branch and arguments | Payload effect | Callback result |
| --- | --- | --- |
| Original wrapper, zero arguments | Replace payload with the current Environment's empty string | Empty primitive string |
| Original wrapper, one tag-5 String argument | Native ToString, first-NUL scan and re-intern into payload | Original complete argument string, before the setter's prefix conversion |
| Null, non-category-8 receiver, or original String.prototype; zero arguments | No payload setter or wrapper allocation in this branch | Current Environment's empty string |
| Same fallback branch; one tag-5 String argument | No payload setter or wrapper allocation in this branch | Native ToString result; for tag 5, the original complete string |

The selected zero/one-argument callback paths do not insert or remove object
properties. Shared construction installs the wrapper's three entries and
discards the callback result. Other argument Value tags retain their native
copy/coercion dependencies; their side effects and conversion outcomes are not
supplied by the tag-5 profile. Existing-wrapper mutation also requires its
actual payload setter to be `0x1401be0e0`, not category 8 alone. The callback's
current Environment selects the argument and ToString operation; re-interning
uses the manager associated with the receiver's existing payload node.

For exact `length`, String raw getter `0x140197790` first calls generic
`0x140197000`. A found value whose tag is **not** the internal marker tag 10
wins unchanged, including a found Undefined or accessor Value; normal outer
lookup may subsequently evaluate an accessor. Only a generic miss or returned
marker selects computation from payload `+0x68`. Thus even deleting a fresh
own marker does not imply computation if an inherited ordinary value is found.
Computed length is returned as native integer tag 4, with no property insertion.

The native counter `0x140096820` returns the stored byte count when its
count-equals-bytes cache bit is set; otherwise `0x14006b480` counts encoded
sequences and sets that bit if the result equals the byte count. For the
supported NUL-free ASCII payload, length equals its byte count; empty is zero.
The retained instructions also show one count per well-formed modern UTF-8
scalar, including a four-byte scalar, rather than two UTF-16 units. This does
not establish a VM encoding bridge or malformed/legacy/overlong/surrogate
behavior. The initial profile should keep ASCII and safe integer/resource
bounds. A blanket UTF-16-unit count would not reproduce the native non-ASCII
counter once that broader profile is implemented.

## Callback and exact ASCII transformation

**VERIFIED:** complete callback `0x14018d460..0x14018d4f3` requires nonnull
FnCall.this and its native category getter to return 8. It subtracts interface
offset `0x20`, reads the payload at object base `+0x68`, and calls
`0x1400978f0`. It does not read the argument count or argument Values. On
success it writes result Value tag 5 and the retained result StringNode. It
does not change the receiver's payload. The null/wrong-category path tailcalls
native logging `0x140186b80` without writing the result in this callback;
this document does not invent an exception or return-value contract for that
unsupported path.

`0x1400978f0` constructs a temporary byte string from the payload's explicit
pointer and byte count, calls uppercase loop `0x140036660`, then interns the
result with the original payload node's manager through `0x1400972e0`.
Output is an interned primitive string and may reuse an existing node. The
temporary object and byte buffers do not imply a new unique result identity.

The complete uppercase loop spans `0x140036660..0x14003674a`. It:

1. Bounds traversal by the input byte length.
2. Decodes through `0x14006a960`, narrows the decoded value to 16 bits, and
   calls native word mapper `0x140071fd0`.
3. Encodes the mapped value through `0x14006acd0` into a stack buffer.
4. Appends a chunk through `0x140034b40` at input end or when its byte count
   reaches `0x1f8` (504); then continues with an empty chunk.

For ASCII, the decoder advances one byte and returns it; the encoder writes
one byte and increments the chunk count. Shared append/copy receipts establish
explicit-length concatenation through `0x140034860` and the pinned
`MSVCR120.dll!memcpy` import. The output is independent of those chunk
boundaries. The empty temporary-string constant at `0x142adb2f0` has masked
length zero and a NUL payload, so an empty input returns empty output.

Native word mapper `0x140071fd0..0x140072062` uses a page bitmap at
`0x1425019f0`, then binary lower-bound lookup in 640 sorted 16-bit source/result
pairs at `0x1425020d0`. The ASCII page selects bitmap block `0x100`. Private
validation checks the complete pair ordering, every bitmap-selected ASCII
lookup's exact source match, and **all 128 ASCII outputs**. Exactly 26 change:
lowercase Latin ASCII letters become their uppercase equivalents. No OS locale
lookup or external Unicode database is used on this path.

**DERIVED:** a private calculation follows these retained branch semantics for
empty input, all nonzero ASCII values and lengths 503, 504, 505 and 1,017.
It confirms the same mapping across chunk boundaries. This is a calculation
from static evidence, not execution of the retail function. Embedded NUL is
excluded by the earlier boxing contract; the mapper's checked zero input does
not expand the public supported profile. Non-ASCII decoding, 16-bit narrowing,
legacy/malformed sequences and the client's non-ASCII tables require separate
compatibility work.

## Bounded implementation contract

The smallest faithful implementation has typed String callable/prototype/
wrapper identities, a primitive payload, real method descriptors and the
current Environment. It may initially support the original zero-argument
`toUpperCase` call on a NUL-free ASCII primitive after verifying:

- Current GLOBAL.String resolves to the canonical native String binding in
  this context; GLOBAL.Object satisfies the original distinct-identity branch.
- The selected callable, public prototype/constructor metadata, cached parents
  and current prototype chain still match the supported graph.
- Ordinary lookup returns the supported native `toUpperCase` tag-8 function;
  missing/replaced/accessor/resolver methods do not silently select it.
- The fresh wrapper's marker entries, parent, category and payload follow the
  proved construction path, and the active caller environment is preserved.

A direct string-spelling dispatcher bypasses observable constructor lookup,
allocation and property lookup. A partially represented native prototype must
also retain an explicit boundary for its other stored members; model absence
cannot become native missing. Descriptor writes, deletions, reparenting,
enumeration, arbitrary `apply`/borrowed receivers, unrelated String methods,
failed allocations and unsupported coercions are not established by the ASCII
mapping alone. The runtime's source-version choice is a separate investigation;
this work proves propagation of the active Environment, not which environment
an arbitrary cross-movie script should use.

## Evidence, validation and provenance

Private evidence lives under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-string-uppercase/combined-proof-receipt.json`, SHA-256
  `c0a824584a7fc3ec2c6ec04a601b8739144827eb0b1e02c4a3c80c51f028c6b8`:
  binds the component receipts and all 17 new REA observations; independently
  checks 1,253 decoded spans and 4,279 data/literal bytes against the pinned PE.
- `rea-string-uppercase/complete.bundle.json`, `ascii-derived.json` and original
  `derive_ascii.py`: callback, loop, mapper, private tables, source-body and
  exhaustive ASCII checks. Seven new REA records; 247 decoded spans and 3,512
  data/literal bytes match the pinned PE.
- `rea-string-native-binding/finding-summary.json`: two new REA records; 473
  decoded spans, 440 table bytes and four literals checked; deferred factory,
  category/parent links, method identity and descriptor flags.
- `rea-string-native-binding/graph-profile.json`: supplemental static-member
  table, complete fresh graph and concrete prototype predicate; the raw
  inventories remain private.
- `rea-string-primitive-call/finding-summary.json`: exact original call,
  primitive boxing, current-global dependencies, normal method lookup and
  active FnCall propagation; three new records plus named reused records.
- `rea-string-primitive-call/constructor/finding-summary.json`: concrete
  allocator, callback, payload setter and three marker entries; two new records
  plus retained common construction evidence.
- `rea-string-primitive-call/constructor-profiles.json`, `length-profile.json`
  and `constructor-prototype-predicate-join.json`: separately retained
  zero/one-argument callback branches, length override precedence/counting and
  the exact original prototype predicate. The last receipt closes the earlier
  constructor-profile receipt's explicitly unresolved predicate dependency.
- `rea-attached-name-classification/final.bundle.json` and
  `buffer-final.bundle.json`: retained ASCII encoder, append, explicit-length
  copies and pinned import evidence. These are reused without interpreting
  the earlier lowercase table as an uppercase table.

Each claimed new REA observation completed with exit 0, no truncation and
unchanged target SHA-256; each selected decoded span was checked against PE
section offsets. Padding and the first captured non-ASCII decoder instruction
are excluded from semantic claims. REA sessions were exported and reset.
No original executable was launched, no shared tool configuration changed and
no native table, executable byte, movie payload or extracted method inventory
was added to git. This change is research documentation, not VM implementation
or a retail runtime comparison.

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned executable and manager movie above | Proprietary owner-installed artifacts; private interoperability study | Selected native code/data and original caller evidence |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/connection-and-recovery.md`, `references/native-and-artifacts.md` | REA MIT documentation | Supported target-free native evidence workflow |
| REA 6.9.0 and Rizin 0.9.1 | MIT; LGPL-3.0-only | Bounded disassembly/data observations and Evidence export |
| Python standard library and GNU objdump tooling | PSF; GPL-3.0 tooling | Original private byte/hash checks, ASCII derivation and bounded candidate discovery |
| Original project `AS2_GLOBAL_BASELINE.md`, `EXTERNAL_INTERFACE_BINDING.md`, `DEFERRED_BUILTIN_VALUES.md`, `REA_QUALIFIED_VARIABLES.md`, `REA_FUNCTION_CALLS.md`, `AVM1_VM.md`, `LOCALIZED_LABEL_LOOKUP.md`, preceding prototype/construction findings and private receipts named above | Original project research; underlying native artifacts remain proprietary | Reuse registration, lookup, ownership, source and property contracts; localization's separate lowercase algorithm was not used as this method's implementation |

No external String/VM implementation or SDK source was read or copied.
