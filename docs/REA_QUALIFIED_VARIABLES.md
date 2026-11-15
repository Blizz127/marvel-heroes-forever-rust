# Native qualified-variable lookup in the manager registrations

## Result and scope

**VERIFIED — static target instructions:** the two final registration lookups
`gfx.controls.ListItemRenderer` and `gfx.controls.ScrollingList` use ordinary
first-segment variable resolution followed by member reads on the returned
objects. The final stored Function value is copied, including its auxiliary
frame reference; this path does not perform the separate `Function(value)`
conversion.

This supports a deliberately small implementation profile: nonempty ASCII
identifier segments separated by single dots, an ordinary first-segment scope
lookup, existing ordinary data properties on the namespace objects, and the
loaded target's case-sensitive version context. It does not justify treating
every qualified name as a direct global lookup or accepting the full movie-path
grammar.

**OBSERVED — Rust interpreter probe:** after the earlier native hash-order
increment, 447 of the manager's 452 initializer entries completed before the
first qualified lookup was rejected. That is progress in the reconstruction,
not an observation of retail execution. The retail executable was not run.

The current movie target remains significant. Native first-segment lookup tries
that target before the global object. For an attached MovieClip, absence of an
own property alone is insufficient to prove a miss: named display children and
the prototype can also provide a member. The normal scope resolver and its
unsupported-state checks must remain in use.

## Original bytecode and target identity

The owner's private Win64 executable is build `1.52.0.1700`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The private manager GFX has version 15 and SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Both are pinned outside the repository. Version and ownership receipts are
listed in [the native study](REA_NATIVE_ALTERNATIVE.md).

| Initializer | Literal Push | GetVariable | Exact operand | Following use |
| --- | --- | --- | --- | --- |
| `2055321 + 91` | `2055321` | `2055355` | `gfx.controls.ListItemRenderer` | `Object.registerClass` at `2055409` |
| `2055649 + 85` | `2055649` | `2055680` | `gfx.controls.ScrollingList` | `Object.registerClass` at `2055731` |

The original Push opcode is `0x96`, contains a NUL-terminated string value, and
is followed by GetVariable `0x1c`. These exact bytes and offsets are checked in
private `evidence/rea-qualified-variable/actual-source-receipt.json`. No asset
payload was added to this repository. Both resolved values are arguments to
registration; they are not called at the lookup site.

The private namespace capture is a behavior-preserving inspection of the Rust
VM at the first rejection. It finds target/root `ObjectId(1)` as a Plain object
without a parent or `gfx` member, and global `ObjectId(0)` owning data member
`gfx -> ObjectId(25)`. That Plain object owns `controls -> ObjectId(173)`, also
Plain. Its own data members are `ListItemRenderer -> Script(472)` and
`ScrollingList -> Script(266)`, with definitions at `54730` and `33788`.
All four namespace descriptors have flags zero; both functions retain captured
scopes `[target, global]`. This confirms the selected controlled profile while
leaving the attached-retail-root limitation above intact.

## Entry and first-segment precedence

**VERIFIED:** the raw `0x1c` dispatch enters `0x140209796`, then
`0x140202930` and `0x1402027d0`. The operand's existing value is released and
cleared to Undefined, and that same Value slot is supplied as the output.
The router recognizes `.`, `:`, and `/` as reasons to use the path resolver
`0x1401fb6a0`; cached StringNode flags can avoid rescanning.

The router supplies a packed argument block containing the name, final output,
scope vector, optional target output, temporary owner output, and flags. For
this GetVariable call the resolver's filter and separate segment-output
arguments are zero. The temporary owner output is distinct from the final
value output; copying it is not a callable conversion.

At `0x1401fbb9b..0x1401fbbca`, the first ordinary token is passed to
`0x1401faf10` with the original scope vector. It is not read directly from the
global object. The selected top-level profile has no With entries or activation
locals. In this profile the resolver tries the current target before the
global object. `0x14018f070` returns no activation entry when the environment's
activation count at `+0x188` is zero.

The current-target call is at `0x1401fb546`, using virtual member slot `+0x58`.
On a miss the global object is read from `[[Env + 0xe8] + 0x60]` and queried
through its interface slot `+0x20` at `0x1401fb60b`. The global interface
`0x142512aa0` maps that slot to `0x1401e8670`, which adjusts the context to
`Env + 0xe8` and tail-calls slot `+0x58`. Its global-specific getter
`0x1401db080` recognizes `gfxExtensions`; ordinary `gfx` takes the tail call
to generic `0x140197000`.

A null native current-target pointer fails immediately, without the global
fallback. The selected profile therefore also requires a current target.

The parser's first-token special comparisons at builtin offsets `+0x2c8` and
`+0x340` are `..` and `_parent`. Neither matches `gfx`. The ordinary first
segment still retains the normal resolver's reserved-name and scope behavior;
this study does not replace those checks with a new grammar.

For an attached current target, interface `0x1425175c0`, slot `+0x58`, enters
`0x140203840`, which passes the request to `0x1401ddcd0`. The separate attached
member study establishes its ordinary missing-name search through storage,
named child, and prototype. An implementation must establish that these
sources miss before taking the native global fallback. A synthetic Plain
target with no own `gfx` is a narrower test fixture, not evidence that the
retail root has no shadowing member.

## Dot tokenization and namespace member reads

**VERIFIED:** for a name beginning with ordinary `g`, the resolver chooses the
NUL-terminated delimiter string `":./"` at `0x14251699c`.
At `0x1401fb850` it passes the current byte and that delimiter set to
`0x141c6f4ec`; the executable's import thunk binds this to
`MSVCR120.dll!strchr`. A nonempty token is interned through `0x1400972e0`
without changing its byte sequence. The original two names therefore produce
exactly `gfx`, `controls`, and their respective class name.

After a successful segment, the dot branch at `0x1401fbdcf` retains the current
Value and repeats tokenization. It does not restart scope resolution. Later
segments take `0x1401fbb0b..0x1401fbc37`: for an ordinary Object Value, tag 6,
`0x140199640` returns the underlying object pointer, and the parser calls that
object's interface slot `+0x20` with the next interned name and a temporary
result Value.

The native ordinary object interface `0x1425185c0` maps `+0x20` to the same
`0x1401e8670` context thunk and `+0x58` to generic getter `0x140197000`.
For version contexts above 6, the getter searches the own hash by the interned
name's hash and exact interned pointer. On an existing ordinary own property,
`0x1401971b3..0x1401971bb` copies its raw Value with `0x14018d9d0` and reports
success. This selected branch neither consults a parent nor invokes an
accessor. The full getter also has parent, reserved-property, and resolver
branches; they are not evidence that an unimplemented namespace shape can be
silently accepted.

The parser handles a returned accessor tag separately at `0x1401fbd0d`.
Ordinary data instead reaches `0x1401fbd1c`, copying the segment result into
the current Value. The implementation profile above requires data properties
so that this separate accessor route is not approximated.

## Final Function value

On successful completion, `0x1401fbfad..0x1401fbfba` copies the current Value
to the packed final-output pointer with `0x14018d9d0`. The router's owner-output
copy is separate. Wrapper `0x140202930` does extra handling for accessor and
resolver tags, while the ordinary Function tag 8 passes through unchanged.

The value-assignment jump-table entry for tag 8 is the RVA `0x18db76` at
`0x14018dbfc`. That branch copies the source function-object pointer at `+8`
and retains it. If present, the auxiliary frame at `+0x10` is passed to
`0x1401883c0` along with its ownership bit. That helper stores the same frame
pointer and adjusts reference ownership. It does not substitute a fresh
function object or clear a non-null auxiliary frame.

This is the ordinary FunctionRef copy contract. The distinct object-to-function
conversion branch and `Function(value)`'s interface `+0x60` behavior are not
used to obtain these already-tagged Function values. See the separate native
Function conversion study for its narrower conversion contract.

## Evidence and validation

All paths below are relative to the private
`~/.local/share/marvel-heroes-forever-rust/` directory:

- `evidence/rea-qualified-variable/actual-source-receipt.json`: original Push
  bytes, offsets, names, and movie digest.
- `evidence/rea-qualified-variable/final.bundle.json`: full path resolver,
  delimiter/import binding, ordinary-object interface slots, and FunctionRef
  copy-table/auxiliary-copy evidence.
- `evidence/rea-qualified-variable/combined-proof-receipt.json`: joined
  record IDs, artifact identity, byte-mapping checks, and explicit limits.
- `evidence/rea-qualified-variable-entry/final.bundle.json`: opcode entry,
  router, first-segment scope resolution, reserved names, and global fallback.
- `evidence/avm1-qualified-variable/namespace-profile-receipt.json`: frozen Rust
  probe source hashes, exact target/global frame, own namespace descriptors,
  and original class-definition identities at the 447-initializer boundary.
- `evidence/rea-script-allocation/final.bundle.json`, record
  `ev_6bffb64cda175ce7843116d3f64216fe28f36f08f5a7ae861ad5c248c225011c`:
  reused complete generic getter `0x140197000`.
- `evidence/rea-asflags-followup-return/final.bundle.json`, record
  `ev_41547e36654d7df1ab3b7edf950a81e8f63d9fdc9cb633f9b0723c760ab7ee88`:
  reused ordinary Value assignment `0x14018d9d0`.
- `evidence/rea-function-calls/apply-conversion.bundle.json`, record
  `ev_aa26bdd087418e61873edce4f4459c8f5c5037b68e30e7c45fcf7c3ad28faf25`:
  reused tag-6 object extraction in `0x140199640`.

Target-free REA Rizin calls used bounded selected ranges. The raw `.text`
mapping was independently checked against PE section offsets; mixed
data/import queries used PE loading. Complete instruction/data claims are
restricted to their actual decoded boundaries. Byte validation, source-byte
checks, and `git diff --check` were run. The six new REA records completed
without truncation or target change; 1,867 instruction spans and 298 data bytes
matched the pinned PE independently. The `strchr` import was additionally bound
through the PE import descriptor and IAT. No retail process, server, network
protocol, login, rendering, or gameplay check was performed for this study.

## Provenance

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe` and `MarvelGame-frontend_manager.gfx` | Gazillion/Marvel proprietary; owner's private installation | Exact variable resolver, original caller operands, and member/value-copy study only; no target code or payload copied into the repository. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/connection-and-recovery.md`, `references/native-and-artifacts.md`; REA 6.9.0 | MIT toolkit; local usage guides | Required bounded native workflow and Evidence preservation. |
| Rizin 0.9.1 through REA | LGPL-3.0-only | Selected native instruction/data and PE import decoding. |
| Existing project native receipts and private interpreter probes listed above | Original project analysis; probes execute the Rust reconstruction | Reused getter/value-copy evidence and identification of the exact unsupported operation; no retail-runtime claim. |

No SDK source, third-party VM implementation, or reconstructed game source was
used to choose these semantics.
