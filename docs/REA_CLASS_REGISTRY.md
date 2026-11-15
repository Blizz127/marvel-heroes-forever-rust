# Native AVM1 class registry

**OBSERVED (REA, pinned retail executable):** `Object.registerClass` stores its
name-to-constructor association in a registry owned by the calling environment's
AS2 runtime root. The stored constructor's defining SWF does not select the map.
The map is also read by a deferred display-constructor callback. A runtime root
identity is therefore necessary; a static movie path or `CodeOrigin` alone is
not the registry namespace.

This is a static compatibility finding, not a native execution comparison. The
terms *root*, *context* and *FunctionRef* below describe recovered roles and data
relationships; no original C++ type declaration or leaked SDK source was used.

## Registered entry and supported behavior

**OBSERVED:** The literal `registerClass` at `0x14250c388` is paired with callback
`0x1401d4870` in the table at `0x14250e7e0`. The decoded callback directly uses
diagnostics naming `Object.registerClass`. The initialization prefix at
`0x14020ea40` passes that table to `0x1401e9bd0`, whose complete body iterates
16-byte name/function records, creates native function values and installs them
on the supplied object interface. The callback's complete contiguous unwind
fragments cover `0x1401d4870..0x1401d4cf0`.

A separate selected initialization store at `0x14020fa85..0x14020fad8`
publishes a function value through context `+0x60` using the interned key at
root `+0x140`. REA confirms that builtin-array index 1 is `Object` and that its
array starts at root `+0x138`. This corroborates the named diagnostic and table;
the intervening initialization instructions were not all decoded with REA, so
this selected store alone is not treated as a complete register-liveness proof.

The callback initially writes Boolean false to its result. It checks for at
least two arguments, converts argument zero through a shared string helper,
and tests argument one's native value kind. Function kinds `0x08` and `0x0b`
take the registration path; null kind `0x01` takes removal. Other kinds report
the function-or-null diagnostic and retain false. Too few arguments likewise
report an error and return false.

For the project's supported nonempty-string/function-or-null input profile:

| Operation | Observed map action | Boolean result |
| --- | --- | --- |
| Register a new name | Insert the name and retained function reference | true |
| Register an existing name | Replace its retained function reference | true |
| Remove an existing name with null | Remove the matching map entry | true |
| Remove an absent name with null | Leave the map unchanged | false |

Replacement uses `0x14018c6e0`, which releases the previous retained reference
and copies the new reference, auxiliary frame pointer and flags. Insertion is
performed by `0x1401a3d20..0x1401a3eb4`, including its contiguous unwind fragments.
The callback returns true after either normal path. Allocation failure and
exception behavior have not been established.

Complete removal helper `0x1401c3280..0x1401c333a` first checks membership. A
missing table/key returns false; a present key dispatches the version-selected
removal helper and returns true. The outer callback copies that Boolean into
its result. This distinguishes absent removal from successful removal without
assuming an unconditional-success stub.

The current VM's exactly-two-argument, nonempty-string profile is narrower than
the native entry: native code accepts an argument count greater than two and
does not itself test string length before registration. Full first-argument
coercion, empty-name behavior and native kind `0x0b` resolution remain outside
this finding's supported input profile. The display lookup callback does reject
an empty name before performing its map lookup.

## Exact runtime ownership

Let `E` be the calling native environment, `T` its display target, `R` the
associated AS2 root, and `C` the allocated registry/context owner. The observed
paths are:

```text
registration: C = *(E + 0xe8); map address = C + 0x18
environment setup: C = *( *( *(owner + 0x20) + 0x20 ) + 0x50 )
                              T              R
root initialization: allocate C (0x78 bytes); R[0x50] = C
context initialization: C[0x28] = MovieImpl
root binding: R[0x10] = MovieImpl; MovieImpl[0x18] = R
```

The notation above denotes byte offsets and pointer reads, not Rust array
indices or proposed public ABI structs.

**OBSERVED:** Environment-owner constructor `0x140203560` stores its target at
owner `+0x20`, constructs the environment at owner `+0x38`, and initializes
`E+0xe8` through two virtual calls. For the independently traced factory case,
the owner vtable is `0x142517670`. Its `+0x08` slot points to the complete
four-byte getter `0x1405b93f0`, which returns its receiver. Its `+0xb8` slot
points to `0x14017b880`, which returns the pointer reached through receiver
`+0x20`, target `+0x20`, and root `+0x50`.

Root-vtable `0x142511240` slots `+0x08` and `+0x88` point to
`0x1402100f0` and `0x140210180`. Both complete bodies allocate a fresh 0x78-byte
context, zero its registry field at `+0x18`, call `0x14020ea40` with that context
and the associated MovieImpl, release the previous root `+0x50` reference when
present, and store the new context there. The latter routine also performs other
root initialization. Its complete lifecycle semantics are not assigned here.

Existing independent REA receipts tie the same root to the target: the factory
loads `MovieImpl+0x18`, passes it to the target constructor, and that constructor
stores it at target `+0x20`. Root constructor `0x1401d4490` stores MovieImpl at
root `+0x10` and calls binder `0x1400559f0`, which stores the root at
MovieImpl `+0x18` and `+0x4368`.

Thus the observed map belongs to one allocated runtime context associated with
an AS2 root/MovieImpl instance. Multiple static SWF definitions can contribute
code or symbols while sharing that runtime context. Conversely, two independently
allocated runtime roots must not share a registry merely because they load the
same file. The native code here does not key registration by caller SWF,
constructor SWF, export-owner SWF, movie filename, or movie-definition pointer.
Caller and constructor origins remain useful provenance but do not replace the
runtime context identity.

## Name matching and display lookup

**OBSERVED:** Registration, removal and the deferred callback compare the
calling environment's version byte at `E+0xf0` with 6. Above 6, hash lookup
compares interned name-node pointers exactly. At version 6 or below, lookup uses
the alternate node cached at string `+0x10`, preparing it through
`0x140097b50` when absent. The complete alternate lookup helper is
`0x1401822e0..0x1401823a4`.

Exact interned-name matching supports a case-sensitive key for the observed
modern frontend movies. The alternate path must not be implemented with an
invented ASCII or Unicode fold rule: this investigation has not decoded that
string helper's complete transformation. The independently established
environment-version producer must be used, rather than a process-wide version
or arbitrary imported movie version.

**OBSERVED:** Callback `0x1401cc220..0x1401cc83d` reads the same
`C=*(E+0xe8)` and hash at `C+0x18`, uses the same version split and matching
operations, and retains the resulting function reference. It passes the
function's object interface and target to `0x14019c5e0`, then prepares records
with raw kinds 2 and 3 for `0x14017cf80`; the kind-3 record contains the retained
registered function. A missing key takes the kind-2 path without that function.
This establishes shared registration/lookup storage.

**OBSERVED:** Complete helper `0x14017cf80..0x14017d059` executes those records;
it is not a queue-insertion helper. After target-state checks and target-adjusted
AVM dispatch, raw kind 2 uses virtual `+0x128`, kind 3 passes the retained
FunctionRef at record `+0x38` to virtual `+0x130`, and kind 4 passes the saved
native callback at `+0x50` and argument array at `+0x58` to virtual `+0x138`.
The final virtual-call implementations and complete scheduler order were not
decoded in this increment.

The complete producer `0x1401d0f70..0x1401d13b8` obtains a context through its
receiver's virtual `+0xb8`. It obtains the target's definition through virtual
`+0x208`, passes a resource ID to `0x14008b040`, interns the returned name, and
looks it up in context `+0x18`. Helper `0x14008b040..0x14008b0e1` performs a
numeric-key lookup in a definition-owned map and returns the associated string
wrapper. The separate loader that populates this map was not traced here; its
precise alias-selection rule remains unknown.

When that initial lookup succeeds, the producer retains the function and fills
records returned by `0x1401c7dc0` with raw kinds 2 and 3. When it misses, it
instead stores raw kind 4, callback `0x1401cc220` and the name argument. Therefore
the native program contains a later lookup path, not merely one irrevocable
lookup at display creation. Replacement or registration before that callback
executes can matter; exact intervening scheduling has not been established.
Complete second producer `0x140203920..0x140204229` corroborates the same
definition/name/context map and callback, with additional branch conditions and
action records. Those broader event-order branches are retained as evidence,
not implemented by guessing their order.

A selected display-owner constructor prefix at `0x14020d5a0` stores its target
at `+0x20` and installs primary vtable `0x142517d70`. REA confirms that this
table's `+0xb8` entry is the already decoded `0x14017b880` context getter.
Its secondary interface at `+0x30` has thunks `0x1401d4310` and `0x1401d4320`,
which subtract that displacement before entering `0x1401d13c0`. Ordinary
candidate disassembly connects that wrapper to the complete producer above;
the whole wrapper is not promoted to a REA-decoded body. Independently, the
deferred callback's direct `E+0xe8` map access establishes its shared namespace
without relying on this candidate wrapper edge.

Keep constructor-definition provenance with the retained function; it does not
select this class registry. Export resolution remains a separate step: this
investigation does not establish duplicate-export precedence, imported
alias choice, unload semantics, or the full action/initialization order. A
registry implementation can implement the observed map contract now while
display construction keeps these remaining behaviors explicit.

## Evidence and limits

**VERIFIED:** The target is the privately pinned Win64 1.52.0.1700 executable,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Selected REA Rizin commands use explicit byte ranges and do not launch the
client, open a live debugger, run automatic whole-image analysis or change
shared configuration. All six REA calls completed with exit code zero,
untruncated output and unchanged target hash. Rizin warned that its automatic
string inventory skipped large image regions; the explicit selected reads were
complete, and no exhaustive string-inventory claim is made. Complete canonical
receipts and derived private assembly
readings are under `evidence/rea-class-registry/` in the private project store.
Some initial probes ended at unwind-fragment boundaries; the referenced complete
helpers were subsequently read through their remaining contiguous fragments.
The initialization prefix is intentionally a prefix, not a claim to decode all
of the large initialization function.

Own REA evidence:

- `ev_9d9c5487c7b09cd8ecf432cf48e3273921ced2a921bc61e7080f2b3b8886a779`:
  literal/table, complete register callback, initialization prefix.
- `ev_03a87c62e4c5bcea6484d7582ccf5a695f0d004b677cc6dbb568c6c2f57ade6e`:
  removal helper, insertion prefix and initial helper fragments.
- `ev_5d1d7b7e92625bf40b5f3383c74badbe7bba1b79b7f85184dfb845c411084d9a`:
  owner getters/slots, both context allocation methods, insertion tail.
- `ev_6a3ef294e0f93be00362ab76b3f85adad17bf1242bc644ad9fa95b6009d76dc4`:
  root slots, complete reference assignment/table installer, folded-lookup tail
  and complete deferred lookup callback.
- `ev_48e57d0b466d5180f8f3f83037835dd3c251e04f165a6dfbe215ef2b0c83fe0e`:
  both complete display/name lookup producers and callback-record preparation.
- `ev_598fe04144afafa708e23ddbfcfe29fe4ca8a2aaaf87b83bdacdb7b24e8ea2f6`:
  action-record executor, function-property helper, definition numeric-name
  lookup, selected display-owner/thunk/slot proof, Object-key store and literal.

`final.bundle.json` retains all six canonical Evidence records;
`verification.json` records their checked completion and source identities.
The independent REA ledger was checkpointed and reset after export, with no
active target retained.

Reused independent REA evidence, without repeating those producer calls:

- `rea-asflags-followup-version-proof/environment-owner.bundle.json`,
  `ev_6bbbd9e3d4aafddee5a2f80cf5a75fb643d3404a1d6c9474043ee9e3fcba716d`:
  environment construction and target-adjusted virtual dispatch.
- `rea-asflags-followup-version-proof/final.bundle.json`,
  `ev_e4fdacf297d28e87b9f3ea63f71cbe0822727757d4037dc66cd5495bbf80f4c3`
  and `ev_63afb28a4e5a8587d3d55625096a552ed22a947f9c384cac9c1ace7f5dfa051c`:
  MovieImpl/root reverse binding, factory and target constructor.
- `rea-native-alternative/extends-name-binding-final.bundle.json`,
  `ev_5de603be82d92b5907b35369248200732fda049b1d4c60fe5880ced8d9b92b76`:
  root construction, builtin-string array and reverse-binder call.

Read-only Python/objdump scans supplied selected address candidates only. Their
receipts are private and are not substituted for the cited REA instructions.
No native invocation, cross-root runtime test or full rendering/gameplay parity
claim follows from these static observations. This increment changes only this
document; the integrator owns VM, registry and display implementation/tests.

Source provenance: Blizz's private retail executable (proprietary; authorized
compatibility/interoperability study), the local REA skill/usage guide (tool
instructions; installed-tool licence not inspected; no game implementation
copied), and previously retained project REA receipts (owner-private derived
evidence). All binary bytes, instruction listings and derived captures stay
private. No external SDK implementation or new public source was used.
