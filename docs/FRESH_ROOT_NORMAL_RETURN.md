# Conditional fresh AS2 root at normal return

**VERIFIED static composition:** the selected original root setup
`0x140210180` can produce the following state at its normal return
`0x14021034c`, from actual context, global, prototype and target constructor
stores. The result includes cold XML/XMLNode registration. It no longer
requires an assumed preexisting MovieClip binding, empty bootstrap map, or
exhaustive 46-member global as an opening state.

This is one bounded construction contract, **not a captured running client or
a complete native object image**. Its remaining input, resource and callback
conditions are listed below. No native executable, original script, VM, GPU,
service or network operation was run. VERIFIED refers to retained native
evidence and source/hash checks; OBSERVED to static effects; INFERRED to the
finite mathematical histories and implementation use.

## Return snapshot

Let R be the concrete AS2 root, M its movie implementation, D the selected
MovieDef, C the context allocated by this call, G its fresh global, T the
case-3 target, S its actual source DataDef, O its owner at `T+0x148`, and P/F the produced original
MovieClip prototype/callable. These are related by the actual pointer stores
and reads below; matching filenames or numeric IDs do not establish identity.

| Selected state at normal return | Result |
| --- | --- |
| DWORD `T+0x6a` | `0x52004f80` |
| WORD `T+0x88` | `0x0008` |
| WORD source `T+0x140` | `0x0011` |
| QWORD `T+0xb8` | `0xffff000000200190`, under the clear policy-bit profile below |
| Public native name | Exact `_level0` in the successful handle's `+0x10` |
| Root/context/global association | `R+0x50=C`, `C+0x28=M`, `C+0x60=G` |
| Source/owner association | `T+0x20=R`, `T+0xb0=D`, `T+0x110=S`, `O+0x20=T`; owner bound-storage slot `O+0x1d8` remains null |
| Embedded Environment | At `E=O+0x38`, actual target `E+0xe0=T`, context `E+0xe8=C`, cached selected source-version byte `E+0xf0=V` |
| Numeric prototype cache `C+0x10` | Exactly keys 1/7/26/6 for the produced Object/Function/Key/MovieClip prototypes; count 4, capacity 8 |
| Registered-class map `C+0x18` | Null; the native `registerClass` method is installed but never invoked by this path |
| Deferred factory map `C+0x20` | 36 exact original entries, capacity 64; every cached function reference null |
| Standard-name map `C+0x70` | 117 original names/IDs, capacity 256 |
| G's own members | 48 exact source-derived names, capacity 64: 25 native function Values, 2 ordinary namespace objects, 21 deferred name tokens |
| G's descriptors | Flag 1 on the 46 members visited at the end of bootstrap; flags 0 on the later XML/XMLNode additions |
| MovieClip logical binding | `G.MovieClip=F`, own `F.prototype=P`, initial owner prototype `O+0x18=P` |
| Level publication | M's first level slot is `{level:0,target:T}`; count 1, capacity 4; `M+0x50=T` |

Capacities/counts are derived from the actual constructor-established empty
maps and finite source writes through the recovered insertion rules. They
are not heap observations. Collision relocation can change physical buckets;
the snapshot does not invent their indices. Native function Values retain
the original null auxiliary frames/zero reference flags; those flags are
distinct from member descriptors. Tag-11 tokens are not instantiated callable
objects. Registration stores the factories without invoking them, and the
selected cache/namespace reads do not resolve any deferred slot.

The source Environment's cached version is the low byte selected from D's
actual source data, **not a universal 8**. Its modern selector path requires
that byte to be greater than 6. The bootstrap and late XML helper contexts
separately contain literal version 8. Frame capacity comes from another
source field and is required positive; it is not the zero target integer used
to bypass the publication event predicate.

The name handle's internal path at `+0x18` is separate from public name
`+0x10`. The path-building branch depends on the actual root byte. Pending
assignment lookup uses that actual internal path, not a substituted `_level0`
path identity. This snapshot does not assign coordinates, stage dimensions,
locale, fonts, imported resources, camera state or arbitrary unlisted bytes.

## Construction sequence establishing the snapshot

| Native phase | Produced state and included work |
| --- | --- |
| Context seed and early global construction | Successful root allocation explicitly clears context map/global slots. Original Object/Function/Key/Array/MovieClip producers establish their native identities and numeric cache keys. G starts with an explicitly empty own map. |
| Native global installer | 21 exact native-function entries, four same-key replacements and four eager additions produce G count 25 / capacity 32 before the first deferred registrar. The four real old-function zero-reference destructions are included. |
| 34 registrars and namespace interleaving | Starting from the produced null factory map, register 34 distinct factory/null-cache pairs. Nineteen add globals; namespace traversal adds exactly the two root names `flash` and `System`. G becomes 46 / 64, factory map 34 / 64. |
| Bootstrap suffix | Build the separate 117-entry standard-name map; OR flag 1 on existing global descriptors; perform the bounded temporary/reference cleanup. |
| Root/render prefix and target/owner creation | Publish the same C; construct the concrete root-source target, Environment and positive-capacity frame ledger. The produced numeric/global/prototype data supplies the initial owner selector; no lazy factory is needed. |
| Resource and naming | Attach/initialize the selected native resource, set source bit 4, insert the render node, format/copy/intern `_level0`, and assign the public name. |
| First level and viewport | Insert level 0, perform the exact status/list changes, replace definitions/state references, update real viewport/matrix data and possibly enqueue resize work. |
| Cold late registration and tail | Add XML/XMLNode factory rows and global tokens, then take the admitted optional-state/pending-assignment branches, apply the 55.0 projection update and release the root's temporary target reference. |

The early producer is
[FRESH_GLOBAL_BOOTSTRAP.md](FRESH_GLOBAL_BOOTSTRAP.md), joined with
[FRESH_BOOTSTRAP_BINDINGS.md](FRESH_BOOTSTRAP_BINDINGS.md). It closes the
previous early map/history qualifications in
[FRESH_ROOT_COLD_REGISTRATION.md](FRESH_ROOT_COLD_REGISTRATION.md), rather
than merely supplying their desired results as fixture inputs. The derived
Object.prototype nine-key/null-parent state supplies the namespace miss proof.
Original Object/Function source constructors and typed insertion establish the
function/object representations used by later relocation.

Global growth from capacity 32 to 64 occurs before total insertion ordinal
27. The subsequent exact globals finish at count 46. Late XML comparisons
are `5*46 <= 4*64` and `5*47 <= 4*64`, so its two global insertions do not
grow. The separate factory-map comparisons use counts 34 and 35. Both maps
admit their real collision branches; neither requires empty destination
buckets. Same-name XML overwrite is excluded by the now-composed exhaustive
fresh global production and unchanged intervening names, rather than by
absence from a stand-alone write-name inventory.

The early source behavior includes details that a reconstruction must retain:
13 native methods created before numeric cache key 7 have null internal
parents; the later relink visits only the Object and Function callables.
Key's listener Array is also created before its numeric key 2 exists and has
a null internal parent. These are different from native global functions
installed after the original Function prototype is available.
Key also performs the proved static Array-storage retain and six MovieImpl
listener-pointer stores. The selected snapshot does not claim that collector,
StringManager or MovieImpl memory is otherwise byte-for-byte unchanged.

The target transition is likewise ordered, not a bag of defaults:

| Checkpoint | DWORD `+0x6a` | WORD `+0x88` | WORD `+0x140` | QWORD `+0xb8` |
| --- | --- | --- | --- | --- |
| Completed source-owner/factory return | `0x52004f80` | `0x000a` | `0x0001` | `0x10` |
| Resource helper return `0x1402102dd` | same | `0x000a` | `0x0011` | `0x10` |
| Successful nonempty name assignment | same | `0x0008` | `0x0011` | `0x10` |
| Selected level publication and later tail | same | `0x0008` | `0x0011` | `0xffff000000200190` |

The target's native child count, initialized at `T+0xe0`, carries zero
through the mandatory base projection to its actual loop read. This native
child list is separate from the render containers, which do receive children.
It is also separate from a populated static Scene assembled from saved SWF
placements. No frame-zero child construction or action delivery is implied.

The empty render-child slot used when naming T is also a produced result in
this composition. Root setup explicitly zeros the new container payload's
`+0x90/+0x98`, then publishes that container at `R+0x28`. Inserting it below
M's render parent changes the parent's child storage and the new node's parent
link, not the new container's own child slots. The closed intervening
factory/resource work preserves those slots until naming inserts T's node.
The outer M render parent need not be assumed empty. Current-record, valid
child-storage and disjoint allocation conditions still apply.

## Remaining gates

These conditions are still material. They are not replaced by the new
bootstrap producer proof or inferred from a successful outer return.

| Gate | Required selected state / reason |
| --- | --- |
| Concrete source and root association | The actual compatible AS2 tables, same R/M association including `R+0x10=M` and `M+0x18=R`, selected ordinary MovieDef/DataDef identity and category `0x8000`. Source/header/resource selection and cached/substituted loader alternatives are not inferred from a filename. |
| Header and frame inputs | Actual selected version byte greater than 6 and positive source frame capacity. Normal successful cookies and valid source storage. No `memset(NULL,0,0)` or arbitrary version-fallback behavior is added. |
| Ordinary operations | Successful valid distinct allocations and matching frees; valid finite tables, capacities, indices, pointers and nonoverflowing counts; ordinary bounded CRT/locking/math contracts; no concurrent, external or reentrant mutation. Generic allocator/import behavior was not recovered as universally inert. |
| Four early zero releases | The exact original one-marker/null-watch/null-resolver native function shapes, rooted replacement parent/key, valid idle collector, and the documented queue/root-buffer conditions. Their actual destruction/free runs; zero release is not globally forbidden or assumed harmless. |
| Other managed cleanup | The established positive-reference/root-buffer contract, including successful page growth where needed. Allocation-failure collector processing, unsupported captured frames/categories or unrelated destructor graphs remain outside. |
| Old root/resource references | Actual old `R+0x50` and `R+0x28` null at the root-prefix reads. Publication has `M+0x30=null`, old `M+0x48=null`, and an old inherited StateBag reference that is null or stays positive after release. Prior constructor zeros alone do not close every intervening MovieImpl operation. |
| Resource association | Valid finite resource list and exact MovieDef/Boolean matching; concrete root/MovieDef resource factories and the selected successful secondary ownership path. Arbitrary resource subclasses, allocation failures or replaced old payload callbacks are not generalized. |
| Render resources | Valid nonempty node free list where selected, required existing current change records, valid outer-parent child storage, typed matrix descriptors/payloads and disjoint render graph. Record/state growth and known typed matrix destruction are included; node-pool growth and arbitrary backing-copy callbacks are not silently covered. Naming's new-parent empty slot is derived above. |
| Level/list and policy inputs | Actual empty M level array at its first insertion; valid distinct prior list head; actual `M+0x4310` bit 19 clear and `M+0x4314` bit 3 clear at the status reads. Their constructor checkpoints do not establish current values by themselves. |
| Exact naming | Successful formatting/interner/copied-storage/name-handle paths. Allocation fallback can produce another string; a normal return alone does not prove `_level0`. |
| Optional state and pending assignments | State ID `0x1d` absent through the finite proved local/inherited StateBag getter chain, and pending-assignment map null or exact actual handle-path miss. Found states, matched setters/renames and foreign getters are separate branches. |
| Queue and final target lifetime | Any selected resize slot is freshly initialized or actually cleared/valid; no queue drain or intervening target reselection mutation. The final T release leaves a positive reference, supported by the surviving level slot and optional queued reference under valid ownership. |

The policy-bit variant is explicit: with all other selected conditions fixed,
setting only `M+0x4314` bit 3 changes the final status QWORD to
`0xffff000000a00190`. It is not acceptable to choose the clear result merely
because it is convenient for a diagnostic VM. Other policy changes need their
own branch result.

The exact root caller ignores the level publication Boolean, sets AL to 1
and returns. Therefore AL=1 alone is not evidence that duplicate-level,
allocation, callback or name-fallback alternatives satisfied this profile.

## Pending work and use of the contract

Viewport changes may add kind-4 work containing callback `0x1401f6c50` and a
retained level-zero target. The caller stores it without invoking it; insertion
is cursor-relative. Preserve any supplied queue state and this added work.
There is no claim that the whole queue was empty or that this is its only
entry. Known native retain/release, typed matrix deletion, the resource
secondary destructor and the four old native-function destructions do run
inside the selected profile. “No native callback ran” would be inaccurate.

The checkpoint ends before delivery of that queued work, saved InitActions,
Construct handlers, dynamic methods, registered constructors, frame actions or
input. It is not the post-452-initializer runtime, the controlled all34 Construct
fixture, or a 147-node static Scene. This composition neither replays those
scripts nor relabels their previously supplied state as fresh native defaults.

**INFERRED implementation use:** keep a fresh-construction path separate from
adopting existing instances. Its output can carry the produced map identities,
selected raw fields/name, explicit source/version and queued work, together
with the still-required resource/branch profile. Unsupported branches stay
explicit. No VM, host, renderer or adopted-display code is changed here.

## Evidence and provenance

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-root-composition/`.
Its receipt binds this document, immutable producer/source receipts,
independent checkpoint/state/gate joins and finite map-history checks.
No duplicate REA capture or original-client execution was needed.

The main new dependency is commit
`f9e4956579011e3554539a213f7857b3b657bbae`, private
`fresh-global-bootstrap/receipt.json` SHA-256
`17a61810411220f443b18d0c273991946691b947a20b0c917cdf83ed0049f77a`.
The composition also consumes the independently sealed early binding,
prototype carry, target completion, resource, name/publication and cold
registration receipts. Their native ranges overlap; coverage counts must not
be summed. Historical earlier documents remain honest checkpoint studies;
this composition identifies which inputs their later producer proofs discharge.

The prior viewport record with unknown process completion remains unchanged.
No uncertain analysis was replayed. Existing canonical record status and
complete-function/chained-unwind boundaries are preserved in the source ledgers.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's proprietary build 1.52.0.1700 executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` and retained REA records | Private interoperability evidence; no redistribution licence | Constructor, map, source, dispatch, lifecycle and ownership facts |
| Referenced project docs and sealed private receipts | Original project investigation; native extracts remain private | Exact same-identity checkpoint composition and residual gates |
| Existing REA 6.9.0/Rizin workflow and skill guidance | Previously recorded MIT / LGPL-3.0 tooling | Retained canonical static evidence; no new provider work |
| Python standard library | PSF tooling; new analysis is original | Finite count/capacity arithmetic and source/hash readback |

No proprietary bytes, extracted name/method tables, assets or private receipt
enter git. This is the scoped source ledger for integration into
[PROVENANCE.md](PROVENANCE.md).
