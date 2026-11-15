# Saved login instance inputs

**VERIFIED structural inventory:** the actual saved login first frame has 147
nodes: one root, 74 Sprites, 33 EditText instances and 39 Shape leaves outside
the attached receiver API. Every node retains its parent chain, complete child
list, definition/import source, header, raw placement and original action spans.
This prepares adoption inputs; it creates no live target, VM owner, Environment,
prototype, current format, event installation or completed constructor.

The private original probe depends on the complete frozen project at
`b28eddbe27338ebeedf254e72dc02acb58fd6cc0`. It opens only the three existing owner
pins, loads `MovieLibrary` with package order `[0,1,2]`, and constructs the login
`Scene::first_frame`. `DisplayState::from_scene` accepts and preserves all 147
nodes with zero object/source bindings and no property calls. No renderer, UI,
VM script, native client, server or network operation runs.

## Exact source selection

The inputs are `MarvelFrontEnd_SF.upk` from the private alienware-candidate pin,
`MarvelGame.upk` from frontend-startup, and `MarvelFonts_SF.upk` from
frontend-fonts. All full paths, sizes and hashes are retained in the execution
receipt. The only supplied alias is
`MarvelFonts.gfxfontlib → MarvelFonts.fonts_en`. It is an explicit English
inspection route, not evidence of the native locale-selection state.

There are 26 loaded library movies. The Scene registers only these three:

| Movie ID | Saved object path | Header version | Package / export |
| --- | --- | --- | --- |
| 5 | `marvelfrontend.marvel_assets` | 10 | 0 / 9 |
| 13 | `marvelfrontend.marvel_login_dialog` | 15 | 0 / 17 |
| 19 | `marvelhud.!uisource` | 11 | 0 / 23 |

IDs are identities within this exact library/load order. They are not native
pointers or `LoadedMovieSourceId` values. Each movie record retains the complete
parsed header, including signature, declared length, stage, frame rate/count
and tag offset. The explicit English font movie is ID 22, header 8, package 2 /
export 18. Its presence in the library does not make it an attached Scene owner.
The manager is movie 20, header 15, package 1 / export 17004; it is outside this
login Scene and does not become its Environment merely by being loaded.

`movie_binding::Validator` is invoked for every node with all four
`ConditionalProfile` fields explicitly true **as probe assumptions**. It accepts
all 75 root/Sprite candidates: one LocalRoot, 59 LocalSprite and 15 exact
SavedReferenceImport routes. Each import preserves original URL/symbol, unique
export, source/target tags and contributing placement tags. No selected route
uses the English font alias. Source/header versions remain distinct across
imports; the placing parent's version is never substituted.

All 33 EditText and 39 Shape nodes receive the validator's explicit non-Sprite
rejection. This is its documented case-3 profile, not a corrupt-source result.
Every EditText has a tag-37 definition local to the source selected for its
parent under the conditional profile. That is useful input to the separate
case-4 adoption route; it does not establish the field's live Environment.

## Complete tree and field inventory

| Saved category | Instances | Named | Unnamed |
| --- | --- | --- | --- |
| Root | 1 | Not a child | Not a child |
| Sprite | 74 | 62 | 12 |
| EditText | 33 | 33 | 0 |
| Shape / unsupported attached category | 39 | 0 | 39 |

The 146 parent-child edges contain 95 names and 51 unnamed instances. No saved
name is empty, NUL-bearing or duplicated among siblings. The private inventory
still includes every unsupported and unnamed child. There are **no named
unsupported-category leaves in this full snapshot**. A fixture that adopts only
a subset must nevertheless retain other known named receivers as
`AttachedChild::Unsupported`; silently dropping them changes lookup behavior.
Names and ascending source depths remain exact; no synthetic instance names are
assigned to unnamed children.

The 33 fields reference 21 distinct EditText definitions. Ten instances belong
to source movie 19 and 23 to movie 13. Their original flags, bounds through the
definition/Scene record, FontID/class, height, color, layout, variable name,
initial text and byte spans remain private source data. Font dependencies are:

| Exact saved route | Field instances |
| --- | --- |
| Movie 19 font 270 → explicit English alias → movie 22 font 5 | 10 |
| Movie 13 font 74 → explicit English alias → movie 22 font 4 | 3 |
| Movie 13 font 77 → explicit English alias → movie 22 font 5 | 20 |

Six placed fields carry saved HTML. Their markup must not be installed as a
plain current document or stripped into a substitute string. Even a plain saved
initial string does not establish current native text/default-format state.
The inventory preserves declared font routes without manufacturing effective
format handles, loader selection or constructor effects.

## Actions and source spans

The Scene retains 123 unexecuted action bindings: three frame actions, 50 init
declarations, 36 per-instance Sprite initializers and 34 placement ClipActions.
All 34 placed handlers carry saved Construct mask `0x40000` and source bytes in
the **placing login movie 13**, including handlers on imported Sprites. Their
code source is not replaced with the imported definition movie. Each raw handler
maps to exactly one enclosing original PlaceObject tag. The private records
retain tag header/body offsets, handler-record offsets, declared sizes, action
spans, hashes, masks and optional key codes.

Across all 26 loaded movie sources there are 332 serialized handlers: 331
Construct and one MouseMove (`0x08`). They occupy 331 ClipActions aggregates;
raw-byte checks preserve reserved/header fields, each per-record mask/size and
the terminating zero. In these originals each saved AllEventFlags equals the
union of its record masks. That equality is a source observation, not an event
installation rule. No KeyPress record occurs in this corpus.

The MouseMove belongs to manager movie 20's cursor placement, outside the login
Scene. There are 889 unbound action tags in the Scene's registered movies; those
remain source declarations, not scheduled executions. Counts across the full
library, the registered movies and placed action bindings are deliberately kept
separate. Native event loading and execution require their own established
contract; serialized masks do not supply current target/load/source flags.

## Required supplied-state schema

Private `required-supplied-state.json` has one record per saved node, including
its exact sources, parent/children and action indices, plus explicit unsupplied
runtime fields. It is an original requirements representation, not an executable
adoption plan or JSON Schema. `value: null` marks **missing supplied input**;
it never asserts a native null value. No runtime IDs or flags are assigned.

| Area | Required supplied state and relation |
| --- | --- |
| Binding conditions | Independently established final ordinary pair, no substitution, compatibility success and eligible non-null resource. The inventory's true probe assumptions are insufficient. |
| Source domain | A coherent `LoadedMovieSourceId → MovieId` map, separate from source paths, native pointers and Scene IDs. |
| Root/Sprites | Unique live `DisplayTargetId`, complete already-adopted parent chain, `AttachedClipSource`, final binding witness, actual `AttachedEventState { target_flags, load_flags, source_flags }`. Root's source sprite ID 0 is the API root convention, not an invented dictionary definition. |
| VM prerequisites | Actual installed Object/MovieClip constructor/prototype identities; root adoption and any native-name profile installation before original script registration. Constructor availability does not mean constructor execution completed. |
| Children | One complete named-child snapshot after all child creation/adoption; exact parents, unique names/targets, and Unsupported entries for named unimplemented receivers. A saved name alone does not establish lookup visibility. |
| EditText owner | Unique live target, exact local parent MovieDef/header association, parent clip, actual +6a flags, current nonzero selector-10 `TextFieldPrototypeId` and watch state. |
| EditText Environment | The supplied owner must be the first parent ancestor whose actual +6a flags contain `0x400`. This cannot be chosen from the source header, script origin, manager path or root position. |
| Current text | Domain/owner/node identity, actual +fc and view +149 flags, separate last-input/document state, current format/default handles, definition-HTML and post-nonempty defaults, pending invalidation. These flags are distinct from +6a and serialized EditFlags. |
| Text side paths | Current profile explicitly excludes watches, translation, stylesheets, editor, variable binding and image callbacks. Unknown is not Absent. Runtime format bindings require explicit handle/style provenance. |
| Events/Environment | Original registered code/span identities, actual installation state and scheduling, supplied current target, operand stack and all four global registers. Empty stacks/Undefined registers are not inferred from saved files. |
| Optional native paths | Actual focus-rectangle state if accessed; explicit root/player/receiver/context/backend association before ExternalInterface calls. |

At the frozen source, case-3 adoption's owner profile requires target flags'
high byte `0x52`; named Clip/TextField lookup also requires `flags & 0x180 ==
0x180`. These acceptance conditions do not justify assigning those bits to every
saved instance. The separate field host accepts watch state Absent and a
restricted text snapshot; it does not discover missing native state.

## Structural blockers and readiness

No missing parent, duplicate depth/name, unresolved selected Sprite import or
malformed source header blocks this saved-tree inventory. The following remain
required before a complete login startup execution can be claimed:

- The saved binding profile must become an explicit supported runtime contract;
  conditional source facts alone are not a native adoption witness.
- All live target/source identities, raw flags, Environment owners and current
  field prototypes/text formats must be supplied or produced by established
  construction code. The prior selected-node fixture's constants are not defaults.
- The 39 Shape leaves remain unsupported attached receivers. Their source
  structure is retained, with no fabricated MovieClip replacement.
- The generic movie-binding validator excludes EditText. Its separate local
  parent/source candidate check succeeds for all 33 fields, but live case-4
  admission and effective text state remain explicit work.
- The six authored HTML fields exceed the current plain snapshot profile if
  their raw markup is passed unchanged. Established initialization and formatting
  must produce an admissible document; stripping markup is not a substitute.
- Saved action spans and init declarations establish availability, not event
  installation, scheduler order, constructor completion or native startup.
  At this frozen revision the saved-action/Environment containers are not wired
  into the VM. Later VM work must be assessed at its own revision.

## Verification and evidence

The focused private build/run passed with all 204 tracked project files,
harness inputs and three original package hashes unchanged. No product source
was edited. All 147 complete SceneNode records match the earlier renderer
inventory exactly. That renderer's action file deliberately selected owners
`[0,30,103,105,111,129]`; all its 54 records match, without pretending it covers
all 123 bindings. Separate checks validate every current action span, 1,422
unique raw tag headers, 61 original text-string spans, EditText IDs/flags and
all 332 clip-record byte layouts. These are structural/source checks, not
retail runtime parity. The private preflight notes retain a corrected
harness-only import error and the narrowed prior-action comparison scope.

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/`.

| Artifact | SHA-256 |
| --- | --- |
| `login-instance-inputs/probe/MANIFEST.json` | `ac3d938011abaee74e3a4f5b3088a85f4ecb0940d374401a25e75e5118cd6755` |
| `login-instance-inputs/probe/execution.json` | `4878cfa722f9f8b41f236924ed87f47c5f6635bf131a123d2488e8fb16d091ae` |
| `login-instance-inputs/probe/inventory.json` | `be8b5bdf79639e69b1a6d3ef3be9ea805122350179bcee8b5e85c1a348c48258` |
| `login-instance-inputs/probe/summary.json` | `570fbe3a10a007b168c9ae2a536630ac63853f586872d30f5da92abc9a513fc1` |
| `login-instance-inputs/probe/clip-wire-records.json` | `b5cff15dffd56d311111f8383ee9881d21d634892f1cc52f037ac27030640cf0` |
| `login-instance-inputs/probe/required-supplied-state.json` | `549d2b9db1282f654da640213aace5e74814c4e8bbcc6172c58f8de91ff88627` |
| `login-instance-inputs/contract-audit/receipt.json` | `a0269b80400857ab0d5b72c7ade0a2cb72e641b324d7dcdf1940c07ae5d2e27b` |

## Provenance

The probe and requirements representation are original. Outside primary inputs
are the owner's private proprietary packages, used solely for compatibility:

| Pin (relative to private root) | SHA-256 |
| --- | --- |
| `pins/alienware-candidate/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFrontEnd_SF.upk` | `af8c2016b4606c26076c34b3e70f4c7bf78c4af4e583cfec5585aedd35c8f190` |
| `pins/frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelGame.upk` | `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6` |
| `pins/frontend-fonts/UnrealEngine3/MarvelGame/CookedPCConsole/MarvelFonts_SF.upk` | `43b7f8576a487601e812607e1c18f3ced659629de22b0cf7c7ae97b601adf76e` |

Project sources read were the original GFX/Scene/text, frontend AssetSet/library,
movie-binding/display, attached clip/field and text-state APIs and their existing
docs. `contract-audit/sources.json` records 18 exact Git-source hashes/read scopes.
The probe source manifest authenticates the entire compiled revision. Reused
original project receipts/scripts were `saved-movie-binding-check/oracle-comparison`,
`runtime-text-preparation/actual/src/{scene_recipe,scene_binding}.rs`, and
`runtime-text-visibility/scene/geometry/{nodes-saved.jsonl,actions.jsonl,receipt.json,src/main.rs}`.
They retain their owner-data provenance and are not treated as native captures.
No third-party implementation, new dependency, website or new REA/native session
was used. Existing native contract provenance remains in the referenced project
studies; no source bytes, extracted scripts, names inventory or private build
artifact enters git with this documentation.

## Complete adoption from explicitly supplied state

The original `frontend::scene_adoption::adopt_saved_scene` orchestration adds a
bounded product path over the existing binding, attached-object, display and
TextField APIs. Its implementation is frozen at `daa8e34`; this section describes
a later capability than the immutable inventory above. It neither discovers
native startup state nor runs initializers, constructors, events, ExternalInterface,
saved actions or a scheduler.

`SuppliedSceneState` has no `Default`. The caller provides the conditional
movie-binding profile, an explicit assertion that all admissible Clip owners
share this VM's builtin context, a nonzero association domain, distinct loaded
source identities for exactly the Scene movies, and a unique target identity
for every adopted Clip, field and named unsupported leaf. Root/Sprite inputs
carry the complete event-state words and separate focus-rectangle state. Field
inputs carry their raw `+6a` flags, prototype handle, watch state, first marked
Environment ancestor, restricted text `Profile` and complete current `Snapshot`.
No current field document is manufactured from saved initial text or HTML.

The result owns the adopted root, node-to-Clip and node-to-field maps, complete
`DisplayState` and bound `TextFieldHost`. The source Scene remains borrowed and
unchanged. Clip owners receive the existing builtin-context binding only under
the explicit one-context assertion; the existing VM does not admit TextField
owners as direct builtin targets. Fields retain their separately checked
ancestor context route.

Before any VM mutation the helper:

- Checks the complete preorder/tree and exact registered source identities,
  then validates every Root/Sprite through `movie_binding::Validator`.
- Requires each field to be an exact local tag-37 definition in its admitted
  parent's selected movie, with matching dictionary, raw placement, ID, bounds
  and original parsed EditText body. Other admitted unsupported categories must
  remain local definition leaves. Imported or class-only non-Clip variants
  reject explicitly in this initial profile.
- Requires exact category-map coverage, globally distinct supplied targets and
  complete named unsupported-target coverage. Unnamed Shape leaves stay in the
  Scene without acquiring fabricated MovieClip owners.
- Checks each Clip's supplied `+6a` bit 14 against saved **local** visibility.
  It never compares this flag with effective inherited visibility or writes a
  synthetic `_visible` property to make the states agree.
- Checks field owner/prototype/absent-watch conditions and walks the complete
  supplied parent chain to the first `+6a & 0x400` ancestor. A later marked
  ancestor cannot substitute for the first one.
- Replays every admitted parent's complete saved first frame and compares all
  child depths, order, raw placements, names, source movies and parent links.
  Sibling names must be nonempty, NUL-free and unique. Named unsupported leaves
  remain explicit `AttachedChild::Unsupported` entries.

Source replay has independent tag-visit and output-object budgets, in addition
to existing Scene/binding budgets; parsed field-definition bytes and retained
text are bounded separately. Caller-owned input allocations are not counted as
module-created memory. Existing VM and host budgets remain authoritative.

After preflight, Clips are created in preorder; the root goes through the
unchanged authoritative `Vm::adopt_attached_root` fresh-phase guard. Display,
focus and builtin contexts bind to those real owners. Fields then pass through
existing parent-movie adoption, current-snapshot admission and TextFieldHost
binding. Every Clip receives its complete frozen named-child list only after
all owners exist. This replaces fixed selected-node loops without introducing a
second metadata-only receiver representation.

`AdoptionFailure` reports the failing node, stage and `vm_may_be_partial`.
Preflight failures report false and leave the VM untouched. **Every failure once
mutation begins reports true and requires discarding the partial VM/session.**
There is no rollback or safe retry claim. In particular, a non-fresh root, VM
capacity failure, or rejected current field `Profile`/`Snapshot` can occur after
Clip allocation. Snapshot validation remains in `TextFieldState::from_snapshot`
with the actual allocated owner identity; the helper does not duplicate that
contract using a fabricated owner.

Four focused synthetic test groups pass: complete adoption with an explicit
named unsupported Shape; hidden-parent versus locally visible nested Clip;
source/category/identity/context/first-ancestor/budget/name rejection before VM
mutation; and fresh-root/current-field rejection after mutation with mandatory
discard. Their fabricated package, names, geometry and text are original test
inputs, contain no game data, and create no temporary files. These checks prove
Rust orchestration behavior only.

The implementation reads only the original project APIs and existing
`ATTACHED_VISIBILITY`, movie-binding and TextField contract documentation;
no new third-party source, dependency, REA session or native execution is used.
The previously recorded owner-package and native-name-profile provenance applies
to the separate private actual-scene check below.

**VERIFIED controlled actual-scene adoption:** a new immutable private harness
loads the same three packages and explicit English alias, then supplies complete
controlled state to the frozen product helper. It uses a clean detached copy of
`daa8e34de05d997842bc50c4e79511df3f4cf365`; all 210 tracked source files, harness
inputs and owner-file hashes remain unchanged. A private module wrapper adds the
unhooked helper without editing any product source. The old inventory remains
immutable and supplies the independent source oracle.

The focused build and run both exit zero. A separate comparator passes 4,255
scalar/collection checks covering all 75 Clip source/header/parent/target/event/
focus/display bindings, all 33 field source/Environment/prototype/watch/profile/
current-snapshot bindings, 108 distinct targets/owners and all 95 typed named-child
lookups. Twenty supplied field Environment routes deliberately climb above the
immediate parent. All 75 child snapshots are frozen. The 39 unnamed unsupported
leaves remain structural Scene nodes without fabricated AS2 owners.

Each child lookup uses the existing native-name inventory authenticated against
its SHA-256 and the owned executable's hash; it does not inject plain object
members as lookup substitutes. The original 147 Scene nodes, 123 action bindings
and all 26 decoded movie byte buffers are unchanged. Instruction count, text-host
calls and display-property calls remain zero. No scripts, constructors, events,
ExternalInterface, renderer, native process or server operation runs.

The controlled target/event/focus words, Environment marks, prototypes, watch
conditions, one-context assertion, current text and format handles are **fixture
inputs**, not native observations or defaults. Actual saved visibility has no
local/effective disagreement in this corpus; that distinction is exercised only
by the fabricated hidden-parent test. The six saved HTML definitions remain
unchanged, while supplied current field documents are explicitly controlled
ASCII strings. This does not establish their native construction or formatting.

The private first build attempt exposed two harness-only mistakes (a Result
alias and output placeholder). Its source/log and correction notes are retained;
the successful run uses the subsequent frozen harness and unchanged product code.

| New private artifact under `evidence/scene-adoption/` | SHA-256 |
| --- | --- |
| `module-check/focused-result.json` | `a71be81af03a30938cc0760178702d9164ab6a6380224b018b1ebbd6d0a8dbba` |
| `actual/receipt.json` | `533e88e3618ea05d0ae7ff0083eac3b51a43ef97f445bd8658b3b49839b90aad` |
| `actual/execution.json` | `95e1fd6aace619802ab7cf4b8eb509284a6fdb2e4bbf4f18406fbd6720f197bb` |
| `actual/MANIFEST.json` | `2e7eb1b3ffe12e267f6dd09bacf222d6c46ea79a588b513526ef4690494cf554` |
| `actual/expected-structure.json` | `ee9dd833d202c2aff1bdfd2c526ed33fce734ea47d40b13ab463dc7734e00265` |
| `actual/fixture-inputs.json` | `c06d1fee8e947454e0229b20b388c8760af3bb9efec4259a83614826d6c091e1` |

The only additional outside input is the existing private extracted name
inventory `evidence/rea-attached-name-classification/native-name-profile.v1.tsv`
(SHA-256 `902b4f15629d3ae32aa6867165d275f2ef3ea289ced3696b9bf132155fd00b91`),
used solely for authenticated name membership. Its source is the owner's
proprietary `MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Neither file enters git. The execution receipt records their exact private
paths and unchanged hashes alongside the three previously documented packages.
