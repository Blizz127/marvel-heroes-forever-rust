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
outside git. The crate-level module hook and runtime integration belong to the
integrator.

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
