# Fresh movie default construction

This study grounds the native fresh-allocation checkpoint and the selected
action's inherited layout. It does **not** turn an omitted saved tag into an
effective default. No game process, sequence, service, configuration import or
original handler was executed.

`VERIFIED` below means exact owned-package/native-byte correspondence or the
specified bounded arithmetic. `OBSERVED` means a saved record or static
instruction. A computed layout is conditional on the stated native metadata
and ordering; it is not a live object inspection.

## Concrete result

The ordinary load path requests recalculation of the class layout. Replaying
the actual selected declarations with the recovered native type methods gives
Core.Object `0x60`, SequenceObject `0x98`, SequenceOp `0xec`, SequenceAction
`0x108`, and GFxAction_OpenMovie `0x150`. The action's five Boolean declarations
share the word at `A+0x120`; StartPaused has mask `4`. This independently agrees
with the bit read by native activation.

The seven inherited properties admitted to the second default-copy pass are
five arrays and two strings. Each has one 16-byte inline header; all seven
intervals end by `A+0x108`, before the action flag word. Their direct header
writes are bounded separately from callbacks operating on allocated elements.
Nested element callbacks and the complete subsequent default-load lifecycle
are not silently assumed to preserve all object state.

For the base GFxMoviePlayer's fresh construction, the selected parent is
Core.Object. Its conditional extent is `0x60` and its inherited bit22 copy list
is empty. This removes the inherited property-copy dependency for the
`P+0x114` flag word on that specific route. Derived Marvel players copy their
own parent's defaults; they do not inherit this empty-list conclusion.

No product fallback or effective-default API was added. The saved class/default
provenance reader remains unchanged.

## Fresh route and checkpoint boundaries

The retained resolver `0x140745a20` conditionally creates a missing class default
using the superclass default as template and object flags `0x20400000200`.
It calls the generic wrapper `0x1407a9f00`, whose allocator is `0x1407a8d20`.
Cache hits and existing-object reuse are distinct paths.

On the strict branch where lookup found no existing object and allocation
succeeds, `0x1407a9cf8` calls the imported zero-fill operation with the loaded
class extent. A target byte covered by that extent is zero immediately after
this call. The allocator then initializes the header and calls the default-copy
wrapper at `0x1407a9e36`. Only after allocation/copy returns does the outer
wrapper invoke the native constructor at `0x1407aa087` or `0x1407aa0bd`.

The optional contexts are not interchangeable:

- The fresh getter's copy context `K` remains zero. This excludes the first
  special property-copy pass in `0x14079d6d0`.
- Object flag bit41 selects a null object context `O`.
- Extra context `X` begins null but can be synthesized by the wrapper when
  runtime class flag `0x800000` is set. Its effects require a separate check.
- With `X` null, the fresh header leaves object flag bit20 clear before the copy
  wrapper. Its prior-value cleanup pass is then skipped; it sets bit20 before
  calling the inner copier.

The selected native class constructor starts with flags `0x80` for the action
and `0x84` for the players. The saved action/player class flags are
`0x08000092`/`0x08000096`; all clear `0x800000`. These observed checkpoints do
not prove that no intervening runtime code changes a class flag.

The inner copier uses the explicit template's class extent when one is supplied.
Its bulk interval is `[0x60, extent)`. The second property pass tests only a
property's start offset against the extent, clears `dimension * element_size`
bytes, then invokes property virtual `+0x2c8`. Therefore an extent comparison
alone is insufficient: field length and callback footprint must also be known.

After constructor return, resolver finalization can call `0x140741410`,
`0x14079d590` and gated configuration/localization-related helpers. This study's
fresh-memory and copy checkpoints do not describe the result after those phases
or establish the full ordering of saved CDO export loading.

## Loading really recomputes the layout

The exact direct chain is UClass serializer `0x140750850` → UState serializer
`0x140751170` → UStruct serializer `0x140751330`. In the ordinary loading branch
(`archive+0x14 != 0`), the last serializer sets `R8d=1` at `0x140751d0f` and
invokes virtual `+0x278` at `0x140751d1b`. UClass table `0x141df82c8` selects
`0x14074a2d0`, which passes that argument to UStruct Link `0x14074a910`.

This happens before UClass's own saved flags are read into `class+0x124` at
`0x14075087d`. The named class factories' extent arguments are cross-checks,
not substitutes for the loaded layout:

| Class | Factory | Declared extent | Native constructor callback |
| --- | --- | --- | --- |
| Engine.SequenceAction | `0x140d34000` | `0x108` | `0x140d3f7a0` |
| GFxUI.GFxAction_OpenMovie | `0x14126dc80` | `0x150` | `0x141276000` |
| GFxUI.GFxMoviePlayer | `0x14126e200` | `0x1e4` | `0x141276180` |
| MarvelGame.MarvelGFxMoviePlayer | `0x1414e6380` | `0x264` | `0x1414ed240` |
| MarvelGame.MarvelGFxFrontEnd | `0x1413b8070` | `0x340` | `0x1413bce30` |

The already recovered constructor chain stores the factory's extent at
`class+0x88`. Nonzero Link mode subsequently resets/recomputes it from parent
and child properties. Zero Link mode instead restores prior offsets and Bool
masks; that different route is not used in this replay.

UProperty serializer `0x1407cbfc0` reads dimension into `+0x68` and the eight-byte
flags into `+0x70`. On load it clears the runtime offset `+0x8c` and copy-list
link `+0x98`. Array/Str Link methods assign element size16 and alignment4;
they set flag bit22 unless bit12 is present. Object/Class Link has a different
conditional bit22 update. These operations are included when selecting the
copy list; saved bit22 alone is not treated as the complete runtime rule.

## Actual parent declaration replay

The bounded package inspection walks four complete Children/Next chains:
SequenceAction, SequenceOp, SequenceObject and Core.Object. It reads 456 field
prefixes, including 48 property headers, using 8,448 bytes; it does not decode
the function bodies. Eighteen subtype-reference words add 72 bytes. All chains
terminate at null and all selected properties have dimension1.

The 13 Core.Object properties exactly match the prior independent
[loaded-layout study](SCALE9_LOADED_LAYOUT.md). Five use the saved Pointer
struct, one uses QWord. Their native name registration and size8/alignment4
rules are retained; no host pointer-size assumption or IntPointer substitution
was made. The new comparison checks all 520 header bytes and typed references.

Replaying Int4/4, Byte1/1, Name8/4, Object/Class8/4, Array/Str16/4 and the native
Bool packing rule yields:

| Declaring class | Incoming extent | Result |
| --- | --- | --- |
| SequenceObject | `0x60` | `0x98` |
| SequenceOp | `0x98` | `0xec` |
| SequenceAction | `0xec` | `0x108` |
| GFxAction_OpenMovie | `0x108` | `0x150` |

The seven inherited copy fields are:

| Property | Inline interval, end exclusive |
| --- | --- |
| SequenceObject.ObjName | `[0x6c, 0x7c)` |
| SequenceObject.ObjComment | `[0x7c, 0x8c)` |
| SequenceOp.InputLinks | `[0x9c, 0xac)` |
| SequenceOp.OutputLinks | `[0xac, 0xbc)` |
| SequenceOp.VariableLinks | `[0xbc, 0xcc)` |
| SequenceOp.EventLinks | `[0xcc, 0xdc)` |
| SequenceAction.Targets | `[0xf8, 0x108)` |

No inherited Bool enters that list. Core's Pointer/QWord declarations carry
bit12, preventing the relevant Struct Link addition. The selected Object/Class
fields do not satisfy their bit22-addition predicate.

The action's local properties give Movie at `0x108`, MoviePlayerClass at
`0x110`, MoviePlayer at `0x118`, then the Boolean word at `0x120`:

| Property | Mask |
| --- | --- |
| bTakeFocus | `1` |
| bCaptureInput | `2` |
| bStartPaused | `4` |
| bEnableGammaCorrection | `8` |
| bDisplayWithHudOff | `16` |

The replay requires exact concrete property classes, successful referenced
metadata loading, qualifying owner masks, and the recorded declaration order
remaining in effect after any native reorder/relink path. It is checked original
arithmetic over the saved records, not a runtime metadata snapshot.

## Selected copy callback footprints

Array table `0x141ded6a0` selects copy `0x1407bba70`; Str table `0x141ded368`
selects the element-copy wrapper `0x1407bc230`, whose selected scalar operation
forwards through `0x1407bc450` to string assignment `0x1406d5720`.

The inner copy's preceding 16-byte clear is significant: destination pointer,
count and capacity are zero at each callback. With the selected nonzero field
offset, Array cleanup `0x1407bc9d0` skips its metadata-initialization branch;
zero count/pointer exclude nested destruction and freeing. The inspected Array
and Str callbacks write pointer/count/capacity within the same 16-byte header.
Their data copies and clears use allocated pointer-backed storage. Source counts,
element sizes, multiplication, allocation success and non-aliasing are explicit
validity conditions, not Rust safety guarantees borrowed from native arithmetic.

The Array callback prepares capacity equal to the source count. On valid
nonnegative input, its selected append helpers cannot enter their growth path.
For a nonempty source whose inner property has bit22, however, it calls that
inner property's virtual `+0x2c8` at `0x1407bbb66`. Heap-element arguments do
not establish an arbitrary transitive callback's complete write footprint.
The five parent arrays have one ObjectProperty inner and four StructProperty
inners; their applicable nested copies were not expanded here. The allocator
wrapper also dispatches the actual allocator through virtual `+0x18`, possibly
after initialization. No live allocator/source-array state was observed.

Thus the seven field intervals now exclude the zero prelude and **direct inline
writes** from touching `A+0x120`. Full post-copy preservation remains separate
from that established local-write result.

## Constructors, saved tags and player flags

The complete action and SequenceAction constructor callbacks call the shared
base leaf `0x140793fd0` and install their vtable. They do not write the action
flag word. The GFxMoviePlayer base constructor `0x14125ea70` clears selected
native containers and wrapper `P+0x60`; its writes do not cover `P+0x114`.
MarvelGFxMoviePlayer and MarvelGFxFrontEnd add other container initialization,
also outside that word. Constructors preserve the word received from the copy
phase; their lack of a store is not an independent source of zero.

On the explicitly bounded fresh GFxMoviePlayer route with unchanged Core.Object
metadata, parent extent0x60, empty parent copy list, null extra context and no
intervening mutation, its covered flag word remains zero through the selected
constructor. This is a conditional pre-finalization checkpoint. It is not the
saved GFxMoviePlayer default, a derived player's default or a live movie state.

The new GFxUI header-only supplement checks 145 field prefixes / 60 properties
in 3,760 bytes. GFxMoviePlayer has 26 consecutive scalar Bool declarations,
beginning MovieIsOpen, DisplayWithHudOff, EnableGammaCorrection. All clear
saved property bit14 (`0x4000`) and bit22. The only bit14-set local player
property is SplitscreenLayoutYAdjust, an IntProperty. This narrows the direct
property-config predicate; it does not exclude other initialization writes.
The native MapProperty layouts preceding the Boolean run were not replayed
here, so this supplement does not claim a full independent player field layout.

Saved tags remain as documented in [the earlier movie-default study](FRONTEND_MOVIE_DEFAULTS.md):
the action default explicitly sets DisplayWithHudOff=true and omits StartPaused;
GFxMoviePlayer explicitly sets DisplayWithHudOff=true; MarvelGFxFrontEnd
explicitly sets it false. Native action activation later copies action bit4
to player bit1 and action bit3 to player bit2, then passes action bit2 to Start.
An action field and a player's class default are separate inputs with a real
write order; they must not be collapsed into one merged asset record.

## Remaining boundary

Absent StartPaused still is not admitted as an unconditional effective false.
The conditional linked layout and the inline copy intervals are now grounded.
The remaining proof must cover any selected nested array-element copy callbacks,
the actual template/default state, permitted extra-context effects, and every
later phase included in the claimed checkpoint. It must also connect that
checkpoint to the selected loaded CDO and subsequent instance construction.

The base player's fresh zero checkpoint does not remove the saved true/false
tags, class configuration, action writes or later movie initialization. No native
invocation, completed frontend bootstrap or retail behavioral parity follows
from this static study.

## Evidence and provenance

All paths below are beneath the private directory
`~/.local/share/marvel-heroes-forever-rust/evidence/`. No executable, package,
serialized value, decompilation or capture is committed with this document.

- `fresh-movie-default-state/native/first-two.bundle.json`: two complete
  target-free REA records, `ev_b8d19f15bf1b523c45d0878405f01af2e1c9569355b225ac206876068c0ea50e`
  and `ev_0915db6acc9bf899876b2f00a9983684a2f2f6cce798d01c45e2475674dba2a5`.
  `native/verification.json` checks 559 instructions / 2,321 code bytes and
  260 data bytes against the unchanged executable.
- `fresh-movie-default-state/link-mode/receipt.json`, SHA256
  `5c8c4a8a8341777694e90ae774d98fd3a26543614932d85e19774d878596b795`:
  new complete UState/UStruct coverage, 3,024 bytes, evidence
  `ev_ea41fab5042ab1b820ea72e5625d7f2492d228c3777122832986b47701587321`;
  exact UClass virtual-slot dispatch and retained Link checks.
- `fresh-movie-default-state/link-mode/core-addendum/receipt.json`, SHA256
  `e86f40903ad575bdb425093d9bf1866ab0ea20fbf6c52747bca540951fdbd7c0`:
  independent current/retained Core metadata comparison and 1,572 native code
  bytes / 110 data bytes checked. Its source ledger lists each reused REA ID.
- `fresh-movie-default-state/package/receipt.json`, SHA256
  `7ca0d57d3a76442dd46404b4670b071768d22a74da6289873a5fceebfa79a770`,
  and `package/player-action/receipt.json`, SHA256
  `bb6f096e576f0a5cf6cb728866797dc20c2cc1152dfae5f78bcf488970b0fcd8`:
  bounded actual declaration/CDO reads and independent prefix correspondence.
- `fresh-movie-default-state/fresh-copy/receipt.json`, SHA256
  `289c9cf1d59954fd68f7c4913aa085e7d46aa22d30d536af4ad1e42193862146`:
  fresh-route argument ABI, zeroing/copy order and exact list predicate.
- `fresh-movie-default-state/fresh-copy/footprint/receipt.json`, SHA256
  `3fe0482f01fff89e4c5a489bff4e9bd41b06e0ec8a1464880b146568bd9b2412`:
  selected direct callback footprint and remaining transitive boundary. New
  complete REA evidence `ev_f6ebb2ad0674fbae12cef98d88730ffbab7ec99638960f9b815c7283be42616e`
  covers six direct helpers / 712 bytes; the receipt also verifies reused spans.
- `fresh-movie-default-state/action-layout-replay.json`, SHA256
  `fa52762191398f9b89b8b2ee6aa7221707b8e9c37e73d3e616d2eb1dc54ac10d`:
  per-field arithmetic, masks, explicit conditions and source hashes.
- `fresh-movie-default-state/native/reused-constructors-verification.json`:
  five retained native spans, 1,809 bytes / 437 instructions, rechecked against
  the unchanged executable. Coverage totals above overlap and are not additive.

| Source | Licence / use |
| --- | --- |
| Owner's pinned MarvelHeroesOmega.exe, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner copy; static interoperability study of selected construction, serialization and copy operations |
| Owner's pinned Engine.upk, Core.upk and GFxUI.upk; exact paths/hashes in the two package receipts | Proprietary owner assets; selected class links, field prefixes, declaration order and saved default records |
| Retained MarvelGame.upk and MarvelFrontEndMap.upk metadata in `current-movie-defaults/package-audit/receipt.json` and `frontend-initial-instance/` | Proprietary owner assets; existing class/default and activation findings reused, no new execution |
| REA usage guide `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`; installed rea-agents toolkit | MIT toolkit; bounded target-free evidence handling and session export/reset workflow; no shared setup/config edits |
| Installed Rizin, invoked through REA | LGPL-3.0-only; selected static decoding. Provider version is not inferred from an absent identity probe |
| Installed GNU objdump | GPL-3.0-or-later; bounded ordinary candidate disassembly cross-checked by REA and PE bytes; no implementation copied |
| Prior UELib profile corroboration, [EliotVU/Unreal-Library](https://github.com/EliotVU/Unreal-Library), commit `8236c39b79e30887a487ecf9b018204e430a5ebc` | MIT; historical grammar provenance of the original class reader reused here; no new third-party source read or implementation copying |
| Original project evidence cited above and [CLASS_DEFAULTS.md](CLASS_DEFAULTS.md), [WORLD_TRANSFORM_DEFAULTS.md](WORLD_TRANSFORM_DEFAULTS.md), [SCALE9_TEXTURE_INPUTS.md](SCALE9_TEXTURE_INPUTS.md), [SCALE9_LOADED_LAYOUT.md](SCALE9_LOADED_LAYOUT.md) | Internal original research reused with explicit conditions; source-specific ledgers remain authoritative |

Native read sessions were exported and reset with no retained target. Focused
checks were byte identity, chain/reference bounds, source hashes and conditional
layout arithmetic. No product source changed, so no product build or full suite
was run for this documentation increment.
