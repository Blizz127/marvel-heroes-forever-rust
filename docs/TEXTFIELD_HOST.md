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
