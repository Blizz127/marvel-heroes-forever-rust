# Texture update manager and resource callbacks

**VERIFIED static native operations and dispatch; conditional object state.**
This continues [SCALE9_TEXTURE_RESOURCE.md](SCALE9_TEXTURE_RESOURCE.md) at the
two manager methods which receive the original Texture2D. The incoming call
conditions remain part of the contract: global `0x142aee960` is zero, and
no object in the selected Texture2D's finite outer chain has flags mask 0x600.
The concrete remove/update path `0x1409c66a0` then calls manager virtual+0x70;
the conditional insertion path `0x1409b8b30` tail-calls manager virtual+0x68.
Both select the manager through qword global `0x142c2bf80`.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-texture-updates/`.
This increment contains documentation only. No native client, network,
server, GPU, renderer flag or current-dimension default was run or changed.

## Concrete initialization and dispatch

A targeted RIP-relative instruction-pattern search for this one global found
two direct-store candidates. Complete containing functions were retained for
navigation, then the selected operation blocks were independently bounded.
This search covers its stated MOV/immediate patterns; it is not a proof that
other instruction forms, indirect stores or later replacements do not exist.

On the selected initializer path, block `0x141333c1a..0x141333cc9`:

1. Allocates a 0x288-byte child C, calls `0x140a78020` on allocation success,
   and keeps its return in RBX.
2. Creates two policy objects and passes each to `0x140a7c470(C, policy)`.
3. Allocates a 0x2c-byte manager M and calls `0x140a77fa0` on success.
4. Stores the returned M into global `0x142c2bf80` at `0x141333cb7`.
5. Calls `0x140a7c2c0(M, C)` at `0x141333cc4`.

The complete 115-byte M constructor installs table `0x141f325b0` and clears
its pointer array/count/capacity at +0x10/+0x18/+0x1c. The complete 2,222-byte
C constructor installs table `0x141f32678` and returns that original C.
Its similarly numbered fields belong to C, not to a Texture2D: constructor
writes to C+0x180 must not be mistaken for texture dimension writes.

The policy attachment call is also bounded at 255 bytes. The C constructor's
fresh internal helper at C+0x124 has +0xa4, +0xa8 and +0xb0 zeroed after its
own construction. With those values retained and a valid helper, attachment
skips its wait/callback branches and appends the policy pointer to C's
+0x1b0 array. That path does not change C's installed table. On a nonfresh
helper those callbacks remain separate effects. The complete 119-byte child
append increments M+0x18, resizes its pointer array if required, and stores C
at the old index; it does not replace M's table.

The retained complete initializer contains one direct call to this append
routine. That bounded source observation is not a claim that the current
manager always has one child or that later registration never occurs.
The selected construction argument assumes the block is reached, allocations
are valid and constructors return normally. It does not assert that this
initializer ran in the user's current process.

The second direct-store block, `0x141330ae7..0x141330b04`, conditionally calls
the existing manager's deleting virtual before writing zero to the global.
The preceding R15 zero definition and complete retained caller establish the
stored zero on that path. A global read therefore cannot be treated as an
eternally live manager merely because a constructor exists.

The exact installed slots are:

| Receiver | Slot address | Selected method |
|---|---|---|
| M virtual+0x68 | `0x141f32618` | `0x140a7c340` |
| M virtual+0x70 | `0x141f32620` | `0x140a9a920` |
| C virtual+0x68 | `0x141f326e0` | `0x140a7c3d0` |
| C virtual+0x70 | `0x141f326e8` | `0x140a9a9b0` |

Both M methods are complete at 136 bytes. They walk M's pointer array,
forward unchanged Texture2D O to each child's same virtual slot, and reread
M+0x18 for iteration. They do not snapshot or infer a fixed child count.
The following preservation result selects the proved child C and valid
current tables/list storage; it does not cover arbitrary substituted children.

## The selected child changes indices, not dimensions

The complete 146-byte child +0x68 method appends O to the pointer array at
C+0x12c, using count+0x134 and capacity+0x138. It writes the old array index
to **O+0x1ec** at `0x140a7c451`. Those are its original-object effects; no
object virtual or callback receives O from this method.

The complete 502-byte +0x70 method reads signed index O+0x1ec. It first
checks whether that index is valid in C's pending-pointer array and whether
the indexed pointer equals O. If so, it removes that element, updates a
moved object's +0x1ec when necessary, and sets O+0x1ec=-1.

It then rereads O+0x1ec and checks a second array at C+0xf0, count+0xf8,
whose records have stride 0x9c and begin with an object pointer. On a matching
record, nonzero C+0x1a0 nulls the record's pointer. Otherwise it removes the
record and repairs any moved nonnull object's +0x1ec. The selected object's
index is set to -1 again on exit. These pointer-identity checks are part of
the contract; an arbitrary saved index does not authorize array removal.

Both exact removal helpers are included: `0x14085eab0` (144 bytes, stride8)
and `0x140a9adf0` (155 bytes, stride 0x9c). They copy the selected trailing
element(s) into the removed slot, decrement count and optionally resize the
supplied allocation. They do not invoke element destructors or object
methods. Their pointer arguments refer to the supplied array header/data,
not to the Texture2D dimensions.

The selected copy thunk `0x141c6ed86` jumps through IAT `0x141dbbcd0`.
Independent bounded PE import lookup identifies **MSVCR120.dll!memcpy**.
The descriptor's terminated thunk-table bounds are checked; a nearby DLL
descriptor is not accepted merely because a calculated offset reaches the
same name entry. The current loaded CRT/IAT and successful ordinary
allocation/copy semantics remain conditions, not a native-execution result.

**INFERRED conditional composition:** on valid disjoint manager, array and
Texture2D storage, the concrete child
methods and these array operations preserve **O+0x180..0x18f**. Texture-side
writes are to +0x1ec, including moved objects' indices. This closes the
previous unknown manager-method receiver for the identified child profile.
It does not assert arbitrary manager callbacks are harmless, current child
identity, concurrent list mutation safety, or lifetime-wide preservation.

## Resource callbacks keep counter and deferred paths separate

The earlier poll obtains **R = QWORD[O+0xd8]**, not the address O+0xd8.
Both wrappers reload their Texture2D backpointer from **QWORD[R+0x5c]**.
That identity is not silently assumed unchanged across arbitrary callbacks.

`0x1409a9080`, complete at 235 bytes, exchanges 3 into the selected O+0x1c0.
With global `0x142c27978` zero it calls `0x1409b9090(R)` directly. With the
global nonzero it uses the alternate storage path to construct a 16-byte
wrapper with table `0x141ecdf18` and R at+8, then publishes that storage.
The wrapper's later execution is not established by the direct-call proof.

`0x1409a8f80`, complete at 253 bytes, first performs a locked decrement on
the selected O+0x1c0. It calls `0x140962970`; a nonzero result takes the
direct `0x1409b1760(R)` path. Otherwise the same global selects direct
execution or a stored wrapper with table `0x141ecdf30` and R at+8.
The complete 66-byte predicate does not dereference incoming R: it compares
the current thread ID against a selected thread object's virtual+0x38, with
its null-global branch explicit. Thread selection and current global state
are not inferred from the saved texture.

The complete direct worker `0x1409b9090` (838 bytes) updates resource fields,
processes selected mip work and supplies **&O+0x1c0** to backend calls.
Its normal exits perform a locked decrement on that counter. The complete
finalizer `0x1409b1760` (660 bytes) updates resource handles/state, likewise
decrements the counter on normal return, and passes O to `0x140a9b1f0`.
Neither complete body directly accesses the four dimension fields. Backend
calls, retained backpointers, mip identity and deferred execution remain
explicit effects, rather than a whole-callgraph absence claim.

The direct worker's bounded O helper `0x1413b2820` (263 bytes) checks the
previously recovered Icons predicate. On a matching path it obtains an
outermost-package/path result, looks up a record, and can write value 4 to
its supplied stack-local mode. It does not directly write O's dimensions.
Its name/lookup dependencies retain the earlier successful-resolution gates.

The finalizer's immediate O receiver `0x140a9b1f0` (104 bytes) calls
`0x1409a93b0(O, count)` for each of O+0x1b8 and O+0x1b4, then applies
signed conditional atomic additions to the supplied global accounting
receiver. The complete 147-byte size helper starts at
`signed_max(wrap32(N-selected_count), 0)` with N=O+0x104. It loads each
selected mip pointer from O+0xfc, calls `0x1407450b0(M)`, and adds the
returned EAX into a wrapping 32-bit sum. The loop rereads the current count;
neither stable array state nor unbounded mathematical sum is substituted.

That size helper directly writes neither O nor its dimensions. The remaining
immediate receiver is selected mip M in `0x1407450b0`, whose implementation
is not promoted as verified here. Similarly, backend counter/payload calls
and the deferred wrappers are not replaced with no-op assumptions. This
resource evidence narrows their inputs; it does not yet close the entire
callback/lifetime profile.

## Outcome and evidence scope

The former unknown manager dispatch now reaches exact constructor-installed
methods and the initializer's concrete appended child. The selected child's
texture effects are index bookkeeping, with the array-helper and ordinary
memory contracts stated above. This is a useful conditional preservation
result for the four loaded dimension fields; it is not a claim about the
current runtime manager or all future streaming work.

Resource callbacks are bounded through their immediate Texture2D receivers
and count/size bookkeeping. Deferred wrappers, backend operations, selected
mip callees and current resource/backpointer lifetime remain open. The
earlier queue-membership, class-reorder and version-selected serialization
conditions still apply. No new renderer default or native input adapter is
enabled by this document.

All primary native bytes are from the owner's privately pinned
`MarvelHeroesOmega.exe`, build 1.52.0.1700, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
This is proprietary owner-owned material used for compatibility study; no
game bytes enter git. No outside engine implementation was read or copied.
Original project evidence supplied the prior call sites, field layout,
resource backpointer and `GetCurrentThreadId` PE import binding. The REA
skill supplied procedure only. The exact source ledger and original
navigation/import helpers remain in the private evidence directory.

New canonical REA records:

- `ev_bbe74681570a892d1f788d793da1692a81ed49793704e9b20884d1995e936324`:
  bounded producer/clear blocks, complete manager/child constructors,
  policy attachment and child append.
- `ev_bb69f99772df297c78f13586061ee7639c8b0e920115e9b09042682574459acd`:
  complete manager/child methods, exact array helpers and dispatch slots,
  selected copy thunk and bounded PE import data.
- `ev_95abc9a267194b4d2501dfdc19775c9a89b2e0b093d0a6f6d79377a12c7d93a2`:
  eight complete resource/counter/size callback bodies.

Independent final validation covers exactly **three new records**:
**22 instruction arrays, 1,498 instructions / 6,709 code bytes**, plus
**8 data arrays / 82 bytes**. Every requested range exhausts exactly and
matches the unchanged pinned PE. Captures are complete and untruncated.
The final receipt
`verification/runs/142311f94b6a27561fa4/capture-validation.json` has SHA256
`07bfbb74c96b607df5709c25d2b448c9cded5cacbd5c0537c645437eb7c5623d`.
Import-span byte validation is distinct from the original bounded import
parser's name composition, and neither observes the loaded runtime IAT.

Supporting receipts:

- `manager-producer/candidate-receipt.json`: SHA256
  `5da00eb8aa436a73b590c3f57d03515edfd58ae2840b2b64178a369b6580c4fd`.
- `resource-callbacks/final-index.json`: SHA256
  `6dd709543598f71b79221539c3b790309bc88a2727098f8642715c840e012a2c`.
- `resource-callbacks/selected-mip-frontier-addendum.json`: SHA256
  `91b66ef2eeff938dde7cefe4cdcd85b4acfb5c8f9d5d0c69c214d5aa74696bf0`;
  this explicitly keeps `0x1407450b0` unreviewed and distinguishes valid
  disjoint storage from sufficient proof of arbitrary callee effects.

The cumulative `native.bundle.json` was exported with 83 records and zero
unknown records, then REA was closed. No runtime or pixel-parity check was
performed.
