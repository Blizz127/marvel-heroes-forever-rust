# Texture resource construction and dimension effects

**VERIFIED static native operations; OBSERVED saved declarations.** This
continues [SCALE9_TEXTURE_LOAD.md](SCALE9_TEXTURE_LOAD.md). The selected resource
constructor and ordinary update helpers are now bounded directly. Their local
operations distinguish the Texture2D object, its new resource, mip descriptors
and payload allocations. The continuing manager callbacks receive the texture
object and therefore remain explicit limits on a dimension-preservation claim.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-texture-resource/`.
This increment contains documentation only. No native client, network, server,
GPU, renderer option or default was run or changed.

## Constructor receiver and actual call arguments

The previously proved concrete Texture2D resource factory `0x1409ac880`
selects the constructor at `0x1409acd9b`. On its allocation-success branch,
it supplies:

| Argument | Exact source |
|---|---|
| RCX | the newly allocated 0x17c-byte resource R |
| RDX | the original Texture2D object O |
| R8D | dword O+0x1b4, the selected count B |
| R9 | the factory's stack-local string/path storage |

Constructor `0x1409a04f0..0x1409a08c6` is complete at 982 bytes. It retains
O in R13 and R in RDI, installs resource table `0x141ecb4a8`, and writes
**R+0x5c = O**. It copies object+0x1f4 to resource+0x64 and copies the supplied
string into R+0x70 through `0x1406d3a60`. The latter writes its explicit
destination and a newly allocated copy; the constructor does not pass O as
its source or destination.

The constructor derives R+0x54 from the inverse of O+0x60 bit 0. If O+0x1c0
is zero at its test, it performs a locked decrement at `0x1409a05c7`.
That is the only direct original-object store in this complete constructor.
It sets **R+0x6c = i32[O+0x104] - B** and initializes fourteen resource
payload-pointer slots starting at R+0x80. This subtraction and count are
native operations; no mip-clamping policy is substituted for them.

The complete body has no direct O+0x180..0x18f reads or stores. This excludes
a local assignment of SizeX, SizeY or OriginalSize by the constructor. It
does not exclude effects through calls, existing aliases or the retained
R+0x5c backpointer.

## Mip and payload branches

For each index 0..13, the constructor clears the corresponding resource
payload pointer. Indices at or above i32[O+0x104] skip mip processing. A
continuing index loads a pointer from the pointer table at O+0xfc. Call this
selected mip descriptor M. The native loop treats O+0x104 as its count;
this does not assert a live array merely from the saved file's one-mip tail.

Two complete leaf predicates are now retained:

| Method | Exact operation on M |
|---|---|
| `0x140749e90..0x140749e9c` | returns `(~(u32[M+8] >> 5)) & 1` |
| `0x14074a2c0..0x14074a2c7` | returns `u32[M+8] & 1` |

For an index below R+0x6c, the optional branch controlled by global
`0x142afd28c` and `0x140749ea0(M)` can obtain then release a temporary
payload. For an index at or above R+0x6c, the first predicate must be true
to continue. A true second predicate calls `0x1413b27c0(O)` and can enter
a diagnostic path. A false second predicate calls `0x140745880` with M,
the current resource payload-slot address, and a boolean obtained from
O+0xc6 !=0x1a.

The complete 286-byte `0x140745880` manipulates M+0x28 and the explicitly
supplied output pointer. Depending on payload availability, flags and that
boolean, it transfers an existing pointer, allocates/copies, releases a
payload, or delegates acquisition to `0x14074b930`. Copy lengths use a
selected mip virtual+0x10 result multiplied by M+0x0c. No Texture2D pointer
is an explicit argument to this helper, but its virtual/acquisition calls
and allocation/alias validity remain conditions; this is not a proof that
all reachable code is effect-free.

A later conversion branch additionally requires O+0x190 equal 3, a selected
global-helper mask, O+0x60 bit 0 and a nonnull resource payload. It reads
M+0x40/+0x44 to size a new four-byte-per-element payload and expands the
old one-byte elements into four repeated bytes. Writes target that new
payload and the resource slot; this local block does not assign the four
Texture2D dimension fields. The numeric format comparisons remain numeric
until the loaded ByteProperty enum mapping is independently established.

The owned Engine declaration now supplies an independent saved-data witness:
`EPixelFormat` export 8466 has 31 ordered names with exact payload exhaustion.
`PF_DXT5` is at position 7, while positions 3 and 0x1a name `PF_G8` and `PF_A1`.
The saved `Texture2D.Format` field references this precise enum through
object reference 8467. The eight-byte `PF_DXT5` name reference is at logical
package offset 3526256, export-relative+76. The stable Engine package SHA256
is `66dd5dca80f275a8cc2de5bf4b03042949cfccd0afc0edc6b7ba9b577cde4f51`.
The saved tooltip's exact enum value is `EPixelFormat.PF_DXT5`. These facts
bind the saved declaration and value; they do not by themselves prove the
runtime ByteProperty's numeric encoding or admit the constructor's numeric
branch exclusions.

## Ordinary update helpers and the remaining receiver

`0x1413b27c0` is a complete 82-byte predicate. Its return 1 path requires O
nonnull, O+0x194 mask 2 set, O+0x60 mask 0x100 clear, O+0x104 equal 1, and
the qword at O+0x19c equal the qword returned by `0x1413b2f20`. The latter
receives a stack address, not O. The predicate's local operations only read
O. The resource factory clears qword O+0x1b4 when it returns1, changing
both adjacent count fields; the constructor uses the same predicate on its
conditional mip branch.

The operand producer `0x1413b2f20` is complete at 97 bytes. It tests bit 0 of
global `0x142c4c708`; on the initialization path it sets that bit **before**
calling `0x140793e60` with destination `0x142c4c700`, the exact UTF16 literal
`Icons`, and R8D=R9D=1. Both paths copy the cached full 64-bit value to the
supplied output. This identifies the intended literal, but not a universal
numeric FName identity, initialization success or the current cache contents.
The saved tooltip names its cache `textures`; earlier PostLoad code can change
O+0x19c, so the saved spelling alone does not settle this later comparison.

The subsequent `0x1409c66a0` is complete at 161 bytes. With global
`0x142aee960` zero and no object in O's +0x40 outer chain having flags
mask 0x600, it invokes **virtual+0x70 on the manager at `0x142c2bf80`, passing
O in RDX**. It also unlinks O's node referenced at+0x1d4, adjusts neighboring
list pointers and globals, and clears O+0x1dc/+0x1e4. Its direct stores are
distinct from the four dimension fields. The manager call's effects are not.

The factory's conditional `0x1409b8b30` is complete at 159 bytes. It links
O and the node at+0x1d4 into the global list and adjusts its count. Under
the same global/outer-chain gates, it tail-calls **manager virtual+0x68 with
O supplied**. These two exact callbacks are the next object-receiving
preservation boundary. The selected current manager implementation has not
been identified here; assuming generic registration behavior would discard
an actual native dependency.

The resource-update poll `0x1409c6f40`, complete at 256 bytes, also matters
before construction. Its already recovered caller passes EDX=0. Equal
O+0x1b8/+0x1b4 returns0. State O+0x1c0 equal 0x64 (decimal 100) or 2 calls respectively
`0x1409a9080` or `0x1409a8f80` with resource O+0xd8 and returns1. In state 0,
O+0x194 mask 4 or a nonnull resource with signed dword+0x178 >0 selects
O+0x1b4 = O+0x1b8 and clears mask 4. Otherwise it copies O+0x1b4 to O+0x1b8.
Both return 0; other nonzero states return 1. There are no direct dimension-field accesses
in this body. The selected resource callbacks can reach O through the
constructor-proved R+0x5c backpointer and therefore remain distinct effects.

## The earlier metadata calls are bounded separately

The object-receiving `0x14079bab0` follows O+0x40 to the outermost object and
checks that object's class chain against a lazily resolved `Core.Package`.
Exact owned `Core` and `Package` literals and the class builder establish
that role. On the successful initialized profile it returns that outermost
object. It has no direct receiver/root writes; lazy class registration and
the explicit failed-cast diagnostic keep their own call boundaries.

`0x14079c930`, its bounded path builder `0x14079c860`, and the selected name
formatter construct a counted name/path string in the supplied output.
They read object names/outer links, not the four texture dimension fields.
The output is not a texture GUID. The caller supplies stack-local output
separate from O, a stop object or null, and must provide valid finite
outer/name-pool state. String append, conversion and registration callees
are not silently declared side-effect-free.

The already retained `0x1413f2e60` caller is itself mutable: after successful
lookup using the computed key it writes O+0x19c at `0x1413f3046` and can
invoke `0x140744940` on each selected mip at `0x1413f30bc`. Therefore the
metadata helpers' local read behavior does not promote that whole PostLoad
branch to read-only or preserve the original saved cache-name value.

## Scope of preservation

For a valid successful allocation with disjoint object/resource/mip/payload
storage, the enumerated direct writes do not overlap O+0x180..0x18f. This
is a local store-set conclusion. It is not a whole-constructor or whole-load
theorem about every callback, allocator, acquisition routine or shared object.

The static renderer input therefore remains the exact saved property tuple
until the selected live/simulated load profile accounts for these effects.
The prior version 868 producer excludes the two old-version serializer writes
under its conditions, and this increment removes the resource constructor
as an unexplained local dimension assignment. It also replaces a generic
"update helper" gap with exact manager methods and resource callbacks that
can carry O onward. Queue membership, class reorder, current flags, manager
selection and resource lifetime retain their separate gates.

No 64x64 runtime default, selected mip fallback, algebraic matrix cancellation
or renderer input adapter follows from equal saved values or from the local
absence of dimension stores.

## Source ledger and validation

The primary native source is the owner's private build 1.52.0.1700 executable,
SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It and the owned packages are proprietary compatibility-study inputs.
No game bytes or outside implementation are committed. Existing original
project findings supplied the precise factory, wrapper and field-layout
anchors. The REA skill supplied evidence procedure only; target-free finite
Rizin reads are checked against the exact unchanged PE mapping.

Constructor/payload canonical record:
`ev_10351e92949006176615b107e9e41504ecae1b622e3e457ddb84f8b9a7e17529`.
It includes the constructor, supplied-string copy, both exact predicate
leaves and the selected payload transfer/acquisition helper. Discovery probes
that included padding or neighboring functions are not substituted for the
final exact leaf spans.

The other new canonical records are
`ev_a53a5642e4a50450eb5aee976e47420ac458ce80bf01a1da0e3c47924655a391`
for the finite package/path/name helpers and exact literals, and
`ev_bd51e2623898d4bd3de49a12c7120eee8319bc2b0d8b8b7aa7aa9dd52166cddb`
for the five update/predicate methods and `Icons` literal.

Focused independent validation covers exactly three new canonical records:
**16 instruction arrays, 868 instructions / 3,349 code bytes**, plus
**10 data arrays / 150 bytes**. Every requested range exhausts exactly and
matches the unchanged pinned executable; captures are complete and
untruncated. Receipt
`verification/runs/0b31671a6da918f36b81/capture-validation.json` has SHA256
`2d4670e8a3707cb570d839a1812968bd70677c089994460768b9020d635bdaa0`.

Supporting private receipts:

- `update-helpers/update-helper-receipt.json`: SHA256
  `08c8e0b0ce1373d8ac293ce7e7ed3088f28c5f4713e7d966764eaaca27ff12c4`.
- `metadata-helpers/candidate-receipt.json`: SHA256
  `115776394522bac415ec1dce1831d411591b0e574f581752dde349e12c5042e6`.
- `owned-format/receipt.json`: SHA256
  `34df3c2655a7e6495a4594d9ccd683f7ad679de1a3ecfc18ca2792d44693d3f7`.

The saved enum's fresh extraction matches the retained original-reader
payloads and ends at the exact native-tail boundary. This is owned-asset
validation, not live enum resolution. The final evidence index retains the
complete source ledger, canonical records and cumulative bundle separately
from the repository. No runtime or pixel-parity test was performed.
