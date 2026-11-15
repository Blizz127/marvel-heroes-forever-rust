# Saved-event ActionSetVariable assignment

**VERIFIED bounded static path:** raw opcode `0x1d` has its own native handler,
which calls ordinary assignment helper `0x140203170`. For a simple String
name, empty captured/With scope and the saved invocation's null activation
fence, assignment goes directly to the **current Environment target's normal
SetMember** interface. It does not choose a global binding merely because that
name already exists globally, and it does not search the target first to decide
where to write. This binds ActionSetVariable separately from DefineLocal.

The profile here is the existing pure saved Construct raw mode 4, coherent live
case-3 attached owner, current Environment version 11, no target changes, no
With and no dynamic/captured activation. Primitive String names and String or
Boolean right-hand values are the initial supported inputs. The target's full
normal setter remains responsible for native members, watches, accessors,
flags and storage. Routing an assignment does not prove that any particular
property write succeeds.

No original client was run. This document changes no executor or host state,
does not establish defaults, and does not claim native event/runtime parity.

## Exact owned source and controlled stop

The owned `marvel_login_dialog.gfx` has SHA-256
`56a39b5546161f3b96e96def43b81fba27103f20d8337079f778924d4904738f`.
Its saved Construct source is `39236+449`, aggregate native mask `0x40000`,
on placing tag 1345. The source-span SHA-256 is
`f68fe1b85573841b9c5a18018092b9e8e0b0ab34720a3ed0ee422cbce396f431`.
ConstantPool 39236 supplies 25 entries; Push 39506 supplies entries 0 and 1,
the name `autoSize` and String value `none`; opcode **39513 is `0x1d`**.
The private source-only decode is `evidence/gfx-buttons-study/targeted.txt`.

**OBSERVED controlled Rust diagnostic:** frozen `7f233ca` visits three
instructions and stops at 39513, before operand pops, with explicit error
`saved action SetVariable resolution profile`. The retained operand stack is
`[17, "autoSize", "none"]`; the supplied fixed registers remain unchanged.
No receiver storage exists before or after that rejected instruction, and
the rejecting host receives zero requests. An independent unsupported public
`autoSize` read is an inspection result, not this execution failure.

The supplied receiver is Scene 86, AttachedClip 2, owner ObjectId 20, target
`0x7a00000000000056`. Its source and selected Environment are movie 19,
header **11**. The saved action's placing source is movie 13, header **15**;
the root is a separate header-15 target. These identities and versions remain
distinct. Source association, parent/children, initial stack/registers and
native raw flag words in this diagnostic are controlled inputs. No native
initialization/default or full startup sequence is inferred.

The sealed private receipt is
`evidence/avm1-original-construct-saved-increment1/receipt.json`, SHA-256
`ccee48198d73654daa7354514d5b433caed4cce97f9c6e26400549fe5e63f165`.
The action's installed source wrapper and saved Environment contracts remain
those in [SAVED_EVENT_EXECUTION.md](SAVED_EVENT_EXECUTION.md).

## Independent opcode binding and operands

The interpreter's opcode switch uses table base `0x14020d34c`. Its raw `0x1d`
slot at **`0x14020d3c0`** contains RVA `0x209906`, selecting handler
**`0x140209906..0x1402099b5`**. This evidence does not borrow the `0x3c`
table entry or its mode-conditioned branch.

The handler uses the selected Environment's existing operand stack:

1. Select the second-from-top Value as the name, including its stack-page
   boundary address adjustment.
2. Call `0x1401a2920` at `0x14020992f` to convert the name, with this same
   Environment, an output String reference, numeric argument -1 and final
   Boolean argument false.
3. At `0x14020994a`, call `0x140203170` with Environment, converted name,
   actual top Value, captured/With vector and final Boolean argument true.
4. On normal return consume the two original operands, release the temporary
   converted name, and continue. No assignment result Value is pushed.

For an already primitive String name, the conversion's tag-5 table slot
`0x1401a2ebc` selects `0x1401a2988`. That branch retains the same StringNode
and publishes it at `0x1401a2e7a..0x1401a2e8d`; it performs no user script
coercion. The original top Value is forwarded unchanged, not converted into
a String assignment merely because the name is converted.

There is no local-declaration or raw-mode switch inside this opcode handler.
The saved-mode-specific behavior follows from its established Environment,
scope and activation state. Other modes are not thereby equivalent to saved
mode 4.

## Simple names, local fence and target selection

`0x140203170` distinguishes simple names from paths using StringNode cached
classification bits and, when needed, searches for `:`, `/` and `.`.
The scan calls at `0x140203230`, `0x140203240` and `0x140203250` target
`0x141c6f4ec`. That six-byte thunk jumps through IAT slot `0x141dbbf08`;
the pinned image's declared import is **MSVCR120.dll!strchr**. A successful
no-delimiter scan caches bits `0x06000000` and selects the simple-name path.
Microsoft's documented API returns null when the character is absent.
[Microsoft CRT reference](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/strchr-wcschr-mbschr-l?view=msvc-170)

This closes the delimiter dependency that earlier helper summaries treated
as a known search routine. The import declaration is static evidence; a
loaded IAT or CRT implementation was not inspected. The bounded name profile
is nonempty, NUL-free ASCII with none of those delimiters and original,
consistent intern-node classification. Qualified names and altered native
classification state remain outside the profile.

Simple dispatch at `0x140203264..0x140203275` passes the same four arguments
to **`0x140194f20`**, then returns internal true. The latter helper:

- Skips its scope-candidate read loop when the vector is empty.
- Calls `0x14018f070` at `0x140194ff4`. The null last activation returns null
  without searching an outer activation below that fence.
- At `0x140195045`, obtains the **current** target from `Env+0xe0`, uses its
  AVM offset and primary `+8` identity conversion, then calls the resulting
  owner's object-interface **`+0x18` normal setter at `0x140195087`**.

For the concrete attached owner that interface slot is `0x1401aa130`.
The supplied arguments are current Environment, converted name, the original
right-hand Value and zero initial property flags. The helper does not perform
a global lookup, a target-member existence check or a separate local insert
on this admitted route. A same-named global, inherited target property or
missing own target member does not redirect assignment; the target setter
decides the property's actual write behavior.

This differs from ordinary variable **reads**, which can fall back to global
after a target miss. Reusing a read-oriented generic scope search to select a
write destination would not reproduce this saved simple-name path. The target
is the Environment's current target, not necessarily the placing movie/root;
the no-SetTarget profile keeps it bound to the selected saved owner.

## Cleanup, native effects and bounded implementation

Same-page cleanup `0x140209991..0x1402099b0` releases each reference-bearing
operand and moves the stack cursor down by two 32-byte Value slots. Boolean
operands skip reference release. Complete helper `0x1401887a0` gives primitive
String tag 5 its StringNode reference decrement, with the existing interner
release path when the count reaches zero. The temporary converted name is
released separately at `0x14020983e..0x140209858`.

The page-crossing branch also loops for two operands and invokes stack-page
helper `0x140185840` when needed. Its cursor/page bookkeeping is retained;
underlying vector resize helper `0x14013d170` and allocation-failure details
were not audited here. No native page allocation-count equivalence is claimed.
Malformed native stack/underflow behavior is outside this normal-return proof.
Existing values below the consumed pair are not discarded by this opcode,
and it does not reset the persistent register bank, pool or saved Environment.

The simple assignment helper returns true independently of the target setter's
Boolean, and the opcode does not test that return before normal cleanup.
This is not evidence that an unsupported Rust operation should report success,
nor that a read-only native property necessarily changed. Native exception,
reentrant mutation and unsupported target-setter paths remain separate gates.

There are additional native effects outside the plain store itself. Name
classification can update interned-node cache bits. At `0x14020319f`, optional
movie diagnostic flag **mask `0x4` (bit index 2)** selects a temporary copy and
String conversion of the right-hand Value followed by diagnostic output.
The original assignment Value is retained separately and remains the value
forwarded to the setter. Diagnostic output and general coercion behavior are
not modeled or claimed equivalent here; the helper is not described as pure.

**INFERRED first implementation:** admit the established pure-mode-4 state,
validate the simple String name and bounded scalar value without losing the
pair on an unsupported gate, then route through the typed current target's
existing normal setter. After supported normal completion consume the two
operands and preserve prior stack/register/pool state. Keep each property
contract and context guard; this proof alone does not implement `autoSize`,
native watches, arbitrary accessors or qualified assignment.

## Evidence, validation and provenance

The pinned private Win64 build `1.52.0.1700` executable has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
New records are in `evidence/rea-saved-variable-assignment/complete.bundle.json`:

| Canonical REA ID | Coverage |
| --- | --- |
| `ev_25adf65d3416d77e0b8cea6271882653e65f54b18fbd084b0ed067e73362fb98` | Raw opcode table entry, complete selected handler, temporary-name release tail |
| `ev_d325155fe430b3175655bedfa9ea81d8360667b639c06433c6618b5792c57498` | Delimiter-scan import thunk; only its first complete six-byte jump is used |
| `ev_7136989a0b4ae1b35612e44bddcbe16bc3fbf8ad0f1245aaed529cbdb780c919` | Declared PE imports, exact `strchr` IAT/ILT/name binding |

Retained evidence includes `rea-saved-event-execution/semantics.bundle.json`
(`ev_359c226...`, `ev_ce06e425...`), `rea-saved-event-frame/final.bundle.json`
(`ev_0241bd...`), `rea-qualified-variable-entry/final.bundle.json`
(`ev_c4a75...`), `rea-deferred-builtin-values/conversions.bundle.json`
(`ev_4b7a27...`), `rea-asflags-comma-selector/final.bundle.json`
(`ev_7ffe6...`), and native-alternative `extends-name-binding-final.bundle.json`
(`ev_88b1ea...`) and `avm1-pop-boundary.bundle.json` (`ev_22b3e...`).
Attached owner slot/identity evidence is reused from attached-movieclip and
source-version-env bundles. Exact ranges and full IDs are in the receipt.

The original private checker matched **832 complete instruction spans and
184 table bytes** to PE section mappings, independently checked selected
IAT/ILT/name bytes, and verified the owned source span, first pool entries,
raw Push/SetVariable and sealed probe digest. `verification.json` SHA-256:
`f2f8c689b5839f1dc55e33f3642f42a4ce214986f69459ae1d0b23b09601bf3a`.
The complete new bundle SHA-256 is
`6fb7e54f568330954538830c17420445fbf0a4031194bacd174abb3a6fe56d89`.
REA exported/reset all three records with no retained target. Partial adjacent
instructions, runtime IAT resolution and native execution supply no claims.

Sources read: the proprietary owner-installed executable and owned movie;
original project saved-event docs, targeted source decode, receipts and private
candidate listing (GNU objdump, GPL-3.0 tooling); local REA skill/native guidance (MIT toolkit),
Rizin (LGPL-3.0-only) and Python standard library (PSF). The Microsoft CRT
reference linked above supplied only the imported API's delimiter-search
contract; MicrosoftDocs/cpp-docs documentation uses
[CC BY 4.0](https://raw.githubusercontent.com/MicrosoftDocs/cpp-docs/main/LICENSE).
The API contract is paraphrased; no Microsoft example code or external VM/SDK
implementation was copied. Original assets, bytecode, tables and native
disassembly remain private outside git.
