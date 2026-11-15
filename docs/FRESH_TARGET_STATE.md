# Fresh source target flag checkpoints

**VERIFIED static conditional checkpoint:** the exact root-source and owner
constructor path can establish these fields immediately after
`0x14020366a`, at `0x14020366e`, before frame-capacity lookup, ledger allocation
and prototype selection:

| Native field | Value | Width / interpretation |
| --- | --- | --- |
| Target `T+0x6a` | `0x52004f80` | DWORD; includes the encoded owner byte at `+0x6d` |
| Target `T+0x88` | `0x000a` | WORD; the later event adapter's load-flags input |
| Source target `T+0x140` | `0x0001` | WORD; case-3 source flags, not EditText listener flags |
| Target `T+0xb8` | `0x0000000000000010` | QWORD; low DWORD includes focus/tab/status bits |

This requires the concrete definitions, getter bindings, successful distinct
allocations and root inputs below. It is not the current state of a placed or
adopted login instance, nor a completed root-construction invariant. The
previous all34 Construct probe supplied its raw state explicitly; these findings
do not retroactively convert those inputs into fresh native defaults.

No client, VM, original script, renderer, service or live session was executed.
VERIFIED denotes selected complete native instructions and pointer witnesses
matched to the pinned executable. OBSERVED denotes their static control flow;
the proposed implementation boundary is INFERRED from that evidence.

## Field widths and exact base returns

Let `T` be the new target, `O=T+0x148` its case-3 owner and `E=O+0x38` its
embedded Environment. The allocation is `0x348` bytes; the allocator is not
assumed to return zeroed storage. Valid definition, MovieDef, target and heap
objects must not alias the newly initialized fields.

The low WORD at `T+0x6a` contains flags, while the full DWORD also contains
bytes `+0x6c` and `+0x6d`. Owner binding writes only the latter byte. A low-WORD
OR cannot initialize those high bytes. Conversely, the `+0xb8` initializer is
a QWORD store: it initializes both the low status DWORD and the upper DWORD
at `+0xbc`. Later DWORD writes preserve that upper half.

| Complete function / return instruction | DWORD `+0x6a` | WORD `+0x88` | WORD `+0x140` | QWORD `+0xb8` |
| --- | --- | --- | --- | --- |
| `0x14009c270` / `0x14009c2e6` | `0x4000` | Uninitialized here | Uninitialized here | Uninitialized here |
| `0x14009e6d0` / `0x14009e72d` | `0x4100` | `0x000a` | Uninitialized here | Uninitialized here |
| `0x1400a1130` / `0x1400a11e9` | `0x4180` | `0x000a` | Uninitialized here | `0x10` |
| `0x1400b4dd0` / `0x1400b4e32` | `0x4380` | `0x000a` | Uninitialized here | `0x10` |

The full DWORD initializer is at `0x14009c2cb`. The successive low-WORD
ORs are `0x100` at `0x14009e715`, `0x80` at `0x1400a11ad` and `0x200` at
`0x1400b4e26`. Load WORD `0x000a` is stored at `0x14009e71e`;
QWORD `0x10` is stored at `0x1400a11a2`. The first base's explicit null
render-node pointer excludes its conditional visibility callback. Complete
definition-refcount leaf `0x140031ce0` and child-list initializer
`0x140130b30` preserve the fields above; the latter also establishes the fresh
zero child count.

These refine and reuse [ATTACHED_TARGET_INITIALIZATION.md](ATTACHED_TARGET_INITIALIZATION.md).
The EditText route shares only the common base checkpoint. Its `T+0x140`
belongs to a different structure and cannot use the case-3 source WORD here.

## Conditional root-source return

The complete root allocator `0x140210180..0x14021034d` forms its source from
`[selected MovieDef+0x38]+0x18`, supplies factory case 3 and a null parent,
and passes resource ID `0x40000`. The complete case-3 factory
`0x140178440..0x1401786d0` calls source constructor `0x1400b6d40` with that
source, the selected MovieDef, the root pointer and final Boolean zero. Its
source-constructor return site is `0x14017862d`.

The exact source profile is a nonnull concrete DataDef with primary table
`0x142504680`. Existing constructor `0x140090520` installs that table. The
new REA witness binds its virtual `+0x10` slot, address `0x142504690`, to
complete six-byte getter `0x140086370`, which returns `0x8000` and makes no
calls or stores. This is the **virtual return**, not a presumed source field.

Source constructor `0x1400b6d40..0x1400b6f2a` compares that result, masked
with `0xff00`, against `0x8400`. The concrete `0x8000` result excludes its
Scale9 branch and its nested geometry callbacks. Null parent plus final Boolean
zero exclude the parent virtual `+0x208` and resource-context helper
`0x1400b3390` at this point.

The remaining direct operations initialize WORD `T+0x140` to zero at
`0x1400b6de1`, OR target low-WORD `+0x6a` with `0x400`, set source bit 0,
clear source bits 1 and 5, and clear source bit 4 for the zero Boolean. No
wider overlapping store intervenes. The final cookie helper
`0x141c6ed60..0x141c6ed7f` has a successful comparison/return path with no
stores or calls; its failure tail is outside the successful constructor profile.
Thus the conditional return at `0x14017862d` establishes:

```text
DWORD T+6a  = 0x00004780
WORD  T+88  = 0x000a
WORD  T+140 = 0x0001
QWORD T+b8  = 0x0000000000000010
```

The source identity is deliberately a condition. The adjacent root-association
study proves that fresh loader lookup-status 3 retains its supplied DataDef
through the MovieDef task and into this root source argument. It does not
identify the concrete subtype of every cached MovieDef or observe the actual
engine's live source category. See [FRESH_ROOT_NAME.md](FRESH_ROOT_NAME.md) and
private `evidence/fresh-root-name/movie-association/receipt.json`.
An arbitrary cached source, source Sprite category `0x8400`, nonnull parent or
different constructor Boolean does not satisfy this profile.

## Owner binding and the later closed prefix

Factory case 3 next calls `0x140203560` with `O=T+0x148`. Its prefix writes
only owner-relative fields, then calls complete binder
`0x140099640..0x14009964f`. That leaf encodes `(O-T+3)>>2` into BYTE
`T+0x6d`, giving `0x52`. Immediately after the binder, at `0x1402035b2`,
the conditional target DWORD is therefore `0x52004780`; `+0x88`, `+0x140`
and `+0xb8` still have the source-return values above.

The later prefix through `0x14020366e` is closed under these additional
specific conditions:

1. The embedded Environment constructor succeeds with valid, distinct ordinary
   page/array allocations. Its structure stores start at `E+8 = T+0x188`
   and remain disjoint from all four target fields. Its allocator failure,
   arbitrary heap hooks and reentry are not modeled by this condition.
   [FRESH_AVM1_ENVIRONMENT.md](FRESH_AVM1_ENVIRONMENT.md) retains the complete
   Environment and both page-stack constructors.
2. The selected MovieDef has table `0x1425048b0`; its task points to a concrete
   DataDef with table `0x142504680`. Target table `0x1425069d0` virtual
   `+0x208` binds getter `0x1400a12b0`, returning `T+0xb0`.
   `0x140099650` then follows MovieDef virtual `+0x20` to `0x1400877f0`,
   which follows task `+0x18` and tail-calls DataDef virtual `+0x48`,
   `0x1400863a0`. The leaf follows DataDef `+0x40` to state and reads its
   DWORD `+0x54`; the caller retains the low result byte as the cached version.
   These complete getters contain no target stores; the selected version is
   bound to the Environment, not invented from a filename.
3. Owner table `0x142517670` has virtual `+8` bound to self-return leaf
   `0x1405b93f0`, and virtual `+0xb8` to `0x14017b880`. The latter follows
   `O+0x20 → T+0x20 → root+0x50` to the nonnull actual context. These are
   **owner** virtual calls, not a presumed root virtual. The complete getters
   contain no stores or nested calls.

With those closures, owner source-byte operation `0x14020365b` only clears
already-clear source bit 5, and `0x14020366a` ORs target low-WORD `+0x6a`
with `0x800`. The opening table follows at `0x14020366e`. Local visibility
bit 14 is set there. That fact does not establish a descendant's effective
visibility after placement or a current live visibility snapshot.

## Exact survival boundary and later mutations

The checkpoint stops **before** source frame-capacity virtual `+0x50` at
`0x14020367c`, ledger allocation/zeroing, and prototype selector
`0x14019b570` at `0x1402036b6`. Initial prototype selection can perform
factory or accessor work; its bounded warm selection rules are described in
[FRESH_MOVIECLIP_PROTOTYPE.md](FRESH_MOVIECLIP_PROTOTYPE.md). Direct absence
of target stores in the rest of the owner body is not proof that those
transitive effects preserve target state.

After factory return, the root calls `0x1400b5150(T,1)`. Its direct source
byte write sets bit 4, then it calls `0x1400b3390(T,false)` even though the
source constructor's null-parent branch skipped that helper. The resource
helper's root/MovieDef virtuals are a later state boundary. The naming helper
`0x1401871a0`, level publication `0x14005fdb0`, later virtuals and actions
must likewise be traced as their own transitions. This study does not carry
the table's values through them or call `source_flags = 0x11` a root-return
invariant merely from that one direct OR.

Already retained later writers show why snapshot adoption stays separate:

| Later operation | Relevant mutation / limit |
| --- | --- |
| Case-3 child publication | AND low WORD `+0x6a` with `0xefef`; later helper ORs `0x2000`, and the load producer can OR `0x20` |
| Load/publication setup | Changes WORD `+0x88`; later conditional work can set source `+0x140` bit 7 |
| Visibility/list work | Can change target `+0x6a` bit 14 and low DWORD `+0xb8` bits 3, 21, 22 and 23 under their retained guards |
| Boolean-false `_focusrect` | Low DWORD `F → (F & ~0x80) | 0x100`, subject to the exact admitted setter route |
| Boolean-false `tabEnabled` | Low DWORD `F → (F & ~0x20) | 0x40`; existing watch/value-conversion guards remain relevant |
| Adjacent retained tabIndex fragment | WORD write at `T+0xbc` overlaps the upper half of QWORD `+0xb8`; no complete tabIndex behavior is claimed |

The private audit also retains other direct masks without assigning unsupported
property names. This is a selected writer inventory, not an executable-wide
proof of every possible mutation. It does not schedule events, deliver saved
handlers, execute constructors or establish their completion.

## Implementation boundary

**INFERRED:** a future fresh-construction adapter can use the four exact values
at a named checkpoint only after validating or explicitly receiving the source,
MovieDef, owner/root/context and successful-allocation profile. It should retain
the checkpoint identity and run each later construction transition against the
same state. A live-instance adopter still needs supplied coherent current
state. A synthetic `AttachedEventState` must not silently select this profile
because the receiver happens to be a root or case-3 owner.

This increment adds documentation only. It neither edits VM defaults nor adds
an adapter that skips frame-ledger/prototype/resource/name/publication work.

## Evidence and provenance

Private root: `evidence/fresh-target-state/` beneath
`~/.local/share/marvel-heroes-forever-rust/`. The original Python verifier checks
**26 selected ranges, 1,203 instructions / 4,897 instruction bytes, and six
selected pointer witnesses / 48 bytes** against the independently pinned PE.
It also checks each pointer's complete captured table window, source hashes,
contiguous range endpoints and available PE exception-table extents. Corrected
old overreads are retained in their canonical records and excluded from the
selected ranges. These byte checks support static analysis, not execution parity.

Two new target-free REA calls retain complete, untruncated results with exit
zero and unchanged target hash:

- `ev_fe1eeb5060222322ad2792752c50ab6f32033548ab277a414db1f3baa1e8a414`:
  DataDef category pointer and complete six-byte getter.
- `ev_845eeb2df6d4de367c5e77dcf13bb462f50687b66f2ca0ff63ea25102e9a7414`:
  complete 31-byte cookie checker, successful return and excluded failure tail.

The calls' automatic string-region warnings do not affect the bounded pointer
or instruction results and do not establish a global string search. Canonical
`new-read-checkpoint.bundle.json` exports both records; REA was reset with
`retained_target=false`. No shared configuration or engine setup was changed.

`verification.json` and `checkpoint-analysis.json` retain the exact ranges,
source hashes and conditions. Independent retained-evidence readback is in
`b8-audit/AUDIT.md`, `extra-flags.json` and `owner-getter-closure.json`.
`receipt.json` and `MANIFEST.json` bind the complete private result and source
ledger. No recovered instructions or proprietary bytes enter this repository.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's `MarvelHeroesOmega.exe`, build `1.52.0.1700`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; private interoperability study | Exact target/owner constructors, getter slots and successful-return checkpoint |
| Existing project documents cited above; retained REA bundles under `rea-attached-movieclip/`, `rea-asflags-followup-version/`, `rea-asflags-followup-version-proof/`, `rea-native-alternative/`, `rea-class-registry/`, `rea-attached-root-name/`, `frontend-image-context/`, `attached-target-initialization/`, `attached-visibility-prerequisites/`, `fresh-avm1-environment/` and `fresh-root-name/` | Original project investigation; underlying native artifacts remain proprietary | Reuse complete constructor, source-association, version/context, publication and mutation evidence; exact files/hashes in private ledger |
| Installed `reverse-engineer-anything/SKILL.md`, its native/artifact and evidence guides; REA 6.9.0 / Rizin 0.9.1 | Previously registered MIT / LGPL-3.0 tooling; no implementation copied | Bounded static reads and canonical private Evidence retention |
| Python standard library | PSF tooling; verifier is original project code | Independent PE mapping, byte/extent comparisons and receipts |

No external SDK/reference implementation, account, service, original install,
launcher, native process or other project's source was modified.
