# TextField source, target movie, and Environment versions

**VERIFIED static evidence:** a resource definition's source header and a placed
case-3 target's MovieDef are separate native identities. They coincide for the
bounded local-definition/imported-local-export path described below. A field's
attached Environment instead comes from its first marked ancestor. A script
function can select a captured target's Environment independently of its receiver.
No rule makes all four versions equal to the current Rust VM's default version.

This supports preserving UISource's header **11** while executing a function
captured in a manager Environment with header **15**, provided the concrete
resource binding and Environment selection satisfy the witnesses below. It does
not justify inferring a target MovieDef from every definition source, or choosing
an Environment from script-code origin alone.

## Evidence scope and verification

The only binary studied is the owner's pinned final client:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses below are preferred-image virtual addresses. Offsets are hexadecimal.
Names such as `MovieDef`, `BindingTask`, and `CreateInfo` describe recovered roles;
they are not a claim that original source type declarations were recovered.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/source-version-env/`.
`complete.bundle.json` contains nine REA Rizin observations, 1,896,891 bytes,
SHA-256 `fa4083c0c93da04d41483534eb45f4d7b275a5f62b0f501c8d17c9e741128101`.
All nine report complete, exit zero, untruncated output and unchanged artifact.
The original `verify-selected.py` independently maps PE sections and compares
2,124 newly observed instructions, 94 retained instructions, and five pointer
slots against the pinned bytes: 8,512 unique instruction bytes and 40 pointer
bytes match. `verification.json` records exact end-exclusive ranges, record IDs,
and exclusions. Padding, neighboring functions, two partial/misaligned candidate
captures, and the initial unwind-only lookup fragment are excluded.

REA was used through target-free static commands after reading its skill and
native-artifact guide. No executable was opened or run. Needed records were
exported before `reset_investigation`; `reset-receipt.json` records nine cleared
records and no retained target. No native runtime, actual import-loader success,
script execution, rendering, or retail behavior parity was tested by this study.

## Version lookup and distinct constructor inputs

Retained version evidence establishes:

1. The movie header byte is parsed at `0x1400511b4`, initially stored at stream
   `+0x284`, and propagated to loader state `+0x54`.
2. DataDef virtual `+0x48` (`0x1400863a0`) reads that state version.
3. MovieDef virtual `+0x20` (`0x1400877f0`) follows MovieDef `+0x38`, BindingTask
   `+0x18`, then DataDef's version method.
4. Target version helper `0x140099650` calls target virtual `+0x208`, then the
   returned MovieDef's virtual `+0x20`. Getter `0x1400a12b0` returns target `+0xb0`.
5. Case-3 owner construction `0x140203560` supplies its embedded Environment at
   owner `+0x38`; Environment `+0xe0` holds its target, and `+0xf0` caches the
   target's movie version. Retained target-changing paths refresh this cache.

The factory at `0x140178440` takes a CreateInfo triple. In case 3 it passes
CreateInfo `+0` as the definition and `+8` as MovieDef to `0x1400b6d40`.
Case 4 similarly passes those two different words to `0x140143880`.
The latter forwards its MovieDef argument to base constructor `0x1400a1130`,
which stores target `+0xb0`; the field definition is stored separately at target
`+0xd8`. There is no constructor equality check proving the two inputs share
an origin. The case-4 table at `0x14250a800`, slot `+0x208`, is also
`0x1400a12b0` (new pointer check).

Normal placement `0x14020d8d0` gets parent MovieDef from parent target `+0xb0`.
When no CreateInfo is supplied, it looks up the placement resource ID through
`0x14008ccf0`, copies the returned triple, and sends it to the factory at
`0x14020dc26`. The supplied-CreateInfo path is separate and excluded here.

## Bounded resource-binding proof

The following conditions are essential; merely observing a source file is not
this witness.

| Step | Native observation | Consequence |
| --- | --- | --- |
| Binding owner | MovieDef constructor `0x14008d520` calls BindingTask constructor `0x14008c9a0` with itself in R9 and stores the task at MovieDef `+0x38`. The task constructor initializes a binding at task `+0x30` and stores that MovieDef at task `+0x78` (`0x14008ca69`). | Binding `+0x48` identifies its owning MovieDef. |
| Local export | Export lookup `0x14008ac00`, direct branch `0x14008acbf`, stores `[exporting MovieDef+0x38]+0x30` in result pair `+8`, and the resource in pair `+0`. | A directly defined exported resource carries its exporting movie's binding. Restrict to the established direct handle kind; the branch's `kind != 1` test is not permission to accept unknown kinds. |
| Import insertion | `0x14008d7b0` calls the imported MovieDef's export lookup at `0x14008d80f`. After export and importing resource-ID lookup succeed, `0x14008d881` calls pair writer `0x1400c9c20`. | Resource and binding are copied together into the importing binding table. The writer preserves both words. |
| Selected resource | `0x14008ccf0` keeps the current binding for a direct local definition; for an indexed/imported resource it reads the selected stored pair, including its binding. | The parent's MovieDef is not unconditionally reused for an imported resource. |
| CreateInfo | After nonnull resource and virtual `+0x10` result bit 15 checks, `0x14008ce07` reads selected binding `+0x48` and puts it in CreateInfo `+8`. | The selected binding owner, rather than raw definition source or parent MovieDef, determines the new target's MovieDef. |

Thus a directly defined export in UISource, imported into another movie with its
binding pair retained, creates a case-3 target whose own MovieDef is UISource's.
A nested local definition then resolves against that target's UISource binding.
The same selected-binding rule supports a direct local definition in its own
movie. These are the supported `LocalSourceOrImportedLocalExport` cases.

**The witness concerns the final selected pair.** The import binder also has a
later resource-list matching/copy path at `0x14008daab..0x14008db2d`. This study
does not classify that resource family or prove it cannot replace an arbitrary
slot. Reaching the first import copy alone therefore does not prove an export
binding remains selected. Root/frontend admission must establish the final
local/export/import chain, including no relevant replacement. Indexed aliases,
recursive export-name fallback, custom bindings, substitution, missing/failed
resources, unknown handle kinds, and supplied CreateInfo remain outside this
witness. They require a separately proven target MovieDef identity or rejection.

## Import compatibility is not header equality

The normal import-binding process `0x14008df70` receives an imported MovieDef
from loader `0x14004ef00`. At `0x14008e4b9..0x14008e4d4` it compares a bit in
main loader state `+0xd0` with the result of imported MovieDef virtual `+0x78`:
XOR followed by testing **0x08**. The associated diagnostic describes an
ActionScript-version mismatch. Slot `0x142504928` points to `0x140087870`, which
returns imported loader state `+0xd0`. This is distinct from header version
state `+0x54`; the branch does not compare header bytes 11 and 15.

**OBSERVED owned data:** `source-headers.json` records a bounded parse of the
retained uncompressed movies. `MarvelGame-frontend_manager.gfx` has header 15
and `!uisource.gfx` has header 11; both FileAttributes tag values are zero.
UISource exports character 1039 as `Btn_Stndrd_Primary`. The movie digests and
exact tag offsets are private in that receipt. This study has not traced the
FileAttributes loader into state `+0xd0`, or executed the full import loader.
Successful compatible loading remains an explicit condition. The bit test is
evidence against a universal native header-equality gate, not proof that every
combination of files loads.

## Field-owned state and Environment selection

Case 4 has its own target MovieDef but no embedded case-3 Environment. Its
primary table `0x142517970`, slot `+0xf0`, points to `0x14017a130` (new check).
That method gets target from owner `+0x20`, starts at target's parent `+0x38`,
and walks parents until word `+0x6a` has bit 10 (**0x400**) set. It recovers
that ancestor's owner as `parent + 4 * byte[parent+0x6d]` and calls the owner's
primary virtual `+0xf0`. A chain ending at null returns null. This is a parent
walk; it does not create a field timeline or choose the field's source header.

For an admitted case-3 owner, virtual `+0x08` is the identity leaf
`0x1405b93f0` and virtual `+0xf0` is `0x140203720`, returning owner `+0x38`.
A field beneath the imported UISource clip therefore obtains that clip's
Environment when it is the first marked ancestor. A directly imported field
beneath a version-15 clip could instead have a source/own MovieDef version 11
and an attached Environment version 15. Preserve both identities.

Normal field accessors receive the active Environment as a separate argument.
Setter `0x1401eb7d0` retains that Environment separately from the receiver
interface; receiver target `[interface+0x18]` selects the field state. The
`html` branch uses the passed Environment for conversion and writes receiver
flags. The `htmlText` branch at `0x1401eb938` separately calls target-version
helper `0x140099650` on the receiver. Getter `0x140195640` likewise selects the
receiver's target. Raw bridges `0x140203490` and `0x1402034f0` recover attached
Environment through owner `+0xf0` before forwarding normal accessors. Active
and attached Environment must not be interchangeable just because one path
obtains the other.

## Captured function Environment is another identity

ScriptFunction constructor `0x1401e14c0` captures creation Environment's target
public handle at function `+0x70` and target/root association at `+0x68`.
Its code reference is separate at `+0x78`. Environment getter `0x14017f710`
uses those captured fields and calls `0x14009a540`. A live handle's nonnull
`[handle+0x08]` immediately returns its target. If the target passes its bit-7
eligibility check, the getter recovers its owner and Environment. Otherwise it
falls back to FnCall `+0x30`. Invocation `0x14020e190` can also substitute the
caller Environment when selected target `+0x6a` bit 4 is set.

The bounded capture case therefore requires a live captured target, eligibility,
bit 4 clear, and no retirement/rebinding or unsupported target switch. The null
live-handle path invokes a resolver through the captured association; that path
is not admitted. A manager function can retain manager Environment 15 while
its receiver field belongs to source/target movie 11. The function's code-source
header by itself is insufficient evidence for this choice.

## Integration contract and limits

**INFERRED implementation contract from the verified branches:** preserve the
old same-profile API, and add an explicit binding witness for the bounded
local/imported-local-export profile. Store definition source movie/header and
selected case-3 target MovieDef/header as separate fields, even when the witness
makes their values equal. Store a field's source identity independently from its
own target MovieDef and first marked ancestor Environment identity. Carry active
function Environment separately in native requests. Source/header relabeling and
“every target uses Vm.movie_version” would erase native distinctions.

The original login fixture's Scene movie numbers and concrete import chain are
root/frontend responsibilities; they are not proven by assigning a Rust enum.
This document supports source 11 under the bounded selected target 11 with a
captured active Environment 15. It does not establish all version-dependent AVM1
semantics, remove any unrelated unsupported-operation gate, or prove an arbitrary
source 7–19 is equivalent to 15. Any widened admission range must retain every
existing opcode/native semantic restriction.

## Sources, licences, and retained record map

- Owned pinned executable and owned movie samples above: Gazillion/Marvel/Scaleform
  proprietary copyrighted artifacts; private interoperability study only. No
  binary bytes, disassembly, scripts, or assets are included in this repository.
- REA 6.9.0 skill and native-artifact guide at
  `~/.agents/skills/reverse-engineer-anything/`: workflow and evidence handling;
  tooling documentation, licence not established by this study. No implementation
  copied. REA invokes installed Rizin only for bounded static observation.
- Original project documents [TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md),
  [ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md),
  [REA_DISPLAY_LIFECYCLE.md](REA_DISPLAY_LIFECYCLE.md),
  [REA_NATIVE_ALTERNATIVE.md](REA_NATIVE_ALTERNATIVE.md),
  [AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md), and
  [AVM1_VM.md](AVM1_VM.md): project-owned findings and APIs, reused as evidence
  indexes. Native addresses and semantics were checked against retained records.
  No outside VM implementation was read.

The private `finding-summary.json` contains full new and reused record IDs and
paths. The principal new records are:

| Observation | REA record ID |
| --- | --- |
| Captured and ancestor Env getters | `ev_fe0edbf834bab2c0207456ec7a3e8793afd504843078c00ba01528149ca09f1a` |
| Selected CreateInfo binding | `ev_9b951ff4a8a25a48bd1ff5dc2ff842b261d5dfde3311de02079f1d08486b596a` |
| MovieDef and binding lookup | `ev_6ed1486ed2b8291c98fdec8e8875d13bee40762c37d8c8d7b6ec51daf73ebea8` |
| Pair writer | `ev_c2d92fd0ac7d5f43a62f46d7a0daf794960c5b6d5b90d13df2de42c4d2d1e2dd` |
| Import pair insertion | `ev_85330a1cfaf2a18b5a149276644fbea3808efe753cf7237c4067e9251c59f128` |
| Direct export and import compatibility | `ev_0684376d3792d0ef2043ce79628b5fbffd06c53d5df4ef3e2a1ac82e3096acb8` |
| Virtual slots and compatibility getter | `ev_c0cdcea45e0af155d4a6e0b185cda251d2dbd8503d934a29db4e2c7a7e543aec` |
| Live captured handle | `ev_a35bbbf7344bb95c4ee63211690d584dadda5931afd2ac6e88ab1383b3282191` |
| Case-3 Env leaves and complete tails | `ev_02263dbd05f95ac45707e5aef9e156d13a5f78298cb445de2873d48505022ef2` |
