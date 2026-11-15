# Attached MovieClip `_focusrect`: exact Boolean-false assignment

**VERIFIED static path:** on the pinned build's concrete case-3 attached
MovieClip, normal assignment of Boolean `false` to `_focusrect` clears bit
`0x80` and sets bit `0x100` in the receiver target's DWORD at **`+0xb8`**.
The native getter subsequently returns Boolean false. The selected state
update has no root/child distinction, parent traversal, global focus-default
lookup, ordinary member insertion, or script callback. This proves a bounded native contract,
not executed retail behavior or complete frontend initialization.

**The field is not `AttachedEventState.target_flags`.** That existing field
models target DWORD `+0x6a`. The controlled diagnostic value `0x52000180`
therefore establishes nothing about `_focusrect` before assignment. A bridge
must keep the two native fields separate and must not invent a default.

Only this receiver class, authenticated native member ID, and exact Boolean
false value are admitted by this study. Other value conversions, display
classes, removed targets, and rendering/focus-navigation consumers remain
outside the contract. No VM or host implementation is changed here.

## Actual original-script frontier

The preserved original-byte diagnostic is
`evidence/avm1-attached-manager-init-increment4/receipt.json`, relative to
`~/.local/share/marvel-heroes-forever-rust/`. Its SHA256 is
`67926731aa7377f276e51001365dffa6fb6b1691ce098913847cb78c6c57fd61`.
The owned manager movie has SHA256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`
and header version 15.

**OBSERVED in that controlled Rust diagnostic:** initializer 0 completes;
initializer 1 stops at original `SetMember` source offset **13632**, with one
Selection listener already added. The FocusHandler function definition is at
13523, body `13534+135`, Function2 flags `0x169`, with preloaded registers
`r1=this`, `r2=_root`, and `r3=_global`. Original Push `13623+9` supplies
register 2, constant-pool entry 12 (`_focusrect`), and Boolean false. The
initializer action span is `12444+3955`; its pool begins at 12444.

The receiver in that diagnostic is explicitly adopted attached root 0,
owner ObjectId 18. Its saved first-frame three-child fixture and minimal
global environment are controlled inputs, not a captured live retail state.
No own-storage object exists after the rejection. The diagnostic remains
unchanged: this native study does not retroactively turn that rejection into
a successful write or claim another initializer completed.

## Receiver, name, and calling Environment

The original initialized standard-member table identifies `_focusrect` as
**ID 17, selector 1**. Its row at `0x142adcbb0` points to the exact name and
contains those two fields. The complete 117-entry map and classification
rules remain in [ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md).
The private authenticated inventory has SHA256
`902b4f15629d3ae32aa6867165d275f2ef3ea289ced3696b9bf132155fd00b91`.

The concrete object identities are those already established in
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md): source target table
`0x1425069d0`, attached owner at target `+0x148`, owner primary table
`0x142517670`, and object-interface table `0x1425175c0` at owner `+8`.
Owner `+0x20` refers to this exact target. Owner `+0x38` is its embedded
Environment; that Environment's target field `+0xe0` also refers to the same
target.

The normal setter `0x1401aa130` saves the supplied calling Environment at
`0x1401aa15e`. Standard dispatch at `0x1401aa1da..0x1401aa1e8` instead passes
the full owner, member ID, input Value, and mode zero to primary virtual
`+0x110`, concretely `0x1401c3e50`. It does not pass the calling Environment
as a standard-setter argument.

Concrete primary virtual `+0xf0`, `0x140203720`, returns **owner +0x38**.
Both the derived setter and base setter obtain that receiver-owned
Environment. The conversion call in case 17 obtains it again. No caller-root
substitution occurs. Concrete virtual `+0x108`, `0x140067900`, returns
`0xffffffff`, so the base property mask admits bit 17.

The copied native input is Value tag 2 with Boolean payload at `+8`.
`0x14018d770` selects its primitive Boolean copy arm through table entry
`0x14018d9a0 -> 0x14018d7ab`. The Boolean-conversion helper
`0x14017d730` reaches `0x14017d83a..0x14017d846` and reads that payload
directly. This exact branch reads neither Environment version nor a global
default. Other conversion branches are not covered by the admitted value.

## Write, side effects, and return

Derived setter `0x1401c3e50` routes ID 17 through its `ID <= 21` branch to
base `0x1401a8f20`, bypassing the extension-property watch-handling path.
The base ID table entry at `0x1401a96f4` selects case `0x1401a93c6`.
Boolean false follows the conversion result to `0x1401a9408`.

For the receiver's current unsigned DWORD `F` at target `+0xb8`, the two
native writes have the combined result:

```text
F' = (F & 0xffffff7f) | 0x00000100
```

The clear occurs at `0x1401a9408`; the set occurs at `0x1401a9412`.
All other bits in that DWORD are preserved. The helper called with the same
target at `0x1401a9420`, **`0x14058fea0`**, consists only of a three-byte
return instruction. It performs no state access, notification, callback,
allocation, invalidation, or parent/global traversal in this executable.
This is a verified body, not an unresolved call treated as a no-op.

The case jumps to successful base return at `0x1401a8fd5`. The derived
setter returns true at `0x1401c3fc8`; Boolean type 2 skips reference cleanup.
Normal setter success at `0x1401aa1f3` goes directly to its epilogue. Thus
the ordinary lazy storage allocation, inherited accessor lookup, member
flags, and generic watch machinery are not reached by this exact route.
No ordinary `_focusrect` data member is inserted. Existing ordinary storage
does not redirect this admitted native standard assignment.

There is no target-parent test, root identity test, or root lookup in the
selected state-write path. A root and a child with the same concrete
attached class each mutate **their own target**. This does not establish
the result for another display class with different virtual methods, or
claim how later display code interprets a parent's and child's flags.

## Getter, previous state, and implementation boundary

Normal getter dispatch at `0x1401ddd86` calls concrete primary `+0x118`,
`0x1401dd870`, which first calls base `0x1401dce90`. Its ID-17 table entry
at `0x1401dd824` selects `0x1401dd50d` and reads target `+0xb8`.
The observed result mapping is:

| Current `F & 0x180` | Native getter result |
| --- | --- |
| `0x000` | Null, Value tag 1 |
| `0x080` | Boolean false, Value tag 2 |
| `0x100` | Boolean false, Value tag 2 |
| `0x180` | Boolean true, Value tag 2 |

For a nonzero mask, comparison at `0x1401dd51f` and Boolean output at
`0x1401dd576..0x1401dd589` establish the result. A zero mask writes Null
at `0x1401dd52f`. Both return native success, and the derived/normal getter
return without probing ordinary members, children, or prototypes.

After the exact false assignment the mask is always `0x100`, independent
of its prior two bits. Consequently no guessed baseline/default focus
value is required to establish the post-write readback. The native setter
still preserves all other `+0xb8` bits. This study neither supplies a full
initial `+0xb8` word nor establishes a pre-write getter result for the
existing fixture. Do not initialize it from the event flags at `+0x6a`.

**INFERRED Rust design:** maintain a separate receiver-bound state for the
proven `+0xb8` focus bits, or use a separately supplied authentic raw word.
An initially unknown focus state can become known false after this admitted
write; its earlier read must remain unsupported unless actual state has
been supplied. Preserve unknown/unmodeled bits instead of manufacturing a
full zeroed word. Central admission should verify the supported attached
class/name/value before mutation and keep every other native property or
conversion gated. This document does not implement that design.

The return-only helper proves no immediate drawing work on the selected
setter path. It does not prove that later render/focus traversal ignores
the modified bits, or establish focus-rectangle rendering parity.

## Evidence, validation, and provenance

Primary source: the owner's proprietary final PC build `1.52.0.1700`, used
for compatibility study only, at
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`.
SHA256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`;
size is 48,544,080 bytes. Addresses above use preferred image base
`0x140000000`. The owned manager movie is also proprietary; only its
original structural offsets and the selected operation are summarized here.
No recovered bytecode, asset, or native instruction dump enters git.

All new native records are under private
`evidence/rea-attached-focusrect/complete.bundle.json`:

| Canonical REA Evidence ID | Coverage |
| --- | --- |
| `ev_cd9426965b8211c6c378544591fa639b211d07f154b768082843d3f2dc53dfbf` | Exact setter routing/returns, case 17, receiver Environment and mask leaves, Boolean copy/conversion, selected getter path, concrete virtual and case table pointers |
| `ev_f52bdcbd4ab387b503142855bd3b3d8c44db5cceb5ce4b772c20cb4374e2a3d4` | Complete base-getter dispatch and normal-getter success/epilogue ranges; exact standard row/name and interface slots |

The three earlier discovery records in the bundle are retained for audit.
Their unrelated trailing bytes and incomplete terminal instructions are
not semantic evidence. The second canonical record also replaces two
short-ended getter ranges in the first. `verification.json` identifies
all selected complete proof ranges and excludes those partial tails.

Original Python helpers `pe.py` and `verify.py` independently map PE sections
and compare every retained instruction/data byte with the pinned file.
Verification passed for **1,521 instruction spans and 279 data bytes**
across five complete, exit-zero, untruncated, unchanged-artifact REA records.
The selected complete proof ranges contain **514 instruction spans**;
**92 direct branch/call targets** were independently decoded from original
relative-displacement bytes. No claim relies on Rizin's ESIL output.
Concrete virtual pointers, ID table entries, Boolean copy-table entry, and
the `_focusrect` row/name also have explicit independent assertions.

Reused local project studies are `ATTACHED_MEMBER_NAMES.md`,
`ATTACHED_MOVIECLIP.md`, and original `src/avm1/attached_events.rs` (field
identity only). The owner constructor comes from private
`rea-asflags-followup-version-proof/environment-owner.bundle.json`,
`ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d`;
its retained selected spans were independently byte-checked again.
The earlier receiver-Environment leaf record is
`source-version-env/rea-env-getter-tail.json`,
`ev_02263dbd05f95ac45707e5aef9e156d13a5f78298cb445de2873d48505022ef2`.
The private `existing-records/` notes retain their paths/hashes. These are
original project findings from the same proprietary artifact, not borrowed
implementation code. No external SDK, VM implementation, or source code was
used.

Analysis tooling: installed `rea-agents` 6.9.0, following
`~/.agents/skills/reverse-engineer-anything/SKILL.md` and its native-artifact
and evidence-workflow references; tool/skill material was used for procedure
only and is not incorporated. Supported target-free bounded Rizin reads were
used because this REA session's Ghidra backend reported unavailable JDK
detection. No shared configuration was changed. The initially empty ledger
was exported, then reset: five records cleared, no open target retained.
`ledger-reset.bundle.json` equals the exported bundle byte for byte.

The prior probe, manager bytes, native inventory, and full executable hashes
were rechecked unchanged. Documentation validation is `git diff --check`.
No native executable, runtime debugger, renderer, server, or new startup
probe was run for this study. Full frontend, readback-in-retail, and gameplay
parity remain untested.
