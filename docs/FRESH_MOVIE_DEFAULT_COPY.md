# Selected fresh movie-default copy

The selected inherited parent-copy graph is now closed under an explicit
ordinary-success profile. Its copy callbacks preserve the action word at
`A+0x120`, including StartPaused mask `0x04`. Combined with the fresh allocation and
constructor checkpoints in [FRESH_MOVIE_DEFAULT_STATE.md](FRESH_MOVIE_DEFAULT_STATE.md),
that word remains zero **at the pre-finalization construction checkpoint**.

This is a conditional native-contract result, not a captured CDO or an effective
live default. No product default was changed. Saved-tag serializers, alternate
reinitialization, target reentry and later instance writes remain separate
from the inherited-copy result.

`VERIFIED` means exact owned-byte correspondence or the stated finite graph and
range computation. `OBSERVED` denotes saved data or static instructions. No
native client, service, configuration import, original handler or sequence ran.

## Admitted copy conditions

The proof uses the strict fresh route already recovered: a new destination
object, the selected parent default as source, copy context `K=0`, object
context `O=0`, and no extra-context/helper mutation before the checked copy.
The ordinary linked type identities, property flags, offsets, masks and
Children/Next order must remain stable. The enclosing class flags must leave
extra context `X` absent on that route.

All inline spans and pointer-backed source buffers must be valid. Dynamic
counts are nonnegative, dimensions/products/offsets do not overflow, and the
allocator succeeds with storage confined to its bookkeeping and new buffers,
disjoint from the destination object, source objects and property metadata.
No concurrent mutation or target reentry may change these inputs during copy.
These are explicit successful-operation conditions, not claims about a sampled
heap, a corrupted file or every possible allocator implementation.

The source array counts are not assumed zero. The proof admits valid nonempty
arrays by following their actual inner-property types. Saved counts remain
separate from current source/template values.

Outer copy context `K` is distinct from the property's fourth ABI argument.
The proof does not assign that argument a zero value: the Array, Struct and
static-dimension wrappers forward it, and the admitted leaf footprints are
independent of it. The Struct evidence has a preserved correction addendum
with this exact distinction.

## Exact finite schema

The five inherited array roots are SequenceAction.Targets and
SequenceOp.InputLinks, OutputLinks, VariableLinks and EventLinks. Starting only
from their declared inner references, the bounded package walk reaches 70
nodes: 65 properties and five ScriptStruct descriptors. It reads 2,936 prefix
bytes, checks 99 serialized reference fields and follows no function bodies,
struct-default tails or referenced Object/Class definitions.

The graph contains eight arrays, including three nested arrays, and has no
inline cycles or unresolved traversed types. All static property dimensions
are one. A separate metadata check binds all 70 node class references to the
ten exact `Core.<Type>` imports, each with metaclass `Core.Class` and the actual
Core package outer. Callback selection does not rely on an unqualified type
name from an arbitrary package. The five structs have null superclass references:

| Struct | Direct properties | Conditional native extent |
| --- | ---: | ---: |
| SeqOpInputLink | 12 | `0x34` |
| SeqOpOutputInputLink | 2 | `0x0c` |
| SeqOpOutputLink | 13 | `0x38` |
| SeqVarLink | 17 | `0x54` |
| SeqEventLink | 8 | `0x30` |

The original replay uses the already recovered native widths, alignment and
Bool packing rules. Every field interval fits its enclosing struct. The
conditional extents depend on correctly initialized native cast masks, normal
recalculating Link mode, the recorded order and unchanged referenced metadata;
they are not measurements of a live struct.

The nested arrays are OutputLink.Links, VarLink.LinkedVariables and
EventLink.LinkedEvents. Links contains SeqOpOutputInputLink, whose only fields
are an Object reference and an Int; its inner StructProperty has no bit22 and
needs a plain 12-byte element-buffer copy. The other two contain Object
references. The four larger structs have their actual String/Array child
properties, so their constructed-copy branch is admitted and analyzed.

The independent package checks report 3,206 checks and zero violations. They
preserve identity-only Object/Class/enum references and two non-followed root
sibling Next links as outside the inline closure; those are not missing copy
edges.

## Native callback closure

The top-level parent copy first clears each selected one-element 16-byte
Array/Str header. Array construction also zeroes each newly allocated
constructed-element buffer before invoking its inner callback. These zero
preludes establish the nested destination-header conditions; they are included
in the footprint, rather than attributed to the callbacks themselves.

| Property family in the selected graph | Checked operation |
| --- | --- |
| Int / Float | Copy four raw value bytes; the scalar dimension-one branch makes no calls |
| Name | Copy eight raw name bytes; no name lookup or reference traversal |
| Bool | Replace only its mask within one four-byte field |
| Object / Class | With `O=0`, copy eight reference bytes per static element and bypass instancing/helper calls |
| Str | Write its 16-byte header and separately allocated counted character storage |
| Array | Write its 16-byte header, allocate separate element storage, then copy plain elements or call the exact inner property |
| Struct | Copy its bounded byte extent when bit22 is clear; otherwise walk reflected children and dispatch their exact copy callbacks |

Int and Float use complete-copy leaf `0x1407bbdd0`; the selected dimension-one
branch writes one dword. Name uses `0x1407bbe10`, writing one qword. Bool's
element leaf `0x1407bc2f0` combines the source mask with the destination's
other bits and writes one dword. Its complete-copy wrapper is the already
bounded `0x1407bc230`. These operations do not invoke a VM, constructor,
configuration handler or target-referent method.

Object and Class tables share complete-copy `0x1407bbe50` and element thunk
`0x1407bc420` → `0x1407c69c0`. The propagated fifth ABI argument is `O`.
When it is null, both paths reach the direct qword copy before any instancing
helper, global access or `X` read. The result does not require the referenced
object to be null or the property bit22 to be clear: reference values are
copied without following their referents. It does not generalize to nonnull
object context.

StructProperty table `0x141e120a0` selects complete-copy stub `0x1407bc2c0`
and element method `0x1407bc460`. With bit22 clear it uses a bounded plain byte
copy. Otherwise it uses the generic static-dimension wrapper and walks the
referenced struct's fields, invoking **every reflected child property's**
copy slot `+0x2c8` with destination/source plus that child's offset. There is
no separate native custom-copy operation in this method.

The selected ScriptStruct table is `0x141df7878`. Its `+0x270` slot is the
five-byte getter `0x14025f3d0`, returning descriptor `+0x78`; it is not an
unconstrained callback in this profile. All five saved superclass references
are null. Its Bool-packing slot `+0x260` selects the known return-one leaf.
Class-mask initialization remains a stated metadata admission condition. Its
concrete producer is now identified: registration `0x1407490f0` sets the
superclass and conditionally dispatches UClass virtual `+0x118` to
`0x14074e2c0`. That method ORs the parent's cast mask into the child's at
`0x14074e322`. ScriptStruct's own mask `0x10` thereby inherits UStruct's `0x08`
on that route. The predicate reads global `0x142ba3af0`; its live value was not
observed or assumed.

Str and Array callbacks retain the detailed allocation/count conditions from
the preceding study. Array capacity is prepared for the exact source count;
the selected append path cannot require further growth on valid input. Nested
Object/Class callbacks take the null-context reference-copy path. Nested
constructed structs recurse only through the finite graph above. A String or
Array pointer is not treated as inline data following its header.

## Composition at the action word

The superclass copy extent is conditionally `0x108`. Bulk copy therefore ends
before `A+0x120`. The seven inherited copy headers occupy the exact intervals
retained in the previous study: two String headers, four SequenceOp Array
headers and Targets at `[0xf8,0x108)`.

The finite composition checks copy contracts for 63 property nodes, including
nodes whose selected Array route uses plain copying instead of dispatch.
The two remaining declared members belong to the plain 12-byte OutputInputLink
element block; they do not cause separate callback dispatch on that path. Every inline child
write stays within its current element or field. Each new pointer-backed
region is disjoint from the enclosing action under the allocation conditions.
Object references do not cause traversal of their pointees. Induction over
this acyclic graph therefore excludes `A+0x120` from the complete admitted
parent-copy footprint, including nonempty nested arrays.

The source default need not have empty collections. Its exact values do not
enter this preservation result. Correct source selection, valid buffers and
stable counts do. The subsequent action constructor writes only its vtable,
preserving the established word at the checked checkpoint.

Four focused private admission checks reject an unknown callback family, a
static dimension outside this profile, a child write beyond its struct extent
and an inline callback cycle. They validate the bounded composition machinery;
they are not synthetic evidence of retail behavior.

## Separate finalization and saved tags

Ordinary loaded UClass Link writes `class+0x25c=0` when its rebuilt property
list is nonempty. The selected action has such a list. With that checkpoint
unchanged, finalizer `0x140741410` returns at its entry gate and does not execute
its later copy/configuration/callback paths. Factory-time metadata initially
has `+0x25c=1`, so a pre-load route is distinct.

Other paths can still invoke reinitialization helper `0x14079d590`. In
particular, UClass serialization has a separate global-gated call after reading
its terminal CDO reference, when `0x142aee960` is nonzero. Its null optional
arguments exclude its direct header-clearing/Outer writes; stable selected
action flags exclude its native
virtual `+0x1f8` path. It can nevertheless call the default-copy wrapper again.
Because the first copy set object flag bit20, that later wrapper can run
cleanup over the **action's** copy list, including CaptureKeys and
FocusIgnoreKeys. The fresh zero-header cleanup proof cannot be reused for
already populated storage. This alternate route remains separate from the
fresh-copy certificate.

The complete tagged loader has no post-None loading pass that initializes
omitted fields. A terminating tag name goes straight to the restore/return
epilogue. The apparent subsequent property traversal is its saving path.
Thus the absence of StartPaused does not itself trigger a load-time clear or
fill operation.

The selected action CDO saves exactly five tags: MoviePlayerClass,
bDisplayWithHudOff=true, OutputLinks, VariableLinks and ObjName. The recovered
Bool fast path ORs HUD mask `0x10` and preserves StartPaused mask `0x04`. Other tags call
their property **serialization** slot `+0x290`; the copy slot `+0x2c8` proof
above cannot substitute for those serializers. Reference-loader callbacks,
target reentry and actual tag-stream invocation remain explicit boundaries.

The selected ClassProperty serializer is now bounded independently. Both
ObjectProperty and ClassProperty tables select `0x1407cc6b0`. MoviePlayerClass's
declaration flags are `0x1`, so its optional archive `+0xc0` notification is
skipped under the unchanged flag profile. The on-disk tag type is ObjectProperty;
the resolved declaration is ClassProperty. That distinction is preserved.

For the established ordinary CLinkerLoad archive subobject `Ar=L+0x1c8`, table
`0x141dfd600` selects `0x14075d0a0` for virtual `+0x30`. It reads a four-byte
serialized reference into stack storage, resolves it and writes one qword to
the supplied property destination. The saved reference `37` takes the ordinary
positive-reference branch through `0x14076bb70` to `0x140760610(L, 36)`.
That export-loader call is an explicit reentry boundary. A bounded local
qword store does not establish that the complete loader call is pure or that
no target mutation happens during reference resolution.

The separate Array/Str serializer study confirms only their immediate
header/storage contracts. Array `0x1407cc1e0` obtains the count through archive
`+0x08`, can clean up old elements through `+0x2e0`, updates its 16-byte header
and allocates/zeroes pointer-backed element storage. It then calls the actual
inner property's **`+0x290`** at `0x1407cc331`, with a null extra context.
Str's thunk `0x1407cc710` calls `0x140776fa0`, which reads a signed length,
updates its header and handles character data in allocated storage. Valid
lengths/products, old-element cleanup, allocator/archive behavior and nested
serializers remain conditions; native count arithmetic is not a safety check.

For the selected tag destinations, MoviePlayerClass occupies `[0x110,0x118)`,
OutputLinks `[0xac,0xbc)`, VariableLinks `[0xbc,0xcc)` and ObjName
`[0x6c,0x7c)`. Their immediate field/header writes are disjoint from the action
flag word. HUD writes that same word but only ORs mask `0x10`. The remaining exact
serialization seams are the link-struct inner `+0x290` callbacks, cleanup of
already populated collections, archive reference resolution and possible
target reentry. No full saved-tag preservation claim is made from the direct
writes alone.

The retained count audit contains five explicit positive array-count
occurrences and 20 absent occurrences across the selected default lists.
Neither those absences nor the new copy proof assert an effective inherited
count or current template value.

## Evidence and provenance

All raw bytes, prefixes, disassembly and receipts remain under the private
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-movie-default-copy/`
directory. Only this original analysis document enters git. Evidence IDs,
hashes and the aggregate `receipt.json` are retained with the completed intake.

Sealed component receipts include:

| Receipt | SHA256 |
| --- | --- |
| `package/receipt.json` | `7cbb7ad6b2ba911f773133849cf4eee1855b7dd823806f31e173df4b1e29ea8e` |
| `package/class-import-identity/receipt.json` | `314d2abeeacfad285d64e4f402d21d6144ce69f2340a117b5f81bdd5f21dcb48` |
| `object-native/receipt.json` | `aaf00afb0d056b53d4318f852c143149a0c1afb8f47898eee1d2ed5530e7ab7f` |
| `struct-native/receipt.json` | `486adbb52882ffa96860bca268f39755005c969cb6e0bc9382b36ae086e2d8a1` |
| `struct-native/argument4-addendum/receipt.json` | `f5cb7fc1b2eef4237ad6312b222bcb703f131ce455d10ff0a0498d5efe365408` |
| `tag-object/receipt.json` | `1da3acececf95e435de1178ef73229f9a99b30eccb9cf2262daa604bad8e7723` |
| `finalization/receipt.json` | `6d9a836c9505546096d2f954ac2aef9d31a521a59d2ee7955c315cd656349715` |
| `finalization/global-address-correction.json` | `f32b8f5320c8a6d9144d738d27eb2cf7389af334b1eddf247d16c400269629ac` |
| `finalization/tag-serializers-receipt.json` | `29da62e715bdfde7790516ed6eb12fc5f78649efe1ef0616446f138b25839acd` |
| `finalization/copy-arg4-receipt.json` | `56e667bcb7429bad1e4dfcf3ba5791d70b36106796ed5de57afd571a91cc1693` |

The global-address addendum corrects a spelling in the sealed finalization prose:
exact signed RIP-displacement arithmetic gives `0x142aee960`, not
`0x142ae6960`. It preserves the original receipt and does not change the
nonzero predicate. This document uses the corrected address.

The new scalar record is
`ev_47c57d85e73e9d2f207fcd9cd310d9a28355119212180444018c099e8f51a2f4`;
`composition/scalar-verification.json` checks 176 code bytes / 63 instructions
and 80 table bytes. The Float identity reuses the exact named factory and
constructor from the prior scale9 study, including its shared Int copy slots.
The retained scalar argument audit and the Array/Str/Struct addenda keep the
arbitrary fourth callback argument separate from the outer copy context.

`composition/schema-layout.json` contains the original linked-field arithmetic;
`composition/copy-certificate.json` contains the region/callback composition and
its explicit conditions. No live pointer or source count is supplied by either.

| Source | Licence / use |
| --- | --- |
| Owner's pinned MarvelHeroesOmega.exe, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner copy; bounded static copy, finalization and tag-loader contracts |
| Owner's pinned Engine.upk, Core.upk and retained GFxUI.upk default records; exact hashes in package receipts | Proprietary owner assets; selected declarations, subtype graph and saved tag/count occurrences |
| [FRESH_MOVIE_DEFAULT_STATE.md](FRESH_MOVIE_DEFAULT_STATE.md), [FRONTEND_MOVIE_DEFAULTS.md](FRONTEND_MOVIE_DEFAULTS.md), [SCALE9_TEXTURE_INPUTS.md](SCALE9_TEXTURE_INPUTS.md), [SCALE9_LOADED_LAYOUT.md](SCALE9_LOADED_LAYOUT.md), [WORLD_TRANSFORM_DEFAULTS.md](WORLD_TRANSFORM_DEFAULTS.md) and their private evidence | Original project research; exact type identities, layout methods, prior copy ABI and saved records reused |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md` and installed rea-agents | MIT toolkit; bounded target-free investigation, Evidence export and reset; no setup/config edits |
| Installed Rizin and GNU objdump | LGPL-3.0-only and GPL-3.0-or-later respectively; selected static decoding, then exact owned-PE checks; no implementation copied |
| Python standard library | PSF licence; original byte, schema, reference, arithmetic and receipt verification scripts |
| Historical [EliotVU/Unreal-Library](https://github.com/EliotVU/Unreal-Library) profile corroboration at commit `8236c39b79e30887a487ecf9b018204e430a5ebc` | MIT; provenance inherited through the original class/profile readers; no new external implementation source read |

No product source changed. No product build, full suite, native runtime or
behavioral-parity check is claimed by this documentation increment.
