# Stage binding for original frontend startup

**VERIFIED static contract:** the retained native evidence supports a deferred
Stage factory, its separate callable/prototype objects, and actual listener
registration. The existing `frontend::stage_state` supplies geometry calculations
only. It does not establish a global Stage object, loaded viewport, extensions
state or delivered resize event.

**OBSERVED execution boundary:** the latest examined original manager probe
completes 452 initializers and then stops after 154 root-frame instructions at
source offset 17373, whose saved operands describe
`ASSetPropFlags(target, "dispatchQueue", 1)`. It has not reached a Stage call.
The target's exact argument identity was inferred from source flow rather than
dumped by that probe. The requirements below come from saved script bodies and
native implementation evidence, not a successful startup or retail-pixel run.

This document closes the VM/host implementation boundary. No VM, host or
`stage_state` source was changed, and no native game or GPU run was performed.
The broader native arithmetic and event evidence remains in
[STAGE_RUNTIME.md](STAGE_RUNTIME.md).

## Actual saved requirements

The pinned manager contains 94 direct literal Stage lookups: 42 width reads,
35 height reads, five `originalRect` reads, three `visibleRect` reads, seven
`addListener` calls, and one write each to `align` and `scaleMode`. This inventory
does not resolve arbitrary aliases or prove that every containing function runs.
All 94 sites are inside declared functions. Their receiver is obtained by
unqualified `GetVariable("Stage")`, so ordinary scope resolution and possible
shadowing must run before builtin materialization; these are not hard-wired
`_global.Stage` accesses.

The seven listener calls each pass register 1 (`this`) as their sole argument
and discard the result:

| Saved function | Body offset + length | CallMethod offset |
| --- | --- | --- |
| `MarvelUIManager.Initialize` | 169101 + 271 | 169119 |
| `CharacterSelectView` constructor | 922713 + 288 | 922919 |
| `MarvelBuffManager.configUI` | 973805 + 325 | 973895 |
| `MarvelLoginDialog` constructor | 1344465 + 282 | 1344707 |
| `OptionsMenu` constructor | 1518849 + 442 | 1519276 |
| `concar.ConsoleOptionsPanel` constructor | 1845423 + 93 | 1845498 |
| `StagePanel.configUI` | 2053154 + 859 | 2053207 |

Offsets are bytes in the uncompressed original manager GFX, not executable
addresses. The login constructor's call follows its saved modal visibility
writes; registration is therefore a concrete prerequisite for continuing that
constructor once its earlier dependencies execute. It is not authorization to
perform the visibility writes manually.

StagePanel first calls its superclass, reads `Stage.originalRect`, registers its
listener, assigns `Stage.align = "C"`, and assigns `Stage.scaleMode` from the
instance. Its prototype's saved `scaleMode = "noScale"` is a script assignment,
not a native or effective instance default. The original shared UIComponent
scan found no direct Stage use; its method names alone do not require a Stage
substitute.

The root DoAction and its declared `UpdateStageScale` body contain no direct
Stage lookup. Completing all selected initializers does not execute their
declared methods. The retained probe's one observed listener host call is
`Selection.addListener`, not Stage; its minimal diagnostic globals and chosen
initializer-before-frame order establish neither native Stage absence nor
retail startup timing.

`MarvelUIManager.onResize` calls ExternalInterface `CalculateUIScaling`;
`MarvelLoginDialog.onResize` reads Stage dimensions and calls further UI/native
bridges. Registering either listener must not claim those bodies ran. The
bounded literal inventory found no direct `removeListener` or
`broadcastMessage` call, but the native factory still installs those members.

## Deferred factory and object identity

The factory path is wrapper `0x140201c00` → `0x1402014a0`. Global bootstrap
installs a tag-11 name token, `Stage`, with member flags 1. A raw read or copy
retains that token. Resolution uses the calling Env's context, its factory
registry and cache; neither the token nor the script definition filename
selects a context. Successful repeated materialization returns the cached
callable. Publication replaces the original value while preserving flags 1.

This matches the existing deferred-name model documented in
[DEFERRED_BUILTIN_VALUES.md](DEFERRED_BUILTIN_VALUES.md). A first implementation
can require the original global token/flags, the existing explicit current-target
association, one admitted context per VM, and unmodified native metadata. Those
are admission bounds; changed globals, arbitrary namespaces and multiple
contexts must not be flattened into the fresh path.

The fresh graph uses cached parents, even if script-visible global constructor
properties were changed:

| Object | Parent / retained identity | Explicit own members |
| --- | --- | --- |
| Callable, native category 23 | Cached Function prototype (7); retains context MovieImpl at callable `+0x70` | `__proto__`, three listener methods, `_listeners`, four special markers, `showMenu`, `prototype` |
| Stage prototype, category 24 | Cached Object prototype (1); cached under builtin index 18 | `__proto__`, `constructor`, three distinct prototype methods |
| Fresh `_listeners` Array | Cached Array prototype (2) | Empty listener elements initially |

The callable has 11 explicit own keys. Its three methods and `_listeners` have
flags 1; width/height/scaleMode/align markers and Boolean-true `showMenu` have
flags 0; metadata `__proto__`/`prototype` have flags 3. The separate prototype
has five keys: metadata flags 3 and its three methods flags 1. Native metadata
tags are not ordinary mutable data descriptors merely because the VM may use
guarded internal identities to represent them.

The callable's methods are `addListener` (`0x1401a8e10`), `broadcastMessage`
(`0x1401fc0c0`) and `removeListener` (`0x1401a8e90`). The prototype has separate
method objects which all invoke `0x14018d3a0`: they clear the result to Undefined
and do not access a listener collection. Installing the real methods only on
the prototype would be incorrect.

The callable callback at `0x1401894f0` writes null. This does not establish
arbitrary `new Stage` allocation, borrowed construction or mutation semantics;
those paths can remain explicit unsupported operations in the startup subset.
`onResize` and extended rectangle names are not factory-created own data keys.

## Minimal VM and host interface proposal

The following names describe a proposed interface, not existing APIs:

```text
StageMovieBinding(opaque host-owned identity)
BuiltinFactory::Stage { retained_movie: StageMovieBinding }
StageObjects { callable, prototype, listeners, actual_methods, prototype_methods }

StageMemberRequest {
    origin, source_offset,       // script provenance only
    context,                   // active interpreter target / calling Env
    receiver,                  // resolved typed Stage callable
    retained_movie,            // captured factory identity
    member,
}
Host::get_stage_member(request)
Host::set_stage_member(request, converted_value)
```

Use a dedicated typed native kind, factory variant and native-marker dispatch;
an ordinary object filled with width/height numbers loses the getter and owner
semantics. The opaque binding is not a `CodeId`, source movie ID, Scene index,
filename or surface handle. A frontend host may map it to the explicit
`StageState::retained_movie()` identity. The VM should not import frontend
geometry policy into the deferred name token.

Factory graph creation need not choose geometry. It needs a valid explicit
retained movie association; subsequent geometry reads require admitted state.
Host hooks should reject by default. Fresh graph allocation/publication can use
the existing guarded factory rollback policy, clearly distinguished from native
allocation-failure behavior.

Keep three identities distinct: script origin, current calling Env/target, and
the Stage callable's retained movie. Existing `HostCall.context` already models
the active target independently of `this` and `origin`. A direct host invocation
without an Env must not infer one from the Stage receiver. Property-name rules
use the calling Env's version; versions above 6 use exact names. Imported code's
source-header version does not replace the actual execution environment.

| Operation | Owner and required boundary |
| --- | --- |
| `width`, `height` | Read the retained movie's effective rectangle; use existing ordered f32 arithmetic and native signed integer conversion. Return the exact i32 value through the VM's numeric representation; no raw native Value-tag parity is claimed. |
| Width/height assignment | On the unchanged native data profile, raw setter `0x1401e43f0` recognizes the names and succeeds without writing dimensions. Do not turn flags 0 into writable window dimensions. Normal setter/watch/accessor paths outside that profile remain gated. |
| `scaleMode`, `align` reads | Read retained movie state; no surface-derived defaults. |
| `scaleMode`, `align` writes | VM performs admitted Value-to-string conversion in the calling Env; host applies supported retained-state changes and preserves required event effects. `StageState` only admits explicit ASCII without NUL. |
| `showMenu` | Fresh stored Boolean true is proved. This does not prove a downstream menu effect or arbitrary mutation profile. |
| Extended rectangles | Availability uses calling-context extensions state; geometry uses calling target/root movie, independently of retained Stage movie. |
| Listener methods | Operate on the actual method `this` interface and its raw `_listeners` property, not a host list implicitly keyed to global Stage. |

For `originalRect`/`visibleRect`/`safeRect`, an unknown extensions setting is not
false. With explicitly disabled extensions and the unchanged graph, normal
lookup falls through to the base member path. When enabled, the native getter
constructs a fresh Rectangle through the caller's public `flash.geom.Rectangle`
route. A plain host `ObjectFields` result, static SWF rectangle, or retained
StageState rectangle is not an established replacement for that allocation and
ownership path. Keep the enabled extension path unsupported until its caller
geometry and Rectangle construction are supported.

## Listener registration closes the first useful subset

**VERIFIED native conversion chain:** `addListener` converts argument zero with
`0x1401997f0`, passes the actual `this` interface to `0x14019c2f0`, raw-reads
`_listeners`, requires an Array and scans ascending elements. Duplicate tests
compare resolved interface pointers at `0x14019c3b5`; they do not use generic
AVM1 loose equality or script-storage identity.

Before append, `0x14019c3d5` calls interface-to-Value helper `0x1401937f0`.
For the admitted category-2 attached MovieClip interface, that helper follows
interface `+0x18` to the display target and target `+0x70` to its public handle,
then writes Value tag 7 at `0x140193853`. Thus the Rust list must preserve
`Value::DisplayClip`, not `Value::Object(owner)` or lazy storage. Existing live
case-3 handle/owner witnesses remain necessary; this conversion does not create
an attached instance or prove handle retirement behavior.

The complete real callbacks do not require `this` to have Stage's category.
Borrowing a real method onto another interface with an appropriate `_listeners`
Array is a native path, separate from the prototype's no-op methods. A bounded
first implementation may accept only its canonical Stage receiver and genuine
fresh Array, but must reject unsupported borrowing/list mutation rather than
silently operate on the global collection.

For one supplied argument, public `addListener` returns true even if no element
is appended. Its helper returns false for null interfaces or duplicates, but
returns true without append for an absent or unusable Array; the public callback
discards that helper result. Actual startup sites pass `this` and discard the
public result; exposing a helper Boolean would still be the wrong callback
contract. `removeListener` scans backward, removes
one matching resolved interface and returns its helper Boolean. `broadcastMessage`
copies listener Values into a separate Array, visits that snapshot in order,
looks up each named method at visit time, and invokes with the listener as
`this`; later list mutation does not replace the copied iteration set.

All three real callbacks leave the result slot untouched on zero arguments.
That is different from the prototype methods' explicit Undefined store. Until
caller result initialization and full callback invocation are admitted, reject
unsupported zero-argument or broadcast profiles explicitly. Do not report a
successfully delivered resize because registration or array insertion succeeded.

## Setters and resize remain a separate integration boundary

`StageState::set_scale_mode_ascii` reports a `NoScaleResizeRequest` when mode
changes to noScale. Native `0x140201060` first invokes retained movie virtual
`+0x70`, then directly calls `0x1401f69b0` with the calling Env. The latter
broadcasts `onResize`, adding a caller-associated visible Rectangle only when
extensions are enabled. This is a synchronous path after the state update,
not merely a queued request to be dropped by the host.

The underlying viewport setter can separately enqueue a notification depending
on effective geometry and additional mode/descriptor conditions. Its later
callback raw-reads global Stage and explicitly skips unresolved tag 11; queued
notification must not materialize Stage. The direct setter path and queued
viewport path are distinct and can both matter. The pure geometry result is
not a complete event plan, even when its `geometry_changed` field is false.

A first factory/listener implementation can leave these setters gated until
the host/VM preserves their side effects. It must not mutate state and then
return success after discarding a resize request. Likewise, descriptor changes
from a window resize require their own established host mapping and event
ordering; current physical window dimensions are not Stage defaults.

## Validation and remaining work

The intended first increment is lazy factory materialization and canonical
`Stage.addListener(this)` with real typed listener storage. That can advance an
original constructor without fabricating geometry. Before an actual startup
run reaches it, focused implementation checks should cover deferred identity,
fresh graph/flags, stable cache, typed repeated registration and distinct
prototype methods. A later original action probe must retain its actual outcome
and exact registration site. None of those new implementation checks was run
by this docs-only study.

Width/height can subsequently use explicitly associated state. Actual loaded
movie/viewport/extension inputs, enabled Rectangle results, generic borrowed
methods, arbitrary public list mutation, zero-argument result handling, resize
delivery and full native lifecycle remain separately bounded requirements.
No native fallback values, completed startup or wire/pixel parity follows from
this contract.

## Evidence and provenance

Target remains build 1.52.0.1700, private executable SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Pinned manager GFX SHA-256 is
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.

This pass reuses retained REA records; it makes no new provider call. Private
`evidence/stage-vm-binding/inspect_retained.py` independently checks 1,854 unique
instructions in 16 complete selected function ranges against the pinned PE and
retains ten source evidence records. The fresh result is
`retained-verification.json`; frozen Stage receipts were not overwritten.

Core retained evidence:

- `stage-runtime/final2.bundle.json`, SHA-256
  `c048ece78f4c7dd48d3ab585ddf3e69b3538952cf2c79131b9437ab0ea5c1d83`:
  graph/factory `ev_7cb2da192e6c14bfaebba816a50c09f0a5f5ce76b09efddc4e5ce2e7dd8b8b51`,
  member tables/setters `ev_59aaddc801e301458fb7d919d6ec92ce3cde8f54fd5a6eecf72a43484b827414`,
  normal getter/methods `ev_21722c2c6ec451830106449bf7b0638f58457af252e6c3cf9a019f37290f0bcc`,
  raw getter `ev_eac58803730ca5d869e84467922ce856a716fb100acc81c60d57cba726ddd88e`,
  flags/width-write `ev_4a63ba221b6a74689a1da0c0941af2a08a9c1118b5d79a7fdbda2feaa741a672`,
  listener helpers `ev_c3407e75bb90d592635afe940b2cc1b77320489af3ffe679b618b5c75bf9d2d7`.
- `rea-function-calls/function-binding.bundle.json`, record
  `ev_ceb0b6b4241d0b1b4625bb26bb29866a83de58568e4eac93b099ca54166e5092`:
  complete interface-to-Value helper `0x1401937f0..0x140193895`.
- `stage-runtime/script-uses/receipt.json`, SHA-256
  `ebd053895a7b8fd5d0678a6331b1f5286e985ca9b03e40fbe4c734e08f951bd6`:
  original literal-use inventory and source-byte checks.

The private Stage binding receipt indexes the renewed script/runtime-frontier
check, listener conversion check, source files, original artifacts and their
hashes. Raw bytes, decoded instructions and extracted action bodies remain
private and do not enter this repository.

The renewed `stage-startup-usage/receipt.json` has SHA-256
`9f9775ed159f70705317cd745768975f6c604586ed445bf51a13943b24397a90`.
Its read-only verifier passed, including the original operands/header and
retained execution frontier. The independent listener check at
`stage-vm-binding/listener-check/receipt.json` has SHA-256
`d1a3d7dbc91ca1b6bd1e5e0fea2034d4fc010a743bbfc6f94e015e30fec9b7da`;
it checks seven retained records, 1,031 instructions across 14 selected ranges
and 224 method-table bytes. Those counts overlap the parent check and must not
be added as unique coverage. Neither check executes native or VM behavior.

| Source | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned executable and manager GFX; retained REA records above | Proprietary Gazillion/Marvel/Scaleform installation; private interoperability study | Native factory, member routing, typed listener identity and saved startup requirements |
| Original `STAGE_RUNTIME`, deferred builtin, ExternalInterface, attached MovieClip, Function-call and VM documentation/evidence | Original project analysis; underlying artifacts remain private | Reuse established context, handle and callback contracts |
| Original `stage_state`, `deferred`, `vm`, `string`, `external_interface` and attached frontend sources | Original project code | Identify the missing VM/host boundary and propose additive interfaces |
| Retained REA 6.9.0 Rizin output | REA MIT; Rizin LGPL-3.0-only tooling, not linked into client | Reuse canonical observations and compare retained instructions with original PE bytes |

No external VM/SDK implementation, web source, platform default or substitute
UI was used.
