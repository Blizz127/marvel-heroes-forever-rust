# AS2 root-context class registry

`frontend::class_registry::ClassRegistry` supplies one bounded registration map
for one explicitly owned VM/AS2 root context. It stores actual constructor
ObjectIds under exact UTF-16 names and retains registration/definition origins.
It does not create instances, select assets/exports, modify an existing instance,
construct classes, execute constructors or run startup actions.

## Native evidence and scope

**OBSERVED (retained REA native instructions):** the retail registration callback
at `0x1401d4870` reads its map from the current environment's root registry
context, then uses the context's `+0x18` map. The root/context getter at
`0x14017b880` and allocation path at `0x1402100f0` establish a fresh context during
root initialization. The call-site movie and constructor-definition movie do not
select separate namespaces.

The callback's version-greater-than-six branch compares interned key identities
without the older lowercasing path. Replacement updates an existing function
reference; insertion adds one; both return true. The null-removal helper at
`0x1401c3280` returns false for an absent key and true after removing a present
key. The recovered lookup follows the same map. The final [native registry ledger](REA_CLASS_REGISTRY.md) also establishes a
later same-map lookup after an initial miss. Final dispatch and scheduler order
remain unresolved; this registry does not implement that larger lifecycle.

Exact retained records under private `evidence/rea-class-registry/`:

| Receipt | Evidence ID | Used boundary |
| --- | --- | --- |
| `entry.bundle.json` | `ev_9d9c5487c7b09cd8ecf432cf48e3273921ced2a921bc61e7080f2b3b8886a779` | Registration callback, case/version branch, replacement/insertion and result |
| `helpers.bundle.json` | `ev_03a87c62e4c5bcea6484d7582ccf5a695f0d004b677cc6dbb568c6c2f57ade6e` | Removal membership and function-reference helpers |
| `owner.bundle.json` | `ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a` | Environment/root ownership and context allocation |
| `lookup.bundle.json` | `ev_6a3ef294e0f93be00362ab76b3f85adad17bf1242bc644ad9fa95b6009d76dc4` | Same-map lookup path |

These REA records report complete, untruncated provider commands, exit zero and
unchanged selected-artifact hashes. They retain Rizin identity/version and
external-input limitations. They are static instructions from the pinned retail
executable, not a native runtime trace. Private immutable copies of the exact
receipts read by this implementation are in
`evidence/class-registry/native-evidence-inputs/`, indexed by
`native-evidence-manifest.json`.

## Host API and identity

```rust
let mut registry = ClassRegistry::new(vm.root(), movie_version, Limits::default())?;
let accepted = registry.register(request)?; // ClassRegistration -> Result<bool, String>
let registered = registry.lookup(name_utf16_units)?;
```

`new` admits the current VM's version 7–19 profile. Versions six and below reject
because their native case-folding semantics are not fully recovered. Higher
versions remain outside this VM profile. The supported native branch is greater
than six; the upper bound comes from the current VM, not a newly inferred native
cutoff. Native matching uses the calling environment's cached version; this
field records the current VM's single-version admission profile, not a new
process-wide or root-version rule. Mixed legacy/modern environments would require
additional per-request matching support before being admitted. The caller must
supply the correct loaded version and retain exactly one
registry in each VM/root host. Constructor and root ObjectIds are VM-local;
numeric equality across different VMs does not join their registries.

`RegistryContext { root, movie_version }` is diagnostic metadata. It is not a
new global namespace key or a native pointer. The registry is not Clone and
contains no process-global map. `ClassRegistration` has no VM token, so the caller
must route it to the registry owned by that VM/root. This module cannot validate
an unrelated VM heap, and it does not reinterpret the request's origins as a
substitute identity check.

`register` expects the existing VM's validated function-or-explicit-null request.
A non-null request stores `RegisteredClass { constructor, origin,
constructor_origin, revision }`. Neither origin is required: a native constructor
can lack a script definition origin, and an explicit host-driven VM call can
lack a script call-site origin. Replacing a key stores the new constructor
ObjectId and origins without wrapping or copying the function. Removing a key
changes only this map; an already returned/copied constructor reference or
already created instance is not rewritten.

The map compares all UTF-16 code units exactly. It performs no case folding,
Unicode normalization, lossy String conversion, movie-name qualification or
filename lookup. Embedded zero units and unpaired surrogates are retained by
this storage layer; native string-conversion equivalence for those unusual
inputs has not been established. Empty names explicitly reject, matching the
current deliberately narrower VM profile. Native callback string coercion and
its acceptance of additional arguments remain VM work, not implicit host changes.

`lookup(&[u16])` returns an immutable optional RegisteredClass after name-budget
validation. It performs no constructor invocation or display lookup and has no
side effects. A missing lookup is not cached: a subsequent lookup reads the
current map, which is necessary for the recovered native late-lookup path. The
caller remains responsible for when to perform that lookup. `entries()` provides sorted UTF-16 inspection order only; that
order must not be exposed as recovered native hash or enumeration order.
`context()`, `stats()` and `events()` provide bounded diagnostics.

The optional `Host` implementation delegates `register_class` and rejects native
calls and commands. A composite BootstrapHost can instead call `registry.register`
from its own callback while routing native Selection calls through SelectionHost.
No Selection object, VM builtin installation or module hook is installed here.

## Mutation receipts and budgets

Each accepted request produces a `RegistrationEvent`: original request, previous
mapping if any, Inserted/Replaced/Removed/Absent outcome, returned bool and map
revision. This preserves removal/replacement provenance without changing the
callback's result. Insertion, replacement and present removal advance revision;
an absent removal returns false without advancing it. No source lookup or
constructor activation is inferred from an event.

Defaults bound 16,384 live entries, 4,096 UTF-16 units per key, 1,000,000 live key
units, 100,000 registration calls, 100,000 accepted events and 4,000,000 history
key units. History accounting conservatively counts each accepted request's key
length, even when Arc storage is shared. Replacement consumes no additional live
key capacity; removal releases live capacity but retains its bounded receipt.
Lookup work is bounded by the key limit and live map size.

Every relevant capacity check occurs before membership or stored metadata
changes. Call accounting can advance before a subsequent rejection; events,
live-key counts, revision and membership remain unchanged on an error. Exhausted
history can therefore reject a removal explicitly, preserving the mapping. The
host does not silently discard evidence, return false for a budget failure or
pretend an association was accepted. Integer overflow produces an explicit error.
Rust allocation failure and the future VM's garbage-collection/reference-root
integration are outside this increment; the present VM retains its heap objects
for its lifetime.

## Focused validation

**VERIFIED (original-code tests):** four selected tests passed. They cover
cross-origin replacement in one root context; exact constructor and call/definition
origin retention; present/absent removal; isolation of separately owned contexts
even with equal numeric VM-local root IDs; case differences, embedded zero units
and unpaired-surrogate storage; all six resource limits; and replacement at live
capacity. Rejected history admission leaves the existing mapping unchanged.
A real VM builtin call registered an actual callable native ObjectId, returned
Boolean results for insert/removal/absent removal, and never invoked that
constructor. Versions six and below reject explicitly. No routine full suite ran.

**OBSERVED (owned bytecode in the Rust VM):** the controlled probe loaded the
original manager and login GFX inputs. It selected each original constant pool
and constructor-definition instruction, supplied an explicit root variable
binding, then executed each entire saved registration snippet:

| Original registration | Source span | Call offset | Original constructor definition | Instructions | Registry entries after call |
| --- | --- | ---: | --- | ---: | ---: |
| Manager `Cursor` | 2055420+64 | 2055481 | Manager definition at 55654 | 8 | 1 |
| Login `TextOnlyButtonWide` | 167848+76 | 167921 | Manager definition at 152783 | 8 | 2 |

Both mappings lived in one explicitly owned version-15/root-ObjectId-1 registry.
The login call retained its distinct call CodeId and manager constructor CodeId;
neither origin partitioned the map. The resulting two entries used 24 live UTF-16
key units and revision 2. A second separately owned registry with the same
numeric root ID had no mappings, and lowercase `cursor` did not match `Cursor`.
Both original constructor objects were retained without invocation. Complete
class initializers, display creation and native execution were not run.

Private evidence under `evidence/class-registry/`: `check.rs`, `focused-test.txt`,
`owned-register.rs`, `owned-register.tsv`, `owned-register.stderr`,
`source-snapshot.json`, `native-evidence-manifest.json`, and `verification.json`.
The receipt pins original input hashes, the frozen original-project source,
compiled probes and exact retained REA receipts. Both compiles, the four selected
tests and the actual registration probe exited zero. This stage does not claim
native instantiation, complete startup or initialized UI behavior.

## Sources and licence ledger

All code is original. No third-party implementation was copied.

| Source read | Rights/licence | Use |
| --- | --- | --- |
| Project `src/avm1{.rs,/vm.rs}`, `src/frontend/{bootstrap,selection}.rs`, `docs/REA_CLASS_REGISTRY.md`, and their original APIs | Original project code; earlier format/source studies retain their existing provenance | Exact callback/value identity, composable host boundary and explicit lifecycle scope |
| Private REA entry/helper/owner/lookup bundles and their derived assembly views listed above | Original REA results over owner-held proprietary client; private compatibility study | Native map ownership, version branch, insert/replace/removal/lookup contracts |
| Private `evidence/avm1-runtime/{run-owned-register-class.rs,register-class-evidence.json}` | Original project probe over owned game bytecode | Reuse source spans and controlled definition setup for actual registration checks |
| Owner's private manager and login GFX inputs | Proprietary game files; private compatibility study | Execute selected original definitions and full original registration snippets |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` | Installed REA operational instructions, not an implementation source | Retained Evidence handling and distinction between static native evidence and Rust execution |

No new network reference or external implementation was read for this registry.
The lead integrates this ledger into `docs/PROVENANCE.md`; this lane changes only
`class_registry.rs` and this document.
