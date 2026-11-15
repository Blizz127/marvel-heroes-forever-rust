# Ordered saved handler bindings

`frontend::saved_handlers::SavedHandlers<I>` retains the ordered saved action
handles for one supplied attached owner. `I` is the owning VM's action arena
handle, not a CodeId, source span, scene node or bytecode copy. This original
module applies the installed-wrapper and split-key contracts recovered in
[event dispatch](FRONTEND_EVENT_DISPATCH.md) and
[saved execution](SAVED_EVENT_EXECUTION.md).

The initial profile accepts native Load (`1`), Construct (`0x40000`) and their
combined aggregate (`0x40001`). It rejects all other masks, including key events
whose descriptors have additional identity fields. Serialized ClipEventFlags
are not automatically converted to native masks by this module.

Installing a combined aggregate appends the same action handle to both keys.
Each key retains installation order. The original aggregate remains available
independently of the selected key: selecting Construct from a combined mask
does not turn its mode-3 callable into mode 4. Installing identical source twice
with distinct action handles creates two ordered entries and preserves their
separate VM source-wrapper state. Reusing an installation handle is rejected
by this fresh-installation API.

The caller must obtain owner/mask facts from the same VM arena before installing
a handle. The table checks the supplied owner, duplicate identity, supported
mask and installation/reference budgets before publishing entries. Failure
leaves logical entries and order unchanged; vector reservation can change
capacity. Successful allocation is the supported profile, not a model of
native allocation-failure behavior.

Lookup returns presence and an ordered borrowed slice. It does not execute
anything, classify a native callable return as successful delivery, or select
a dynamic `onLoad`/`onConstruct` member. Dynamic lookup belongs after saved
calls return. Borrowed lookup excludes mutation during the bounded delivery
loop; native reentrant handler-map/vector mutation remains unsupported. Empty
static vectors and arbitrary noncallable saved Values are outside this
source-wrapper installation profile.

The table is compiled into the frontend module, but startup does not yet use
it to deliver events. Central VM saved-mode execution, original source loading,
raw target/descriptor gates and lifecycle scheduling remain separate work.

## Validation and provenance

Focused synthetic checks cover shared handles across split keys, distinct
installations, per-key order, retained aggregate masks and rejection without
partial publication. They do not execute original actions or demonstrate
native event timing. Private terminal output and source hashes are retained
under `evidence/saved-handler-table-*` in the private project directory.

**VERIFIED Rust state checks:** both targeted tests pass with exit 0 and no
compiler warnings (373 unrelated tests filtered out). The private
`saved-handler-table-test-result.json` retains terminal output;
`saved-handler-table-verification.json` binds the module/hook hashes and command.

There are no new external dependencies or implementation sources. The native
evidence/source ledger is inherited from the two owned-executable studies linked
above. No original action bytes, native tables or captures enter the repository.
