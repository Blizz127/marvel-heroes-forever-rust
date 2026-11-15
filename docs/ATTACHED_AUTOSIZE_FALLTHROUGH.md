# Attached MovieClip `autoSize`: exact native fallthrough

**VERIFIED static owned-build evidence:** exact `autoSize`, standard member
ID **48**, falls through the concrete attached **case-3 MovieClip** native
getter and setter to their ordinary member routes. This permits reuse of the
already supported ordinary storage semantics with their existing guards. It
does not make every standard member ordinary, return a fabricated native
success, or supply TextField autosizing behavior.

The result is specific to build `1.52.0.1700`, the initialized member-name
profile, and this concrete live owner. No native client was launched. The
existing Rust source probe stopped before performing this assignment.

## Actual source and receiver

The owned login movie, movie 13/header 15, contains saved Construct record
39228, original mask `0x40000`, action span **39236+449**. Its first action is
a constant pool with 25 entries; entries 0 and 1 are `autoSize` and `none`.
Push at **39506** supplies those two constants. **SetVariable at 39513** is
the assignment `autoSize = "none"` on the current target resolution route.
The SetVariable opcode/helper is a separate proof; this document begins at
normal attached-member dispatch after a receiver and exact key are selected.

The sealed controlled association selects Scene86, attached clip 2/owner 20,
from the saved import of login symbol 170 to UISource movie 19/symbol 1039.
Its definition and target Environment remain movie 19/header **11**; they are
not relabeled as the placing movie's header 15. The fixture supplies the
final import compatibility witness and current state. It is not an observed
native final-binding or startup capture.

The existing probe, `avm1-original-construct-saved-increment1/receipt.json`,
visited three actions and stopped at the explicit saved SetVariable gate.
The input stack item plus `autoSize`/`none` remained, the four registers were
unchanged, and no receiver storage was created. That probe establishes the
real source frontier, not successful execution of the assignment.

## Name identity and dispatch cache

The complete native standard-member table has row **`0x142adcda0`**:

| Field | Owned-build value |
| --- | --- |
| Literal pointer | `0x14250d1f8`, exact ASCII `autoSize` |
| Member ID | 48 (`0x30`) |
| Selector | 0 |

Initializer `0x1401954c0..0x1401955b1` requests StringNode flag bit 29 for
this row and does not request bit 28. The interner ORs requested flags into
an existing node; this is not a claim that it clears previously held bits or
that all later cache writers have been found. The initialized map and node
classification are separate prerequisites.

With original-node bit 29 set, normal setter `0x1401aa130` and getter
`0x1401ddcd0` call `0x14018ff00`, which uses the initialized exact interned
name map to obtain ID48. They pass selector/mode zero to primary native
dispatch. With bit 29 clear, this nonempty ASCII key starts with `a`, so it
does not enter the normalized underscore-name gate and goes directly to the
ordinary route. This latter observation does not authorize bypassing the
VM's authenticated loaded-name profile. Unknown cache/map state remains an
admission error. No assumption that bit 31 is clear is needed below.

## Concrete case-3 getter and setter decline ID48

The attached MovieClip owner has primary table `0x142517670`; its AVM
interface at owner+8 has category **2**. Here *case 3* identifies the native
instance creation branch, not the interface category or script `typeof`.
The actual primary slots are:

| Slot | Concrete target |
| --- | --- |
| `+0x110` setter | `0x1401c3e50` |
| `+0x118` getter | `0x1401dd870` |

Base setter `0x1401a8f20` takes its mode-zero path at `0x1401a8f60`.
At `0x1401a8f65..0x1401a8f68`, ID48 is greater than `0x20`, so it returns
false through `0x1401a8f56..0x1401a8f5f`. It does not reach the base member
mask, Environment lookup, conversion, or property body.

The derived setter still checks its own table after this false result.
For `(48-4)`, selector byte **`0x1401c4438 = 8`** selects jump entry
**`0x1401c4408 -> 0x1401c43ba`**. This arm cleans up its temporary assigned
Value as needed, sets AL false at `0x1401c43c9`, and returns. There is no
autoSize setter body on this concrete route.

Base getter `0x1401dce90` similarly rejects ID48 at
`0x1401dcee6..0x1401dcee9` and returns false through `0x1401dced3`.
Derived getter selector byte **`0x1401ddcb8 = 10`** selects jump entry
**`0x1401ddc88 -> 0x1401dd9c3`**, the direct false-return arm. It does not
write a successful native result Value, including an invented Undefined.

These are exact table entries for ID48, not a generalized interpretation of
the tables' default cases.

## Normal caller fallthrough

After derived setter false, normal setter's extra-ID switch at
`0x1401aa20f..0x1401aa215` tests unsigned `(ID-89) <= 26`. ID48 fails that
test and reaches **`0x1401aa6ad`**. For admitted versions greater than 6,
the separate exact `__proto__` comparison fails for `autoSize`. The path at
`0x1401aa728..0x1401aa750` obtains or creates storage through primary
`+0xc8`, then calls storage's **normal** setter interface `+0x18` with the
original calling Environment, name, assigned Value and flags. It never
selects a named child as a write destination.

The native false result therefore means *not handled by that native member
dispatcher*. It is not necessarily the final property-setter result. Normal
storage may succeed, reject a read-only member, require an unsupported
callback, or fail. A language assignment must not expose this internal AL
as a new Boolean script result.

Normal getter also has an extra-ID switch. For ID48, selector byte
**`0x1401de400 = 8`** selects entry
**`0x1401de3f0 -> 0x1401dddc6`**. The separate `__proto__` checks then fail,
so the getter reaches ordinary handling at **`0x1401de14b`**:

1. Probe existing storage with its parent temporarily detached; restore that
   parent before returning or continuing. A present own Undefined wins.
2. Probe the real named-child state. The owner's cached version controls case
   matching; the selected child must pass target bit7 before conversion to
   public display Value tag7.
3. Probe the attached owner's display prototype, preserving supported
   accessor/resolver behavior or rejecting unsupported callbacks explicitly.
4. Only underscore names enter the final special fallback. `autoSize` returns
   missing if all earlier sources miss.

The getter does not allocate storage. Writes can shadow an existing named
child; reads must retain this complete own/child/prototype precedence. Neither
native false nor absence from a single object map proves an overall miss.

## Watch and pre-write requirements

ID48 is greater than 21. Before it calls the base setter, derived setter
`0x1401c3e50` can invoke the existing-storage watch at
`0x1401c3ea7..0x1401c3f73`. It can replace the local copied Value even though
the native member ultimately returns false. The initial implementation must
therefore retain its grounded **no-watch invariant**. Native fresh storage
initializes watch field `+0x58` to zero, but arbitrary adopted or untracked
storage cannot assume this. See [REA_ATTACHED_TAB_ENABLED.md](REA_ATTACHED_TAB_ENABLED.md)
for the constructor and supported prototype-visitor proof.

Ordinary storage's normal setter `0x1401ad600` calls pre-write hook
`0x1401ad440` before generic normal setter `0x1401a1020`. The hook remains
part of the route. For this exact name:

- The event-name branch requires first byte `o` at `0x1401ad4c2` and then
  `n`. `autoSize` fails the first comparison, so it cannot classify or add a
  mouse-event marker through this branch.
- When name bit31 is set, the extension branch still requires exact identities
  `rendererString`, `rendererFloat`, or `disableBatching`, compared at
  `0x1401ad540`, `0x1401ad58a`, and `0x1401ad5b5`. `autoSize` matches none;
  their conversions and target writes are not reached.

The hook still handles its bound target token and reference lifetime. This
is a proof that those particular event/renderer effects are excluded for
the exact name, not that the hook is absent, has no calls, or is safe for a
retired/unknown receiver. The existing live owner/handle/storage guards stay
in force. A new own entry may run the hook again through the raw storage
override; there is no single universal post-write hook position.

Generic normal assignment must retain its existing behavior:

- Resolve own data or a selected inherited accessor; inherited data is
  shadowed by a new own member. Do not silently replace an accessor callback
  or unresolved resolver path with a plain write.
- Respect selected member ReadOnly flag `0x04`; preserve existing data flags.
- Preserve the separate generic watch gate at `0x1401a129e..0x1401a12cc`.
- Supply normal new-entry flags and lazy storage/prototype construction, not
  a direct raw-map insertion substituted for the normal language assignment.

For an admitted ordinary data route with absent watches and no unsupported
selected callback, the assigned String `none` is stored as that Value. This
path does not interpret it as a text-layout mode or convert it to Boolean.
It does not prove a fixture's baseline property is absent or writable.

## TextField and implementation boundary

Attached **case 4** has a different primary table `0x142517970`, interface
`0x1425178c0` with category **4**, and category-13 lazy storage. Its text
document and inherited Environment differ from the MovieClip layout; see
[TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md). This study does not trace its
ID48 getter/setter and authorizes no TextField fallback or autosize behavior.
Public tag7 alone cannot select the case-3 contract.

**INFERRED bounded integration:** keep `autoSize` classified as authenticated
Standard(ID48, selector0), then allow only that exact key on the proved live
case-3 owner to continue through the existing ordinary getter/normal setter.
Retain target/root identity, version/profile, complete child state, lazy
storage, flags, callback/watch and lifetime gates. Other standard names,
case4, generic unknown display owners, raw-set shortcuts, and assignment
opcode semantics require their own proof. Unrelated prototype accessors do
not invalidate an otherwise grounded exact-key route; selection matters.

## Evidence, checks and provenance

Private evidence root: `~/.local/share/marvel-heroes-forever-rust/evidence/`.
The original checker `rea-attached-autosize-fallthrough/verify.py` rereads the
owner's pinned Win64 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and validates **1,506 complete instruction spans / 5,491 instruction bytes,
2,629 table bytes, and 64 literal bytes** against PE section mappings. It
also validates the exact source pool, Push/SetVariable bytes, action hash,
and sealed frontier receipt digest. `verification.json` records every
selected range, bundle digest and original Evidence ID.

| Retained canonical bundle | Evidence ID / use |
| --- | --- |
| `rea-attached-tab-enabled/complete.bundle.json` | `ev_0cd9f9ecac7ce857515a7f2b9922c7b7dd203492ffb02784096043ca11861c42`: complete derived get/set branches, watches, dispatch tables |
| `rea-attached-focusrect/complete.bundle.json` | `ev_e236173cc89a9aafe84cd7de1be73eee9be64730c91e5009b2ef263f225a476b`: base get/set ranges; `ev_cd9426965b8211c6c378544591fa639b211d07f154b768082843d3f2dc53dfbf`: concrete virtual slots |
| `rea-attached-movieclip/complete.bundle.json` | `ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973`: normal setter, storage, category; `ev_f9c7c7b69028559f5f1e9be45c969fe83eb73034694c2636a0c8d749539ab15e`: pre-hook and storage setters |
| `rea-event-dispatch/complete.bundle.json` | `ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54`: complete normal getter and table |
| `rea-native-alternative/asflags-scope-final.bundle.json` | `ev_820980a2b7e2c4340592ed0dcd933652c281056d7b90c1a3849e64a5d1a0564a`: generic normal setter |
| `rea-attached-name-map/final.bundle.json` | `ev_06752c5af92f95580a84cd9200320202857cc0052006fab5a6a5833224f08650`, `ev_a7b803905b375352deb3ec300246f684190dbb8db9efe96f7645677f3c9e2330`, `ev_89cb16c8abed146a0071376ee2eb978c82bb47d08a2504144dea383b87a6726d`: exact table, initializer, lookup and literal |

All selected records are complete, exit-zero, untruncated, and report the
unchanged pinned digest. This study needed **zero new native tool calls**:
complete retained REA observations already cover the necessary branches.
No REA session was opened or shared; previous producers' bundles remain
unchanged. Native execution, callback effects outside the profile and full
frontend parity remain unobserved.

Sources read: the proprietary owner-installed executable and login movie for
private compatibility; the original project documents linked above plus
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md) and
[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md), and their private REA
receipts; the local `reverse-engineer-anything` skill and native guide
(REA MIT toolkit); Rizin via those REA records (LGPL-3.0-only); Python's
standard library (PSF licence). No outside VM or SDK implementation was
read or reused. Native bytes, movie data and extracts remain private.
