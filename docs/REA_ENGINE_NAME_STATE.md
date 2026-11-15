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

The continuations below extend that conditional proof across the actual
E+0x100 container's 135 addition attempts and a local stack initializer,
stopping before the global-receiver call **0x141272764 → 0x1406f4870**.

The subsequent configuration study closes selected cached/disabled branches
and identifies a real virtual-call boundary on a miss. Current configuration
state remains unknown, so it provides no unconditional extension through that
global-receiver call.

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

## Continuation: adapter calls and the next global receiver

**VERIFIED:** complete **0x14127bc40..0x14127bc7e** is a 62-byte adapter to
`0x141263340`. It writes the incoming name identifier and pair pointer into a
local descriptor, supplies a separate local output index and null fourth
argument, and calls the already bounded container helper at **0x14127bc5f**.
After a valid successful insertion/update, it reads the returned index and
M's entry-array pointer, computes `array + index * 20 + 4`, and returns that
pointer. It performs no additional allocation, native callback or object
conversion. This returned pointer's validity depends on the successful
container result; allocation failure is not assigned a fabricated result.

The original caller performs **38** such calls from **0x141271e02** through
**0x14127273a**. The first uses the previously prepared stack pair. Of the
remaining 37, 32 use **0x14125ea50..0x14125ea5d**, which stores `{EDX, -1}`
and returns RCX; five use **0x14125ea60..0x14125ea6a**, which stores
`{EDX, R8D}` and returns RCX. These five actual inputs have EDX=0 and second
DWORDs 0, 1, 2, 4, 3. **0x141598070..0x141598073** is a three-byte DWORD
load from RCX into EAX, used to obtain the name identifier. Neither scalar
constructor nor this load calls another function. Captured padding and the
neighboring function after these leaves are excluded from the selected proof.

Actual RCX remains M=E+0x100. Each pair is a live stack object during the
synchronous call. The underlying helper dereferences its pair pointer and
copies the eight bytes on both insertion and replacement; it does not retain
the stack pointer as the stored payload. The caller does not use the adapter's
returned entry pointer as a later callback argument or object receiver in this
segment. Therefore the extra wrapper introduces no new ownership operation.

All 38 corresponding fixed name inputs were independently checked: ASCII,
length 3..17 UTF-16 units, no trailing underscore-digit suffix and the same
bounded wrapper profile. At most 36 conversion bytes fit the inline buffer.
The actual identifier values and lookup hit/miss results remain unobserved.

### Crossing the inline allocation-bit capacity

There are now at most **135** new entries across the preceding 97 attempts
and these 38. A duplicate remains an ordinary replacement and can only reduce
or delay growth. This larger bound requires the already retained dynamic
bit-storage branch; the earlier 97-attempt inline-only result is not silently
extended past its limit.

At new-entry count **129**, `0x1406d6460` sees old capacity 128 bits, requires
five words, and passes old/new word counts 4/5 to the checked scalar growth
arithmetic. The resulting capacity is **22 words / 704 bits**. Through
`0x1406dba10 → 0x1406dbc80`, the selected first dynamic allocation:

- Requests **88 bytes** through `0x1407213b0` with the old heap pointer null.
- Stores the returned pointer as the allocation bitmap's heap pointer.
- Copies **16 initialized inline bytes** through the checked `memcpy` thunk.
- Zeroes the remaining **72 bytes**, beginning at new pointer+16, through
  the checked `memset` thunk.

The allocation is distinct valid storage under the inherited ordinary allocator
profile. No old bitmap heap allocation is freed on this first transition.
The previous 128 bits and cleared padding are preserved, and new bits are
initialized before the subsequent iterator reads them. No second bitmap growth
is needed by count 135. This is bounded arithmetic and branch analysis, not a
native allocator observation or a claim about unspecified allocation failure.

The all-distinct maximum also adds entry-array growth at count **131** to
capacity **196 / 3,920 bytes**, and bucket growth at count **114** to
capacity **128 / 512 bytes**. The same already bounded entry copying, bucket
initialization/rehash, valid index and no-reentry conditions apply. The extra
heap bitmap is another distinct allocation in the conditional write footprint;
no live alias is inferred from a static field name.

### Final stack initializer and precise next boundary

After the last adapter returns to **0x14127273f**, the caller passes its stack
object at RSP+0x48 to retained **0x14059c400..0x14059c40d**. This 13-byte leaf
zeroes exactly two QWORDs at RCX/+8 and returns the same pointer. It has no
callee, allocator or destructor. In this caller its effect is solely a
16-byte local initialization, returning to **0x141272749**.

The next unresolved call is **0x141272764 → 0x1406f4870** with:

| Argument | Actual producer |
| --- | --- |
| RCX | Pointer loaded from global `0x142ae68d0` at `0x14127275d` |
| RDX | Fixed owned-image source at `0x1421bc9a0` |
| R8 | Address of the just-zeroed 16-byte stack object |
| R9 | Address `0x142af1a00`, not a value loaded from that address |

This is a new global receiver and output path. Its implementation and effects
are not covered by the name/container result. Under the inherited valid-live-
engine, disjoint allocation, valid graph, ordinary primitive and no-reentry
conditions, the bounded continuation preserves actual L/O/C_L through this
call site, **before entering `0x1406f4870`**. No conclusion is made about the
callee, later engine constructor return, current state-4 cache, or Mouse.hide.

### Adapter continuation evidence

Private files are in `evidence/rea-engine-name-state/post-container/`.
The original verifier matches **1,214 selected instruction spans / 4,632
bytes**, including the adapter, exact scalar leaves, selected caller and
reused growth helpers. `verification.json` SHA-256:
`7695b786a911c841c0b4c321876d44bde142c92411eb626dbe82305ed1e3b3af`.
The first heap-bitmap expansion and updated capacities are independently
recorded scalar derivations; they do not execute a target or allocator.

Independent `caller-review.json` checks **1,391 instruction spans / 5,207
bytes** and **868 literal bytes**, including exact stack lifetimes, all 38
source inputs and the expanded allocation bounds. Its final SHA-256 is
`2c9aa291751cda277f137ba179adbcc56f4f4c4ea20f27cd4c15ff0dcfb4efc8`.
The joined `preservation-receipt.json` binds both checks and the preceding
preservation receipt; SHA-256
`3374680bede8dcc31d921ca07f81e6bcc43069e690b4cfca133f7211545eb7bf`.
The independently sealed earlier 97-entry container review remains separate
and unchanged; it is not substituted for the new heap-bitmap branch proof.

One new target-free REA call produced complete selected helper record
`ev_4ca3dff4f343e83ba06a890185be19a0934ba64653b9d353d5ada60c173b6424`.
The selected 62/13/10/3-byte bodies are byte-checked; padding and neighboring
instructions are excluded, including an invalid decode outside all selected
leaf ranges. The complete reply is exit-zero, untruncated and unchanged-target;
`helpers.bundle.json` and its reset export are identical. The 13-byte local
initializer reuses `ev_df97bcfd…` in the preservation sealed bundle. All other
source/tooling provenance remains as above. No native process, AVM1 replay,
VM implementation or frontend behavior changed.

## Continuation: configuration lookup and its real external boundary

**VERIFIED:** complete **0x1406f4870..0x1406f4af1** first clears its supplied
output array through `0x1406ef710`. In the actual caller the output was just
zeroed, so that helper skips element destruction, free and reallocation and
only retains a zero count. It then calls **0x1406f1400** with the original
global receiver G, the address **0x142af1a00** as a UTF-16 source, and flag 0.
That address is passed directly; it is not a pointer loaded from the global.
Its current string contents and length were not captured.

This is **not** an unconditionally read-only lookup. A missing entry can invoke
a different global object's virtual function and proceed to insertion/loading/
logging. The selected flag 0 does not suppress that route. Consequently the
previous unconditional-before-call preservation frontier becomes a set of
conditional branch frontiers below; no actual branch is selected from guessed
configuration contents.

### Concrete receiver producer, without a current-state assumption

The native bootstrap caller at **0x1413330f8** supplies factory
**0x1406f1160** in its ninth argument slot, stored at RSP+0x40 before calling
`0x14078c980` at **0x14133313e**. The callee's prologue maps that slot to
RBP+0x350, loads it at **0x14078c9d4**, and saves it at RBP-0x28. It invokes
that function pointer at **0x14078defd** and immediately publishes the returned
RAX to **global 0x142ae68d0 at 0x14078df00**. This is a checked native caller
and publication path, not a runtime observation of the owner's current client.

Complete factory **0x1406f1160..0x1406f11b2** requests **0x54 / 84 bytes**, with
alignment 8, through `0x140721150`. Allocation failure returns null. On the
successful branch it installs primary table **0x141de8360**, clears the
selected map fields at G+8/+0x10/+0x28/+0x30/+0x38/+0x44/+0x4c, and returns
the object. The last QWORD clear includes **DWORD G+0x50=0**. As with the
previous container, inline storage holes are not assigned invented zeros.

These are fresh producer facts only. Startup makes additional configuration
setup/load calls immediately after publication, beginning at **0x14078df21**.
The study does not close that entire bootstrap or later receiver lifetime.
An original limited REX MOV/LEA candidate scan also found another possible
store; it is not a complete cross-reference inventory or a sole-writer proof.
No current G pointer, G+0x50, map contents, filename or file availability is
inferred from the constructor or on-disk image.

### Initial lookup, selected writes and the external miss path

Complete **0x1406f1400..0x1406f1644** copies the supplied filename into a
bounded temporary string, searches the map at **G+8** through
`0x1406f20e0`, and releases the temporary. Complete
**0x1406f20e0..0x1406f21cf** follows bucket/next-index chains of **108-byte**
entries and writes only the caller's local index. A found value is selected
at entry+0x10. For valid maps and strings this read path has no object virtual
call or cache mutation; its temporary allocation/free and string comparison
still require the stated ordinary primitive profile.

If the entry is absent, the branches are exact:

- **G+0x50 != 0:** return null without the subsequent external check.
- **G+0x50 == 0 and actual flag 0:** load **[0x142ae68e8]**, select its primary
  table, and call **virtual +0xc0 at 0x1406f14fb**, passing the filename source.
  A negative EAX returns null. A nonnegative result proceeds to unclosed
  insertion/loading/logging calls, including `0x1406e6ee0`, `0x1406fc150` and
  `0x14077f590`.

The concrete second global receiver and its current implementation are not
proved here. This virtual dispatch is a real effect boundary, even if an
apparent role suggests a filesystem query. A miss cannot be converted into a
no-op or a preserving failure without that additional evidence. The factory's
fresh zero field actually permits this route; it does not exclude it.
If all fresh factory map fields survive unchanged, the empty initial map and
actual flag 0 necessarily select this virtual path. That fresh checkpoint is
not assigned to the owner's current engine.

### Cached entry: section lookup, empty success and output construction

When the first map supplies an existing entry, `0x1406f4870` copies the fixed
owned-image source at **0x1421bc9a0**. Its checked length is 16 UTF-16 units,
so the temporary requires **34 bytes including NUL**. It searches a second
map through complete **0x1406f22d0..0x1406f23c3**, whose entries have stride
**96 bytes**, then releases that temporary. This lookup likewise changes
only its local result index under valid read-only map conditions.

Both lookup helpers call the retained wide hash and complete comparison
**0x1406d57c0..0x1406d5856**. The comparison's concrete import is
**MSVCR120 `_wcsicmp` at IAT 0x141dbbef0**. Negative string lengths reach
assertion helpers and are outside the valid-input profile. The wide hash's
202 bytes of jump/selector data were independently matched to retained raw
bytes and the PE: every destination remains within its checked no-call code.
Its CRC reads are bounded to 256 DWORDs. The table is not decoded as code, and
no hidden external dispatch is assumed away.

A missing second-map entry returns **EAX=0** with the output still empty.
A found section selects its entry+0x10 and initializes the retained bit
iterator. If its bit count is zero, the iterator produces a nonnegative index
and the caller immediately skips output construction, even though a readable
first bitmap word can have unspecified contents. No invented zero bitmap
padding is needed for this zero-count branch. The selected local cleanup
flags remain zero and **0x1406f4acb returns EAX=1**, with the output unchanged.

For a nonempty section, preservation is closed only up to the first output
append **0x1406f49b8 → 0x140828b20**. Later calls include key/string preparation
`0x1407b85c0`, formatting `0x140775830`, iterator advancement `0x140a7aa00` and
conditional cleanup `0x14098f570`. Those effects are not declared ordinary
merely because their apparent purpose is string construction. No nonempty
result, bounded output count, formatting result or normal completion is
fabricated. This lane did not enter the separate root diagnostic/Environment
investigation.

### Preservation consequences and caller frontiers

In addition to the preceding E/L/O/C_L premises, the closed lookup branches
require a valid live G, bounded terminated filename, valid finite maps and
readable string/bitmap storage. All temporary allocations and graph storage
must remain disjoint from protected engine/loader state. The ordinary
allocation/free, `_wcsicmp`, memory-copy, cookie/ABI and no-reentry conditions
remain explicit. They do not cover the new virtual/load/format boundaries.

| Supplied branch state | Closed result and next unresolved frontier |
| --- | --- |
| Initial map miss, G+0x50 != 0 | Outer result 0; original caller reaches `0x141272aff → 0x141260940(E+0x148, E+0x100)` |
| Initial map miss, G+0x50 == 0 | Before `0x1406f14fb`, primary virtual +0xc0 of `[0x142ae68e8]`; no return/effects assumed |
| Cached first entry, missing section | Outer result 0; same E+0x148 fallback at `0x141272aff` |
| Cached section, zero bit count | Outer result 1, zero output count; caller reaches `0x141272b09 → 0x141269560(E+0x100, 0)` without the fallback |
| Cached section, nonempty iterator | Before output append `0x1406f49b8 → 0x140828b20`; later caller behavior remains unproved |

The caller tests EAX for **zero**, not positivity. Any hypothetical nonzero
return with signed output count <=0 skips the configured loop and the fallback;
that observation does not license new return values beyond this callee's proved
branches. With positive output count the first later unclosed row helper is
`0x1412727dc → 0x1406dacc0`. Dynamic row-name sources are outside the earlier
fixed ASCII bounds. Later cleanup/helper calls are also not closed by this
branch map.

Thus conditional preservation extends through the stated no-external branches
to their exact caller frontiers. It does not extend unconditionally through the
lookup, select the owner's current branch, prove whole-engine preservation,
or establish current state-4 absence or Mouse.hide acceptance.

### Configuration evidence

Private evidence is under `evidence/rea-engine-name-state/config-lookup/`.
The original verifier matches **3,020 instruction spans / 12,300 bytes**,
actual factory forwarding, import binding, fixed-source bounds and 202 bytes
of hash dispatch data. `verification.json` SHA-256:
`a7d4b7e4dc0197aa9d32486365f42d9dac6c9502b4b1cecf72074682af08e073`.
Full producer-body byte coverage is not a claim that every bootstrap callee's
effects have been analyzed.

The independent caller review checks **298 spans / 1,143 instruction bytes**
and **132 fixed literal bytes**. `caller-review.json` SHA-256:
`63a57a0bad17ee4cb63df8a336edc506507880b90aa72733b30a88b3f0a1527d`.
It distinguishes return-zero fallback, empty success, positive-count loops,
dynamic sources and later cleanup instead of assigning them one shared effect.

Independent `global-producer-review.json` checks the 2,007 producer
instructions / 8,563 bytes, 82-byte factory and 75-byte caller join; SHA-256
`202d6fd69d37a243e1dad7d58016e7ab301f428d98b3c3a0d7162478c21dfc4d`.
The saved factory slot is later exposed and rewritten elsewhere in the
bootstrap; the forwarding claim ends at its immediate publication.

Joined `preservation-receipt.json` binds these checks to the separate branch
frontiers and inherited conditions; SHA-256
`1303beb65712b3af18df4ace67d8b9a978245689452833ded19bfe9aa89e03da`.
It explicitly declines an unconditional extension through the lookup.

Five new target-free REA records are sealed in `entry`, `lookup-helpers`,
`find-helpers`, `producer` and `factory` bundles. All are complete, exit-zero,
untruncated, unchanged-target and exported/reset with identical paired files.
Full IDs and hashes are in the verification receipt. The original candidate
scanner and independent retained-source review add no target execution or
current-state capture. Source/licence provenance remains the owner's private
PE and prior original project evidence with the tooling recorded above; no
config contents, native bytes, VM implementation or host behavior entered git.

## Continuation: selected external object and native path-size callback

**VERIFIED static binding:** the selected bootstrap passes static object
**F=0x1429e1660** as argument six at **0x141333115**. The callee loads that
slot into RBX at **0x14078c9cd** and publishes RBX to **0x142ae68e8 at
0x14078ca32**. The owned image initializes F's primary pointer to
**0x142231048**. Immediate initialization dispatch at **0x14078ca3f** selects
slot zero, **0x14058fea0**, a three-byte `RET 0` with no write or callee.

The .data source has a different file mapping from .text/.rdata: F's pointer
is at file offset **0x29dfe60**, captured at raw REA address **0x1429e0a60**.
The checker explicitly joins those bytes to F's preferred image address; it
does not mistake the raw mapping for the native object address. Relevant
primary slots are independently byte-bound:

| Slot | Selected native function |
| --- | --- |
| `+0` | `0x14058fea0`, immediate-return initializer |
| `+0xa8` | `0x1406ec770`, first path-string transform |
| `+0xb0` | `0x1406ec390`, second path-string transform |
| `+0xc0` | `0x1406f11c0`, callback selected by the cache miss |
| `+0xe0` | `0x1406f6ea0`, host path-size query |

This is a producer/image-table join, not a current-process table observation.
The caller also publishes F+0x30 through another global, so F is not assumed
isolated. Its current fields, aliases, table stability and lifetime remain
conditions. The two calls between the argument load and publication,
`0x1407215a0` and `0x14119e300`, preserve RBX under the normal nonvolatile-
register ABI; their other effects remain separate prerequisites. The selected
slot-zero initializer above adds no direct effect.

### Callback order and exact return decisions

Complete **0x1406f11c0..0x1406f1343** calls F+0xa8 with the original filename
and a local output string, then F+0xb0 with that output and a second local
string. It calls F+0xe0 with the second string's data and saves the low DWORD
result. It releases both temporary buffers through the existing allocator
wrapper.

Only an exact result **0xffffffff** triggers the fallback. That branch calls
F+0xa8 again with the original filename, passes that first-form path directly
to F+0xe0, releases the new temporary and returns the second DWORD. It does
not repeat F+0xb0. Every other first result is returned unchanged. The outer
`0x1406f1400` then tests the sign bit of EAX at **0x1406f1504**. Therefore an
initial result in `0x80000000..0xfffffffe` skips the callback fallback but
still takes the outer negative-return path. No existence Boolean or full
64-bit file length is substituted for this exact behavior.

### Host imports: an open/query/close sequence

Complete **0x1406f6ea0..0x1406f6f0d** binds its imports to KERNEL32
`CreateFileW`, `GetFileSize` and `CloseHandle`. The first call receives the
supplied path, access **0x80000000**, share **1**, null security attributes,
disposition **3**, attributes **0x80**, and null template handle. These are
read access, read sharing and opening an existing path with normal attributes.
This native operation can create an OS handle; it is not a pure metadata
function. [CreateFileW documentation](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).

An invalid returned handle yields DWORD `0xffffffff` immediately. Otherwise
the code calls `GetFileSize(handle, NULL)`, saves its low DWORD, attempts
`CloseHandle(handle)`, ignores the close result and returns the saved value.
It neither asks for the high size word nor queries last-error information.
Thus `0xffffffff` remains ambiguous at the OS contract boundary, and the
native caller still applies the fallback above.
[GetFileSize documentation](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfilesize),
[CloseHandle documentation](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle).

The investigation performed none of these operations. The selected path,
handle, OS result and close outcome remain unknown. An ordinary import profile
may preserve protected engine memory while still changing OS handle and thread
error state; those are separate effects.

### Path helpers and bounded memory footprint

The complete path methods and their selected string dependencies are retained.
F+0xa8 first constructs a distinct UTF-16 copy through `0x1406d3b90`. It tests
its first two characters against the owned literal consisting of two dots,
using `0x1407087a0` and the concrete `_wcsnicmp` import. The matching branch
uses `0x1407d0df0` and counted-string concatenation; the other branch returns
the independent copy. It is not assigned a guessed filesystem-normalization
API or a guessed current path.

`0x1407d0df0` returns the shared module-directory buffer at **0x142c23f20**.
If its first WORD is zero, it calls `GetModuleFileNameW` using the module
handle loaded from **0x142c48b28**, a **512-WORD** destination and then scans
for NUL. It shortens the buffer at the selected last separator and returns its
address. The native code ignores the API return value. Its unbounded NUL scan
occurs before the final index-range check, so the admitted branch requires an
already valid bounded cache or a bounded terminated result from the import.
Failure/truncation without that prerequisite is not silently accepted. The
range-failure branch reaches `0x141c6f1b8`, which stays outside the normal
profile. [GetModuleFileNameW documentation](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulefilenamew).

F+0xb0 reads F's counted-string fields and selected globals. Its visible and
transitive normal writes are to caller outputs and local copied strings,
subject to aliasing conditions. `0x140789a90` is an eight-byte leaf returning
the address of the command-line buffer **0x142b19180**; it does not execute
that text. Retained case-insensitive search/replace, append and concatenate
helpers use bounded ordinary CRT/allocation contracts. Their jump/selector
tables are checked data with destinations inside the selected code, not
instruction-shaped writes inferred from table bytes.

One important write is real: `0x1407828b0` temporarily writes a NUL into the
matched source and later writes the pattern's first WORD back. Because its
search can ignore case, that is not guaranteed to restore the original code
unit. Both F+0xb0 call sites pass a local copied string, so the write remains
within temporary storage only under the proved call-site and disjoint-buffer
profile. The helpers are not described as universally read-only.

The allowed write footprint therefore includes stack/output objects, distinct
owned temporary buffers, allocator metadata and the shared module-directory
cache. F fields and protected E/L/O/C_L are not direct write targets on these
normal selected methods. That statement depends on stable valid F descriptors,
valid module/command-line buffers, bounded arithmetic and disjoint allocations;
it is not a statement that arbitrary aliased pointers, overridden tables or
reentrant imports preserve the engine.

### Resulting preservation frontier

With the selected F/table identity, valid bounded path state, ordinary Win32/
CRT/allocator contracts, successful required allocations, valid stack/cookie
state and no reentry or concurrent mutation, this callback preserves actual
E/L/O/C_L and the supplied bag state while allowing the effects just listed.
No current host operation is run or declared successful by this static proof.

A signed-negative final DWORD makes `0x1406f1400` return null and the outer
lookup return zero; the next engine helper remains **0x141272aff →
0x141260940(E+0x148, E+0x100)**. A nonnegative DWORD enters a distinct path:
a bounded filename copy and local empty-container setup, then the next
unclosed cache insertion **0x1406f15b8 → 0x1406e6ee0**. Later loading
`0x1406fc150` and logging `0x14077f590` are not covered. Current OS outcomes
cannot select either branch, and neither branch proves whole-engine or
current Mouse-handler absence.

### External-object evidence and provenance

Private evidence is under `evidence/rea-engine-name-state/external-lookup/`.
The original verification matches **1,289 instruction spans / 4,720 bytes**,
source/table mappings, import bindings and dispatch data. Its
`verification.json` SHA-256 is
`347a88457ae3f29f2cdc66a805e3098fc2ee6b013884424bc81c7856e27d7f8b`.
The independent producer review's SHA-256 is
`f64331c81e940701e04a76bfb4e13cfc3cacbd77f9bf568f656e27539892917a`.
The independent retained-path review covers 15 executable scopes, both
internal dispatch tables and all selected call inventories. Its
`path-review.json` SHA-256 is
`395924b19878530dd3c27bac73bfdbe065d273e10bf5b8fef202afa16493585e`.
The final verifier wording distinguishes an ordinary `CreateFileW` failure
sentinel, whose branch is covered, from invalid descriptors or allocation
failure, which remain excluded. The earlier draft hash is superseded.
Five new REA calls were exported/reset in `entry`, `methods`, `path-entry`,
`path-helpers` and `string-leaves`; completed output is untruncated,
exit-zero and unchanged-target, and each reset export matches its bundle.

The primary Microsoft API pages linked in this section were used only to
interpret the observed import arguments and returns. Documentation licence:
[MicrosoftDocs/sdk-api CC BY 4.0](https://github.com/MicrosoftDocs/sdk-api/blob/docs/LICENSE).
Exact URLs, names and uses are also recorded in private `provenance.json` for
the integrator's `docs/PROVENANCE.md` update. No SDK implementation or sample
code was copied. Owned-artifact and original tooling provenance remains as
recorded above. No native process, script replay, file/config operation, VM
code or host acceptance was performed by this study.

## Continuation: nonempty configuration output

**VERIFIED selected static bodies; conditional output construction:** the
separate cached-section branch of `0x1406f4870` can now be followed through
normal output iteration. This does not close the distinct cache-miss insertion
and load helpers above, establish current configuration contents, or choose a
native branch.

### Append and row construction order

At **0x1406f49b8**, `0x140828b20` receives the output descriptor in RDX. It
increments and stores its signed count before any row construction. If needed,
it computes capacity through the existing `0x1406d74e0` rule, reallocates
**16 × capacity** bytes with alignment eight, and stores the new pointer. It
returns `base + 16 × old_count`; it initializes no row bytes. Valid monotone
counts, nonoverflowing sizes, successful allocation and disjoint live storage
are required. A null returned row skips construction without undoing the
count, so no transactional or graceful-failure behavior is claimed.

The caller then converts the selected entry's key through
`0x1407b85c0 → 0x1407b8600 → 0x140795e10`. The first helper zeroes a local
16-byte string descriptor. The later helpers use the current name table at
**0x142c23b40**, its count at **0x142c23b48**, the selected entry's encoding
flag and text, and its separate biased numeric suffix. They may copy UTF-16,
convert byte text through the observed `MultiByteToWideChar` import with
code-page argument zero, and append an underscore plus a signed decimal
suffix. The current entry, encoding, suffix and conversion result are not
captured. A numeric index check alone does not prove a valid table or string.

The byte conversion ignores its import return, so the supported ordinary
profile explicitly requires a valid terminated result. The decimal suffix
helper is complete and uses integer division and the checked digit units;
its signed 32-bit input fits its local buffer. The complete counted UTF-16
append helper stores the new count before growth/copy, replaces the old
terminator when appending, and copies the new terminator. Safe arithmetic,
valid terminators, disjoint copy extents and stable lifetimes remain required.

At **0x1406f4a70**, the actual fixed format at **0x141de44a0** is `%s=%s`.
Arguments are the converted key and the row's UTF-16 value, with a checked
empty literal used when the relevant count is zero. Negative descriptor
counts take a diagnostic route outside this profile; positive counts alone
do not certify the pointers or terminators.

`0x140775830 → 0x140781d50` first formats into a 128-WORD stack buffer.
It reuses the independently sealed `0x1407d2a80` adapter to the concrete
`MSVCR120!_vsnwprintf_s` import; no diagnostic formatter body was probed.
A result at least the capacity or exactly -1 enters a heap-buffer retry with
growing capacity. The accepted path requires finite successful formatting,
safe growth and **0 ≤ returned count < capacity**. Other negative results
would pass the native comparisons and index before the buffer; those error
paths are not accepted. No universal termination or arbitrary-format purity
is claimed.

On the admitted return, the code writes a terminator and constructs the fresh
row with `0x1406d3b90`. That constructor does not inspect or release old row
contents. Under the ordinary successful fixed-format contract, the copied
string is key, equals sign and value; if their lengths are k and v, its
count and capacity are **k + v + 2** UTF-16 units including NUL. This is an
**INFERRED conditional import-contract result**, not executed output or a
current configuration observation. Both the array count and row metadata
become visible before all allocations/copies finish, so nonreentry and valid
exclusive access are substantive conditions.

### Iterator, cleanup and protected state

Complete `0x140a7aa00` is call-free. It modifies only the iterator's word/bit/
index fields at offsets **+8, +0xc, +0x18, +0x1c and +0x20**, plus its own
stack temporary. After iterator initialization, the caller explicitly sets
the separate flags at **+0x34 and +0x38** to zero at **0x1406f4992/4997**.
The zero invariant starts at those stores. The admitted advance does not
change them, so the later `0x14098f570` cleanup call is unreachable on this
profile. Its body was not analyzed and no general cleanup effect is assumed.

The iterator reads a current bit word before testing exhaustion. Its valid
word extent, coherent logical count and stable bit/row storage must therefore
be supplied. A padding bit beyond the logical count is rejected by the
caller's signed index/count test before row access. This proof does not
silently import the earlier 135-entry limit into configuration output.

Temporary key buffers are released after each normal construction. Each
output row owns its copied string under the stated profile. The allowed
writes are the caller's output descriptor, allocated array/strings, local
iterator/temporaries and ordinary allocator/CRT state. With valid disjoint
E/L/O/C_L extents, stable source data, successful nonreentrant imports and
allocations, these writes preserve the protected engine/bag state. Invalid
parameters, diagnostic callbacks, allocation failures, concurrency and
unexpected CRT results remain excluded.

### Joined branch frontiers and receipts

Normal output iteration returns EAX=1 at **0x1406f4acb** and resumes the
engine caller at **0x141272769**. The next unresolved work remains split:

| Selected condition | Next unclosed native helper |
| --- | --- |
| Lookup returns zero, including a signed-negative final host size result | `0x141272aff → 0x141260940(E+0x148, E+0x100)` |
| Lookup succeeds with nonpositive output count | `0x141272b09 → 0x141269560(E+0x100, 0)` |
| Lookup succeeds with positive output count | `0x1412727dc → 0x1406dacc0`, the first output consumer |
| Cache miss has a nonnegative host size result | `0x1406f15b8 → 0x1406e6ee0`, before the separate load/log path |

No current cache state, external result, row count or branch is selected by
these static proofs. They do not establish whole-engine initialization,
current state-4 absence or permission to accept Mouse requests.

Private `evidence/rea-engine-name-state/config-output/verification.json`
checks **865 instruction spans / 3,071 bytes** across 15 complete selected
functions, fixed literal/digit data and concrete import bindings. SHA-256:
`800246d0acec52dbfda1e77a40141f90db24bdabfdaf5eea686e8170e3c584b1`.
Its five new REA records are complete, untruncated and unchanged-target;
`complete.bundle.json` and `reset.bundle.json` have the same SHA-256,
`0d200fa1c3aeb5fe52a742cab462c0dff0883bf6aada66245647574e3cd52190`.
The reset clears five records and retains no target. Both original verification
scripts were rerun successfully before this documentation handoff. Their
instruction counts overlap on shared helpers and are not added as a unique
coverage total. No runtime tests were run for this docs-only change.

The independent output audit reproduced the 865 spans and all 47 listed
calls, and checked the two outgoing tail routes separately. Its
`config-output/independent-review.json` SHA-256 is
`fb81998b099f54428c7e7acf70226e2da267950393ee83e928fa1e95a9bf5b3e`.
The joined `external-lookup/preservation-receipt.json` binds the original
verifiers, independent reviews, prior caller/preservation receipts, import
provenance and export/reset pairs to the four separate frontiers above.
SHA-256:
`0af4620134dcf7544931c505e1f46718c7a35604c6dafde0fd7803ee163ae193`.
