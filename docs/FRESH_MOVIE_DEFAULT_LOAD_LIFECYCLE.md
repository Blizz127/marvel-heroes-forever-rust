# Selected movie-default preload lifecycle

The owned executable establishes a concrete CDO preload route and a bounded
normal-return interval through its enclosing loader. **CDO preload bypasses the
ordinary UObject serializer.** It reinitializes the selected object, then calls
the class's dedicated default serializer. Its direct original-archive path
returns from the final saved `None` without an additional property sweep.

This closes a specific lifecycle interval, not effective StartPaused. The
alternate-data map, reference-cache state, cold registration/reentry, backing
callbacks and subsequent PostLoad remain explicit. No product default changed,
and no native client, sequence, configuration or service ran.

`VERIFIED` means exact owned bytes, source identity or bounded correspondence.
`OBSERVED` describes saved values and static instructions. A conditional native
contract is not an observed native invocation. This document extends
[fresh construction](FRESH_MOVIE_DEFAULT_STATE.md),
[inherited copying](FRESH_MOVIE_DEFAULT_COPY.md), and
[saved-tag serialization](FRESH_MOVIE_DEFAULT_SERIALIZATION.md).

## Concrete enclosing route

For loader L, the retained archive subobject is A=L+0x1c8. Its table
`0x141dfd600` slot+0x20 selects complete 1,086-byte `0x14076d640`.
The same-loader path requires object O's load-needed flag
`0x20000000000`, matching O+0x28=L and a valid signed export index at O+0x30.
There are repeated flag checks and real recursive/class-initialization branches
before serialization. A different linker redirects to its own archive; it does
not inherit this archive's identity or state.

Preload saves its current backing pointer and position, chooses a backing reader
from runtime flags/globals, seeks to the export offset and requests its span.
The backing methods are callbacks, with no captured successful I/O in this study.
It clears the load-needed flag before serialization, then distinguishes:

| Current object flag 0x200 | Actual dispatch |
| --- | --- |
| Clear | Object virtual+0x68 at `0x14076d831`, with a temporary current-object global |
| Set | `0x14079d590(O,O.class,0,0)`, then `0x140752720(O.class,O,A)` at `0x14076d80f` |

The selected saved CDO has flag 0x200, but that saved flag does not establish the
current load-needed flag or an actual invocation. The ordinary UObject
`0x1407a7720`/`0x1407a7e90` route in the previous serialization study remains a
valid separately checked contract. Its archive+b0/+b8 accounting and final
archive+0x28 call must not be inserted around this CDO path.

## Reinitialization before the tags

The concrete call to `0x14079d590` must be accounted for even if fresh allocation
and copying were already checked. With stable selected action class flags,
its zero optional arguments exclude its header/outer-replacement paths and the
class-specific virtual+0x1f8 branch. It calls the existing copy wrapper with
the object's superclass, explicit source null and the selected zero contexts.

The wrapper can clean nine linked headers when object flag 0x100000 is set;
property object flag bit 41 may skip individual callbacks. The seven inherited
headers were covered by the populated-storage copy/cleanup receipts. The two
additional action-local headers are CaptureKeys `[0x130,0x140)` and
FocusIgnoreKeys `[0x140,0x150)`. Every possible header is disjoint from the
flag dword `[0x120,0x124)`.

For the strict fresh route, the local headers start zero, parent copying ends
at 0x108, the selected constructor only installs its vtable, and none of the
five saved action CDO tags names either local header. With no additional or
reentrant writers, both therefore remain zero across these reinitializations.
Their nonzero property offsets bypass the cleanup Class-cache guard; zero
count bypasses element callbacks and zero pointer bypasses freeing. The only
local cleanup write clears an already-zero count/capacity qword. This is a
conditional zero-header invariant, not a claim about arbitrary loaded objects.

The inner copy also has a useful exact source guard. With explicit source null,
`0x14079d718..0x14079d72f` calls default getter `0x140745a20` only after checking
the superclass cache+0x1d4 nonnull and extent+0x88 nonzero. Under stable metadata,
the getter immediately returns that cached pointer without calls. Missing
cache or zero extent skips the selected source-copy route rather than forcing
cold creation here. No cache identity or source contents are inferred from
nonnull alone.

On the admitted successful cache route, SequenceAction's linked extent 0x108
and the seven inherited copy contracts preserve action+0x120. Parent counts
need not be zero. Valid nonaliasing source/target buffers, stable descriptor
graphs, bounded old counts and confined allocation/free services remain
requirements. Checking all nine possible cleanup headers is a preservation
superset, not a measured callback count.

## Dedicated default serializer and alternate input

Complete 469-byte `0x140752720` first calls the NetIndex helper `0x1407a7dc0`.
The complete helper can read a 32-bit value through archive+0x08 and invoke
`0x1407a8950`. That setter returns on equal index; otherwise it can call further
package/index helpers and writes O+0x3c. These are pre-tag effects and retained
reentry boundaries, not an assumed inert four-byte prefix.

The default serializer then increments dword **field** A+0x50. Under ordinary
loading A+0x14 nonzero and A+0x20 zero, it reaches the tagged route regardless
of global `0x142aee960`. Other archive profiles can select different serializers.
Field A+0x50 is distinct from virtual slot+0x50, which queries backing position.
With a valid nonnegative initial counter, no signed overflow and no callback
mutation, this unconditional increment makes it positive during the direct
tagged interval. It therefore supplies the selected bit 13 property-filter
condition previously described for a different enclosing serializer. Nested
tagged structs share the archive; other filter conditions still apply.

Archive virtual+0x48 is the verified leaf returning L. The subsequent lookup
`0x140766a10` uses the object's loader/index and the map at L+0x72c:

- Its complete read-only lookup returns null immediately when dword L+0x770 is
  zero. Nonzero metadata performs a bucket/chain lookup; finite valid metadata
  is required, and no corruption bound is invented for the native loop.
- Fresh loader construction clears L+0x72c and L+0x770. This is a checkpoint.
  No later map-write audit establishes that it stays empty during package load.
- A null lookup calls the retained tagged loader directly at `0x1407528d0`,
  with class C, original archive A, target O, C.super, O.archetype and zero.
- A nonnull lookup passes the stored map value to stack-archive constructor
  `0x140739f40`, installs a linker context and invokes the tagged loader at
  `0x140752891` using that alternate archive. The original archive is separately
  sought to the export end. Constructor input representation and transitive
  effects remain unexpanded; cleanup can free the alternate archive's buffer.

The nonnull path is not proven to contain the original CDO bytes or to delegate
to the original backing reader. Accordingly, the original saved record's
absence of StartPaused **does not cover the alternate-input branch**. No patch,
configuration or other semantic name is assigned to that map without evidence.

## Four nonnull property references

The selected default's nine object-valued occurrences include five nulls and
four nonnull references. Its one MoviePlayerClass value is GFxUI export reference 37;
three ExpectedType values are GFxUI import reference -30, Engine.SeqVar_Object. The signed
reference dispatcher sends these to export index 36 at `0x140760610` and import
index 29 at `0x140761b50`, using the same GFxUI loader.

Both valid-index nonnull-cache branches return without helper calls or writes
through the returned object pointer. They do not validate its identity or
initialization state. They also have observable global effects: the export
branch sets and clears diagnostic context; the import branch increments and
decrements depth and conditionally sets/clears its surrounding context. Neither
is globally pure or protected by a visible lock on this branch.

Warm-cache admission requires valid stable tables, the same loader and index,
stable nonnull slots, valid target identity/nonaliasing and consistent depth.
The archive property reader's subsequent qword destination write remains a
separate operation. Cold resolver behavior is not admitted by the warm proof.

Fresh declaration reads sharpen the earlier-cache candidates:

| Declaration | Actual subtype tail target | Relationship to saved value |
| --- | --- | --- |
| GFxUI MoviePlayerClass, export index 141 | GFxUI export reference 37, GFxMoviePlayer | Same package, loader reference and target |
| Engine SeqVarLink.ExpectedType, export index 2888 | Engine export reference 15869, SequenceVariable | Different target from GFxUI import reference -30, SeqVar_Object |

The exact ClassProperty metadata serializer reads the two subtype references
through archive+0x30 after the UProperty base. MoviePlayerClass metadata thus
invokes the same export reference 37 resolver before that metadata serializer returns.
Successful nonnull resolution, ordering before the CDO's tags and preservation
of that cache still require a lifecycle join. Metadata return alone is not a
warm-cache proof.

The selected 70-node, 99-edge schema has no explicit SeqVar_Object dependency.
That bounded absence is not a whole-engine assertion. Neither loading
SequenceVariable nor resolving a same-named target through another package
establishes the GFxUI import index 29 cache. Its first selected reference remains a
cold/reentry boundary; three repeated values do not prove cache persistence.

## Type caches and registration readiness

Tagged Boolean comparison uses cache `0x142c23cd8`; offset-zero Array cleanup
uses the distinct native Class cache `0x142b186f8`. Both cold paths publish their
pointer **before registration completes**. Nonnull therefore does not establish
a completed descriptor or exclude reentry.

Complete common registration `0x1407490f0` establishes the Class cache before
assigning child+0x50, regardless of its later registration predicate. The cold
ScriptStruct registration in recomputing UStructLink reaches this checkpoint.
Completed successful registration, unchanged cache identities and absence of
invalidating reentry/reset permit that checkpoint to be used later. A nonnull
ScriptStruct cache or existing typed metadata alone does not establish it.

The exact BoolProperty instance constructor, Link method and direct metadata
serializer do not access the Bool cache. Their existence therefore cannot
exclude the tagged helper's cold branch. That helper snapshots P.class before
cold registration, then compares the saved pointer with the reloaded cache.
Its complete registration helper can reach common class registration and the
class virtual+0x118 path, including default retrieval. Allocation, ancillary
factories, base construction and target reentry remain real dependencies.

## Final None through normal preload return

On the direct original-archive route, the retained final-None epilogue returns
to `0x1407528d5`. The rest of the default serializer only decrements field
A+0x50, restores the frame and returns. There is no subsequent call, target
field write or omitted-property sweep in that interval.

The enclosing preload then queries current position and checks the consumed
export size. A mismatch enters a substantial diagnostic/allocator path that
receives the object; it is not treated as a guaranteed harmless return. On a
matching size it seeks back to the old backing position and restores the prior
backing pointer. Under current CDO flag 0x200 and global `0x142aee960 == 0`, it
calls `0x14079de30(O,0,0,0,0)` and `0x14079f4e0(O,0,1)` before returning.

The selected action class flags 0x08000092 clear configuration bit 0x4 and
localization bit 0x20. Under unchanged metadata, the first helper only updates
global `0x142c23c80` before exiting; the second exits before its argument 3 test.
The three selected player classes have flags 0x08000096 and **set bit 0x4**,
admitting real configuration/property processing. Their default state is not
covered by the action's exclusion. No configuration contents were read.

The complete preload body has no direct target field write after the serializer
returns. That local fact does not make its backing/diagnostic/finalizer calls
pure. A skipped finalizer branch can admit another default-getter/recursive
preload path. Ordinary ABI-preserving returns, exact action identity, stable
flags/global state, matched byte count and valid disjoint storage remain part
of the selected normal-return contract.

Consequently, the strict fresh-zero, inherited-copy, reinitialization,
original-tag and matched-size action route preserves the StartPaused mask
through **this conditional Preload-return checkpoint**, if all stated callback,
cache, ownership and no-reentry conditions hold. This does not establish an
effective loaded action value: the alternate-data/cold-reference joins and
subsequent ConditionalPostLoad are not closed. No player/action instance
overrides or runtime invocation are merged into this conclusion.

## Verification and provenance

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-movie-default-load-lifecycle/`.
The aggregate `receipt.json` seals component digests and source identity.
Component byte counts overlap and must not be added as distinct native coverage.

| Component | Focused verification |
| --- | --- |
| package | Two 48-byte declarations, four subtype reference words, 13 identity/outer records; fresh/retained comparison; source hashes unchanged |
| references | 1,294 exact code bytes, eight slot bytes, relative branch checks and three conditional call-free warm paths |
| type-caches | 6,577 distinct bytes / 1,500 instructions within that component; constructor/registration/cold-path boundaries |
| preload | 2,890 bytes / 714 instructions plus 16 slot bytes; separate 5,686-byte retained finalizer audit |
| reinitialization | 1,602 bytes / 394 instructions; nine header intervals, two zero-header paths and three rejected out-of-profile cases |

An independent reinitialization audit checks those same 1,602 bytes using its
own PE mapper and confirms the conditional source guards, header intervals and
both zero-header cleanup branches. This overlap is not additional byte coverage.

Four new bounded target-free REA records were exported and reset across the
reference, Bool-helper and preload lanes. Earlier retained complete records
were rechecked against the pinned PE. No native process or source file was
modified. Preliminary verifier corrections are retained privately: one reader
needed a JSON `rows` envelope; another rejected an extracted carrier's digest
being confused with the whole PE, and excluded an incomplete duplicate record.
Final receipts do not turn those failed attempts into passed checks.

Every outside input used here is recorded in the component ledgers/verification
sources and the aggregate source ledger. The source/provenance scope is:

| Source | Licence / permitted use here | Use |
| --- | --- | --- |
| Owner's 1.52.0.1700 Win64 executable, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner install; private compatibility study | Exact selected native methods, tables, branches and globals |
| Owner's GFxUI.upk and Engine.upk in the verified private pin | Same proprietary owner assets | Selected declaration words and target identities; no game bytes committed |
| Original project evidence: fresh-movie-default-state/copy/serialization, world-transform-defaults/copy-inner, scale9-texture-load, frontend-movie-defaults and rea-class-registry | Original local study and private proprietary evidence; redistribution not authorized | Retained REA, schema, layout, PE mapping and source identity; exact paths/hashes retained privately |
| Original project documents linked above; FRONTEND_MOVIE_DEFAULTS, SCALE9_TEXTURE_LOAD and WORLD_TRANSFORM_DEFAULTS | Original local project work | Existing conditional contracts and source context |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`; rea-agents 6.9.0 | MIT | Bounded target-free acquisition and Evidence handling |
| Rizin; GNU objdump 2.46; Python; rustc | Tool-only use: LGPL-3.0-only; GPLv3-or-later; PSF; MIT/Apache-2.0 respectively | Native inspection, original verification scripts and bounded declaration extraction |

No outside implementation code was copied. Tool licences do not grant rights
to redistribute the original client or extracted content. There is no effective
default, current runtime, wire behavior or visual-parity claim in this study.
