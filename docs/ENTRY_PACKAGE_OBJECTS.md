# Initial entry package object boundaries

`ObjectProperties` now admits the four native map classes `Engine.Level`,
`Engine.Model`, `Engine.Polys` and `Engine.World` for the existing UE3 **868/3**
profile. Admission requires a `Core.Class` import directly beneath the root
`Core.Package` named `Engine`. A matching short name, local class export or
custom subclass is insufficient. Every class using this four-byte reader rejects
the native state-frame flag `0x0200000000000000`.

`WorldInfo` continues through `world_scene::SavedSceneObject`; its actor prefix
is not added to the generic four-byte reader. Earlier generic CLI rejection of
the entry exports did not establish that the actor reader lacked WorldInfo
support. This distinction matters when interpreting the initial map findings in
[FRONTEND_STARTUP_MAP.md](FRONTEND_STARTUP_MAP.md).

## Private inputs and exact spans

All paths below are relative to
`/home/blizz/.local/share/marvel-heroes-forever-rust/` unless absolute.

| Input | Private path | SHA-256 |
| --- | --- | --- |
| Owner's initial map, 1,221 stored bytes | `pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelEntry.upk` | `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` |
| Owner's pinned 1.52.0.1700 Win64 client | `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` |

**OBSERVED:** the package contains 20 names, six imports and five exports.
The header independently identifies file version 868 and licensee version 3.
The following ranges are relative to each decompressed export, with exclusive
ends; they are not compressed-file offsets.

| One-based export | Native class | Serialized bytes | Prefix | Property list including `None` | Opaque native tail |
| ---: | --- | ---: | --- | --- | --- |
| 1 | Level | 704 | `[0,4)` | `[4,12)`, zero tags | `[12,704)`, 692 bytes |
| 2 | Model | 228 | `[0,4)` | `[4,12)`, zero tags | `[12,228)`, 216 bytes |
| 3 | Polys | 24 | `[0,4)` | `[4,12)`, zero tags | `[12,24)`, 12 bytes |
| 4 | World | 140 | `[0,4)` | `[4,12)`, zero tags | `[12,140)`, 128 bytes |
| 5 | WorldInfo | 119 | `[0,26)` | `[26,119)`, three tags; `None` at `[111,119)` | Empty |

The four short prefixes contain serialized NetIndex. The independent Python
property grammar and the original Rust reader agree on these boundaries and on
the WorldInfo tag/payload spans. That grammar uses Rust-extracted exports and the
same retained name dictionary; it is not independent package decompression.
Raw exports, names, values and native bytes remain private.

## Native evidence

**VERIFIED (static):** bounded REA Rizin observations were checked instruction by
instruction against independently mapped PE bytes. The base UObject constructor
selects vtable `0x141e0bc40`; slot `+0x68` selects `0x1407a7720`.
The complete 1,690-byte body was decoded, and its range agrees with `.pdata`.
It handles the optional state frame, then calls the NetIndex helper at
`0x1407a7dc0`, followed by property dispatch at `0x1407a7e90`.

Class-name registration literals, constructor callbacks and primary vtable stores
were independently recovered using PE mapping and GNU objdump. REA confirms
that each selected serializer starts by directly calling the shared UObject
serializer before any class-specific archive operation:

| Class | Class factory | Constructor callback → constructor | Primary vtable | Serializer | First call to base |
| --- | --- | --- | --- | --- | --- |
| Level | `0x140bb9190` | `0x140bbb8b0` → `0x140bacc70` | `0x141f90e50` | `0x140bc0690` | `0x140bc06bf` |
| Model | `0x140c027a0` | `0x140c08aa0` → `0x140bed980` | `0x141f9b050` | `0x140c0e010` | `0x140c0e03a` |
| Polys | `0x140b533a0` | `0x140b583d0` → `0x140800e60` | `0x141e25058` | `0x140b61d00` | `0x140b61d0f` |
| World | `0x140e3dcf0` | `0x140e3fcb0` → `0x140e351e0` | `0x14201ad20` | `0x140e474c0` | `0x140e474d0` |
| WorldInfo | `0x1409ee580` | `0x1409f6840` → `0x1409d1640` | `0x141f042b0` | `0x140bc04c0` | `0x140bc04ed` |

All five vtables also resolve the optional pre-NetIndex predicate at `+0x168` to
`0x1405ac2d0`, a verified three-byte false-return leaf. Thus this class-specific
hook does not add another prefix for these five native classes.

**OBSERVED:** the base serializer tests object flag `0x0200000000000000`.
At package version 868, the selected state-frame path accounts for WorldInfo's
prefix as follows. Field offsets into the runtime frame are recorded instead of
assigning unproved gameplay meanings:

| Serialized offset | Width | Native operation |
| ---: | ---: | --- |
| 0 | 4 | Archive object reference into frame `+0x14` |
| 4 | 4 | Archive object reference into frame `+0x44` |
| 8 | 4 | Frame `+0x4c`; selected for archive version ≥691 |
| 12 | 2 | Frame `+0x50`; selected for archive version ≥566 |
| 14 | 4 | Count of frame array at `+0x54`; zero in this export |
| 18 | 4 | Conditional signed offset, present when the first frame reference is nonnull |
| 22 | 4 | NetIndex |

The array helper at `0x140794bd0` has three chained unwind ranges through
`0x140794d6a`. Its zero-count load consumes only the count. The NetIndex helper
has three chained ranges through `0x1407a7e8e`; it serializes four bytes from
object `+0x3c`. The same field is used by the native AddNetObject diagnostics,
which explicitly name NetIndex. These native bodies, rather than an assumption
that all actors have a fixed prefix, explain this particular 26-byte profile.

Property dispatch reaches `0x140755f10` on the ordinary loading path. Its
initial tag read calls `0x14073c980`; the complete 343-byte tag routine reads the
name first and returns immediately on `None`. The existing property grammar is
therefore consistent with the four empty lists. Class-native tails remain
opaque; their internal fields and post-load behavior are not implemented here.

## Evidence and limits

Private evidence root: `evidence/entry-package-objects/`.

- `rea-evidence-bundle.json`: 16 exported REA records. No active binary was
  opened. The completed investigation was reset after export with no retained
  target.
- `native-verification.json`, `verify_native.py`: exact PE-byte checks and
  chained unwind ranges. One exploratory property-body window ends with three
  incomplete decoder rows; those rows support no semantic claim. Selected
  serializer-entry windows do not claim complete class-tail recovery.
- `class-vtable-candidates/all-five-receipt.json`: class-registration,
  constructor and vtable evidence; five selected body spans total 4,508 bytes
  and independently match the owned PE. Its sealed manifest covers 100 files.
- `export-bounds/verification.json`, `independent.json`: original bounds and
  independent grammar comparison; `export-bounds-followup/summary.json`
  independently checks the header version and prefix widths.
- `product-check/verification.json`: focused admission tests and actual
  five-export comparison, exact source/input hashes and command exits.
- `source-ledger.json`: source paths, ownership/licence and hashes.

**VERIFIED:** five focused `upk::objects` tests pass. The actual package smoke
parses all five exports through the appropriate reader, preserves every native
tail, and agrees with the independent grammar on every property/tail boundary
and all three tag/payload spans. The input hash and 81 Rust source/manifest hashes
are unchanged across that check. The product verification receipt SHA-256 is
`3881d5d1e572a846214bc68ab87d4a3e413da3b9e1c4f8061ae8be07b4feaf7e`.

No native client, VM, sequence, renderer, network or gameplay execution was run.
Empty property lists and absence of selected known Kismet classes do not prove
absence of arbitrary startup behavior. The transition from the initial entry
map to the frontend remains a separate unresolved runtime question.

## Source read ledger

All implementation is original. No external implementation was copied.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| The two owner inputs above | Proprietary; private compatibility study only | Actual serialized bytes and native control flow |
| [UEViewer `UnObject.cpp`](https://github.com/gildor2/UEViewer/blob/a0bfb468d42be831b126632fd8a0ae6b3614f981/Unreal/UnObject.cpp), `UnObject.h`, `UnCore.h` at the same pinned revision | MIT; already registered in [PROVENANCE.md](PROVENANCE.md) | Corroborate ordinary NetIndex-before-properties framing; this source does not establish the actor state-frame layout |
| Original project UPK/object/scene/startup-map APIs and their documentation; private PE and bounds helpers | Original project source | Package identity checks, original parser reuse and independent verification |
| REA 6.9.0 tool schemas and `~/.agents/skills/reverse-engineer-anything/` guide | Installed analysis tooling; REA MIT as previously recorded | Bounded target-free Rizin evidence, export and cleanup |
| GNU objdump and Python/Rust standard tooling | Installed inspection/build tools; no linked or copied implementation | Independent disassembly, byte mapping and focused checks |
