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
