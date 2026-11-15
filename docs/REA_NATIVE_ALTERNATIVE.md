# Bounded native evidence through REA and Rizin

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED:** The installed REA `execute_rizin_command` route supplies static
native instruction evidence without Ghidra. The initial package/font study's four successful calls completed with
exit code zero, untruncated output, and an unchanged target digest. This resolves
the immediate lack of a working static instruction-analysis route; it does not
repair Ghidra, recover a complete call graph, or establish runtime parity.

## Target and method

The only game executable inspected was the owner's private
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`
under `~/.local/share/marvel-heroes-forever-rust/`: 48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The earlier `evidence/retail-version.json` records embedded version 1.52.0.1700.
The executable was never launched.

**OBSERVED:** REA 6.9.0 advertises `execute_rizin_command` and
`inspect_with_objdump` as available target-free operations. The installed Rizin
version command reports 0.9.1, commit
`c3a90e9226d977f58f4e9c75f78fa6b07afe13c7`. REA's command Evidence itself leaves
the provider version unknown; this separate version report is not executable
authentication. The objdump alternative was discovered but not needed.

Ordinary, read-only byte inspection first searched selected ASCII/UTF-16 strings,
mapped file offsets through PE sections, and found candidate x86-64 RIP-relative
`LEA` encodings. PE `.pdata` records supplied bounded unwind ranges. These scans
are candidate discovery, not REA observations or an exhaustive reference search.
REA then decoded the selected ranges and string locations. Calls used `psj` and
`pDj`, with 90,000 ms command budgets and at most 512 KiB of output admission.
No whole-image automatic analysis, debugger, target execution, Ghidra retry,
toolkit edit, registration edit, or `rea setup` was performed.

The initial 30,000 ms `ij` call timed out after producing metadata. Its partial
Evidence was exported, and a process check found no remaining Rizin process for
this target before following the tool's explicit deadline-increase remedy.
A subsequent completed call reported unsupported `psz` syntax and yielded no
disassembly. It is retained as a failed command, despite its process exit code
being zero. The installed Rizin help supplied the corrected `psj` syntax.

## Calligraphy package path behavior

**OBSERVED (REA instructions and literal strings):** The unwind range
`0x14160bd20..0x14160bf08` has 119 decoded instructions. References at
`0x14160bdf0` and `0x14160be32` load the `Calligraphy.sip` literal. The surrounding
code supplies it to two formatting calls with these exact format strings:

1. `Data/Game/%s`
2. `Data/Game/CalligraphyPak/%s`

At `0x14160be5a` it calls `0x141607c20` with the first constructed value. A test
of `AL` branches past the second call when nonzero; otherwise the call at
`0x14160be74` uses the second constructed value. **INFERRED:** This is a primary
path and fallback path for the Calligraphy package. The callee's full contract
and error behavior remain untraced.

The separately inspected `Frontend` substring is actually
`FrontendProtocol.proto`. Its nearby native code is not evidence of rendering
the login/front-end UI.

## Font map and locale-selected library

**OBSERVED (REA):** The complete contiguous range
`0x1412732f0..0x141273a11` decodes to 393 instructions and ends in `ret`.
It constructs a mapping object, supplies alias/face strings to
`0x140052760`, installs that object through a virtual call with state value
`0x14`, constructs a second object through `0x1400668c0`, loads one of five
font-movie paths, passes the result to `0x1400663e0`, then installs the second
object through the same virtual-call slot with state value `0x11`.
Object field/type names are inferred, not recovered C++ declarations.

REA also decoded all 35 instructions in
`0x1400668c0..0x140066948`. This constructor loads `gfxfontlib.swf` at
`0x14006692a` and passes it to a string-setting call. The font setup routine's
call to this constructor connects that library name with the locale-selected
font movie; it is more than a string co-occurrence.

The setup routine calls four predicates in this order. Each predicate calls
`0x1418aee40`, obtains the pointer at result offset `0x40`, and, when non-null,
compares its 32-bit field at offset `0x98` with the value below. Their diagnostic
strings include `currentLocale`. Null-pointer paths contain assertion/diagnostic
calls, so no harmless-null fallback is assumed.

| Predicate address | Compared value | Selected movie suffix |
| --- | ---: | --- |
| `0x141392be0` | 10 | `fonts_ru.swf` |
| `0x141392a90` | 0 | `fonts_cn.swf` |
| `0x141392b00` | 5 | `fonts_jp.swf` |
| `0x141392b70` | 6 | `fonts_kr.swf` |
| All four predicates false | Other values | `fonts_en.swf` |

The literal prefix is **`/ package/MarvelFonts/`**, including the space after
the initial slash. It has not been normalized into a different URL or UE3 object
name. The same predicate order chooses these native alias-to-face arguments:

| Alias | English/fallthrough | Russian | Chinese | Japanese | Korean |
| --- | --- | --- | --- | --- | --- |
| `$Strat2Bold` | Stratum2 Bold | Cervo Medium | Noto Sans TC Bold | Noto Sans JP Bold | Noto Sans KR Bold |
| `$Strat2Medium` | Stratum2 Medium | Cervo Regular | Noto Sans TC Medium | Noto Sans JP Medium | Noto Sans KR Medium |
| `$RobotoBold` | Roboto | Roboto | Noto Sans TC Bold | Noto Sans JP Bold | Noto Sans KR Bold |
| `$RobotoMedium` | Roboto Medium | Roboto Medium | Noto Sans TC Medium | Noto Sans JP Medium | Noto Sans KR Medium |
| `$RobotoRegular` | Roboto | Roboto | Noto Sans TC Medium | Noto Sans JP Medium | Noto Sans KR Medium |

The numeric fourth argument is `0x02` for the English/Russian `$RobotoBold`
mapping and `0x10` for the other listed mappings. Flag names/semantics are not
assigned here. The table combines REA-decoded argument-loading instructions with
separate bounded reads of the referenced literal bytes in the same hash-pinned
executable; the complete annotation is private. The independent UPK lane's
font-export observations are corroborating asset evidence, not a replacement
for this native selection logic.

**OBSERVED (REA):** A further 358-instruction range at
`0x14008ce50..0x14008d38b` includes a length check against 14, supplies a pointer
to a string tail and `gfxfontlib.swf` to `0x140034000`, and chooses numeric
diagnostic values `5` or `4` according to its result. **INFERRED:** This consumer
recognizes the library basename within a stored path. It does not establish the
complete import resolver for `../MarvelFonts/gfxfontlib.swf`.

## Evidence index

All paths below are relative to the private project root. Complete code bytes,
disassembly, literal inventories and target paths remain private, not in builds
or source control.

| Result | Evidence ID | Private bundle |
| --- | --- | --- |
| Corrected package/protocol string and range decode | `ev_9546303da75f84eec64d0c176d2547c833ca7fb1215f657eeed49e7e670d2b73` | `evidence/rea-native-alternative/selected-functions.bundle.json` |
| Font setup and named-library constructor | `ev_f19d0f6ec897253c0709fe0bae4159aa06bb2c369349200d064002483c30a3ae` | `evidence/rea-native-alternative/font-routing.bundle.json` |
| Four locale predicates and package format strings | `ev_b21ac28ea08dd5b166097670fa605cf53bd13b069597aa999c831acd63af4f43` | `evidence/rea-native-alternative/locale-predicates.bundle.json` |
| Selected basename consumer | `ev_d76954bb1cd5156b23d6bb4f8caba1dec58956c53e7ef537441461251db1862e` | `evidence/rea-native-alternative/font-import.bundle.json` |

The final cumulative checkpoint is
`evidence/rea-native-alternative/final.bundle.json`. It also retains the initial
timeout and unsupported-command results. Supplementary ordinary-file evidence
is `pe-string-candidates.json`, `font-string-candidates.json`,
`font-disassembly-annotated.json`, `font-import-instructions.json`, and
`finding-receipt.json` in the same evidence directory. Those annotations are not
relabeled as REA-produced evidence.

## Coverage and remaining unknowns

- The native locale selector's initialization from configuration or account
  state is untraced. `INT` and `eng.all` do not yet prove a selected numeric value.
- Full relative-URL/package/object resolution and the live UI's library bindings
  remain unverified. Font-family aliases do not prove glyph metrics, shaping,
  fallback, rendering, or runtime correctness.
- These are bounded instruction/string observations and PE unwind ranges.
  Rizin did not recover original function names, complete function boundaries,
  every indirect target, or a whole-program call graph. In particular, a `.pdata`
  range may cover only part of a split function.
- Rizin warned that its automatic string inventory skipped large regions.
  The explicit reads above succeeded, but no exhaustive string-coverage or
  absence claim follows. Some `psj` encoding labels are heuristic and disagree
  with the plainly ASCII underlying bytes; string-byte observations take priority.
- REA records unchanged pre/post digests, not a guarantee that no concurrent
  process could change a file transiently. The selected private pin was not
  modified by this investigation.
- Ghidra's existing Java-readiness limitation remains as recorded in
  `REA_NATIVE.md`. No live client or server validation was performed.

## AVM1 empty `Pop` follow-up

**VERIFIED (static native control flow):** On a valid initialized stack at zero
logical depth, opcode `0x17` preserves depth zero, clears the restored sentinel's
tag byte to zero, and returns to the action loop. Its empty-page branch does not
raise a stack-underflow error or call an assertion routine. This supports a
scoped empty-`Pop` no-op in the original Rust VM; it does not establish permissive
underflow behavior for other stack-consuming actions.

The AVM1 lane supplied the motivating private movie observation: the first
manager `DoInitAction` at movie offset 7,239 reaches `Pop` at offset 8,511 with an
empty operand stack on both known namespace branches, while the original Rust
VM initially stopped with `StackUnderflow`. This report investigates the native
handling, not a new run of the retail application.

The proof consists of these independent static anchors:

1. At `0x14020912e`, the native action loop reads the current opcode byte. It
   routes high-bit opcodes separately, and at `0x140209145` bounds the low-opcode
   dispatch by `0x69`. The table load at `0x14020914e` uses RVA `0x20d34c` with
   the raw opcode as its four-byte index.
2. REA reads the four bytes at the opcode-`0x17` table entry
   `0x14020d3a8`. They encode RVA `0x209749`, binding `Pop` to
   `0x140209749` in image base `0x140000000`.
3. That handler checks the current slot's tag for cleanup, subtracts 32 from the
   top pointer at environment offset `0x08`, compares it with the current page
   base at offset `0x10`, and calls `0x140185840` only when it crossed below that
   base. It then returns to the shared action loop.
4. The boundary helper receives the address of the top-pointer field. At
   `0x140185846` it checks the page count at helper-relative offset `0x28`
   against one. For a count at most one, it jumps to `0x1401858e3`, adds 32 back
   to the top pointer, writes tag byte zero at that restored slot when non-null,
   and returns normally at `0x1401858f4`.
5. The same action loop's depth computation at `0x140209ce6..0x140209d03`
   is `(page_count << 5) + ((top - page_base) >> 5) - 32`. Thus one page with
   `top == page_base` represents depth zero. The `Pop`/boundary-helper path
   restores exactly that pointer and count relationship.

**Scope:** This is a logical-depth no-op on the normal empty representation,
with a sentinel tag reset. The internal tag-zero semantic name and stack
constructor are not independently recovered here. Corrupt/null stack structures
and other opcodes are outside the conclusion. No original client was run and no
general stack-underflow policy was inferred from this one action.

The boundary helper spans multiple adjacent `.pdata` records; the first record
alone is only 41 bytes and omits the decisive empty branch. The followed code
ends at `0x1401858f5`; the subsequent `int3` bytes are padding outside its returns.
The bounded dispatcher prefix likewise ends inside a later unrelated instruction;
those trailing undecoded bytes are not part of the proof.

| Result | Evidence ID | Private bundle |
| --- | --- | --- |
| Raw dispatch, exact Pop table entry, handler and first helper fragment | `ev_9ad09fdbeaf5ea2d7e1863c3061aebe83f0432cd76e1d6119d12299f6dbb4853` | `evidence/rea-native-alternative/avm1-pop.bundle.json` |
| Full page-boundary helper and independent depth arithmetic | `ev_22b3e9316258dea0ee949de2291bbf916ba35f5605696c3d60877a53959d8dcd` | `evidence/rea-native-alternative/avm1-pop-boundary.bundle.json` |

Both calls completed with untruncated output, exit code zero, no command-error
diagnostic, and the original target hash unchanged. The complete follow-up
checkpoint is `evidence/rea-native-alternative/avm1-pop-final.bundle.json`.
An earlier probe of another large dispatch table is retained for audit but was
not established as AVM1 and is not used for this conclusion. Private candidate
JSON files remain explicitly ordinary-file discovery evidence.

## Native `ASSetPropFlags` follow-up

An independent helper performed four bounded REA calls in a separate session,
then exported/reset that session. Its complete private findings, source register
and limits are in `evidence/rea-asflags-native/finding-summary.json`; all four
records are retained in `evidence/rea-asflags-native/final.bundle.json`.

**OBSERVED (REA):** The pointer pair at `0x142adca70` references the literal
`ASSetPropFlags` at `0x14250d588` and code at `0x1401cb620`. The latter spans
three unwind fragments through `0x1401cba6c`, with 303 decoded instructions.
**INFERRED:** This is the native function binding. The registration consumer and
a live invocation were not traced.

**VERIFIED (static instruction behavior):**

- The set and clear arguments are converted to integers and each reduced to its
  low three bits. The selected-member setter argument and the all-members-style
  visitor callback both compute `(old_flags & ~clear_mask) | set_mask`. Setting
  wins when set and clear masks overlap. Old high bits survive in that argument.
- For exactly three arguments, the clear mask defaults to zero unless the
  environment byte at offset `0xf0` equals five, in which case it defaults to
  seven. That field's name and initialization, including any relation to SWF
  version, have not been recovered. It must not silently be treated as the
  movie's version byte.
- Selector tag `5` is converted to a string and split using an ASCII comma.
  Tag `6` obtains an object: internal object kind `7` supplies indexed elements,
  and kind `8` takes the string-conversion/comma-splitting path. Other selected
  tags/kinds return without mutation. Semantic names such as string, object,
  array and boxed string are inferred from those operations.
- Selector tag `1` invokes a member visitor with numeric option `0x0c`.
  The motivating startup call supplies `[prototype, null, 1]`, supporting the
  null-selector interpretation; the visitor's inherited/hidden-member scope
  remains unverified. For selected indexed names, the flags setter is called
  only when a member lookup reports success, and null element slots are skipped.
- The native routine does not directly access a return slot. This does **not**
  establish the caller's default return value as `undefined`.

For the motivating three-argument startup call, the static mask calculation sets
bit zero on members reached by the visitor while preserving other flags when
that environment byte is not five. If it is five, the calculation clears the
other low bits as well. This study does not independently assign public names
such as `dontEnum`, `dontDelete` or `readOnly` to those bits.

The inspected conversion helpers fast-return the signed 32-bit payload for
internal tag `4`. Their other paths map zero and exponent-all-ones floating
values to zero, truncate finite in-range values, and call additional math
helpers for large magnitudes. Complete coercion, comma/whitespace edge cases,
invalid calls with fewer than three arguments, visitor scope, symbolic flag
effects, and default return behavior remain outside the verified contract.

| Result | Evidence ID |
| --- | --- |
| Complete native body | `ev_3dc579930d25232b639ea0820bfafe3a0b26a1620c7c5f900a88a65eefbc5b5f` |
| Visitor callback, mask conversion helpers and related bounded instructions | `ev_3efb471712a41c834644763cee836639f11b77969f6fd3966d5e67b22f9fb910` |
| Exact comma and numeric constant bytes | `ev_42ab3e81cdfe1ee82ce645aa6c664ab830f55832984c8ea011404133adc38c0e` |
| Paired binding pointers and initial native fragment | `ev_a17441906ca8554599741d498b0e1e59075bda5a7a4e76b09c5cd7e1bee6f635` |

All four calls completed with untruncated output and unchanged target hashes.
The helper used only the same private executable, installed REA/Rizin tooling
and mandatory REA guidance already registered below; it wrote no client source,
changed no configuration and ran no original executable.

## Study provenance

| Source | Licence/status | Use |
| --- | --- | --- |
| Owner's private Win64 executable and existing version receipt, exact identity above | Proprietary Gazillion client; owner-authorized private interoperability study | Static strings, PE section/unwind data and selected native instructions. No client bytes distributed. |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/connection-and-recovery.md`, `references/native-and-artifacts.md` | Installed REA guidance, version 62; tool package MIT, no guidance/code copied | Supported alternate routing, timeout recovery and evidence preservation. |
| `/home/blizz/.local/lib/node_modules/rea-agents/LICENSE`, `dist/rizin/RizinCommand.js`, selected `dist/application/reverse/ReverseEngineeringService.js` lines and discovery matches under `dist/application/` and `dist/native/` | REA 6.9.0, MIT | Verify command construction and public Rizin/objdump timeout/control boundaries. No edits or implementation reuse. |
| Installed REA MCP schemas and returned command Evidence | REA 6.9.0, MIT tooling; returned game content remains proprietary | Select supported operations and retain provenance/limitations. |
| `/home/blizz/.local/opt/rizin-0.9.1/bin/rizin`, `share/man/man1/rizin.1`, installed command help, `include/librz/{rz_main.h,rz_core.h}` | Header SPDX `LGPL-3.0-only`; installed analysis tool, not linked or distributed; bundled dependency licences not audited | Static decoder, supported print commands and separate observed version. No tool implementation copied. |
| `/home/blizz/.local/bin/r2` wrapper | Owner-installed wrapper, no separate licence declared | Installation discovery only; radare2 was not selected for analysis. |

All project documentation added here is original analysis prose. No reference
implementation or recovered client function was copied into client source.

## First startup `ASSetPropFlags` call: scope, flags and return

This follow-up resolves the earlier visitor-scope, flag-name, binding-consumer
and default-return limits above. It does not resolve every argument coercion or
every native object class. The AVM1 lane identified the motivating call at
manager movie offset 12,374 as `(gfx.core.UIComponent.prototype, null, 1)` with
three arguments. Its prototype was replaced by `ActionExtends` at movie offset
8,705, whose superclass is `MovieClip`.

**VERIFIED (static native control flow):** The null selector visits the
`Extends`-created prototype's own stored members, including members already
excluded from enumeration. It does not traverse the prototype chain or invoke
member getters. Setting numeric flag `1` makes these members non-enumerable.
The registered native function leaves an `Undefined` result, which the
`CallFunction` handler copies onto the operand stack.

### Binding the actual prototype to the visitor

The raw opcode `0x69` table entry at `0x14020d4f0` points to `0x14020c4a4`,
which calls `0x1401e9d80`. Its successful extension path allocates an object and
calls constructor `0x1401e4310` at `0x1401e9fb2`. That constructor installs
interface vtable `0x142513cd0` at object offset `0x20`; slot `0x40` contains
`0x1401924c0`, the complete member visitor recovered here. The new object is
passed to the subclass prototype-assignment helper `0x1401911c0` at
`0x1401e9fca`. This ties the scope proof to the class used by the startup call,
rather than assuming that every native object shares one implementation.

The visitor at `0x1401924c0..0x1401926d4` walks occupied entries in the hash table
at interface offset `0x18`. Its precise option handling is:

| Visitor option | Native behavior | Null-selector option `0x0c` |
| --- | --- | --- |
| `0x04` | Include stored entries whose property flags contain `0x01`; otherwise skip them at `0x140192582..0x14019258b` | Included |
| `0x08` | For special value tag `0x0a`, pass a temporary tag-zero value instead of resolving it through object virtual getter `0x58` | Resolution skipped |
| `0x01` | After own entries, recursively call the linked prototype's visitor at `0x140192687..0x1401926c4` | No recursion |

Other stored values are passed directly to the callback. The flags callback
`0x140179ef0` uses the member name and current flags, computes the already
verified mask expression, and invokes the flags setter; it does not consume the
member value. Thus this operation does not execute accessor getters. The
special internal tag `0x0a` is deliberately not assigned an accessor type name
here. Native builtins represented by own stored entries are included; properties
absent from the own hash are not synthesized by this visitor. This distinction
prevents an unsupported claim that all native builtins are visited.

### Numeric flag effects

These public names describe effects traced through the same prototype's native
virtual methods, rather than names guessed from the mask arithmetic.

| Mask | Effect | Native consumer |
| --- | --- | --- |
| `1` | `dontEnum`: suppress enumeration | Raw `Enumerate`/`Enumerate2` reach `0x140208bf0`, which calls the visitor with options `0x0b` at `0x140208e61`. Option `0x04` is absent, so the visitor skips entries carrying property flag `1`. Its callback `0x140188340` pushes visited member names. |
| `2` | `dontDelete`: reject deletion | Raw `Delete` (`0x3a`) reaches `0x14020a269` and object slot `0x30`. This prototype binds that slot to `0x1401b8040`; its test at `0x1401b8090` returns false before removal when flag `2` is set. |
| `4` | `readOnly`: reject assignment | Raw `SetMember` (`0x4f`) reaches `0x14020b571`; its general member path calls object slot `0x18` at `0x14020b6a3`. This prototype binds that slot to `0x1401a1020`; the test at `0x1401a11da` returns false before value assignment when flag `4` is set. |

These are scoped behaviors for the recovered stored-member paths. Array index
fast paths, absent-member deletion results, arbitrary native subclasses, and
all inherited accessor cases have not been established by this flag proof.

### Registered binding and return value

The native registration consumer is now recovered: `0x14020f6a1` loads the
name/callback table beginning at `0x142adc940`, and `0x1401e9bd0` consumes its
16-byte pairs. The previously observed `ASSetPropFlags` pair at `0x142adca70`
therefore has an actual consumer. The loop creates a function object with
vtable `0x1425150c0`, places callback `0x1401cb620` at object offset `0x68`, and
registers its named function value. This supersedes the earlier inference from
adjacent pointers alone.

The raw `CallFunction` handler initializes its local result to value tag zero at
`0x14020a78e`, puts its address into call-structure offset `0x08`, and invokes the
function object's slot `0x50`. That slot resolves to wrapper `0x14018c5b0`;
both wrapper branches preserve the same result pointer when invoking the native
callback. `ASSetPropFlags` does not access the result field, and call-structure
cleanup does not modify it. The handler copies the result to the operand stack
at `0x14020a8a8..0x14020a8b2`.

The tag name has an independent opcode anchor: raw `Push` (`0x96`) subtype `3`
writes internal tag zero at `0x14020cc82`, while subtype `2` writes internal tag
one at `0x14020caea`. The Adobe SWF v19 operand table names these subtypes
`Undefined` and `null`, respectively. Consequently the unchanged return is
`Undefined`, and the earlier null-selector tag-one interpretation is also
grounded. This uses the primary specification citation already read by the
AVM1 lane; this helper did not reopen or copy the specification.

### Receipts and remaining limits

Seven complete REA records are retained in
`evidence/rea-native-alternative/asflags-scope-final.bundle.json` (SHA-256
`924946c886060bb00a71eb22bed863a99511d228f8a671ee1791bf9febfb074b`).
The private `asflags-scope-proof-receipt.json` lists the commands, exact
addresses, limitations and validation results. The separate five-record return
study is retained in `evidence/rea-asflags-followup-return/final.bundle.json`
with its detailed `finding-summary.json`.

| Result | REA Evidence ID |
| --- | --- |
| Complete own-member visitor and flags setter | `ev_98b99cd0c3a2d13784a9234ec13683d46c66d31af33997387bd8ee3e12bb26e7` |
| Enumeration helper, raw Extends dispatch and base constructor | `ev_08e85d17a2f786f20e56b1e529d18ae148f9481d8ad5c4fb86e70d35ea829f04` |
| Extends allocation path and deletion flag check | `ev_2d00d168d94ae46572e5953c919b32938697f5b31f0bdde53b5d04693991762b` |
| Exact prototype constructor/vtable and name-enumeration callback | `ev_b86d375ba8c9ca1cdfd66172094d00fc60a72a691fd09a3e93dbfebeddf0b700` |
| Complete assignment method and raw Delete case | `ev_820980a2b7e2c4340592ed0dcd933652c281056d7b90c1a3849e64a5d1a0564a` |
| General SetMember virtual call and exact Delete/SetMember dispatch entries | `ev_403f108412a6bdc90b4eaaa9f9b12465557a439946b5b57a15a99daec08138bc` |
| CallFunction result initialization and result copy | `ev_e89e7eb58f807a894f6625381f61cf4bb1cf0841476ede648ed737eed353bfea` |
| Registration consumer, native wrapper and Push tag mapping | `ev_baa1bc0e27483ab8df8eee94de03d3795c651dc840a70de0902a005289939e72` |

All scope and return calls completed with exit code zero, untruncated output and
the original target hash unchanged. Both investigations exported and reset
their REA state. Some bounded windows include adjacent functions, padding or a
partial final instruction; claims use only the stated complete instructions
and followed branches. The automatic string-region warnings do not imply a
complete global string search.

The three-argument default clear mask still depends on the environment byte at
offset `0xf0`: zero except when that byte equals five, when the clear mask is
seven. A two-call follow-up establishes that the byte initializes to zero and
is refreshed when the target at environment offset `0xe0` changes or is restored.
The exact getter `0x140099650` calls target slot `0x208`, then the returned
object's slot `0x20`; the low result byte is cached. Receipts are in
`evidence/rea-asflags-followup-version/finding-summary.json` and
`final.bundle.json` (two complete records, unchanged target). Its relation to
the SWF version remains unproved at this checkpoint. This is an
explicit remaining dependency, not permission to replace it silently with a
movie-header byte. No retail executable was run, no runtime parity was claimed,
and no game data or recovered implementation entered this repository.

### Extends metadata and ordinary flag preservation

**VERIFIED (static native setup):** Both metadata members created on the new
prototype start with flags `3` (`dontEnum | dontDelete`). Constructor parent
setter `0x14018edf0` supplies flags `3` and a special tag-`0x0a` value for the
interned `__proto__` key, then stores the linked parent at interface offset
`0x10`. The subsequent Extends helper `0x14018ec90` supplies flags `3` and a
function value for `__constructor__`. Both use the raw member setter slot
`0x50` on the new object.

The names have an independent interned-table anchor: constructor `0x1401d4490`
constructs the builtin string array at offset `0x138`. Array constructor
`0x140181020` passes literal pointer table `0x142add2e0`, count `0x9c`, to
`0x140097f70`, whose loop interns consecutive pointers into consecutive
eight-byte destination slots. Exact literal-pointer reads at `0x142add550` and
`0x142add560` identify `__proto__` and `__constructor__`, giving the member
offsets `0x3a8` and `0x3b8` used by those setters. The flags-setting call's null
visitor includes these own stored members even though they are already hidden.
Consequently the unproved clear-mask choice is material for the real first
call: old flags `3`, set mask `1` produce `3` with clear mask zero and `1` with
clear mask seven. Equality on flags initially zero is insufficient to remove
this startup dependency.

The ordinary value-assignment helper `0x14018d9d0` copies the value tag and
payload without writing property-flags byte `+1`. Its old-value cleanup
`0x1401887a0..0x1401888bb` also leaves that byte untouched on every recovered
return path: it modifies payload slots and referenced-object reference counts,
and passes referenced objects rather than the destination value to cleanup
callees. This supports preserving flags on ordinary well-formed stored-value
updates, including string/object/function replacements. It does not establish
all property accessor, watch callback or inherited-write semantics.

Private supporting evidence:

- `evidence/rea-native-alternative/extends-baseline-final.bundle.json`,
  `ev_b1581a5e345f23786489233d0206cad994918ebfca3304e02b1d2abea26eef0f`:
  both complete metadata setters and initial flag arguments.
- `evidence/rea-native-alternative/extends-name-binding-final.bundle.json`,
  `ev_88b1ea56646a79ebfe7b823771f0b22d29b8e6c8bfcb4dcc64a0c4dca32602b4`
  and `ev_5de603be82d92b5907b35369248200732fda049b1d4c60fe5880ced8d9b92b76`:
  interned array construction/order, literal pointers/names, and complete
  value-cleanup control flow. Data after cleanup's last return is its dispatch
  table, not additional executed instructions.
- `evidence/rea-asflags-followup-return/assignment-flags-summary.json`:
  saved assignment-helper paths, now complemented by the cleanup evidence.

All three added REA calls completed without truncation, exited zero and
preserved the target hash; their sessions were exported/reset. The final
name-binding call also contains a separate, explicitly preliminary MovieDef
constructor lead for the continuing cached-byte investigation.

Additional provenance for this follow-up: the AVM1 lane's already-read Adobe
*SWF File Format Specification v19*, Chapter 5 `ActionPush`, printed page 69 /
PDF page 71, [primary specification mirror](https://open-flash.github.io/mirrors/swf-spec-19.pdf#page=71)
(Copyright 2006–2012 Adobe Systems Incorporated, all rights reserved; study
only), supplies the raw operand names. Existing project AVM1 notes and private
startup observations identify the first call and its `Extends` construction;
they are project-owned investigation records with their own source ledgers.
All other outside inputs are the same executable, installed tools and REA
guidance listed in the provenance table above.
