# Attached TextField host

**INFERRED implementation from VERIFIED native contracts:**
`frontend::textfield_host::TextFieldHost` connects an explicitly adopted typed
AVM1 TextField to its exact scene instance and conditional `TextFieldState`.
`FrontendHost` forwards typed `text`, `html`, and `htmlText` requests to this
optional host. [TextField runtime](TEXTFIELD_RUNTIME.md) records the owned
executable evidence; [instance text preparation](SCENE_TEXT_INSTANCES.md)
describes the separate renderer preparation seam.

Admission checks the supplied VM/scene lifetime domain, attached owner, source
movie and header, EditText ID, tag 37 definition, immediate scene parent, and
parent/environment chains reaching the adopted root. Field, scene-node and
owner identities must be unique. An absent watch state and all conditional
state/profile inputs are explicit caller assertions backed by the applicable
snapshot evidence. The opaque native prototype identity is retained; this host
does not manufacture a generic prototype or infer native fresh-field defaults.

Additional caller contexts must be bound explicitly. Each request must match
the admitted field, owner, display target, source and environment owner. The
caller must retain the same VM and Scene lifetime; numeric IDs do not establish
cross-VM identity. There is no automatic attachment, retirement, reparenting,
constructor execution, style-handle resolution or lifecycle scheduling.

Writes require actual Boolean/String values. Strings must have valid UTF-16,
no embedded NUL and at most the state's supported input bound. The underlying
conditional state currently admits its documented plain ASCII profile; other
HTML, translator, stylesheet, editor, variable and image-callback paths remain
unsupported. `text` returns document text. An active HTML `htmlText` getter
rejects until native serialization is implemented; it does not echo the input.
Unchanged-input and forced-rebuild behavior comes from `TextFieldState`.

Default bounds are 4,096 fields and contexts, parent depth 256, 100,000 admitted
requests and 16 MiB of retained last-input plus document bytes. The latter is
accounted string storage, not RSS. Request attempts consume the call budget even
when a later identity or backend check fails. Successful operations record
source origin/offset, field, scene node, context, environment and compact result
metadata without retaining text histories. A failed replacement preserves the
admitted document, formatting and retained-byte accounting. Native VM storage
may already have been allocated before the backend rejects, as required by the
typed setter contract; that storage is deliberately retained.

**VERIFIED focused Rust checks:** the product compiled with the window feature.
`cargo test --offline -j2 frontend::textfield_host -- --nocapture` passed both
tests. Synthetic bytecode wrote `html` and `htmlText`, then read `text` through
the typed VM and composed `FrontendHost`. It changed only one of two instances
sharing an EditText definition, retained the neighbor, and checked source-offset
traces and string accounting. An unsupported replacement preserved backend
state while retaining the VM's allocated native storage. The second test
checked domain, scene-node, context, owner, environment and retention-budget
failures. The fixture contains original synthetic metadata, not game bytes or
an assertion of retail fresh-state defaults.

The actual login target comes from GFX 15 while its imported UISource field
comes from GFX 11. At this commit the legacy VM adoption gate still rejects that
mixed-source route. Its original localization/update-label sequence has **not**
run through this host. A later explicit selected-MovieDef binding witness and
the native String uppercase path are required before that check can proceed;
source versions and the saved force-uppercase flag must remain unchanged.

The subsequent [source/Environment increment](TEXTFIELD_SOURCE_ENV_VERSION.md)
adds explicit selected-MovieDef admission and local-field adoption from that
parent movie. It preserves the legacy gate and requires the caller's bounded
ordinary-binding profile; a version number alone does not prove the native
loader selected that binding. At the dev11 build checkpoint, original-label
execution through this newly admitted route is still unverified.

Sources are the owner's private executable/package studies cited above and
original project modules. No outside implementation was copied. These checks
do not establish native startup, rendered text updates, a functioning login
screen, or retail behavior parity.

## Frozen original-script composition check

**VERIFIED Rust execution, 2026-11-15:** private fixtures frozen from
`d4587131c32a821b8729b3050921a3c15ffe6a97` exercised the actual saved login
scene and unchanged selected manager methods through this host. The earlier
`a86cc5a` snapshot compiled both fixtures and rejected imported parent node 86
at the legacy version guard. Its receipts remain separate; no later script or
host check was claimed from that rejected run.

The retained scene selects login movie 13/header 15, its local import 170,
UISource movie 19/header 11 exported sprite 1039, and local EditText 1034/tag 37
at node 89 beneath parent 86. The package's saved `References[67]` is positive
reference 24 to local SwfMovie export 23; its separate `ReferencedSWFs[0]`
names UISource. The resolver reports `SavedReference`, and the exact exported
symbol resolves uniquely to 1039. The English font inspection alias affects
only the font import; it does not establish this movie binding or native locale
selection. Neighbor node 136 shares EditText 1034 but has a different parent
and remains an independently bound instance.

The new adoption APIs receive an **explicit supplied profile**: the final
native resource pair is the eligible ordinary local/exported pair, loader
compatibility succeeds, and no later resource-list substitution occurs. Saved
bytes and Rust scene linkage do not observe that final retail decision. Source
11, the parent target's selected MovieDef 11, the first marked ancestor/root
Environment 15, and captured manager-script target 15 remain separate fields.
Other supplied conditions include live target identities and raw flags, opaque
current prototype and format/default handles, absent watches and unsupported
text side paths, the controlled player/handler/receiver association, and the
English catalog. The controller and slot-data receivers are explicit Plain
fixtures; no class constructor or saved Construct event is executed.

Both binaries built offline/locked with `nice -n 19`, `-j2`, and exited zero.
The original-script fixture checks twelve unchanged selected declarations and
three original constant pools against the owned manager file. With controlled
`forceAllCaps=false` and `verticalCenter=false`, the original
`updateLocaleTextFromLabel` route obtains **`Log In`** from actual
`ConfiguredLabels` and `LocaleCatalog` through the fresh ExternalInterface graph,
uses the original slot-data setter/getter, and enters original
`updateLabelText`. The typed writes are `html=true` at source offset 94201 and
`htmlText="Log In"` at 94252, with correct code origin, field, owner, source,
active context and attached Environment. The original false recenter branch
returns normally. This call executes 142 instructions and three host calls.
The backend applies the supplied definition-format handles and invalidation
state; it does not derive native fresh defaults from the authored field.

The second case preserves the saved true-uppercase branch. It obtains the same
unmodified locale value and writes `html=true`, then rejects the original
primitive `toUpperCase` call at 94251. The prior HTML change and lazy VM storage
remain; no `htmlText` write or recenter call is claimed. No pre-uppercased result,
replacement script or substitute native callback was used. Across the two cases,
two actual locale callbacks and three successful typed field writes occur.
Neighbor 136, a nonempty Selection listener set and a nonempty class registry
including its audit event remain unchanged.

The separate fixture records 22 bounded case receipts: four binding identity
rejections, two direct host read/cache groups, eight request identity/context
rejections, five host/state budget groups, and three original-VM setter failures.
Direct reads verify document text and HTML flag; active HTML serialization
rejects. A repeated input preserves a deliberately different document/cache
pair. Original unsupported markup fails at 94252 after allocating field storage
and toggling HTML; both effects remain while document/formatting and the neighbor
stay intact. VM host-call limits zero and one reject at 94201 before storage and
94252 after storage/HTML respectively. These read checks use the host API;
this receipt does not claim an original VM getter path or other VM budget classes.

Private receipts live under
`~/.local/share/marvel-heroes-forever-rust/evidence/textfield-host-validation/`.
`runs/d458713/receipt.json` has SHA-256
`9c14099a7486d0e632c5e4c2b132eb3ceb6ece3369519a695c26dc2d5b4bb7e5`;
`MANIFEST.validation.json` indexes 186 retained artifacts and has SHA-256
`43c193305f094b2ab31471fcf43136b43dc64d5ce1c104fb4d43b710b4aa7416`.
All 186 frozen product source files, 28 private harness files and 32 input paths
are unchanged before/after execution. Dependency metadata is byte-identical;
the locked graph contains twelve packages, including the root's SHA-256
library dependencies. Exact executed binaries, commands, logs, source archives,
script comparisons and the static import witness are retained privately.
The initial capture command's wrong private manifest path is preserved alongside
its corrected invocation. No product source was changed for this check.

This closes the conditional original-label-to-state composition path for the
false-uppercase branch. It establishes neither native startup/final binding
selection nor rendered text, full initializer/event execution or retail parity.
The true-uppercase failure belongs to this immutable snapshot and must remain
visible when a later String implementation is tested.
