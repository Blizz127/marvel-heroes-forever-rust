# Native option owner and mesh-source identity

**VERIFIED native stores and argument edges:** the original wrapper publishes the
option owner at `Q+0x10`; its draw forwarder supplies that pointer to fresh
root-cache construction. The root constructor stores it at `Croot+0x28`.
Selected shape-cache creation copies its supplied cache receiver's current
`+0x28` into the new node's `+0x28`. Both previously recovered mesh getters
then pass that exact field as the ordinary mesh's owner argument.

**INFERRED conditional composition:** those edges carry the same A to the
ordinary mesh when the listed current-state and callback-preservation
conditions hold. These are actual constructor, field-store and argument edges,
not an equality
inferred from common offsets. The selected root/child dispatch, constructor
carry and existing-cache limitations below remain part of the result. No live
node, current setting, default option or saved Scene is assigned this state.
The separate conditional option-value preservation result remains in
[SCALE9_OPTIONS_CONSTRUCTION.md](SCALE9_OPTIONS_CONSTRUCTION.md).

## Keep the native identities separate

| Name | Role |
| --- | --- |
| E | Enclosing engine object containing the renderer and wrapper fields |
| R | Engine renderer constructed by `141202f50` |
| Q | Wrapper constructed by `031c20`, published at `E+0xf4` |
| A | Option owner constructed by `071960`; `Q+0x10` holds its returned pointer |
| P | A's parent, stored at `A+0x50`; selected from current `E+0xec` during wrapper construction |
| D | Render-root node supplied to the draw forwarder |
| Croot | Fresh root cache published at `D+0x18` |
| C | Cache receiver supplied to shape-cache creation |
| N | New shape-cache node whose mesh getters read `N+0x28` |
| M | Newly allocated ordinary mesh context |

This document does not equate R with A. The distinct cache at `R+0x314` and
its upload/draw associations in [SCALE9_DRAW_MESH_IDENTITY.md](SCALE9_DRAW_MESH_IDENTITY.md)
do not create that equality either. A's parent P can be associated with R
only with the separate engine-field/header carry conditions already recorded.

## Wrapper construction and the draw argument

The earlier complete `031c20` allocates A, calls `071960(A,P)`, and publishes
the returned pointer at `Q+0x10`; allocation failure stores NULL. Engine
initialization `141273a20` supplies P from its current `E+0xec` and publishes
Q at `E+0xf4`. Pointer publication does not itself prove all option values
survive the constructor's backend-active branch.

Complete nine-byte nonvirtual forwarder `031b90` replaces its first argument
with `[Q+0x10]` and tail-transfers to `0714f0`. It does not change the other
argument registers. Thus its supplied render-root D accompanies the exact
current owner pointer from Q. The nearby wrapper table's destructor slot
does not make adjacent unrelated tables extra Q methods.

The selected engine draw caller obtains Q from its own enclosing argument's
current `+0xf4`, then passes a root obtained from its selected item to
`031b90`. Its item/filter guards and intervening root-access operation remain
explicit. Equating that enclosing instance with the initializer's E requires
the actual caller association; equal offsets alone do not supply it. Even
for the same E, later replacement of its wrapper field remains possible.

## Fresh root-cache owner store

Complete `0714f0` has two chained unwind fragments. On its selected branch
with actual `D+0x18 == NULL` and both selected viewport dimensions nonzero,
it allocates a 0x1f0-byte root
cache and calls `070ed0(Croot,A,[A+0x50],flags,D)`. The constructor forwards
A as the third argument of base `0ecca0(Croot,D,A,flags)`.

The viewport source is the current payload's `+0xa0` block when its two
leading dimensions are nonzero, otherwise A's `+0xe28` block. The later
admission tests are nonzero tests, not signed-positive tests. A false result
from the separate rectangle helper does not by itself prevent this allocation.
This investigation supplies neither viewport values nor a missing default.

Complete call-free `0ecca0` writes that incoming owner at `Croot+0x28`.
The root constructor's remaining heap metadata query and fixed stores are
recorded with ordinary valid heap/ABI, nonaliasing and no-reentry conditions.
On that successful construction path, `0714f0` publishes Croot at `D+0x18`
before the subsequent root callback.

This establishes the initial association with the same A loaded by the
forwarder. A preexisting nonnull `D+0x18` takes the reuse branch and does
not perform this rebinding. It must not be treated as a freshly constructed
root or assumed to belong to the current Q merely because drawing supplies Q.

## Selected root-to-shape propagation

The constructed root table `142500360` has `+0x10 -> 0ee410`. This callback first
calls `0ed9a0`, then enumerates child
render nodes and passes the same Croot as the cache argument to the selected
child data-record's virtual `+0x30`. The concrete shape-data table
`1424fea30` selects `0b0450`; its actual installer is retained. That method
retains its incoming cache argument
and supplies that pointer as the receiver of `0af6e0` on the selected
single-source branch. This is the native argument connection; it is not a
substitution of a saved Scene parent or a global renderer.

The count-not-one branch instead creates a compound cache K through `0af210`,
passing current `[C+0x28]` as the owner argument. Its call-free base `0ecca0`
stores that value at `K+0x28`; the derived constructor has no further calls
and installs table `142506340`. That table's exact `+0x10` slot is `0b0740`.
When that update is actually invoked, its selected empty-list, positive-count
loop passes the same K to `0af6e0` for each source. This supplies a concrete
compound ancestry edge, but the reached attachment helper `0ecd80` has not
been shown to invoke that update on every count-not-one path. Invocation and
current owner preservation remain explicit conditions.

At the selected successful allocation in complete `0af6e0`, the constructor
argument is loaded from current `[C+0x28]`. It is passed to `0ed4a0` as R9,
and that base constructor stores R9 verbatim at `N+0x28`. The final shape-node
table is then installed. The complete source table binds virtual `+0x20`
and `+0x70` to the two actual mesh-producing methods, `0b1220` and `0b1570`.

The source-owner sample is deliberate: if the current C belongs to the
fresh-root lineage and retains its owner, new N receives that same A. Reusing
a different C, mutating its owner before the sample, or a later N replacement
is not collapsed into the fresh-root case. Temporary key/resource callbacks,
postpublication attachment and later traversal are preserved in the private
frontier rather than described as universally inert operations. In particular,
the source's virtual `+0x48` supplies the count; `0ed4a0` writes `N+0x28`
before key-retain callbacks; later temporary/key release callbacks and
`0ecd80` attachment are separate effects. Concrete descriptor wrappers
`13ad40` and `13ad60` forward nonnull payloads to native retain/release,
but a named release is not proof that destruction is skipped. Alternative
descriptor and optional-object interfaces remain unbound by this study.

## Node to ordinary mesh

Complete `0b1220` and `0b1570` both load their receiver's current `+0x28`
as RDX for `0b0f10`. Their matrix inputs are separate. The latter saves this
incoming pointer and forwards it to ordinary constructor `03bf20` on the
selected new-context branch. Base constructor `03bda0` stores it directly at
`M+0x50`.

That store precedes a nonnull provider's virtual `+0x08` retain. Retaining
only the store instruction would therefore be an incomplete return-carry
argument. The concrete provider interface tables `142507100` and `142509cb0`
have exact slot `+0x08 -> 0c2490`. The complete nine-byte adjustor subtracts 0x10 from
the interface pointer and tail-transfers to complete eleven-byte `031ce0`.
The leaf atomically increments the four-byte counter at interface-minus-eight
and returns its previous value; it has no callback, release or destructor.

For either actual selected interface, valid separate provider storage and
ordinary live, nonoverflowing reference-count state, this finite retention
path cannot write `M+0x50`. The rest of `03bda0` and derived `03bf20` does
not replace that field. This closes the constructor's carry for that concrete
retention profile; it does not identify an unobserved provider instance or
permit arbitrary implementations of virtual `+0x08`.

Existing mesh-cache reuse and the alternate `03d450` context remain separate.
The ordinary sampler's later read from `[M+0x50]+0xcd8` does not establish
preservation of those option values between owner construction and sampling.

## Useful consequences and remaining boundaries

The dataflow supports an original typed association: Q owns a separate A;
fresh root-cache construction records A; selected new shape nodes receive
their supplied cache receiver's owner; the ordinary mesh constructor records
the node's sampled owner. A native snapshot or future scheduler must retain
these source identities and distinguish creation from reuse.

It does not authorize a universal default owner, an A/R cast, field assignment
to already populated Scene nodes, or bypassing a failed association check.
The remaining obligations include actual branch/type selection, successful
allocations and valid object lifetimes, ordinary heap/key/reference effects,
owner carry through reached attachment/traversal callbacks, reused-root/cache
identity, and later option mutation. No render, VM or native execution is
reported by this static investigation.

## Evidence and provenance

Private evidence is under `evidence/scale9-options-mesh-identity/`.
`receipt.json` and `MANIFEST.json` bind this document, the original validators,
the reused sources and the delegated native ledgers. Raw direct-call scans
are discovery aids; only selected aligned instructions and exact table cells
support the argument edges. Chained unwind fragments and adjacent padding
are distinguished from logical function coverage.

Main checks cover 89 instruction rows / 382 code bytes and 16 provider-table
bytes. The wrapper ledger checks 480 rows / 2,056 code bytes plus eight table
bytes; the receiver ledger checks 337 rows / 1,218 code bytes plus 104 table
bytes. These are selected/reused coverage counts, not a sum of unique new
executable coverage. The retained-source audit adds 29 focused checks.

The five new records are preserved in `main.bundle.json`,
`wrapper-flow/final.bundle.json` and `receiver-producer/final.bundle.json`:

- `ev_f67e80bb03a724f2e0e4bf58a8896a1ea2a0433bda2922db5c48de7136a4bdb3`
- `ev_1eaeb01103b2b5efec8c22023e6b558854deb122855cc62921ef3d56a04b0aa5`
- `ev_3d4daaaacc5ddcd721ca10fdd9caba43ffe2e84345f5c537282d1f9b6a597641`
- `ev_28f1ac3ee65cad177ae55bc47cddfb5d09edbf4c7f3793993a94ce1fad2474b0`
- `ev_70d7bafe4dc846443c4da60f18c21cd8a7d0486715767137241327ed7d155738`

All completed with exit zero, untruncated output and unchanged pinned hashes.
The ledgers were exported and reset with no retained target. These statuses
describe static tooling, not successful execution of the represented game paths.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary private installation | Exact native constructors, arguments, tables, reference operation and publication edges |
| Retained `scale9-options-construction`, `scale9-output`, `scale9-output-receiver`, `scale9-rendering`, `scale9-position-reconstruction`, `scale9-draw-mesh-identity`, `scale9-primary-source-format` and `attached-target-initialization` records | Original project investigation of that owner artifact | Reuse original construction, caller, provider-table and getter evidence with explicit old proof boundaries |
| New `wrapper-flow`, `receiver-producer` and `retained-lineage` evidence | Original project investigation; inputs remain private | Root construction, selected shape propagation and prior-evidence inventory |
| Installed REA skill/native/recovery guides, REA 6.9.0, Rizin 0.9.1 and local objdump | MIT / LGPL-3.0 / GPL-3.0 tooling respectively | Bounded target-free static evidence and byte inspection |
| Python standard library and original project PE mapper | PSF tooling / original project code | Independent exact-byte, chained-extent, identity and hash checks |

No extracted native listing, proprietary table dump, asset or external runtime
implementation enters git. Constructor pointer identity is separate from the
conditional option-value profile and from any claim of retail rendering parity.
