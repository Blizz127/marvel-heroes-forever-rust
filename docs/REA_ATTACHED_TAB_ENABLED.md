# Attached MovieClip `tabEnabled`: Boolean assignment and raw state

**VERIFIED static native path:** the concrete case-3 attached MovieClip's
`tabEnabled` property is native member **29, selector 0**. With no active
storage watch, assigning Boolean false updates its actual display target's
DWORD at **`+0xb8`** to `(old & ~0x20) | 0x40`. Native readback returns
Boolean false. This changes the same raw word used by `_focusrect` and
visibility; it must preserve every other bit.

The native setter has no post-write notification, scheduling, invalidation,
allocation or script callback on this exact path. However, its derived entry
can invoke a watch **before** conversion and mutation. That prerequisite is
material: the earlier ID-17 `_focusrect` path bypassed it, while ID 29 does not.

This is a bounded static compatibility contract. The original executable was
not run. It does not establish tab traversal, keyboard focus, rendering,
loaded-state defaults, or complete frontend behavior. No VM code changes here.

## Actual original-script boundary

**OBSERVED in the separate controlled Rust probe:** frozen VM `5734d6b`
completes 452 initializers, then reaches original SetMember **60254** after
159 frame instructions. Its rejection is the unsupported native member.
The source is the owned manager movie, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.

The source/context audit identifies `_global.MarvelObject`'s constructor:
Function2 definition 60217, body `60228+160`, flags `0x19`, no parameters.
It preloads `r1=this` and `r2=super`. Its preceding super call at 60243 returns;
Push 60245 supplies register 1, pool entry 2 (`tabEnabled`), and Boolean false
to SetMember 60254. There is no preceding register overwrite or branch inside
this body. The class is installed at 60392 and extends `gfx.core.UIComponent`
at 60423. Its enclosing stream is `55878+15510`, tag ordinal 439.

The actual receiver is source/context-derived attached root `DisplayClip(0)`,
owner ObjectId 18, target 1000, through preserved apply/super receiver flow.
It is not an instrumented raw operand dump. The root, child snapshot and
minimal global setup are controlled inputs. In particular, target event flags
`+0x6a=0x52000180` and separate status `+0xb8=0x5a5a0180` were supplied by
the harness. The earlier original focus assignment leaves status `0x5a5a0100`.
None of these words is a retail capture or native default.

The sealed private probe receipt is
`evidence/avm1-attached-manager-frame-action-increment4/receipt.json`, SHA-256
`edbe93eefca2f22f283102bcc1596c31fc0dc4dc9bd44034d6ef12cd867cd67e`.
Its source audit is `static-notes/source-context-audit.json`, SHA-256
`6cf078d3f7c2b8497723fc54ca6d2b74e4b8865d522b6f54c14729abd8cf58ec`.
The probe stops before this native write; this study does not retroactively
claim that its setter succeeded.

## Receiver, classification and watch prerequisite

The original finite standard-name map has row `0x142adcc70`, pointing to
`tabEnabled` at `0x14250d2f8`, ID 29 and selector zero. Initialization sets
StringNode classification bit 29, without bit 28. The complete map and name
lookup evidence are in [ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md).
This is an authenticated native identity, not an underscore/name heuristic.

Normal attached setter `0x1401aa130` resolves the member ID at `0x1401aa1d1`,
then invokes owner primary virtual `+0x110` at `0x1401aa1e8`, passing the
owner, ID, input Value and mode zero. The concrete virtual is
`0x1401c3e50`. The display target is owner `+0x20`; the attached public Value,
owner and optional script-storage object remain distinct identities.

The normal setter saves the supplied calling Environment, but does not pass
it as a native standard-setter argument. The derived setter and base setter
obtain the receiver-owned Environment through primary virtual `+0xf0`,
`0x140203720`, which returns **owner +0x38**. Exact Boolean conversion reads
the input payload directly, so that conversion does not use an Environment
version or substitute the active caller's target.

Derived entry `0x1401c3ea7..0x1401c3f73` checks:

1. The member ID is greater than 21.
2. The receiver-owned Environment is nonnull.
3. Existing bound storage at owner `+0x1d8` is nonnull.
4. Its watch-related field at storage `+0x58` is nonnull.

If all hold, it calls `0x140199330` at `0x1401c3f3c`. A successful watch
result replaces the local assigned Value at `0x1401c3f5f`, before base native
dispatch. The incoming Bool(false) alone therefore cannot establish the final
write when watch state is unknown. This study admits absent watch state; it
does not implement or guess watch callback behavior. Missing storage bypasses
the watch check and does not itself force storage allocation on this route.

The admitted fresh-storage construction supplies a grounded no-watch invariant.
Factory `0x1401de450` calls `0x1401da850`, which calls generic base constructor
`0x1401ccd80` at `0x1401da886`. The base explicitly writes **QWORD
`[storage+0x58]=0` at `0x1401ccdc7`**. This does not depend on zeroed allocator
memory. The remaining constructor installs tables, a target token and event
fields, then calls prototype hook `0x140190a50`; publication to owner `+0x1d8`
occurs afterward at `0x1401de4e0`.

For the already admitted prototype profile, that hook links the parent through
`0x14018edf0` and the raw `__proto__` marker path, then performs the bounded
event-marker visitor. Generic visitor `0x1401924c0` receives options `0x0d`;
marker properties bypass virtual reads, and callback `0x140182980..0x1401829a2`
ignores property values and only updates event marker `+0x72`. The raw setter
does not enter normal assignment's watch callback. None of these admitted
steps installs a watch. The existing-storage factory branch does not reset
`+0x58`. Therefore a VM may preserve watch absence through this fresh
construction and supported preserving mutations while watch installers remain
unreachable. Arbitrary adopted storage, untracked mutation or unknown prototype
virtual visitors cannot inherit that invariant. See the existing bounded
construction and prototype contracts in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md).

## Native setter and side effects

Base setter `0x1401a8f20` admits mode-zero ID 29 through its range check and
primary `+0x108` mask. That concrete leaf, `0x140067900`, returns `0xffffffff`.
The ID-29 table word at `0x1401a9724` selects **`0x1401a9342`**.

The copied Boolean is native Value tag 2, payload byte `+8`. Copy helper
`0x14018d770` selects its primitive Boolean arm through table entry
`0x14018d9a0 -> 0x14018d7ab`. The called Boolean conversion
`0x14017d730` takes `0x14017d83a..0x14017d846`, directly loading that payload.
No object conversion or script callback occurs in this exact conversion.

For current target status DWORD `S`, the native writes are:

| Input / observed native branch | Resulting status |
| --- | --- |
| Boolean false | `(S & 0xffffffdf) | 0x40` |
| Boolean true | `S | 0x60` |
| Raw native Value tag 0 or `0x0a` | `S & 0xffffff9f` |

Tag 0 is Undefined. The meaning of raw tag `0x0a` is deliberately not assigned
a public script name here; in particular, it is **not** Null, whose native tag
is 1. Non-Boolean assignments remain outside the initial implementation scope.
Their reset branch is recorded to distinguish unset state from false, not to
authorize broad conversion support.

The false writes are at `0x1401a9378` and `0x1401a937f`. They access target
`[owner+0x20] +0xb8`, preserve every other bit in that DWORD, and jump directly
to successful base return `0x1401a8fd5`. There is no post-write helper call.
Derived success `0x1401c3fb9..0x1401c3fcf` skips reference cleanup for primitive
tag 2 and returns true. Normal setter success `0x1401aa1f3` goes straight to
its epilogue.

Thus the admitted path does not create an ordinary `tabEnabled` member, obtain
lazy storage, inspect ordinary member flags, traverse inherited accessors or
children, call the normal storage pre-write hook, alter a prototype, or notify
a rendering/focus subsystem. Existing ordinary storage does not replace native
classification. The internal true return means the setter handled the member;
it is not a script result Value manufactured by SetMember.

The selected case ends at `0x1401a939b`. Adjacent `tabIndex` code captured after
that address is excluded. The derived fallback ID-29 selector points to a
false-return branch; successful handling is established by the base case,
not by confusing neighboring derived properties with `tabEnabled`.

## Getter and shared status ownership

Normal getter `0x1401ddcd0` invokes primary `+0x118` at `0x1401ddd86`.
Concrete `0x1401dd870` first calls base `0x1401dce90`; the ID-29 table word
at `0x1401dd854` selects **`0x1401dd564`**. It reads the same target DWORD:

| `S & 0x60` | Native result |
| --- | --- |
| `0x00` | Undefined, Value tag 0 |
| `0x20` | Boolean false, Value tag 2 |
| `0x40` | Boolean false, Value tag 2 |
| `0x60` | Boolean true, Value tag 2 |

The zero-mask branch writes tag 0 at `0x1401dd596`. Nonzero states compare
against `0x60` and write Boolean payload at `0x1401dd585`. All four states
return native success through `0x1401dcf49`, derived `0x1401dd8dd`, and normal
`0x1401dde97`. Undefined here is a successfully handled native property,
not a missing own member. Ordinary storage, children and prototypes are not
searched after this success.

**INFERRED integration requirement:** keep one current receiver-bound raw status
word, or a representation that preserves all known and unknown bits across
updates. Independent stale focus/tab/visibility copies can overwrite each
other. The tab mask `0x60` and focus mask `0x180` are distinct parts of the
same `+0xb8` DWORD. Event flags at target `+0x6a` are a different field.

The exact false write establishes a subsequent false read independent of the
previous tab mask. It does not establish the previous read value or supply the
rest of the status word. A masked-state model can mark just these bits known;
a full-word bridge must receive a valid current word and preserve its other
bits. A synthetic zero word is not an authentic initial snapshot.

Retained base target constructor `0x1400a1130` writes QWORD `0x10` at target
`+0xb8`, at `0x1400a11a2`. At that construction point the tab mask is zero.
This is not proof that the value survives subsequent constructors, attachment,
timeline setup, scripts or callbacks. No universal loaded-state default is
claimed. The controlled root's current `0x5a5a0100`, if admitted unchanged,
would become `0x5a5a0140`; that is an arithmetic prediction from controlled
input, not an observed native run.

## Evidence, validation and provenance

The owner's pinned Win64 executable, build `1.52.0.1700`, is private at
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`
under `~/.local/share/marvel-heroes-forever-rust/`. SHA-256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses use preferred image base `0x140000000`.

New bounded REA observations are retained in
`evidence/rea-attached-tab-enabled/complete.bundle.json`:

| Evidence ID | Relevant coverage |
| --- | --- |
| `ev_0cd9f9ecac7ce857515a7f2b9922c7b7dd203492ffb02784096043ca11861c42` | Derived setter watch processing and return; derived getter/base call and return; fallback tables |
| `ev_91cef003f03bffdc2e2bbc251a6492464ced30bd2b486dad549fefe80ff6c62a` | Exact ID-29 base setter/getter jump entries |
| `ev_5328387ded80f2ce429d00317cc4781b91db491dae988129cb155ecc05df8a9f` | Selected base setter case and getter mask/result branches |
| `ev_d549b3bff18fe57acd0c04a18f6b25b6d52a979794e711650d1487cc47db26e2` | Complete getter Undefined-write and success jump |

Retained canonical evidence joins the new cases to the concrete owner:

| Private source | Reused fact |
| --- | --- |
| `rea-attached-name-map/final.bundle.json`, `ev_06752c5...`, `ev_89cb16c...`, `ev_a7b803...` | Complete initialized map, exact literal, ID lookup |
| `rea-attached-movieclip/complete.bundle.json`, `ev_a256724...` | Normal setter, bound storage and concrete ownership |
| `rea-attached-focusrect/complete.bundle.json`, `ev_cd942...`, `ev_e236...` | Base admission, concrete virtual pointers, receiver Environment, Boolean copy/conversion, return |
| `rea-event-dispatch/complete.bundle.json`, `ev_90b1d7...` | Normal getter dispatch and success return |
| `rea-asflags-followup-version-proof/target-constructors.bundle.json`, `ev_6c4d08...` | Constructor-point raw status write; no loaded-default inference |
| `rea-attached-movieclip/complete.bundle.json`, `ev_f9c7c7...`, `ev_6f043...` | Fresh storage watch initialization, concrete raw setter and native pre-write hook |
| `rea-script-allocation/instance-read-final-3.bundle.json`, `ev_fe4c8...`, `ev_a26f9...` | Prototype-link wrapper and event-marker-only callback |
| `rea-native-alternative/extends-baseline-final.bundle.json`, `ev_b1581...`; `asflags-scope-final.bundle.json`, `ev_98b99...` | Raw prototype marker installation and bounded generic visitor |

The private original `verify-evidence.py` checks **1,560 complete instruction
spans, 2,500 table bytes and 11 literal bytes** against PE section mappings and
the pinned digest. It also checks the original Function2 header, literal false
assignment, body hash, sealed probe digest and derived mask-preservation cases.
Its `verification.json` records exact ranges, hashes and exclusions. Research
validation does not execute native code. REA exported and reset its four
records; `reset-receipt.json` confirms no retained target or unknown history.
The complete new bundle has SHA-256
`2f43e4acb03919c8836ef6fcf9f2b82bf0c34c1d1bc4b5257b7b2de8cb726db5`;
the verification receipt has SHA-256
`0351056ee6e12ecbb54316925542dc8871744c686be31f2c6383aadedc6f61d6`.

Sources read: the proprietary owner-installed executable and owned movie for
private interoperability; original project documents and private receipts
cited here and [ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md),
[ATTACHED_FOCUSRECT.md](ATTACHED_FOCUSRECT.md) and
[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md); the local
`reverse-engineer-anything` skill and native guidance (REA MIT toolkit);
Rizin via REA (LGPL-3.0-only); Python standard-library tooling (PSF licence).
No outside VM or SDK implementation was read or copied. Assets, bytecode,
instruction dumps and extracted tables remain private, outside git.
