# Selected saved Scene handler installation

`frontend::saved_scene_handlers::install_selected` is an original checked bridge
from an explicit saved Scene selection to real VM source wrappers and per-owner
`SavedHandlers<SavedActionId>` tables. It does not execute bytecode, deliver an
event, create an Environment, choose defaults, invoke dynamic members or schedule
startup. Installing a wrapper is not evidence that its body succeeds.

The implementation at `4887db2` uses the retained AS2 placement/source contract
in [SAVED_CLIP_ACTION_LOADING.md](SAVED_CLIP_ACTION_LOADING.md), the table contract
in [SAVED_HANDLER_TABLE.md](SAVED_HANDLER_TABLE.md), and the existing dedicated
VM installer described in [SAVED_EVENT_EXECUTION.md](SAVED_EVENT_EXECUTION.md).
No native code or game data is copied into this module.

## Caller inputs and result

The caller supplies the owning `Vm`, exact `MovieLibrary` and saved `Scene`, its
`scene_adoption::AdoptedScene`, strictly increasing selected Scene action indices,
an explicit `Profile`, and bounded engineering `Limits`. Profile has no Default:
it repeats the conditional movie-binding assertions, requires the caller to
establish the same VM/session association, and asserts that the supplied
instances still retain these saved AS2 placement lists. A changed timeline,
replacement resource or later handler replacement cannot be inferred from the
static Scene. Numeric IDs alone do not establish a cross-VM association.

The result contains:

- `tables`: one ordinary `SavedHandlers<SavedActionId>` per selected owner node.
- `installations`: each new action handle and its `RecordSource`, in canonical
  Scene order. Provenance includes Scene action/owner indices, actual attached
  Clip owner, placing movie/header, placement tag, serialized record index and
  offset, full movie-absolute action span and original per-record mask.
- `unselected_bindings`: every remaining Scene action index, including frame,
  initializer and unsupported event bindings. Unselected does not mean supported,
  rejected, executed or eligible for delivery.
- `unbound_action_tags`: the unchanged ordered list of available action tags with
  no binding in this snapshot. Loading a manager movie does not bind its cursor
  handlers into a login Scene.

The returned tables and handles belong to the supplied VM lifetime. This API
creates a fresh table batch; it neither extends an existing table nor infers
cross-call deduplication. Duplicate indices within a batch reject. Distinct
record/instance indices remain separate installations even when their source
bytes or physical source spans are identical.

## Checked source-to-owner join

Before VM mutation, the bridge checks the exact library/Scene source identities
through `movie_binding::Validator`, exact Clip-map coverage, distinct attached
IDs, adopted root, source/header/parent and display bindings, retained saved-node
identity and frozen child snapshots. It validates every Scene action's owner
backlink once and requires canonical order. A selected non-Clip owner or binding
rejects rather than receiving a fabricated receiver.

For each selected owner, it follows the validator's ordered contributing
placement tags and selects the **last tag with HasClipActions**. That is the list
retained by the existing first-frame replay: a later placement update replaces
the whole list. The character-defining placement tag can be earlier and is not
an adequate substitute. The complete ordered list must match both the owner's
raw placement and all its Scene Clip action bindings, including unselected
records. An optional ActionNode source tag must agree with this exact tag.

The action's `movie_id` remains the placing source. It is independently checked
from the receiver's resolved definition movie and Environment header. Imported
receivers must not rebase action source offsets into their own movie.

The admitted source profile is recovered AS2 PlaceObject2/3 with header versions
7 through 19. PlaceObject3 HasImage (`0x10`) and opaque-background (`0x40`) variants
reject because the existing format parser is broader than the recovered native
route. The version restriction also excludes the parser/native ambiguity for
PlaceObject3 below version 6. The bridge checks original envelope mask and size
bytes immediately before the retained span, within its selected tag body.
It never derives a callable mask from the envelope's AllEventFlags union.

Selected per-record masks are Load `1`, Construct `0x40000`, or their combined
record `0x40001`; selected key descriptors and other masks reject. The native
loader's direct-copy rule grounds these masks. A combined record creates one
mode-3 wrapper shared by its two table keys; this installation does not make
combined-mask execution supported. Pure Load/Construct retain mode 4.

Selected source must be nonempty and begin with a nonzero byte. The native
installer skips a failing record; this explicit-selection API instead returns
a preflight error identifying the un-installable request. It does not silently
remove it from the caller's selected batch. Original bytes are parsed with
`Program::parse_with_limits(bytes, original_offset, limits)` over the full declared
span. Existing End and trailing ranges remain part of that span; no terminator
is added, source is not rebased to zero, and no post-End trimming occurs.

## Publication, limits and failure

All source parsing and selection/table budget checks happen before registration.
The bridge bounds total Scene bindings, unbound tags, selected records, total
selected source bytes, placement-tag work and record-list work, in addition to
existing binding and AVM decoder limits. Each owner's installation and split-key
reference budgets are checked before mutation. These are engineering bounds,
not recovered native allocation limits.

For each prepared record the bridge registers a fresh `Program`, calls the
existing authoritative `Vm::install_saved_action`, then calls
`SavedHandlers::install_from_vm` with that actual action handle. It does not
manufacture owner/mask facts for the table or install a second wrapper for a
split key. Registering separate Programs is a bounded implementation choice;
sharing a CodeId was unnecessary for this first bridge. Distinct wrapper state
is preserved either way.

Tables are returned only after the entire batch succeeds. `Error` identifies
the Scene action, owner node, stage and `vm_may_be_partial`. Preflight failures
leave the VM untouched. **Any failure once registration starts requires
VM/session discard**, including a later program budget, authoritative installer
rejection or table allocation failure. No rollback or safe partial retry is
claimed. The VM's existing owner eligibility, loaded Environment version,
first-instruction, full-Program-span and wrapper rules remain authoritative.
Successful allocation is the supported profile; native allocation-failure
behavior is not modeled.

## Focused checks

Four original synthetic test groups pass. A fabricated nested movie places the
same source twice and updates its child ClipActions list before the first frame.
Checks cover selecting the later complete list, distinct wrappers for the same
physical source in two instances, identical bytecode at different record offsets,
per-owner order, shared combined-mask split keys/mode 3, and full post-End ranges.
Unsupported masks, duplicate/out-of-order selections, wrong owner association,
altered source binding, first-zero/malformed programs and bounded work reject
before any program or wrapper is registered. A deliberately exhausted VM program
budget fails after one wrapper and reports mandatory discard. No handler executes.

The private wrapper includes both frozen `scene_adoption.rs` and this module
because the base commit precedes their final root module hooks; it edits no
product hook. Only this module's tests run (four adopter tests are filtered out).
The fabricated package contains only original invented names and bytes.

## Provenance

The bridge and tests are original project code. No dependency, external
implementation, new native capture or REA session was added. The exact source
ledger for the bounded contract audit is private at
`evidence/saved-scene-handlers/contract-audit/`: `receipt.json` SHA-256
`1e590f4bd5e4427a82cf193154668ea5a31cd8bbabbe714d15001df5c96f5d9e`.
It records 11 source/doc files, all matching base `4515aa4`.

Sources read: original project `src/gfx.rs`, `src/gfx/scene.rs`,
`src/avm1.rs`, `src/avm1/{vm,saved_runtime,saved_state}.rs`,
`src/frontend/{saved_handlers,scene_adoption,movie_binding,display_state}.rs`
and the linked original project contract documents. Their reused native
findings retain the owner's proprietary executable provenance in
SAVED_CLIP_ACTION_LOADING; no extracted native implementation is linked.
The actual-scene check below reuses private owner assets and immutable project
receipts solely for compatibility. Their paths/hashes remain in private evidence;
no original handler bytes enter git.

## Original saved login selection

**OBSERVED original-source inventory:** the explicit selection contains all 34
Clip Construct bindings from the saved login Scene, totaling 16,735 source bytes.
Every source is movie 13/header 15, while receiver definitions belong to movie
13/header 15 (23 instances), movie 19/header 11 (10) and movie 5/header 10 (one).
The login-button instance therefore preserves placing source 15 separately from
receiver 11. All 34 selected owners and physical spans are distinct in this corpus;
repeated-source/order cases remain the separate synthetic checks above.

The immutable pre-implementation oracle retains all 89 unselected bindings and
889 unbound action tags. An independent original-byte verifier reparses the
selected placement envelopes and passes 1,818 assertions against the frozen
inventory, without reading the bridge implementation. Its receipt is
`evidence/saved-scene-handlers/actual/oracle-audit.json`, SHA-256
`3121878d3ac9429584617e76e203f611fd2926df533b00d0748dd732c2d7153d`.
The selected-record oracle `expected-bindings.json` has SHA-256
`38b334bd2b69da666ec4588d0df7ec38af745d0d1f8e2ae6ca6a7bce4c407307`.

The private Rust harness uses a clean detached copy of
`4887db25bf1714914bf17b4b6de84dc3658d4efc`. Focused build and run exit zero; all
220 tracked files, harness sources and owner inputs remain unchanged. It adopts
all 147 saved nodes with the previous explicitly controlled 75-Clip/33-field
fixture, and supplies distinct nonempty stack/register states for all 34 saved
Environments before installation. Those stacks, registers, raw flags, prototypes,
formats and same-instance assertions are controlled inputs, not native defaults.

**VERIFIED controlled Rust installation:** 1,204 handler comparisons and 4,255
complete-adoption comparisons pass. The result has exactly 34 distinct saved
wrappers, CodeIds and owner tables, each with one Construct reference and no Load
key. Every installed action reports the original full span, mask `0x40000`, mode
4 and no first constant-pool offset. All 89 unselected indices and 889 unbound
tags match the frozen oracle exactly.

All 34 supplied Environment stacks, four-register banks, owner/target identities
and non-halted states remain unchanged. VM object count stays 126; observed own
enumerable global data values/flags, attached state, focus state, current field
snapshots and source buffers remain unchanged. Instruction count is zero.
No saved body, event, ExternalInterface, renderer, native process or server runs.
The public VM API exposes CodeIds and full spans through SavedActionInfo but no
registered-Program byte getter: source ranges are independently hash-matched to
original bytes, while exact registration admission remains the VM's own check.

Actual source has one selected record per owner; combined masks, per-owner
multi-record order and shared physical source are verified only by the synthetic
checks. Nothing here establishes that these original bodies execute successfully,
that the supplied live inputs match retail initialization, or that Load/Construct
should be delivered in this order. A later delivery phase must establish its own
owner/Environment/table association and runtime preconditions.

A private Python comparator syntax error was detected before comparison. The
initial successful build/run and its logs remain preserved; the comparator alone
was corrected and the final capture froze that identity. No product or Rust
harness change was needed. Both captures execute zero original instructions.

| Private artifact under `evidence/saved-scene-handlers/` | SHA-256 |
| --- | --- |
| `module-check/focused-result.json` | `5acb37eefa2e59f356471ca2bd0b27e13a1ac8394f43975c41a1287879c32173` |
| `actual/receipt.json` | `a0372e81298543c44052b8a1367e586c2787c9bd39213f0e43c545113cec1e23` |
| `actual/execution.json` | `fb588c4aeee885adcf967ee069e24b75979febbfc6ca17609721da0b45add16b` |
| `actual/MANIFEST.json` | `6410d2f8ef30fded2ea1f193f4edc217814697765c7c9fc7eb187c28db961d49` |

The execution receipt records the frozen dependency lock, complete source hashes,
private original package/movie/executable paths and hashes, authenticated native
name inventory, source oracles and fixture extension. Package and executable
inputs are proprietary owner files used only for compatibility; prior probe and
fixture code is original project work. The same explicit English font alias as
[LOGIN_INSTANCE_INPUTS.md](LOGIN_INSTANCE_INPUTS.md) is an inspection input,
not a recovered native locale decision. All extracted bytes, build products and
validation state remain private.

## Static dependencies of the 34 original Construct records

**VERIFIED static source correspondence; no new handler execution:** a separate
bounded parser dump covers the same 34 source spans in their original selection,
owner and per-record order. Independent raw-byte reconstruction matches all
4,889 dump rows / 45,288 columns byte-for-byte. The parser is the original
`src/avm1.rs` at `2d9b798`, with only its VM-module declaration omitted in a
private copy; its dependency-free binary neither compiles nor instantiates a VM.
The transform, frozen original/copied hashes and build dependencies are retained.

**OBSERVED:** all 1,686 instructions are in one linear top-level chain per record.
Each record begins with its single ConstantPool and terminates exactly at End,
without trailing bytes. The only opcode kinds are:

| Opcode | Count |
| --- | ---: |
| ConstantPool | 34 |
| Push | 792 |
| GetVariable | 118 |
| SetVariable | 556 |
| InitObject | 34 |
| SetMember | 118 |
| End | 34 |

No direct CallFunction/CallMethod, branch, With, SetTarget/SetTarget2,
DefineFunction/DefineFunction2 or Try instruction occurs. There are no nested
bodies whose availability could be mistaken for a reached function. This does
not waive runtime member/accessor/resolver guards: the absence of explicit call
opcodes does not prove all member paths return without callbacks or failure.
Later instructions are source-reachable only if earlier operations return.

A source-only literal/stack-provenance pass retains exact pool indices, original
numeric bits, opcode/operand offsets and conditional object aliases. It consumes
no unknown incoming stack values or registers, adds at most three stack values,
and has net stack effect zero. These are relative source properties, not chosen
native stack/register defaults. Its 556 SetVariable right-hand sides are 222
Strings, 285 Booleans, 34 fresh empty Objects, 13 double-encoded zeroes and two
integer-encoded ones.

Every record creates one empty object and immediately assigns it to `soundMap`.
All 118 GetVariable operations subsequently read that exact name; all 118
SetMember operations assign literal Strings through those reads. The alias to
the earlier fresh object is **conditional on successful ordinary write/read
behavior**, not an observed live lookup. No case or instance label is used to
infer the receiver or path.

The 54 exact SetVariable key spellings classify as 50 ordinary names and four
native Standards. The Standard assignments are `autoSize` ID 48 (20), `maxChars`
ID 65 (two), `password` ID 68 (two), and `wordWrap` ID 49 (two). Ordinary name
admission alone is not write-success evidence. The pool also contains `scroll`
ID 57 once, but operand provenance puts it on the conditional **plain soundMap
object** at offset 43610, not on a Clip. It is not an additional Clip Standard
assignment or a reason to extend that dispatcher.

### Exact implementation profiles and next dependencies

The initially proposed primitive-only saved SetVariable change became
`640e4d1`. Static analysis showed a fresh Object RHS restriction in all 34 records
and earlier Number RHS restrictions in four. That finding prompted the separate
raw-value preservation change `eed5e65`; the audit pins both source revisions
instead of describing the earlier limitation as current at every revision.

**INFERRED candidate selection at exact `eed5e65`:** 32 of 34 records have no
remaining identified opcode, operand or Standard-name gate. This source preserves
the original RHS Value through the normal current-target setter, consuming the
two operands only after success. Normal owner/Environment, name, storage,
prototype, property, allocation and budget conditions still apply. No successful
execution of those 32 records is claimed by this audit.

The remaining records are Scene actions 84 / owner 66 and 86 / owner 68. Their
exact unresolved Standard writes at this revision are:

| Action / owner | `maxChars` ID 65 | `password` ID 68 | `wordWrap` ID 49 |
| --- | --- | --- | --- |
| 84 / 66 | Offset 36629, double zero | 36637, false | 36733, false |
| 86 / 68 | Offset 37044, double zero | 37052, true | 37148, false |

These owners are adopted case-3 Sprite components, not direct case-4 TextFields.
At `eed5e65`, the implemented attached-name exception admits only exact ID 48
`autoSize`; IDs 49/65/68 require their own established case-3 route. A parallel
native study reports their ordinary fallback, but that report is not silently
assumed implemented in this frozen source comparison. Future fixes and runtime
checks need their own revisions and receipts.

For the narrower primitive profile, Number operands occur only in actions
64 / owner 32, 66 / owner 34, 84 / owner 66 and 86 / owner 68. The private report
retains all 15 exact names/bits/offsets and each record's first dependency, so the
later raw-RHS fix can be assessed without executing or changing source. No With,
target-switch or function-call implementation is required merely to decode this
particular Construct subset; broader startup frames remain separate work.

Private `evidence/construct-opcode-audit/analysis/per-record.tsv` retains every
selected ordinal, Scene action, Scene/attached owner, source/receiver movie and
header, source span, mask and previously installed action ID. `records.json`
retains complete typed assignments and source witnesses;
`runtime-profile-eed5e65.json` records the version-specific 32/2 grouping.

| Private artifact under `evidence/construct-opcode-audit/` | SHA-256 |
| --- | --- |
| `parser/receipt.json` | `94769bd43576c591cd985d27ff707992cf444cab339c09dd50fc44a07ccbca25` |
| `parser/MANIFEST.json` | `9bd3f1abdef1b948754505bb73a7871b49ed7d80784c1cc09a8c549286ae8819` |
| `contract/receipt.json` | `c8ffdb692c0f4b807937bb2fa8463ef672387b41484f31546b5042abffa4bf87` |
| `standard-names/MANIFEST.json` | `5296441d08fe92c85ce1dcf14743362d32406dd3c30fd799930d0b75e82606e2` |
| `analysis/receipt.json` | `10535d26ddce6887b3608a304806ff093f284ff165ceadb4d58f2b4b204389a1` |
| `analysis/MANIFEST.json` | `c6ced935c476e72afb28966ef5f1660e3e9c36d03dd6e7534887dd424ba5dbfd` |

Provenance is the same private owner-movie spans, previously authenticated native
name inventory and original project code/docs already recorded above, plus
frozen `saved_dispatch`, attached TextField, TextField host/state and saved-action
contract sources. Component source ledgers record each read and licence/use;
`analysis/profile-sources.json` pins the additional `640e4d1`/`eed5e65` files.
No website, new third-party implementation, REA/native process, event delivery,
VM reexecution or product-code change was used for this static audit.

## Controlled full-scene saved Construct phases

A separate private execution stage uses the complete saved 147-node login Scene,
all 75 adopted Clips and 33 fields, the original 34-record installer, and
`saved_dispatch::execute_saved_phase`. Every target, prototype, Environment,
current text/format state and one-context association remains an explicit
controlled fixture. Selected phases are requested in saved Scene selection order;
that is a test input, not a recovered native queue or event-production order.

The source is a clean, owner-authorized private composite
`f251a7e44c30c31beada860f116c0140a1f0043c`: full root `c860b1a` plus native-name
proof `db8ce34` and its finite case-3 implementation `f2b2a5f`. Their local
cherry-picks are `77ebcc3` and `f251a7e`. Only the three documented files differ
from the clean base; root's separate in-progress visibility-host work is excluded.
`evidence/construct-saved-phases/composite.json` records the exact construction
and source hashes (SHA-256
`cd8e39ad08224a47e15e6a619b3576f515da2f2b4027b5a881c660871abd5005`).
An earlier 32-record preparation at `c860b1a`, excluding actions 84/86, remains
preserved as **unrun**. It was superseded before any harness build or execution
when the exact three-name implementation became ready.

The complete input selection installs all 34 records, retaining the 89 unselected
Scene bindings and 889 unbound tags. It supplies distinct nonempty saved stacks
and four-register banks, then requests each owner's immutable Construct vector
once. The harness stops the entire batch on the first unexpected body error,
without retrying or continuing a partial VM. All nine Host entry points count
attempts and reject; no interface, property, text, class-registration, command or
visibility callback is silently accepted.

An independent read-only oracle was prepared without seeing runtime output. It
retains all 556 source-derived target writes, 118 exact String writes to 34 fresh
soundMap objects, all original pool entries/references, and each source's expected
instruction count and relative stack behavior. Duplicate target or soundMap
names and repeated selected owners are absent. Under the explicitly untouched
fixture, source inspection predicts 68 new VM objects; this accounting is a
conditional product expectation, not a recovered native allocation count.

A returned `DynamicLookupPending` means only that the selected saved vector
returned. Dynamic `onConstruct` lookup remains pending. No component constructor,
initializer, dynamic handler, complete event delivery, UI behavior, native startup
state or game parity follows from that outcome.

**VERIFIED controlled saved-phase run:** all 34 requested phases return
`DynamicLookupPending`, each with one completed handler and no skipped handler.
Their completed bodies account for exactly 1,686 original instructions. All 556
target assignments and 118 soundMap member assignments match the independently
prepared source oracle, including both previously excluded component records.
The 34 soundMap Objects are distinct. VM object count rises from 126 to 194,
exactly the predicted 34 attached storage objects plus 34 fresh plain maps.

All 34 supplied nonempty Environment stacks and four-register banks retain their
exact values. Owner/target, source/header, parent, prototype and saved action/code
identities remain unchanged. Each wrapper's first pool marker becomes `Some(0)`;
constant-derived names and values match the original source. The public API has
no retained-pool byte getter, so this does not claim direct internal pool-buffer
inspection. The source pool bytes and indices remain independently checked by
the static audit.

There are zero attempted calls across all nine rejecting Host routes. Observed
own enumerable global data/flags, the display and text hosts, all current field
snapshots, complete saved Scene/actions and original asset buffers remain
unchanged. Attached Clips gain only the expected storage association and data;
other owner identity/state checks remain exact. No visibility/text host mutation,
ExternalInterface installation/call, native process, UI rendering or server
operation is involved.

The private harness's initial compile exposed duplicate wrapper imports; the
failed source/log were retained and corrected before any saved body ran. The
successful frozen harness makes one all-34 execution pass. Its complete 233-file
tracked source manifest, harness/dependency inputs and owner asset hashes remain
unchanged. Neither the unrun 32-record preparation nor any partial VM is replayed.

The final independent comparison also passes: 8,103 direct scalar/collection
comparisons plus 4,255 checks of the pre-execution adopted structure, for 12,358
total. It matches all 674 values, 34 retained Environments, 68 distinct new
objects and 102 soundMap metadata flags against the frozen oracle and captured
single run. It performs no additional body execution. Owner own-property flags
and retained pool contents were not directly read back; this preserves the
inspection limits above and establishes no dynamic or complete event delivery.

| Private artifact under `evidence/construct-saved-phases/` | SHA-256 |
| --- | --- |
| `composite.json` | `cd8e39ad08224a47e15e6a619b3576f515da2f2b4027b5a881c660871abd5005` |
| `oracle-all34/expected.json` | `8a7d9cc730438451c96e222148dde03e6e6795cd8106f33bf179de74da1eb3b0` |
| `oracle-all34/receipt.json` | `c1693db99652492db270dbae5bb525c976367f187d00483905549b6c52ac1242` |
| `oracle-all34/MANIFEST.json` | `7499145e8f7f6b0fbf53890d27968cedbc91ca64b4e336525e99af03637f5b5e` |
| `all34/execution.json` | `429752f76235bc1dabf1a2a21d73a20c886300acd07592122beaa1bd6252890e` |
| `all34/receipt.json` | `4b414c46fa02f4321da209f2be163e8cd9a2e97a9239081426a7566728612e94` |
| `all34/independent/comparison-receipt.json` | `98fcf4a430225aff5a9d5453d4a027479d146fd6b1b6086cabb99ecb1e76ab89` |
| `all34/MANIFEST.json` | `069828f9b61b4ba7dbd7f8f63c9cfc1c7ff1fb9b90d264d30494778de94b2f09` |

The successful private binary has SHA-256
`1d2063d7fcdfa9bbc9d982f3aee7d59562cba30b00fc7deb4e8443e11dba33e6`.
The execution receipt binds complete frozen sources, dependency lock, controlled
fixture inputs, original package hashes and the single run's output. The oracle
has its own source/data ledger and was prepared without runtime-output access.
No asset, captured state or private executable is added to the repository.
