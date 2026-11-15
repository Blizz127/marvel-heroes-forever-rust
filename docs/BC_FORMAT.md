# BC1, BC2 and BC3 software decoding

Task a3d58c579542, 2026-11-15. This is original dependency-free Rust code for
DXT1/BC1, DXT3/BC2 and DXT5/BC3 mip levels. It does not parse containers, choose
texture semantics, perform gamma conversion or premultiply alpha.

## API and limits

`texture::bc::decode(Format, width, height, input)` returns
`std::io::Result<Vec<u8>>`: row-major RGBA8, top row first. The public format
variants are `Bc1`, `Bc2`, `Bc3`; `Format::block_bytes()` returns 8 or 16.
The caller must select the format from validated asset metadata.

The input must contain exactly the complete 4×4 block grid for the logical mip
size. Partial edge blocks remain fully encoded; pixels outside the logical
dimensions are discarded. Zero dimensions, size arithmetic failures, truncated
or trailing bytes and decoded output larger than 256 MiB fail before allocation.
Output allocation uses a fallible reserve. No external decoder is linked.

## Deterministic pixel behavior

**OBSERVED (Microsoft format documentation):** RGB endpoints occupy 5:6:5 bits.
BC1 endpoint ordering selects either four opaque colors or three colors and a
transparent selector. BC2/BC3 always use four color entries, independent of the
endpoint order. BC2 has explicit four-bit alpha; BC3 has endpoint-interpolated
alpha with either six intermediate values or four plus zero/full opacity.

The software implementation expands endpoints to bytes by repeating high bits.
It follows the Direct3D 9 documentation's nearest-integer examples for thirds,
fifths and sevenths; the BC1 midpoint uses integer division by two. BC1's
transparent entry is zero RGBA. BC2/BC3 output straight alpha with unchanged RGB,
including when alpha is zero. A caller needing premultiplied output must request
or implement that separately; DXT2/DXT4 are not exposed as supported formats.

**Unverified:** Exact hardware decompression, sRGB filtering, blend state and
retail rendered pixels. Deterministic software integer output is not a claim of
byte-identical GPU/retail output. Other BC families and normal-map conventions
are outside this module.

## Focused verification

**VERIFIED:** Six standalone Rust tests cover color selector order, BC1
transparent/equal endpoints, BC2 nibble expansion and four-color behavior, both
BC3 alpha modes and cross-byte selector packing, cropped multi-block images,
exact input lengths, invalid dimensions, output budgets and rounding examples.
The test log is private:
`~/.local/share/marvel-heroes-forever-rust/evidence/bc-validation/unit-tests.txt`.

An independent validation harness wraps synthetic blocks in DDS and compares
the RGBA output to separately installed Pillow 13.0.0. Its complete case and
per-channel comparison receipt remains private under
`evidence/bc-validation/pillow-comparison.json`. No proprietary game bytes are
used in these standalone cases; game-mip integration is a separate check.

**VERIFIED:** All 450 synthetic cases had matching output size and at most one
unit of difference in any RGBA channel against Pillow. Ninety-three cases matched
byte for byte. The corpus covers all three formats at logical sizes 1×1, 2×3,
4×4, 5×5, 8×4 and 7×9. The other cases are not claimed byte-identical; integer
interpolation rounding differs. This comparison establishes bounded numerical
agreement with this software oracle, not retail GPU parity.

## Outside source register

All implementation and test code was written for this project. The complete
outside reading list for this module is:

| Source | Licence/status | Use |
| --- | --- | --- |
| [Microsoft: Block Compression (Direct3D 10)](https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression) | MicrosoftDocs documentation, CC-BY-4.0 | BC family layout, mip block padding, color/alpha interpolation. |
| [Microsoft: Opaque and 1-Bit Alpha Textures (Direct3D 9)](https://learn.microsoft.com/en-us/windows/win32/direct3d9/opaque-and-1-bit-alpha-textures) | MicrosoftDocs documentation, CC-BY-4.0 | BC1 RGB bit fields, selector ordering, endpoint comparison, rounded thirds and transparent color. |
| [Microsoft: Textures with Alpha Channels (Direct3D 9)](https://learn.microsoft.com/en-us/windows/win32/direct3d9/textures-with-alpha-channels) | MicrosoftDocs documentation, CC-BY-4.0 | Explicit and interpolated alpha, forced four-color mode, straight versus premultiplied format distinction. |
| [MicrosoftDocs/win32 licence](https://github.com/MicrosoftDocs/win32/blob/docs/LICENSE) and [raw licence](https://raw.githubusercontent.com/MicrosoftDocs/win32/docs/LICENSE) | CC-BY-4.0 | Verify documentation licence; no licence text or sample code copied. |
| [Microsoft: DDS_HEADER](https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dds-header) | MicrosoftDocs documentation, CC-BY-4.0 | Private independent-oracle fixture header only; not shipped implementation. |
| [Pillow DDS documentation](https://pillow.readthedocs.io/en/stable/handbook/image-file-formats.html#dds), installed Pillow 13.0.0 metadata and its `pillow-13.0.0.dist-info/licenses/LICENSE` | MIT-CMU for Pillow; separately installed tool | Independent DDS decoder in private validation; licence/version check. No Pillow decoder source read or copied, no client dependency. |

Primary pages were read on 2026-11-15. Microsoft documentation is acknowledged
as a format reference, not a source of game behavior or endorsement.
