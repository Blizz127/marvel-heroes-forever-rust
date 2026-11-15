# Native option-owner construction and publication

**VERIFIED conditional static result:** the eighteen words established in
[SCALE9_OPTIONS.md](SCALE9_OPTIONS.md) survive to the normal return of
`0x140071960` on the concrete parent-predicate-clear profile below. This
extends the earlier immediate-store checkpoint; it does not establish current
options, all constructor branches, or the identity of a mesh's live option
owner. No product defaults, renderer paths or presets change here.

The predicate-set path performs real backend setup. Its remaining callback
frontier is recorded separately rather than covered by an assumed ordinary
allocation contract. One helper REA request has unknown provider completion;
all its requested instruction bytes were independently verified, and the
unknown status remains in the receipt. No native client was executed.

## Objects and checkpoints

The notation in this document deliberately separates these objects:

| Name | Exact role |
| --- | --- |
| A | Destination of constructor `071960`; protected interval `A+[0xcd8,0xd20)` |
| P | Its original second argument; retained and stored at `A+0x50` |
| H | Heap obtained through the selected global heap's lookup operation |
| B | Distinct 0x60-byte allocation eventually stored at `A+0xdd0` |
| X | Distinct 0x1c98-byte allocation eventually stored at `A+0xdd8` |
| Q | 0x18-byte wrapper constructed by `031c20`; its `+0x10` publishes A |
| M | New ordinary mesh context whose `+0x50` stores its supplied option owner |
| R | Engine renderer constructed by `141202f50`, with primary table `1421ba230` |

R's renderer cache at `R+0x314` and rendering context at `R+0x508` are not A.
Matching offsets or describing two objects as renderers cannot equate them.
The proven conditional predicate uses P's concrete primary table; the engine
producer supplies a separate, qualified route from R to that P.

The complete 935-byte constructor establishes all eighteen option words at
`071a25..071acf`; the following instruction starts at `071ad9`. The private
inventory retains their exact bits. The earlier selected tolerance and width
values therefore remain unchanged at constructor return in the profile below.
This document adds no speculative names for the other option words.

The initial retain, workspace constructors `070bf0` and `070ce0`, and heap
lookups precede these stores. The preservation argument starts after their
normal completion and the actual stores, with valid A/P/heap/stack objects.
It does not claim that every possible prefix, allocation failure or corrupt
parent reaches that checkpoint successfully.

## Preservation after the option stores

The subsequent direct stores initialize the cache beginning at `A+0xd20`,
the inline object at `A+0xd48`, two optional pointers, configuration fields,
arrays and list sentinels. None directly overlaps the protected option block.
Indirect writes require the following stronger, explicit alias conditions.

Complete leaf `0451e0` writes only its destination's first 0x88 bytes. With
the actual destination `A+0xd48`, that interval ends immediately before
`A+0xdd0` and is disjoint from the option block. It makes no calls. The
caller then explicitly stores zero at `A+0xdd0` and `A+0xdd8`; these are
actual stores, not an assumed zero-filled allocation.

The constructor inserts its embedded node `A+0x38` into P's intrusive list.
Besides A/P-local stores, it writes through the previous node obtained from
`P+0x18`. Valid, separate list backing is required: an arbitrary predecessor
could alias the options or the freshly initialized optional pointers. The
proof does not dismiss this write because it lacks an A-relative operand.

On a successful separate allocation, complete `071380` initializes B and
passes only `B+0x10` to lock-initialization shim `0682d0`. The latter either
uses a consistent cached initializer or resolves the original imported
`InitializeCriticalSectionAndSpinCount`, with `InitializeCriticalSection`
as its null-function fallback. Its two process-global cache writes are real.
The ready byte is published before the cached pointer, so the admitted
profile requires consistent cache state and no conflicting initialization.
The system calls must return normally with their ordinary designated-storage
effects and no reentry into A; successful lock initialization is not inferred
from a return value that the constructor ignores.

Those finite effects and the stated heap/list/import contracts carry
`A+0xdd0 == NULL` to its old-value test at `071c85`. Thus its old-object
release is skipped before B is published. This is a derived branch under
the profile, not an assumed harmless release callback.

Next, `117720(X,H)` receives the distinct allocation and H; it is not passed
A or P. Its complete body and bounded embedded-constructor helpers initialize
X, build its lookup data and perform scalar math. Its late static-singleton
path can initialize distinct process-static storage and register a future
exit callback. Registration is not invocation of that callback. The contract
requires ordinary scalar-math and CRT/locking/exit-registration behavior,
valid separate global storage, normal return and no reentry into A. It does
not claim blanket process-memory preservation or reverse every CRT routine.

With those footprints, `A+0xdd8` remains its actual initialized NULL until
`071cc1`; its old-object release is also skipped. The caller then publishes
X and calls `114610(X,P,A+0xd20)`. The X constructor stored H at `X+0x58`;
final setup stores P at `X+0x60` and the inline cache pointer at `X+0x68`.
The heap and parent roles are not interchangeable.

## Concrete final predicate and admitted return profile

The final 44-byte helper stores the two association pointers and calls P's
primary virtual slot `+0x20`. For table `1421ba230`, the exact slot is
`141254020`: a complete seven-byte read-only leaf returning
`DWORD[P+0x10] & 1`. It neither writes A nor invokes another routine.

The admitted profile requires that actual table and an actual clear bit at
this call. No constructor seed, saved setting or runtime observation in this
study proves the bit clear. Under that supplied condition, `114610` skips
`114250`, returns, and the outer constructor performs only its register/stack
epilogue before returning A at `071d06`. All eighteen option words survive.

The full profile therefore requires normal completion to the option-store
checkpoint, valid separate objects/list/stack/storage, successful ordinary
heap operations with the supplied identities, the stated lock/CRT/math
contracts, successful X allocation and initialization, concrete parent table
`1421ba230`, and the clear predicate at the final call. No reentrant mutation
or concurrent write to these protected objects is admitted. It does not
require all native bookkeeping to be absent; the documented list, cache,
reference, allocation and process-static effects still occur.

The allocation failures are separate. In particular, failure to allocate X
does not turn final setup into a safe no-op: the caller still passes NULL
to a helper that accesses X. No successful-return result is claimed there.

## Predicate-set frontier

A set predicate calls complete `114250`. The retained chain includes fresh
cleanup, a manager query through P virtual `+0x1c8`, count-dependent setup,
resource/cache preparation and a final P virtual `+0x1d8` association.
Concrete table leaves now establish that `+0x1c8` reads `P+0x4590`, while
`+0x1d8` returns `P+0x510`; the final association consequently writes at
`P+0x518`. These are concrete parent effects, not writes to a fresh X alone.

The concrete `+0xe0` wrapper also allocates a resource and copies its supplied
descriptor. That does not close the returned manager's virtual `+0x18`, all
subsequent image/resource operations or their supplied flags and identities.
The private receipt preserves the exact remaining frontier and the helpers
already checked. No all-path option-preservation theorem is claimed for this
branch, and the runtime predicate is not forced clear to avoid it.

## Engine publication and the remaining mesh-owner join

Complete 134-byte `031c20` receives Q/P, requests a 0xe68-byte allocation,
calls `071960(A,P)` on success and stores its result at `Q+0x10`. Failure
instead stores NULL. This proves the wrapper-to-option-owner construction
and publication edge without relying on object naming.

In complete `141273a20`, engine setup constructs R using `141202f50` and
stores it at the enclosing engine object's `+0xec`. Later, at `141273bea`,
it reloads that field as the second argument of `031c20`, then publishes Q
at the enclosing object's `+0xf4`. Intervening configuration/backend paths
remain a field/header carry obligation: the normal branch invokes R virtual
`+0x250`, and the alternative branch has additional calls. This study does
not silently carry R's identity or status bit across those operations.

Separately, complete `0b0f10` saves its incoming option-owner pointer and
forwards it unchanged to new ordinary context constructor `03bf20`. Its
base `03bda0` stores that argument at `M+0x50`; the source/cache publication
is already recorded in [SCALE9_OPTIONS.md](SCALE9_OPTIONS.md).

The two retained complete direct callers, `0b1220` and `0b1570`, obtain that
argument from their receiver's `+0x28`. Neither body establishes that this
field equals `Q+0x10`. The association producer is still required to join
the actual newly constructed A to M. Raw direct-call candidates are not an
exhaustive caller search. Existing-cache reuse, the alternate context, later
setters and preservation from constructor return to adapter sampling remain
separate from this conditional constructor result.

## Evidence and provenance

Private root: `evidence/scale9-options-construction/`; `receipt.json` and
`MANIFEST.json` bind this document, the checks and the helper receipts.
The main original
validator checks 691 instruction rows / 3,071 bytes, 24 exact vtable bytes,
the eighteen immediate stores and selected argument/publication edges.
Its 2,165 checks pass against the pinned executable. Helper receipts retain
their independent extent, byte, import, footprint and alias checks.

Main record `ev_fa79fe6fbf093264c44e682431222963d40c58374cd0d1e99b553dc074d12c2f`
completed with exit zero, untruncated output and unchanged pinned SHA. It was
exported and reset, retaining no target. The inline/small-object record
`ev_a9621b071fa6b33d4fc3eba3b1da4623dc0c986e13560213e656c60fe143c46a`
hit a provider deadline: completion remains **unknown**, despite retaining
all requested 91 instruction rows / 343 bytes and 24 import bytes. Their
independent exact-byte checks passed. That record was exported/reset without
an unchanged retry; a clean later session does not retroactively prove its
provider completion.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary private installation | Native instructions, constants, PE extents and exact import/vtable bindings |
| Retained `scale9-output/rea-workspace-and-sentinels.json`, `rea-image-creator-installation.json`, original `function_ranges.py`, selected `scale9-rendering`/receiver records and `scale9-draw-mesh-identity/native/rea-preparation-and-renderer-constructor.json` | Original project investigation of that owner artifact | Reused complete constructors, engine publication, exact renderer table and ordinary mesh argument flow |
| New private helper ledgers under `inline-and-small`, `secondary-constructor`, `final-bind` and `caller-binding` | Original project investigation; proprietary inputs remain private | Finite helper effects, imports, conditional branches and exact remaining identities |
| REA 6.9.0, installed `reverse-engineer-anything/SKILL.md` and native/recovery guidance; Rizin 0.9.1 and local objdump | MIT / LGPL-3.0 / GPL-3.0 tooling respectively | Bounded target-free static reads and byte inspection; no copied implementation |
| Python standard library | PSF tooling | Original PE/hash/extent and argument-edge checks |

No extracted native listing, full native option table, game asset or external
implementation code enters the repository. No Cargo, VM, GPU or live-client
test is represented as run by this documentation-only investigation.
