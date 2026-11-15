# Conditional bootstrap binding preservation

**VERIFIED static conditional result:** the late-bootstrap suffix and the
root's following render setup, and the selected factory/owner prefix preserve
the effective original MovieClip cache/global/prototype bindings through
**`0x1402036a0`**, immediately before owner prototype selection. The suffix includes stateful
name registration, property-descriptor updates and reference cleanup; those
effects are retained explicitly. This advances the missing interval identified
in [FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md).

The opening checkpoint is **`0x14020fee7`**, after successful eager/global
registration, and the first proved ending checkpoint is **`0x1402102cd`**,
immediately before the source-owner factory call; the final composition below
extends it to selector entry. Earlier registration call
sites identify the intended objects, but their uninspected insertion helpers
do not prove the actual opening state. In particular, this document does not
turn the controlled all34 adoption inputs into fresh native defaults.

No native client, original action body, VM, renderer, server or service was
executed. No product code or shared configuration was changed.

## Required effective checkpoint

Use `R` for the AS2 root, `M=[R+0x10]`, `C` for its new script context,
`G=[C+0x60]` for the actual global, `F` for the original native MovieClip
function, and `P` for its original prototype. The opening state requires:

| Identity / storage | Actual checkpoint condition |
| --- | --- |
| Numeric cache `C+0x10` | Key 6 contains retained nonnull `P`. |
| Global `G` | Original concrete interface `0x142512aa0`; own `MovieClip` data contains `F`, native tag 8, null auxiliary frame and zero function-reference flags. |
| Function `F` | Original primary table `0x142514280` and interface `0x142514430`; own `prototype` data contains `P`, native tag 6. |
| Prototype `P` | Original MovieClip prototype interface `0x142514db0`, category `0x0b`. |
| Standard-name map | Actual `C+0x70=NULL` at `0x14020fee7`. This is distinct from numeric cache `C+0x10`. |
| Global inventory | The unchanged original 46 own entries have the established tags 6, 8 and 11; no internal tag `0x0a`. |
| Lifetime and allocation | Valid live object/StringManager/collector/render representations; finite nonwrapping counts and indices; separate storage where indicated; ordinary successful allocation/free and bounded CRT operations. |

Root setup writes `C+0x70=NULL` at `0x1402101e3`. The complete bootstrap
contains only one direct operand at that offset, but earlier calls receive
`C` and aliases. That direct-operand observation does **not** establish
transitive preservation before this checkpoint. Likewise, pointer presence
alone is not a proof that the reference count covers a later local release.

The registration call sites and original interfaces are retained in
[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md),
[REA_MOVIECLIP_CONSTRUCTOR.md](REA_MOVIECLIP_CONSTRUCTOR.md) and
[FRESH_MOVIECLIP_PROTOTYPE.md](FRESH_MOVIECLIP_PROTOTYPE.md).
This increment does not claim complete bodies for numeric-cache insertion
`0x1401a3b10` or initial own-member insertion `0x1401cffd0`.

## Standard-name rebuild

The complete 135-byte `0x140183fa0` receives **`C+0x70`**. With the actual
null map, it returns without freeing storage or destroying keys. Its nonnull
alternative decrements StringNode references, may call `0x140097150`, frees
the old table and clears the slot. That alternative is excluded here.

Next, complete `0x1401954c0(C)` rebuilds the separate standard-ID map.
The exact 117-name table has distinct names and IDs; ID 6 is `_alpha`.
It contains none of `MovieClip`, `prototype` or `__proto__`.
Reserve `0x140187ad0` rounds request 146 to 256 buckets, allocates 6,160
bytes, initializes count zero/mask 255 and empty links, and publishes the map.
The null old map excludes rehash and free. Before each insertion count is at
most 116, so `5*count > 4*capacity` is false; the standard-map insert helper
`0x140187f10` does not grow the table. Its replacement/relocation paths retain
before releasing, and a successful insertion retains each key before the
caller's temporary release. Valid nonwrapping counts exclude zero-key cleanup.

The StringManager is `S=[[[C+0x28]+0x18]+0x618]`. Its intern hash at
`S+0x10`, node free list at `S+0x20`, and page list at `S+0x28` are separate
from both context maps. Complete constant interner `0x1400971e0` has two
supported paths:

| Path | Established writes and calls |
| --- | --- |
| Exact existing name | Read-only hash/lookup, with bounded `MSVCR120.dll!strncmp`; OR the supplied classification flags into the same StringNode and return its unchanged identity. |
| New node | Pop the free list; if empty, `0x140096ac0` allocates a `0x13e0`-byte page and links 127 node slots. Initialize the selected node and insert its pointer with `0x140096d20`. |
| Intern-hash growth | Complete `0x140097010` rounds positive capacity, allocates a separate table, reinserts existing node pointers, frees only the old table and publishes the replacement. No node retain/release or object/Value callback occurs in this selected body. |

The node flags intentionally change: classification bit 29, selected bit 28,
and bit 30 for newly created constant nodes. Existing node identity, string
bytes, length, cached-normalized pointer and low 24 hash bits remain unchanged.
Shared key nodes are allowed; aliasing their tables/pages with `C/G/F/P` Value
or object storage is excluded. Table masks must remain within the stored
24-bit hash domain, and capacity/index arithmetic must remain valid.

This is not an existing-names-only argument. Only four of the 117 names occur
in the retained initial 156-builtin inventory. Other earlier interning might
have supplied more; actual absence of the remaining 113 was not observed.
Both hit and successful allocation paths are therefore included. Allocation
failure fallback, zero-capacity growth, zero-key destruction, malformed
tables and arbitrary replacement heap/import behavior remain outside the proof.

## Final descriptor visitor and temporary cleanup

The actual global interface binds slots `+0x28`, `+0x38` and `+0x40` to
`0x1401a1540`, `0x1401e3ef0` and `0x1401924c0`. The final visitor uses
options 4 and callback `0x14017b7a0`, whose complete body is 30 bytes.
It includes hidden own entries, excludes parent traversal, and ORs descriptor
flag 1 while preserving each stored Value. It does not traverse `F` or `P`.

Options 4 do not disable the visitor's internal-tag-`0x0a` raw-getter branch.
The unchanged original inventory's tags 6/8/11 exclude that branch; a modified
or reused global is not covered merely because it has a MovieClip property.
Descriptor flag changes are distinct from native function-reference flags.

The preceding temporary global release at `0x14020ff62` and the eight final
`0x1401860e0` calls have exact identities:

| Tail call | Released local reference |
| --- | --- |
| `0x14020ff8a` | MovieClip prototype `P` |
| `0x14020ffa7` | Key prototype |
| `0x14020ffc3` | Key callable |
| `0x14020ffdf` | MovieClip callable `F` |
| `0x14020fffc` | Function.prototype |
| `0x140210019` | Object.prototype |
| `0x140210034` | Function callable |
| `0x14021004d` | Object callable |

Require each reached decrement's actual low-26 count to be at least two
immediately beforehand, with the successful retained roots and no overflow.
The positive-count cleanup proved in
[FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md) then preserves the
data bindings while changing header/collector metadata and possibly growing
the root buffer under ordinary successful disjoint allocation. Zero-count
virtual destruction and root-buffer failure into `0x1401817f0` collector
processing are explicit excluded callback boundaries. Nonnull Function/Object
callable allocations are also required because the tail dereferences them.

## Root publication and render setup

After successful bootstrap return, root `0x140210180` publishes this same
`C` to `R+0x50`, allocates and links render storage, and prepares the source
factory arguments. This interval is complete through immediately before
`0x1402102cd` under these additional conditions:

1. Old `R+0x50` is null at the actual `0x1402101f1` read, and old `R+0x28`
   is null at `0x140210258`. Earlier constructor zeros alone are insufficient.
   These conditions exclude old-context `0x140031cc0` and old-resource
   `0x14004a290` destruction.
2. The render-slot allocator `0x140049480` has a valid nonempty free list;
   its pool-growth `0x140049290` branch is excluded. Change-record allocation
   uses complete `0x140049670`/`0x1400488d0` with successful ordinary heap
   allocation where reached.
3. The parent node passed to `0x14004a420` has a nonnull current change record.
   The missing-current-record virtual payload-copy branch is excluded.
4. Root/render/script objects, records, pages, lists and buffers are valid and
   disjoint where required. Reached heap operations allocate fresh suitable
   storage or free only the selected retired render buffer, without reentry.
   Imported `memset`/`memcpy` obey their ordinary bounded contracts.

Complete child insertion `0x1400a96e0` is **532 bytes across two chained
unwind entries**, not just its first 253-byte fragment. At the root's exact
index zero it supports empty, one-inline, two-inline and tagged-buffer
representations. Inline insertion changes pointers; promotion/growth allocates
separate storage, moves the existing child pointers and conditionally frees the
old buffer. Capacity, length, reference-count and size arithmetic must be valid.
The parent container need not be assumed empty. The zero stores to the new
child payload's `+0x90/+0x98` do not establish the separate parent's contents.

These paths change root publication fields and render state. Their selected
direct helpers are complete; remaining reached external operations are named
allocator/CRT contracts. They preserve the separate `C/cache6/G/F/P` bindings.
Earlier incomplete helper findings remain in the private evidence, with a
separate final supplement superseding their missing-body status.

## Factory, Environment and ledger composition

The retained complete factory/source/owner bodies close the remaining interval
from `0x1402102cd` through `0x1402036a0` under the concrete fresh root profile:
dispatch to `0x140178440`, case 3, null parent, source ID `0x40000`, DataDef
table `0x142504680` with category `0x8000`, MovieDef table `0x1425048b0`,
and factory-supplied final Boolean zero. These dispatch/source conditions are
explicit; arbitrary subclasses, other categories and parent paths are excluded.

Factory allocation at `0x1401785ed` calls heap `[M+0x40]` virtual `+0x50`
for `0x348` bytes with R8 zero. Success and distinct writable storage are
required. Its failure path still forms null plus `0x148` before owner setup;
no safe failure return is claimed. The selected constructors write the new
target `T`, owner `O=T+0x148`, stack arguments and separate fresh buffers.
Two shared writes require additional care:

| Call site | Complete leaf effect |
| --- | --- |
| `0x1400a1185` | `0x140031ce0` atomically increments DWORD actual MovieDef `+8`. |
| `0x1400b6db4` | The same leaf atomically increments DWORD actual DataDef `+8`. |

Those actual definition count fields must be valid, nonoverflowing and
distinct from the script-binding storage. The selected path contains no
reference decrement, destructor, Value/Function release or script-collector
call before `0x1402036a0`.

The freshly cleared render pointer excludes base call `0x1400a93e0` at
`0x14009c2d9`. The concrete category getter returns `0x8000`, excluding the
Scale9 block; null parent and Boolean zero exclude parent virtual `+0x208`
and resource-context `0x1400b3390`. The normal successful cookie-check path
has no stores or calls. Their alternate branches are not assumed harmless.

Fresh Environment construction uses four ordinary successful heap allocations:
page/array sizes `0x408/0x80` and `0x108/0x80`. Its direct writes stay in
`E=O+0x38` and the new buffers. Version/context getters are concrete read-only
paths; `0x140203646` stores the actual `C` into `E+0xe8`, obtained through
`O+0x20 → T+0x20 → R+0x50`. This is neither a context retain nor a cache or
global assignment. See [FRESH_AVM1_ENVIRONMENT.md](FRESH_AVM1_ENVIRONMENT.md).

For the positive source frame count `N>0` admitted by
[FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md), fresh ledger resize
allocates separate storage and zeros its first `N` bytes using the exact
`MSVCR120.dll!memset` import. Existing-buffer realloc/free branches are
excluded. This composed profile keeps `N>0`; the unobserved
`memset(NULL,0,0)` edge is not added to its contract.

All reached allocations and definition-count fields must be disjoint from
`C`, its numeric cache, `G`, `F`, `P` and their Value storage; allocator/import
implementations must obey the stated non-reentrant contracts. This explicit
store/call audit establishes preservation of the selected bindings. Preservation
of the four target flag fields alone would not establish it.

**Conditional handoff:** at `0x1402036a0` the actual cache6 value, global
MovieClip data and function prototype data remain the opening `P/F/P`.
Together with the exact version and positive-cleanup conditions in
[FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md), the selector then
returns the same `P` and the owner stores it directly. Earlier bootstrap
insertion, actual truth of the checkpoint guards, failure/GC/reentry behavior,
post-factory resource/name/publication work, InitActions and queued constructor
or event delivery remain separate obligations. This result supplies a bounded
native profile; it does not broaden adoption defaults or invoke any action.

## Evidence and provenance

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-prototype-bindings/`.
Five target-free REA reads completed with exit zero, untruncated results and
unchanged source hash. `final.bundle.json` preserves their canonical records;
`reset.json` records export/reset with no retained target.

The original verifier checks new selected ranges, the complete assembled
bootstrap, exact PE bytes, chained unwind metadata and retained global-slot
witnesses. The bootstrap is 1,396 instructions / 5,670 bytes through exclusive
end `0x140210066`. The new reads contribute 2,252 selected instruction bytes;
overlapping captures and bytes beyond actual function ends are excluded from
new-body claims. Independent context, StringManager and render audits retain
their source ledgers, explicit conditions and separately checked ranges.
The factory-to-selector audit independently matches 24 retained function
selections and six pointer selections to the PE, including the shared
definition-count writes and fresh Environment/ledger allocation boundaries.
`verification.json`, `receipt.json` and `MANIFEST.json` bind final results.

The first local verifier treated an inline REA result as a canonical bundle
and stopped before publishing results. Its source and error are retained;
the corrected loader uses the existing canonical export. No native capture
was repeated for that correction. No Rust build/test was needed for this
docs-only increment, and no runtime or retail-parity check is claimed.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's build `1.52.0.1700` `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Selected startup, map/interner, cleanup, render and PE/import evidence |
| Project docs cited above; `FRESH_TARGET_STATE.md`, `REA_CLASS_REGISTRY.md`, `REA_FUNCTION_CALLS.md`, `ATTACHED_MEMBER_NAMES.md`, `ATTACHED_NAME_CLASSIFICATION.md`, `REA_ENUMERATION.md`, `REA_ATTACHED_NAME.md`; retained canonical bundles and inventories individually hashed in the private ledgers | Original project investigation; underlying artifacts remain proprietary | Reuse exact source, table, name, Value and constructor evidence without recapturing known bodies |
| Installed REA skill/guides and REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling | Bounded static reads, complete Evidence export and reset |
| Python standard library | PSF tooling; all verifier/receipt scripts are original | Independent hash, PE, byte and manifest checks |

No external VM/SDK implementation was copied. Binaries, native instruction
extracts, names and raw inventories remain private and do not enter git.
