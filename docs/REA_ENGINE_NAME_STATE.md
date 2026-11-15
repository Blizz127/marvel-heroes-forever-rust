# Engine name initialization and loader-state preservation

**VERIFIED static route:** the engine prefix at **0x14125e17a** enters name
initialization through `0x141270c20 → 0x140793e60 → 0x14079d210`. Its first
name is the fixed UTF-16 string **"0"**, with a stack output and effective
arguments `(out, text, 0, 1, 1)`. The first previously unexpanded helper,
`0x14077bed0`, only returns a global address. Its conditional initialization
callee is substantial: it constructs/registers 555 distinct name records.

**INFERRED bounded preservation:** under the explicit valid-name-graph,
allocation/import and no-reentry conditions below, this first selected name
call returns without changing the actual loader or its state bag. The next
caller exposure is **0x141270ccc → 0x141263340**, passed **E+0x100**. This study
does not prove the rest of the 7,974-byte initializer, engine-constructor
return, current state-4 absence, or successful Mouse.hide execution.

The continuation below extends that conditional proof across the actual
E+0x100 container's first 97 additions and the following fixed name lookup,
stopping before the distinct helper at **0x141271e02 → 0x14127bc40**.

Target: owner's Win64 build 1.52.0.1700 PE, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All evidence is static. No native process, original script, diagnostic, host
callback or cursor operation was executed.

## Actual published identities and first input

The existing [REA_MOUSE_STATE_PRESERVATION.md](REA_MOUSE_STATE_PRESERVATION.md)
proof establishes engine E already published through global **0x142c45980**
at **0x14125e0b0**, before this call. The actual loader is **L=E+0x44**,
**O=[L+8]**, and its backing bag **C_L=[O+0x20]**. Thus an arbitrary callback
could potentially reach the engine even when no explicit loader argument is
passed. Different directly addressed globals alone do not prove preservation.

At **0x141270c78**, the caller places UTF-16 code unit 0x30 into its local
buffer, with trailing zero already stored at **0x141270c6a**. At
**0x141270cb1**, it passes a separate stack output, that buffer, R8d=1 and
R9d=1 to **0x140793e60**. The wrapper transfers the incoming R8d to R9d, clears
R8d and passes fifth argument 1 to **0x14079d210 at 0x140793e77**. The wrapper
returns the same output pointer at **0x140793e7c**, then RET **0x140793e84**.
No E, L, O or C_L pointer is among these explicit arguments.

This exact input avoids the numeric-suffix conversion: length is one, and the
character tested as a separator at **0x14079d2c8** is 0x30 rather than underscore.
The nonempty path stores zero in output+4 at **0x14079d337**. Mode 1 also
excludes the existing-entry replacement paths gated by mode 2. Name-table
contents and hit/miss state are not supplied by these source facts.

## Ready flag and cold initializer

Complete leaf **0x14077bed0..0x14077bed8** returns the address
**0x142b21420**. It makes no call or memory write. **0x14079d257** tests the
DWORD at that address, and zero invokes **0x1407b1170**. The current DWORD
value was not captured and must not be assumed.

Complete **0x1407b1170..0x1407b724f** contains 5,046 decoded instructions. Its
initial guarded loop initializes a separate 256-entry CRC table when required.
It sets the ready DWORD to 1, clears the name-array pointer/count/capacity, and
clears 65,536 hash-bucket pointers. Its direct name-state regions are:

| Native region | Role in this selected path |
| --- | --- |
| `0x142b21420` | Ready DWORD returned by the leaf |
| `0x142c23c78` | CRC-initialization guard |
| `0x142b23690` | CRC table |
| `0x142b23a90` | Hash-bucket pointer array |
| `0x142c23b40`, `+8`, `+0xc` | Name-array pointer, count and capacity |
| `0x142c23b50`, `+8`, `+0x10` | Pool cursor, end and allocation counter |
| `0x142ba3a90`, `+4` | Name-allocation statistics |

This is a write-footprint description, not an assumption that pointers loaded
from those globals are trustworthy or disjoint from the loader.

The initializer creates 59 inline records and 496 records through
**0x140795cf0**, registering each through **0x14079cca0**. The last transfer is
a tail jump at **0x1407b724a**, so there are 555 registrations, not 554. The
candidate identifiers are distinct across both groups. Their text inventory
and individual address/size data remain private.

All 59 inline copy calls resolve to **MSVCR120.dll!strcpy_s** at IAT
**0x141dbbc58**. The selected literal byte counts including NUL are 4..18 and
exactly equal the supplied destination capacities. All 496 factory literals
are also bounded NUL-terminated ASCII in the owned image, with 3..39 bytes
including NUL. Their maximum record requirement is 59 bytes, fitting the
64-KiB pool allocation. These are finite original-literal checks, not a
license to handle arbitrary inputs or to assume secure-copy error callbacks
are harmless.

Factory **0x140795cf0..0x140795e08** preserves the identifier in R13d, stores
`(identifier << 1) | wide_bit` at entry+8, links the supplied previous head at
entry+0xc, and returns the same new entry. The selected cold R9d=1 gives
wide_bit=0. Thus the 496 constructor identifiers join the inline records'
actual headers. Their distinctness excludes duplicate *index slots* after
successful fresh array setup; ordinary hash-bucket collisions are different
and remain allowed.

Registration **0x14079cca0..0x14079cf84** hashes entry+0x14, links the previous
bucket head at entry+0xc, derives the index from header+8, grows/zero-fills the
separate pointer array as needed, and stores the same entry pointer at that
index. It does not register a Scaleform state. Complete capacity arithmetic
**0x1406d74e0..0x1406d754d** has no calls. Array reallocation remains the
allocator boundary described below. The identifier and pointer-array metadata
must not be confused with a native state's category at state+0x10.

## Selected first-name conversion, lookup and addition

Complete predicate **0x14078eba0..0x14078ebcb** scans the UTF-16 text for code
units above 0xff. For this exact input it returns 1, with no writes or calls.
The selected route is therefore:

`0x14070b140 → 0x1407215d0 → 0x1407b8c30`.

The alternative wide hash and wide-comparison routes are not selected by this
incoming name. Numeric-suffix helpers are likewise excluded by the exact input.

**0x14070b140..0x14070b20d** resolves its imports to KERNEL32 `lstrlenW` and
`WideCharToMultiByte`. The selected length including NUL is two UTF-16 units,
so its output capacity is four bytes, below the inline 128-byte capacity. It
uses the stack buffer at its supplied object+4 rather than entering its
allocator branch. The conversion receives code page 0, flags 0, those two
input units, the four-byte output and null optional substitution outputs.
This profile requires ordinary successful selected conversion; failure or
hooked import behavior is not silently assigned a valid output.

The helper returns its inline buffer pointer at object+0x84. Consequently the
cleanup comparison at **0x14079d390** selects the inline case, and the optional
**0x140720f60** heap free at **0x14079d395** is skipped. The caller then uses
**0x1407215d0..0x1407215f8**, whose import is MSVCR120 `strncpy_s`, with a
1,024-byte destination and limit 1,023. These selected valid bounds exclude
secure-copy constraint errors under the ordinary import profile.

The narrow hash's executable code is **0x1407b8c30..0x1407b8cf4**. It has no
calls and writes only its stack; the incoming ASCII and verified cold ASCII
literals avoid the high-byte selector route. The subsequent bytes associated
with its unwind extent contain jump/selector tables, not additional decoded
instructions. The analogous wide hash's code ends at **0x1406dc774**, before
its inline tables. No invalid linear decoding of those data bytes is used as
an instruction witness.

The resulting low 16 hash bits select a bucket. For a valid finite chain,
lookup reads entries and compares selected narrow entries through `_stricmp`.
A matching name returns its recorded identifier into the stack output. A miss
with actual mode 1 increments the separate name count, grows the pointer array
if needed, creates an entry through `0x140795cf0`, links it into the selected
bucket and writes its pointer into the name array. These writes are confined
to the name graph only under the explicit pointer/bounds/allocator conditions;
no direct-global-negative search substitutes for them.

The normal selected return is **RET 0x14079d58f**, then the wrapper return
above, reaching **0x141270cb6**. No current name identifier or hit/miss result
was observed here.

## Allocator and diagnostic boundaries

The name allocator is selected through **global 0x142aee9f8**. It is distinct
from the earlier state-bag heap selector at 0x142c947f0; an admitted profile
for one does not automatically establish the other.

| Helper | Concrete forwarding behavior |
| --- | --- |
| `0x140721150..0x14072118c` | Size/alignment forwarded by tail jump to allocator primary +0x10 |
| `0x1407213b0..0x1407213fe` | Pointer/new size/alignment forwarded to primary +0x18 |
| `0x140720f60..0x140720f90` | Pointer forwarded to primary +0x20 |

Each checks the allocator global and calls **0x141330ca0** if null. That
bootstrap remains unexpanded. A nonnull pointer alone is also not a proof of
ordinary allocator behavior: the preserving profile requires the selected
implementation to perform only the stated allocation/reallocation/free effects,
with valid distinct returned memory, preserved contents and no reentry into
published engine state. Otherwise the virtual or bootstrap is an explicit
unresolved effect boundary.

Invalid index/assertion and duplicate-slot paths are not harmless. They call
**0x14078b240** and/or **0x14077f3c0**; the latter reaches **runtime-object
primary virtual +8 at 0x14077f542**, after constructing diagnostic text. The
complete selected bodies are retained, but their remaining callees and concrete
runtime diagnostic object/effects are not closed here. They may be reached by
invalid state and must not be dismissed because their apparent purpose is
logging. The fixed 555 distinct indices plus valid fresh allocation/growth
exclude the duplicate-slot path in the admitted cold sequence; valid index
arithmetic excludes its assertion paths. No such exclusion is inferred for
arbitrary corrupted or externally mutated name tables.

This diagnostic chain is separate from the Mouse missing-handler helper
**0x140186d10**, which was not probed or invoked by this lane.

## Conditional preservation and next boundary

To use this first-call result as a preservation step, supply all of:

- The actual E/L/O/C_L identities and pre-call state from the existing engine
  prefix, with valid distinct object/storage extents.
- A finite coherent name graph: bounded count/capacity/index arithmetic,
  valid terminated entry strings, valid bucket chains, and pool/array memory
  that does not alias E, L, O, C_L or their state entries. Cold setup instead
  supplies the proved reset/default sequence and valid pool prerequisites.
- The name allocator's own ordinary successful implementation profile, or a
  separately closed null-global bootstrap, plus ordinary successful selected
  string/conversion imports. No custom callback or invalid-parameter handler
  may be silently covered by those primitive assumptions.
- No concurrent/untracked mutation or reentry while globals are partially
  initialized; no assertion/duplicate/error path outside the proved valid
  sequence.

Under those conditions, the selected memory writes are to stack outputs,
fixed name globals, the valid name array and valid pool records. The selected
callees introduce no additional engine/state operation. Therefore L, its O
pointer, C_L, C_L's parent/map and the supplied state entries are preserved
through the first return. This is a conditional static derivation, not a
current process-state observation or a public runtime admission certificate.

The next call **0x141270ccc → 0x141263340(E+0x100, ...)** is outside this
first-name closure. The retained full initializer contains further engine
container/registration/configuration calls. Later **0x1412708b0** begins with
`0x1406f4d20`, and **0x141272b50** contains additional state-removal/installation
branches. None is licensed as a no-mutation barrier by this document. The
MovieDef publication exposures **0x14004e591/0x14004e645** remain with the
separate definition lane. No Mouse acceptance or native-global absence is
inferred from advancing this one prefix.

## Evidence and provenance

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/rea-engine-name-state/`.
Original `verify.py` checks **6,300 unique selected instruction spans** against
the pinned PE, selected import bindings, complete selected call bodies and
finite literal sizes. Its `verification.json` SHA-256 is
`5e44c71de77b38924079d78393e5d090536e3c0d127708dbc1d845b0092e2171`.
Private `factory-literal-bounds.json` SHA-256 is
`05f700f7bedde2e4b05ff63468fb92237892e1fd1abb47e74fead10bee8f59e9`;
it retains address/length metadata without publishing the name inventory.

Independent retained audits are:

| Receipt | SHA-256 | Scope |
| --- | --- | --- |
| `source-profile-review.json` | `b7d3a2ee9ac9409920b9dd287a45c396a683c6f0254e3428495adf5512725f93` | Exact first source arguments/branches, output zero, later caller exposure |
| `cold-init-review.json` | `5c5e39439411df916bcf1210e94e3d48972ac4a85710bc18d02f25e6f947fca0` | Complete cold/register bodies, 555 distinct candidate indices, 59 exact bounded literal copies |

The joined `preservation-receipt.json` binds those checks to the conditional
E/L/O/C_L postcondition and the next unresolved caller exposure. Its SHA-256 is
`6ed7be75d47f07d5f17543cd5edfdbe894f62e24172508d80e97e7c023160f5d`.
It explicitly records that no native execution, current-state capture, script
replay or VM admission occurred.

Five new target-free REA calls were exported and reset in `initializers`,
`helpers1`, `helpers2` and `capacity` bundles; each reset export is byte-identical.
All completed with exit zero, no truncation and unchanged target hash. The
short capacity prefix is superseded by the selected complete 109-byte leaf;
padding, neighboring code and inline table bytes are excluded from instruction
counts. Canonical new records are `ev_b1a7a1f7…`, `ev_a83a3151…`,
`ev_786806c8…`, `ev_7a4a4492…` and `ev_e97039e2…`. Full IDs, input bundle hashes
and exact selected ranges are in the verification receipt.

Retained canonical sources include the complete engine/prefix records from
`rea-mouse-state-preservation/sealed.bundle.json`, `frontend-image-context`,
the name-registration/conversion records from `scale9-texture-load`, and the
original allocation/release records from `fresh-movie-default-state` and
`fresh-movie-default-serialization`. The helper audit also crosschecks the
owned import inventory in `retail-pe-imports.bundle.json`.

Sources/licences: the owner's proprietary executable, used only for private
interoperability study; the original project preservation/fresh-state/Mouse
host docs and retained evidence; the installed reverse-engineer-anything skill
and REA 6.9.0 (MIT tooling), Rizin (LGPL-3.0-only tooling), and original Python
standard-library (PSF) PE/byte/receipt scripts. Imports are identified from the
owned artifact rather than copied external implementations. No third-party
name system or VM code, client bytes, name inventory or asset was added to git.

## Continuation: the actual E+0x100 container

**VERIFIED static continuation:** the next call at **0x141270ccc** passes
**M=E+0x100** to complete **0x141263340..0x1412634e5**. It inserts or updates
a plain native key/value entry. There is no script Value conversion, callback
field, retained stack address or Scaleform state registration in this helper.
The incoming stack descriptor contains a name identifier and a pointer to a
separate eight-byte key-code pair. The helper copies that pair's bits into
entry+4; it does not retain the pointer. Its output is a separate stack index.
The actual optional fourth argument is null.

The selected layout, grounded by reads and writes in the complete helpers, is:

| M-relative offset | Selected purpose |
| --- | --- |
| `+0`, `+8`, `+0xc` | Entry-array pointer, slot count, capacity |
| `+0x10..+0x1f` | Four inline allocation-bit words |
| `+0x20`, `+0x28`, `+0x2c` | Optional bit-array pointer, bit count, bit capacity |
| `+0x30`, `+0x34` | Free-slot head and count |
| `+0x38` | Single inline hash bucket |
| `+0x3c`, `+0x44` | Optional bucket-array pointer and bucket count |

An entry is 20 bytes: name identifier at +0, copied eight-byte pair at +4,
next-entry index at +0xc, and bucket index at +0x10. A matching key replaces
only the identifier/pair at **0x1412634c0/0x1412634c3**. A missing key enters
slot allocation **0x1413216e0**, then bucket sizing **0x141266660**.

The actual constructor has RBP=0 and explicitly clears M's entry pointer/count/
capacity, optional bit pointer/count/capacity, free-slot metadata, optional
bucket pointer and bucket count. In particular, **0x14125df1e/0x14125df25**
clear E+0x100/+0x108. Those are producer checkpoints; preservation through
intervening calls remains a premise from the preceding engine work.

The constructor does **not** directly clear the four inline bit words at
E+0x110..+0x11f or the inline bucket at E+0x138. Neither is assigned a guessed
fresh value here. The checked first insertion establishes their values before
use. No engine allocator-result/extent join was found in the retained scope:
E is the constructor's incoming RCX. Its direct store footprint reaches E+0x354,
which requires valid storage but does not prove a requested allocation size.
The independently allocated O and C_L retain their earlier 0x70/0x58 allocation
identities; non-aliasing with E still needs the valid-live-engine and ordinary
allocation premises.

### Fresh first insertion and initialization-before-read

With the supplied constructor fields preserved, hash count M+0x44 is zero.
**0x141263374/0x141263376** therefore skip the initial bucket read entirely.
Free-slot count is zero, so **0x141321722** appends slot zero, sets entry count
to one, obtains capacity four through `0x1406d74e0`, and requests 80 bytes
through the already bounded reallocating helper `0x1407213b0`.

Allocation-bit append **0x1406d6460..0x1406d655f** changes bit count from zero
to one and bit capacity from zero to 128. It calls complete
**0x1406dba10..0x1406dba91** with old bit count zero. Its nested allocator
**0x1406dbc80..0x1406dbd25** selects the four-word inline case and, because
M+0x20 is null, performs no copy, free or allocation. The caller then invokes
MSVCR120 `memset` through **0x141c6ed8c**, writing exactly 16 zero bytes at
E+0x110. Only afterward are the selected allocation bits set. Thus the first
inline-word read does not depend on unspecified initial bytes.

Bucket sizing chooses one bucket. Complete rebuild
**0x14127a490..0x14127a6cb** calls
**0x1413118c0..0x141311965** with capacity one and null heap pointer; that
selected inline case returns without allocation or copy. At **0x14127a519**,
the rebuild writes -1 into E+0x138 before any selected bucket-chain read. It
then iterates the now-initialized allocation bits through complete
**0x140fa5c10..0x140fa5d11**, linking slot zero into that bucket. These helpers
contain no native-object destructor or user callback dispatch. The selected
external primitive calls remain the allocator and memory imports described
below.

### Finite prefix, growth and preservation boundary

The actual caller makes three bounded loops—10 digit inputs, 26 uppercase
inputs and nine two-character inputs—followed by 52 fixed name/container
pairs. This gives **97 addition attempts**, ending with `0x141263340` at
**0x141271dc9**, returning to **0x141271dce**. This is not a runtime claim of
97 distinct native identifiers: duplicate keys take the proved replacement
path without growing the container.

All 97 source strings were checked in the owned PE: ASCII, length 1..12
UTF-16 units, no trailing underscore-digit suffix. Every selected wrapper
call has the same effective `(0,1,1)` profile proved above. Their conversion
capacities are at most 26 bytes, below the inline 128-byte limit. Therefore
they stay within the bounded narrow-name route, subject to the same valid
name graph, ordinary imports and allocator conditions. The private receipt
keeps source ranges and predicates; the name inventory is not distributed.

For the all-distinct case, the checked scalar growth arithmetic gives:

| New-entry count | Entry capacity | Requested entry bytes |
| --- | --- | --- |
| 1 | 4 | 80 |
| 5 | 22 | 440 |
| 23 | 47 | 940 |
| 48 | 82 | 1,640 |
| 83 | 130 | 2,600 |

Bucket growth occurs at new-entry counts **1, 4, 18, 50**, giving capacities
**1, 16, 32, 64**. Duplicate updates can only delay or avoid these steps.
The maximum 97 allocation bits fit the initialized 128-bit inline storage;
no later bit-storage expansion is selected. Bucket rebuild initializes each
new bucket to -1 before inserting entries. Rehashing rewrites entry chain/
bucket fields and bucket heads, with finite iteration over initialized bits.

Entry and bucket growth forward to the same native allocator global
**0x142aee9f8** through `0x1407213b0`. Bucket rebuild may first request size
zero for its old allocation, then reserve the replacement. An admitted
allocator must provide valid corresponding zero-size/reallocation behavior,
preserve needed existing bytes, return suitably distinct live storage and
avoid reentry. A null selector still reaches unexpanded `0x141330ca0`.
Non-nullness alone does not close those virtual effects. The bounded copy/
zero thunks are independently rebound to **MSVCR120 memcpy/memset** in the
owned import table; ordinary exact-span effects are explicit premises.
Assertion/error paths remain excluded by valid bounded indices and arithmetic,
not by assuming diagnostics are inert.

Under these conditions, writes are confined to stack outputs, the previously
bounded name graph, M's inline metadata/storage and its distinct entry/bucket
allocations. They preserve L/O/C_L and its supplied parent/map/state entries.
This is a conditional static induction over at most 97 additions, not a live
heap observation or proof that the entire engine was initialized successfully.

A further fixed name call at **0x141271df3** has separately checked ASCII
length seven, no numeric suffix and the same wrapper profile. It returns to
**0x141271df8** without a container write. The next distinct helper is
**0x141271e02 → 0x14127bc40**, with RCX=M, EDX=that name identifier and R8
pointing to the stack pair `{0x90, 0xffffffff}`. Its behavior is outside this
continuation closure. Later getter/configuration/global and E+0x148 routes
also remain outside; neither whole-initializer preservation nor current
Mouse handler absence follows.

### Continuation evidence

Private additions are in `evidence/rea-engine-name-state/continuation/`.
The original `verify.py` independently matches **1,938 selected instruction
spans / 8,179 bytes**, concrete branch/write anchors and both memory imports.
Its `verification.json` SHA-256 is
`e2d41a7df5db0c80d791664aeac0e7e8f751d8a8a81d4ecd89ca3feacaeab1ab`.
The separate caller review checks all 1,740 caller instructions / 7,974 bytes
and the 37-byte wrapper; coverage of later bytes is not a preservation claim.

| Private receipt | SHA-256 |
| --- | --- |
| `receiver-profile-review.json` | `ec8b9e746e5c626869b29f627530ea49c4f2ae9f09e58e026e977aec217e28fb` |
| `caller-profile-review.json` | `3d410230bb965ccd8314e174de79ecd8efad4a0cbed78abb418d5359e88ce827` |
| `next-name-profile-review.json` | `b17a6b5dc599076c8b2ac9e4b7a1b014dbf2e984ab39eae8bf0513396c2b3e17` |

The continuation's joined `preservation-receipt.json` binds these inputs and
the earlier first-name receipt to the conditional postcondition before
`0x14127bc40`. Its SHA-256 is
`16116c7325a6fd2211a866617070d7e7c6df55e7acb3eff16092b348667ad2b9`.

Three new target-free REA calls supplied complete selected helpers in records
`ev_fa062002…`, `ev_7238cc9c…` and `ev_62e6b1d5…`; full IDs, ranges and bundle
hashes are in the receipt. All were complete, exit-zero, untruncated and
unchanged-artifact; export/reset pairs are byte-identical. Retained engine,
name/capacity and allocator evidence was reused. The memory-import join also
reuses the original private `world-transform-defaults/copy-inner` receipt.
No target, script, native callback, allocator or import was executed. Source
and tooling licences remain as recorded above; the additional analysis and
checking scripts are original project work. No VM or frontend code changed.
