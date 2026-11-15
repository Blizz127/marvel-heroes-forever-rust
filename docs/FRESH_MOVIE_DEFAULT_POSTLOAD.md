# Selected action default: PostLoad

The selected GFxAction_OpenMovie class has an exact PostLoad binding. Its
complete handler and the saved CDO's empty linked-variable arrays allow a
bounded preservation contract for the StartPaused word through this stage.
The result does not require assuming the handler's mode global is zero.

**This is not an effective-default value or an observed successful load.**
Recursive archetype processing, cold class registration, unchanged metadata
and loaded-value correspondence remain conditions. The alternate-input and
cold-reference boundaries in the
[preload lifecycle study](FRESH_MOVIE_DEFAULT_LOAD_LIFECYCLE.md) still apply.
No Rust default changed, native action ran, or client/service was launched.

`VERIFIED` denotes exact private source bytes, selected bindings or bounded
correspondence. `OBSERVED` denotes saved records and static instructions.
The conditional paths below are static contracts, not measured invocations.

## Exact action binding

The retained named factory `0x14126dc80` supplies the constructor callback
`0x141276000` for the exact GFxAction_OpenMovie name. That complete 34-byte
constructor calls the shared base leaf and installs table `0x1421c1e50`.
The table's+0x40 entry at `0x1421c1e90` selects complete 2,384-byte
`0x140d467b0`. Both the factory/constructor bytes and the table entry are
independently matched to the pinned executable.

The slot's PostLoad role is grounded by the retained conditional caller
`0x140797020`: it calls current object virtual+0x40, then checks the marker
required by its explicit PostLoad/Super::PostLoad diagnostic. The base slot
selects `0x1407a2440`, which sets that marker. A remembered UE vtable layout
is not used to assign the slot.

This remains an exact selected-class binding. A different class, changed table
or replacement callback is outside the contract. The conditional caller reloads
the object's current table after dependency processing; a prior constructor
binding does not make intervening mutation impossible.

## Conditional entry and CDO dependency branch

Complete307-byte `0x140797020` tests runtime object flag
`0x0100000000000000`. When it is clear, the method returns without calling
PostLoad. When set, it clears that flag and marker 0x8 in O+0x10, then performs
dependency processing before dispatching the handler. It contains **no CDO
flag 0x200 or class-flag test**. Saved CDO metadata therefore does not establish
either the live whole flags word or invocation of this method.

The exact selected action table+0x148 is a null-return leaf. Consequently the
dependency context is null on this route: its optional context cleanup and
freeing are excluded. The caller can still recursively invoke ConditionalPostLoad
on `qword[O+0x58]` when nonnull. This recursive archetype dependency is
not silently treated as a no-op or a completed earlier load.

The caller next passes O and the **original null context**, not the archetype
pointer, to `0x140797160`. The complete 573-byte helper first tests object flag
`0x80000000000`. A clear flag returns; a set flag together with preserved CDO
flag 0x200 selects a short branch that only clears `0x80000000000` in O+0x10.
It bypasses the general instance/outer traversal, context construction and
property-copy callbacks. Its common stack-cookie check remains a normal-return
integrity condition. There is no action+0x120 write on either selected branch.

The conditional caller's earlier flag clear preserves 0x200. Applying this
reduction after recursive dependency processing still requires that the target
identity, CDO flag and table remain unchanged. The scope does not admit arbitrary
reentry or concurrent mutation from the dependency graph.

## Base PostLoad and action-class gates

The selected handler first tests qword O+0x64. If null, it calls
`0x140b6b530(qword[O+0x40])` and stores the result into O+0x64. The complete 102-byte
helper checks the incoming object's class/super chain against a cached class;
it is a cast of that one input, not an outer-chain walk. Its cold cache path
can construct/register a descriptor. The field's semantic name and a saved
default value are not inferred: ParentSequence is outside the selected 70-node
schema used in this study. The direct write interval `[0x64,0x6c)` is disjoint
from StartPaused's dword `[0x120,0x124)`.

The handler always calls base PostLoad `0x1407a2440` next. Its complete 101-byte
body ORs marker 0x8 into O+0x10. Further calls require global `0x142aee960 == 0`
and either class mask 0x400 or object mask 0x10000000000. The selected stable
action class flags 0x08000092 clear 0x400, 0x4 and 0x20.

Thus either the base returns after setting its marker, or the object flag
admits `0x14079de30(O,0,0,0,0)` followed by
`0x14079f4e0(O,0,0)`. Under the selected unchanged class flags, the first helper
only stores its selected class pointer to global `0x142c23c80` before returning;
the second returns before further calls. Their argument profile is retained
exactly: the last helper's argument 3 here is 0, unlike the enclosing preload's 1.
No action property writes follow from either selected base route.

These exclusions apply to the action class. The selected player classes have
configuration bit 0x4 set and cannot inherit this no-configuration-work result.
No configuration file contents were consulted or modeled.

## Actual saved links and the large handler branch

After base PostLoad, the handler tests global `0x142aee944`. Zero branches
directly to its epilogue. No runtime value, immutability or editor/game meaning
is assigned to that global. A nonzero value enters the action's VariableLinks
array at O+0xbc, with count at O+0xc4 and element stride 0x54.

The exact original GFxUI action default has three such entries:

| Selected saved input | All three entries |
| --- | --- |
| ExpectedType | GFxUI import reference -30, Engine.SeqVar_Object |
| LinkedVariables | Explicit serialized count 0 |
| Packed+0x50 fields | bAllowAnyType, bMoving, bClampedMax and bClampedMin all explicitly false |

The existing conditional Link replay maps the last four fields to masks 1, 2, 4, 8
at element+0x50. This establishes saved zero **under covered mask 0x0f**, not a
captured whole native word or unknown upper bits. bWriteable is a different
field at+0x38 and is true only in the second entry. It is not bit 0 of +0x50.
The retained correlation checks every selected saved tag/value/span; it does
not claim that later callbacks cannot change the loaded arrays.

With those exact loaded values preserved and valid nonaliasing storage, each
element can perform type/class work before the count test. The possible calls
must not be omitted merely because LinkedVariables is empty:

- Three named-class cache factory/registration pairs can be reached directly.
- Class-chain comparison can call getters `0x140b38e90` and `0x140b39050`.
- The alternate classification path calls complete 288-byte `0x140d4f9d0`
  with the link element and a selected class pointer.

Those helpers contain real cold-registration branches. The two getters return
a cache directly when warm; on cold paths they publish before registration
completes. The classification helper has no direct input-data write, but calls
those getters and other cache machinery. Nonnull pointers do not establish
readiness, finite class chains, correct identity or transitive purity.

Regardless of the normal-return type-classification result, count 0 excludes
both linked-object traversal loops. It also excludes the handler's target
virtual+0x1c8 modification calls. Its per-element temporary arrays remain empty,
so the corresponding allocator/free and warning-formatting paths are excluded.
The direct body neither changes the VariableLinks headers nor writes the
action's flag dword on this admitted empty-link route. The checked path is
stronger than assuming a mode flag disables the whole branch, while preserving
the real cache/registration obligations before those count gates.

Changing any LinkedVariables count, loaded field/layout identity, stable
descriptor chain or callback effects invalidates this selected reduction.
Nonempty linked objects can expose real virtual calls and mutation behavior;
the full handler is not declared generally read-only.

## Conditional stage result and remaining joins

For an exact selected CDO with stable action flags, table and admitted loaded
arrays, this stage's direct target writes are confined to object lifecycle
flags at+0x10 and the optional qword at+0x64. The selected base helpers do not
write action properties. The empty-link handler avoids linked-object mutation
and target+0x1c8 calls for either value of its mode global.

Under normal returns, valid distinct storage and confined cold-registration
and recursive-dependency effects, StartPaused's mask 0x04 in action+0x120 is
preserved through the selected handler and its conditional wrapper's marker
check. This extends the prior conditional preload checkpoint; it does not
observe the word or establish its effective value.

Concrete remaining joins include:

- The object being scheduled for this stage, with the necessary live flags.
  The retained load-completion dispatcher visits a pending queue; saved metadata
  alone does not establish membership or order.
- O+0x58 dependency identity, successful recursive completion and preservation
  of the selected action state across that call.
- Correct completed class caches, or confined cold initialization/registration
  without target reentry. Warm branches are available; warmth is not assumed.
- The prior direct-archive versus alternate-map input choice, original-tag
  reference resolution and ownership/I/O conditions.
- Any later per-instance construction, map overrides or actual action invocation.

No absence of StartPaused in the original saved CDO is converted into a runtime
false value by this document. No player defaults are merged with this action.

## Verification and sources

Private results are retained under
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-movie-default-postload/`.
The aggregate `receipt.json` seals exact component and document hashes. Byte
coverage across components overlaps and is not additive.

| Component | Focused evidence |
| --- | --- |
| native-verification | 3,076 bytes / 797 instructions and 46 literal/table bytes; exact factory/constructor/table, handler, four helpers and branch arithmetic |
| base | 6,094 retained bytes / 1,491 instructions; conditional caller, base handler and exact action finalizer gates |
| saved-links | 339 raw checks plus 143 independent comparisons; all 51 saved fields and 24 Bool tags in the three entries |
| dependencies | Exact selected+0x148 leaf and complete conditional helper; CDO/null-context branch proof |
| empty-links-review | Independent full-handler branch analysis under the actual empty-link profile; callback boundaries retained |

The independent CFG check also removes the empty-count premises as a negative
admission: linked-object, target+0x1c8 and allocation sites become reachable.
That rejects reuse of the empty-profile conclusion for arbitrary populated
objects; it is a static boundary check, not a captured positive-count execution.

New target-free REA captures were bounded to the selected methods and table
entries, checked against the unchanged pinned PE, exported and reset. No native
target execution, broad automatic analysis, source/configuration mutation,
network access or service operation occurred. The checks concern native bytes
and static contracts; they are not synthetic evidence of retail runtime parity.

Every outside source read is retained in the private component ledgers and
aggregate source ledger:

| Source | Licence / use | Contribution |
| --- | --- | --- |
| Owner's 1.52.0.1700 Win64 executable, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner install; private interoperability study | Exact selected instructions, table entries and class identity |
| Retained original GFxUI default/schema evidence; fresh-movie-default-copy/composition and fresh-movie-default-serialization/package | Private proprietary owner records plus original local study | Saved link values, offsets/masks and bounded tag correspondence; no new game bytes in git |
| Original frontend-initial-instance/native.bundle, scale9-texture-load/postload, fresh-movie-default-load-lifecycle/preload and original PE/extraction helpers | Original local evidence; underlying client redistribution not authorized | Reused complete REA, conditional caller, base/finalizer effects and source mapping |
| FRESH_MOVIE_DEFAULT_STATE/COPY/SERIALIZATION/LOAD_LIFECYCLE and SCALE9_TEXTURE_LOAD documents | Original local project work | Existing conditional contracts and source context |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`; rea-agents 6.9.0 | MIT | Mandatory bounded acquisition and Evidence workflow |
| Rizin; GNU objdump 2.46; Python | Tool-only LGPL-3.0-only; GPLv3-or-later; PSF respectively | Native inspection and original static verification |

No external implementation was copied. Tool licences do not authorize sharing
the client, original assets or extracted data. All such material remains private.
