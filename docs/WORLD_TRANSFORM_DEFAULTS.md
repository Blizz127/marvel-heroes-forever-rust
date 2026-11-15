# Saved Vista transform defaults

This is a bounded static application study for the owner's pinned
`MarvelHeroesOmega.exe` build 1.52.0.1700. It extends
[WORLD_TRANSFORMS.md](WORLD_TRANSFORMS.md); it does not change the explicit-input
Rust transform module or claim that the saved Vista meshes can already be placed.

**OBSERVED:** fresh allocation, class-default selection, template copying,
property callbacks, native constructors and subsequent class-default processing
are distinct stages. A nearest-tag merge does not reproduce the observed native
path. **VERIFIED:** the selected instruction bytes and original saved references
described below match the private pinned inputs. Final effective values for the
74 omitted component inputs and four omitted actor inputs remain unresolved.

## Actual source obligations

The Asgard collection is export 1340, object reference 1341, with 45 saved
StaticMeshComponents. Their original array order remains significant. All 45
explicitly reference archetype import `-140`, resolved by package, full path and
class to Engine export 24529, object 24530. This template has archetype zero and
contains none of the requested placement fields. The collection itself has
archetype zero. A template's outer actor supplies neither the collection's actor
transform nor the component's Scale.

| Omitted input | Count | Saved ancestor source | Effective value |
| --- | ---: | --- | --- |
| Component Rotation | 15 | Absent from the inspected default chain | Unresolved |
| Component Scale | 45 | PrimitiveComponent default explicitly saves `1` | Unresolved application |
| Component Scale3D | 14 | PrimitiveComponent default saves `(1,1,1)` | Unresolved application |
| Actor Location and Rotation | 2 | Absent from the inspected default chain | Unresolved |
| Actor DrawScale and DrawScale3D | 2 | Actor default saves `1` and `(1,1,1)` | Unresolved application |

The previous complete bounded serialized-class profile supplies explicit
class-to-default references for nine Engine/Core classes. It does not identify
defaults by `Default__` names. Its native serialization semantics remain
externally corroborated inference, distinct from verified byte coverage.
The PrimitiveComponent unit-scale payloads are at Engine logical offsets
3014524 and 3014560; Actor's corresponding payloads are at 2961422 and 2961458.
Those bytes are source values, not observations of live object memory.

All 135 absolute flags, 45 component Owner tags and 45 LocalToWorld tags are
also absent. The actor's additional pivot input and runtime base/update path
remain separate obligations beyond the 78 placement slots counted above.

## Allocation and selected default source

The generic path is shared with the independent frontend-instance lane. Cached
REA observations from the animation-default study provide the complete
constructor wrapper `0x1407a9f00`, allocator `0x1407a8d20` and default resolver
`0x140745a20`. No game process was launched to observe an actual call.

1. The fresh allocator path calls imported `memset(object,0,class_size)` at
   `0x1407a9cf8`. A reuse path can return before this call. Zero initialization
   is therefore conditional on reaching the fresh branch.
2. The default resolver's fresh construction passes the superclass default as
   template, flags `0x20400000200`, and zero extra constructor contexts. The
   allocator's flag-bit-9 path selects the superclass as its default-class
   input. A supplied template's class size supplies the copy extent; otherwise
   the selected default class supplies it.
3. Factory size propagation is now established at construction time:
   `0x14073a870` preserves factory `R8d` through `0x14073ad70` into
   `0x14073ae90`. The latter saves it in `EBX`, calls its base constructor,
   then writes it to class `+0x88` at `0x14073aebe`. It also initializes the
   class lists at `+0xb0` and `+0xb8` to null. Later linking/loading can form
   those lists or alter the layout; the construction-time assignment does not
   establish their final contents.

For example, Core.Object's factory size is `0x60`; that value now has an exact
constructor-time route to `+0x88`. This closes a size-propagation gap in the
earlier animation analysis, while leaving loaded runtime layout separate.

## Default-copy ABI and overwrite passes

**OBSERVED native contract:** the complete 215-byte wrapper `0x1407a5d10`
takes object, destination, integer parameter, default class, explicit source,
extent, object context, copy context and extra context. It passes the last eight
to inner helper `0x14079d6d0`, dropping only the first object argument.

If the object's flag bit 20 is already set, the wrapper first traverses its
class `+0xb8` list. Unless a property object's flag bit 41 is set, it invokes
property virtual `+0x2e0` with the destination plus that property's signed
offset `+0x8c`. The interpretation as prior-value cleanup is inferred from the
call pattern; the callback's write footprint is not proved here. This pass has
no extent test. The wrapper then sets object flag bit 20 before the inner copy.

The inner helper occupies 625 bytes and 150 instructions across seven adjacent
unwind fragments. Treating its first 47-byte unwind fragment as the whole
function would omit the copying behavior.

An entry gate returns without calls or writes when global `0x142aee960` is
nonzero and global `0x142aee940` has mask `0x200`; its actual state is unknown.
Without an explicit source, fallback requires a nonnull default class, its
nonnull default pointer `+0x1d4`, and nonzero size `+0x88` before invoking the
getter. This is conditional source selection, not unconditional zero filling.

| Pass | Source and selection | Preservation limit |
| --- | --- | --- |
| Bulk copy | Explicit source and supplied extent, or eligible default class's resolved default and class size | Copies `[0x60,extent)` only when source is nonnull and extent exceeds `0x60` |
| First property pass | Nonzero effective copy context; object-context class list `+0xb0`; property mask `0x202000`; source independently obtained from the selected default class | Can overwrite data copied from an explicit template; no extent gate |
| Second property pass | Selected default-class list `+0xb8`; selected bulk source | Tests signed property start offset `< extent`, not `offset + width <= extent`; virtual write footprint remains separate |

Copy context `-1` selects the object context. A zero effective copy context
skips only the first property pass. In the second pass, a nonzero context skips
properties selected by mask `0x202000`; zero context does not apply that skip.
The native masks are retained numerically; this study does not guess their
named meanings from a serialized flag word.

The fresh class-default resolver's zero context therefore excludes the first
property pass. It does not prove that all subclass bytes remain zero. The
second pass and its property callbacks require applicable runtime list,
offset and width evidence. An ordinary template-backed instance can additionally
use the first pass's class-default source after its template bulk copy.

Both property passes invoke property virtual `+0x2c8` on relevant paths. The
native copy/zero count uses a signed extension of a low-32-bit product from
property `+0x68` and `+0x6c`, without an observed checked range. A safe original
implementation must validate those ranges and reject unknown callbacks; it
cannot replace every virtual copy with a byte copy and retain this contract.

## Native constructors preserve the scalar inputs

The exact native class factories and their constructor callbacks are tied to
the owned executable, rather than selected by a similar class name.

StaticMeshCollectionActor's callback `0x140e06750` calls the 14-byte base leaf
`0x140793fd0`, installs its vtables and clears dword `+0xb4`. Neither body
writes actor Location/Rotation/DrawScale/DrawScale3D storage at `+0x80..0xa7`,
nor the additional pivot-like input at `+0xa8..0xb3`. It preserves whatever
allocation and copying supplied there; it does not independently set unit scale.

StaticMeshComponent's physical callback chain reaches `0x140df4850` and
`0x14092d510`, then the same base leaf. The inherited constructor explicitly
writes an identity matrix to parent-matrix storage `+0x1c0..0x1ff`.
The selected chain does not write the consuming transform inputs
Translation/Rotation/Scale/Scale3D at `+0x200..0x227`, absolute-control word
`+0x17c`, runtime actor context `+0x78`, or result matrix `+0xc0..0xff`.

The identity parent matrix is a concrete **constructor-stage** value. It is
not a claim that the Vista component's later parent is identity: the previously
recovered update route replaces it from an actor matrix. It is also distinct
from LocalToWorld output storage. No runtime owner is inferred from the package
outer or from this initialization.

## Subsequent class-default processing

The default resolver can call `0x140741410` after construction. Its work entry
requires nonzero global `0x142ae68f0`, nonzero class `+0x25c`, and either class
flag `0x10000000` or global `0x142aee940` bit 1. The applicable runtime gate
values are not observed. The constructor's initial class flags alone do not
settle subsequently linked/loaded state.

When entered, superclass recursion precedes further early returns. Continuing
work can invoke class virtual `+0x278`, reinitialize default data, call a class
callback and invoke two further property/data helpers. A proved early return
would exclude only the calls after that return, not earlier recursion.
The shared factory callback `0x14058fea0` is exactly `ret 0`, with no object
writes; that particular callback is not an unknown mutator.

Reinitialization helper `0x14079d590(CDO,class,0,0)` has no literal transform
initializer. With zero fourth argument it skips its header-zero branch.
If CDO flag bit 20 and class flag `0x10000000` are both set, it clears CDO
flag bit 41 and invokes object virtual `+0x1f8`. Otherwise it calls the existing
property-initialization wrapper with superclass/default context. Thus even a
preserving native constructor does not close the subsequent copy obligation.

The broad helper `0x14079de30` is **INFERRED** configuration/property-text
import from its selected calls and mutation paths. It requires class flag
bit 2 and property runtime flag bit 14; runtime flag bit 35 is rejected when
global `0x142aee944` is zero. With the observed zero arguments, superclass and
other-object recursion and its final virtual notification are skipped.
Applicable property eligibility and external entry values remain unknown.

The separate 442-byte helper `0x14079f4e0(CDO,0,0)` requires global
`0x142aee960` to be zero and class flag mask `0x20` to be set. Its zero third
argument skips recursion. It calls `0x14079b570` to prepare three local buffer
descriptors; only a nonzero return reaches `0x1407a0420`, with class, three
selected buffer pointers and the CDO contexts. There are no direct object-field
stores in the captured body, but those two callees' effects remain unresolved.
Calling this pass localization would be a further inference, not a fact used
to establish preservation here.

## Implementation boundary and remaining proof

No automatic saved-to-effective resolver is authorized by these findings.
`world_transform` continues to accept explicit finite inputs and return the
saved-source obligations without filling them. In particular, neither zero
allocation nor a native constructor that preserves bytes is a substitute for
loaded default application.

The remaining narrow dependencies are:

1. Tie the saved class/default/archetype references to the native loaded objects
   actually selected for the Vista collection and its components, including
   loader allocation versus reuse and applicable initialization contexts.
2. Prove target property offsets, widths, runtime flags and copy-list membership,
   or bound the actual selected property callbacks sufficiently to exclude
   unrelated writes. A start-offset extent test alone is insufficient.
3. Establish tagged/native serialization ordering and the application of the
   explicit saved unit scales through inherited defaults and the component
   template. Prove preservation of genuinely zero-initialized absent fields.
4. Bound applicable postconstruction/configuration/localization/post-load and
   runtime construction effects. Afterwards, resolve the saved collection to
   runtime component-owner/update binding separately.

The 78 missing placement slots remain missing in the source audit. Resolving
them will still not establish an active camera, live collection selection,
material/GPU behavior or retail visual parity.

## Private evidence and provenance

All native bytes, package extracts, instruction arrays and per-object receipts
remain under `~/.local/share/marvel-heroes-forever-rust/evidence/`.
The executable SHA256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`;
the Asgard package SHA256 is
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.

| Private evidence path | Scope |
| --- | --- |
| `animation-runtime/native/rea-*.json` | Reused canonical allocator/default-getter/copy/finalization observations |
| `world-transform/defaults/class-default-link/` | Nine complete bounded class/default profiles and original saved default tags |
| `world-transform-defaults/copy-wrapper/` | Exact wrapper ABI, new constructor-size propagation, original byte checks |
| `world-transform-defaults/copy-inner/` | Independent complete copy-helper ABI, loops, callbacks and seven-fragment byte audit |
| `world-transform-defaults/class-construction/` | Vista-specific callback/constructor chain and preserved target fields |
| `world-transform-defaults/postconstruction/` | Narrow subsequent helper gates and unresolved write categories |
| `world-transform-defaults/effective-input-obligations.json` | All 78 omitted inputs in original object/field order; no effective value filled |

The copy-wrapper check matches 511 bytes and 112 instructions against the
original PE. A 20-second raw-span request reached its provider deadline; its
partial result is retained separately. After cleanup and absence of an owned
provider child were checked, a bounded explicit-quit request completed. The
three-record canonical bundle includes the failed attempt and both complete
new observations; it was exported and checkpointed with no retained target.
No client process was launched; settings, accounts and other repositories were
not modified.

The constructor-chain audit matches 302 bytes/63 instructions and replays four
null/non-null callback cases in an independent static model. The postconstruction
audit matches 5,900 selected bytes plus the 429-byte caller, including exact
configuration diagnostic literals. These are static byte/model checks, not
execution of client code. In the per-input receipt, 61 omitted slots point to
explicit saved scale payloads requiring application proof; 17 have no saved
value candidate. All 78 retain `effective_value: null` and named obligations.

All analysis code is original. The owner-provided EXE, Engine/Core and Asgard
packages are proprietary compatibility-study inputs. Existing REA skill,
native/evidence guidance, prior project source and receipts were reused.
GNU objdump 2.46 (GPLv3-or-later) provided independent static disassembly.
The previous class-default profile studied MIT UELib at commit
`8236c39b79e30887a487ecf9b018204e430a5ebc`; exact prior source URLs, hashes,
licence and use remain in `world-transform/defaults/class-default-link/sources.json`.
No external implementation was copied into the product. This increment adds
no new renderer, source module, runtime claim or native game execution.
