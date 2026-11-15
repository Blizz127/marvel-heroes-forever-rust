# Explicit frontend world transforms and unresolved saved defaults

`src/world_transform.rs` implements bounded, explicit-input native transform
math and audits the original saved inputs. It does **not** evaluate the 45 Vista
mesh placements yet: effective defaults, runtime collection membership and owner
selection remain unresolved. No absent field becomes zero, identity, false or
null. Camera selection and projection remain separate from this work; see
[CAMERA_MATH.md](CAMERA_MATH.md) and [WORLD_SCENE.md](WORLD_SCENE.md).

**OBSERVED** describes selected owner-file bytes. **VERIFIED** describes bounded
REA/independent byte checks or the stated numerical comparison. **INFERRED**
marks a semantic interpretation not established by native execution. No retail
client, native transform, game session or GPU was executed in this investigation.

## Explicit numeric contracts

All matrix memory is row-major, with row vectors. The original camera module
supplies the recovered 16,384-entry rotator table and raw `RotatorWords`. All
operations below retain binary32 rounding order, sign-bit negation and explicit
zero terms. Round-to-nearest/ties-even is the checked model; runtime MXCSR,
FTZ/DAZ settings and retail execution have not been captured.

### Scaled rotation and translation

**VERIFIED REA:** helper `0x14092d230`, 452 bytes, reads an explicit scale vector,
raw rotator and translation and writes 64 bytes. It evaluates the same grouped
rotation terms as the camera's forward constructor, then scales each first-three
row's first-three values by that row's scale component. Translation is copied
unchanged. The homogeneous column is written as `+0,+0,+0,1`; its zeros are not
multiplied by scale.

This supports:

```text
scaled_matrix(&CameraMath, ExplicitScaledTransform) -> Result<Matrix4, TransformError>
ExplicitScaledTransform { translation, rotation, scale }
```

`scale` is an explicitly supplied effective vector. The function does not combine
saved Scale, Scale3D, DrawScale or DrawScale3D properties or decide their defaults.
Zero, negative and nonuniform finite scales are permitted. Nonfinite inputs and
nonfinite output matrices return typed errors.

### Actor-specific transform

**VERIFIED REA:** actor-context helper `0x1408598c0`, 935 bytes, reads position
words at `+0x80/+0x84/+0x88`, rotator words at `+0x8c/+0x90/+0x94`, uniform scale
at `+0x98`, vector scale at `+0x9c/+0xa0/+0xa4`, and a further vector at
`+0xa8/+0xac/+0xb0`. Named DrawScale/DrawScale3D setter paths corroborate the two
scale field associations. The further vector has a pre-pivot effect in the
algebra. Engine declares Actor.PrePivot, but mapping that saved name to native
`+0xa8` remains **INFERRED**; no named native pivot accessor was recovered.

The real-number interpretation is a pivot adjustment before scale/rotation and
final position. The native arithmetic expands those terms and differs from a
generic matrix product. For example, with `A=uniform*scale.x`, one stored basis
term is `(A*cos(pitch))*sin(yaw)`, while its pivot contribution multiplies the
pivot between those latter operations. Using the rounded stored basis term
changes float words. Zero pivot does not authorize skipping the native terms.

```text
actor_matrix(&CameraMath, ExplicitActorTransform) -> Result<Matrix4, TransformError>
ExplicitActorTransform { position, rotation, uniform_scale, scale3, pivot_input }
```

The implementation retains the complete independently checked grouped schedule
in `evidence/world-transform/actor-oracle/schedule.txt`. `pivot_input` explicitly
means the native `+0xa8` input; it is not an automatically resolved saved field.
Finite inputs that overflow still return `NonFiniteMatrix`.

### Component with an explicit parent

The recovered component matrix path uses runtime Translation at `+0x200`,
Rotation at `+0x20c`, Scale at `+0x218` and Scale3D at `+0x21c/+0x220/+0x224`.
Named setters establish those physical field associations. The matrix helper
`0x140951ec0` combines scalar and vector scale, calls the scaled helper above,
then multiplies the local matrix by its stored parent matrix.

For the supported zero-absolute-flags profile, each output entry is computed as:

```text
(((local[i,0]*parent[0,j] + local[i,1]*parent[1,j])
   + local[i,2]*parent[2,j]) + local[i,3]*parent[3,j])
```

Every product and addition rounds separately, including zero terms and row
three. This differs from the camera view helper's final row-three operand order.

```text
component_matrix(&CameraMath, ExplicitComponentTransform, explicit_parent)
    -> Result<Matrix4, TransformError>
ExplicitComponentTransform {
    translation, rotation, uniform_scale, scale3,
    absolute: AbsoluteFlags { translation, rotation, scale }
}
```

The caller must supply all flags. Only all-false is supported. Any true flag
returns `UnsupportedAbsoluteFlags`; omission never becomes false. Native bits
23/24/25 modify parent translation/rotation/scale. The scale-removal path uses
an RSQRTSS estimate and a refinement step, whose CPU estimate bits have not been
observed. This increment does not substitute an exact reciprocal square root or
silently ignore an absolute flag. The parent must be supplied explicitly and
contain only finite values. No owner is inferred from an export outer.

## Native consuming path and its limits

Selected static class-construction paths and exact vtable reads supply the
following route. Complete bounded REA selections cover the update and matrix
helpers; independent decoder/byte receipts retain the class anchors and exact
call/slot operands:

1. The StaticMeshCollectionActor class-name factory reaches constructor callback
   `0x140e06750`, whose final vtable is `0x14200f178`. Its base leaf at
   `0x140793fd0` writes only a vtable. The collection callback additionally writes
   zero at `+0xb4`; these bodies do not assign position/rotation/scale defaults.
2. Named Actor.ForceUpdateComponents calls virtual slot `+0x258`; the exact
   collection table points to `0x140a222b0`, which calls slot `+0x688`.
   That table entry reaches `0x140a4fad0`. Its complete selected path extends
   across three consecutive unwind ranges; one range alone was incomplete.
3. The latter path constructs the actor matrix once and iterates a runtime
   component pointer array at actor `+0x60`, count `+0x68`. It supplies the actor
   pointer and matrix to `0x140a4f9c0`. A true update argument additionally filters
   runtime flags; an unconditional saved-array replay is not established.
4. Component update stores that actor pointer at component `+0x78` and passes the
   supplied parent matrix through virtual `+0x260`. StaticMeshComponent's class
   factory/callback/constructor selects vtable `0x141e25d10`. Its `+0x260` entry,
   `0x140951650`, copies the 64-byte parent to component `+0x1c0`.
5. The transform-dirty path calls component `+0x270`, whose exact table entry is
   `0x140954810`; that invokes `+0x400`, reaching matrix helper `0x140951ec0`.

This is a static consuming route. It does not prove the live dirty flags, object
receiver, invocation time, or how the saved StaticMeshComponents array becomes
the runtime component array. In particular, outer ownership is not promoted to
the runtime `+0x78` owner or to a parent transform. Normal transforms, determinant
consumers, culling/winding, lighting and GPU upload conventions are outside the
implemented matrix subset.

## Actual saved Vista inputs

**VERIFIED original data:** Asgard export 1340, object reference 1341, contains
45 StaticMeshComponents references identifying exports 1343–1387. Their saved
array order is preserved and is not ascending export order. The two other static
mesh components in the package belong to fog-volume actors and are outside this
collection scope. The package SHA256 is
`2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153`.

| Direct saved placement slot | Present / 45 | Still absent |
| --- | ---: | ---: |
| Translation | 45 | 0 |
| Rotation | 30 | 15 |
| Scale | 0 | 45 |
| Scale3D | 31 | 14 |

There are 106 directly saved values and 74 missing values among those slots.
All 135 AbsoluteTranslation/AbsoluteRotation/AbsoluteScale slots, all 45 Owner
tags and all 45 LocalToWorld tags are absent. The collection itself saves none
of Location, Rotation, DrawScale or DrawScale3D and has archetype zero. Absence
of a native tail in that collection is not proof of an identity transform.

Every one of these components explicitly imports
`engine.default__staticmeshactor.staticmeshcomponent0`. Exact package/path/class
resolution identifies Engine export 24529, object 24530. That object has
archetype zero and saves none of the requested placement, absolute, owner or
matrix fields. The explicit instance-archetype route therefore fills none of
the missing slots. The template's outer actor is a separate context; its actor
scale values cannot be assigned to the actual collection or to component Scale.

The source audit is deliberately narrow:

```text
audit_saved_transform(&SavedSceneObject) -> SavedTransformAudit
```

It borrows the complete original object, preserving field property indices,
tag/payload spans, raw bits, outer/archetype references and opaque native payload.
It lists absent placement fields already interpreted by `world_scene` and
retains an actor-base/pivot/update or component-owner/absolute/update obligation.
Additional native inputs such as the actor pivot are not synthesized. An empty
missing-field list would still not establish the update/parent policy. There is
no automatic saved-object-to-effective-transform conversion.

## Class defaults are evidenced separately from their application

**VERIFIED selected byte coverage; INFERRED serialized-field semantics:** an
independent bounded UClass profile covers all 9,327 bytes of nine Engine/Core
classes. It reaches explicit default-object references at predicted boundaries,
without searching for a `Default__` name. The profile is corroborated by
[UELib's MIT UClass serializer at commit 8236c39](https://github.com/EliotVU/Unreal-Library/blob/8236c39b79e30887a487ecf9b018204e430a5ebc/src/Core/Classes/UClass.cs).
All nine targets have matching classes and superclass-aligned archetype edges;
their tagged bodies cover 74 saved properties exactly.

| Linked default object | Explicit saved value |
| --- | --- |
| Engine.PrimitiveComponent default, export 1686 | Scale `1`; Scale3D `(1,1,1)` |
| Engine.Actor default, export 1039 | DrawScale `1`; DrawScale3D `(1,1,1)` |

Translation, Rotation, Absolute flags, Owner, LocalToWorld and actor Location
remain absent from all nine default lists. These saved unit scales do not prove
the loader's initialization, native construction, default-copy ordering,
archetype merge or subsequent updates for the Vista instances. The module does
not turn them into effective defaults. The exact unresolved dependency is native
initialization/default application, including how absent fields acquire their
values, followed by runtime owner/collection binding.

## Verification and private evidence

All game-derived bytes and generated oracle data remain under
`~/.local/share/marvel-heroes-forever-rust/evidence/world-transform/`.
The immutable `implementation-receipt.json` SHA256 is
`8b793bd2a0c4b447ae6091d94962bfd3f7983c3ec5730fce11892b6fd5c2a521`.
The parent lane's four-record canonical REA bundle SHA256 is
`3f45c71f8f4ed31f95a4fd2b777f461c0f76a8a04b497f8b0d1c20c8b42d4552`;
it was exported/checkpointed with no retained target. Separate composition
receipts preserve the helper's isolated CLI results.

| Evidence | Scope |
| --- | --- |
| `native/scaled-matrix-candidates.json` | REA scaled helper; unrelated axis-angle candidate retained separately as discovery only. |
| `native/actor-matrix-rea.json` and `actor-matrix-source.json` | Complete actor helper, exact original-file/span hash linkage. |
| `scaled-oracle/`, `actor-oracle/` | Independent original instruction interpreters and frozen grouped schedules; no product source read. |
| `composition/`, `component-class/` | Named entrypoints, class/vtable identities, update route and component matrix branch. |
| `defaults/receipt.json` | 46 Asgard objects/440 tags plus bounded Engine/Core archetype evidence. |
| `defaults/class-default-link/receipt.json` | Nine complete class/default profiles, exact target tags and external-source provenance. |
| `scaled-rust-check.json`, `actor-component-rust-check.json`, `saved-audit-check.json` | Changed-feature Rust checks and actual saved-data audit. |

The optimized original Rust harness matches 32 scaled matrices, 32 actor matrices
and 12 component/parent matrices: **76 matrices, 1,216 float words, no mismatch**.
The three independent frozen instruction oracles did not read the Rust source.
Nineteen rejection checks cover nonfinite inputs/parents, computed overflow and
each unsupported absolute flag. The actual-source audit matches the collection
and all 45 components, including exact array order and all per-field missing
lists. The harness compiled with rustc 1.98.1, optimization level 2, at nice 19.
No routine full suite or complete application build was run in this lane; root
integration requires `pub mod world_transform;`. These checks establish static
transcription consistency, not retail execution or world rendering.

REA uses exact owner-file paths or private raw spans whose original-file offset,
preferred VA, length and SHA256 are recorded and checked. Raw-span analysis was
used under measured CPU pressure to avoid repeated full-PE loading. Complete
selected instruction arrays recovered from one timed-out full-PE request are
kept distinct from the request's incomplete completion status; later bounded
captures and independent byte checks provide the required selected coverage.
No native process was launched. GNU objdump 2.46 (GPLv3-or-later) supplied an
independent static decoder; its output is not confused with REA Evidence.

The private validation attempts also retain two corrected harness assumptions:
one check started before a compiler session had completed, and another initially
expected ascending component indices. Final checks wait for the completed
binary and compare the actual saved array order; no source values or ordering
were changed to satisfy a check.

## Outside-source provenance

Implementation is original. The owner's pinned retail EXE, Asgard, Engine and
Core packages are proprietary compatibility inputs; their bytes are never
included in git or builds. Existing original `camera`, `world_scene`, UPK readers
and private instruction interpreters supply the shared table/property APIs and
checked analysis machinery.

The class-default helper studied the MIT-licensed UELib sources at pinned commit
`8236c39b79e30887a487ecf9b018204e430a5ebc`: UClass, UState, UStruct, UField,
UObject, UExportTableItem, DefaultEngineBranch, UnrealFlags and legacy-version
definitions. These establish only a corroborated serialization profile. No
UELib code/parser was executed or copied into the product. Exact URLs, source
hashes, licence, discovery reads and usage are retained in
`defaults/class-default-link/sources.json`. REA skill/native/evidence guidance
and GNU objdump are tooling sources. The integrator owns the corresponding
`docs/PROVENANCE.md` index entry.

The unresolved defaults and runtime binding prevent a claim that these matrices
already place the saved Vista meshes correctly. Active camera, animation,
materials, normal transforms and GPU/retail parity remain separate work.
