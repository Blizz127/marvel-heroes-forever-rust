# Saved frontend startup-map preparation

`frontend::startup_map::prepare(&PackageArchive, Limits)` prepares the saved
frontend sequence records for a future runtime. It returns owned `StartupMap`
data without loading imports, constructing objects or scheduling actions.
The caller supplies the already loaded archive; this module performs no path
search, configuration read or network operation.

**VERIFIED implementation scope:** the owned 37-export map produces one
Sequence, one LevelLoaded event, one OpenMovie action and one PlayerVariable.
The LevelLoaded event retains all six output links in serialized order. Five
referenced action bodies remain explicitly unprepared. This result establishes
saved graph preparation, not completed map initialization or a running login
screen.

The supplied frontend map is not automatically the process's initial map.
**OBSERVED separate entry package:** the pinned `MarvelEntry.upk` is 1,221
stored bytes (SHA-256
`2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33`),
with 20 names, six imports and five exports. The five classes are Engine Level,
Model, Polys, World and WorldInfo. Running this preparation API on that archive
selects zero sequence/event/movie/player objects; it does not reveal a transition
to the frontend map. At that dev13 inspection, the generic object reader rejected
all five property streams because their class preambles were not yet established.
[ENTRY_PACKAGE_OBJECTS.md](ENTRY_PACKAGE_OBJECTS.md) records their subsequent
admission. The World/Level selection below now follows their saved native links;
it still supplies no runtime map transition.

These read-only observations used the immutable dev13 binary, SHA-256
`d963ba5138f2c7310ff4761a20fa8553acdcedbc55b4238d6520d38725f704cf`.
The source identity agrees with the complete selected-distribution inventory.
Private command outputs and exact exit codes are retained in
`evidence/entry-map-study/`; no native client, sequence, VM or network operation
was executed by these inspections.

## API and retained source identity

`StartupMap` contains `sequences`, `level_loaded`, `open_movies`,
`player_variables`, `unprepared_targets`, the total export count and resource
usage. Each prepared object retains its exact signed package reference, full
outer path, class identity, serialized range, net index and complete
`PropertyList`. Import identities retain their declared class package, class
name and outer reference. Export identities retain their class, outer and
archetype references. Retaining an archetype reference does not apply it.

The typed records expose:

| Record | Saved information |
| --- | --- |
| `Sequence` | SequenceObjects members and ParentSequence |
| `LevelLoaded` | Output ports, descriptions, state fields and ordered execution links |
| `OpenMovie` | Movie and MoviePlayerClass references; input/output/variable links; explicitly stored focus, capture, pause and HUD display flags |
| `PlayerVariable` | Explicit PlayerIdx and bAllPlayers properties when present |
| `ExecutionLink` | LinkedOp and nonnegative InputLinkIdx, including both source origins |
| `VariableLink` | Description, ExpectedType, linked references, writable flag and variable-count metadata |

`Saved<T>` pairs a value with `Origin`: export index, export's logical serial
offset, tag span, payload span and encoded-value span. Spans are relative to
the uncompressed export; `logical_value_offset()` performs checked addition.
BoolProperty stores its value in the tag: its encoded-value span is that one
byte, while its payload span remains empty. Array elements additionally retain
their own value spans and the containing property's origin. Unknown properties
remain in the full lists rather than being discarded.

`None` means the property was not serialized. It is distinct from an explicitly
stored false, zero, empty array or null reference. `ObjectReference::Null`
represents only an explicit zero object reference. Float activation delays
retain their exact IEEE bits; preparation does not interpret them as timing.

Movie and player-class fields are `UnresolvedResource` values. Their declared
classes are checked, but this module has no caller-supplied cross-package
namespace map, so it does not resolve them by a filename, similarly named
export or `Default__` object. Likewise, a Player Owner variable link identifies
a saved variable object; it does not select a local player. An empty External
Interface link does not establish the effective receiver after construction.

## Accepted saved layout and rejection boundaries

The selected classes are the imported `Engine.Sequence`,
`Engine.SeqEvent_LevelLoaded`, `GFxUI.GFxAction_OpenMovie` and
`Engine.SeqVar_Player`. Matching uses ASCII case-insensitive comparison while
retaining source spelling. Selection validates the complete declared class
path and Core.Class identity with a declared root Core.Package; a matching
leaf name alone is insufficient. A selected-looking but unproved class is an
error rather than an implicitly supported subclass.

The accepted object body has the established four-byte net-index prefix,
tagged properties and terminating None, with no trailing native bytes. Known
arrays of tagged structs are bounded by their own declared payload, consume
exactly their declared elements, and cannot borrow a terminator from the next
property. Object arrays must contain exactly their declared four-byte
references. Known scalar and object properties must have compatible types;
object fields specifically require ObjectProperty tags.

Preparation rejects duplicate case-insensitive field names, numbered/indexed
control properties, malformed counts, truncated values, invalid references,
cyclic or excessive outer chains and unsupported selected layouts. Sequence
members must be unique local export references whose outer matches the
Sequence. Explicit ParentSequence and available SequenceObjects membership
must agree. Execution targets must be non-null and input indices nonnegative;
an edge to a prepared OpenMovie is checked against its explicit input-port
array when present. Other action classes retain exact target/class identities
without a claim that their bodies or input indices were validated.

`Error` separates budget exhaustion, invalid data, unsupported layouts and
archive read failures. The default budgets are 65,536 exports, 4,096 selected
exports, 8 MiB of selected serialized data, 65,536 properties, 65,536 array
items, nested-list depth 8, outer-chain depth 128 and 4 MiB of aggregate
resolved-name bytes. Identity accounting charges repeated name resolution;
it is a work limit, not an exact retained-heap-size measurement. The existing
package reader supplies its own input/decompression bounds. No arbitrary
native struct decoder or Kismet interpreter is introduced.

## Owned-map observations and focused validation

**OBSERVED saved data:** the selected map's LevelLoaded output
`Loaded and Visible` contains references +2, +31, +27, +32, +30 and +28 in that
order, with input indices 0, 0, 0, 1, 0 and 0. OpenMovie +2 explicitly names the
frontend-manager movie and MarvelGFxFrontEnd class. Its Player Owner link
contains +35, while External Interface and Movie Player links are explicitly
empty. Its Success and Failed outputs are empty. The Sequence retains all ten
saved members. StartPaused and DisplayWithHudOff are absent, as are the
PlayerVariable's PlayerIdx and bAllPlayers values; no defaults are supplied.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/frontend-startup-map-check/`.
The source is the pinned owned `MarvelFrontEndMap.upk`, stored SHA-256
`9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2`.
The original independent raw-byte oracle was frozen before inspecting this
implementation's output; its expected-data hash is
`4976c3173439d6c1c3b4b47bf53965464e3273566e43b5eb487fe50766aa9e22`.
All 147 emitted property records match that oracle in names, types, exact
values and tag/payload/encoded-value spans. The comparison also checks each
encoded value against the owned raw bytes; there are no missing, extra or
duplicate property records. The detailed receipt is
`oracle-comparison/properties-result.json`.

The focused standalone compile uses this original module and the project's
existing UPK reader. All four module tests pass, covering absent/false/null
distinctions and BoolProperty origins, nested array boundaries, malformed
counts, duplicate properties, invalid resource classes, outer cycles and
budgets. The actual-map preparation consumes four selected exports, 4,288
serialized bytes, 147 properties, 26 array items and 3,841 resolved-name bytes.
Eight separate limits set below the actual input/work requirement each return
the corresponding typed budget error. Compiler logs are empty; actual
preparation and focused tests exit successfully. Other package tests and the
full project suite were not run for this change.

No native game, script VM, sequence scheduler, player creation, import loader,
default-object finalization, receiver assignment or live login was run by
these checks. Raw input, oracle data and compiled private harnesses remain
outside git. The library is exposed as `frontend::startup_map`.

**VERIFIED integrated CLI:** `mhf frontend startup-map <package.upk>` reports
the prepared counts, LevelLoaded ports and ordered links, OpenMovie resource
references, explicit Boolean origins, and variable links. Missing values and
saved empty arrays remain distinct. A focused smoke on the pinned map passed
nine checks, including the six-link order, absent start-paused flag, unchanged
input hash and rejection of extra arguments. Private evidence is
`evidence/startup-map-cli/receipt.json`; the four module tests also passed after
root integration. This command reads saved data only and makes no connection.

## Evidence and provenance

This is original code. The supported layout comes from the owner's pinned
retail map and the existing original reader in `src/upk.rs` and
`src/upk/objects.rs`. Those client data remain proprietary and private; no
extracted tables or asset bytes are part of this module or its tests.
The prior [initial-instance study](FRONTEND_INITIAL_INSTANCE.md) supplies
package and REA evidence for the selected action and its conditional native
creation/Start path. The [receiver study](FRONTEND_EXTERNAL_RECEIVER.md)
documents why a saved empty binding cannot establish the eventual receiver.
These are repository-authored studies, not newly consulted external code.
No additional outside implementation, network source or license material was
used for this change. Native default/copy investigations remain separate and
do not authorize promoting absent properties into effective values here.

## Saved World startup selection

`frontend::startup_map::prepare_world` follows one explicitly selected saved
`Engine.World` through its `PersistentLevel` link and the first native Level
sequence-array element. It returns that route alongside the existing all-export
startup inventory. A package's inventory alone does not select a runtime entry.

**VERIFIED (static):** the owned native `GetGameSequence` body at
`0x140bb6cd0..0x140bb6d23` returns Level field `+0xe8` element zero when its
count is nonzero; otherwise it returns null. It does not search for a later
nonnull element. [UPK_WORLD.md](UPK_WORLD.md) establishes World `+0x80` as the
PersistentLevel link; [UPK_LEVEL.md](UPK_LEVEL.md) establishes its serialized
Level sequence array. This selector uses those established readers.

The native getter's implicit Level argument comes from runtime current-Level
state. Following a saved World→PersistentLevel link does **not** prove that
runtime initialization has selected that Level. No Entry→FrontEnd transition,
event delivery, sequence execution, class default, player or GFx receiver is
created or inferred by this API.

### API and supported identities

```rust
prepare_world(
    archive: &PackageArchive,
    world_export_index: usize, // zero-based, selected explicitly by the caller
    limits: Limits,
) -> Result<WorldStartupMap>
```

The result retains the complete parsed `WorldObject` and `LevelObject`, their
exact local object/class identities, the `persistent_level` reference, and
`main_sequence`:

| Variant | Saved meaning |
| --- | --- |
| `EmptyArray` | The native sequence array contains zero elements |
| `FirstNull { link }` | Element zero exists and encodes null |
| `Local { link, export_index, inventory_index }` | Element zero identifies the checked local Sequence export |

`selected_sequence()` returns only the selected inventory Sequence, or `None`
for either absent result. The enum retains the distinction between those absent
results. The full sequence array, including later elements, remains available
on `level`. Every `NativeReference` retains its owning zero-based export index,
package-logical serial offset, four-byte export-relative value span and resolved
identity or explicit null. Encoded object references retain the package rule:
positive one-based exports, negative one-based imports, zero null.

The supported saved route requires the exact imported classes `Engine.World`,
`Engine.Level` and, when selected, `Engine.Sequence`, with the established
`Core.Class`/root `Core.Package` import identities. Names follow the existing
case-insensitive FName identity checks. World must have outer zero, Level must
have that World as outer, and Sequence must have that Level as outer. These
are admission rules for the observed saved-map profile, not a claim that every
possible native package has these relationships. Unsupported state-frame
preambles are rejected; World and Level readers enforce UE3 profile 868/3.

A null PersistentLevel, invalid reference, wrong outer, cyclic outer chain or
inconsistent inventory identity is an error. Imported Level or main-Sequence
links remain unsupported; the selector does not load them or search filenames,
names, another World or a later array element as a replacement. The selected
Sequence must occur exactly once in the prepared inventory with the same full
local identity.

`prepare()` and the `StartupMap` inventory API are unchanged. The new result's
`inventory` contains the same all-export preparation, including unrelated
Sequences. Its own `inventory.usage` accounts for inventory work;
`WorldStartupMap::usage` additionally includes the two native exports and their
references/properties/arrays. Existing budgets bound export scans, selected
export reads, bytes, identity walks, properties and arrays. Native serialized
byte/selected-export budgets are charged before export reads; the native
readers also apply their own bounded parsing limits. Inventory preparation uses
the remaining aggregate budget. Unknown native data remains retained by the
underlying readers rather than treated as understood or executable.

### Paired original-package observations

**OBSERVED:** actual package reads produce these saved routes. All object
references in this table are one-based; API input indices are zero-based.

| Package | World → Level → first Sequence | PersistentLevel logical value offset | First Sequence logical value offset | Retained Level suffix |
| --- | --- | ---: | ---: | ---: |
| `MarvelEntry.upk` | `4 → 1 → null` | 2,049 | 1,166 | 615 bytes |
| `MarvelFrontEndMap.upk` | `36 → 11 → 34` | 50,970 | 26,512 | 1,109 bytes |

Both arrays contain exactly one element: Entry is `FirstNull`, not
`EmptyArray`. FrontEnd's selected Sequence is export 34, with ten saved members.
Its saved LevelLoaded event has six links on the “Loaded and Visible” output;
all ten members and six targets retain the expected local ownership. This
describes stored references, not event activation or execution order. Both
returned inventories compare equal to separate calls to the unchanged
`prepare()` API.

Private pins under
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/`:

| Package | Bytes | SHA-256 |
| --- | ---: | --- |
| `MarvelEntry.upk` | 1,221 | `2b11493012b90c8ef95ea775f2b57e8ece60a9a9ab6d09dcca55703a3de36f33` |
| `MarvelFrontEndMap.upk` | 12,038 | `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` |

### Focused validation and evidence

**VERIFIED:** seven focused `startup_map::tests` pass, including three new tests
covering class/namespace/outer/state-frame admission, invalid references and
origins, duplicate/mismatched inventory identities, zero array versus first null,
later nonnull entries, ordinal zero and work limits. Synthetic fixtures exercise
invalid and otherwise unobserved shapes; they establish no retail behavior.

The standalone original-package smoke passes against the actual new module and
Level helper commit `63fd7e332cdfb15ca502832f2b565fe4e6f32bab`. It confirms exact
aggregate-budget success, twelve budget rejections and four identity rejections
across the two packages. An independent Python byte walk agrees with 43 grouped
comparisons of route identities, source spans, sequence members, event links,
opaque-tail hashes and unchanged inputs. All 1,724 opaque Level bytes agree.
The original package and frozen source hashes are unchanged. No full suite,
native client, script, server or network execution was run for this increment.

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/startup-map-selection/`.

- `check1/verification.json`: command/check receipt, frozen source and input
  hashes; SHA-256
  `2afc4fbb5b23d13b90cbcdf7a888e5e296962424297a3a1f4a2c167dcdc28e73`.
- `check1/sources.json`, `focused-tests.log`, `retail-smoke.tsv`, `verify.py`:
  exact original source composite, focused commands and comparison details.
  The private composite supplies the Level module hook without editing the
  integrator-owned `upk.rs`.
- `oracle/expected.json`: independent expected raw references/spans and native
  evidence links; SHA-256
  `34b77c79e74ca3642f0c6ad6f392579d19d9ee38322e0f4bd6d182d4ca13e001`.
- `oracle/receipt.json`: byte-walk provenance and unchanged source/package pins;
  SHA-256
  `6b04a23356da7fb095fd00b87b967fcb93e6408a1344bf7a65b723fbded91094`.

The independent comparison shares original Rust decompression/export extraction
and captured name metadata; it is not an independent package decompressor.
Native grammar comes from retained REA evidence. Sequence tag boundaries reuse
the prior independent Python oracle with scalar byte readback. Those limits
prevent treating agreement as runtime or wire parity.

### Provenance for World startup selection

All new implementation and validation code is original. No third-party
implementation source or new web source was read for this increment.

| Source | Licence/ownership | Use |
| --- | --- | --- |
| Owner's two pinned packages above | Proprietary; private interoperability study | Actual saved route, identities, member/link spans and retained suffixes |
| Owned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`, through retained REA receipts | Proprietary; private interoperability study | World serializer `0x140e474c0`, Level serializer `0x140bc0690`, GetGameSequence `0x140bb6cd0`; no new native execution |
| Original project `src/upk.rs`, `src/upk/{objects,lzo,world}.rs`, existing startup preparation, and `docs/UPK_WORLD.md`, base `9abb8668d83158703c50fa3a95caad362f7ce536` | Original project code/evidence | Reuse existing bounded identities, archive/object APIs and native source provenance |
| Original `src/upk/level.rs`, `docs/UPK_LEVEL.md`, commit `63fd7e332cdfb15ca502832f2b565fe4e6f32bab` | Original project code/evidence | Established Level prefix and first-element getter contract |
| Private `evidence/entry-world-native/`, `entry-level-native/` and the independent oracle above | Original project investigation; underlying owner bytes remain private | Exact static evidence, raw-span comparison and provenance limits |

The three reused REA records are
`ev_8c1890ff39c091a4e15b5ba766c07cb7265a2e3032bfe0ed4ce9923ceb640405`
(World serializer),
`ev_d066e0f5437df38bcdc102315eb0950dc51877dde470eac57d125f1eb9b6a421`
(Level serializer), and
`ev_abceb68da7908d6ea67c8e1cda235e412248e498bb6f0bd52bed8d6f958bfbb4`
(83-byte getter). Each retained instruction range was matched to the pinned PE;
the Level field-identity chain is recorded in
`entry-level-native/field-identities/verification.json`.
