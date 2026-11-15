# Attached MovieClip `_visible`

**VERIFIED static contract:** exact named `_visible` on the pinned build's
concrete case-3 attached MovieClip is native member **7**. A normally returning
normal assignment of Boolean false clears the receiver target's local visibility
bit `0x4000` at `+0x6a`. Its getter then returns Boolean false independently of
the parent's visibility. The setter also updates the target's rendering-node
visibility and can change separate status/list/child state. It is not an
ordinary script-member insertion.

This is implementation evidence from original bytes, not native execution or
permission to hide panels in a diagnostic. The four saved constructor writes
below remain subject to their actual script execution and receiver resolution.
No VM, display-state implementation or renderer behavior changes in this commit.

## Exact saved constructor sites

**OBSERVED owned script:** the manager movie with SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`
has DoInitAction tag 1479 for sprite 308, action span `1340411+15046`. Its
249-entry constant pool occupies `1340411+4013`. DefineFunction2 at
`1344454+293`, body `1344465+282`, has three registers and flags `0x19`;
the saved definition is assigned to `_global.MarvelLoginDialog`.

| Receiver member read from register 1 | Pool index | Original SetMember offset |
| --- | ---: | ---: |
| `compatLevelWarning_mc` | 6 | 1344534 |
| `legalDoc_mc` | 16 | 1344656 |
| `legalBG_mc` | 17 | 1344672 |
| `queuedLoginMessage_mc` | 18 | 1344688 |

Each 16-byte sequence pushes register 1 and that member name, performs
GetMember, then pushes pool entry 7 (`_visible`) and Boolean raw zero before
SetMember. The selected sequences start at 1344519, 1344641, 1344657 and
1344673. Existing original decoding was reused, and all 64 selected action bytes,
the pool and function header were read back from the owned source. Earlier
Point/property operations precede the later writes; this inspection does not
claim all four sites are reached in a running constructor.
The private movie is `evidence/frontend-startup/MarvelGame-frontend_manager.gfx`;
the reused original decode is under
`evidence/frontend-localization-runtime/textfield/acquire-followup/`.
Both paths are relative to the project's private data directory.

## Name, receiver and conversion

The original standard-name row at `0x142adcb10` points to exact `_visible` at
`0x14250d400` and stores ID 7 with selector 1. Name-map initialization,
classification flags and exact/normalized lookup remain grounded in
[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md) and
[ATTACHED_NAME_CLASSIFICATION.md](ATTACHED_NAME_CLASSIFICATION.md).
This study admits the exact name with that original authenticated map; it does
not add generalized spelling, replacement-map or other display-class behavior.

Let T be the concrete source target and O its attached owner. As established in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md), T has primary table
`0x1425069d0`, O is T+0x148 with primary table `0x142517670`, and O+0x20
points back to T. The script-facing object interface is O+8.

Normal setter `0x1401aa130` looks up the standard ID and calls O primary
virtual `+0x110` at `0x1401aa1e8`, passing O, ID, input Value and mode zero.
Concrete `0x1401c3e50` routes ID 7 through its `ID <= 21` branch to
`0x1401a8f20`. The base member mask from O virtual `+0x108` is all ones on
this concrete class, admitting bit 7. Table entry `0x1401a96cc` selects
`0x1401a9133`.

The base setter obtains the **receiver-owned** Environment through O virtual
`+0xf0`, concretely `0x140203720`, which returns O+0x38. At
`0x1401a9140`, `0x14017d730` receives the copied input Value and that
Environment. The admitted input is exact native Value tag 2 with canonical
Boolean payload zero or one at +8. Its copy path and conversion arm read that
payload directly; this Boolean arm does not inspect the Environment version or
coerce a string, number or object. The four original writes supply false.
Other Value types and conversion callbacks are outside this contract.

The converted Boolean is passed in EDX to T virtual `+0x1c0` at
`0x1401a914c`. The concrete slot at `0x142506b90` is `0x1400b6c00`.
After that method returns, the base jumps to handled=true at `0x1401a8fd5`;
the derived and normal setters return success without reaching ordinary lazy
storage, inherited data/accessor assignment or ordinary member-flag handling.
Existing ordinary storage does not redirect this native `_visible` assignment.
The selected native method still has the additional effects described below.

## Local visibility and rendering-node association

Case-3 method `0x1400b6c00` first calls base `0x14009ad30(T, value)`.
That 64-byte base setter updates the WORD at T+0x6a:

```text
false: flags' = flags & 0xbfff
true:  flags' = flags | 0x4000
```

Other bits of that WORD are preserved by this update. This is the same field
read by the getter and the existing native event-state model; it is separate
from the DWORD at T+0xb8 used later in this method. No initial visibility bit
or full initial flags word is supplied by this study.

For true, the base updates an existing node at T+0x48 if one is present. For
false, it calls `0x14009a760` to obtain or create that node, then sends false
to `0x1400a93e0`. The creation helper stores the concrete returned node in
T+0x48 and initializes its visibility from T's current bit 14. Thus this is
the target's associated node, not an unrelated scene or parent object.

Node visibility helper `0x1400a93e0` compares the requested Boolean with bit 0
of its current backing-data WORD at +0x0a. If unchanged, it skips the update.
Otherwise it obtains writable data through `0x14004a420(node, 4)`, changes
that bit and conditionally calls `0x140048980`. These are concrete retained
state changes and notification dependencies; this study does not replace them
with an assumed no-op or claim a completed GPU draw.

The concrete T virtual+8 producer uses the target's root/movie rendering context
to create the node. Allocation and reference-management details remain private
in the base-setter receipt. The false path assumes the returned node is usable;
there is no surfaced Boolean failure result for failed native allocation.
The post-write contract above therefore concerns normal completion, not a
promise to survive arbitrary allocation failure.

## Separate conditional status and descendant effects

After the base visibility update, `0x1400b6c00` computes a separate Boolean:

```text
hidden_status = !requested_visible && bit11(MovieImpl DWORD +0x4310)
```

MovieImpl is obtained through `[T+0x20]+0x10`. The method compares this value
with bit 3 of T's DWORD at +0xb8. If equal, it returns immediately from this
wrapper; the base visibility/rendering-node work has already occurred.
Otherwise it updates only that bit before continuing. The movie-level option
and prior bit are not inferred from a saved placement or the host renderer.

The changed-bit path then performs:

1. Call `0x1400b60d0` with T and the current predicate
   `bit21(T+0xb8) && !bit22(T+0xb8)`.
2. For result -1, set bit 22 at T+0xb8. For result 1, call `0x1400a1380`.
   Other results do not enter either branch.
3. If T+0x38 is non-null and that parent's +0xb8 bit 3 is clear, call
   T virtual+0x338, concretely `0x1400b61f0`.

The status helper is not a constant result. It reads T+0xb8 bits 2/3,
T+0x6a bit 6, a concrete target virtual, movie pointer membership and T+0x120
state. It can query the attached owner's event-presence predicate through its
secondary interface: O primary+0x10 returns O+0x30; secondary+0xf8 is
`0x14017a410`, which constructs native event mask 2 and invokes O primary+0x38.
The existing event-presence study covers that predicate's static-handler and
raw storage/prototype lookup. This is not a reason to invent its result from
visibility alone or claim a script event has executed.

`0x1400a1380` checks target/movie flags and list membership, then may splice T
into the movie's list at +0x4300, write target links +0xa0/+0xa8 and update
target +0xb8 bits 21, 22 and 23. Its guards and exact stores are retained in
the private native output. No scheduler meaning beyond those observed state
operations is required by this visibility contract.

The concrete descendant helper walks 16-byte child-list entries, selecting
children with +0x6a bit 7. It changes their **+0xb8 bit 3**, recursively invokes
child virtual+0x338, and calls child virtual+0x330 before the same conditional
list/status work. The selected body does not directly overwrite a child's
local visibility bit 14. Other child classes' virtual implementations,
arbitrary removal/rebinding and complete scheduling consequences remain separate
dependencies. This must not be collapsed into changing every child's `_visible`.

## Getter and implementation boundary

Normal and raw attached getters converge on `0x1401ddcd0`. Standard dispatch
calls primary+0x118 (`0x1401dd870`) and base `0x1401dce90`. ID-7 entry
`0x1401dd7fc` selects `0x1401dcfda`, which obtains T from O+0x20 and calls
T virtual+0x1c8. Concrete slot `0x142506b98` points to `0x140099bd0`:

```text
result = (u16[T+0x6a] >> 14) & 1
```

The selected getter releases any prior output Value, writes tag 2 and the
Boolean payload at +8, then returns handled=true through the base, derived and
outer getter. It does not probe ordinary `_visible` storage, named children or
the display prototype after that success. The leaf does not read the parent.

**INFERRED implementation guidance:** an exact named-Boolean bridge can share
the host's receiver-bound local Visible value with the numeric property route
once the attached target/host node association is established. Preserve the
distinction between local visibility, renderer-facing effective visibility and
the separate native status flags. A hidden ancestor does not make this getter
false for a locally visible child. The existing `display_state` subtree AND
calculation is a host rendering design; this study establishes the associated
native visibility state, not complete render-tree culling parity.

Implementing only local visibility must keep unimplemented conditional
status/list/child effects explicit. It cannot certify the complete native setter
for arbitrary movie options or live state. No guessed option value, default
flags, panel-name heuristic or synthetic write may substitute for the original
constructor reaching its actual SetMember operations.

## Bounded Rust getter

The typed attached-member reader now admits exact `_visible` and returns Bool
from the supplied current target-state bit 14. An installed name inventory must
classify that key as standard ID 7. The read precedes lazy storage, children
and prototype lookup, preserves the shared event-state word and creates no
ordinary property. It does not read effective parent visibility or acknowledge
renderer work. The named setter remains gated by the dependencies above.

**VERIFIED focused Rust check:** one test passes with no warnings, covering
opposite parent/child local states, no storage allocation, unchanged native
state, native precedence over an injected ordinary accessor, independently
updated current bits, invalid target and inconsistent native-name inventory.
The existing setter rejection is retained. Private terminal output and exact
source hash are in `attached-visibility/getter-test-result.json` and
`getter-rust-verification.json`, whose SHA-256 is
`051d3d42344aecb9187f84488a09b1d9a33b0f11f811ce1d1703659f4c8e0fd3`.
This is neither an original-action execution nor a live retail visibility test.

## Evidence and provenance

Primary native source is the owner's proprietary final build 1.52.0.1700:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses use preferred image base 0x140000000. Client bytecode, instruction
outputs and asset bytes remain private.

Evidence is under `~/.local/share/marvel-heroes-forever-rust/evidence/attached-visibility/`:

| Receipt/bundle | Scope |
| --- | --- |
| `main.bundle.json` | Named-setter case, concrete target wrapper, status/list/child helpers and concrete interface slots |
| `getter/receipt.json` | Exact ID/name, getter dispatch, Boolean output and local bit read |
| `base-setter/` | Base local-bit update, concrete rendering-node association and visibility notification dependencies |
| `script-sites/receipt.json` | Four exact original constructor writes; retained decoding and raw-byte readback |

New semantic claims use these canonical REA records:

| Evidence ID | Selected coverage |
| --- | --- |
| `ev_924dbce2c0f3bdb205a3ca75df6d3314b2fe2330dde2abeaad954518a2aafae5` | ID-7 setter case and complete concrete target method |
| `ev_7877c422fca71c290f43e1261bc73dc4dd1f1a6f647794e67b13300541398713` | Complete status and list-update helpers |
| `ev_02578b7166a301dc949748528ca766a9d54c98576aae82b3919cef0240a9f6c4` | Complete three-fragment child propagation body |
| `ev_639f7e8a738bd3d8618ec9d433a602cccf0c416fd0749695b410b62ad8b84212` | Complete status leaves, owner interface adjustment and event-presence adapter |
| `ev_ab786a1af1eb3d0a2e968fdf76d6bb9ea5e0986e0ec2e83634d4e26de539928e` | Getter case, local-bit leaf, table slots and success return |
| `ev_35d7807baffdee76f043e9c31553c4e2d32ea6f7476fee2dd6e44fe9fb4e09b8` | Complete base visibility setter, target-node getter/creator and backing-bit helper |
| `ev_8a8b85be3700a51fe519d197e99881e9626c20db6645434019c3cf538687dcb0` | Concrete rendering-node creator, initializer and node allocator |
| `ev_ae849635910b1d4d73f6bd0990df2ff16c6d3825f0b913b95978e298f21acf6d` | Immediate change-record and referenced-node list helpers |

`main-verification.json` checks 823 selected instruction spans, 2,959 unique
instruction bytes, 60 selected table bytes and 152 independently decoded direct
relative branches, including the explicitly selected reused dispatch/conversion
and owner records. The getter's separate verification checks 186 instruction
spans and its table/name evidence; the base-setter verification checks all 11
selected ranges across its three records. These counts have per-receipt scope
and are not an asserted disjoint aggregate.

Every new REA record completed with exit zero, untruncated output and unchanged
source hash. Rizin's warnings about automatic string scanning skipping two
large sections do not affect the explicitly selected reads; no string-search
completeness claim depends on them. The main five-record ledger was exported,
byte-verified and reset with no active target. `ledger-reset.bundle.json` is
byte-identical to `main.bundle.json`, whose SHA-256 is
`f06debd5a361e4b116a7a9786a1999bc78605f52898a395a87b1658397ee881e`.
The getter and base-setter lanes independently exported and reset their ledgers.

Reused project-authored native records include
`rea-attached-focusrect/complete.bundle.json` for standard setter dispatch,
primitive Boolean copy/conversion, receiver Environment and success returns;
`rea-asflags-followup-version-proof/environment-owner.bundle.json` for the
concrete attached-owner constructor; `rea-attached-movieclip/complete.bundle.json`
for event-presence behavior; and the native name-map records cited above.
All refer to the same pinned executable. They are original compatibility
research, not outside implementation code.

Installed REA 6.9.0 and its reverse-engineer-anything skill/native-artifact guide
were used for procedure only. New native reads use bounded target-free Rizin
commands; no shared configuration or binary was changed. Original Python PE
mapping/readback helpers verify selected instruction bytes, data slots and
direct branch targets against the pinned source. Discovery tails and incomplete
neighboring ranges are excluded from semantic claims. No external SDK or source
implementation was consulted, and no native UI, debugger, VM probe, server or
renderer was run for this study. No retail runtime or gameplay parity is claimed.
The local guides are
`~/.agents/skills/reverse-engineer-anything/SKILL.md` and
`references/native-and-artifacts.md`; their material was used for tool operation
only, and its licence was not independently inspected. The reused project-owned
PE mapper is private `evidence/rea-attached-focusrect/pe.py`; the new verifier
records its exact hash. Detailed helper provenance is retained in the private
base-setter and getter receipts. These tooling reads do not introduce outside
implementation code into the client.

## Appendix: initialization and later visibility-policy writers

**VERIFIED, static checkpoints:** native initialization selects clear
MovieImpl `DWORD +0x4310` bit 11 and clear target `DWORD +0xb8` bit 3 at
specific writes. This does **not** establish their values when the original
login constructor executes. The fresh allocation, subsequent attachment,
queued actions and explicit caller-supplied snapshot are distinct states.

### MovieImpl initialization and AS2 override

The complete MovieImpl constructor is
`0x140065080..0x140065830` (1,968 bytes). It retains `this` in RBX and clears
R12. At `0x140065417`, it writes zero to the eight bytes at `this+0x4310`.
Thus at instruction pointer `0x14006541e`, bit 11 is clear. Its later direct
operation at `0x1400655de` ORs `0x40180` into the DWORD, affecting only bits
7, 8 and 18. That operation preserves bit 11. These are the constructor's
only direct operations on this field; intervening or later callees have not
been exhaustively proved unable to modify it through aliases.

There is a concrete later AS2 writer. The global object's interface
`0x142512aa0`, slot `+0x18`, points to `0x1401e9950`. This complete 636-byte
setter compares the incoming interned name with the current root's builtin
entries. The builtin array begins at root `+0x138`; index 72, root `+0x378`,
is exactly `noInvisibleAdvance`. Its source pointer-table entry is
`0x142add520`, containing string address `0x14250bf80`.

On this name's branch:

1. The global's owner pointer at `global+0x48` must have state byte `+0x68`
   equal to one. This is the native enabled `gfxExtensions` state documented
   in [AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md).
2. The setter obtains the target from `Env+0xe0`, then its owner at `+0x20`,
   then MovieImpl at owner `+0x10`. The MovieImpl pointer must be non-null.
3. It calls `0x14017d730` with the supplied Value and Environment.
   A true result ORs `0x800` into MovieImpl `+0x4310` at `0x1401e9b3c`;
   a false result ANDs `0xfffff7ff` at `0x1401e9b48`. Other DWORD bits survive.
4. It continues to generic member-set helper `0x1401e3b20`. The native flag
   side effect is not itself evidence of the final stored member flags or
   the generic setter's result.

The adjacent `continueAnimation` branch changes bit 13, not bit 11. The
setter also recognizes `gfxExtensions` separately. Primitive Boolean
conversion is the already verified narrow profile above; this study does
not enable arbitrary Value coercion in the Rust VM.

**OBSERVED, bounded source inventory:** all 20 saved frontend GFX samples
plus the original manager are uncompressed `GFX` streams. An exact byte
inventory finds no literal `noInvisibleAdvance` or `continueAnimation` in
these 21 streams. It finds six `gfxExtensions` literals. The manager's
original script separately contains writes enabling `gfxExtensions`, as
recorded in [FRONTEND_FOCUS.md](FRONTEND_FOCUS.md). Absence of a literal is
not absence of a dynamically assembled name, host write, imported script,
or later override. No current frontend bit-11 value follows from this scan.

A second native writer, `0x1402da910..0x1402da94b`, accepts a Boolean in
R8B. It follows receiver `+0x28`, then `+0x80`, requires a nonzero byte
at that object's `+0x3a0`, then follows `+0x2d0` and `+0x10`. It sets or
clears DWORD `+0x4310` bit 11 on the resulting pointer. This is a separately
gated binding path, not evidence that the initial AS2 frontend calls it.
Its verified static row/thunk binding has the exact name
`noInvisibleAdvance`. Descriptor `0x142540070` points to type block `0x142540048` and table
`0x14253fe40`; selected row `0x14253fe90` names `noInvisibleAdvance` and
thunk `0x1402dd610`. The linked type block holds exact strings `Extensions`
and `scaleform.gfx`. **INFERRED:** that descriptor layout suggests the
qualified API label `scaleform.gfx.Extensions.noInvisibleAdvance`; its
consumer and actual runtime class ownership are not proved here.
The complete 92-byte thunk defaults the argument to
false when no argument is supplied; otherwise it calls conversion helper
`0x1402ed7d0` on the first argument. It gates dispatch on caller context
byte `+0xb0` being zero and sends the converted Boolean in R8B through the
runtime member-pointer pair `0x142cab040/+8`. Initializer `0x141d8e870`
installs writer `0x1402da910` and a zero signed adjustment in the low DWORD
of the pair's second slot. No claim about the unused upper DWORD is needed.
This records the initializer's instructions, not that it ran in a particular
execution. The full conversion semantics, guard meanings, runtime AS2/AS3
selection and actual frontend invocation of this binding remain unproved.
The linked namespace strings do not make it the AS2 global setter above.

### Fresh target state and attachment boundary

The base target constructor `0x1400a1130` writes the **QWORD value `0x10`**
at target `+0xb8`, at `0x1400a11a2`. Immediately after this write, status
bit 3 and the shared focus bits 7/8 are clear; bit 4 is set. It is incorrect
to replace the whole field with an invented all-zero default. There are no
later calls or overlapping writes through this base constructor's return
at `0x1400a11e9`. The complete next wrapper `0x1400b4dd0` also preserves
`0x10` through its return at `0x1400b4e32`: its only subsequent call is
the 28-byte leaf `0x140130b30(target+0xd8)`, whose writes lie entirely
between target offsets `+0xd8` and `+0x100`, inclusive. These two exact
return points retain bit 3 and focus bits 7/8 clear. The retained case-3 constructor `0x1400b6d40`
contains no direct `+0xb8` store, but its helper calls and subsequent
attachment remain relevant state boundaries.

The bounded attachment follow-up observes direct changes to other bits:
`0x1400b30a0` can OR bit 22, and the selected list helper `0x1400a1380`
changes bits 21/22/23. Those particular masks preserve bit 3 and focus bits
7/8. They are not a proof that all transitive virtual calls preserve those
bits. Source-bounds helper `0x1403cd2a0` can call target virtual `+0x90`;
attachment also reaches target-bearing virtual calls and the queued
prototype/Construct stages. Their complete state at the later login writes
has not been reconstructed. The parent propagation path already described
above is another concrete writer of descendant `+0xb8` bit 3.

### Bounded implementation contract

An original state helper may use the unchanged-status branch only when its
caller explicitly supplies the **current** MovieImpl policy bit and target
status bit for the exact attached owner. For canonical Boolean request `v`:

```text
desired_status = !v && movie_bit11
unchanged_status = (old_target_bit3 == desired_status)
```

For `v=false`, this requires `old_target_bit3 == movie_bit11`; for `v=true`,
it requires `old_target_bit3 == false`. The private receipt checks all eight
combinations. This is Boolean algebra over the verified native branch,
not a retail execution test or an inferred fresh-instance profile.

The base local-visibility and render-node work happens **before** this
comparison. Equal status only skips the subsequent status/list/descendant
work. If status changes, a helper cannot silently skip the previously
documented `0x1400b60d0`, list insertion and conditional child propagation.
Those effects need implementation or an explicit unsupported result. Any
update to bit 3 must preserve the other `+0xb8` bits, including focus bits
7/8; clearing the whole field would corrupt independently represented state.
Propagated status does not overwrite a descendant's local visibility bit.

A bridge must also check coherence between native target local-visibility
bit 14 at `+0x6a` and its exact scene node's **local** visibility state.
Effective visibility may differ because of ancestors. The controlled VM
root value `0x52000180` has bit 14 clear, while its saved DisplayState can
be visible. That combination cannot be presented as an observed coherent
native/display snapshot or repaired silently by choosing either value.
An explicit coherent initialization/snapshot and owner/node identity are
prerequisites; they do not follow from a saved placement being visible.
No diagnostic panel was hidden, no native input was submitted, and this
appendix enables no VM or renderer behavior.

### Prerequisite evidence and provenance

Private files are under
`evidence/attached-visibility-prerequisites/`. Constructor findings and
`movie-constructor/receipt.json` seal the complete 1,968-byte constructor:
cached prefix record `ev_8615b05f212ecf00f42122f6478fa0948eeeb7020582b36f60f0f9a29df80670`
and new tail record `ev_ac9df45d3e04be8d852183ec238c37855c0579327d7b28529ee7625e4fc2e74f`.
The constructor receipt SHA-256 is
`b3001520fdfdde9900dcdedcfa921fb6cf25b836bf130523a417c2b6f87cf627`.

`target-status/findings.json` distinguishes the two proved base/wrapper
return points from later source/attachment state. Its `receipt.json`
SHA-256 is
`51d3a9ca38c745fb98f0d0b252bd607d885137f4c478df932f603382016cc163`.
That lane verifies 1,029 selected instruction records, 4,036 unique bytes
and two concrete vtable pointers, including reused records. Its three new
REA records were exported and reset; 37 retained artifact hashes pass
readback. These counts overlap prior evidence and are not added to the
main lane's counts as if they were disjoint.

`overrides/final.bundle.json` contains complete writers in
`ev_3a49b925340768565cfd8b7175f5fd14f01f4b8b7be771e767d7195e8d6db289`
and selected table/name bytes in
`ev_4436e29b026b9ed438216d0545ee2cf2ba26d143ae4272efdbd5f63a9bf71424`.
`overrides/verification.json` checks 451 selected instruction records,
1,746 unique instruction bytes, 88 table/name bytes and 62 independently
decoded relative branches, including selected reused builtin-array producers.
The saved frontend literal inventory records each source path/hash and raw
offset. Byte-pattern writer candidates are explicitly discovery-only; they
do not prove an exhaustive native write set. Both new main REA records are
complete, exit zero, untruncated and source-hash unchanged. Their exported
ledger was byte-verified and reset without retaining an active target.
The main `overrides/receipt.json` SHA-256 is
`45c73623af8ccb0c1836ce67abc9c8db00b9ec1d8a759e042c984b622de6c70f`;
it seals 11 artifacts. The main bundle SHA-256 is
`1f3cc88d32e7b4e0bfda788ff12c1f28ff63fd4b0338aa01170e21ee91f3a915`.

`api-writer/findings.md` and `receipt.json` seal the separate named-row,
thunk, initializer and descriptor observations. The receipt SHA-256 is
`babd8cd69253d73fcb888349ab807cf6b5c792f75cf3ce0eb76cd8b3d2316261`.
Its three new canonical records are
`ev_507a110541f7016d5944b9ea1f999dcc358eab806c67dcabe168fc138ffc3148`,
`ev_6dfea8f1e5248477c5ca61681812ce4cc110558a08202c4db33deb25b43aaec5`
and `ev_fdd668ad752673382e049a086b83357de5c9fac079beab84388913a59e4c9db8`.
All ten selected ranges and the reused 59-byte writer pass pinned-byte
verification. That lane exported and reset its three-record ledger and
retains explicit descriptor-consumer, conversion and frontend-reachability
limits.

Reused builtin evidence is `rea-class-registry/helpers.bundle.json`, record
`ev_03a87c62e4c5bcea6484d7582ccf5a695f0d004b677cc6dbb568c6c2f57ade6e`,
and `rea-native-alternative/extends-name-binding-final.bundle.json`, record
`ev_5de603be82d92b5907b35369248200732fda049b1d4c60fe5880ced8d9b92b76`.
All new and reused native records refer to the pinned executable identified
above. The original private PE mapper and verifier, existing project native
and frontend documents, the saved owner-only GFX streams, and the installed
REA skill/native guide are the only additional sources. Proprietary bytes
remain private; no outside implementation, SDK, network source, ClientConfig
or user account was read. Static verification does not establish actual
startup state, constructor reachability or runtime rendering parity.
