# Explicit frontend display state

`frontend::display_state::DisplayState` holds mutable geometry for an existing
`gfx::scene::Scene`. It associates already-created VM objects with particular
scene node IDs only when the caller explicitly binds them. It neither constructs
MovieClip objects/classes nor performs startup, path lookup, event selection,
class registration, timeline advancement, hit testing or rendering.

## Geometry and host API

```rust
let mut state = DisplayState::from_scene(&scene, Limits::default())?;
state.bind_object(existing_vm_object, actual_scene_node)?;
state.set_pointer_stage_twips(Some([stage_x_twips, stage_y_twips]))?;
let execution = vm.execute(explicit_code_id, existing_vm_object, &mut state)?;
let current_nodes = state.snapshot_nodes();
```

The caller supplies the code and selects its execution point; this example is
not an event scheduler. The code can be an exact saved handler span, but such a
probe does not establish initialized UI state. `ObjectId`, code IDs and node IDs
belong to their respective VM/scene instances. Reusing this state with another
VM or rebuilding the scene without rebuilding associations is unsupported.
`bind_object` does not validate another subsystem's heap or create an object.

`FrontendHost::attach_display` combines this property host with its existing
Selection listeners and class registry. It requires an explicit binding between
the registry's owning VM root and the scene root, and rejects a second attachment.
Indexed property requests then use the same DisplayState checks and budgets;
without an attached display they fail explicitly. The caller retains ownership
of scene construction, object identity, input sampling and dispatch order.

**VERIFIED (dev.9 original-script integration):** A private harness runs all 452
manager initializers, consumes the checked VM handoff, explicitly associates the
saved structural scene and executes the unchanged MouseMove handler at source7096
through that same FrontendHost. The handler runs 14 instructions and six property
calls, places the cursor at the supplied `(100,200)` pixels, and preserves the
one Selection listener and five class registrations. Missing-root and replacement
attachments reject. Receipt: `evidence/dev9-bootstrap-handoff/display-host-receipt.json`.
The cursor association is still an explicit Plain-object diagnostic; this does
not establish native MovieClip allocation or complete event ordering.

The separate attached-clip route uses `bind_movie_source` to associate an explicit
loaded-source ID with one of the Scene's movies. `bind_attached` then checks an
existing VM clip's source, sprite ID and header version against the resolved
Scene definition. It accepts only the Scene root or a Sprite, and requires the
attached parent to be bound to the exact Scene parent first. Source and node
identities cannot be rebound. A successful binding associates both the VM owner
and its distinct `DisplayClip` reference with that node; generic Object bindings
do not implicitly authorize display references.

This route allocates no VM clip, installs no child snapshot and chooses no raw
event-state defaults. It consumes the explicit typed state described in
[the attached MovieClip contract](ATTACHED_MOVIECLIP.md). A property request still
requires a bound execution context, including requests with an explicit display
reference. Invalid IDs, unresolved/non-Sprite definitions, missing source maps
and mismatched parents fail before changing bindings.

The state validates the complete preorder tree, parent/child agreement, source
identity indices, depth, mask-reference range, finite matrices, and agreement
between raw placements and their saved derived transform/visibility. It rejects
stale or hand-adjusted derived values that disagree with the raw source. Every
`DisplayNode` retains its immutable original `SceneNode` as `source`, plus current
`local_matrix`, `world_matrix`, `local_visible` and `effective_visible`. Accessors
return shared references; callers cannot mutate geometry around the checks.

Bindings are bijective and bounded. Repeating the same association is harmless;
rebinding either side to a different object/node rejects. A property request must
have a bound execution context. An empty String target means that context; an
Object target must also have an explicit binding. All nonempty paths, including
apparently familiar names, reject. Names in source placements do not create VM
objects or establish path syntax.

`read_property` and `write_property` expose typed `DisplayError` results for
composite hosts. The `Host` implementation delegates the indexed AVM1
GetProperty/SetProperty requests and maps those errors into the existing host
error channel. Native calls, timeline/URL/other commands and class registration
remain rejecting. Generic AVM1 member lookup such as reading a heap member named
`_x` is not automatically intercepted by this module.

## Property boundary

Adobe's published MovieClip API places X/Y in the parent's coordinate system,
provides read-only mouse coordinates relative to the queried clip, and defines
Visible as Boolean. The separate SWF reader retains twenty twips per pixel.
These are documentation contracts, not newly observed native behavior.
[Adobe MovieClip reference](https://open-flash.github.io/mirrors/as2-language-reference/MovieClip.html)
and the project's [scene coordinate ledger](GFX_SCENE.md) record the sources.

| Property | Read | Write |
| --- | --- | --- |
| `X`, `Y` | Current local translation / 20, as Number | Finite Number × 20 replaces only that translation component |
| `MouseX`, `MouseY` | Explicit stage pointer through the full inverse current world affine, then / 20 | Unsupported/read-only |
| `Visible` | Local visibility as Bool, independently of hidden ancestors | Bool updates local visibility and descendant effective visibility |
| Other indexed properties | Explicit unsupported error | Explicit unsupported error |

The host admits only the typed Number and Bool writes above. It does not guess
string/object coercion, numeric Boolean conversion or legacy SWF conversion
behavior. Fractions remain f64 twips with no added pixel/twip rounding; native
setter quantization and floating-point edge behavior remain unverified.

Pointer input is `Option<[f64; 2]>` in the Scene's absolute stage/world-twip
space. None means unavailable and causes a mouse-property read to reject. Screen
pixels, viewport origin, letterboxing, DPI and window coordinates must already
have been converted by the caller; no viewport or (0,0) pointer is invented.
Nonfinite samples reject without replacing the prior sample.

The inverse uses all six affine components, including rotation, skew,
nonuniform scale and translation. It does not subtract translation alone,
round the result or apply an epsilon-based scale fallback. A zero determinant
rejects the mouse read as `SingularTransform`; unrepresentable/nonfinite
arithmetic also rejects. Finite zero-scale source nodes can still be stored and
have their local X/Y/Visible queried or written without attempting an inverse.
This is ordinary f64 matrix arithmetic, not a claim of exact rational arithmetic
or a native floating-point comparison.

## Updates, source preservation and rendering handoff

A successful X/Y or Visible write recomputes the target's entire descendant
subtree in preorder. Other branches are untouched. A child's effective
visibility is its local visibility AND its parent's effective visibility; hiding
a parent never changes the child's saved/current local Visible property.

Each write reserves its bounded descendant work and computes every candidate
world transform before changing geometry. A nonfinite result or insufficient
budget leaves local/world transforms and visibility unchanged. Request/work
counters can still advance on rejection. An entire action stream is not
transactional: earlier successful writes remain if a later VM instruction fails.
The caller must not assume retry is safe.

`snapshot_nodes()` clones original source nodes and replaces only their current
`world_matrix` and `effective_visible`. Raw placement matrices/visibility,
node/parent/child IDs, source movie/definition/import identity, action indices,
clip-action spans, names, masks, colors, filters and scaling grids remain intact.
The root renderer can copy these instances into identity-matched PreparedScene
nodes, or prepare a derived Scene retaining the original movie/action/opaque
records. This handoff does not mutate the original Scene or claim to update the
serialized source placements. Current local matrices remain available separately
through `DisplayNode`.

`traces()` retains successful reads/writes, their typed property, context and
target, source instruction origin, resolved node, supplied pointer and returned
or before/after value. This is an execution receipt, not an automatic event log.
All of its admitted values are scalar values or explicit bound identities.

## Bounds and rejection behavior

Default limits are 100,000 nodes, depth 256, 2,000,000 copied metadata items,
16 MiB copied source strings, 100,000 bindings, 100,000 property requests and
2,000,000 cumulatively recomputed nodes. The depth hard maximum is 256. Source
metadata budgets cover child/mask/action/scaling-grid/unsupported vectors,
placement filters/clip actions, and imported-symbol chains; string budgets cover
placement/segment names, class names and import URL/symbol strings. These are
checked before source metadata is cloned.

Property requests consume a bounded counter, including requests subsequently
rejected for unknown targets, missing pointer or unsupported properties. Traces
are bounded by successful requests. Work reservations bound repeated root-level
updates. Integer counter overflow, revision overflow and nonfinite geometry
reject explicitly. No successful fallback value or silent partial geometry
result is returned.

## Verification

**VERIFIED (focused original-code checks):** four selected tests passed for full
inverse coordinates under a rotated, nonuniform parent; the pixel/twip boundary;
fractional X writes; subtree-only transform updates; local versus effective
visibility; preserved source identities/raw placements; missing pointer; singular
inverse; unknown context/path/property; read-only and coercion rejection; binding
conflict; malformed hierarchy; and geometry remaining unchanged after update
budget exhaustion or arithmetic overflow. No routine full suite was run.

**OBSERVED (owned script, explicit host execution):** the actual frontend,
MarvelGame and font packages loaded into the existing MovieLibrary. The manager's
first-frame Scene contained exactly one MouseMove ClipAction, at original movie
span 7096+77, owned by `cursor_mc` (library movie 20, scene node 4). The probe
allocated one plain VM object and explicitly bound it to that node; it did not
run a constructor or any startup action.

The saved local/world translation was (-800,-800) twips, or (-40,-40)
ActionScript pixels. A supplied stage pointer of (2000,4000) twips produced the
following original six host calls:

| Instruction offset | Operation | Value in ActionScript pixels |
| --- | --- | --- |
| 7127 | Read X | -40 |
| 7138 | Read MouseX | 140 |
| 7140 | Write X | 100 |
| 7158 | Read Y | -40 |
| 7169 | Read MouseY | 240 |
| 7171 | Write Y | 200 |

The original 14 instructions completed with six host callbacks. Both the cursor
node and its geometry child moved to world translation (2000,4000) twips; all
other branches retained their transforms. The local mouse point then read
(0,0). Every source identity, raw placement, action index and mask list survived
the snapshot handoff. A derived Scene with these current world matrices prepared
successfully through the existing CPU bridge: six nodes, one original mesh and
one declared texture. No substitute geometry was created.

The probe also rejected a nonempty cursor-name target and accepted the same
explicitly bound Object target. The interpretation of the single saved handler
as an executable probe is deliberate; no ordering among multiple event handlers
is asserted. The original Scene and package/movie bytes were not modified.
The first probe stopped before VM execution because its harness expected -40 raw
twips instead of the actual -800 twips (-40 ActionScript pixels). The corrected
harness keeps this units distinction; no implementation change was needed. That
failed assertion and its source are retained privately rather than discarded.
Private evidence in `evidence/display-state/`: `cursor-smoke.rs`,
`cursor-smoke.tsv`, `cursor-smoke.stderr`, the retained initial assertion source
and failure, `focused-test.txt`, `source-snapshot.json`, and `verification.json`.
The receipt pins package inputs, exact source snapshot, compiled test/probe,
primary references and transcripts by SHA-256. All final commands exited zero;
the initial pre-execution harness assertion is explicitly recorded as failed.
No GPU, native-client execution, input delivery or initialized frontend behavior
is established by these checks.

## Source and licence ledger

All implementation is original Rust code. No third-party VM or renderer
implementation was read or copied for this increment.

| Source read | Rights/licence | Use |
| --- | --- | --- |
| Project `src/gfx/{scene.rs}`, `src/gfx.rs`, `src/avm1{.rs,/vm.rs}`, `src/frontend/{bootstrap,library,scene_content}.rs`, `docs/{GFX_SCENE,AVM1_VM,PROVENANCE}.md` | Original project code and existing source ledgers | Source geometry, current host seam, explicit initialization boundary and preparation handoff |
| Private `evidence/avm1-runtime/{run-owned-cursor.rs,cursor-execution-evidence.json}` and manager GFX evidence | Original probe plus proprietary owner-held game bytes; compatibility study only | Existing saved handler identity and distinction between supplied geometry and real scene geometry |
| [Adobe-authored MovieClip AS2 reference](https://open-flash.github.io/mirrors/as2-language-reference/MovieClip.html) | Adobe copyrighted documentation, maintained mirror; study only | Published X/Y, mouse-coordinate and Boolean Visible property contract |
| [Autodesk custom-cursor reference](https://help.autodesk.com/cloudhelp/ENU/Scaleform-Help/scaleform_help/as2_reference/mouse_extensions/changing_cursor.html) | Autodesk proprietary documentation; study only | Custom cursor example context; no implementation copied |
| Private pinned frontend, MarvelGame and font UPK packages | Proprietary owner-held client; private compatibility study | Actual source scene, saved handler and preparation smoke |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` | Installed REA operational instructions, not an implementation source | Evidence/observation boundaries; no new native REA claim in this glue increment |

An Adobe AS2 PDF mirror surfaced during reference discovery but the web reader
rejected its size; its contents were not used. Selected primary authored HTML
references are privately retained with hashes in `reference-manifest.json`.
The existing project REA evidence remains separate from the Rust-host execution
reported here. The lead integrates this ledger into `docs/PROVENANCE.md`.

## Typed attached-cursor validation

**VERIFIED original-script execution in Rust; supplied attachment state.**
Private `evidence/attached-display-integration/attempt-3/receipt.json` records
source `92bb18ba97a5365afdde9a8d0fea51ed279bf4ca` unchanged across compilation
and execution. The original MouseMove body at 7096 ran 14 instructions and six
property calls, moving the typed cursor to (100,200) pixels. Explicit DisplayClip
readback matched. Source/movie, parent, sprite, identity and replacement gates
were checked. Four focused display geometry tests also passed after integration.
This uses explicit known source identity and raw-state fixtures, without class
initialization, automatic attachment, child snapshots, event dispatch or timing
claims. Earlier product/harness compile failures remain preserved as attempts
1 and 2; they are not counted as passing runs.

The scene bridge also retains bounded tag-code metadata and exposes read-only
source/movie/version and typed-parent bindings. This allows a later TextField
binding to require an actual DefineEditText tag (37), rather than accepting every
character classified as text. Tag counts are charged to the existing source-item
budget. Window-feature compilation passed with this metadata addition.
