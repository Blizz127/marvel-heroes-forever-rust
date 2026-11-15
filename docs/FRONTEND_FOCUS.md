# Frontend focus-listener contract

**OBSERVED:** The original manager's `FocusHandler` constructor calls
`Selection.addListener(this)` at movie byte offset 13552 and immediately pops
the result. The required initial host operation is registration of that exact
object. The saved code does not inspect a success value. Later focus handling
expects GFx controller extensions beyond Adobe's standard Selection interface.

This investigation decodes saved actions and studies primary documentation.
It adds no VM or host implementation. **No native focus behavior, event order,
or return-stack value was verified**, and no native client was launched.

## Documented Selection and broadcaster behavior

**OBSERVED documentation:** Adobe's Selection reference specifies the following
contract. These are published Flash semantics, not measured retail GFx behavior.

| Interface | Documented behavior |
| --- | --- |
| `Selection.addListener(listener:Object):Void` | Registers an object for focus-change notifications. Multiple listener objects are supported. Registering an existing listener leaves registration unchanged. |
| `Selection.removeListener(listener:Object):Boolean` | Returns true when removal succeeds and false when it fails, including absence. A removed object stops receiving notifications. |
| `Selection.onSetFocus(oldfocus:Object, newfocus:Object)` | Notifies listeners of a focus change; the two focus parameters are described as optional. |

Source: [Adobe Selection reference](https://open-flash.github.io/mirrors/as2-language-reference/Selection.html).
A host's mapping of declared `Void` to AVM1 `Undefined` is an implementation
interpretation; the owned constructor's `Pop` does not establish that value.

**OBSERVED documentation:** Generic AsBroadcaster has a distinct registration
signature. `initialize(object)` installs `addListener`, `removeListener`,
`broadcastMessage`, and the internal `_listeners` array. Its
`addListener(listener:Object):Boolean` is documented to return true;
`removeListener` returns actual removal success. `broadcastMessage` has a
`Void` signature, addresses each listener's named event method, and passes the
remaining arguments. The reference labels `_listeners` read-only and intended
for internal use. Its addListener entry does not specify duplicate handling;
Selection's explicit no-change rule cannot establish that generic behavior.
Source: [Adobe AsBroadcaster reference](https://open-flash.github.io/mirrors/as2-language-reference/AsBroadcaster.html).

Macromedia's *Learning ActionScript 2.0 in Flash*, printed pages 332–335,
corroborates the object-registration and event-method model, including a
Selection example. It supplies no additional ordering or mutation guarantee.

## Actual saved constructor and initialization

The private movie is `MarvelFrontEnd.frontend_manager`, zero-based SwfMovie
export 17004 in `MarvelGame.upk`. The relevant root-frame DoInitAction is tag
383, sprite ID 34, linkage `__Packages.gfx.managers.FocusHandler`. Its action
span starts at byte 12444 and has length 3955. All offsets here are absolute
uncompressed movie offsets, not UPK offsets.

**OBSERVED:** The constructor's DefineFunction2 starts at 13523; its body is
13534 + 135 bytes. Flags `0x0169` preload `this`, `_root`, and `_global` into
registers 1, 2, and 3, respectively, under the SWF register-order contract.
The constructor pushes register 1 and argument count 1, resolves global
`Selection`, then invokes its `addListener` member at 13552. Offset 13553 is
`Pop`. No immediate callback is requested by these actions.

After registration, the saved constructor performs these writes in order:

| Saved receiver/member | Value | SetMember offset |
| --- | --- | ---: |
| `_global.gfxExtensions` | 1 | 13566 |
| `Selection.alwaysEnableArrowKeys` | true | 13580 |
| `Selection.disableFocusKeys` | true | 13594 |
| `Selection.disableFocusAutoRelease` | true | 13608 |
| `Selection.disableFocusRolloverEvent` | true | 13622 |
| `_root._focusrect` | false | 13632 |
| `this.currentFocusLookup` | empty array | 13650 |
| `this.actualFocusLookup` | empty array | 13668 |

The GFx property names and values are saved evidence; their native effects are
unverified. Both lookup arrays are created after registration. This sequencing
does not itself prove whether native registration can dispatch synchronously.

**OBSERVED:** `initialize`, defined at 13818 with body 13829 + 65 bytes, marks
the object initialized, retrieves `gfx.managers.InputDelegate.instance`, and
calls its `addEventListener` at 13892 with three arguments: event name
`input`, this listener object, and method name `handleInput`. Its result is
also popped. That saved event-dispatch interface must not be substituted with
generic AsBroadcaster merely because both register listeners.

The class initialization later retrieves its singleton instance. No
`Selection.removeListener` call or generic AsBroadcaster invocation occurs in
this bounded class/init span; this is not a claim about other manager classes
or the rest of the game.

## Required callback shape and native extension gaps

**OBSERVED:** Saved `onSetFocus` is defined at 15670, body 15716 + 228 bytes.
Its formal arguments are `oldFocus`, `newFocus`, and `controllerIdx`, assigned
to registers 7, 3, and 8. Register 1 preloads the listener's `this` object.
The third parameter is used without a saved default before the native call.

The callback can return early when the old focus is a TextField and the new
focus is null. Otherwise it calls
`Selection.getControllerFocusGroup(controllerIdx)` at 15769, accesses the
listener's lookup arrays using the returned group, and conditionally handles
component focus bits. It stores the new object in `actualFocusLookup[group]`
at 15925 and calls `this.setFocus(newFocus, group)` at 15942. Callback receiver
binding must preserve the registered listener object for these accesses.

The saved `setFocus` method also calls
`Selection.getControllerMaskByFocusGroup(group)` at 14238 and
`Selection.setFocus(focusObject, controllerIndex)` at 14355 while iterating
the controller mask and `System.capabilities.numControllers`. Other input
paths call `Selection.findFocus` and controller-index variants of caret/focus
queries. These are observed required symbols and argument shapes, not
implemented or verified host semantics.

Adobe's standard two-parameter notification does not establish the source,
range, default, or grouping of the third controller parameter. Until that
extension has evidence, registration alone must not be treated as working
focus dispatch. No controller-zero substitution or synthetic initial event
is justified by this investigation.

## Explicitly unresolved cases

The inspected primary documents do not settle callback order; additions or
removals during dispatch; snapshot versus live iteration; self-removal;
removing a listener not yet called; nested dispatch; callback exceptions;
missing/noncallable event methods; primitive/null listeners; or repeated
generic broadcaster initialization. Generic duplicate handling remains
unspecified, as noted above.

Registration is described in terms of later focus changes, but the absence of
an immediate-callback promise is not native evidence of silence. Same-target
`setFocus`, the timing of `getFocus` updates, and order relative to target
`onKillFocus`/`onSetFocus` callbacks also remain unresolved. The exact native
AVM1 stack result for declared `Void` has not been measured.

## Focused validation and private evidence

**VERIFIED:** The owned movie is 2,055,748 bytes, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Its source package is 5,224,388 bytes, SHA-256
`d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6`.
The original bounded Rust decoder completed the selected action span; five
recorded body/action spans were checked against their file bounds and hashes.
The private 753-line decode retains the exact instructions, without executing
them. This checks saved-byte interpretation, not runtime compatibility.

Evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `frontend-startup/MarvelGame-frontend_manager.gfx`: original private movie.
- `frontend-focus/inspect-focus.rs` and `focus-handler-private.txt`: original
  bounded inspection utility and complete selected function decode.
- `frontend-focus/owned-call-evidence.json`: exact input and decoder identities,
  constructor/callback call sites, five source spans and their hashes.
- `frontend-focus/validation.json`: repeated identity/span checks and scope.
- `frontend-focus/docs-research/source-downloads.json` and
  `listener-contract.json`: primary-document receipts, contracts and gaps.

The decoder snapshot is the project's original
`evidence/avm1-runtime/decoder.rs`, SHA-256
`a2822eb755e55f1daf101ffc96c1bccece5a725d90c140bbaad60b8fe25dc128`.
No outside VM implementation was inspected or copied. Original game bytes and
decoded action bodies remain private; only this evidence summary enters git.

## Source ledger

1. Adobe, *Flash CS5 ActionScript 2.0 Language Reference*,
   [Selection](https://open-flash.github.io/mirrors/as2-language-reference/Selection.html),
   page date 2010-04-07. Studied overview, add/remove listener, onSetFocus,
   getFocus and setFocus. © 2004–2010 Adobe, all rights reserved; documentation
   study only. Private snapshot 36,959 bytes, SHA-256
   `37c745e4d2295672e87d0085340f93ce259ea0837d4b39d7e620d25db167c54a`.
2. Adobe, same reference,
   [AsBroadcaster](https://open-flash.github.io/mirrors/as2-language-reference/AsBroadcaster.html),
   same date and rights notice. Studied initialize, registration/removal,
   `_listeners` and broadcastMessage; no example implementation reused.
   Private snapshot 24,874 bytes, SHA-256
   `7e0860bb878d1d936df249164e3cac96be3eb435c20770fd98363aa7200fd1eb`.
3. Macromedia, [*Learning ActionScript 2.0 in Flash*](https://open-flash.github.io/mirrors/learning-as2.pdf),
   first edition September 2005, printed pages 23 and 332–335 plus copyright
   and credits. © 2005 Macromedia, all rights reserved; listener-model study
   only. Private snapshot 6,818,789 bytes, SHA-256
   `e6d8087705bc5a63c7510ab4831ecda78dceab227e84112be42c2327f6cdc612`.
4. Adobe, [*SWF File Format Specification*, version 19](https://open-flash.github.io/mirrors/swf-spec-19.pdf),
   ActionDefineFunction2 register/flag contract, printed pages 111–114.
   © 2006–2012 Adobe, all rights reserved; format study only. Existing private
   snapshot `gfx-fonts/adobe-swf-spec-19.pdf`, 1,724,258 bytes, SHA-256
   `6a1625b0929a86329021d365ba079c7e1f51502dfb4ea7d92ec9947841625cdd`.
5. Owner's privately pinned `MarvelGame.upk` and extracted manager movie,
   identities above; proprietary retail assets, interoperability study only.
   Original project decoder and earlier private startup evidence were used
   solely to locate and decode the selected saved class.

All web documents were read 2026-11-15. These are original Adobe/Macromedia
publications preserved by a mirror. The attempted retired Adobe Selection and
AsBroadcaster URLs could not be fetched; no content from those attempts was
used. No source-code licence is inferred from documentation access.
