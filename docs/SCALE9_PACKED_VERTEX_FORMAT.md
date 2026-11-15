# Packed position conversion and declaration association

**OBSERVED:** the selected native upload converter treats source element code
`0x132` as two signed 16-bit coordinates when converting to `0x162`. Each
coordinate becomes its exact binary32 integer value, without normalization.
The original bounded helper in `src/gfx/native_vertex_format.rs` implements this
element transfer, the raw `0x132` copy, and the separate `0x133` zero-extension
route. It does not select a live destination format or enable rendering.

This closes the conditional interpretation of the packed coordinate words left
open by [SCALE9_OUTPUT_RECEIVER.md](SCALE9_OUTPUT_RECEIVER.md) and
[SCALE9_POSITION_RECONSTRUCTION.md](SCALE9_POSITION_RECONSTRUCTION.md). The exact
mesh/draw-record identity, live format choice, backend declaration implementation
and shader-to-pixel interpretation remain separate dependencies.

## Evidence and provenance

All native evidence is static study of the owner's private x64 1.52.0.1700 client,
SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It is proprietary shipped material used for compatibility study, with no source
reuse or redistribution. The implementation is original Rust. No external SDK
source or inferred UE/D3D enumeration names are used. Private results live under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-packed-vertex-format/`.
Addresses below omit preferred base `0x140000000` unless fully written.

| New file under `native/` | REA Evidence ID | Scope |
| --- | --- | --- |
| `rea-format-shader-setup.json` | `ev_542107053a427b49bd3092db2e34a7e2993b456d1cfb146f806f54492417ea4d` | Destination format to cached declaration/shader binding; converter table pointers |
| `rea-position-conversion-callbacks.json` | `ev_00950338c1ecf8c5bf7c00cd24c61c9336cabd912cca89405a9a7041e079cd1f` | Both position callbacks, raw copy helper, position table and sizes |
| `rea-gpu-format-constructor.json` | `ev_a8df9abd116663b4a78879d6a8e8d888e01848b97bf4df4943930245847397bf` | Backend format object construction and declaration persistence |
| `rea-declaration-mapper-and-format-binding.json` | `ev_29813eaeb86b0cf894dd47ee6c28c9fc21c521f5c500f80c3690194246838053` | Element mapping, declaration dispatch and destination-format producer |
| `rea-inline-declaration-tables.json` | `ev_73e339ef837c881f040fba74c35fc9434e6ffdf59ed896f08ee43d77b4abd4c4` | Exact inline jump-table bytes, distinguished from executable instructions |

The five commands completed with exit 0, no truncation and unchanged target
hash. Their bundle is `native/final.bundle.json`, SHA256
`57fbff94554eecbff731fe57603797cb6cd8fa9708d154198d94096f51f28115`.
The investigation was exported and reset with no retained target. The code
captures use raw mapping base `0x140000c00`; `.data` byte queries therefore use
the preferred address minus `0xc00`. The inline intervals
`1411a51b0..1411a5220` and `141254944..14125496c` are data. Their initial `pDj`
rendering includes meaningless/invalid instruction rows; those rows are not
interpreted as code. The separate `pxj` capture retains the complete tables.
Independent PE mapping checked 1,038 code/alignment rows (3,933 distinct bytes)
and 328 dumped data bytes, including both inline tables: 4,261 distinct bytes
in total. There are zero invalid executable rows and zero instruction/data
boundary crossings. All 73 invalid-labeled decoder rows belong to the two
explicit data intervals. Audit receipt `verification/receipt.json`, SHA256
`6fd27a378d83d0821bab53505da1ff75606dffe74d8885b5620b4966f188ef4a`,
also verifies command metadata and the unchanged executable hash.

Earlier canonical uploads, descriptor bytes and converter dispatch are reused
without recapture. `reuse/reuse-ledger.json` indexes seven existing records and
sixteen exact spans, SHA256
`ebf0d8c6132bb403c4d889cb4eeca451eea853a7214e9070cfc6efe4fbffb612`.
Navigation files are candidate-finding aids; the claims here use the canonical
REA records and the separately retained byte checks.

## Source element and dispatch

The retained packed descriptor at `142adb240` has stride 12 and element list
`142adb168`: `(offset, code)` pairs `(0,132)`, `(4,251)`, `(8,1211)`,
`(9,2211)`, followed by the zero terminator. Earlier writer evidence stores
16-bit X/Y at offsets 0/2, a dword at 4 and bytes at 8/9. This says nothing
about padding bytes 10/11 or whether a particular live upload selects that
descriptor.

Upload `1255030` obtains source format F from `[[B+40]+20]+30` and destination
format G from `[B+30]`, then calls `032c90`. That converter iterates destination
elements, matches source semantics through `code & 0xff00`, and uses the
destination family `(code >> 8) & 15` to select a 24-byte-row conversion table.
For ordinary position elements the family table at `142adb000` is:

| Source code | Destination match | Destination mask | Callback |
| --- | --- | --- | --- |
| `132` | `133` | `ffff` | `032a80` |
| `132` | `162` | `ffff` | `032a00` |
| terminator | | | |

The native dispatcher also has a destination flag-bearing matching exception.
The Rust helper deliberately admits only the three exact destination codes;
it does not infer that aliases or additional flags are safe.

With no matching special callback, the dispatcher calculates element width as
`sizes[((sourceCode >> 4) & 15) - 1] * (sourceCode & 15)` and calls `032bb0`.
The retained size table starts `[1,1,2,2,4,4]`, so exact `132` copies four bytes.

## Exact position operations

**OBSERVED callback instructions:** `032a00` loads each source word with
`MOVSX`, converts the signed integer with `CVTDQ2PS` and stores two floats.
Every i16 integer is exactly representable as binary32. There is no division,
normalization, saturation or matrix application in this callback. Zero becomes
positive floating-point zero.

`032a80` loads/stores the original two words and writes a zero word at
destination offset 4. That operation preserves X/Y bit patterns regardless of
their eventual signed interpretation. `032bb0`'s width-four route copies the
original four bytes. Both special callbacks skip accesses when count is zero.

| Exact route | Bytes read per element | Bytes written per element |
| --- | --- | --- |
| `132 → 132` | 4 | Original four bytes |
| `132 → 133` | 4 | Original four bytes, then two zero bytes |
| `132 → 162` | 4 | Two little-endian binary32 signed-integer values |

The callbacks calculate their source end from a 32-bit stride/count product.
The Rust admission policy rejects overflow instead of emulating wraparound or
out-of-bounds native pointer behavior.

The whole `032c90` wrapper assembles scratch chunks and then copies complete
destination strides. Therefore the helper's preservation of all unrelated
destination bytes is an element-level API guarantee, **not** a claim that the
whole native wrapper preserves pre-existing padding. Other attributes and
their conversions are outside this module.

## Destination format and declaration path

`12541e0` derives output elements from selected shader metadata, looking up
source elements by `code & 0xff00`, assigning sequential byte offsets, and
rounding final stride up to four bytes. In the selected position branch, raw
global dword `142c27e70` values **3, 4 or 8** rewrite scalar code bits `30` to
`60`; exact `132` consequently becomes `162`. Other values leave those bits
unchanged in this branch. The global's producer and live value are unbound;
these numbers must not be relabeled as a particular graphics API or runtime
mode. Missing required source semantics clears the output pointers.

The function interns the resulting format and constructs backend object K via
`1203d60`, passing the format and selected shader descriptor. It stores K in
the format's `+10` field at `1254707` or `125490e`. Those are two concrete
output-format paths, not proof of which one becomes a live B's `+30`.

`1203d60` builds an inline element array at `K+10` through `11a4fd0`, stores
the format stride at `K+11c`, calls `06db830` with that array, and persists the
returned declaration handle at `K+114`. `06db830` dispatches to the current
global device's virtual `+50`; the selected backend implementation is unbound.
The referenced global `142c25c78` lies in the virtual-only tail of `.data`,
so there is no file-backed device pointer to follow. The bounded investigation
stops at this concrete dispatch boundary; its receipt is
`gpu-navigation/active-backend-boundary.json`.

For the three exact isolated position codes, `11a4fd0` plus its captured tables
produce these **numeric** type values:

| Position code | Table selector index | Numeric declaration type |
| --- | --- | --- |
| `132` | `21` | 9 |
| `162` | `51` | 2 |
| `133` | `22` | 0, the mapper's default branch |

Indexes in this table are hexadecimal. The CPU's `133` conversion is proven;
its default numeric GPU type is not promoted to a supported GPU short-three
declaration. The Rust module is a CPU transfer adapter only.

For each isolated position element, the mapper writes a 16-byte declaration
record with byte 0 = stream 0, byte 1 = low byte of the format offset, byte 2 =
numeric type, byte 3 = numeric semantic 0, and byte 4 = semantic index. Without
shader metadata the index is `(code >> 12) & 15`; with metadata it is supplied
by the selected attribute record. The qword at +8 is zero; bytes 5..7 are not
written by this mapper. Other semantic combinations include adjacent-element
merging, outside this position-only contract.

The association continues into draw setup: `1246580` passes `[B+30]` into
`1257710`, which passes the same format into `12582b0`. The latter loads
`K = [G+10]`, caches using K and selected shader indices, and supplies
`[K+114]` plus the strides at `K+11c` to `06db1d0` before binding the resulting
state. The same B format supplies the stream stride in `1246580`. This binds
the destination format, numeric declaration and draw setup structurally; it
does not recover the active device implementation or final shader arithmetic.

## Original bounded API and validation

`transfer_positions(source, destination, PositionTransfer)` accepts explicit
source/destination codes, offsets, strides and count. It requires disjoint safe
Rust slices, nonzero strides, elements wholly inside each stride, checked u32
stride/count products, and enough backing bytes through each last element.
Only touched bytes need backing; trailing padding is not read or written.
Every admission check precedes any write, making rejection failure-atomic.
Zero count still validates codes/layouts but requires no backing bytes.

`MAX_POSITION_COUNT = 65_535` is a bounded engineering policy, not a recovered
native allocation limit. Native aliasing, wrapped spans, arbitrary format flags,
GPU declaration construction and allocation/runtime format selection are
explicitly unsupported. The module adds no dependencies or renderer hooks.

**VERIFIED original CPU implementation:** five focused tests cover signed
extrema and unit values, unaligned offsets and distinct strides, raw copy,
untouched bytes, eight failure-atomic rejection cases, and zero count. A
separate oracle derived from the callback instructions before reading the Rust
source supplies nine complete byte-buffer cases across all three routes.
All nine full destination buffers match and all nine source buffers remain
unchanged, including three zero-count cases. The oracle also retains seven
literal float-bit checks. No native executable or GPU execution was run.

The immutable oracle is `oracle/oracle.json`, SHA256
`293f11283ca073094b0f5b3085803b71d8baa25a80c179580c78547f2883efec`.
The standalone std-only Rust comparison receipt is
`oracle/comparison/receipt.json`, SHA256
`d8d691b34381b96bb6c5877a66cff9d7b9617edcd98d82b5591727cead0f2a79`.
Module SHA256 before/after comparison:
`f0c6976981c569f148e2be506d08b8423cb033d62196790d2762b6b0e4dd1c37`.

No actual destination format, packed-mesh identity, normalization-to-pixel
association or native GPU signedness/declaration enum is asserted by these
CPU checks. The previous independent 88-word matrix comparison is unchanged.
