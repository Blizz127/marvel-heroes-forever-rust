# Model declaration and shader input-layout boundary

**VERIFIED (static, conditional backend):** the owned build's table
`0x1421e5448` implements a declaration consumer through
`ID3D11Device::CreateInputLayout`. Its shader input is a retained copy of the
same explicit byte span supplied to `CreateVertexShader`. This establishes the
generic API boundary following [MODEL_VERTEX_INTERPRETATION.md](MODEL_VERTEX_INTERPRETATION.md).
The selected Model declaration has not yet been traced into a particular
state-creation invocation, and its material has not been associated with a
specific compiled shader record. Neither backend activation nor rendering was
executed.

## Conditional receiver and API identity

The earlier declaration route reaches `0x1412cd120` and `0x1412c4b20` on table
`0x1421e5448`. It allocates a 528-byte declaration object; the logical handle
`D` is object `+0x0c`. Records have stride 32 and count at `D+0x200`.

The same table's slot `+0x2c0` selects `0x1412df370`. Its conditional call to
`0x1412d78a0` preserves the receiver. The latter passes `&self+0x90` as argument
eight to the owned `d3d11.dll!D3D11CreateDevice` import, and `&self+0x98` as
argument ten. The official SDK declares these as the device and immediate
context output pointers. Under successful initialization and stable field
identity, `self+0x90` therefore holds an `ID3D11Device`. The separate `self+0x88`
field comes from `dxgi.dll!CreateDXGIFactory`; it is not established as an
adapter. The initialization body does not locally check the device creation
HRESULT before its later device access.

The official header's `ID3D11Device` method order puts `CreateInputLayout` at
slot 11 and `CreateVertexShader` at slot 12: offsets `0x58` and `0x60` with
eight-byte pointers. These offsets must not be applied to the distinct
`ID3D11DeviceContext` interface. The private header audit independently agrees
on C-vtable order, XFG annotations and C++ inheritance. See the pinned
[Microsoft SDK header](https://github.com/microsoft/win32metadata/blob/15fc870a5e96df7c560877ff30136c0fb5a21e9b/generation/WinSDK/RecompiledIdlHeaders/um/d3d11.h).

## Generic layout call and retained shader span

**VERIFIED:** wrapper `0x1406db1d0` calls global receiver `0x142c25c78` at
vtable `+0xb0`; table A selects `0x1412ca500`. A cache miss allocates a
`0x84`-byte state and invokes `0x1412c3f10`, supplying device `[self+0x90]`.
At `0x1412c4079`, that constructor calls device slot `+0x58` with:

| API argument | Native source |
| --- | --- |
| Element descriptors | Incoming declaration handle `D` |
| Element count | u32 at `D+0x200` |
| Shader bytes | qword at `S+0x14`, where `S = vertex shader handle - 0x0c` |
| Byte length | Signed i32 at `S+0x1c`, sign extended to 64 bits |
| Output layout pointer | Address of state object `+0x54`, initially zero |

The state also retains `[S+0x0c]` at state `+0x5c`. Table A's slot `+0x60`
selects `0x1412cd1b0`, whose explicit R9 byte descriptor contains a pointer at
`+0` and signed count at `+8`. It passes that span to device slot `+0x60`, with
null class linkage, then passes the same descriptor and API output to
`0x1412c4d80`. That constructor stores the API shader pointer at `S+0x0c` and
copies the bytes through `0x1407d6e90` into the descriptor at `S+0x14`.
For valid positive counts and successful allocation, the copy reaches the
owned `MSVCR120.dll!memcpy` import with the exact source count.

Thus the ordinary copy path preserves the creation bytes for subsequent
input-signature validation. Neither API HRESULT is checked locally. Negative
counts, diagnostic fallthrough, allocation failures, callback effects and
concurrent mutation are not a safe parser contract or reproduced behavior.

Cache hits skip layout creation. The retained key contains six handle qwords
(declaration and five shader handles) and the low byte of each of 16 supplied
stride u32 values. Lookup compares all eight resulting qwords. Three shader
handles are explicitly zero in this wrapper route. This is handle/stride
identity, not a compiled-code hash; generic insertion and cache concurrency
remain outside this study.

## Descriptor fields and the pre-shader numeric boundary

**VERIFIED source/ABI join:** with the inspected Windows x64 layout,
`D3D11_INPUT_ELEMENT_DESC` has size 32, alignment 8 and field offsets
`0/8/12/16/20/24/28`: semantic-name pointer, semantic index, format, input
slot, byte offset, slot classification and instance step rate. These match the
owned table-A records consumed by the call. This uses the pinned SDK header
and [Microsoft x64 conventions](https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions?view=msvc-170),
not the host Linux ABI or the archived spec's conceptual declarations.

The earlier native mapper and this API join permit the following conditional
names. The selected setter's original offsets are shown before any separately
gated declaration transformation or duplication:

| Original source bytes | Internal type → table-A format | Official DXGI name |
| --- | --- | --- |
| `0..12` | `3 → 6` | `R32G32B32_FLOAT` |
| `12..16` and `16..20` | `5 → 28` | `R8G8B8A8_UNORM` |
| `20..28` and `28..36` | `2 → 16` | `R32G32_FLOAT` |

Internal type codes are not DXGI enums. Internal fallback type `8` maps to
format `28`, while DXGI enum `8` means `R32G32B32_SINT`. Other native backends
retain their separate mappings. See the pinned
[official DXGI header](https://github.com/microsoft/DirectX-Headers/blob/61c7718954ec73dbdd43c723240fe1a0452a3724/include/directx/dxgiformat.h).

Under this API's input-assembler conversion, an eight-bit UNORM component
represents `byte / 255`; this is the pre-shader value, not a signed basis
decode. The specification requires exact integer operations and 0.5-ULP
precision for the applicable floating arithmetic. Existing 32-bit FLOAT
components are fetched without changing their bits. No shader `2*x-1`,
cross-product reconstruction, material channel, color-space meaning or actual
GPU result follows. See [DirectX functional specification sections 3.2.3.2,
3.2.3.5 and 8.20](https://github.com/microsoft/DirectX-Specs/blob/9c2586441fa2dd0d4953a5c4e9b9ff05717c19eb/d3d/archive/D3D11_3_FunctionalSpec.htm).

`CreateInputLayout` validates the descriptors against the supplied compiled
shader input signature; declaration creation is distinct from later
input-assembler binding. Compatible storage and semantic strings alone do
not establish a shader match. The selected main declaration's repeated fields,
separate fallback stream and conditional offset changes remain as documented
in the preceding study. They do not authorize reads past a saved 36-byte
record. [Microsoft API contract](https://learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createinputlayout).

## Selected Model and stored shader gaps

**OBSERVED evidence frontier:** no retained dataflow carries the selected
Model resource's declaration at `+0x22c` into this generic state-creation call.
The wrapper's initial caller anchor came from the separate Scale9 path. The
existing concrete Model handoff is `0x140bfabc0`'s normal draw-record branch:
it retains Model `+0x1b8` and calls `0x140ceb180` at `0x140bfb3fc`. That direct
consumer is the narrow next Model-side investigation target; its behavior is
not established here. See [MODEL_RENDER_RESOURCE.md](MODEL_RENDER_RESOURCE.md).

The selected FrontEnd Level 11, Model 13 and ModelComponents 14–23 use imported
Engine.Material references `-33` (`removesurfacematerial`) and `-32`
(`defaultmaterial`). These are package references, not zero-based export
indexes. The original material graph stops at external imports and preserves
native material tails; it supplies no effective shader variant or code span.
See [MATERIALS.md](MATERIALS.md).

Existing verified pin metadata contains four named global/reference shader
caches, together with 181 Engine/Shaders `.bin` files. The study checked the
four explicit cache sizes against that metadata, without reading their
contents. Their filenames establish neither the current backend nor a bounded
shader-record format. The missing joins are selected resolved material and
variant → shader handle/source descriptor → owned file hash and bounded record
with any transformation → input signature and shader arithmetic. No arbitrary
cache magic or unrelated material was substituted for those identities.

No new product reader or conversion is added. A specific shader-byte record
and its container/version/length contract remain prerequisites; no native
runtime, shader compilation, renderer execution or product build/test was run
for this documentation-only result.

## Sources, licences and verification

All native bytes remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/model-input-layout/`.
The owned 1.52.0.1700 executable SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Three fresh target-free REA/Rizin records cover 13 complete bodies totaling
3,581 code bytes, three explicit thunks totaling 18 bytes, and 41 metadata
bytes. Independent PE comparisons include complete chained extents and the
one separate padding byte. No discrepancies were found. The exported bundle
SHA-256 is `fef4aaad623d2c74415a241180ca0a90bf370d18a037551684b0ffdb4991c76a`;
REA cleanup records no open target, pending cleanup or retained records.

| Private component | Receipt SHA-256 | Scope |
| --- | --- | --- |
| `native-route/receipt.json` | `40836831674144b013f39a33d15c0bd09468160c5dd73599e104628731e7ac08` | Conditional API, receiver/import and copied-span chain |
| `primary-api/receipt.json` | `22d2698d017c987d33c8aef5244c1265d935505efd5847fff56fe8d874f7a8a6` | 25 official download identities; independent ABI/spec checks |
| `shader-boundary/receipt.json` | `7e4b17b708845a1d5cb891d2d1273ceb7dc0686dc34901eaf9daff0a636b25a6` | Selected imports, existing metadata and shader identity gap |

Exact official snapshots, paths, content hashes and licence evidence are in
`primary-api/provenance.json` (SHA-256
`9315d4e4b3f443d1e2bb90ca5a357a6d89c68a83744ce60b5dea5caa816b50e7`).
These sources are used for compatibility study only; no external implementation
or header enters product code.

| Outside source and immutable revision | Licence and use |
| --- | --- |
| [Microsoft win32metadata](https://github.com/microsoft/win32metadata/tree/15fc870a5e96df7c560877ff30136c0fb5a21e9b), `generation/WinSDK/RecompiledIdlHeaders/um/d3d11.h` | Original Microsoft SDK-header licence retained; repository MIT does not relicense SDK headers. Device ABI, method order and descriptors. |
| [Microsoft DirectX-Headers](https://github.com/microsoft/DirectX-Headers/tree/61c7718954ec73dbdd43c723240fe1a0452a3724), `include/directx/dxgiformat.h` | MIT. Numeric format definitions. |
| [MicrosoftDocs/sdk-api](https://github.com/MicrosoftDocs/sdk-api/tree/656e9656bfaef34dd29cd7ee9e1146aec17e924c), CreateInputLayout, input-element and DXGI pages | Documentation CC BY 4.0, separate code licence MIT. API signature/validation contract. |
| [Microsoft DirectX-Specs](https://github.com/microsoft/DirectX-Specs/tree/9c2586441fa2dd0d4953a5c4e9b9ff05717c19eb), `d3d/archive/D3D11_3_FunctionalSpec.htm` | Documentation CC BY 4.0, separate code licence MIT. Input conversion and precision; conceptual archived declarations excluded as ABI evidence. |
| [MicrosoftDocs/cpp-docs](https://github.com/MicrosoftDocs/cpp-docs/tree/af4947c4bdf5993dcaee98e63d55a2d7a5744d7d), x64 software conventions | Documentation CC BY 4.0, separate code licence MIT. Pointer and aggregate layout. |

Original project documentation/modules, previous private receipts and bounded
Scale9 navigation records are registered in the parent private source ledger.
The latter's direct-call candidates remain navigation evidence; they are not
promoted to verified Model callers. Owned retail files are proprietary, and
original research scripts/results remain private.
