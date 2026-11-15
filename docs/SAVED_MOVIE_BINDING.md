# Conditional saved movie-binding validation

`frontend::movie_binding` validates source facts connecting a structural Scene
to its immutable MovieLibrary. It admits a local root, a local sprite, or one
SavedReference import that resolves to one directly exported local sprite.
It preserves the source header version and exact tag identities. It does not
observe native loading, create a display object or produce an AVM binding
witness automatically.

The native basis is the [source/Environment version study](TEXTFIELD_SOURCE_ENV_VERSION.md):
the final selected resource/binding pair determines the target MovieDef. An
ordinary local definition or directly exported imported resource can retain
its source movie under the documented conditions. The initial import copy
alone does not prove the final pair survives later resource substitution.

## API and caller conditions

```rust,ignore
let mut validator = Validator::new(&library, &scene, profile, limits)?;
let facts = validator.validate(node_id)?;
```

`ConditionalProfile` deliberately has no Default. Its four public booleans
must all be explicitly true:

- `final_ordinary_pair`: the final selected pair uses the supported ordinary
  local/direct-export route, without supplied CreateInfo or custom binding.
- `no_resource_substitution`: no relevant replacement changed that pair.
- `loader_compatibility_succeeded`: actual loading/compatibility success is an
  explicit condition; differing header versions are not rejected as inherently
  incompatible, and equal headers do not establish loader success.
- `selected_resource_non_null_and_eligible`: the final selected native resource
  is non-null and eligible for the documented construction route.

These are caller assertions retained in every returned `BindingFacts`. Passing
true for a controlled snapshot records an assumption, not a native observation.
The validator cannot derive these conditions from filenames, FileAttributes,
header equality or source bytecode. It returns `Error::Profile` if any condition
is false. Callers needing an actual runtime witness must establish those
conditions independently and retain that distinction.

`Validator::validate` returns a reference to cached owned facts. The source and
scene stay immutably borrowed for the validator's lifetime. `usage()` exposes
cumulative work counters. Failed requests can consume budget and cache valid
ancestors, but return no facts for the failed node.

## Source facts and exact checks

`BindingFacts` retains node/parent identity, `MovieSource` with MovieId,
AssetLocation, original object path and complete Header, optional DefinitionRef,
optional PlacementSource, the route and the conditional profile. The root has
no dictionary definition or placement; the module does not manufacture a
sprite ID for it. Definition source and selected-movie role coincide only
under the caller's stated profile. Neither value is a native pointer or a
selected calling Environment.

Every registered Scene movie must be the **same Movie object** as the immutable
library entry at that MovieId. A byte-identical clone under the same ID is
rejected. The retained signature/version/length must agree with the movie's
original bytes. The implementation does not infer identity from equal paths.

The selected node's ancestor chain must reach the declared root without cycles.
Arena IDs, parent/child membership, unique sibling depths, path names and the
placement movie are checked. The parent's validated source selects the
placement timeline. Since Scene does not store placement-tag provenance,
the validator replays the saved first frame and requires full `DisplayObject`
equality at the node's depth. It retains the tag that last assigned the raw
character ID and all placement/update tags contributing to that saved
instance. Removals and replacements reset the retained trace appropriately.
This validates immutable saved placement data, not later VM state or current
world geometry.

A local sprite requires the raw character ID, DefinitionRef, dictionary entry,
DefineSprite tag code/ID and timeline identity/frame count to agree. An imported
sprite additionally requires:

1. Exactly one resolved Scene import link and no unresolved/cycle marker.
2. Exact local raw ID, Imported dictionary entry, ImportAssets tag (57 or 71),
   unique imported ID, URL and case-sensitive symbol.
3. The corresponding MovieLibrary binding's source/local IDs, URL and symbol,
   resolved target and `ImportOrigin::SavedReference`.
4. Exactly one target ID in the library's export map and one matching original
   ExportAssets declaration. The result retains that tag and symbol index.
5. A direct local Sprite definition in the target source, checked as above.

ExplicitAlias routes are rejected even if their chosen target happens to match.
Multiple-hop imports, same-movie imports, duplicate export declarations,
unresolved targets, custom class-name placements, unsupported scene state,
empty sprite timelines and non-sprite definitions are outside this profile.
A matching leaf name or guessed filename never supplies a binding. Font aliases
elsewhere in a library do not validate an alias on the selected route.

No source header is replaced with the placement parent's or VM default version.
TextFields are deliberately rejected by this case-3 validator: their own movie
source and ancestor-selected Environment require the separate case-4 contract.

## Bounds and errors

`Limits` bounds scene nodes and registered movies, uncached ancestor traversal,
tag and symbol visits, cached frames, replayed frame-object count, tracked
metadata bytes, requests and child-list visits. Defaults are respectively
100,000 nodes, 256 movies, 256 ancestors per uncached traversal, 2,000,000 tag
visits, 250,000 symbol visits, 25,000 frames, 100,000 frame objects, 32 MiB
metadata, 100,000 requests and 2,000,000 child visits. Counters use checked
addition. Metadata counts retained names, import strings and placement tag
references; the other count limits bound containers. This is not an exact heap
measurement. Package/movie input limits remain the existing loaders' concern.

`Error` distinguishes unestablished profile conditions, budget exhaustion,
node mismatches, source mismatches and unsupported routes. No error path
changes the Scene, MovieLibrary, native state or VM.

## Validation and private evidence

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/saved-movie-binding-check/`.
The independently frozen oracle precedes implementation-output inspection:
`oracle/expected.json` SHA-256
`97c3739e201fa0bcf3be5c3d1b1ce45127b7da6196a068637687269e4d5da6ee`.
It retains exact owned source hashes, package reference identity and raw tag
spans. Asset data and private test executables stay outside git.

**OBSERVED owned input:** login scene node 86 has parent 49, depth 39 and local
character 170 in movie 13. Its saved-reference import selects movie 19 sprite
1039. The placing login movie has header 15 and UISource has header 11. The
imported local ID is declared by tag 970, the placement assigns it at tag 1345,
and the target DefineSprite is tag 10852. The separate manager has header 15;
that does not select an Environment for this validator.

Focused original synthetic tests cover successful local/root/import routes,
all four missing caller assertions, pointer/header mismatches, raw placement
and definition mismatches, parent cycles and duplicate placement depths,
multi-hop imports, aliases, unresolved bindings, wrong tag codes, duplicate
exports and all ten resource limits. No full project suite or native client
execution is part of this change.

**VERIFIED focused checks:** all five module tests pass. The private actual API
harness accepts root node 0, local sprite node 49 and imported sprite node 86,
and rejects TextField node 89 as outside the sprite profile. The successful
route records root placement tag 1507 for local character 219, then placement
tag 1345, ImportAssets tag 970 and target ExportAssets tag 10925 for the button.
The harness exits zero with empty runtime stderr. Its four true profile fields
are explicit controlled assumptions, not measurements of the native loader.
`oracle-comparison/run.json` records commands, executable/output hashes and the
frozen module hash; `build/focused-tests.log` records the selected tests.

## Provenance and limits

The implementation and synthetic fixtures are original. Sources are existing
project-owned Scene/MovieLibrary/GFX readers, the owner's private original movie
and package data, and the repository-authored native studies
[ATTACHED_MOVIECLIP.md](ATTACHED_MOVIECLIP.md) and
[TEXTFIELD_SOURCE_ENV_VERSION.md](TEXTFIELD_SOURCE_ENV_VERSION.md). The latter
records exact REA evidence, original executable hash and native branch scope.
The client artifacts are proprietary, studied only for compatibility; none are
distributed in this commit. No outside code, network source or new dependency
was used. Saved-data agreement does not establish native resource selection,
loader compatibility, script initialization or retail behavior parity.
