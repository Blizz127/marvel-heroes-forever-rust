# Native Array prototype category and initial length

Investigation: 2026-11-15 UTC, task `a3d58c579542`.

**VERIFIED, static native control flow:** The pinned AS2 implementation creates
`Array.prototype` as an array-category object, links its parent to
`Object.prototype`, and initializes its length to zero. Its actual `length`
lookup reads that zero-initialized count and returns a successful zero result.
This describes the fresh, unmodified built-in prototype. No retail process was
executed, and no general Array-method or constructor parity is claimed.

The motivating owned manager initializer is the DataProvider action span
30334 + 1622 bytes. It resolves global Array at 30974/30979 and executes
Extends at 30980. The VM lane's bounded decode reports no immediate
Array.prototype length/index access in that initializer. Its deferred
constructor and later Array methods are separate requirements.

## Registration and exact prototype identity

**OBSERVED:** Global setup delegates Array registration to `0x1401f81d0`.
That routine uses interned-name offset `0x148`, which is built-in index 2
under the established `0x138 + index * 8` name layout. Its literal table entry
`0x142add2f0` points to `Array` at `0x14250c234`. It registers the lazy factory
`0x1401ea860` and publishes the corresponding built-in name on the global
object. The name/entry and instruction paths were checked with REA.

The factory first creates the Array constructor object, then allocates a
separate `0xd8`-byte prototype object. It retrieves built-in prototype index 1
at `0x1401ea8ee`, corresponding to Object, and installs that object as the
prototype's parent at object offset `0x30`, instruction `0x1401eaa39`.

Final prototype tables are `0x142514940` and object interface
`0x142514990`. Its category slot points to `0x1400733d0`, which returns `7`.
The ordinary array interface table `0x142512b70` uses the same category
method. These table-pointer reads are retained as ordinary PE observations;
REA confirms the selected category method and table-setting instructions.
This is a native category comparison, not a recovered C++ class declaration
or a complete script-visible `typeof` contract.

At `0x1401eaaa2`, the factory passes this same prototype and the constructor
reference into helper `0x1401b80f0`. The helper creates an object value from
the supplied prototype and writes the constructor's `prototype` member at
`0x1401b81bb`, using flags 3. The name slot `0x3a0`, source table entry
`0x142add548`, resolves to the exact `prototype` literal. The factory also
stores this prototype under built-in index 2 through `0x1401d0080` at
`0x1401eab1b`. Thus the inspected allocation is connected to both the built-in
prototype registry and `Array.prototype`, not merely a nearby empty array.

## Initial count and length getter

The factory keeps `r14` zero from `0x1401ea89c` and performs these writes:

| Object field | Initial state | Instruction |
| --- | --- | --- |
| `+0x70` | zero element-storage pointer | `0x1401ea99c` |
| `+0x78` | zero element count | `0x1401ea9a0` |
| `+0x80` | zero adjacent storage metadata | `0x1401ea9a4` |
| `+0x94` | zero length-lookup flag | `0x1401ea9b7` |

**VERIFIED, static:** The prototype's lookup override `0x1401e7700` handles
constructor metadata specially and delegates the `length` name to the ordinary
array getter `0x140198950` at `0x1401e77ba`. The getter compares the requested
name with built-in slot `0x610`; that is index 155, source table entry
`0x142add7b8`, containing literal `length` at `0x14250d22c`. Both versioned
name-comparison paths lead to the same length branch.

The getter reads interface offset `0x58`, equivalent to object offset `0x78`,
at `0x140198a3e`. It writes internal value tag 4 and the count's 32-bit payload
at `0x140198a4c/0x140198a4f`, returns success, and clears the flag. The fresh
prototype's flag at object `+0x94` is zero, so its alternative zero-count
fallback branch is not taken. The prototype override's return epilogue
preserves the getter's result.

The resulting initial length is zero. The zero-count observation does not
authorize assumptions about sparse indices, enumeration, resizing, deletion,
method calls, alternate flags after mutation, subclass construction, or
prototype mutation. The functional `Array(...)`/`new Array(...)` constructor
profiles remain distinct from this shared-prototype metadata proof.

## Evidence and focused validation

The only target was the owner's private
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Version evidence remains in `evidence/retail-version.json`.

**VERIFIED:** Four target-free REA calls completed with exit zero, complete
untruncated results, and unchanged target digests. All 799 decoded instruction
records, including overlaps/padding, were byte-compared with the original PE.
Trailing partial/invalid decodes were excluded from claims. The base getter's
multiple PE unwind fragments were decoded contiguously through its return.

Code used the verified raw-file mapping `0x140000c00` with explicit x86/64
selection. `.data` has a different section delta: pointer-table reads used
separately translated raw addresses, recorded in the receipt. Literal `.rdata`
reads and all pointer values were compared to the original file mapping.
No setup/config changes, broad automatic analysis, native launch, account or
server operation was performed. REA was checkpointed and reset after the
bounded investigation.

Private evidence under `evidence/rea-array/`:

| Bundle | Evidence ID |
| --- | --- |
| `prototype-construction.bundle.json` | `ev_bdb57af2b8a2a5efa9ed878bfba5cd21004c18b3292ff5e46224b682679614ff` |
| `length-getter.bundle.json` | `ev_f211f682dcbe4e9eaf842ab23644fa33cb1084ab30eb00d3f7e6ad083f94b104` |
| `prototype-property.bundle.json` | `ev_2ae57b7eadfd90dffc216f3a6bd56d5e5ff2ccc642fc8eeabb069f4088006256` |
| `final.bundle.json` | `ev_63575b4c87be3e900824e58ff2a0818a771ed250d7e7f21893b3baf3179ca08d` plus the three records above |

`finding-receipt.json` records file identity, mapping, vtable/name pointers,
check counts and scope. `candidates/array-length-candidates.json` and bounded
ordinary disassembly record candidate discovery; they are not relabeled REA
output. Complete provider identity/architecture/plugin-input limitations remain
in the bundles. This source commit contains only this evidence summary.

## Source ledger

- Owner's pinned retail executable above: proprietary, interoperability study
  only; native registration, prototype fields, category and member lookup.
- Original project private manager decode and native evidence, including
  `evidence/rea-function-calls/`, `evidence/rea-class-registry/`, and the
  built-in name-construction proof in `docs/REA_NATIVE_ALTERNATIVE.md`: reused
  address/name evidence only. No third-party implementation source was read.
- `~/.agents/skills/reverse-engineer-anything/SKILL.md`: mandatory workflow and
  evidence handling; REA's installed tools performed the selected read-only
  disassembly. No toolkit code was copied or changed.

This increment introduced no additional web or external implementation source.
All native bytes, disassembly and decoded game actions remain private and are
excluded from source control and builds.
