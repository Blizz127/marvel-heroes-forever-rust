# Mouse user-event routing and host-state prerequisites

**VERIFIED bounded static route:** native `Mouse.hide()` selects the movie
through the active Environment and sends event type `0x16` to the handler
cached at **MovieImpl `+0x228`**, if present. That cache is selected **state ID
4**, with retained-reference ownership. It is not state 5, the external-call
handler at `+0x238`, or the unrelated display-target virtual slot `+0x228`.

**The concrete state-4 provider and its event method are not established by
this study.** The traced engine/loader setup installs other typed states.
Those bounded observations do not establish global absence, a current movie's
selected handler, or an OS cursor-visibility effect. Implementing unconditional
cursor hiding from the event's name would exceed this evidence.

The source is the owned Win64 build 1.52.0.1700 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All findings are static artifact evidence. No original client, service,
network, debugger, account or current-state probe was run. Existing native,
access and live-service holds remain unchanged.

## Callback input, ordering and exact event

Complete callback `0x1401b6fe0..0x1401b70bd` first releases the existing
result through `0x1401887a0` and writes result Value tag 0 at
`0x1401b6ff9`. It then obtains:

```text
E = FnCall[+0x30]
T = E[+0xe0]
R = T[+0x20]
M = R[+0x10]
H = M[+0x228]
```

The selection is from the active invocation Environment. The callback does
not read `FnCall.this`, and a borrowed native callable must not select a movie
from its receiver instead. A valid bound target/root/movie chain is a
precondition of this bounded profile; null-target handling is not invented.
The result's release happens before the handler lookup, which matters if
release or unsupported aliases introduce effects.

For the **argc-zero** case, the handler-present branch constructs a 16-byte
stack record with these initialized fields:

| Offset | Native write | Value |
| --- | --- | --- |
| `+0` | DWORD | `0x16` |
| `+4` | BYTE | `0` |
| `+5..7` | No callback initialization | Unspecified padding |
| `+8` | DWORD | `0` |
| `+0xc` | DWORD | `0`, the default index |

At `0x1401b7096`, it calls **handler primary virtual `+8`** with
`RCX=H`, `RDX=M`, `R8=&event`. It does not directly write a cursor-visible
flag or invoke a platform cursor API. The event's initialized fields must not
be described as a fully zero-filled 16-byte record. The callback also has
argument-conversion branches; this document's event values are specifically
for argc zero. [REA_MOUSE_HIDE.md](REA_MOUSE_HIDE.md) owns factory admission,
the normal getter, and the callback's other branches.

If `H` is null, the callback tail-calls diagnostic helper `0x140186d10` with
the active Environment and literal at `0x142510c80`:
`No user event handler interface is installed; Mouse.hide failed.`
The diagnostic helper's full behavior is not expanded here, so this branch
must not be called silent or asserted to produce a particular console output.

## Cache identity, ownership and timing

Complete MovieImpl constructor `0x140065080..0x140065830` establishes a null
`+0x228` field by storing its preserved zero register at `0x1400651c7`.
The exact immediate checkpoint is **`0x1400651ce`**. Later constructor calls
exist; that store alone is not proof of constructor-return or current-instance
absence.

The later MovieImpl virtual `+0xc0` routine `0x140062cf0` contains the cache
refresh. At `0x140063b13`, it invokes the secondary state-bag interface's
virtual `+0x20` with ten requested IDs from table `0x1424feda8`:

```text
4, 5, 6, 3, 2, 17, 20, 19, 1, 29
```

The output buffer begins at `RBP-0x40`. The first result, state **4**, is
selected from initially null output slots. It is
loaded at `0x140063b1d`. The old `M+0x228` reference is released through
`0x140031cf0` at `0x140063b26` when nonnull, then the returned reference is
stored at **`M+0x228` at `0x140063b2b`**. State 5 instead goes to `M+0x230`
at `0x140063b47`; state 6 goes to `M+0x238` at `0x140063b6b`.

The cache receiver is the secondary interface at `[M+0x80]+0x10`. The
constructor allocates its `0x58`-byte backing object, installs primary table
`0x1424fe438` and secondary table `0x1424fe410`, and publishes the pointer at
`0x14006569c`. The secondary `+0x20` slot selects complete
`0x140050920` (308 bytes). It searches its state collection under a lock,
matches each requested ID against state `+0x10`, and calls `0x140031ce0`
before writing a found state to the output. Missing entries use the parent
state-bag interface's `+0x20` fallback. Thus the selected result is an owned
reference transferred into the cache after release of the old reference;
the state ID alone does not identify the concrete object or its method table.

The same secondary table's `+0x10` points to complete state setter
`0x140050a60`. For a nonnull state, this forwards the object to
`0x140050770`; actual state identity comes from the object's `+0x10` field,
not merely the caller's EDX argument. A registration call with a numeric
argument alone therefore does not establish its stored category. This is a
concrete selection/mutation interface, not proof that a particular state-4
installation happened. `Mouse.hide()` reads the
cache directly; it does not perform a new state-bag lookup or silently refresh
the cache at the callback site. State publication, cache-refresh timing and
later replacement therefore remain distinct operations.

## What the connected engine setup does establish

The bounded investigation follows the already retained host path
`0x14127fdf0 → 0x141276a60 → 0x141277340`, together with the connected engine
constructor `0x14125dd90` and its loader initialization. It does not perform
an arbitrary whole-executable code scan.

The following explicit registration requests appear in complete selected
initialization bodies; nonnull object identity must also be established from
its constructor, as described above. Hexadecimal support IDs are written
explicitly to avoid confusing them with decimal 23 and 24.

| Complete routine | Relevant direct registration requests |
| --- | --- |
| `0x1400515a0` | Fresh backing state bag starts with null parent/map; direct defaults include IDs 2, 11, 22 and 23. |
| `0x1400319a0` | Loader bootstrap installs IDs 9, 13 and 26. |
| `0x1412706b0` | Engine loader setup installs IDs 9, 10, 2, **0x23** and **0x24**. |
| `0x14125dd90` | Creates supplied bootstrap states and later installs IDs 5, 6 and 19. |

The small helper `0x140052eb0..0x140052f14` is explicitly **state 2**: it
allocates `0x28` bytes, constructs through `0x140052db0`, registers through
the supplied bag's `+0x10` with EDX=2, then releases its local reference.
Complete `0x140052db0` independently stores category 2 at state `+0x10`.
Other recovered default constructors fix categories `0x0b`, `0x16` and
`0x17`. This closes their object identities without treating the setter's
numeric argument as sufficient.

There is one earlier fresh absence checkpoint: in complete `0x1400515a0`,
the new `0x58`-byte backing object's parent at `C+0x20` is cleared at
`0x140051695` and again at `0x1400516b0`; its map at `C+0x28` is cleared at
`0x140051699`. Under ordinary distinct allocation, no reentry and the
standard lock-primitive profile, this is an empty local bag with null parent
**before default setup at `0x140051714`**. It is not constructor-return or
current-MovieImpl absence. Map-growth helpers `0x140050230 → 0x1400500d0`,
optional removal/relocation `0x140050680 → 0x14004e1a0`, and state-`0x17`
member-array resizing `0x140094130 → 0x1403d2f30` remain unexpanded in this
receipt. Later engine initializers and callbacks are additional boundaries.
Consequently no complete fresh-constructor-return state-4-absent profile is
admitted here.

In particular, engine constructor instructions `0x14125e0e6..0x14125e114`
construct state **5** with table `0x1421c32c0` and register ID 5. That table
was briefly considered during navigation and then rejected once the cache
indexing was verified. No Mouse event semantics are attributed to its methods.
Neighboring state 6 uses table `0x1421c3348` and
the separately documented ExternalInterface dispatcher.

The normal setup branch of complete `0x141280590` calls `0x14127df10`,
then `0x141275d30`, with the actual engine and wrapper. The former builds
viewport/dimension data and passes it to MovieImpl virtual `+0x60`; it does
not directly install state 4. Complete `0x141275d30..0x141275e0a` registers
the wrapper in engine lists, calls `0x141275e10` twice, and tail-calls
`0x1412795d0`. These three bodies have no direct state-4 construction or
`M+0x228` assignment. The last routine has MovieImpl virtual calls and
reflected player callbacks; it is **not** a proved transitive no-mutation
barrier. Other setup branches and state overrides retain their conditions.

These are bounded positive bindings and direct-installation observations.
They do not prove that state 4 can never be supplied through a parent bag,
another native caller, a later override or a callback. A current handler's
table and its handling of event `0x16` remain the missing links before a
concrete cursor-effect implementation can be claimed.

A further discovery pass was restricted to the connected GFx engine range
`0x141250000..0x141282000`. No selected immediate store pattern assigning
category 4 to object `+0x10` matched. Its three raw EDX=4 candidates were
then checked with canonical full-function REA: `0x141264003` supplies
allocation size 4; `0x14126fc35` calls `0x140053810`, which forwards through
`[MovieImpl+0x18]` root virtual `+0x188`; `0x14127faed` calls
`0x1400537c0`, which forwards through root virtual `+0x170`. These are not
the identified state-bag setter route. Pattern coverage is intentionally
limited, and no transitive absence claim follows from these exclusions.

## Separate saved hardware-cursor call

Retained owned package metadata and the full saved FrontEnd.Start token tree
identify a different route: after its player-controller and settings guards,
the script invokes
`mpc.SetHardwareMouseCursorVisibility(mpc.m_gameSettings.GetUseHardwareMouse())`.
The function is `MarvelPlayerController.SetHardwareMouseCursorVisibility`,
MarvelGame export 5724; the call's name index is 10765. This is a saved
player-controller call, not proof of a running invocation or a state-4
registration. Its native implementation is not expanded here.

The saved call is useful evidence that host cursor behavior has a separate
entry point. It does not justify equating that entry point with `Mouse.hide`,
changing the latter's selected movie, or substituting an unconditional Rust
window-cursor operation for the unresolved handler method.

## Implementation boundary

**INFERRED faithful boundary:** represent this callback as an event sent to
the active movie's selected handler, with the proven argc-zero fields and
Undefined result ordering. Require a concrete host handler/effect contract
before executing platform cursor operations. Keep known-missing and
unknown-handler state distinct, and preserve the missing-handler diagnostic
branch rather than treating an unknown handler as absent.

For a genuinely fresh state-bag profile, absence must come from a proved
construction/selection checkpoint or an explicit coherent input. The one
constructor zero store, a list of other installed types, or a filename does
not establish a later scene's cached handler. No core VM, frontend, cursor,
renderer or live-state code is changed by this document.

## Private evidence and provenance

Private paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`.

| Evidence | Scope |
| --- | --- |
| `mouse-host/callback/receipt.json` | Reused full callback and constructor: 431 instructions / 2,189 bytes, two complete unwind groups; event padding and immediate zero-store checkpoint. |
| `mouse-host/installation/receipt.json` and `storage-helpers.bundle.json` | State-ID/cache ownership and connected setup: 2,891 instruction spans / 11,145 bytes plus 80 table bytes; six new records, with older session records outside these claims. |
| `mouse-host/effect/complete.bundle.json` | Four new REA records for setup callees, bounded candidate exclusions and their exact root-interface wrappers; older records in the same export are outside these claims. |
| `mouse-host/effect/verification.json` | 1,812 instructions / 6,991 bytes in eight complete unwind groups, matched independently to the owned PE. |
| `mouse-host/state4-candidates/receipt.json` | Bounded discovery constraints and raw candidate addresses; canonical candidate follow-up is recorded above. |
| `rea-mouse-hide/` | Native factory/callback ownership and literal evidence maintained by the Mouse core lane. |
| `frontend-localization-runtime/aggregate-followup/native/final.bundle.json` | Previously retained state-cache selection and independent state-6 handler installation. |
| `frontend-external-receiver/metadata/startup-token-interpretations.json` | Saved player-controller hardware-cursor call, kept separate from native execution. |
| `mouse-host/receipt.json` | Joined document/evidence hashes and explicit unresolved provider/effect boundaries. |

The four new setup/candidate records are
`ev_5b2b57da37c1af42fb718cb84fcfd7d2dbc03a34c008c9ca632a6cdc240cd79e`
and `ev_6d65b826315d2c822a01c58005da247a6f85f82ca00522037310281330c98de6`,
plus `ev_71be0612141ef0c82806a4be2b03b7999ebe98827db5be26e5d0e897b098e0ef`
and `ev_2433ac5f6497e4ab6b913e085b24020a7b347bfa8ca0f8e28d0e68a59fd7a3f5`.
All completed with exit zero, no truncation and unchanged target hash.
Local navigation of the rejected state-5 table and the absence of exact
`UserEventHandler` name bytes in the selected data sections are discovery
notes only, not global type/callgraph absence proofs. Full inline replies and
canonical exports remain private.

Sources: the owner's proprietary Gazillion executable and saved MarvelGame
package metadata (private interoperability study, no redistribution licence);
the original project docs/retained evidence cited above, particularly
[FRONTEND_EXTERNAL_RECEIVER.md](FRONTEND_EXTERNAL_RECEIVER.md),
[FRONTEND_LOCALIZATION_RUNTIME.md](FRONTEND_LOCALIZATION_RUNTIME.md),
[EXTERNAL_INTERFACE_BINDING.md](EXTERNAL_INTERFACE_BINDING.md) and
[REA_DISPLAY_LIFECYCLE.md](REA_DISPLAY_LIFECYCLE.md); installed REA skill and
native-artifact guidance, REA 6.9.0 (MIT tooling), Rizin (LGPL-3.0 tooling),
and Python's standard library (PSF) for original navigation/byte checks. No
outside game or Scaleform implementation was copied. This source ledger is
for integration into [PROVENANCE.md](PROVENANCE.md).
