# Attached target initialization checkpoints

**VERIFIED static checkpoints:** the concrete base constructor chain establishes
target flags `0x4000 → 0x4100 → 0x4180 → 0x4380` at four named returns. These are
useful inputs to a future faithful construction path. They do **not** establish
the current flags of an adopted login instance. Source construction, owner
binding, placement, publication and action delivery occur afterward. The actual
publication path clears the saved-action skip bit, sets other flags, and can
execute callbacks before later load admission.

No native client, VM, renderer or script was executed for this study. VERIFIED
means that selected complete instruction ranges and pointer witnesses were
checked against the pinned executable. OBSERVED below describes their static
control flow. The proposed integration boundary is INFERRED from that evidence.

## Identity and scope

Let `T` be the display target, `O` its concrete AS2 owner, and `M` the retained
MovieImpl. The examined factory is `0x140178440..0x1401786d0`:

| Factory case | Allocation | Target constructor / primary table | Owner constructor / primary table |
| --- | --- | --- | --- |
| 3, source Sprite/MovieClip route | `0x348` bytes | `0x1400b6d40` / `0x1425069d0` | `0x140203560` / `0x142517670` |
| 4, EditText route | `0x1b0` bytes | `0x140143880` / `0x14250a800` | `0x140204a40` / `0x142517970` |

Both examined owners begin at `T+0x148`; their script interfaces are `O+8`.
Their owner categories and Environment routes differ. This is not a generic
constructor profile for every display category.

The saved login inventory in [LOGIN_INSTANCE_INPUTS.md](LOGIN_INSTANCE_INPUTS.md)
contains one root, 74 Sprites, 33 EditText instances and 39 Shape leaves. It
provides exact source/placement inputs, not native flags. In particular, saved
visibility and the current effective visibility of a descendant do not supply
the native target's local bit 14. The root/Sprite conditional binding profile
also does not establish the EditText owner's Environment.

`T+0x6a` must be described with its access width. Its low WORD contains the
flags discussed here, but a DWORD read also includes bytes `+0x6c` and
`+0x6d`. The latter is the owner offset. `0x140099640` stores
`(O-T+3)>>2` in that byte; the two concrete allocations yield `0x52`.
Consequently an owner-bound DWORD includes `0x52000000` from that byte alone.
An OR on the low WORD does not initialize the entire DWORD. Prior controlled
fixture value `0x52000180` remains a supplied fixture value, not a newly proved
native default.

## Complete base-return checkpoints

These results assume the valid, distinct target/definition allocation inputs
used by the recovered constructor path. The optional definition refcount leaf
and the child-list initializer were checked rather than presumed harmless.

| Complete constructor | Return instruction | DWORD `T+0x6a` | WORD `T+0x88` | Other established state |
| --- | --- | --- | --- | --- |
| `0x14009c270` | `0x14009c2e6` | `0x4000` | Not initialized here | `T+0x48 = 0`; owner-offset byte zero |
| `0x14009e6d0` | `0x14009e72d` | `0x4100` | `0x000a` | Handle-related fields `+0x70/+0x78/+0x80` cleared |
| `0x1400a1130` | `0x1400a11e9` | `0x4180` | `0x000a` | QWORD `T+0xb8 = 0x10`; selected MovieDef at `+0xb0` |
| `0x1400b4dd0` | `0x1400b4e32` | `0x4380` | `0x000a` | Same `+0xb8`; fresh child-list count `T+0xe0 = 0` |

The first function writes DWORD `0x4000` at `0x14009c2cb`. Its preceding
`T+0x48 = 0` store and read exclude the conditional render-node visibility call
on this fresh path. The successive low-WORD ORs occur at `0x14009e715`,
`0x1400a11ad` and `0x1400b4e26`. The `+0x88` initializer is at `0x14009e71e`.

The complete 11-byte `0x140031ce0` only increments the distinct definition's
reference count. The complete 28-byte `0x140130b30(T+0xd8)` initializes the
child-list object through `T+0x100`, without overlapping these flags. Those
small call closures permit return-value claims for these base constructors.
The same reasoning has **not** established a complete subtype/attachment return
invariant. In particular, local visibility bit `0x4000` is set at these base
returns, while mask `0x400` and saved-skip mask `0x10` are clear there.

## Source target and owner construction

**OBSERVED source path:** `0x1400b6d40` calls the last base above, binds its
source definition at `T+0x110`, and writes WORD `T+0x140 = 0` at
`0x1400b6de1`. It ORs low-WORD `T+0x6a` with `0x400` at `0x1400b6eab`.
The source byte at `+0x140` then sets bit 0, clears bits 1 and 5, and assigns
bit 4 from the final raw Boolean constructor argument. Factory case 3 supplies
zero for that argument. This `+0x140` bit 4 is a different field from the
saved-action skip bit at `+0x6a`.

Source category `0x8400` supplies its saved Scale9 rectangle through
`0x1403cd2a0`. That helper sets target `+0x6a` bit 0 for the ordered positive
extent branch and clears it for the alternative branch. Changed geometry
invokes target virtual `+0x90`, resolved here to `0x1400b38c0`. The verified
fresh child-list count is zero, excluding that helper's descendant loop during
this construction path. The render-node helper `0x14009a760` reads target bit
14 to initialize a separate node; target and node flag offsets are not
interchangeable. Source-category and floating-point branch conditions must be
retained rather than collapsed into a universal bit-0 default.

Parent/resource virtual calls and conditional resource-context helper
`0x1400b3390` remain later boundaries. The latter directly writes `T+0x108`
but also calls resource/root helpers. A direct-store inventory is not proof
that all transitive effects preserve every target field.

The owner constructor `0x140203560` binds `O=T+0x148`, thereby setting
`T+0x6d=0x52`, initializes its embedded Environment and points that Environment
back to `T`. It clears source `T+0x140` bit 5 and ORs low-WORD `T+0x6a` with
`0x800` at `0x14020366a`. It also caches source version, allocates a per-frame
initializer ledger, and performs dynamic MovieClip prototype selection through
`0x14019b570`. This prototype is not an unconditional fixed default. The full
owner's resource/prototype call closure is not established by this study.

## TextField is a separate subtype

The EditText constructor calls the common `0x1400a1130` base, not the
Sprite child-list wrapper. Its complete body has no later direct write to
`T+0x6a` or `T+0x88`, but includes text, state and resource calls. Therefore
`0x4180`/`0x000a` are established at its **base return**, not asserted as its
final return or publication values.

At `0x140143938`, BYTE `T+0x140` is set to `0x0e`. Here the byte belongs to
an embedded listener at `T+0x130`, not to the Sprite source-flags object.
A state-query branch can OR bit 0 at `0x14014396d`. Text-view offsets such as
view `+0x88` or view `+0x140` likewise must not be mistaken for target offsets.
There is no valid generic shared interpretation of the two subtypes' `+0x140`.

The actual source EditText 1034's saved flags `0xad31` become source-definition
WORD `+0x80 = 0x020a` on the retained parser path. Its border bit is clear,
excluding the constructor's border render-node branch. This is one exact
source witness; it supplies neither every login field's current text nor a
complete constructor call closure. A previously retained 85-byte range at
`0x14013d810` is only a fragment and cannot establish the helper's return.

Owner `0x140204a40` binds the same physical owner offset but has category 4
and no embedded MovieClip Environment. Its Environment accessor
`0x14017a130` follows target parents until it finds low-WORD mask `0x400`.
Thus the selected ancestor and its actual flags are required; source movie
version, a field's filename or case-3 defaults cannot supply that identity.
The lookup starts at the parent, not at the field itself. If the selected
ancestor's accessor returns null, it does not retry a more distant ancestor.

## Child publication and action delivery

The complete concrete attachment caller is
`0x14020d8d0..0x14020de66`. The following ordering is OBSERVED on its new-target
path; existing-item paths are separate. Argument numbering here includes the
receiver, so raw argument 7 is at entry `RSP+0x38`.

1. After construction, it clears target low-WORD bit 0 at `0x14020dc53`.
   Ancestor geometry can set the bit again, followed by target virtual `+0x90`.
   Naming and an optional target virtual occur before the subsequent flags.
2. Raw argument 7 bit 2 clear invokes target virtual `+0x198` with zero. The
   case-3 helper `0x14009ea20` clears `T+0x88` bit 3 at **`0x14009eace`**;
   this is the exact store address, refining an earlier documentation address
   `0x14009eac9`. It also sets/clears `+0x88` bit 4 from **current**
   `M+0x4310` bit 13. The alternate argument branch ORs `T+0x88` with 1.
3. Construction producer `0x140203920` returns a token; more target helpers
   run before child publication `0x140132200` at `0x14020dd67`.
4. Publication applies placement inputs and child-list work. At
   **`0x140132319` it ANDs WORD `T+0x6a` with `0xefef`**, clearing masks
   **`0x10` and `0x1000`** while preserving other current bits at that
   instruction. It then tail-dispatches target virtual `+0x240`.
5. For case 3, this slot resolves to thunk `0x1400a0e00`, then
   `0x1400996e0`. The latter ORs `T+0x6a` with `0x2000` at `0x1400996e9`
   and dispatches owner primary `+0x48`. The case-3 owner target is
   `0x1401cbec0`, which queues a load record, ORs target flags with `0x20`
   at `0x1401cbf9b`, and submits frame zero. These are additional state
   changes after the publication mask, not proof of action completion.
6. An optional root virtual runs. The caller then captures eligibility from
   target `+0x6a` bit 10 at `0x14020dd95..0x14020dda8`, **before** the
   construction-token filtered drain at `0x14020ddbf`.
7. With argument bit 2 clear, that drain may execute callbacks and registered
   constructors. Afterward, the previously captured eligible target is tested
   for load-event presence; success ORs source `T+0x140` bit 7. The alternate
   argument branch bypasses this drain/predicate. A false predicate does not
   clear a previously set bit. Further target work precedes return.

Case 4 has a different publication slot: `0x140146ce0`. It calls text helpers
before the common `0x1400a0e00` continuation and may call a further target
helper afterward. Its owner primary `+0x48` resolves to **`0x140204d70`**,
not the Sprite callback. That body handles source text and optional variable
binding, calls `0x1401463b0`/`0x140202930` on relevant branches, and dispatches
text-interface methods. It has no direct write to the three target fields
examined here; their transitive preservation remains unproved. The case-3
load-queue and source-byte semantics must not be copied onto TextField.

The saved event wrapper described in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md) checks current target `+0x6a`
bit 4. Its onLoad path clears bit 5 and then consults `+0x88` bit 3 or source
`+0x140` bit 7. Therefore even a publication step that just set bit 5 cannot
justify a permanent admission-state constant. Presence, admission, queued work,
execution and successful completion are separate checkpoints.

## Root / level-zero route

Root initializer `0x140210180` obtains a target through the factory, calls
`0x1400b5150(target,1)` and `0x1401871a0(owner,0)`, then passes that exact target
to `0x14005fdb0(M,0,T)`. The level helper publishes the array entry at
`0x14005feba`, invokes target virtual `+0x3e0`, and stores level-zero target
`M+0x50` at `0x14005fed5`. It also changes MovieImpl flags. These MovieImpl
flags are separate from the target's `+0x6a`.

Deferred advance later invokes target virtual `+0x378`. The concrete helper
`0x1400b6870` guards on source byte `+0x140` bit 3 and sets it at
`0x1400b688a` before frame-zero initializer, ordinary frame and load submissions.
The normal root drain follows in the retained advance window. The root path
contains unexpanded target/owner virtuals before and after publication; it is
not interchangeable with the child-placement checkpoint or proof of the actual
game's startup timing.

## What an implementation may admit

**INFERRED integration boundary:** a constructor implementation can start from
the exact four base checkpoints when it implements that corresponding path and
preserves the typed identities. It must then apply source, owner, placement and
publication transitions in order, with explicit inputs for their branch
conditions and current state. A snapshot-adoption API skips that lifecycle and
must continue to require coherently supplied current flags/state.

This study does not admit a universal post-attach raw DWORD, source byte, load
word or visibility Boolean for the saved login tree. Remaining concrete gaps
are the bounded subtype resource/prototype/text call closures, placement/raw
creation flags, current MovieImpl policy, selected parent/Environment state,
and actual callback effects. Missing saved fields are not evidence those gaps
are zero. The native visibility prerequisite
[study](ATTACHED_VISIBILITY.md#appendix-initialization-and-later-visibility-policy-writers)
separately establishes base `+0xb8` and MovieImpl initialization checkpoints;
neither substitutes for their current state after creation and callbacks.

## Evidence and provenance

The sole executable is the owner's proprietary build 1.52.0.1700 at
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Study is for compatibility/interoperability. No executable, disassembly,
asset or native implementation is committed.

Private receipts under `evidence/attached-target-initialization/` preserve
source paths/hashes, canonical REA output, original byte checkers and findings:

| Receipt | Scope |
| --- | --- |
| `base/receipt.json` | 836 unique instructions, 17 complete ranges from nine records, two table entries; named base returns and source/owner transition inventory |
| `publish/receipt.json` | Ten complete ranges and PE slot witnesses; child publication, distinct subtype continuations and separate root review |
| `textfield/receipt.json` | TextField target/view/owner identity, subtype checkpoints and explicit unresolved call closure |
| `receipt.json` | Immutable aggregate manifest, document hash and receipt readback |

The TextField receipt's original prose used `0x4` twice for the saved-skip
mask. Its separate `textfield/correction.json` corrects both to bit 4,
mask `0x10`, preserving the original sealed files. This document uses `0x10`.

New base record is
`ev_d471905fab2e0bdc1aeb127ee4db8274f10d9dfb94d748c6a890f20527601534`.
New publication records are
`ev_caa2545d365c592162336c9ceb9b911adf39ecf1464b10120ddf776ed7c3583c`
and `ev_46c7233d324733064d562b153aa94665b2e24dc32503352ba0a302fbcef16431`.
New TextField owner publication record is
`ev_41ba93a8508dd28773c15da610d99b6a0248a3c201bf0dee177e6868f3868cd8`.
Receipts index all reused canonical records, including complete base/source
constructors, owner bindings, visibility helpers and display lifecycle bodies.
Adjacent disassembly tails and incomplete `.pdata` fragments are excluded from
complete-body claims. Historical provider string-scan warnings are retained;
the selected instruction bytes are independently checked against the PE.

Additional sources are this project's original documents cited above,
[SCALE9_RENDERING.md](SCALE9_RENDERING.md), retained private original probes,
and the installed REA 6.9.0 skill/native/evidence tooling already catalogued in
[PROVENANCE.md](PROVENANCE.md). The tooling licence is not newly asserted and
no tooling code was copied. There was no external SDK/source read, new package,
runtime launch, shared REA target switch, configuration change, product edit or
native-pixel parity test.
