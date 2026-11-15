# Selected default input: loader map producer and archive choice

The concrete loader stage that can populate alternate CDO input is identified.
**Its input-provider function returns zero unconditionally in the pinned
executable.** The resulting branch skips the map-population block. This is an
exact native result, not an assumption that an optional data source is absent
from the owner install.

A bounded stage interval therefore preserves the selected lookup map under
the stated callback/ownership conditions. A universal constructor-to-CDO-load
claim is still not established: other intervening stages, recursive callbacks
and the actual selected preload binding are not all closed. Effective StartPaused
remains unassigned. This study makes no native execution, configuration, server
or product change.

`VERIFIED` means exact private source bytes, selected bindings and bounded
correspondence. `OBSERVED` denotes static instructions or retained saved data.
Conditional composition is not an observed runtime sequence. The previous
[preload lifecycle](FRESH_MOVIE_DEFAULT_LOAD_LIFECYCLE.md) and
[PostLoad study](FRESH_MOVIE_DEFAULT_POSTLOAD.md) retain their separate limits.

## Exact input selector

For loader L, the selected embedded archive is A=L+0x1c8. The dedicated CDO
serializer `0x140752720` checks archive+0x48 for nonnull and invokes
`0x140766a10` with the object's linker/export index. Identifying that linker
with L requires the surrounding same-linker preload condition. The thunk
accesses its linker's map at +0x72c. Its complete lookup returns null immediately when the control word
at L+0x770 is zero. This word is called the lookup-control field here; a
logical element-count identity is not invented from the zero test.

Fresh constructor `0x14075a490` zeros that map's storage and control fields.
This remains a constructor checkpoint. A separate constructor also establishes
zero fields; neither checkpoint proves arbitrary later state. A null lookup
selects tagged serialization from the original archive. Nonnull lookup selects
a distinct archive constructed from the stored record, described below.

The selected CDO path remains `79d590 → 752720`, not ordinary object virtual
serialization through `7a7720/7a7e90`. The existing original-archive final-None
and post-serialization proofs are not replaced with a different caller route.

## Concrete population site and the pinned zero provider

Complete 2,615-byte stage `0x14076bdf0` contains a map insertion at
`0x14076c4e2`. The skipped producer builds a key/value pair using a zero-based
export index and a pointer to an input record. The selected input-record address
is `qword[D+0x48] + 40*i`. It filters/matches export metadata, including object flag 0x200,
before passing the pair and L+0x72c to `0x140bf2920`.

The map insertion helper is independently identified over its complete
407-byte chained unwind extent. Its allocator/rehash effects are not assumed
pure and need not be expanded to exclude this particular caller: the pinned
stage never reaches that insertion from its normal entry.

At `0x14076becd`, the stage calls `0x1407cf2b0`. That complete 19-byte function
only performs local stack bookkeeping, sets EAX to 0 and returns. It ignores
its incoming arguments, makes no calls, writes no output through its arguments
and does not inspect a file, package name or runtime option. The caller tests
that result at `0x14076bed2`; the zero branch at `0x14076bed4` goes directly to
`0x14076c723`, beyond the population block.

This also skips the block's alternate backing construction and other input
processing. The selected result does not depend on cache warmth or a guessed
boolean supplied by a host. No binary patch, launch flag or runtime bypass is
involved: these are the original pinned instructions.

The stage has a second admission: when all six progress flags at
L+0x698, +0x69c, +0x6a0, +0x6a4, +0x6a8 and +0x6ac are already nonzero,
its entry skips the provider and population work. A prior nonzero map would
not be erased by either route. The proof is preservation of the existing map,
not a stage-wide reset that manufactures emptiness.

## Complete selected stage interval and remaining calls

The exact zero-provider control-flow graph has 120 reachable stage instructions.
Its direct loader writes set the six progress flags above to 1. It does not
access the selected map or invoke its insertion helper. The skipped branches
must not be used to characterize a different executable or modified provider.

Work before the zero-return call remains real:

- A valid package pointer supplies a name value, or the stage invokes the
  existing name helper `0x140793e60` to construct its fallback name.
- A null global holder may allocate 16 bytes through `0x140721150`, initialize
  them with `0x14059c400`, and publish the result to global `0x142b18f00`.

The holder constructor is an exact 13-byte leaf: it zeroes its 16-byte receiver
and returns it. It has no registration, virtual dispatch or nested call.
Allocation failure skips that leaf; the provider itself still does not
dereference its holder argument. Allocator and name-service effects nevertheless
remain normal-return/nonaliasing conditions, rather than being declared inert.

After setting progress flags, the stage may invoke a global object's virtual
slot+0x48, with integer arguments 4 and 6. L or its map is not passed explicitly,
but this does not prove that global callbacks cannot reenter the loader. The
flag gate is retained, not assigned a guessed current value.

Its suffix calls complete 143-byte `0x14076c930(L, literal, 1)`. That helper
increments L+0x6bc. If the stop field L+0x6b8 is already nonzero, or if it is
zero and timing-mode field L+0x6c0 is zero, it returns the stop field without
calls. Those admitted paths have one loader write, disjoint from the map.
The selected divisor argument 1 is retained; no arbitrary division safety is
assumed for other callers. The general timed path has an imported timer call
and further stop/global-counter effects, outside the call-free shortcut.

The synchronous factory/driver route establishes timing mode 0. With the stop
field still 0 at this suffix, the stage returns success. This is a conditional
join; callbacks are not assumed to preserve those fields merely because a
fresh constructor ran earlier. The map-preservation result is separate from
whether this particular stage invocation completes or is retried.

Thus, from a valid stage entry through an ordinary return, with the selected
provider unchanged and intervening services confined, the map's existing
control/storage state is preserved. If its control word is zero at this entry,
it remains zero at this exit. This is the finite preservation interval proved
here; it is not silently extended across unexamined caller intervals.

## Placement in loader creation and exact remaining binding

Complete stage driver `0x140772a90` reaches the population stage seventh.
The backing and package-summary stages are already identified; the remaining
earlier stages retain their exact call identities without invented semantics.
Every tested failed stage branches to a shared retry/return point;
the synchronous wrapper supplies timing mode 0 and retries incomplete work.
The `0x14076ed50` stage's result is not tested before the next call.
The wrapper discards the driver's returned status before returning its loader
pointer. That pointer alone is not a certificate of completed fresh loading.

The late-stage study identifies a concrete completion checkpoint:

- Stage 15, `0x140766d10`, can iterate export indices through `0x140766a30`,
  under its global/package/virtual-call gates. Its own completion flag is
  L+0x68c.
- Complete `0x140766a30` performs export lookup/binding: it can reuse a cached
  object, recursively resolve an outer, initialize class metadata, call object
  lookup and invoke a found object's virtual+0x58 with the loader/index. It is
  **not a direct call to CDO preload**. Its callbacks/reentry remain explicit.
- Final stage `0x140766060` sets L+0x690 to 1. The driver subsequently treats
  that marker as complete on a new invocation. Its timing/return behavior is
  distinct from merely observing that store.

The bound fresh route therefore has a known zero-map constructor checkpoint,
an exact stage that cannot populate the map through its compiled provider, and
a known loader-completion marker. It does not yet provide a complete identity/
ordering/noninterference proof from constructor return to the selected GFxUI
action CDO's `76d640 → 752720` invocation. In particular, intermediate stages
and found-object callbacks cannot be replaced with an assumed no-write interval.

Targeted displacement discovery also found another constructor checkpoint,
member cleanup and a reset/detach method around this map. Their observed local
zeroing/reset operations do not supply alternate CDO data. Cleanup callbacks,
object lifetime and possible reentry remain distinct from a successful fresh
loading path. The reset method does not zero L+0x770: it uses that control field
to fill existing bucket heads with -1 and retains the zero-size reallocator's
return without assuming it is null. This is distinct from constructor emptiness.
A raw displacement inventory is not exhaustive alias analysis:
copied/interior pointers, generic memory operations and arbitrary callbacks are
not excluded just because no additional literal+0x72c reference was selected.

The narrow remaining binding is therefore explicit: establish this particular
live loader's fresh identity and map state across the remaining callbacks to
the selected lookup, or establish another finite boundary that dominates it.
Neither source-file absence nor the stage-completion marker alone proves that.

## Adjacent concrete result: the alternate archive's actual input

Although the known producer is bypassed, the nonnull-map serializer branch is
now characterized independently. Constructor `0x140739f40` receives the stored
record pointer P and, for a valid positive count, copies the byte array whose
header starts at P+0x18 into new archive S storage. Nonpositive counts take an
empty-buffer path; they are not thereby admitted as valid serialized objects.
It does not simply borrow the original package backing.
The copied data pointer/count/capacity are held at S+0x88/+0x90/+0x94;
cursor S+0x98 starts at 0. The caller subsequently assigns the object's linker,
`qword[O+0x28]`, at S+0x9c. The serializer's earlier archive+0x48 result is only
tested for nonnull there; the surrounding same-linker preload condition remains
necessary for identifying both with the selected L.

The constructor establishes loading flag S+0x14=1, S+0x1c=1, S+0x28=0,
S+0x4c=0, S+0x50=0 and S+0x70=0. Version-related fields come from three globals,
not from the original archive. Their pinned initial data values 868/3078/3 are
not observations of current runtime globals. Copy allocation, source counts,
ownership and source stability remain explicit conditions.

The concrete raw reader copies requested bytes from the owned buffer plus its
cursor and advances the 32-bit cursor. Its local check covers the starting
cursor, not a full cursor-plus-request bound. This native behavior is not a
justification for unsafe Rust reads or admitting truncated input.

Before tagged serialization, the selected prefix helper requests a four-byte
NetIndex transfer under these archive flags. A supported copied payload must
contain that complete prefix. Its possible
object-index setter effects remain separate. The original archive's counter
increment is not copied into S. The differing flags also mean the original
tag-filter profile cannot be transferred wholesale to this archive.

The alternate content's exact source is P's byte array. It is not proven equal
to the original saved CDO export, and no patch/configuration semantic name is
assigned to that record without further evidence. Consequently original saved
StartPaused absence still does not cover an arbitrary nonnull-map route.

## Verification and provenance

Private artifacts are under
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-movie-default-input-route/`.
The aggregate `receipt.json` seals the component digests, original source
identities and this document. Coverage totals overlap across components and
must not be added as distinct native bytes.

The checks include exact stage/provider/insertion bytes and unwind chains,
the selected zero-result CFG, nearby map construction/reset/cleanup references,
retained driver/factory ordering, exact late-stage completion/binding methods,
alternate-reader copying/flags/table bindings, and the 156-byte boundary study.
The latter verifies two call-free suffix profiles and rejects reuse when timed
mode introduces unproved branches. No native game execution is represented by
these static checks.

Every outside input is listed in the component ledgers and aggregate source
ledger, with exact paths/hashes where applicable:

| Source | Licence / permitted use | Contribution |
| --- | --- | --- |
| Owner's 1.52.0.1700 Win64 executable, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner install; private interoperability study | Selected instructions, literals, tables, globals and unwind metadata |
| Original scale9-texture-load and fresh-movie-default-load-lifecycle Evidence | Original private study of the same proprietary binary | Constructors, stage driver, archive identity, input selector and prior normal-return bounds |
| Original frontend-initial-instance and rea-class-registry helpers/evidence | Original local analysis work; underlying owner bytes remain private | PE mapping, bounded candidate extraction and source correspondence |
| Linked project documents and SCALE9_TEXTURE_LOAD | Original local research | Existing conditional contracts and source pointers |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`; rea-agents 6.9.0 | MIT | Bounded target-free capture and export/reset workflow |
| Rizin; GNU objdump 2.46; Python | Tool-only LGPL-3.0-only; GPLv3-or-later; PSF respectively | Inspection and original focused verification |

No external implementation was copied. Original client bytes, extracted
records and raw native output remain private. This result changes no effective
default and makes no current-runtime, gameplay or visual-parity claim.
