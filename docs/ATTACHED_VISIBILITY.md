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
