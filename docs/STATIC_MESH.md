# Observed UE3 StaticMesh geometry

`src/static_mesh.rs` reads original serialized StaticMesh geometry for the
observed package version **868**, licensee version **3**, mesh internal version
**18**. It supplies data for later scene assembly; it does not render a scene,
resolve materials, implement collision, or establish retail visual parity.

## API and retained data

`StaticMesh::from_export(package, export_index, bytes)` checks the package profile,
class, declared export size and object references. It reads the established
NetIndex/property prefix using the existing tagged-property reader, then decodes
the native mesh bytes. `parse_native(bytes, logical_offset)` is the lower-level
entry point for an explicitly selected 868/3 native payload; callers must supply
its logical package offset, not the physical offset in a compressed file.

The result preserves:

- Origin, axis-aligned box extents and sphere radius; BodySetup reference.
- Quantization bounds, six-byte collision-node records and four-u16 collision
  records, without assigning collision behavior to them.
- The optional preceding LOD and the ordered LOD array, kept distinct.
- Float3 positions in their original coordinates, two packed tangent/normal
  values per vertex, raw half/full-precision UV components for every channel,
  optional BGRA8 vertex colors, and all three u16 index buffers.
- Per-section material reference, first index, triangle count, vertex range,
  section number, unknown words and auxiliary pairs.
- Unknown seven-byte records and two words preceding the LOD array; inline
  source/editor bulk bytes when supported; the entire opaque post-LOD tail.
- Tagged properties and NetIndex when parsing through `from_export`, plus exact
  native-relative byte spans for LODs, positions, attributes, colors and indices.

`VertexAttributes::uv(vertex, channel)` exposes float UVs while `uv_bits` retains
the original encoding. Binary16 conversion preserves signed zero and subnormal
values. `PackedNormal::xyz()` and `w()` decode biased bytes with
`byte / 127.5 - 1`. No normal renormalization, basis conversion, color transfer,
triangle reordering, coordinate transform or material fallback is applied.
The third index stream is retained separately; its observed size is not treated
as proof of a particular rendering role.

## Format evidence

**OBSERVED in reference source and owner data:** The native prefix consists of
28-byte bounds, an i32 BodySetup reference, two float3 collision-bound vectors,
bulk arrays of six-byte nodes and eight-byte triangle records, internal version,
an optional-LOD flag/payload, an array of seven-byte records, two i32 words, and
the LOD array. A bulk array is an i32 element stride followed by an i32 count and
the exact element bytes.

A LOD begins with a 16-byte source bulk-data header and sections. A section has
nine i32 words, a counted array of i32 pairs, and a PS3-data flag. The supported
PC path has flag zero. Position metadata precedes a float3 bulk array. Attribute
metadata declares UV channels, stride, vertex count and precision; each element
contains eight packed-normal bytes plus four bytes per half UV pair or eight per
float UV pair. Color metadata has no bulk array when its count is zero. The LOD
vertex count and three u16 bulk index arrays follow.

**OBSERVED:** The MIT reference reader explicitly discards bytes after the LOD
array. Every selected owner mesh has exactly **96 trailing bytes**, which differ
between exports. This reader preserves them rather than claiming to understand
their layout. The four u16 collision fields, section unknown words, auxiliary
pairs/records and some pre-LOD words likewise retain their raw identities.

## Bounds and unsupported variants

Native/export input is capped at 64 MiB. Limits additionally cap an LOD array at
32 entries, each position stream at 1,000,000 vertices, each index stream at
12,000,000 entries, a LOD at 4,096 sections, collision/auxiliary records at
4,000,000, and UV channels at four. Counts must fit the remaining physical bytes
before allocation; arithmetic and allocation reservations are checked. These
limits bound data-reader work and allocations, not total process RSS.

The reader rejects other package/internal versions, unestablished strides,
noncanonical presence/precision flags, PS3 payloads, unsupported source bulk flags
and nonempty external source bulk. Source bulk flag zero supports inline data;
the reference's unused flag `0x20` consumes no payload. Compressed source/editor
bulk is not decoded. Geometry streams must agree on vertex counts. All floats
used as bounds, positions or UVs must be finite, extents/radius nonnegative, all
indices in range, and every nonempty section's triangle indices within its
declared vertex range. Triangle and wireframe stream lengths must match their
respective topology widths.

Unknown trailing bytes are explicitly permitted and retained. Success means
the established geometry prefix was read, not that the whole native object has
been semantically decoded. Material resolution, component instances/transforms,
lightmaps, mesh collision and the trailer remain separate work.

## Private owner sample and validation

**OBSERVED:** The owner inventory identifies
`UnrealEngine3/MarvelGame/CookedPCConsole/MarvelVista_AsgardHub.upk`, 8,475,579 bytes,
SHA-256 `2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.
The previously pinned frontend copy was independently rehashed against that
inventory and copied to `pins/mesh-samples/MarvelVista_AsgardHub.upk`.
No in-progress full-transfer candidate was read or modified. The receipt is
`evidence/static-mesh-pin.json`.

**OBSERVED (original Rust reader):** All **19** StaticMesh exports in that package
parse through the class/property/native entry point. They contain 19 LODs,
**94,385 vertices**, **73,737 triangles**, **36 sections**, zero wireframe indices
and **884,844** third-stream indices. Sixteen meshes have two half-UV channels and
three have one. Only `msh_asgard_waterfall_top` has colors: 594 BGRA values for its
594 vertices. The smallest mesh, `background_waterplane_02`, contains 26 vertices
and 25 triangles. Every sampled source bulk header is empty with flags zero;
optional preceding LODs are absent.

**VERIFIED (synthetic):** Nine focused tests cover distinct geometry component
preservation, half and full UV encodings, optional/multiple LOD separation, every
truncated geometry prefix, excessive/negative counts, unsupported profiles and
flags, malformed section/index ranges, stream disagreement, nonfinite positions
and UVs, inline versus external source bulk, and half/normal conversion extrema.
The fixtures are newly constructed bytes, not game assets. Full-precision UVs,
multiple/preceding LODs and nonempty source bulk are source-established branches
tested synthetically; they were not present in the selected owner sample.

**OBSERVED (independent scanner):** A separate original Python structural parser
also reads all 19 native payloads and identifies their stream boundaries, counts
and section ranges. Its manifest SHA-256 is
`537089f9b78ec91f38238117f6d2d279b78d02ed8176d45b83c69f803d619a84`.

**VERIFIED (independent byte comparison):** All **171 decoded buffers** across the
19 meshes match the independently identified source regions: **6,353,074 bytes**,
zero differences. These include reconstructed positions, packed attributes,
colors, all index streams, sections and opaque tails. All **374,550 half UV
components**, independently decoded with Python's IEEE binary16 support, match
the Rust float32 bytes exactly. All 19 native-source hashes still match the
independent manifest. The comparison receipt SHA-256 is
`482bdaf42b257dd0623a0954d95fa9a29ce45a18f07b7493f46bc569dfe7e206`.

Private evidence paths, relative to `~/.local/share/marvel-heroes-forever-rust/`:

| Evidence | Purpose |
| --- | --- |
| `evidence/alienware-distribution-inventory.json` | Original owner file identity. |
| `evidence/static-mesh-pin.json` | Independent local pin identity. |
| `evidence/static-mesh-study/exports.tsv` | Export names, logical offsets and property boundaries. |
| `evidence/static-mesh-study/extract.rs` | Existing original UPK reader extraction harness. |
| `evidence/static-mesh-study/independent_layout_scan.py` | Separate structural scan. |
| `evidence/static-mesh-study/independent-layout.json` | Independent source-byte regions and counts. |
| `evidence/static-mesh-study/rust_geometry_check.rs` | Original Rust geometry read and value dump. |
| `evidence/static-mesh-study/rust-geometry-check.tsv` | Rust geometry counts and spans. |
| `evidence/static-mesh-study/compare_rust_decoded.py` | Independent source-region and IEEE UV comparison. |
| `evidence/static-mesh-study/rust-decoded-comparison.json` | Exact comparison results and hashes. |
| `evidence/static-mesh-study/tests.txt` | Focused test result. |

These are local source/byte-reader observations, not REA output or running-client
evidence. The project integrator owns the mandatory REA session and separate
retail binary evidence. No renderer or game server was exercised by this task.

## Sources and licences

All external code studied for this increment is **MIT** UEViewer, copyright
Konstantin Nosov, pinned commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981`, read 2026-11-15. The Rust implementation
and its tests are original; no reference implementation is copied or linked.
The following table is the complete format-source ledger for this increment,
including files reused from earlier private reference snapshots.

| Source | Use |
| --- | --- |
| [UEViewer `UnMesh3.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh3.cpp) | StaticMesh, LOD, section, position/UV/color/index stream and kDOP wire layouts; version branches and explicit trailer discard. |
| [UEViewer `UnMesh3.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh3.h) | Type declarations and mesh representation context. |
| [UEViewer `UnMeshTypes.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMeshTypes.h) | Packed normal byte order and biased component conversion. |
| [UEViewer `UnMesh.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh.h) | Float2 and half2 UV serialization. |
| [UEViewer `UnCore.h`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCore.h) | Bounds, BGRA and core bulk declarations. |
| [UEViewer `UnCoreSerialize.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp) | Bulk stride/count, data header and inline/unused skip rules. |
| [UEViewer `LICENSE.txt`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt) | MIT licence identity. |
| Owner's private `MarvelVista_AsgardHub.upk`, paths/hash above | Proprietary game data; bounded compatibility reads and independent validation only. |

Downloaded/reference identities and exact source hashes are retained in
`evidence/static-mesh-reference/sources.json` and
`evidence/static-mesh-reference/mesh-evidence-additional-sources.json`.
The existing core snapshots are under `evidence/upk-reference/`; newly pinned
mesh snapshots are under `evidence/static-mesh-reference/`.
