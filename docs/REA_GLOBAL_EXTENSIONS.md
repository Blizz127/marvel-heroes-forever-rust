# Native global gfxExtensions state and ordered assignment

**VERIFIED static native contract:** assigning an exact Boolean to the native
GLOBAL object's `gfxExtensions` member changes its owning context, updates or
deletes `gfxVersion`, then attempts to store a special marker. Those operations
are ordered. A read-only `gfxExtensions` descriptor rejects the final marker
write **after** the context/version effects; ActionSetMember ignores that false
setter return. A plain Boolean property assignment does not reproduce this.

This is a bounded compatibility contract for explicit fresh or supplied native
global state. It is not a capture of the current manager GLOBAL, a full global
implementation, or a cursor effect. [REA_MOUSE_HIDE.md](REA_MOUSE_HIDE.md) already
establishes that exact own `Mouse.hide` lookup reaches its callback for either
extension-state equality result; this setter study does not close the separate
[Mouse host boundary](REA_MOUSE_HOST.md).

## Original caller and native SetMember

The owned manager SHA-256 is
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Its original frame contains Push 6357 (`_global`), GetVariable 6362, Push 6363
(`gfxExtensions`, Boolean true), and **SetMember 6370, opcode 0x4f**. The selected
14 bytes have SHA-256
`56654574dc17c587fb6c6c7f51ae13e2910bed527feb535353c1fb10ce078849`.
The later Mouse lookup/call is a separate instruction sequence.

Native Push subtype 5 normalizes its source byte and constructs Value tag 2.
The general named SetMember route preserves the actual RHS Value pointer.
For a direct tag-6 GLOBAL Value, `0x140199640` returns the same object and
`0x1401997f0` returns its script interface at object+0x20. This does not license
arbitrary primitive boxing, deferred receivers or accessor conversions.

The complete selected SetMember block is
**0x14020b571..0x14020b6c1**. At **0x14020b68d**, it explicitly sets a local
property-flags byte to **zero**; **0x14020b694** passes its address as the fifth
argument to normal interface +0x18 at **0x14020b6a3**. RCX is the converted
receiver interface, RDX is the active Environment, R8 the exact converted name,
and R9 the untouched RHS pointer. Existing descriptor flags are not the caller
flags; the selected setter later preserves or tests them.

After invocation, the opcode does not inspect AL. It releases the temporary
name, calls **0x140197a20** to remove the three operand Values, and returns to
the interpreter loop. A false ordinary setter return is not an AVM1 exception
or a pushed result. Earlier partial captures ending at 0x14020b6b4 or at the
first unwind fragment of 0x140197a20 are superseded by complete selected ranges.

## Exact native global identity and construction checkpoint

Use these distinct roles:

| Symbol | Native identity |
| --- | --- |
| G | Native global allocation, 0x70 bytes on the fresh bootstrap path |
| I | G+0x20, its script-interface pointer |
| C | Owning AS2 context |
| E | Active Environment supplied to the setter |

The native global primary table is **0x142512a50**, script interface
**0x142512aa0**, category **6**. Relevant interface slots are normal setter
+0x18=`0x1401e9950`, normal getter +0x20=`0x1401e8670`, delete
+0x30=`0x1401b8040`, raw setter +0x50=`0x1401e3b20`, and raw getter
+0x58=`0x1401db080`. An ordinary object with a member of the same spelling is
not this native type.

Fresh bootstrap **0x14020ea40** checks C+0x60 and allocates G only when it is
null. The successful new path calls complete base constructor
**0x1401ccd80..0x1401ccdfe** at **0x14020eab1**. That constructor clears:

| G field | Exact fresh checkpoint state |
| --- | --- |
| +0x30 | Null internal parent |
| +0x38 | Null own-property hash |
| +0x40, +0x48, +0x50 | Null/zero resolver FunctionRef |
| +0x58 | Null watch state |
| +0x60, +0x61 | Zero property-state bytes |

The bootstrap then installs the global tables, writes **G+0x68=C** at
**0x14020eacb**, and publishes **C+0x60=G** at **0x14020eaf1**. Thus at the
next instruction, 0x14020eaf5, the parentless/no-watch/no-resolver checkpoint
and reciprocal association are grounded by the native producer. In interface
coordinates, the context pointer is **I+0x48**, not G+0x48.

Root context construction separately clears C+0x68 before bootstrap at
**0x14021014b** or **0x1402101dc**, then publishes C into AS2 root+0x50.
For a concrete case-3 source owner, context getter **0x14017b880** follows
owner+0x20 to target, target+0x20 to root, and root+0x50 to C. Owner construction
calls that primary +0xb8 slot at **0x140203640**, then stores the returned C
into **E+0xe8** at **0x140203646**. See
[FRESH_AVM1_ENVIRONMENT.md](FRESH_AVM1_ENVIRONMENT.md).

The bounded same-context association is therefore **E+0xe8=C, C+0x60=G,
G+0x68=C**. The setter takes the actual G/I receiver; it does not itself enforce
this equality. Its name/string context comes from E, while the extension byte
is reached through I+0x48. A cross-context receiver is outside this profile.

**Limit:** this proves a fresh allocation/publication checkpoint. The rest of
bootstrap, host setup and subsequent scripts are later operations. The retained
46-name explicit bootstrap inventory contains none of `gfxExtensions`,
`gfxVersion`, `__proto__` or `__resolve`, but that finite inventory is not an
exhaustive preservation proof for all later calls. Reused or adopted current
state must be supplied or tracked; it cannot inherit these fresh values merely
because the diagnostic GLOBAL is a parentless Plain object.

## Exact Boolean setter and getter

Complete normal setter **0x1401e9950..0x1401e9bcc** compares the incoming
interned name with the active Environment context's builtin at offset **0x370**.
The owned pointer-table binding proves that spelling is `gfxExtensions`.
This first profile admits only its exact ASCII spelling and loaded source
versions 7..19.

For that name, **0x14017d730** receives the actual RHS and E. Its exact tag-2
branch at **0x14017d83a..0x14017d846** returns the stored Boolean byte without
script conversion. The setter computes `!value + 1` and stores the result to
**[I+0x48]+0x68 at 0x1401e99a4**:

| Exact Boolean RHS | New context byte |
| --- | --- |
| true | 1 |
| false | 2 |

Neither assignment writes zero. A fresh byte zero is a separate constructor
checkpoint, and a supplied raw state must not be silently reset.

The global-specific raw getter **0x1401db080..0x1401db0ef** recognizes the
same name and reads that context byte directly, before ordinary member lookup:

| Context byte | Value result | Native found result |
| --- | --- | --- |
| 0 | Undefined | false |
| 1 | Boolean true | true |
| Other nonzero | Boolean false | true |

Its result is independent of a stored Boolean or marker and its property flags.
A Value-only getter implementation must preserve the separate presence result
when answering `has` or descriptor-related operations. This proof does not
implement general enumeration or reflection.

## Ordered auxiliary version write or delete

After storing the context byte, the true branch interns the native version
text and key, then calls raw interface +0x50 at **0x1401e9a4a** with
`gfxVersion` = primitive String **`4.2.23`**, supplied member flags **0**.
The two literal lengths are respectively 6 and 10. This operation uses E's
string context and the same I receiver.

For this exact ordinary key, raw setter **0x1401e3b20** reaches its own hash.
It does not invoke a normal accessor setter, watch callback or parent lookup.
The incoming String bypasses the Array/listener hook and the exact key misses
reserved metadata/callback names. An existing entry's Value is replaced through
**0x14018d9d0 at 0x1401e3e87**, preserving its flags. A new entry reads the
supplied flags at **0x1401e3e99**, stores them at **0x1401e3eb3**, then inserts.
Thus a preexisting `gfxVersion` with ReadOnly still has its Value overwritten
by this raw write; new flags are zero. Unknown descriptor/object cleanup is
not admitted merely because the raw native path can accept more Values.

The false branch instead calls interface +0x30 at **0x1401e9aac** for that
same key. Complete delete **0x1401b8040..0x1401b80ed** performs own lookup:
missing entries and entries with **DontDelete bit 2** remain unchanged and
return false. Otherwise it removes the entry through complete
**0x140199fa0..0x14019a08a** and returns true. The special builtin-name branch
which dirties MovieImpl state does not match `gfxVersion`. There is no parent
lookup, accessor invocation or watch callback in this selected delete route.
The extension setter ignores the delete result and proceeds to the marker.

The complete removal helper includes collision relocation, retained Value/key
copying, count decrement and entry cleanup; its first 25-byte unwind fragment
was not a complete function. Complete **0x140199d80** releases the entry's
Value/key and marks the bucket empty. The initial profile admits ordinary
scalar/String or marker values with known flags, not arbitrary native objects
whose destruction would require further evidence. Native allocation/refcount
parity is not claimed by the logical Rust state model.

## Final marker, flags and partial outcomes

At **0x1401e9ad7**, the setter constructs **Value tag 10**, then calls normal
member helper **0x1401a1020 at 0x1401e9ae1** for `gfxExtensions`, forwarding
the original caller-flags pointer. It does not pass the Boolean as the stored
Value. This marker is a native representation choice; it must not escape as
an arbitrary ordinary Rust Value or be confused with a script accessor.

For a supported own ordinary/marker slot, the helper reads its existing flags.
**ReadOnly bit 4** returns false at **0x1401a11da..0x1401a11e0** before the
marker overwrite. An existing writable slot receives the marker while retaining
its flags. An absent slot uses caller flags zero from this exact SetMember.
Own data hits do not need parent traversal; absent slots may inspect ancestors
for an accessor. Watch state can invoke `0x140199330`. Those latter behaviors
remain unsupported and must be excluded by admission guards.

The setter returns this helper's result. The opcode ignores it, so a supported
read-only case completes the assignment instruction with the earlier native
effects still present. For example:

| Supplied descriptors and RHS | Resulting bounded effects |
| --- | --- |
| Both keys absent; true | Byte=1; version String flags 0; extension marker flags 0 |
| Version flags 7; true | Byte=1; version Value replaced, flags remain 7 |
| Version has DontDelete; false | Byte=2; version entry retained; marker attempt still occurs |
| Extension has ReadOnly; true | Byte=1 and version updated; old extension slot retained; false setter return ignored |
| Extension has ReadOnly; false | Byte=2 and permitted version deletion occurs; old extension slot retained |

These are derived static outcomes under admitted state, not native test runs.
In particular, a generic VM ReadOnly check performed before this native-global
hook would produce the wrong ordering.

## Bounded opt-in implementation seam

**INFERRED implementation design from the native paths:** distinguish an
explicit native-global/context binding from a Plain object. A fresh constructor
may create the proved checkpoint and preserve its invariants through supported
operations. A supplied-state profile must name the current global/context
association and carry actual logical parent, watch/resolver, descriptor and raw
extension state; it is not a certificate obtained from matching an ObjectId.

Before mutation, admit only the exact Boolean/fixed-name/source profile, valid
same-context receiver, known descriptor shapes and sufficient resource budgets.
A first implementation can require a parentless global with no watch/resolver
history, plus absent or own scalar/marker slots for the two keys. Native own-hit
paths can support a narrower per-operation ancestry rule later, without
claiming missing members through an incomplete native parent graph.

Once admitted, preserve the native order: set the byte; perform the version raw
write/delete; attempt the normal marker write; complete SetMember regardless of
its ordinary false return. ReadOnly and DontDelete are supported outcomes, not
reasons to roll back the earlier operations. Unsupported accessors, watchers,
coercions or contexts should reject before entering this profile; that bounded
admission policy is not a claim that retail throws the same error. Preflighted
Rust budget rejection is likewise not native allocation-failure parity.

Keep native-known global membership guards: this seam does not install all
other builtin identities, permit their absent reads, or implement arbitrary
global mutation. Getter presence, raw flags and effect ordering need focused
checks, including the two protected-descriptor cases above. No renderer, host
cursor, account, launcher or product code changed in this study.

## Evidence and provenance

All private paths are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. The owned PE SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

The carried initial review is
`rea-mouse-hide/gfx-extensions-setter-review.json`, SHA-256
`a948a5b82fe25a2378ad3700025b805687b4b9a4ede9458898b925dfa8ff40a2`.
The new original checker `rea-global-extensions/verify.py` validates
**1,535 unique retained/new instruction spans**, concrete table/name bindings,
shared constructor/Environment joins, and exact flags/effect branches. Its
`verification.json` SHA-256 is
`83fca8b24ea8b80af9457413898be5ca3db47bd2120f18df8a775cc184b1d202`.
The source/opcode/delete companion `rea-global-extensions/setmember-review.json`
has SHA-256
`2dab17ff8b0c8bad8f0388895e381294ce3d840d118c41b929e3c3425fac7e1c`.
It independently joins 15 canonical source records and nine complete native
ranges; the finite special-name exclusion is closed by the primary verification
above, rather than by treating an unidentified builtin as ordinary.

`rea-global-extensions/complete.bundle.json` retains four new complete,
exit-zero, untruncated, unchanged-artifact REA results. Canonical new records
are **ev_7523fdcc…** (complete selected opcode block), **ev_9031f065…**
(complete chained delete routine), and **ev_be980888…** (complete chained
three-value pop, entry cleanup, global interface and literals). The earlier
short prefix record is retained for provenance, not treated as complete routine
coverage. Full IDs, ranges and all reused bundle hashes are in the verification.
The ledger was exported and reset; its reset export is byte-identical. No
native process or original script was executed by this study.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| Owner's pinned executable and manager movie | Proprietary owner installation; authorized interoperability study | Exact type, fields, source operands, branches and data bindings |
| AS2_GLOBAL_BASELINE, FRESH_AVM1_ENVIRONMENT, FRESH_MOVIECLIP_PROTOTYPE, ATTACHED_VISIBILITY and their retained REA records | Original project analysis; target output stays private | Reuse constructor, context, builtin and normal/raw member evidence |
| Existing generic enumeration/property evidence | Original project analysis | Reuse collision copy and property cleanup paths |
| Installed REA skill/tool schemas and Rizin | Owner tooling; REA MIT, Rizin LGPL-3.0-only | Bounded static disassembly and evidence lifecycle only; no implementation copied |
