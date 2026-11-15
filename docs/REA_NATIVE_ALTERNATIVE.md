# Bounded native evidence through REA and Rizin

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED:** The installed REA `execute_rizin_command` route supplies static
native instruction evidence without Ghidra. Four successful calls completed with
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
