# Bounded attached `_visible` binding proposal

## Host implementation increment

**VERIFIED Rust host checks:** `DisplayState` now retains the bound clip's
root, owner, display target, source and parent identities. Its typed preparation
method validates those associations, the caller's bound context and local-bit
coherence before staging effective visibility for the subtree. The returned
borrowed `FnOnce` publishes the result, trace and revision with no further
fallible work. Dropping it changes no node, trace entry or revision; diagnostic
attempt/work counters and reserved capacity can reflect preparation.

Only visibility Booleans are recomputed. Matrix bits, descendant local flags,
other branches and immutable saved metadata stay unchanged. The host rejects
generic indexed/direct Visible writes to an attached node, including Object-ID
and empty-path aliases, so that path cannot bypass the VM's authoritative bit.
Plain diagnostic-object writes keep their existing behavior. `FrontendHost`
also checks its owning root before forwarding the staged request.

All seven focused DisplayState checks pass, including the existing geometry
checks and three new staging/identity/coherence/error cases. The synthetic tests
use actual VM attachment and saved-scene bindings, explicitly update the supplied
VM bit between prepare and commit, and establish no native fresh-state defaults.
Receipt: `evidence/attached-visibility-host/focused-checks.json`. The separate VM
setter hook and its policy/base-operation admission are required before script
execution can use this host path. No original constructor or GPU run is claimed
by these host checks. The native scope and proposal below remain their basis.

**INFERRED implementation contract, grounded in verified native instructions:**
the original exact-Boolean `_visible` assignment can update a bound Rust display
node when its caller supplies current native state and the native status branch
is unchanged. This document proposes that narrow integration; it does not enable
a setter, execute a constructor, initialize flags, or hide a named panel.

Native evidence is in [ATTACHED_VISIBILITY.md](ATTACHED_VISIBILITY.md), including
its initialization appendix. The source audit below is pinned to project commit
`2d9b798`. The existing getter reads the receiver's supplied local bit; the named
setter is still unsupported in that snapshot.

## Admission inputs

| Input | Required meaning and source |
| --- | --- |
| Current VM/root identity | The explicitly adopted attached root of this VM and its matching FrontendHost. Numeric ObjectIds from another VM are not interchangeable. |
| Active caller context | The actual interpreter Environment target at SetMember, bound in that same root context. It is provenance/context, not the receiver or a source-movie selector. |
| Receiver | A live supported case-3 AttachedClipId, its owner ObjectId and DisplayTargetId, with the admitted source/parent binding. Both a DisplayClip value and its owner must identify this same receiver. |
| Name and input | Exact `_visible`, authenticated native standard ID 7, and `Value::Bool(false)` or `Value::Bool(true)`. No coercion, spelling substitution, TextField or ordinary-object promotion. |
| Current local flags | The receiver's existing `AttachedEventState.target_flags`, representing native DWORD `+0x6a`; bit 14 is its local visibility. This is mutable state, not a placement/default. |
| Current status | The receiver's existing single shared `+0xb8` word from `attached_target_status`. An unbound word rejects. Do not add a separate visibility copy that could diverge from focus/tab state. |
| Current movie policy | An explicit current bit-11 value for the receiver's owning MovieImpl, associated with this root instance. No default, inferred zero word or per-import/source-MovieDef policy. |
| Host association | An existing exact receiver-to-DisplayState-node association, including owner/source/parent checks and the matching VM/root domain. No path/name search, filename guess or automatic binding. |
| Base-operation profile | Explicit normal-completion, stable-policy/status and non-reentrant assumptions for the selected base-node operation, as distinguished below. Scene-node existence does not prove native backing-node state. |

Only bit 11 is needed from MovieImpl `+0x4310`; a typed Boolean is sufficient
if its caller explicitly supplies that bit's current value and identity. A
caller that has the full current DWORD may extract it. Neither representation
may have `Default`. The policy belongs to the root **instance**, not a function's
CodeOrigin, the receiver's imported definition, a SWF version or the startup
manager's filename. Multiple roots require independent associations.

For a first bounded implementation, install one explicit policy snapshot on
the adopted root and reject unmodeled policy-changing operations. A later
supported policy update must replace the live value and invalidate outstanding
prepared writes. Do not freeze a policy and then allow the known native
`noInvisibleAdvance` route to change only an ordinary heap property. Native
initialization checkpoints alone do not establish the value at these writes.

At every admitted write, compute from the **current** fields:

```text
old_local = (target_flags_6a & 0x4000) != 0
old_status = (target_flags_b8 & 0x8) != 0
desired_status = !requested_visible && movie_bit11
require old_status == desired_status
require bound_display_node.local_visible == old_local
```

The equality is the admission condition, not a request to overwrite bit 3.
Reject a mismatch before any modeled visibility mutation. For false, it is
`old_status == movie_bit11`; for true, it is `old_status == false`.
No parent status, saved placement visibility or policy default can replace
these inputs. A hidden ancestor is compatible with a locally visible receiver:
coherence uses **local**, never effective, visibility.

The native equality test occurs **after** base-node work. Preflighting it from
before-state is valid only when that work preserves the two compared bits.
Retained complete bodies establish two finite cases without recursively
reversing the allocator:

- **VERIFIED conditional existing-node/equal-backing case:** target `+0x48`
  is non-null and backing bit 0 already equals the request. The false-path
  obtain-node helper returns the existing node without allocation/refcount
  work. The node helper returns at its equality branch before dirty work.
  Only the receiver's local bit changes.
- **VERIFIED conditional existing-node/change-record case:** target `+0x48`
  and node `+0` change-record pointer are non-null. On a backing difference,
  writable-data helper `0x14004a420` takes its existing-record branch, ORs
  record mask 4 and returns the existing backing. It calls no backing virtual
  or record allocator. The optional `0x140048980` notification is a 41-byte
  leaf containing four intrusive-list stores and no calls. Reachable writes
  affect local bit 14, the record mask, backing visibility and list links.

Both proofs require valid ordinary disjoint typed allocation regions, a
disjoint stack, and no concurrent/reentrant mutation. Under these explicit
conditions, their writes do not overlap target `+0xb8` or MovieImpl `+0x4310`,
so policy/status are stable. They do not establish that a particular saved
Scene or live frontend instance has those native pointers or backing values.

The core Rust API need not expose or fabricate native pointers. It may instead
admit an explicit **successful ordinary-allocation/no-reentry projection**:
its base operation changes only the modeled visibility/render state and
preserves policy/status, while native allocation/backing bookkeeping
equivalence remains unverified. This is an implementation profile supplied
by the caller, not a newly proved native invariant. Do not infer that profile
from constructor intent or silently assume allocator callbacks are inert.
If the caller cannot admit either the finite native case or that explicitly
conditional projection, reject before mutation.

## Native sequence and its Rust projection

**VERIFIED native sequence on normal completion:**

1. The standard ID-7 branch bypasses the extension watch path, ordinary lazy
   storage, ordinary descriptor flags and inherited setters. Exact Boolean
   copy/conversion reads its payload; it invokes no coercion callback.
2. Base `0x14009ad30` writes only target WORD `+0x6a` bit 14. It does so even
   when the old local bit already equals the request.
3. For true, it updates the target's existing render node if one exists and
   does not create one. For false, it obtains/creates the target's associated
   node, initializes a new node from the already-cleared local bit, and applies
   false. A new node's second false application is an equal-backing no-op.
4. Node helper `0x1400a93e0` compares its **backing** visibility bit. Only a
   difference requests writable data/change mask 4, changes the backing bit
   and conditionally queues the referenced link. This backing equality is
   distinct from local-bit equality and status equality.
5. Case-3 wrapper `0x1400b6c00` checks `desired_status`. On the admitted equal
   branch, it returns without changing any `+0xb8` bit, consulting the status
   helper, altering its target list or propagating descendant status.
6. Standard member dispatch returns handled/success. No ordinary `_visible`
   member is created and no script return value is produced by SetMember.

The Rust state transition for the receiver is:

```text
new_flags_6a = (old_flags_6a & !0x4000) | (requested_visible ? 0x4000 : 0)
new_flags_b8 = old_flags_b8
new_local_visible = requested_visible
```

Apply the `u32` mask while preserving all other target bits, including the
upper owner-offset byte and event gates. Load/source flags stay unchanged.
All focus/tab bits and their independent write provenance stay unchanged.
Record visibility provenance separately. Do not update descendants' raw local
visibility or status words. The selected native branch does not do that.

The concrete DisplayState projection updates the receiver's local Boolean and
recomputes the existing subtree's effective visibility with its current parent
state, preserving every descendant's local Boolean and all immutable source
IDs. Current matrices and other branches remain unchanged. The host's subtree
AND rule is its existing renderer-facing model, not newly proved native
hit-testing, input suppression or full render-tree traversal behavior.

## Render bookkeeping boundary

The scene node is an existing **logical** render association; it is not a claim
that native target `+0x48` was already allocated. The Rust renderer need not use
the native allocator, reference counts, 4-KiB page table, backing-pointer layout
or intrusive change records. These are internal representations outside this
binding's exposed VM/display state. Their omission is justified only by this
explicit abstraction boundary, not by declaring the native code a no-op.

The retained evidence does prove a resulting node visibility state and change
notification dependency. The Rust host must publish the new effective state
and invalidate/version its render snapshot before a later frame consumes it.
An already allocated logical node plus a refreshed snapshot can implement that
projection without manufacturing native node pointers or allocator outcomes.
The true-without-native-node case does not authorize allocating a new Scene
node; the admitted host association already exists independently.

Native change-mask counts, allocation counts, exact backing equality and dirty
queue order are not exposed by this profile. A Rust revision/trace increment
on a repeated assignment is local accounting, not a claim that native mask 4
was emitted again. An implementation must not announce native redraw completion
merely because the state changed. Native dirty-list consumers, reference-node
relationships, allocator failure behavior, hit tests and later scheduling remain
outside the proved projection. APIs requiring them must stay unsupported until
their contracts are implemented. This is not full native-setter or visual parity.

The false native path assumes usable allocation output; it has no checked
allocation-error result. Rust budget/admission errors are deliberate boundaries
before mutation, not an invented retail allocation-failure behavior.

## Proposed VM/host interface and commit order

The current SetMember opcode (`vm.rs`, `0x4f`) resolves the receiver and calls
`write_member` with a Host. Its AttachedOwner arm immediately calls `set_key`,
whose attached write route no longer has a Host. Intercept the exact supported
visibility member before this handoff. Keep public hostless
`write_attached_member` rejecting visibility, unless a separately explicit
host-aware API is introduced. Calling it must not create fake host success.

A dedicated typed request is preferable to making a generic indexed request
implicitly authenticate an attached target. A minimal proposal is:

```rust
// Proposal only: no implementation or default implied.
struct AttachedVisibilityRequest {
    root: ObjectId,
    context: ObjectId,       // active caller Environment
    receiver: AttachedClipId,
    owner: ObjectId,
    target: DisplayTargetId,
    expected_local: bool,   // from the VM's current +0x6a bit 14
    visible: bool,
    origin: Option<CodeOrigin>,
}

// Proposed addition to Host. Existing hosts reject by default.
fn prepare_attached_visibility(
    &mut self,
    _request: AttachedVisibilityRequest,
) -> std::result::Result<Box<dyn FnOnce() + '_>, String> {
    Err("attached visibility host is unbound".into())
}
```

The host resolves its own node from this receiver tuple; the caller does not
supply an arbitrary NodeId. Source/parent identity established at binding must
remain immutable. Retain the checked DisplayTargetId and owner alongside the
host's clip/node binding when enabling the bridge: today's map alone stores
only clip-to-node and object-to-node associations. Do not claim it already
checks the new request's target identity. `origin.source_offset` records the actual operation reached;
the script's source movie is never substituted for the receiver policy owner.
The existing `FrontendHost` is the composition point for this request.

**Proposed ordered transaction:**

1. VM preflight checks the receiver/name/value/root/context, current raw words
   and explicit policy, including equal-status admission. Retain the exact
   before/after words; do not allocate script storage or invoke a descriptor.
2. Host preflight checks root/receiver/owner/target/node identity and local
   coherence, and stages the complete subtree result. Admit all limits,
   revision overflow, trace capacity and finite transform checks now. No
   display geometry or raw visibility changes during preparation.
3. With no callback, observer or mutation interleaving, commit the VM's local
   bit-14 update first. This step is infallible after preflight.
4. Publish the already-prepared host local/effective state and invalidation
   infallibly. Commit the matching provenance/trace and return handled.

The proposed method performs steps 2 and its fallible allocations before
returning a `FnOnce` that owns the prepared results and borrows the concrete
host mutably. The VM performs step 3, then consumes that closure for step 4.
The closure has no fallible return and receives no VM or script callback.
It only publishes admitted host data; normal execution must not panic.
An ordinary allocation failure remains outside successful completion rather
than becoming a fictional native error return.

This preserves the native local-bit-before-node dependency. The borrowed
closure prevents host mutation, a second preparation or token replay between
phases; it cannot outlive the host. The VM must not invoke scripts, mutate
policy or yield between obtaining it and committing. Default Host behavior
is an explicit error before constructing a closure. A public arbitrary
callback is not a substitute for the concrete audited DisplayState adapter.
No global lock, native pointer layout or lifecycle scheduler is required.

`DisplayState::update_subtree` already computes finite results and checks its
recompute budget/revision before changing geometry, but exposes no separate
prepare/commit operation. Its existing public write also reserves trace work
and recomputes, so it cannot simply be called as an allegedly infallible second
phase. Factor or wrap the concrete host path to satisfy the transaction above.
No raw-bit mutation may be left behind by an ordinary host admission error.
Diagnostic attempted-work counters may advance on rejection, but no successful
write trace or changed VM/display state may be published.

A host-first write followed by the VM bit update reverses the native order.
It could only be defended as an explicitly indivisible, non-reentrant
observational abstraction; this proposal does not need that shortcut.
General Host callbacks, user script reentry and future getters must not observe
half-committed state. No new native callback or artificial SetMember is needed.

## Coherence and unsupported routes

The current `bind_attached` validates source, sprite/header, owner and parent;
it does not compare local visibility with raw bit 14. Add the coherence check
when enabling this binding and repeat it for every prepared write. The
previous controlled raw value `0x52000180` has bit 14 clear and cannot silently
coexist with a visible local host node under this contract.

Once enabled, these bypasses require coordinated state updates or rejection:

- Indexed Visible SetProperty currently changes only DisplayState. Forwarding
  it unchanged for a bound attached target could make the named getter stale.
  Its complete native indexed-route semantics are not proved by this document.
- Public `update_attached_event_state` can replace bit 14 independently.
  Preserve visibility there or route an explicit coherent snapshot update.
- Direct host `display_mut`/`write_property` calls can change local visibility.
  Do not let them bypass the active binding's transaction.
- Rebinding roots/nodes, adopting unknown status words, changing native movie
  policy through unsupported globals, or resuming after an unmodeled native
  mutation invalidates the profile. Reject or require a new explicit snapshot.

A stale or missing snapshot must produce an explicit boundary. Do not repair
it from panel names, desired screenshots, constructor intent or class defaults.
Do not seed every legal/modal child as hidden before its saved SetMember runs.
The four original source sites retained in ATTACHED_VISIBILITY.md are useful
future trace expectations, not an instruction schedule or evidence that earlier
constructor prerequisites have completed.

## Focused implementation checks and provenance

No VM, constructor, renderer or GPU execution was run for this document.
**VERIFIED compile-only check:** a private original Rust API/lifetime sketch
using the proposed borrowed `Box<dyn FnOnce()>` Host return compiles as library
metadata with rustc 1.98.1. It confirms the borrow/commit API shape, not the
product integration, native behavior, transaction implementation or any test.
No sketch executable was produced or run. A
later implementation should check exact Bool false/true; both equal-status
policy cases; changed-status rejection; missing/mismatched identity/state;
hidden-parent/local-visible distinction; repeated assignments; preservation of
all nonvisibility raw bits, focus/tab state and descendant locals; and every
admitted failure before commit. Verify that the actual original SetMember
produces one coherent VM/host transition without ordinary property creation.
Reaching it in a controlled probe remains distinct from retail startup parity.

Private source hashes, selected cached native Evidence and helper findings are
under `evidence/attached-visibility-binding/`:

- `host-contract/receipt.json`, SHA-256
  `e0ff462656269998656b68eb3f7d366fd0b61ee043b819c5948c3493cabead35`,
  seals the pinned source/API and failure-boundary review.
- `render-node-contract/receipt.json`, SHA-256
  `c1e870b9d28feb392f36f532078ce29679c7fb96452200c176b457fbbcf681bb`,
  seals the normal-completion projection and retained base/wrapper evidence.
- `render-node-contract/status-inert-followup/receipt.json`, SHA-256
  `eafd564fd31b3bcf79648269322cb694925fa5ebddf51fb29296393fd17d38bb`,
  seals the finite existing-node path proofs. Case A checks 54 selected
  instructions/190 bytes; case B checks 113/413; the wrapper continuation
  checks 27/80. These selections overlap and are not a disjoint total.
- `main-source-profile.json` and `api-shape-receipt.json` record source hashes
  and the separate compile-only check. No synthetic check is native evidence.

The native source remains the
owner's proprietary pinned executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The existing exact native recordings were sufficient; no new target or native
process was opened. The project-authored VM/host/display code and the prior
visibility, focus/tab and display-state documents supplied integration context.
Installed REA skill/native-guide instructions govern handling of the retained
evidence; their licence was not independently inspected. No SDK, external
implementation, web source, configuration or account data was consulted, and
no proprietary bytes enter this repository document.
