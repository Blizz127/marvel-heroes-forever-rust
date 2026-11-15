# Scale9 texture loading: archive version and PostLoad

**VERIFIED static native dispatch and operations; OBSERVED owned package bytes.**
This continues [SCALE9_LOADED_LAYOUT.md](SCALE9_LOADED_LAYOUT.md) and
[SCALE9_TEXTURE_INPUTS.md](SCALE9_TEXTURE_INPUTS.md). The fresh loader now has an
exact producer for archive+0x08: the low half of the selected package summary's
packed version. A conditional PostLoad route and the concrete Texture2D handler
are also identified. This closes the previous untraced version assignment; it
does not establish successful live loading or preservation through every call.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-texture-load/`.
No native client, network, server or GPU was run. No renderer flag, product
implementation or image-size default changes are part of this increment.

## Fresh loader and archive identity

The already recovered custom image creator requests `Engine.Texture` through
`0x1407b7710` before passing its returned object to `0x141268d60`. The complete
3,218-byte load function distinguishes a cached-found return from its fresh
continuation. A successful cached lookup at `0x1407b7894` can jump to
`0x1407b8372`; it must not inherit a fresh-load proof merely because the same
public entry was called.

The selected fresh continuation calls `0x14079bcd0` at `0x1407b7927`.
That function can reuse an existing linker. Its fresh factory call at
`0x14079c4a5` selects `0x140762760`, which calls `0x1407627c0`, temporarily
publishes the returned linker and invokes `0x140772a90` with zero timing mode.
The factory itself first searches for an existing linker whose +0x60 equals
the incoming package. Only its allocation branch constructs a new 0x7cc-byte
object through `0x14075a490`.

Call the resulting linker **L** and its embedded archive **A = L + 0x1c8**.
The base constructor `0x14075a300` stores the supplied package at L+0x60;
the package-summary storage begins at **S = L + 0x68**. Constructor
`0x14075a490` installs primary table `0x141dfd398` at L and archive table
`0x141dfd600` at A. Archive virtual+0x48 is the exact eight-byte leaf
`0x14076b5e0`, which returns A-0x1c8.

Constructor reads of globals `0x14292cd70`, `0x14292cd6c` and `0x14292cd78`
initialize A+0x08, +0x0c and +0x10. Their executable-file data is respectively
868, 3078 and 3. These are initial data observations, not evidence that runtime
globals are immutable or that a later loaded version still comes from them.

The constructor zeroes A+0x14 and +0x18 together. It also explicitly zeroes
A+0x1c/+0x20, A+0x48/+0x4c and A+0x70/+0x74, using qword stores at
`0x14075a4fd`, `0x14075a537` and `0x14075a55e`. These checkpoints are useful
to the separate saved-file preamble investigation. They do not certify those
flags after arbitrary callbacks or on an existing-linker branch.

## Header read and exact version assignment

The stage driver `0x140772a90` skips completed linkers when L+0x690 is
nonzero. Otherwise its first two stages are `0x140762900`, then
`0x140771a40`; a zero stage result prevents progression to the next stage.
For the supplied zero timing mode it retries incomplete work. Later stages
remain distinct operations; this is not an exhaustive side-effect audit of
the entire loader.

The first stage's fresh backing-archive path requires L+0x668 initially zero.
Its continuing initialization reassigns the version globals to A+0x08/+0x10
at `0x14076358e` and `0x14076359b`, then **sets A+0x14 to 1** at
`0x1407635a2`. Existing nonnull backing storage jumps to `0x140763603`,
bypassing those writes. Thus the previous class-serializer loading gate has
a concrete producer on this branch; it is not a guessed fresh archive flag.

The summary stage `0x140771a40` skips its read when L+0x684 is nonzero.
On its read path, `0x140771a7a` calls `0x14075beb0(A, S)`. That complete
summary serializer reads four bytes into S through `0x1406d66b0` and checks
the magic. Normal magic `0x9e2a83c1` proceeds directly. Swapped magic
`0xc1832a9e` normalizes S and toggles A+0x4c. An invalid magic exits the
serializer; its contents must not be admitted as a successful header.
At `0x14075bf16`, a second four-byte read writes **S+0x04**, the packed
version word. The next four-byte read targets S+0x08.

The byte-read forwarding is concrete: archive table+0x08 at `0x141dfd608`
selects `0x1407708b0..0x1407708be`. This 14-byte leaf changes the receiver
to `[A+0x4a0] = [L+0x668]`, loads its table and tail-jumps to backing
virtual+0x08. Destination RDX and length R8 are unchanged. It has no local
cursor, swap, null or success handling. The previously retained `0x1406d66b0`
calls that virtual once for four bytes when A+0x4c is zero, or four times in
reverse destination order when nonzero. Current backing implementation,
cursor and read success remain supplied conditions.

After this call, the summary stage performs these exact assignments:

| Native instruction | Destination | Source operation |
|---|---|---|
| `0x140771aca` | backing archive +0x08 | zero-extended word at L+0x6c |
| `0x140771ad9` | backing archive +0x10 | zero-extended word at L+0x6e |
| `0x140771ae3` | **A+0x08 = L+0x1d0** | low16 of dword at L+0x6c |
| `0x140771af0` | **A+0x10 = L+0x1d8** | arithmetic shift right16, then low16 |

L+0x6c is exactly S+0x04. A later package-flag branch can replace the backing
archive pointer at L+0x668; that pointer is distinct from embedded A. Neither
the constructor's defaults nor the replacement backing object substitutes
for the explicit assignments to A.

The owned `MarvelFrontEnd_SF.upk` has a stable complete-file SHA256 of
`af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190`
over 8,905,511 bytes. Its first 12 stored bytes independently decode to normal
magic, packed version **0x00030364**, and raw summary-header size 75308.
The version's low16 is **868** and high16 is **3**. Byte-level, u16 and
mask/shift decoders agree. The exact twelve-byte witness remains private;
`header-witness/receipt.json` has SHA256
`18f3b941216fa006cd23a59c5b8e24b7fc5ecc779d988037bd07e228736b5c35`.

Given a successful normal-order header read from that selected owned package,
the native summary assignments therefore produce **A+0x08=868** and
**A+0x10=3**. This conditional derivation connects the metadata to the native
archive field; it does not report a live file-open, cursor, decompression or
load-success observation.

## The same archive reaches object serialization

Archive table+0x20 at `0x141dfd620` selects `0x14076d640`. It retains incoming
A, computes L=A-0x1c8, and compares the selected object's +0x28 against L.
The previously derived Core.Object declaration identifies that field as
Linker. A nonmatching nonnull linker redirects to its own archive; such an
object must use that archive's version, not the version of this L.

On the same-linker branch, object flags mask 0x20000000000 must survive the
method's initial and repeated tests. The method clears that mask before
serialization. With object flag 0x200 clear, `0x14076d831` calls object
virtual+0x68 with RCX=object and RDX=the retained A for nonnull L. The separate
flag 0x200 branch uses class serialization through `0x140752720` and is not
silently treated as ordinary Texture2D serialization.

The already proved concrete Texture2D table+0x68 selects `0x1409c07e0`.
Consequently, if this same A retains the summary-assigned version through
the selected call, its two old-version comparisons are both false:
**868 < 0x273 (627)** and **868 < 0x27a (634)**. The local Size-to-OriginalSize
copies at `0x1409c08ef`/`0x1409c08fb` are skipped. This conclusion uses the
archive producer and exact signed comparisons; the saved equal 64 values are
not used to decide whether an overwrite occurred.

This does not remove the prior Link conditions. The selected class serializer
requests mode 1 only with archive+0x14 nonzero. The native reorder/relink block
still depends on class object flag bit 21, that loading flag and nonzero mode.
It can change Children/Next and recompute offsets. Saved declaration order,
successful superclass/property loading and any later qualified nested-child
writes remain explicit inputs to the field-layout derivation.

## Concrete PostLoad dispatch

The fresh load function calls `0x140798310` at `0x1407b8015`. This is a global
completion dispatcher, not a direct PostLoad call on the object returned by
the public load entry. It first decrements global `0x142ba3af8`; processing
requires zero depth and nonempty pending work. A local list drawn from
`0x142c23b68`/`0x142c23b70` is visited through `0x140797020` at
`0x140798867`. The selected Texture2D's inclusion in that list remains a
separate queue-production condition.

`0x140797020` requires object flag mask 0x0100000000000000. It clears that
mask and marker 0x8, performs dependency work, calls object virtual+0x40 at
`0x1407970c7`, then checks marker 0x8. Its owned diagnostic explicitly names
PostLoad and the required Super::PostLoad call. The base table's +0x40 slot
selects `0x1407a2440`, whose direct operation sets marker 0x8. This grounds
the slot's role in the binary rather than a remembered engine vtable.

Concrete Texture2D table `0x141ecbe70` has +0x40 at `0x141ecbeb0`, selecting
the complete 72-byte handler `0x1409bb070`. It:

1. Calls `0x1413f2e60` with the object.
2. With object flag 0x800000000000 clear, stores a decremented/clamped count
   derived from object+0x104 into object+0x1f0.
3. Calls `0x1413a64d0`, then tail-calls `0x1409bb0c0`.

These reviewed bodies contain no direct stores to object+0x180..0x18f.
That is a bounded direct-store observation, not a preservation theorem.
The first helper returns without calls when global `0x142c4c6a8` or
object+0x19c is zero. Its active branch passes the object to `0x14079bab0`
and `0x14079c930`, and selected mip pointers to other helpers. Their
side effects and aliasing cannot be discarded.

`0x1413a64d0` tests the original object's **byte+0x60**, mask1, and format
byte+0x190; it does not read object+0xc60. This corrects an intermediate
message transcription, not the retained instruction bytes or final receipt.
The Texture base continuation `0x1409bb0c0` calls base PostLoad, changes
byte+0xc8/dword+0x60 and conditionally +0xd0, then can tail-call object
virtual+0x278. The base PostLoad itself can call `0x14079de30` and
`0x14079f4e0` under its class/object/global gates. No whole-callgraph absence
of dimension writes is claimed.

The selected +0x278 slot resolves to `0x1409c6e60`. Its complete 80-byte
body returns for format byte+0x190 equal 0x1a. Otherwise it polls object
virtual+0x2c8 with EDX=0 while the return is 1, and tail-calls `0x1409c6eb0`.
That 130-byte continuation releases/resets resource+0xd8 and, under its
global/object/helper gates, calls virtual+0x280 and stores the result back
at+0xd8. The exact Texture2D +0x280 slot selects `0x1409ac880`, a complete
1,422-byte resource factory.

The immediate factory contains no direct original-object +0x180..0x18f
accesses. It does pass that object to `0x14079bab0`, the resource constructor
`0x1409a04f0`, `0x1413b27c0`, `0x1409c66a0` and conditional `0x1409b8b30`,
and selected mip storage to `0x14074ee40`. Those concrete effects and the
poll target `0x1409c6f40` remain unexpanded here. A resource constructor may
consume dimensions even when its caller does not; no selected mip, backing
size or preservation conclusion is inferred from that absence.

## Remaining input boundary

The native header-to-version assignment is now grounded, including archive
identity and the conditional object-serialization handoff. For the owned
version 868 input, the two local old-version Texture2D branches are excluded
under the stated same-archive conditions. The first fresh loader stage also
provides the class serializer's nonzero loading flag.

The renderer still needs a justified object state at the wrapper's actual
reads. In particular, existing-linker/cache branches, later class reorder,
object queue membership/dependency work, active PostLoad helper branches,
the resource constructor/update calls, and current image/resource lifetime
are not replaced with fresh defaults. The static saved tooltip's four 64
values remain exact saved fields. They are not yet promoted to unconditional
live `Q` dimensions or a silently cancelled texture matrix ratio.

The next useful dimension-specific target is the concrete resource
constructor `0x1409a04f0`, reached with the Texture2D pointer, together with
the already identified conditional helpers that can receive or alias that
object. This names a necessary continuation rather than introducing a
generic engine-layout investigation. No product input adapter is enabled by
this document.

## Evidence and source ledger

All new instructions and table entries come from the owner's privately pinned
`MarvelHeroesOmega.exe`, build1.52.0.1700, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It and the owned package are proprietary compatibility-study inputs; no bytes
from either are committed. No outside implementation or generic engine SDK
was consulted for the new behavior.

The original offline navigation helper checks PE section mapping and unwind
boundaries. Canonical evidence uses target-free REA/Rizin with finite byte
ranges; independent validation compares every returned instruction/data byte
to the unchanged pin. REA's string-scan size warnings do not replace that
byte verification. The REA skill at
`~/.agents/skills/reverse-engineer-anything/SKILL.md` supplied tool/evidence
procedure only. Prior project documentation and private receipts supplied the
already recovered image creator, wrapper, Texture2D serializer, layout and
four-byte archive helper; no external source code was copied.

New canonical records:

- `ev_e8ddff4a19fdd1563958b66192001b765e38bda1c986e78480bc9d671252de54`:
  fresh load, linker lookup/factory driver, summary stage and summary parser.
- `ev_9e0083a65f2853e77fe0d86f918d23a86b86e19777744c764828719c817fde5d`:
  archive construction, same-archive serialization, fresh loading-flag stage,
  exact archive slots and constructor globals.
- `ev_a8a812e8979990f9c6a97e7d58e6a355595bbb51be2190b48d7353d5f4b5e5da`:
  completion dispatcher, conditional PostLoad caller, concrete handlers and
  exact dispatch slots/diagnostic.
- `ev_6e85fab3fc48435ea2510f97161a2f71d3cc0f4362dfe3b26951f8a49fd713af`:
  selected resource-update/factory continuation, archive read forwarding and
  exact slots.

The archived construction receipt is
`archive-construction/candidate-receipt.json`, SHA256
`46ced33ddb54a3d60369cb8b80e7af0c30357717bf871397de1ef55995d6c455`.
The first PostLoad receipt is `postload/postload-receipt.json`, SHA256
`f7ab286a959697b6a61bc4947e4c48703e1c8901dc0e461e681b34e27758f610`.
These preserve branch conditions and distinguish offline discovery from the
subsequent canonical REA captures.

The additive resource continuation receipt is
`postload/continuation/continuation-receipt.json`, SHA256
`b1cc611fd0baa70999bcbbbde025f7a76eb5d8f407376cdd06558df3d1ecac45`;
the read callback receipt is
`archive-construction/read-callback/candidate-receipt.json`, SHA256
`b72eb4bf031512d964d3fe8552e008d5b5596adf0a75b7f64c00e5a335b14347`.

Focused final validation covers exactly **four new canonical records**:
23 instruction arrays, **5,302 instructions / 22,252 code bytes**, plus
12 data arrays / **282 bytes**. All requested ranges exhaust exactly and
match the pinned executable. All four captures are complete, untruncated
and unchanged. The independent receipt
`verification/runs/7913c7a80f37ec96b781/capture-validation.json` has SHA256
`c59282f850313047f2bae7d09c82f1371b8e73cd9b34f1458fe06e829864af99`.
The exported cumulative `native.bundle.json` contains 77 records and zero
unknown records. REA was closed after export. These are static compatibility
evidence checks; no runtime or pixel parity was tested.
