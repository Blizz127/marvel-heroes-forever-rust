# Bounded primary output composition

`gfx::native_primary_output` connects the existing normalization, generated
vertex export, explicit-color record writer and triangle ordinal transfer.
`PreparedOutput` owns one immutable generated-record namespace. A descriptor
borrows that owner, so later chunks cannot silently switch to another array or
normalize a selected subset. This is an original Rust integration of proved
primitives, not an implementation of the whole native output receiver.

## Ordering and ownership

`PreparedOutput::prepare` checks the 65,535-record engineering cap before any
allocation, then applies the existing inverse-outer and normalization helper
to **every** supplied generated position. In particular, unused mesh `0xffff`
records participate in extrema. Only positions change: record order, u32 local
ordinals, both styles, flags and mesh IDs are retained. Borrowed caller input
does not change. All heap reservations are fallible and bounded.

The owner exposes immutable normalized records and arithmetic metadata,
including inverse outer, forward normalization, inverse normalization, seeded
extrema and branch flags. Its private ordinal array comes from the same original
records. `triangle_indices` uses that array and explicit global generated-record
triangle indices; it does not infer mesh membership, descriptor counts or
connectivity. The native transfer reads ordinal low words, even for records that
the caller would normally omit from a particular descriptor.

`begin_descriptor` applies the native seven-word copy/reset primitive.
`resume_descriptor` accepts an explicit snapshot, preserving all words. The
caller still owns descriptor selection and snapshot provenance. Both operations
borrow the immutable source. `preview` copies the next selected batch without
committing cursor progress, allowing the caller to supply resolved color words.
`write_batch` repeats that selection against the same source, validates and
packs the batch, then commits the resulting cursor. It returns the number of
copied records, not the number scanned. A successful scan with no matches can
still advance to the end of the source.

Any rejected batch leaves both the entire destination and descriptor unchanged.
Successful stores preserve bytes 10/11 of each twelve-byte record and the tail.
This atomic admission policy is a Rust engineering choice. The native receiver's
begin call can write inverse-normalization before returning failure, and native
record stores proceed progressively. Neither behavior is replaced or claimed
to roll back by this API; no receiver is invoked here.

## Explicit remaining inputs

The arithmetic profile remains finite IEEE binary32 nearest rounding, with
subnormal-sensitive operations rejected and loaded CRT/MXCSR equivalence
unverified. The API cannot prove the caller supplied the whole native array,
an actual current outer matrix or a valid native invocation sequence. Empty or
degenerate sources retain the arithmetic primitive's behavior; enclosing native
mesh/count guards are not inferred.

Each write requires an exact format/profile and one resolved native color word
per selected record. These are caller assertions, not defaults. Native style
cache state persists across chunks **and descriptor changes** in the enclosing
output invocation. This module does not create or reset that cache, resolve
resources, synthesize bitmap colors or establish live format selection. The
writer's adjacent-color consistency check is not a complete cache model.
The explicit profile restrictions in [primary packing](SCALE9_PRIMARY_PACKING.md)
remain unchanged, including the unsupported initial style-zero case.

Output receiver allocation, descriptor scheduling, inverse-N association with
draws, native cache lifetimes, resource upload and shader/pixel behavior remain
separate. The module does not enable the renderer or add a finished upgrade.

## Evidence and focused validation

The underlying static native evidence remains the owned 1.52.0.1700 executable,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and retained REA records registered in [normalization](SCALE9_RENDERING.md),
[export](SCALE9_PRIMARY_VERTEX_EXPORT.md), [packing](SCALE9_PRIMARY_PACKING.md),
[triangle transfer](SCALE9_GEOMETRY.md) and
[receiver](SCALE9_OUTPUT_RECEIVER.md). No external implementation was copied.

**VERIFIED focused Rust checks:** three composition tests pass in a frozen
six-module private harness. They exercise unused extrema before selection,
source/ordinal identity, late packing failure retaining cursor and every byte,
resume versus independent cursor progress, full-width no-match IDs, empty
sources, explicit exhausted snapshots and admission limits. One unchanged
support method emits a dead-code warning because its other consumer is outside
the focused harness. Only the new module's three tests ran; native execution
and production startup are separate.

Private receipt: `evidence/native-primary-output/focused/receipt.json`, SHA-256
`5b453737cf5e24089c81c5a30d731f66ea6fbe20f8c41fa40cb4efaac138632a`.
Frozen implementation SHA-256:
`119b22693cdb5d21321051220ce984b4da0ac20bbcf064f1caea95c0eab23d42`.

**VERIFIED independent comparison:** the fixture was frozen before its author
read the new module. An optimized harness passes 282 checks with zero failures,
comparing eight complete normalized records, all matrix/extrema/status bits,
four descriptor sequences with twelve steps plus twelve resumed steps, four
exact atomic rejections, every destination byte and six triangle requests.
Source records, geometry, color inputs and profiles remain unchanged. The two
unused mesh-`0xffff` records uniquely determine the fixture's full extrema;
selecting a mesh first would fail. The inverse-N translation is `4 + 1 ulp`,
not an assumed exact four, and all eight words are compared exactly.

Oracle SHA-256:
`d95e9e9c073874611052601555ab6167655d4c00320063a63892cceb3d522e89`.
`evidence/native-primary-output/comparison/receipt.json` SHA-256:
`13eeaf8c60d41def383ea85faff0ce03590dc1e196ff7fa2f4241ad73a9e85dd`.
All six frozen source hashes and the oracle remain unchanged. These are
synthetic instruction-derived witnesses, not a native execution or live draw.
The later dev25 production build and startup are recorded in `FINDINGS.md`.
