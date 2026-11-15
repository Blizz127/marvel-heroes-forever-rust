# Renderer initialization and the option-construction predicate

**VERIFIED native writes:** `1202f50` returns with renderer DWORD `+0x10`
equal to zero. The concrete backend initializer later sets bit 0 on its
success path, then invokes listeners. The engine ignores its returned failure
status and can proceed to option-owner construction after initialization fails.
The constructor's zero therefore cannot establish the predicate at the later
option call, or stand in for a usable initialized renderer.

**The unconditional bit at the final engine call remains unproved.** Resource
and listener effects, queued execution, cleanup objects and current renderer
identity are genuine intervening conditions. This document closes the explicit
producers, a bounded failure profile, selected common-call behavior and the
concrete successful manager query. It does not replace those remaining
conditions with a fresh-object default. No rendering options or code change.

## Objects and exact construction checkpoint

R is the renderer constructed by `141202f50`; E is the enclosing argument of
engine initializer `141273a20`. Q is the wrapper constructed by `031c20`,
whose separate option owner A is constructed by `071960`. The actual P used
by A is loaded from current `E+0xec` at `141273bea`. This document concerns
E_init; the later E_draw association in
[SCALE9_OPTIONS_MESH_IDENTITY.md](SCALE9_OPTIONS_MESH_IDENTITY.md) remains separate.

The complete constructor and its normal-path member-construction closure
establish these fields without assuming allocator zero-fill:

| R field | Constructor-return value |
| --- | --- |
| primary table | `1421ba230` |
| DWORD `+0x08` | 1 |
| DWORD `+0x10` | 0 |
| DWORD `+0x14` | 3 |
| QWORD `+0x18`, `+0x20` | both point to sentinel `R+0x10` |
| QWORD `+0x28` | NULL |
| QWORD `+0x68` | supplied constructor argument |
| QWORD `+0x4590` | NULL |

Base `042f60` explicitly zeros the status DWORD at `042f93`. Its later
member helpers and `1202f50`'s cache, context and vector initialization have
bounded fixed footprints preserving that field and the empty listener links.
The private receipt includes complete normal-path vector construction and
excludes exceptional unwinding; this is stronger than a prefix-store claim.

The engine supplies the `+0x68` argument by allocating eight bytes, installing
table `1421c3368`, or using NULL on allocation failure. It is an interface
of that provenance, not an inferred heap or an alias of R. The engine then
releases its prior `E+0xec` value when nonnull and stores R there. Old-object
release, valid allocation/lifetimes and any concurrent mutation remain
separate from the proved new-renderer construction checkpoint.

## Actual backend call and parameter stores

For concrete table `1421ba230`, virtual `+0x250` is `141253a50`, all 855
bytes. The flag-zero branch of `141273a20` calls it at `141273b9a`, using
the renderer just loaded from current `E+0xec`.

The caller explicitly writes QWORD parameter `+0`, DWORD `+8`, XMMWORD
`+0x10` and QWORD `+0x20` to zero, and DWORD `+0x28` to `0x100`.
Consequently optional manager `+0x18` is zero at the call checkpoint.
Padding `+0x0c..+0x0f` is not initialized by these instructions. These are
actual stores, not external configuration defaults or a whole-struct memset.

The engine temporarily retains the renderer, calls the initializer, releases
selected temporary fields and its temporary renderer reference, then continues.
It **does not test returned AL**. Reference/lifetime and parameter-field
preservation are necessary when composing this interval; ignoring failure
does not certify successful initialization.

## Failure and success are different native profiles

| Selected condition | Local result | Supported status conclusion |
| --- | --- | --- |
| `042520` returns false | Early AL=0 return; no manager installation or local status store | A bounded carried-fresh profile below preserves bit 0 clear and manager NULL |
| Cache setup `253db0` returns false | AL=0 after shader/resource work; no local status store | Failure alone does not determine effects of the reached resource callbacks |
| Constructed manager is NULL | Stores NULL at `R+0x4590`, performs cache cleanup, returns AL=0 | NULL is exact at that store; later cleanup and earlier callbacks remain conditions |
| Success reaches `141253d83` | `OR DWORD[R+0x10],1`, listener notification, then AL=1 | Bit 0 is exactly set after the OR; return-value carry depends on actual listeners |

The bounded early failure follows real emitted inputs: `042520` constructs
an auxiliary object through R virtual `+0x10 -> 125b870`, populates its
optional resource, copies parameter fields and obtains the current thread ID
through the exact ordinary system import when needed. Final `043220` requests
0x1000 bytes for the actual `0x100` queue entries. A NULL queue allocation
returns false. Under the retained fresh fields, successful usable auxiliary
construction, ordinary reference/free/import behavior and valid separate
storage, this path preserves the clear status bit and NULL manager.

That is an initialization-failure profile. Since the engine ignores AL, it
can still reach `031c20` if subsequent operations return normally. The
clear-predicate preservation result in
[SCALE9_OPTIONS_CONSTRUCTION.md](SCALE9_OPTIONS_CONSTRUCTION.md) must not be
presented as evidence that a successfully initialized backend normally has
this bit clear.

On success, complete `125b8f0` starts at current `R+0x20`, walks to sentinel
`R+0x10`, and invokes each node's virtual `+0x08` with second argument zero.
An actually empty list invokes nothing; a nonempty list invokes real code.
The constructor supplies an empty-list checkpoint, but shader/cache/resource
work before this notification has not been proved universally to preserve it.
The explicit status OR is after that earlier work; it is before the listeners.
Thus neither the prefix zero nor the success return alone proves the final bit.

## The flag-nonzero route publishes an initialization record

The branch on global `142c27978` is not established as a debug-renderer
replacement. Complete `125e3a0` constructs a 16-byte record, reads the current
renderer through the supplied address of `E+0xec`, retains it when nonnull,
reloads the field and stores that pointer at record `+8`. It does not write
`E+0xec` or renderer `+0x10`. The retain and reload are distinct operations;
arbitrary mutation between them is not silently excluded.

The installed record table `1421c3398` has execution slot `+8 -> 14126a950`.
Its complete body loads the stored renderer, emits the same parameter fields
as the immediate branch, invokes that renderer's virtual `+0x250`, cleans
temporary references and returns 16. The separate record destructor releases
the stored renderer. Both branches can therefore reach the same concrete
initializer when their actual identities and execution conditions hold.

Reservation helper `6e18b0` is call-free; commit helper `6e3ee0` publishes
queue state and reaches dynamic factory/notification interfaces. It can call
global factory virtual `+8`, reload its owner pointer, store the returned
interface, then tail-call that interface's virtual `+0x10`. This is not
direct execution of the record and is not proved inert with respect to all
engine/renderer state. Concrete worker dispatch, ordering, interface identities
and identity carry remain explicit. No queue completion or mode value has
been observed by executing the client.

## Common fence and cleanup before wrapper construction

Both engine branches reach `095b3b0` before allocating Q and loading P from
current `E+0xec`. That complete function snapshots and clears global
`142c27958`, initializes a stack counter to zero, calls `958930`, waits through
`967b90(counter,0)`, processes the saved cleanup list, and invokes `1330c60`
for another cleanup list at global `142c4ac90`.

`958930` atomically increments the counter. If its actual mode flag is zero,
it immediately decrements it again without publishing a command. Under
ordinary import behavior, the wait helper consequently sees zero and skips
its callback loop. Its remaining system calls are the exact
`KERNEL32!QueryPerformanceCounter` import, plus timing/statistics stores.

If both actual cleanup-list reads are NULL, and the stated stack/global/import
and no-concurrent-mutation conditions hold, this immediate common-call profile
preserves the renderer bit/header and `E+0xec`. **No initial NULL value is
assumed for either list.** A nonnull list reaches `956c50`, which invokes
each element's virtual slot zero before freeing storage. Their effects need
actual element identities. The wait loop, when reached, also has real dynamic
system, diagnostic and message/poll calls.

In the nonzero mode, `958930` instead publishes a record whose native name is
`FenceCommand`. Exact table `141eab230` selects `95adf0` for execution: that
call-free leaf atomically decrements the DWORD through record `+8` and returns
16 without testing the result. This proves the counter operation, not FIFO
worker behavior or completion of the earlier initialization record. Those
ordering conditions cannot be inferred merely because a fence record exists.

After this common call, the engine obtains P by rereading `E+0xec`. No inspected
record constructor directly replaces that field; resource, listener, publication
and cleanup callbacks still prevent a universal no-replacement claim. A proof
about original R cannot automatically be applied to a different current P.
The final wrapper/owner allocations and constructor prefix also require their
existing valid-storage, dispatch and preservation conditions.

## Follow the set-bit path into the concrete manager

For the actual preserved zero optional-manager parameter, successful backend
setup allocates 0x1d8 bytes, calls `125e8e0(M,[R+0x68])` and stores the result
at `R+0x4590`. The complete constructor installs primary table `1421c2f50`
and builds a format-provider list. Successful allocation, original table/list
bindings and carry are required; table installation alone does not certify
every entry or later callback effect.

For this concrete manager, the query used by `114250(manager,9)` is virtual
`+0x18 -> 14126e930`. The constructed format list includes key 9 under its
normal successful allocation/row-preservation profile; missing membership
returns zero. The matched query reads global `142c278a4` twice, separately.
`114250` tests **signed AL**, not signed EAX, before testing bit 5:

| Key 9 found | First read | Second read | EAX / AL | `X+0x11e0` mode |
| --- | --- | --- | --- | --- |
| yes | zero | zero | `0x1b2 / 0xb2` | 0 |
| yes | nonzero | zero | `0x190 / 0x90` | 0 |
| yes | zero | nonzero | `0x132 / 0x32` | 1 |
| yes | nonzero | nonzero | `0x110 / 0x10` | 2 |
| no | either | not read | `0 / 0` | 2 |

Mode 1 reaches the optional `038f70` image creation at `11440a`; the other
rows skip that call. All rows still reach the later per-record `113e60`
and cache work. An unchanged zero global selects mode 0; an unchanged nonzero
global selects mode 2. The mixed-read mode is not a chosen stable default.
No live global value, image success or whole `114250` preservation theorem
is inferred from this exact branch mapping.

## Remaining closure and evidence

The native-success profile requires the actual receiver/header/lifetime through
resource setup, the reached status OR, preservation by final listeners and
temporary cleanup, applicable common-call/queue conditions, unchanged selected
P, and ordinary wrapper-prefix behavior. With these actual conditions, the
option helper reads a set bit and takes `114250`. This is a conditional
composition, not an unconditional startup witness derived from constructor zero.

The genuine remaining resource frontier includes shader dispatch helpers
`124c1e0`/`1247570`, cache/backend creation and failure cleanup, resource-handle
interfaces, current listener contents, pending cleanup objects and worker
dispatch/order. Capturing their callers does not close all transitive effects.
No callback is treated as pure merely because it is called a getter or allocator.

Private root: `evidence/scale9-options-engine-profile/`. The main validator
checks 198 instructions / 797 bytes and eight import bytes, including all
three wait-helper unwind fragments. Separate receipts bind full constructor,
backend, manager, record and fence coverage; overlapping reused counts are
not added as unique executable coverage. `receipt.json` and `MANIFEST.json`
bind this document and the exact source/evidence hashes. All selected new
REA records completed with exit zero, untruncated output and unchanged pinned
hashes; the ledgers were exported and reset without a retained target.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary private installation | Exact constructor/status writes, engine branches, imports, records and query |
| Retained original `scale9-options-construction`, `scale9-options-mesh-identity`, `scale9-output`, `scale9-draw-mesh-identity`, texture-format and frontend-image-context ledgers | Original project investigation of that owner artifact | Reused complete callers, table bindings and manager/resource evidence |
| New private `renderer-construction`, `backend-initialization`, `debug-engine-path` and `fence-record` ledgers | Original project investigation; the older debug directory name is not a semantic conclusion | Finite callees, selected profiles and explicit callback frontiers |
| Installed REA skill/native/recovery guidance, REA 6.9.0, Rizin 0.9.1 and local objdump | MIT / LGPL-3.0 / GPL-3.0 tooling respectively | Bounded target-free static analysis; no runtime instrumentation |
| Python standard library and original PE/unwind helpers | PSF tooling / original project code | Exact-byte, boundary, import and hash checks |

No game process, VM, GPU, account, server, source configuration or native
prompt was exercised. No proprietary listing or asset enters git. This
evidence changes neither Original nor Forever behavior and makes no retail
visual, wire or lifecycle parity claim.
