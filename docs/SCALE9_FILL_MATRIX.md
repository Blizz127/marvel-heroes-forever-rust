# Native selected-fill matrix arithmetic

`gfx::native_scale9_fill::prepare_finite_ieee_nearest` implements the retained
paint-matrix route from native `0x1400c3121` onward. It accepts the actual
current fill matrix and explicit grid/image branches. It neither selects a
provider nor loads a texture. Any earlier morph interpolation is already part
of the supplied fill matrix; this API does not implement that interpolation.

The input grid record keeps the native source bounds, grid rectangle, builder
matrix and shape-to-grid matrix separate. Its selected-fill bounds come from
matching provider paths, not automatically from the saved DefineShape bounds.
The image input is the actual matrix returned by the selected image's virtual
`+0xb0`. Saved dimensions alone do not supply it. No renderer switch or runtime
input default is introduced.

## Recovered sequence

**VERIFIED static contract:** retained `0x1400c2bb0` initializes its result W to
identity. Empty matching-path bounds or either nonpositive extent return that
identity. Positive bounds produce TL, TR and BR, construct the working Scale9
record with an identity third matrix, transform those three corners and build
one affine from their original positions to the transformed positions. This is
a whole-fill mapping, distinct from the nine piecewise geometry mappings.

The Rust API uses explicit `None` for known empty selected bounds and an ordered
`RectF32` for known bounds. Degenerate bounds skip the grid builder. Arbitrary
inverted/nonfinite inputs are outside this bounded public representation.

The caller then inverts W into an identity-initialized destination I and computes
F times I using its inlined scalar sequence. With rows `[a,c,unused,tx]` and
`[b,d,unused,ty]`, every multiply and add below rounds separately to binary32:

```text
a  = F.a*I.a + I.b*F.c
b  = F.b*I.a + I.b*F.d
c  = I.d*F.c + I.c*F.a
d  = I.d*F.d + I.c*F.b
tx = (I.ty*F.c + I.tx*F.a) + F.tx
ty = (I.ty*F.d + I.tx*F.b) + F.ty
```

Unused lanes become positive zero without reading their previous contents.
There is no extra addition of zero to the four linear coefficients. This
distinguishes the inlined path from the general composition helper for signed
zero and exceptional intermediates. The existing finite arithmetic admission
rejects nonfinite and subnormal-sensitive operations; it does not infer the
native floating-point environment or loaded CRT behavior.

If an image is present, the separate `0x140032970` helper then left-composes its
returned matrix Q, yielding Q times the preceding result. It performs its own
multiply/add/mask sequence even when Q is identity. The public result preserves
the bounds affine, its inverse and the matrix before image composition so their
different roles remain inspectable.

An absent grid is different from a present grid with empty/degenerate bounds:
the latter still inverts and composes identity, including unused-lane writes.
An absent image is likewise different from a present image returning identity.
Neither branch is optimized away by mathematical cancellation.

## Validation scope

Five focused checks pass: absent-branch bit preservation; empty/degenerate
bounds bypassing an invalid grid while retaining the inline composition;
noncommuting image-left composition; positive bounds rejecting an invalid grid;
and the signed-zero difference from generic composition. The private receipt is
`evidence/scale9-fill-matrix/focused/receipt.json`. These are arithmetic/API
checks, not native execution or a live provider/image association.

The independent audit verifies five complete selected routines, 553 instructions
and 2,496 instruction bytes plus 44 constant bytes against the private PE. Ten
synthetic matrix witnesses were frozen before inspecting the new code. Root's
debug and optimized comparisons both match them. Three additional positive-bound
cases run the full public API from supplied grid records through image
composition, including inner and grid-corner bounds. The independent optimized
comparison matches all 40 matrix records / 320 words across those thirteen cases.
Sources and expected results remain unchanged. Private receipts:

- `evidence/scale9-fill-matrix/oracle-comparison/receipt.json`, SHA-256
  `25feae171b03801574f665d2758b9c970bd757d6c3032b33ee21969489551479`.
- `evidence/scale9-fill-matrix/independent/product-comparison/receipt.json`,
  SHA-256 `045864c8e5a4a23cafa4f370df21d87a7b067d19f0075af3601cf3233dee5c2d`.
- Independent 19-file manifest `independent/final-manifest.json`, SHA-256
  `288ca1c4cf815b67db1323cd65f01ca06a37ee97e9f9f6b2a171bd2e3e02831d`.

These cases establish the conditional arithmetic implementation, not native
runtime field selection, image contents, output reconstruction or pixel parity.

## Sources and provenance

All native evidence comes from the owner's private build `1.52.0.1700`,
`MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Its proprietary instructions remain in private REA results; Rust code is original.
The retained records are under `evidence/scale9-rendering/`:

| Record | Evidence ID | Use |
| --- | --- | --- |
| `rea-grid-constructor-and-mesh.json` | `ev_41541b731925019e7936453c3c357a61e4cab53cca8dd47000dfa55e4ad6ad08` | Complete selected-bounds route, identity third matrix, three-point affine |
| `rea-fill-matrix.json` | `ev_92580983e3680646ac72fe2cb956f9ad636740ff173cad53ed13c651318d5fd3` | Current fill copy, inline composition and image-matrix call order |
| `rea-mesh-adapters-and-matrix-helpers.json` | `ev_2606a5cd4f58f08e63076faf650fa6b57c08b665233ffe0f07209b3cf5b1be5c` | Inverse and separate left-composition arithmetic |
| `rea-path-geometry-and-fill-bounds.json` | `ev_83b9fa38f50a20412d699b85d875d6323028f8563798a33f798fbc7f5fb1f7a2` | Explicit fill-ID selection and matching-path bounds |

[SCALE9_RENDERING.md](SCALE9_RENDERING.md) and original
`src/gfx/native_scale9.rs` supply the already checked constructor, point,
three-point affine and inverse primitives. Tool provenance is the existing
REA/Rizin static analysis, Rust compiler/standard library and Python standard
library. No new dependency or third-party implementation source was used.
Actual provider binding, image/resource lifecycle, generated geometry, output
receiver reconstruction and final UV/pixel agreement remain separate work.
