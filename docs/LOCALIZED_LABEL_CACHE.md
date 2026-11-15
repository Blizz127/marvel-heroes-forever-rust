# Bounded localization prototype cache

`ConfiguredLabels` now retains immutable successful `PreparedPrototype` values
inside each resolver. Repeated saved localization labels reuse that preparation;
every request still validates its label, constructs its own provenance and
resolves locale strings and VM values separately. This is original Rust resource
preparation work, not a claim about native cache behavior or frontend parity.

## Source and acceptance contract

The cache lookup follows the existing ASCII/NUL/length and empty-label checks,
configured path join and path limit, native-derived transformation/hash, exact
prototype ID membership, and ordinary-resource flag check. A cached ID cannot
turn a malformed, missing or flagged request into success. Case aliases that
produce the same existing native-derived identity can share preparation while
retaining their distinct request spelling. No new filename search or path
normalization is introduced.

Open still validates the complete prototype directory, duplicate IDs/member
paths, exact archive membership, all blueprints and their schema relationships,
and typed Globals/UIGlobals/directory Asset configuration. A miss still constructs
and validates `SchemaSet`, reads and parses the selected body and explicit parents,
and checks the localized class/assignment/origin profile under the same fixed
limits. Only a complete successful preparation is eligible for retention. Errors
and partial parses are never cached.

Each resolver owns one archive handle, immutable configuration/index data and
fixed parse/cache limits. The map is private and keyed by prototype ID within
that instance. A separate resolver cannot borrow its entries, and misses do not
reopen a pathname. The existing pinned-source contract still excludes in-place
file mutation or a custom reader that changes bytes; an `Arc` does not make an
external file immutable. A future reload or mutable-limit API would need a new
instance or explicit invalidation.

`ResolvedLabel::prototype()` keeps its existing `&PreparedPrototype` API. Its
private owner is now `Arc<PreparedPrototype>`. Each returned `ResolvedLabel` has
fresh public `label`, `joined_path` and `hash_bytes` storage and copied identity,
plus the existing shared configuration. Mutating public request fields cannot
alter the cache. Results remain usable after the resolver is dropped.

`PreparedPrototype::localized_data` still applies the caller's locale catalog on
every request. `LocalizationCallback` still enforces its own arguments and
request/input limits, commits only its last successful resource, and returns
fresh VM field descriptions. No locale text, `HostResult`, script object,
property setter or failure is cached. Successful preparation does not imply
successful locale resolution or subsequent VM materialization.

## Bounds and API

Existing `open(path, Limits)` and `from_reader(reader, Limits)` use
`CacheLimits::default()`: **64 entries and 256 KiB of accounted decoded storage**.
`open_with_cache_limits` and `from_reader_with_cache_limits` accept explicit
policies; `cache_limits()` returns the fixed policy. Setting either limit to zero
disables admission. Existing `Limits` and existing caller struct literals are
unchanged.

Admission charges `PrototypeValues::statistics().storage_bytes` with checked
addition. A full map, excessive charge, arithmetic overflow or failed optional
map reservation leaves the successful result uncached; it does not reject that
result. There is no eviction or global cache. The entry cap bounds the number of
retained per-result indexes alongside existing per-result structural limits.

The charge is **not exact heap usage or RSS**: allocator overhead, map/index
storage and some field-view storage are outside it. Results retained by callers
can outlive the cache and its owner. The cache policy bounds cache-owned retention,
not all memory deliberately kept by callers. The extra `Arc` allocation uses
ordinary Rust allocation behavior; this change does not claim a completely
fallible allocator redesign.

`cache_statistics()` returns a copy of saturating preparation/retention counters:

| Field | Meaning |
| --- | --- |
| `hits` | Valid request preamble reused an admitted preparation. |
| `misses` | Valid request preamble reached fresh preparation; that preparation may fail. |
| `preparations` | Fresh preparation completed successfully. |
| `admissions` | Successful preparations retained by this cache. |
| `uncached_successes` | Successful preparations returned without retention. |
| `retained_entries` | Current retained map entries. |
| `accounted_storage_bytes` | Sum of admitted decoded-storage charges. |

Rejected request preambles change none of these counters. Existing source-value
statistics remain preparation evidence; a cached value's `loaded_files` is not
current-call I/O. The callback's `resolved` counter still means a successful
resource request, including cache hits; it is not a count of newly parsed bodies
or completed script calls.

## Measured repeat cost

**OBSERVED before implementation:** the frozen original resolver at `eaf94b7`
opened 5,921 members: 5,659 blueprints, 258 type files, two directories and two
configuration defaults. Each of 81 traced successful lookups then read exactly
two members: the requested child and its shared localized-info default. No
blueprint or directory was reread during lookup. The expensive repeated schema
construction operated on already loaded metadata.

The old lookup median was 19.860 ms process CPU, with 2,069 allocations and
3,019,005 requested allocation bytes. A separate schema-construction microprobe
used 2,025 allocations and 3,013,560 requested bytes. That allocation fingerprint
motivated the small success-cache change. The separate schema timing is not
profiler attribution and is not subtracted from whole-lookup timing. Reusing an
owned schema index for distinct misses remains a separate ownership/API change.

**VERIFIED paired original-data measurement:** private code built against clean
commit `5a54fea9dc44168df93592c36327782bfbb57155` uses two fixed handles to the
same pinned archive, one with admission disabled and one with the default cache.
Each of the original 27 labels is resolved in adjacent pairs with alternating
order across four passes. The first three passes count allocation/read/seek
operations; the fourth disables those counters while retaining wrapper branches.
All 216 results match 648 prototype/display/tooltip reference checks against the
frozen independent oracle. All 156 tracked crate files, harness, inputs and binary
remain unchanged across build/run.

| Phase | Calls per mode | Disabled median CPU / wall | Enabled median CPU / wall |
| --- | --- | --- | --- |
| First lookup, counters enabled | 27 | 20.965 / 26.595 ms | 20.968 / 22.763 ms |
| Repeated lookup, counters enabled | 54 | 20.069 / 22.961 ms | 0.003809 / 0.003219 ms |
| Repeated lookup, counters disabled | 27 | 20.184 / 21.992 ms | 0.004186 / 0.003486 ms |

Each traced warm hit issued **zero archive reads and seeks**, with three
allocations and a median 126 requested bytes for fresh request fields. Disabled
repeats used a median 2,070 allocations and 3,019,253 requested bytes, plus two
member reads/seeks. The extra owned `Arc` is also present in the disabled mode;
this comparison measures optional retention within the same implementation.

Enabled open still read the same 5,921 members. The 27 first lookups read 54
members; the following 81 lookups were cache hits. Final enabled counters were
27 misses, preparations, admissions and retained entries; 81 hits; zero uncached
successes; **18,039 accounted bytes**. Disabled mode prepared all 108 requests,
retained none and returned 108 uncached successes. Source-value `loaded_files`
remained two on both modes, correctly recording preparation provenance.

The paired run used release optimization level 2, no LTO, no default features,
no incremental compilation and nice level 19. Its 2026-11-15 10:25:54–10:26:00 UTC
window had 48 affinity CPUs, one-minute load 144.72–145.14 and CPU pressure
`some avg10` 67.48–63.35%. Pages were already warm and were not evicted. Traced
warm enabled CPU ranged 1.871–7.012 microseconds; disabled CPU ranged
13.413–27.353 milliseconds. Counter-disabled warm CPU ranged 2.092–8.535
microseconds versus 14.813–28.205 milliseconds. Scheduling and clock/instrumentation
cost matter, particularly on microsecond hits. Buffered file-read requests are
not physical storage I/O. These timings do not measure frame rate, native retail
execution, server login or a whole frontend startup.

## Focused checks

**VERIFIED:** `nice -n 19 cargo test --offline -j2 --lib
frontend::localized_label::tests` passed seven tests, with 281 unrelated tests
filtered out. The focused fixtures exercise preserved configuration/membership
checks, duplicate member paths, case aliases sharing one prepared pointer,
fresh request ownership, malformed requests after warmup, source isolation,
uncached failures, result lifetime after owner drop, zero/tiny budgets, and
entry/byte-cap fallback. Synthetic checks establish these implementation
contracts, not native behavior parity.

**VERIFIED independent actual-resource checks:** six passes over the 27 original
labels produced 162 source/configuration/locale observations and 4,876 comparisons
against the frozen oracle. These include 324 representable field origins and
324 locale source records. All 54 repeated/uppercase-alias pointer checks passed;
request spelling stayed fresh. Caller mutation, retained-result lifetime,
independent resolver instances, five malformed warm requests, and five cache
capacity policies passed. A one-entry or exactly-first-charge cache retained its
first success while returning later valid resources uncached. Raw prototype
field spans are not exposed by the API and were not falsely counted as compared.

**VERIFIED controlled original-script execution:** a second frozen-crate probe
reuses the original manager pool and functions at source offsets 100257, 108846,
110231 and 110321. After preparing 27 source resources, all 82 successful backend
resource requests reused the exact prepared pointers. The original string caller
returned all 27 oracle strings; 54 direct wrapper returns were distinct VM Objects
with the exact four data fields, flags and original Object metadata. It matched
1,242 resource scalars, 16 configuration values and 109 configured-path
observations. All 15 rejection/state checks passed and preserved the last
resolution, including its prepared pointer.

There were 81 completed script calls and one deliberate failure during VM object
materialization after successful backend resolution. Final callback counters
remain `requests=95` and `resolved=82`; the latter does not mean 82 fresh parses
or 82 successful VM calls. Namespace/native-binding setup remains an explicit
controlled fixture, not evidence of native ExternalInterface installation or the
full frontend lifecycle. All 27 tooltip references in this corpus are zero; a
true-tooltip native example was not tested. Both independent probes kept their
156 tracked crate files unchanged. Their private source/build/input manifests
and failed harness-only compile attempt are retained outside git.

## Private evidence and provenance

All paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. Original game data, labels,
text and private executable artifacts remain outside git.

| Receipt/artifact | SHA-256 |
| --- | --- |
| `localized-label-performance/execution.json` | `a979b9d539508b83ae528e0fc64c0bdd1794034a6a3904943f890ddb514aca1d` |
| `localized-label-performance/measurement.json` | `a05a522cfb135ad468a401443d6761c0e489ce5262c8e4b62634caa31fcd0489` |
| `localized-label-performance/focused-cache-tests.json` | `7e1844e9a5154050078607205d6f24f6708bf7402b766df3319629889aa6feea` |
| `localized-label-performance/cached/execution.json` | `dc957dd211956555721b91568c2ce9c22ccc41406672097052579c1f622684ec` |
| `localized-label-performance/cached/measurement.json` | `0488464c352e5611eb840a09ff37df7ae950f74ef42b56164e76c3e64ec1adcc` |
| `localized-label-performance/cached/result.tsv` | `1ee7bd561f106bce265cc0a21cc2110d94a2ff2299052c9ac505d96c7d914263` |
| Paired measurement executable | `6e2789ab194ee1673b142b8383089dbd7db6ce39fe82ee0b988da78a9735fb2e` |
| `localized-label-cache-check/api/receipt.json` | `61beb3fa5aa1cc2f19d08ca61e96809695b5c1063c4e5ea3021210038a200df2` |
| `localized-label-cache-check/api/MANIFEST.json` | `34a23a1d09e6fe7e38dd641a27c8992c88134d5fc770e3ec9f76fcc04394d690` |
| `localized-label-cache-check/vm/receipt.json` | `a4cf9ba22b7dff36c94ef771da0a234c3a028fa8d148a693fa6c008efa20d0ea` |
| `localized-label-cache-check/vm/MANIFEST.json` | `73db381e4c47e3f4c208cf3003453c653b042fae107c1a9e171f518ba75a74f3` |
| `localized-label-cache-check/vm/execution.json` | `798199ae4c3c90984c1a586ca0ca6915e53f90f1dd91c5f09389b7bdb1885178` |
| Tested `src/frontend/localized_label.rs` | `e2c4cdf6520fc352f2b77da9e2a590ebd7fef1a40c9ae8af8af2c11b35f240bc` |

Implementation and validation scripts are original. Sources read for this
increment were:

| Source | License/ownership | Use |
| --- | --- | --- |
| Project `src/frontend/{localized_label,localized_prototype,localized_data,localized_callback}.rs`, `src/{prototype_values,prototype_catalog,sip}.rs`, `src/calligraphy/prototype.rs`, associated localization docs and prior private validation scripts | Original project; no separate license invented | Ownership/validation seams, accounting, existing locale/callback behavior and private instrumentation. |
| Owner pin `pins/alienware-candidate/Data/Game/Calligraphy.sip`, SHA-256 `1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c` | Owner's proprietary game data; private | Exact original-label resources, metadata, bodies and archive member-read evidence. |
| Owner locale pin `pins/frontend-locale/Data/Game/Loco/eng.all.locale` and four `eng.all/eng.all_{3,7,B,F}FFFFFFFFFFFFFFF.string` members | Owner's proprietary game data; private | Exact locale text and source-location checks. Every complete path/hash is in the API/VM execution receipts. |
| Owner movie `evidence/frontend-startup/MarvelGame-frontend_manager.gfx`, SHA-256 `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4` | Owner's proprietary game data; private | Unchanged selected original pool/caller/wrapper records, authenticated against their prior byte receipts. |
| Prior private `localized-vm-integration/oracle/expected.tsv`, configured-label oracle and callback-backend probe | Original analysis of owner data; private | Independent expected identities, origins, locale values, original wrapper/caller records and object invariants. Underlying sources retain their recorded ownership. |
| `/usr/include/time.h`, `/usr/include/x86_64-linux-gnu/bits/time.h`, `/usr/include/x86_64-linux-gnu/bits/types/struct_timespec.h` | glibc LGPL-2.1-or-later headers | Private probe's process-CPU clock constant, function declaration and timespec ABI only; no implementation copied. Exact hashes are in execution receipts. |

The independent cache-seam source review and exact read scopes are retained in
`localized-label-performance/cache-review/`. No new third-party implementation
or website was read for the cache. Prior native evidence is unchanged and linked
through [LOCALIZED_LABEL_LOOKUP.md](LOCALIZED_LABEL_LOOKUP.md). No native runtime,
ExternalInterface installation, receiver/lifecycle association, game/server or
routine full test suite was run for this increment.
