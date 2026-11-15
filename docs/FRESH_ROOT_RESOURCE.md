# Fresh root resource setup after factory return

**VERIFIED static conditional checkpoint:** the selected post-factory call
`0x1400b5150(T,1)` can return to **`0x1402102dd`** with the following four
fields. This requires the eligible same-target factory-return state from
[FRESH_TARGET_COMPLETION.md](FRESH_TARGET_COMPLETION.md), plus the concrete
resource bindings and ordinary heap conditions below.

| Field | Supplied factory-return value | At `0x1402102dd` |
| --- | --- | --- |
| DWORD `T+0x6a` | `0x52004f80` | `0x52004f80` |
| WORD `T+0x88` | `0x000a` | `0x000a` |
| WORD `T+0x140` | `0x0001` | `0x0011` |
| QWORD `T+0xb8` | `0x0000000000000010` | `0x0000000000000010` |

Only the source flag changes: the entry helper sets bit 4 of BYTE `T+0x140`
and preserves the other bits and upper byte. The target is a case-3 source
target; this is not the differently laid-out TextField listener. The next
root instructions derive the owner and call naming helper `0x1401871a0` at
`0x1402102e8`. This checkpoint precedes naming, level publication and later
actions; it is not the complete root allocator's return invariant.

No native executable, original script, VM, renderer, service or player session
was run. VERIFIED labels exact selected instructions and pointer witnesses
matched against the pinned executable. OBSERVED refers to their static control
flow. The proposed implementation contract is INFERRED from those facts.

## Entry requirements and branch selection

The prior factory-return qualification remains necessary: the concrete source,
MovieDef, root/context, successful Environment/ledger work, and effective warm
own-data prototype bindings are supplied or separately established. This
increment does not infer those inputs from a filename, root designation, saved
Scene, bootstrap registration or cached prototype alone. Its predecessor is
the source-completion study at commit
`c84ee06eebdcf40ae8098538a1d6fe0496096f9e`.

The resource path additionally requires:

1. Valid, distinct target, root, MovieImpl, MovieDef, task, heap and resource
   objects. The selected root has primary table `0x142511240`; the selected
   MovieDef has table `0x1425048b0`. Pointer witnesses do not establish which
   objects are live in the native frontend.
2. A finite, well-formed MovieImpl resource list and valid reference counts.
   Matching uses exact MovieDef pointer identity and the stored Boolean byte.
   The helper does not select an arbitrary resource by name.
3. For the fresh allocation branch, successful ordinary allocations of the
   requested sizes, disjoint from all live objects and the target's fields.
   The matching free of the temporary object preserves other live objects.
   Generic heap hooks, reentry, concurrent mutation, collector side effects,
   failure and overflow are outside this condition. The native heap virtual
   implementations themselves are not recovered by this study.

Complete helper `0x1400b5150..0x1400b517d` performs the byte update, then calls
`0x1400b3390(T,false)`. The latter's actual complete range is
`0x1400b3390..0x1400b34fe`, comprising five contiguous chained unwind fragments
and 90 instructions. Its first 34-byte exception-table entry is only a
fragment. There are three distinct paths:

| Condition | Observed effect |
| --- | --- |
| `T+0x108` already nonnull | Return without nested calls |
| Matching MovieImpl list resource | Increment its DWORD `+0x18`, assign `T+0x108`, return |
| No existing or matching resource | Create a resource, initialize its secondary state, then publish it in the MovieImpl list |

The helper's only direct target store is QWORD `T+0x108`. Resource/list writes
do not overlap the four fields under the distinct-object condition. Direct
stores alone are insufficient; the fresh branch's transitive closure follows.

## Resource and secondary construction

Root virtual slot `0x142511270` (`+0x30`) binds complete 92-byte factory
`0x14017d060`. It requests a `0x40`-byte resource R and initializes its
refcount, MovieDef identity, Boolean byte and **R+0x38 = null**. It receives
the heap and MovieDef, not T. The caller assigns R to `T+0x108`, copies task
DWORD `+0xf4` into R+0x2c and, for this false Boolean, obtains the frame count
through MovieDef virtual `+0x28`.

That slot, `0x1425048d8`, binds the complete 11-byte leaf `0x140087900`:
it reads `[MovieDef+0x38]+0xf0`, with no stores or nested calls. The caller
retains that count at R+0x28. No particular count or task value is invented.

The caller then requests `0x70` bytes for secondary S. Complete 174-byte
constructor `0x1400add10` initializes S, including **S+0x48 = null**, and
calls `0x1400347c0(S+0x50)` followed by `0x1400acb00(S)`. The first is a
complete 27-byte leaf: it writes its shared pointer and atomically increments
the separate global counter at `0x142adb2f8`. It does not receive T.

`0x1400acb00` is **346 bytes / 86 instructions**, ending at `0x1400acc5a`.
Its six-byte initial unwind fragment contains only the prologue. All six
chained fragments are now retained, decoded and checked against original
bytes; the following six padding bytes are excluded.

On the selected successful path it creates three distinct support objects:

| Object | Allocation | Selected initialization |
| --- | --- | --- |
| A | `0x88` bytes | Leaf `0x1400d7a10`, 105 bytes; primary table `0x142507d30`, refcount 1 |
| B | `0x40` bytes | `0x1400d0a80`, 125 bytes; table `0x1425078e8`, refcount 1, retains A at `+0x18` |
| C | `0x40` bytes | Inline wrapper construction; refcount 1, retains B's `+0x18` value A at `+0x30` |

B explicitly starts its `+0x10` and `+0x18` fields at null. Its only helper
apart from reference retain/release is complete 13-byte `0x14059c400(B+0x28)`,
which zeroes B+0x28 and B+0x30. The third constructor argument is zero and
is retained at B+0x20. A's constructor has no calls. These facts do not imply
that padding or every allocation byte is zeroed.

C's tagged-reference helper `0x140035d30` is a complete 31-byte leaf, followed
by one excluded padding byte. It updates the same separate global counter
and stores a tagged shared pointer at C+0x20. The new C is assigned to S+0x48.
The old-value release is excluded because S+0x48 was explicitly null and no
intervening selected operation changes it.

## Exact temporary cleanup

The successful path's reference progression is finite:

| Step | A refcount | B refcount |
| --- | ---: | ---: |
| Constructors before B's retain | 1 | 1 |
| B retains A | 2 | 1 |
| C retains A | 3 | 1 |
| Temporary B is destroyed and releases A | 2 | 0 |
| Temporary A reference is released | 1 | — |

Complete 11-byte `0x140031ce0` increments DWORD `object+0x08` atomically.
Complete 30-byte `0x140031cf0` decrements it and dispatches the primary
destructor only on zero. Thus A's two cleanup releases, 3→2 and 2→1, do not
invoke A's destructor under these exact counts.

B does reach zero through complete 60-byte `0x14004c720`. Its optional
notification through B+0x10 is excluded by the constructor's null value.
Its actual primary slot `0x1425078e8` binds complete 104-byte deleting
destructor `0x1400d0b00`, called with delete flag 1. That destructor's optional
virtual release through B+0x28 is likewise excluded by the proved zero. It
releases A, updates B's own table, and calls the global heap's `+0x60` free
for B. This matching free remains part of the ordinary heap condition; it
is neither skipped nor presented as a recovered allocator implementation.

After secondary construction returns, the resource helper's old R+0x38
release is also excluded by the resource factory's explicit null value.
The caller assigns S to R+0x38 and links R into the MovieImpl resource list.
These are real allocation, ownership and list effects even though the four
selected target fields survive. The excluded callbacks above remain excluded
through the closed non-heap callees; the native frame-count getter and exact
B destructor are retained operations. No script accessor or prototype factory
is invoked on this selected path.

## Limits and implementation use

The native resource factory's null return is not handled safely by the
observed caller before a dereference. Some later allocation failures branch
differently; this study does not generalize successful-path counts to them.
It does not establish cleanup behavior for substituted tables, corrupted
lists, aliases, changed support fields, overflowing counts or reentrant
allocator hooks.

**INFERRED:** an original fresh-construction adapter can model this as a
bounded resource transition on the same target: source bit update, exact
existing/list lookup, resource and secondary ownership work, and publication.
The four-field checkpoint is an output of that admitted transition, not a
license to assign `source_flags=0x11` to arbitrary instances or bypass the
resource work. Current-instance adoption still requires coherent supplied
state. No VM, host, display, default-state or lifecycle implementation changes
are made here.

## Evidence and provenance

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/fresh-root-resource/`.
`verification.json` checks **16 complete selected ranges, 516 instructions /
2,001 instruction bytes, and three pointer witnesses / 24 bytes** against the
owner's pinned PE. It includes the complete root caller establishing return
address `0x1402102dd`. `contract.json` records the conditional transition and
excluded calls. `receipt.json` binds the complete private artifact ledger.

Parent-retained inputs are preserved: `root-prefix.bundle.json`,
`retained-functions.json`, `retained-discovery.json`,
`resource-callees.rea.json` and `resource-secondary.rea.json`.
`retained-audit/` checks their canonical-result consistency;
`unwind-audit/` independently parses the chained exception/unwind metadata.
The older six-byte `0x1400acb00` capture remains explicitly partial.

New complete, untruncated, exit-zero REA records report unchanged target hashes:

| Evidence | Selected scope |
| --- | --- |
| `ev_da6ae5a7f8b2f1384eca821075820fbeaf6aee38449157e34d980a5dfba5249c` | Complete `0x1400acb00` |
| `ev_27bd461758bfcf3a1b9d17145f92eabadaf819d14a1b047b247c832c60e30e3c` | Tagged-reference leaf and both release helpers |
| `ev_b4e43131c71f7d481bdef0a8a684ccb64fa60701fd37a9f120e3fa11be99a7a4` | Exact B destructor slot and body |
| `ev_2350b2f80e2d31ae8b63767ec06a6724d32b5773ccdcbf1c174279ac2d02fa3c` | A/B constructors and B pair initializer |

The first three are exported in `new-native.bundle.json`; the fourth is in
`secondary-constructors/native.bundle.json`. Both sessions were reset with no
retained target. No shared engine configuration was changed.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's `MarvelHeroesOmega.exe`, build `1.52.0.1700`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private compatibility study | Native instructions, dispatch slots and unwind ranges |
| Parent-retained files above; `attached-target-initialization/base/first.bundle.json`; `rea-class-registry/owner.bundle.json`; fresh target-state/completion documents and receipts | Original project investigation; underlying executable remains proprietary | Reuse selected retain helper, caller, entry qualification and existing resource observations |
| Installed reverse-engineer-anything skill and native/artifact guide; REA 6.9.0 / Rizin | Existing MIT / LGPL-3.0 tooling provenance | Bounded static reads and private canonical Evidence |
| Python standard library | PSF tooling; verifier is original code | Independent PE mapping, byte checks, hashes and receipts |

No recovered code, original byte arrays or extracted resource data enter git.
