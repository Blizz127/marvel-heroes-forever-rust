# Skeletal mesh reader

`src/skeletal_mesh.rs` is an original bounded reader for the observed Marvel
Heroes **UE3 868/3 PC GPU-skinned mesh** layout. It exposes saved reference bones,
poses, material references, LOD sections, index streams, chunk palettes, vertex
positions, packed normals, UV bits and four-slot byte weights. It does not run
animation or render or skin the mesh.

## Actual asset and selection boundary

**VERIFIED:** the complete fixed startup pin
`~/.local/share/marvel-heroes-forever-rust/pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelVista_AsgardHub.upk`
is **8,475,579 bytes**, SHA-256
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.
The hash matches the saved owner source inventory, and its stat fields stayed
stable during verification. The transfer destination was not changed.

The selected export is zero-based **1316**, qualified name
`loki_vista.lokiandstaff_vista`. It has 244,533 serialized bytes at logical package
offset 3,685,057. The four-byte UObject prefix contains -1; tagged properties end
at export byte 572, leaving 243,961 native bytes. Its saved SkeletalMeshComponent
export 1318 references this mesh through signed object reference 1317.

This is a real frontend vista asset, not a substitute character. No runtime
CharacterSelect selection binding is asserted. At the recorded selection time,
no inventory-listed `UC__MarvelPlayer_*.upk` package of at most 4 MiB was present
under the candidate copy. The 130,085-byte Spider-Man Classic package was listed
in the inventory but absent locally; it was not opened or fetched. Exact paths,
stat checks and the bounded availability scan are private in
`evidence/skeletal-mesh-study/independent/sample-selection.json` and
`available-candidates.json`.

## Observed format

All native offsets below start at the first native byte, export byte 572.
Serialized arrays use signed 32-bit nonnegative counts. References remain signed
package-local references; positive values are one-based exports, negative values
are imports, and zero is null.

| Native region | Actual observation |
| --- | --- |
| 0–67 | Seven bounds floats, two material references, mesh origin, integer rotation origin, bone count |
| 68–3863 | 73 reference-bone records of 52 bytes |
| 3864–3871 | Saved skeleton depth 14 and one LOD |
| 3872–3901 | Two section records, each 13 bytes after the count |
| 3902–60554 | Index-container header and 28,320 16-bit indices |
| 60555–60886 | 67 active bones and two chunk records |
| 60887–60991 | LOD size/count fields, 73 required bones, empty auxiliary bulk, UV count |
| 60992–243019 | GPU header plus 5,687 vertices of 32 bytes |
| 243020–243036 | Empty extra-influence array and empty adjacency-index container |
| 243037–243960 | **924 opaque trailing bytes**, preserved without interpretation |

A bone record contains an FName pair, flags, quaternion X/Y/Z/W, position,
child count, parent index, and an auxiliary signed word. The reader preserves
the auxiliary word and exact finite float values. The root references itself
at index zero; this sample's other parents precede their children. No quaternion
normalization, coordinate conversion, root rotation adjustment or parent-transform
composition is applied.

Sections retain material index, chunk index, first index, triangle count and
an auxiliary byte. Index containers retain an auxiliary word, one-byte element
width, serialized stride, count and values. Width/stride agreement is required.
Two-byte indices have actual coverage; four-byte indices have focused synthetic
coverage.

Chunks retain first vertex, bone palette, declared rigid/soft vertex counts and
maximum influences. The cooked CPU vertex arrays are empty in the actual sample;
nonempty CPU arrays return Unsupported. Declared rigid/soft counts still describe
the GPU vertices and are not replaced by those empty array lengths.

**OBSERVED PC distinction:** the GPU header saves a packed-position flag of **1**,
but its 32-byte vertices contain full **12-byte float positions**. This agrees
with the studied UEViewer PC serializer branch, which treats that declaration
separately from stored position layout. Each observed vertex contains two packed
four-byte normal/tangent words, four palette indices, four weights, three float
positions, and one pair of binary16 UV values. The reader preserves the declared
flag while requiring the explicit float-position stride. It does not reinterpret
an unknown stride as packed positions.

The actual GPU header has one UV channel, half precision UVs, and stride 32.
The reader also admits the bounded declared full-precision UV layout, preserving
raw bits; no actual full-precision skin stream was exercised here. Packed normals
remain bytes and UV values remain binary16/binary32 bits, without speculative
normal/tangent reconstruction or texture-coordinate conversion.

## Actual structure and independent checks

**VERIFIED (original independent raw-byte walk):**

- 73 bones, one root, consistent saved child counts, and 14 hierarchy levels.
  Every reference-pose float is finite; quaternion squared-norm differences are
  small floating-point rounding, without modifying the saved quaternions.
- 5,687 vertices and 28,320 indices, forming 9,440 triangles in two sections.
  The sections cover the index buffer exactly and stay within their own chunks.
- Chunk 0 covers vertices 0–5111, with 2,640 declared rigid and 2,472 soft vertices,
  a 66-entry palette and maximum four influences. Chunk 1 covers 5112–5686, with
  575 rigid vertices, one palette bone and maximum one influence.
- All actual weights sum to 255. Nonzero-influence counts are 3,215 vertices with
  one, 1,936 with two, 505 with three and 31 with four. The reader preserves byte
  weights and does not normalize or require that sum for otherwise bounded data.
- All weighted palette indices are valid; the two palettes together select the
  67 active bones. The required-bone array contains all 73 bones.
- Native material references 582 and 583 identify local material instances
  `loki_vista.loki_classic_vista_mat` and `loki_vista.loki_classicstaff_mat`.
  They are preserved as references; no material or texture dependency is loaded.

The nearby AnimSet lists 84 bone tracks, including these 73 mesh bone names, but
its saved preview mesh name refers to `loki_traveling.loki_traveling`. Name-set
agreement does **not** establish an animation binding, runtime playback or
compatibility of every track. Animation payloads were not decoded in this task.

## API, bounds and unknowns

```rust,ignore
let bytes = archive.read_export(export_index)?;
let mesh = SkeletalMesh::from_export(archive.package(), export_index, &bytes)?;
let slots = mesh.lods[0].influences(vertex_index)?;
```

`SkeletalMesh` retains the original tagged property list, `has_vertex_colors` as
an optional serialized property, native bounds/origins, bones, material references,
LODs, decoded end offset and copied opaque tail. `Lod::influences` resolves each
nonzero weight slot through the containing chunk's palette into a skeleton index;
zero-weight slots stay None, and slot order/weights remain unchanged.

Spans on bones, sections, chunks, index buffers, skin buffers and LODs are
export-relative. Add `serial_offset` through `Span::logical_offset` to obtain a
logical package offset. Compressed file offsets are a different address space.
Opaque property and auxiliary bulk spans require the caller to retain/reload the
original export bytes; `opaque_tail` owns its preserved trailing bytes.

Limits are 64 MiB per serialized export, 4,096 bones/materials/chunks,
32 LODs, 16,384 sections, two million vertices per LOD, 12 million indices per
stream, four UV channels, four influences and 4,096 bytes per bone name. Counts
must fit remaining bytes before reading their streams. Reference bounds, section
ranges, ordered chunk coverage, palette indices, finite geometry floats, canonical
boolean words and stream stride/count agreement are checked.

Explicitly unsupported profiles include unstripped CPU vertices, nonempty extra
influence arrays, vertex-color streams, packed-position storage, unobserved UObject
preambles, and non-topological reference skeletons. The optional vertex-color
property is absent in the actual file; that observed layout has no color stream.
Absent remains None rather than being reported as a saved false value.

Auxiliary bulk flags support the observed empty zero-flag block, an exact bounded
inline integer-array range, and the documented unused marker. Other external,
separate or compressed auxiliary payloads return Unsupported. Only the empty
zero-flag block was exercised by the actual mesh.

ClothingAssets, Sockets and LODInfo property payloads remain available as tagged
spans. Clothing simulation, morphs, physics, socket evaluation, skeletal animation,
GPU skinning, inverse bind matrices and the 924-byte trailer are not implemented.
There is no renderer or root module edit in this increment. Root integration
requires `pub mod skeletal_mesh;` and a provenance link to this document.

## Validation and private evidence

**VERIFIED:** one actual-file smoke loaded the mesh through the new Rust reader.
All **5,771 emitted rows / 46,356 fields** matched the independent canonical walk,
including all 79 emitted source offsets, all bones/vertices/weights, two sections,
two chunks, headers, bone lists and resolved weighted influences. Five output
streams matched the original bytes exactly: **243,412 bytes**, including the
opaque tail. The comparison does not claim that the opaque bytes are understood.

Three focused original tests passed, covering the PC packed-declaration/float-
position distinction, two/four-byte indices, every truncation of a small LOD,
count limits, extra-influence rejection, invalid indices/palettes, zero influences,
unknown strides and nonfinite positions. Nineteen unrelated tests were filtered
out. No full project suite, rendered pose, animation execution or retail-runtime
comparison was run.

Private directory: `~/.local/share/marvel-heroes-forever-rust/evidence/skeletal-mesh-study/`.
Key records:

- `independent/loki-validation.json`: raw constraints and canonical field checks.
- `independent/loki-canonical.json`: complete private interpreted records.
- `check.rs`, `check.tsv`, `decoded-*.bin`: Rust smoke and reconstructed streams.
- `independent/rust-comparison.json`: exact Rust/Python comparison, SHA-256
  `f588e79b02b32a4a930a659b24a0e66a488966a84daf4ef9f3cc798096fbc19d`.
- `tests.txt`, `source-ledger.json`, `validation.json`: focused checks, exact source
  identities/read scopes, and final receipt.

Section/chunk/vertex offsets were not individually emitted by the smoke TSV;
the 79 compared offsets are the ones actually emitted. All raw assets, extracted
records, geometry buffers and captures stay private and outside git/builds.

## REA status and source register

The REA skill and its connection/recovery guide were read. A read-only
`binary_session` call found no open target/evidence and reported Ghidra unavailable
in that MCP process because its JAVA_HOME did not identify the required JDK.
The exact response is private in `rea-session-status.json`; the lead was informed.
No shared configuration, target binding or other session was changed. No native
serializer address was established, so no speculative native analysis was launched.
The format findings above are ordinary source/owned-byte observations, **not REA
native-decompilation evidence**. Existing project REA work remains separate.

All implementation is original. These permitted UEViewer sources were studied
read-only at commit `a0bfb468d42be831b126632fd8a0ae6b3614f981`; no implementation code
was copied. Cached source SHA-256 values and exact ranges are in the private
source ledger.

| Source | Licence | Use |
| --- | --- | --- |
| [UnMesh3.cpp](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh3.cpp) | MIT | Section, index, chunk, GPU skin and LOD field layouts; PC packed-position distinction; palette mapping; explicit trailer discard |
| [UnMesh3.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh3.h) | MIT | Skeletal mesh/vertex-color property declarations |
| [UnMesh.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnrealMesh/UnMesh.h) | MIT | Reference bone and joint-position serialized fields |
| [UnCore.h](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCore.h) | MIT | Quaternion/bounds order, bulk declarations and unused marker |
| [UnCoreSerialize.cpp](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnCoreSerialize.cpp) | MIT | Auxiliary bulk header, empty/unused and inline-data behavior |
| [LICENSE.txt](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/LICENSE.txt) | MIT, copyright 2022 Konstantin Nosov | Confirm licence identity |
| Owner's verified Asgard pin and inventory | Proprietary game data; private compatibility study | Actual layout, range identity and independent comparisons |
| Original project `src/upk.rs`, `src/upk/objects.rs`, `docs/STATIC_MESH.md`, saved actor/source receipts | Original project source/docs | Existing bounded package/tag interfaces and sample discovery |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and `references/connection-and-recovery.md` | Supplied operational instructions; licence not examined | Tool routing, retained-result handling and observed provider limitation |

Python standard-library validation tools and the Rust standard library were used;
no dependency or external runtime library was added to the client.
