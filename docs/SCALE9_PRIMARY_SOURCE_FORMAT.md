# Native primary fill and source-format association

This study follows the ordinary provider's selected fill into the source format
used by the upload preparation described in
[SCALE9_DRAW_MESH_IDENTITY.md](SCALE9_DRAW_MESH_IDENTITY.md). It is static
interoperability evidence from the owner's pinned client, not a retail execution
or an assertion that the current Rust renderer selects the route.

The target is the private build 1.52.0.1700 `MarvelHeroesOmega.exe`, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Addresses below are preferred virtual addresses with the common `0x140000000`
base omitted for code. Field offsets and flag masks are hexadecimal. P denotes
the ordinary collection object, I the provider interface, and V its full owner
at `I-10`; these are distinct objects.

## Provider selection

**OBSERVED:** constructors `c9740` and `122130` install interface tables
`142507100` and `142509cb0`, respectively, at full-provider `+10`.
Both tables' `+58` entries point to `beb80`. This selector takes a layer index,
a subfill index, output record D, and an explicit flag argument. It writes:

| Selected branch | D format at `+18` |
| --- | --- |
| Input flags include bit `2` | `142adb240` |
| `bea80` returns null | `142adb240` |
| Returned fill's `+18` is nonnull | `142adb240` |
| Image branch, without the exception below | `142adb240` |
| Image branch, layer dword `+10` nonzero and input flags include `8` | `142adc3b0` |

The image branch also changes D's numeric `+10` field to `0xb` for the
exception. It obtains its resource through the selected fill's `+10` virtual
`+48`; it does not select an image by a filename here. Helpers `03a9c0`,
`03aa00`, and `03aa30` initialize the temporary records. Although `03a9c0`
with kind 1 initially uses `142adb1c8`, the selector's flags-2 branch explicitly
overwrites the output format with `142adb240`.

**OBSERVED conditional exclusion:** the existing `c5d50` dispatcher reads
`H=[M+60]` and the same selected provider layer's dword `+10`. With H nonnull
and that dword nonzero, it diverts to `c27f0` and does not call primary `c4dd0`.
Thus a nonnull-H call that actually reaches `c4dd0` has layer dword `+10` zero,
which excludes the selector's alternate-format branch when both operations use
the same stable provider/layer. H-null calls bypass this first test and can
reach `c4dd0` with a nonzero layer dword; primary dispatch alone is insufficient.
The layer dword `+0c` must also be zero for the selected `c4dd0` route.

These are pointer, branch, and stable-input conditions. A saved ScalingGrid tag
alone does not establish a live nonnull H or unchanged layer storage.

## Source-side fill selection

**OBSERVED:** source constructor `0af6e0` invokes `0af320`, passing its actual
provider and layer. `0af320` forms interface I at full-provider `+10`, calls
its `+50` count query, and uses the ordinary fill path only when that result is
at most one. The other branch invokes `13b0d0` and is outside this association.

On the ordinary path, it initializes D with kind 3, then calls I's `+58` with
the selected layer and subfill zero. The selector flags are formed from the
source argument, rather than assumed equal to the later mesh flags:

- Start with 1 exactly when `(sourceFlags & 0x0c) == 4`, otherwise 0.
- If `sourceFlags & 0x40`, replace that value with 2.
- If `sourceFlags & 0x80`, additionally set bit 8.

It passes D to `03c420` and passes the returned fill pointer to `13b080`.
That helper stores the pointer unchanged as the second word of a two-word key.
The key's first word is one of the explicit globals `142adc440` or `142adc450`,
selected by a separately derived ancestry flag. This flag is not a format code.

**OBSERVED:** both globals' initial file-backed table pointers are
`14250a1c0`, whose `+20` entry is `13c000`. Constructor `0ed4a0` copies the
key interface and payload to the new source's `+88` and `+90` respectively,
then invokes the key interface's `+8` reference operation. The source constructor
subsequently installs `1425063b0`; its later mesh getter is the existing one
from the draw/position study. The destructor-like `13b8f0` installs a different
base table `14250a190`; its receiver offset must not be substituted for the
live global key table.

The selected `03c420` factory contains a six-entry jump table. In its image
branch it copies D's format pointer at `+18` to a temporary descriptor's `+20`.
A successful resource callback supplies the descriptor's resource pointer;
the resource-missing fallback uses a distinct numeric color/type descriptor
but preserves that same format pointer. This study keeps the external image
resource callback and its side effects outside the geometry contract.

**OBSERVED, reused canonical evidence:** `03c300` receives that descriptor.
On a null cache table or lookup miss, it calls the cache owner's `+10` object's
virtual `+e0` with the descriptor. The concrete renderer table `1421ba230`
has `1412452f0` in that slot. On a successful returned allocation, this backend
passes `newFill+10` and the unchanged descriptor to `141203760`. The latter
copies descriptor `+20` to its destination `+20`. Consequently this concrete
backend stores D's selected format in `newFill+30`.

This chain requires the actual cache backend to have that concrete table and
successful, nonaliasing storage. Resource-reference operations and allocator
callbacks are real calls, not assumed pure operations. The cache wrapper then
inserts the result and stores its back-reference at fill `+38`; it does not
write a substitute format itself.

The selected source's cache identity is explicit: complete getter `0ebbd0`
returns `[source+28]+d20`; complete getter `0ebf60` returns
`[[source+28]+50]`. These are the owner/cache offsets recovered by the separate
options-construction lane. Matching those fields is a concrete prerequisite;
the getter bodies do not identify an arbitrary live owner's constructor or
prove that intervening calls preserve the fields.

Cache hits need a separate statement. `03b320` compares the descriptor's two
dwords, two bytes, and two resource pointers with the cached object's embedded
descriptor. For its final format comparison it first accepts pointer equality;
otherwise it calls `03a900`. Therefore a successful cache hit must not be
described as always preserving the literal incoming format pointer.

**OBSERVED complete comparator `03a900..03a980`:** each iteration may skip
one element on each side when `(element.code & 0xf0)` is `0x70` or `0x80`.
It compares the next surviving element's offset and full code, and requires
the terminal pair to match as well. It does **not** compare the format's
stride field at `+0`. This is a particular native compatibility comparison,
not full descriptor equality and not a repeated skip-until-ordinary operation.
The 144-byte bounded probe also includes a separate 9-byte leaf at `03a980`
and seven padding bytes; neither is part of this comparator.

Consequently a freshly constructed fill carries the selector's literal format,
and an exact-pointer cache hit carries that same pointer. A compatibility-only
cache hit retains its existing format, potentially with a different stride or
skipped elements. A future packed-record adapter must inspect/admit that actual
format instead of treating successful cache lookup as a literal-format proof.

## Collection constructor and remaining dispatch boundary

**OBSERVED:** `13c000` saves its incoming RDX unchanged. On its missing-bundle,
successful-allocation path it constructs P at `bundle+48` and passes that saved
pointer as argument 3 to `03d1d0`. The latter stores argument 3 at `P+20`.
Thus this method's incoming fill becomes the exact fill object whose `+30`
the previously recovered queue initializer reads as source format F. The
method also has a preexisting-bundle branch; creating a new P must not be
claimed on that branch. The separate `173590` caller constructs a derived P
from an indexed provider and is retained only to distinguish that route.

**UNRESOLVED:** the caller that dispatches the saved key's `+20` method with
its retained payload as RDX has not been identified. Having the key's concrete
table, payload storage, and callee argument contract is not a proof that the
actual caller joins them. No direct source-to-P runtime association is claimed.

The bounded navigation searched RVA `030000..180000`, with a 64-byte window,
for paired source fields `+88/+90`, then key-like fields `+0/+8` and
`+18/+20`, followed by indirect `+20` calls. The first and last forms had no
matches; the middle form had four unbound candidates and was not promoted to
a key-dispatch claim. A final indirect tail-jump variant of the two key pairs
had no matches. Exact encodings, limits, and family metadata are private in
`constructor-navigation/`; these negatives do not exclude other register flows,
offset bases, calling forms, or locations. This specific search was stopped.

The implementation-ready result is therefore conditional: with the saved
payload demonstrably supplied to `13c000`, a fresh successful collection and
fresh or exact-pointer-hit fill preserve the selector's literal format. The
nonnull-H/stable-layer primary condition above excludes the selector's alternate
image format. Compatibility-only cache hits require inspecting the actual
cached format. Callback lifetimes and protected-field carry remain explicit.

## Packed record contract

The retained primary writer `bff70` and its caller establish a separate
serialization contract. It writes a 12-byte record with raw packed X at `+0`,
Y at `+2`, a native color dword at `+4`, and two flag-derived bytes at `+8`
and `+9`. It makes no store to the padding bytes `+10` and `+11`.

The exact table at `1425001ec` is `[0,127,255,255]`. The first byte uses
`vertexFlags & 3`; the second uses `(vertexFlags >> 2) & 3`. These flags belong
to each exported vertex and are not the mesh descriptor flags. In particular,
descriptor `0x8008` does not imply that a vertex has flag `0x8000`.

For the selected `c4dd0` call, argument 6 to `bff70` is zero, excluding the
forced-style branch. The ordinary writer starts with color zero. A vertex's
`0x8000` bit skips color lookup; otherwise it obtains a color from the actual
shape-data receiver `[V+58]` via virtual `+20`, using the selected style ID and
a 16-byte result buffer. It copies the first dword. This receiver is distinct
from provider interface I; the latter's `+20` happens to be a no-op and cannot
substitute for the shape-data color method.

The non-AA rectangular output's vertex flags `0x000a` select style 0 of its
two retained style words, which is style ID 1 on that recovered rectangle.
The callback's color remains an explicit native input. Neither bitmap resource
presence nor style ID 1 proves that it is white. A morph owner at `V+60` and
nonzero morph factor introduce additional callbacks and arithmetic. The
dual-style flag `0x10` branch uses `((color0 | color1) >> 1) & 0x7f7f7f7f`;
it must not be replaced with ordinary component averaging.

Source vertices, normalization, output ordinals, and triangle selection retain
the contracts in [SCALE9_TESSELLATION.md](SCALE9_TESSELLATION.md) and
[SCALE9_GEOMETRY.md](SCALE9_GEOMETRY.md). The signed coordinate conversion and
downstream destination selection remain distinct, as documented in
[SCALE9_PACKED_VERTEX_FORMAT.md](SCALE9_PACKED_VERTEX_FORMAT.md).

## Evidence and limits

New evidence is private under `evidence/scale9-primary-source-format/` beneath
the project private-data directory. The only outside source used in this study
is the owner-provided proprietary client binary, read for interoperability.
All other inputs are prior project evidence and original project source.
No third-party implementation or asset bytes are added to the repository.

The seven new canonical records are:

| Private file under `native/` | Evidence ID |
| --- | --- |
| `rea-provider-format-selector.json` | `ev_cbe47f86da33946f97154c2c507f710301e6a8960a4f282e53c5260e9b68b113` |
| `rea-fill-object-callers-and-pack-table.json` | `ev_636cfd31cadc290e53e3d07f1528b53cfb595f4d83766ed51ec5298a8a868bc7` |
| `rea-source-fill-key.json` | `ev_799b101190a786c0e99158bf902a56c73cf4e412c123ad7f974930e9278b2675` |
| `rea-fill-factory-and-key.json` | `ev_628a6ab9e4b3e85f13f04cd9cf485df323f1e1b843e1518149f5328c840fb422` |
| `rea-source-key-storage-and-global.json` | `ev_a6da7a6a68f541341b59e0e55ad0516d2c5f827ad374ba625680a2de2043318a` |
| `rea-cache-getters-and-lookup.json` | `ev_c1ab0f09d481eef84c5d786692d48c27a8519d0f9930431ccbd6b6ee2f8786ed` |
| `rea-cache-format-comparison.json` | `ev_fcac84cbbf7ea326cc27efb540f2104956bc4115d99acdea907f2619acc5c6c4` |

`native/final.bundle.json` contains those seven records, SHA256
`207bb5fbe05aa5112ef7de3640469183ce95640c9998435e4ab21c5d10e0415b`.
The investigation was exported and reset. Every command completed with exit 0,
untruncated output, and unchanged pinned executable hash.

**VERIFIED:** the original private `constructor-navigation/native_audit.py`
checks every captured instruction/data byte through the actual PE sections and
the tool's raw-file mapping. It passes all seven records with zero errors or
warnings: 930 code/alignment rows covering 3,435 distinct bytes, plus 244 bytes
of explicit data captures including overlapping table observations. The six
invalid decoder rows within the 24-byte `03c420` inline jump table are retained
and classified as data, independently confirmed by a separate `pxj` capture.
The `.data` globals use a different raw mapping from `.text/.rdata`; the audit
records both tool addresses and canonical PE addresses rather than conflating
them. Audit success verifies captured bytes, not runtime behavior.

Reusable evidence is indexed, with complete IDs, hashes, and exact spans, in:

- `reuse/reuse-ledger.json`: primary writer/descriptor/export/color contracts.
- `reuse/fill-storage-reuse.json`: concrete renderer backend and embedded
  descriptor copy from the options lane's already sealed records.
- `reuse/key-dispatch-reuse.json`: source construction and the original missing
  dispatch boundary; later key-table corrections are retained separately.
- `selector-navigation/primary-path-join.json`: exact ABI and eight symbolic
  dispatcher/selector branch cases. These are static hand traces, not native
  execution tests.

No Rust module is added in this increment. A subsequent bounded 12-byte writer
can compose the existing position packer with explicit, already resolved color
words and the verified flag-byte table, preserving padding `+10/+11`. It must
be separate from descriptor-cursor selection, normalization, native color
resolution, actual-format admission, and renderer integration. The finite
format/payload conditions above must survive that composition.

This document does not establish a live device, actual texture resource,
resource dimensions, source-style color, selected runtime mesh options, or
final pixel/UV association. No client launch, native draw, GPU run, renderer
enablement, or retail parity check is performed by this study.
