# Saved frontend world scene

`src/world_scene.rs` reads explicitly saved actor/component properties from the
observed UE3 **868/3** frontend packages. This is a file reader, not a scene
runtime: it does not fill defaults, resolve archetype inheritance, combine
transforms, evaluate animation, load dependencies, or render the world.

## Owned inputs and scope

**VERIFIED:** both complete stored files matched the previously saved owner
inventory before this investigation. Paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/`.

| Package | Private pin | Stored bytes | SHA-256 |
| --- | --- | ---: | --- |
| MarvelVista_AsgardHub.upk | `pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelVista_AsgardHub.upk` | 8,475,579 | `2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153` |
| MarvelFrontEndMap.upk | `pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFrontEndMap.upk` | 12,038 | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` |

FrontEndMap was already pinned under `alienware-candidate`; it was read there
without modifying the full-copy destination. No other retail package contents
were opened in this increment. No remote access or new asset copies were needed.
The verification receipt is `evidence/world-scene-study/pin-verification.json`.

## Observed saved layouts

Offsets and indices below are zero-based unless described as signed references.
All spans are in the decompressed export address space, not compressed-file
positions.

**OBSERVED:** all 221 component exports across the two maps have the same opaque
8-byte prefix, hex `00000000ffffffff`, followed by tagged properties. These
comprise 210 Asgard components and 11 FrontEndMap components. The reader preserves
the prefix without assigning meanings to those two words.

**OBSERVED:** 68 ordinary actors have object flags `0x0207000100000000`, a 26-byte
prefix, and a property list ending exactly at export EOF. Their classes are
CameraActor, Emitter, FogVolumeSphericalDensityInfo, LightmassImportanceVolume,
PlayerStart, PostProcessVolume, SkeletalMeshActor and WorldInfo. Prefix bytes vary
and remain opaque. The reader accepts the observed class/flag combinations;
`0x0200000000000000` distinguishes this prefix from the collection-actor profile
in these inputs. This is not a universal UE3 actor-layout claim.

**OBSERVED:** the two collection actors have flags `0x0007000100000000` and a
4-byte prefix containing `ffffffff`:

| Asgard export | Class | Saved reference array | Following native bytes |
| ---: | --- | ---: | ---: |
| 1340 | StaticMeshCollectionActor | 45 StaticMeshComponents | 0 |
| 1320 | StaticLightCollectionActor | 99 LightComponents | 6,336 |

The light tail has the numerical size `99 * 64`; it remains opaque. The reader
does not call these records matrices. The mesh collection has no native tail
from which to obtain additional placements.

## Mesh and material bindings

**OBSERVED:** Asgard contains 47 StaticMeshComponent exports, indices 1341–1387.
The 45 references in the mesh collection identify exactly the 45 components
with directly saved StaticMesh references. The remaining two components belong
to FogVolumeSphericalDensityInfo actors and have no directly saved StaticMesh.
Their imported archetype references remain visible; inheritance is not loaded.

| Direct component field | Number of components |
| --- | ---: |
| Materials | 47 |
| StaticMesh | 45 |
| Translation | 45 |
| Rotation | 30 |
| Scale3D | 31 |
| Scale | 2 |

The 45 mesh references identify 18 different local StaticMesh exports. Their
signed reference values are 1322–1338 inclusive and 1340. Positive references are
one-based, so these are not export indices. The package contains 19 StaticMesh
exports; one does not appear in these direct component bindings.

Materials arrays contain a signed count followed by that many signed 32-bit
object references, with no remaining bytes. All 64 saved slots in these 47
arrays are nonnull local Material or MaterialInstanceConstant references. The
17 distinct signed references are:

`277, 278, 280, 300, 571, 572, 573, 574, 575, 576, 577, 578, 579, 580, 581, 584, 585`.

Slot order is preserved. For example, component export 1343 saves mesh reference
1322 and Materials `[573, 571]`; export 1344 saves mesh reference 1323 and Materials
`[573, 572, 277]`. These values can be supplied to `MaterialGraph::inspect` after
the separate material module is integrated. This reader does not impose a
section-to-slot selection rule or infer a fallback from null native mesh-section
material references. See [STATIC_MESH.md](STATIC_MESH.md) and
[MATERIALS.md](MATERIALS.md) for those separate readers and their limits.

**OBSERVED:** FrontEndMap has no StaticMeshComponent or StaticMeshCollectionActor.
It contains ten ModelComponents and one CylinderComponent. Model/BSP geometry
and native component lighting/collision data remain undecoded here.

## Placement and relationships

Vector structures contain exactly three little-endian 32-bit float words;
Rotator structures contain exactly three signed 32-bit words. FloatProperty
scalars retain their IEEE-754 bits. This module preserves the exact values,
including signed zero and nonfinite encodings, without assuming coordinate
axes, angle units, handedness, scale defaults, or transformation order.

**VERIFIED:** 157 explicitly saved actor placement fields were compared with a
separate actor scan: Asgard has Location65, RelativeLocation1, Rotation45,
DrawScale5 and DrawScale3D39; FrontEndMap has Location1 and RelativeLocation1.
Another 108 explicitly saved static mesh component placement fields were
compared with raw property payloads. Missing fields remain `None`.

FrontEndMap PlayerStart export 24 saves a Base reference of 37, pointing to
WorldInfo export 36, as well as RelativeLocation. Both are exposed independently;
no composed position is synthesized. Outer ownership and archetype references
also remain named references, rather than being treated as parent transforms.

## Original reader API

```rust,ignore
let bytes = archive.read_export(index)?;
let saved = world_scene::SavedSceneObject::from_export(
    archive.package(), index, &bytes,
)?;
```

`SavedSceneObject` exposes:

- `object`, `outer`, `archetype`: checked `ObjectReference` values with the original
  signed index, class name, and separate outer-name path components. Null,
  imports, and local exports remain distinct. No filenames are inferred.
- `placement`: optional `SavedField` values for Location, RelativeLocation,
  Translation, Rotation, Scale, Scale3D, DrawScale and DrawScale3D.
- Optional `static_mesh`, `base`, `materials`, `static_mesh_components`, and
  `light_components`, preserving array order and absent-versus-null values.
- `properties`: every original property tag/value/span, including fields not
  interpreted by this reader. `SavedField::property_index` points back here.
- `prefix` bytes and an opaque `native_payload` span. The caller retains the
  original export bytes for inspecting opaque property/native spans.

`Property::payload.logical_offset(saved.serial_offset)` gives the corresponding
logical package offset. `supported_class` selects only the actor/component
classes whose prefix offsets were observed in these two packages. Other
classes and unobserved actor-prefix flag combinations return Unsupported.

Bounds include the existing 64 MiB export and 16,384-property limits, at most
4,096 references across interpreted arrays, 64 outer links per reference, and
4,096 combined name bytes per path. Array payload width, reference bounds,
outer cycles, duplicate singleton fields, nonzero singleton array indices,
structure types and widths, and source-span arithmetic are checked. No
filesystem paths or dependency loads are derived from package names.

Root integration requires `pub mod world_scene;`; no renderer or root-module
files were edited by this increment.

## Focused verification and evidence

**VERIFIED:** one actual-file smoke parsed all 291 supported objects: 278 Asgard
and 13 FrontEndMap. All 70 actor objects matched the separate actor scan's prefix
and native-tail spans. Exact comparisons covered 157 actor placement fields,
108 static mesh component placement fields, 49 reference arrays, and 46 selected
scalar object references. Collection membership matched the exact 45 components
with direct mesh references. Material target classes and names were checked
without opening other packages.

Three original focused tests passed. They cover absent/null/import distinction,
exact float-bit retention, malformed array sizes, duplicate fields, unsupported
prefix combinations, truncated prefixes, extreme signed references, and cyclic
outer chains. Nineteen unrelated tests in the standalone harness were filtered
out. No complete project suite or live rendering check was run.

Private receipts and reproducible scripts are under
`evidence/world-scene-study/`:

- `components.rs`, `components.tsv`, `component-fields.json`: direct component
  tag and raw-payload inspection.
- `independent/actor-records.json`, `independent/verification.json`: separate
  actor scan, exact prefixes/spans, payload hashes and counts.
- `check.rs`, `check.tsv`, `saved-scene-records.json`: actual new-reader smoke.
- `compare.py`, `validation.json`: exact-value comparison and result counts.
- `tests.txt`, `source-ledger.json`, `pin-verification.json`: focused checks,
  read ledger, and full-input hash verification.

No asset bytes, extracted data or private receipts enter git. No renderer,
client-runtime behavior, spatial interpretation or retail parity is asserted.

## Source read ledger

No external implementation or new third-party format source was read for this
increment. All implementation is original and based on the owner's two package
files and the project's existing original package/property interfaces.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The two private owner package pins above | Proprietary game data; private compatibility study only | Actual property/prefix/range/reference evidence |
| Original project `src/upk.rs`, `src/upk/objects.rs` | Original project source | Checked package/export/tag APIs and existing limits |
| Original project `src/static_mesh.rs`, `docs/STATIC_MESH.md`, `docs/UPK_OBJECTS.md` | Original project source/docs | Existing mesh API and the documented gap in actor/component decoding |
| Original project `Cargo.toml` | Original project metadata | Confirm the standalone reader needs no added dependencies |
| Private original `world-scene-study/independent/*` | Original investigation scripts/results | Independent actor byte interpretation and comparison |
| Python standard library `struct`, `json`, `hashlib`, `ast`, `collections`, `pathlib`, `subprocess`; Rust standard library | Existing language tooling; no copied implementation or added dependency | Private harness, receipts and independent value checks |

Exact read paths and current source hashes are in the private `source-ledger.json`.
Earlier outside-source study underlying the existing UPK implementation remains
in that module's provenance; this change did not copy or re-read those sources.
The integrator should link this document from `docs/PROVENANCE.md`.
