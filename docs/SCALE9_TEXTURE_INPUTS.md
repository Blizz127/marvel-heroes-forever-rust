# Scale9 external Texture2D inputs

**VERIFIED static native operations and OBSERVED owned asset bytes.** This
investigation connects the conditional `Engine.Texture` request
`MarvelHUD.Tooltip_Main_BG_PNG` from [SCALE9_RENDERING.md](SCALE9_RENDERING.md)
to the exact saved texture and separates the dimension sources consumed by
the returned image and its normalization matrix. No native client, network,
GPU or renderer profile was run or enabled. A successful lookup and its
current loaded object/resource state are conditions, not observations.

Private paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/`. New evidence is under
`evidence/scale9-texture-inputs/`.

## Exact saved object and independent reference

The fully hash-verified `MarvelFrontEnd_SF.upk` has zero-based export **562**,
signed object reference **563**, name `tooltip_main_bg_png`, outer reference
**4** to package `marvelhud`, and class reference **-3** to `Engine.texture2d`.
The stored 8,905,511-byte package has SHA256
`af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190`;
its independently expanded 38,738,644-byte logical package has SHA256
`2c199b2b523e4334e8193b875024de101a8702714f2752c9502711f22ca1082c`.

The same package's `marvelhud.!uisource` SwfMovie export23 contains exactly
the retained 355,670-byte movie with SHA256
`49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5`.
Its References array slot224, logical package offset3362760+4, holds **563**.
This is a direct saved reference to the selected texture, in addition to the
earlier external-image filename and export-name evidence. It does not prove
that an executing object loader selected the object or reused this reference.

The selected texture's 4,477-byte export is at logical38305321 and has SHA256
`fa0feefdc0f1acb3f489a07175757ad91ac61d7911efba0c0035ec55e5dac03b`.
It contains exactly nine tagged properties:

| Property | Saved value | Logical value location |
|---|---|---|
| `SizeX` | i32 64 | 38305349+4 |
| `SizeY` | i32 64 | 38305377+4 |
| `OriginalSizeX` | i32 64 | 38305405+4 |
| `OriginalSizeY` | i32 64 | 38305433+4 |
| `Format` | `EPixelFormat.PF_DXT5` | 38305469+8 |
| `TextureFileCacheName` | `textures` | 38305501+8 |
| `MipTailBaseIdx` | i32 6 | 38305533+4 |
| `SRGB` | explicitly false | Boolean tag byte38305561 |
| `LODGroup` | `TextureGroup.TEXTUREGROUP_UI` | 38305594+8 |

Names above use familiar capitalization; the package's exact FName strings
are preserved in the receipt. `NeverStream` and `LODBias` are absent. Their
defaults are not inferred. `MipTailBaseIdx=6` is not seven resident mips.

The complete 4,188-byte native tail has empty SourceArt, one primary mip,
the cache GUID, and empty retained cached-format arrays/bulk data. Mip0 is
64×64 with **4,096 inline bytes**, flags0, exact payload at38305646. Its
SHA256 is `3aa3f438afc77ffbc504d08a7002b133326f42702667ddd8e6312e3c72ceffa1`.
The cache name does not make this mip external. No TFC is needed for these
saved bytes. The single saved mip proves neither resource residency nor a
live GPU image size.

`owned-texture/owned-texture-receipt.json` retains every tag, payload and
native-tail span. Its SHA256 is
`3791c85a01eb8239277897625841ed62fa9321ccdf8cac9ed42584c12c214be1`.
An independent original decoder and the project's Rust reader over the
stored compressed package agree on the complete export, all nine properties
and the native tail; `owned-texture/project-check.txt` records PASS.

## Native class checks and returned dimensions

The three query helpers return the original object pointer or null; they do
not select a mip or ask a resource for its dimensions. With a successfully
initialized nonnull class cache, they walk object+0x50's class ancestry via
class+0x78 and compare against these exact compiled class names:

| Query | Class | Class cache |
|---|---|---|
| `0x1408f5df0` | `Engine.Texture2D` | `0x142c28208` |
| `0x1409c7750` | `Engine.TextureRenderTarget2D` | `0x142c28550` |
| `0x1409c76e0` | `Engine.Texture2DDynamic` | `0x142c28710` |

If lazy registration returns normally with a still-null target cache, the
helper accepts the original pointer without this ancestry check. Therefore
the source is not rewritten as an unconditional strict cast. Registration
success is not observed, nor is this branch an allocation-failure guarantee.

Wrapper `0x141268d60` rejects a null asset and chooses dimensions in this order:

1. Texture2D succeeds: dwords at object+0x180/+0x184.
2. Otherwise Texture2DDynamic succeeds: object+0xfc/+0x100.
3. Otherwise query TextureRenderTarget2D: object+0x104/+0x108.

The all-query-null fallback reads caller home-space words whose initialization
at the selected factory call is unproved; no saved-declaration or zero fallback
is invented. For the successful branch, the selected words are snapshots passed
to texture-manager virtual+0xd0 and independently saved at returned image
+0x2c/+0x30. They are not reread from the manager's resulting Q object. Image
virtual+0x28, `0x1400cb740`, copies these two u32 words. These are precisely the
returned dimensions used by the existing binder ratio proof.

## Q state and the separate normalization pair

Use **Q** for the concrete engine texture wrapper and **R** for the record
through Q+0x5c. Base constructor `0x14125e780` copies the supplied size to
Q+0x30/+0x34 and clears Q+0x40. Subclass `0x14125e5d0` sets mask0x04,
points Q+0x5c to its inline R, saves the asset at R+0x10, and copies the
supplied dimensions to R+0/+4. It also copies asset+0xd8 to R+8.

Initializer `0x141274180` enters the mask0x04 branch, calls `0x141273f60`,
then returns. The generic resize loop and its later mask0x01 setter are
bypassed in this branch. The initializer queries and records all three types.
Its dimension precedence is **Texture2D, TextureRenderTarget2D, Texture2DDynamic**,
which differs from the wrapper's precedence. An all-null result retains R's
constructor-supplied pair. No class-mutual-exclusion assumption is required.

For successful Texture2D, it additionally copies object+0x188/+0x18c to
Q+0x30/+0x34 and byte object+0x1b8 to Q+0x3c. Without that type result,
Q+0x3c becomes1. Only when **both** Q+0x30 and Q+0x34 are zero does the
initializer replace them with R+0/+4; it does not repair each zero axis
independently. The initializer writes Q+0x38=2 and returns success in the
retained path without a dimension-nonzero check or final nonnull Q+0x50 guard.

Q virtual+0x38, `0x14126ea10`, chooses its input pair as follows:

| Current state | Selected normalization dimensions |
|---|---|
| Q+0x40 has mask0x01 | Q+0x30/+0x34 |
| Otherwise | R+0/+4 through Q+0x5c |

It zero-extends each u32 to a 64-bit integer, converts with CVTSI2SS, and
performs separate `1.0f32 / width` and `1.0f32 / height` divisions. The first
reciprocal store precedes the second division. There is no zero guard or mip
index. This describes the getter's current supplied state; the constructor
alone is not proof that the mask remains unchanged at a later draw.

The external initializer separately clears R+0x30 and chooses R+8 from that
override if present, otherwise asset+0xd8. Format selection writes Q+0x50
using a format byte and manager call; the Texture2D byte is object+0x190.
Neither path replaces the direct dimension loads with a resource size query.
These observations do not establish actual GPU backing, atlas region,
resource lifetime, selected mip or upstream changes to the object's fields.

## Matrix inputs stay distinct

The saved tag1009 declaration, returned image pair and selected Q normalization
pair remain separate inputs. As proved in `SCALE9_RENDERING.md`, the binder
computes declared-height / returned-height before the width ratio, scales the
identity rows with separate f32 multiplications, then calls image virtual+0x68
to store the **native inverse** of that ratio matrix. Image virtual+0xb0 later
composes Q's normalization matrix with that stored inverse before its rectangle
origin operations. Constructor-null image+0x20 is not the final bound state.

Equal dimensions in this one saved asset do not license cancellation, a fixed
`1/64` input, or substituting the selected mip dimensions for every native
pair. The existing non-equal independent witness demonstrates that native
inverse rounding differs from dividing the opposite ratio directly.

## Conditional native field-name correspondence

The owned Engine declarations prove the order `SizeX → SizeY → OriginalSizeX
→ OriginalSizeY`, all IntProperty/ArrayDim1. They do not serialize native
offsets. The preceding fields are four StructProperty references to
`Core.Object.IndirectArray_Mirror`, an IntProperty maximum-resolution field,
and a StructProperty reference to `Core.Object.UntypedBulkData_Mirror`.
The saved Core mirror declarations alone do not establish native 8/16/64-byte
extents: `Pointer` contains a saved Dummy IntProperty. The following native
rules supply the missing layout operations without using this texture's
matching 64-valued fields.

The complete retained UStruct Link at `0x14074a910` has distinct modes. With
nonzero third argument it resets/recomputes the extent, obtains the inherited
extent/alignment, then calls each eligible direct property's virtual+0x280.
After each property it writes owner+0x88 = property+0x8c + property+0x6c *
property+0x68 using 32-bit operations. It passes the previously linked property
as the next call's third argument. The zero-argument mode instead saves and
restores property offsets around linking; it is not interchangeable with this
recipe. Conditional reorder/preload gates and live metadata remain relevant.

Exact class-name builders, constructor-installed tables and table slots now
ground these selected property methods:

| Property | Link method | Size and alignment contract |
|---|---|---|
| Int / Float | `0x1407c7b20` | size4, alignment4 |
| Byte | `0x1407c7a60` | size1, alignment1 |
| Bool | `0x1407c7990` | size4, alignment4; conditional consecutive-bit packing |
| Struct | `0x1407c7da0` | preload referenced struct, align its size to its alignment, align the qualifying owner extent likewise |

These methods choose the owner extent only when the owner's class mask has
bit0x08; otherwise they use zero. The concrete class metadata table installed
by `0x14073a870` is `0x141df82c8`; its virtual+0x260 returns1. Under that
policy, a preceding Bool property with the required class bit and a nonzero
lower31-bit mask lets the next Bool reuse its offset and double the mask.
Thus 18 adjacent scalar Bools share one four-byte word under this profile.
It is not an eight-Bools-per-byte rule. Struct linking queries alignment twice;
the replay requires unchanged referenced metadata between those calls,
including no intervening helper-side mutation.

The exact native name-registration block `0x1407b2489..0x1407b250d` writes
entry+8=0xc2, copies the eight-byte NUL-terminated `Pointer` literal to
entry+0x14 through `strcpy_s`, then calls `0x14079cca0`. That complete helper
publishes the entry at index `entry+8 >> 1`, **0x61**. The retained Link
name check compares the whole ordinary FName qword at struct+0x48, with
struct+0x38 != -1. Matching unnumbered Pointer/0x61 overrides its computed
Dummy extent to **size8/alignment4**. Successful registration and ordinary
name identity are explicit conditions; no generic host-pointer rule is used.

The saved IndirectArray mirror is Pointer + two Ints, giving **16 bytes**.
The saved bulk mirror has three Pointers and ten Ints, giving **64 bytes**.
Both have alignment4. This composition is an original replay of the recovered
operations, not a native runtime invocation.

Using the supplied incoming Surface extent0x60, its two saved Function children
add no instance property. The 37 saved Texture property declarations produce:

| Saved Texture group | Conditional resulting extent |
|---|---|
| 18 Bools | 0x64 |
| UnpackMin[4], UnpackMax[4] | 0x84 |
| SourceArt bulk mirror | 0xc4 |
| Five bytes | 0xc9 |
| Alignment and three Ints | 0xd8 |
| Resource Pointer | 0xe0 |
| Six Floats and InternalFormatLODBias Int | **0xfc** |

The named Texture builder `0x1409b57b0` independently supplies native extent
0xfc through `0x14073a870 → 0x14073ad70 → 0x14073ae90`, whose last function
stores it at class+0x88. This is a constructor-time cross-check; it does not
prove the active native loader preserved that extent or chose this Link mode.
Core.Object's native constructor extent0x60 is independently retained in
[WORLD_TRANSFORM_DEFAULTS.md](WORLD_TRANSFORM_DEFAULTS.md). The full loaded
class-link policy and incoming extent remain a stated boundary.

From that same conditional Texture extent0xfc, the exact Texture2D declaration
order produces the native offsets consumed by the image path:

| Saved field | Offset | End |
|---|---|---|
| Mips | 0xfc | 0x10c |
| CachedPVRTCMips | 0x10c | 0x11c |
| CachedATITCMips | 0x11c | 0x12c |
| CachedETCMips | 0x12c | 0x13c |
| CachedFlashMipsMaxResolution | 0x13c | 0x140 |
| CachedFlashMips | 0x140 | 0x180 |
| **SizeX / SizeY** | **0x180 / 0x184** | 0x188 |
| **OriginalSizeX / OriginalSizeY** | **0x188 / 0x18c** | 0x190 |
| Format | 0x190 | 0x191 |

`layout-replay.json` retains every property-level arithmetic step, exact input
receipt hashes, metadata conditions and the constructor cross-check. Under
that profile, the wrapper and ordinary R pair correspond to **SizeX/Y**; the
alternate Q pair corresponds to **OriginalSizeX/Y**. Resource corresponds to
Texture+0xd8. The full loader policy is not replaced by this conditional
correspondence, and byte+0x1b8 is not given a property name by this limited
prefix calculation.

The product's `Texture2D::Descriptor` retains SizeX/Y but currently drops
OriginalSizeX/Y and LODGroup. `ObjectProperties` retains their exact typed
tagged values and spans. A future adapter can use those explicit properties
once the native name-to-offset profile and load-state contract are admitted; it must
not silently use Descriptor size for all three dimension roles.

## Input adapter handoff and exact remaining gate

A minimal future adapter should retain three typed values independently:
`declared_image_size` with its movie/tag identity, `asset_size` and
`original_asset_size` with exact export/property spans, and a separate supplied
current native/Q state. A selected mip should carry its own dimensions and
payload identity; it is not an implicit substitute for any of those values.
The existing saved asset supports all these static property/payload records.

This increment does **not** add an adapter that presents the saved words as a
successfully loaded native snapshot. The precise missing bridge is admission
of the compiled class-link profile/inherited extent and proof of what the
successful object load/PostLoad/resource lifecycle has done to the four
fields before wrapper construction and external initialization. A current
normalization mask, retained resource lifetime and selected GPU backing also
cannot be recovered from the UPK alone. A future conditional adapter may
accept those as explicitly supplied assertions, but must label them as such.
Returning an explicit unsupported-profile result is preferable to filling
those gaps with 64×64, current mip dimensions or inferred default flags.

For the saved tooltip, all three saved pairs happen to be 64×64. With an
explicit successful fresh external Texture2D profile whose loaded fields
preserve those values, the returned image pair and the initial mask0x04
normalization pair would both be64×64. That is a conditional consequence of
the complete source sequence, not evidence that the native profile ran.

## Sources and validation scope

| Source | Licence/notice and use |
|---|---|
| Owner's pinned `MarvelHeroesOmega.exe`, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary, authorized private interoperability study; bounded native functions, tables, class literals and exact byte validation. |
| Owner's stored/logical `MarvelFrontEnd_SF.upk`, hashes above | Proprietary; exact saved movie, texture reference, properties and payload spans. |
| Owner's pinned `Engine.upk`, SHA256 `66dd5dca80f275a8cc2de5bf4b03042949cfccd0afc0edc6b7ba9b577cde4f51`, and `Core.upk`, SHA256 `5cbe7d50003197d7096efcae9fc2eaf392a6d44458dfe51ed6487266dcf36ccb` | Proprietary; saved class/property/struct declarations and exact Children/Next chains. |
| Original project `texture.rs`, `upk.rs` and property readers | Original code; focused independent parser agreement and API assessment. Exact hashes are retained in `owned-texture/source-ledger.json` and `api-review/`. |
| `scale9-output`, `external-image-route`, `frontend-image-context`, `localized-result-object`, `world-transform-defaults` and `animation-runtime` private evidence; corresponding project docs | Original owner-artifact research; reuse existing wrapper, binder, state, reflection, Core.Object extent and matrix proof. Earlier constructor-null image-matrix speculation is superseded by the binder setter proof. |
| REA target-free Rizin, GNU objdump and Python standard library | Analysis tools only; no target code execution or borrowed implementation. |

Every new capture is compared against the hash-pinned PE section mapping.
`verification/capture-validation.json` retains provider completion, truncation,
unchanged-artifact status, contiguous instruction spans and literal byte checks.
`verification/size-operation-independent/size-operation-receipt.json` independently
audits the retained size, initialization and reciprocal operation order, SHA256
`b90fad7df10cec9806fdca171d8304a3502fe5b927be0abb159c6cc29eafdb4d`.
No full suite, native runtime, GPU render or pixel-parity check was run.

Four new complete, untruncated, unchanged-artifact REA records retain 984
instruction spans / 4,264 instruction bytes and 358 exact data bytes. Every
returned byte independently matches the pinned executable:

| Scope | REA Evidence ID |
|---|---|
| Three texture-class queries and named builders | `ev_44154a8e1903b9cb326598d777fd6c25f079b434efaf4e0fc04298ffb38217b2` |
| Class extent constructors and Pointer registration | `ev_95dc00f6b2c8917288cb4762ddbb249b72e540f4e85d670b6716f8b444a24d1e` |
| Int and Struct property layout | `ev_4a2173dc8fc6e0816b4119abecfa040a382d51b6c04be3528f1a751d66c7ee4c` |
| Byte, Float and Bool property layout | `ev_a9d7d7310694892f54425b2aa9a5e899f72aad8db3da10c1c57dd875c13059ac` |

The retained UStruct Link and serializer anchors are reused from
`localized-result-object/native.bundle.json`, especially
`ev_cf2350e1a236a88f6656959e8fb7f6ee98d26fb5f02c57db13f9ef361f3be092`.
No new broad scan or native execution is implied by that reuse. The complete
task bundle `complete.bundle.json` has 70 accumulated records, SHA256
`d66fdf3fdf481f8be44112efc93f1c9381fa85a4a0e6660e7b411f983a00d8d5`.
REA resource close completed. `layout-replay.json` has SHA256
`57f0193d896a04606b80bf0df9c11bf605420b3f9b56a6bcffafa84f7aab1cdf`.
