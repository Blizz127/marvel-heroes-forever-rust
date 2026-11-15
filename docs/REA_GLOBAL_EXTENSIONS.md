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

## Numeric RHS extension: original initializer SetMember 13566

The exact-Boolean admission above is the initial bounded profile. This appendix
extends only its scalar conversion step; the global identity, context,
descriptor, source and ordered-effect guards remain necessary.

**OBSERVED in the parent Rust probe:** manager increment 12, frozen source
`bc320b30e341d31949fbd0d1696ecdd265d8033f`, completed one initializer (220
instructions), then stopped after 112 instructions of initializer index 1,
tag 383/sprite 34, at **SetMember 13566 / CodeId 1**. The error was the exact
Boolean admission gate. The supplied extension byte was unknown (`None`), and
remained unknown; no extension marker or last-write record was produced. The
containing frame did not execute. This is not the earlier 452-initializer
completion and is not native execution. The retained receipt is
`avm1-attached-manager-frame-action-increment12/receipt.json`, SHA-256
`85d8e4fead6ab94cecec88af7883b2c325bf53ac64b997dc1cd405427a5a05f4`.

**VERIFIED original source:** the same pinned manager contains anonymous
DefineFunction2 at **13523**, body **13534+135**, zero parameters, register
count 4, flags **0x169**. Its preloads are this=r1, root=r2, global=r3. There
is no preceding StoreRegister or branch in that body before the selected
assignment. ConstantPool **12444+931** contains `gfxExtensions` at index 7.
Push **13554** supplies `[Register(3), Constant8(7), Integer(1)]`, followed by
SetMember **13566**. The selected 13 bytes have SHA-256
`0a9103ed90726cb20e3c82c9e8fe584fc8f416ddba67d6016e8bfad8975035e9`.
The body SHA-256 is
`fb5a323085d45a36a03d026fc72de886429ce1379fb88a6cfe639a35927d4285`.
These are source/context facts, not a runtime operand-register dump. No original
script was rerun for this study.

### Native Push and scalar conversion

**VERIFIED:** native Push subtype 7 at **0x14020cba5** reconstructs the four
little-endian source bytes into EDI, writes **Value tag 4** at
**0x14020cc05**, and writes the 32-bit payload at Value+8 at
**0x14020cc08**. Thus this original literal reaches the setter as native
integer payload 1. It does not become native double tag 3 on that Push path.
The Rust VM's represented Number(1.0) preserves this exact zero/nonzero value.
Operand-stack page growth is a separate possible Push allocation, not an
effect of the conversion helper below.

The complete conversion helper is **0x14017d730..0x14017d87b**. The global
setter calls it at **0x1401e9991**, retaining the original RHS pointer:

- Tag 4 reaches **0x14017d828..0x14017d839**, compares the 32-bit payload
  with zero and returns `setnz`. Every nonzero 32-bit pattern is true; zero
  is false. Signed interpretation makes no difference to this conversion.
- Tag 3 reaches **0x14017d7c9..0x14017d827**. It reads the 64-bit payload
  and tests exponent/fraction bits. An all-ones exponent with a nonzero
  fraction returns false before floating-point comparison, including signed,
  quiet and signaling NaN encodings. Otherwise **UCOMISD at 0x14017d805**
  compares against the byte-checked positive-zero constant at
  **0x1425040c0**. Equality returns false; nonzero returns true, subject to
  the denormal caveat below.
- Tag 2 retains the previously established Boolean payload path.

The selected numeric arms contain no calls, no Environment/receiver
dereferences, and no non-stack object/state writes. Only the conversion is
scalar and free of external callbacks. On actual Integer(1), it returns true;
the enclosing setter then performs the earlier context-byte=1, raw version
write and final marker attempt in their established order. An old unknown
context byte does not prevent that admitted write from establishing byte 1;
it does not establish any earlier state or native default.

| Admitted represented input | Bounded Boolean result |
| --- | --- |
| Native 32-bit integer zero | false |
| Native 32-bit integer, any nonzero bits | true |
| Native tag-3 positive or negative zero | false |
| Native tag-3 NaN, any sign/fraction payload | false |
| Native tag-3 nonzero finite normal | true |
| Native tag-3 positive or negative infinity | true |
| Nonzero f64 subnormal | Unsupported until native floating-point mode is known |

This supports the existing Rust Number representation for exact native
32-bit integers and the admitted tag-3 payload categories. It does not admit
String, Object, DisplayClip, Null, Undefined or arbitrary conversion callbacks.
A bounded implementation must reject nonzero f64 subnormals before context,
version, marker or budget mutation. Integer-bit NaN/zero classification also
avoids introducing a new floating-point comparison for those classifications.

### Floating-point and origin limits

The current native MXCSR mode was not captured. DAZ can convert denormal source
operands to signed zero before computation; therefore this study does not
assert that every nonzero subnormal compares unequal to zero in retail.
See the primary [Intel SetDenormAreZeros documentation](https://www.intel.com/content/www/us/en/docs/ipp/developer-guide-reference/2021-12/setdenormarezeros.html).
No MXCSR investigation is needed for the selected integer literal 1.

The retained Push subtype 1 path uses **CVTPS2PD at 0x14020cac4** to promote
float32 to native tag 3. A float32 subnormal may be affected by that earlier
native hardware mode even when an independently widened Rust f64 would be
normal. This appendix therefore does not establish every float-literal origin
or all prior arithmetic results. Push subtype 6 reconstructs the SWF swapped
32-bit words into the 64-bit tag-3 payload at **0x14020cb50..0x14020cba5**;
conversion admission still depends on that resulting payload category. No
whole-VM numeric or native allocation-failure parity is claimed.

### Numeric evidence and provenance

Private `rea-global-extensions-numeric/verify.py` byte-checks **450 instruction
spans** across four canonical records, the 8-byte zero constant, the exact
original source slice and the unchanged parent receipt. It includes 40
call-free numeric-arm spans and derived checks for five integer patterns and
15 double payload cases, with subnormals explicitly gated. These are static
checks of retained bytes and derived outcomes, not native executions.

Its `verification.json` SHA-256 is
`5b351c20fc1d488ea740098fbe540ebc35c7f30689e1e50371ce40de8776f188`.
`selected-instructions.json` and `source-actions.json` retain the private
selected evidence. One new bounded REA capture closes the Push numeric tail:
**ev_0c47a81028c3ba523d26836a981e87f204fe3a201c0646573e41a72a1bb79cb7**.
`complete.bundle.json` SHA-256 is
`9a1cf1c493232c6011a15772d140d6962c82e0c34eef103a9c9f4a4e45df0290`.
It is exit-zero, untruncated and unchanged-artifact; the exported reset bundle
is byte-identical. Retained Push, complete conversion and global-setter record
IDs, full ranges and bundle hashes are recorded in the verification receipt.

This reuses the owned executable/movie and original REA tooling provenance
above. The additional outside source is Intel's copyrighted primary hardware
documentation, used only to preserve the unproved DAZ boundary; no code or
substantial text was copied. The source reference is retained privately in
`rea-global-extensions-numeric/intel-daz-reference.json`. No native client,
original script, host callback or cursor operation was executed. This appendix
is a single-agent byte/source audit; attempted helper reuse hit the agent-thread
limit and is not claimed as independent review.
