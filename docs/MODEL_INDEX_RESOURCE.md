# Explicit Model component index resources

`upk::model_index_resource::ModelIndexResource::construct` implements the bounded
integer phase of the owned `0x140bf5a30` writer for explicitly selected saved
Engine.ModelComponent records and one Engine.Model. The caller supplies resolved
object equality keys and the initial logical map/buffers. The result retains
ordered generated indices, per-element metadata, source records and the complete
existing Model and component parser results.

This is a conditional construction from explicit inputs. It does not resolve
materials, infer runtime map state, invoke callbacks, run native initialization,
enable a renderer or establish that the native path executes for the selected
map. Generated indices are not claimed to be a saved component index array.

## Input contract

`Request` contains one zero-based `ExportInput` for the Model and an ordered
slice of component exports, all interpreted in the single supplied `Package`
namespace. The existing readers enforce ordinary UE3 **868/3**, exact imported
Engine classes, prefix/property/native framing, reference bounds and the
ModelComponent local Engine.Level outer profile. Each component must link the
selected Model by its positive one-based export reference. The builder does not
discover components or impose the observed FrontEnd node partition. Callers can
use the separately checked [Level/component binding](MODEL_COMPONENT_BINDING.md)
to choose an authoritative Level list before constructing a request.

The following inputs have no inferred defaults:

| Input | Required meaning |
| --- | --- |
| `phase` | `AfterElementCallbacks`, a stable snapshot at the integer phase; `BeforeElementCallbacks` returns `Unsupported` |
| `references` | Unique signed package references explicitly associated with opaque `ObjectKey::Resolved(u64)` or `ObjectKey::Null` equality identities |
| `initial_buffers` | Explicit ordered buffer contents; an empty slice is an explicit empty-state assumption |
| `initial_map` | Unique keys pointing at valid buffer ordinals; different keys may share the same buffer |
| `components` | Ordered integer-phase invocations; duplicate selections deliberately append again |

An opaque token is not a native address and is never dereferenced. The caller
asserts the equality relationship; the library does not prove resolution or
material class. Different raw references may share a key, and a nonzero raw
reference may explicitly resolve to `Null`. Raw reference zero must use `Null`.
`Resolved(0)` remains an opaque non-null identity distinct from `Null`. All
supplied references must be in range, including unused bindings; every selected
element's key reference needs a binding. Extra valid bindings are retained.

The snapshot is after the native writer's earlier element `+0x30` virtual
callbacks. The API composes stable integer phases, with no intervening mutations
or callbacks. It does not claim that a single function-entry snapshot would
survive those callbacks, or that a particular native lifecycle traverses the
selected components in the caller's order. Missing keys create new empty
buffers; existing keys preserve prior contents and count. Initial aliases point
to a single mutable logical buffer, so appending through either key affects
subsequent appends through both. Unreferenced initial buffers are preserved too.

Initial buffer order and supplied map-entry order survive. New entries and
buffers appear in first encounter order. This is deterministic inspection
ordering, not native hash/sparse-slot order or GPU draw order.
`initial_map_entries` identifies the retained initial map prefix, and each
buffer's `initial_count` identifies its retained initial index prefix.

## Ordered construction and retained data

For each component element, in caller component and saved element order, the
builder looks up its native `+8` key, chooses/creates a buffer and snapshots the
current buffer length as `first_index`. It initializes `triangle_count=0`,
`min_vertex=0xffffffff`, and `max_vertex=0`. It then visits every saved u16 node
index occurrence in order, preserving duplicates.

Each visit retains the complete 64-byte Node, its span, the complete selected
60-byte serialized c8 record and raw references. Node signed `+0x14` selects the
c8 record. The first c8 word represents native `+8` flags. Bit 26 skips the
node. Otherwise bit `0x100` permits two passes; without it there is one.
Pass `s` is accepted when the Node byte at `+0x34+(1-s)` equals the component
word `+0x240`, or that word is `0xffffffff`. The result retains rejected pass
decisions and the buffer position before each pass as well as accepted passes.

For an accepted visit, `N` is Node byte `+0x36`, and `B` is Node u32 `+0x18`.
The native source helper's proved successful stable contract returns one list
`L=[0,1,...,N-1]`. For `d=2..N-1`, each accepted pass appends:

```text
B + s*N, B + s*N + d, B + s*N + d - 1
```

These are packed 36-byte resource **occurrence indices** in Model `+0x19c`.
They are not source pool or unique position indices. Duplicate positions are
not deduplicated, and no area/degeneracy filter is added. Each triple increments
the element triangle count, updates the minimum with its first value and the
maximum with its second value. For `N<=2` no indices or endpoint updates occur.
An element with no emitted triples retains the original integer seeds, while
the native floating bounds could still have changed outside this API's scope.

The native accepted path reads Model `+0x80` pool and `+0xb0` positions. The
builder checks and retains these source corners once per accepted visit,
including raw pool words and exact IEEE32 position bits. It does not recompute
packed vertices or floating bounds. A zero count leaves the unused pool start
unvalidated; a skipped node or entirely rejected visit does not dereference its
source pool/positions. c8 and Node indices remain checked because the native
writer consumes them before those gates. Both accepted passes reuse the same
retained source corners as an inspection accounting policy.

`components` retains original borrowed bytes and each full component result,
including original element metadata, ordered node arrays, raw fields, properties
and opaque suffixes. `resource` retains the complete Model and packed-record
view. `elements` associates every constructed result with its component
ordinal/export, element ordinal/span, raw key link, supplied identity, buffer
ordinal and ordered visits/passes. Original source state is not overwritten.
Repeated invocations therefore retain separate results even though a native
element's metadata storage would eventually hold only its latest write.

## Bounds and errors

Default inspection limits are 32 MiB aggregate serialized Model/component
bytes (counting repeated component inputs), 4,096 component invocations,
65,536 elements, 262,144 node visits, 262,144 accepted source corners,
1,048,576 total index values including initial buffers, and 65,536 each of
buffers, explicit reference bindings and map entries. Existing reader limits
also apply. Source corners are charged once per accepted visit, not per pass.
Index values count unique logical buffers, so map aliases do not multiply the
initial charge. Limits are checked before corresponding output growth; the
aggregate byte bound is checked before parsing component objects.

Exceeded inspection limits and unsupported snapshot/class/version/layout
profiles return `Unsupported`. Missing or duplicate reference bindings,
duplicate initial map keys, invalid buffer targets, wrong selected Model links,
invalid source indices, index bounds and arithmetic errors return `InvalidData`.
Every initial and emitted index must be within the selected packed resource.
Buffer counts must fit the native signed 32-bit domain. Unused Node bases are
not validated when no triple is emitted.

The native arithmetic wraps at 32 bits. This loader deliberately admits only
nonwrapping checked sums, with all output spans/counts checked, and rejects
overflow. It does not emulate native invalid-memory, allocation-failure,
concurrent-mutation or exception behavior. There is no native map allocation,
capacity shrinking, sparse-slot replacement or floating-point emulation here.

## Exact native evidence

**VERIFIED (static):** prior complete owned-body evidence binds the
Engine.ModelComponent `+0xb0` vtable slot to `0x140c0b910`. Its null-global
branch invokes `0x140bf5a30`; the tested global is `0x142c35830`. The full
2,309-byte writer supplies the element buffer/first-index/triangle/endpoints
at native `+0x48/+0x50/+0x54/+0x58/+0x5c`. The separate normal consumer
`0x140bfabc0` copies them to its output record before unresolved `0x140ceb180`.
The saved ModelComponent serializer does not store these runtime fields.

The prior source helper/copy proofs establish the returned ordered list and
count under successful stable ordinary input conditions. The raw resource and
consumer context is in [MODEL_RENDER_RESOURCE.md](MODEL_RENDER_RESOURCE.md).
This module uses the specific `bf5a30` construction, not the separate Level
rebuild branch or dynamic `bf3380` behavior.

Fresh REA and independent byte checks cover seven complete map helpers totaling
**1,875 bytes**: `140a85690` (277), `140bf2e90` (345), `140b432b0` (278),
`140cc46f0` (161), `141103390` (252), `140bed630` (88), and `140cd2380` (474).
The lookup hashes low 32 bits but compares full 64-bit pointer identity,
permits null keys and returns a sparse slot through the caller's output slot.
The typed writer reuses a nonnull existing buffer on a hit. On a stable miss,
it inserts a newly allocated empty buffer directly. Direct duplicate insertion
has different virtual/copy behavior and is outside this admitted typed path.
Sparse free-slot reuse and bucket relinking do not guarantee encounter order.

The owned PE SHA256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Private evidence root is
`~/.local/share/marvel-heroes-forever-rust/evidence/model-index-resource/`.
Its `native-map/receipt.json` SHA256 is
`a37a6ef928a900827b8d102e4c6240a7183c7e5866a4130a4900b1f77ba9d0b7`;
`native-map/MANIFEST.json` is
`105e9a4a13550021dc66b5310ce63b89b475b59bbd43f05ab57ab91cea6eb36b`.
The two successful untruncated REA records are retained in `rea-bundle.json`,
SHA256 `c6c00cef2e4e9e89fad515fbc46555daf9ea30a2de86f0b0fa5f99a59a72838c`.
The joined `native-receipt.json`, SHA256
`f4a5454d1725d8a2f4f15d6c82ff416ba07a1a3c246bc3becd036459558d60a1`,
links the unchanged writer/source dependency receipts and the fresh map seal.

## Owned records and explicit fixture expectations

**OBSERVED:** selected Entry Model 2 has no packed records or selected
components. FrontEnd Model 13 has 48 packed records and twelve Nodes with
`B=0,4,...,44`, `N=4`, c8 flags `0x00000e00` and bit `0x100` clear. The ten
selected Level components 14..23 preserve their saved element/node ordering,
raw import references and selector words. Component 18 visits nodes `[4,7,8]`;
no contiguous-node or one-to-one component partition is imposed by the API.
Package SHA256 values are Entry
`2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33`
and FrontEnd
`9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2`.

Two independent original calculations agree on the following explicitly
**hypothetical stable snapshots**, using unchanged saved fields after callbacks:

| Fixture | Explicit identity/state inputs | Conditional result |
| --- | --- | --- |
| EntryEmpty | No components, bindings, buffers or map entries | No output buffers or elements |
| Front A | Raw `-33`→`Resolved(33)`, `-32`→`Resolved(32)`, initially empty map/buffers | Two buffers of 36 indices each; 24 constructed triangles and ten element results |
| Front B | Same keys, initial buffer `[47,46,45]` mapped to `Resolved(33)` | Buffer lengths 39/36; seed retained and corresponding first indices shifted by three |

All 147 final indices, 20 element results and 24 ordered node occurrences
across the two Front fixtures agree, including component 18's six triangles
and integer endpoints 16/35. These values are conditional construction
expectations, not observed runtime buffer state or material identity. The
independent comparison receipt is
`independent/expected-cases/cross-check/receipt.json`, SHA256
`4824895d2533a1e45780d34a684f973e9a12fe1a06b933d3b1703fd31f37d6cf`.

**VERIFIED (focused product checks):** four new module tests passed in a frozen
private composite, followed by one offline low-priority two-job build and the
three explicit retained-capture fixture runs. The tests cover existing-buffer
aliases, repeated components/nodes, two-pass filters, null keys, zero/one/two
corners, unused bases, rejected source access, missing/duplicate/out-of-range
bindings and map targets, selected-Model/class errors, source/index bounds,
overflow and all nine inspection limits. No baseline or full suite was run.
The actual-capture harness also checked complete parser-result equality,
original borrowed backing, reference order and retained initial prefixes.
The frozen module source SHA256 is
`7a6430fddbc81af2d50887e545cf18fc7b7263b45b5678bdb6625f6ea9ca0b64`.

**VERIFIED (complete output comparison):** all **289** captured public-output
rows match the sealed independent integer and retained-data expectations:
147 final index values, 20 element results, 24 ordered node visits and passes,
96 source corners, 48 c8 reference fields, 312 surface words, 576 pool words
and 288 position words. Map/reference order, usage, complete retained parser
results, initial map/index prefixes and borrowed byte identity are preserved.
The comparison is `independent/product-check/comparison.json`, SHA256
`fdf4f1c88a72cd8f4f766b4d2acdfb7eb3e5f3e71502fa80ca7f2de665e9255c`.
It records the unchanged exact module and baseline
`4fc12bedc19d5da85f25d490087f93b32f029927`. These are Rust construction checks
against conditional fixtures; no native runtime or rendering was executed.

## Provenance and remaining frontier

All new implementation and fixture/oracle code is original. The only outside
content used is the owner-owned proprietary 1.52.0.1700 PE and the two pinned
packages, plus the already registered MIT REA skill/guides as tooling
instructions. No external implementation source was used or copied. Exact
paths, licenses/ownership, uses, source identities and reused original project
modules are retained in private `source-ledger.json` and the child ledgers;
the shared source registrations are in [PROVENANCE.md](PROVENANCE.md).

Remaining native boundaries include callback effects, actual object resolution
and aliases, lifecycle/global branch values, generic map/allocator failure and
virtual behavior, packed-record regeneration, floating bounds, backend
declaration translation, shaders/materials and downstream draw submission.
The saved captures and synthetic failures do not establish native execution,
screen winding, visible rendering or retail gameplay parity.
