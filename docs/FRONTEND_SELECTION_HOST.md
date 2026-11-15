# Selection listener host

`frontend::selection::SelectionHost` installs explicit `Selection.addListener`
and `removeListener` bindings and maintains a bounded list of actual VM object
identities. It supports the exact one-object argument profile needed by the
original frontend. A different receiver, unsupported argument or exhausted budget
returns an error without changing listener membership.

`addListener` adds an absent object once; registering the same object again leaves
membership unchanged. It returns `Undefined`, reflecting Selection's documented
Void signature. `removeListener` returns true only after removing a present object;
an absent object returns false. This differs from the documented Boolean return
of generic `AsBroadcaster.addListener`, which this host does not implement.

The list records registration order as state. No claim is made about native
callback order, reentrant list changes or notification on registration. No focus
change, native input or callback is manufactured. The caller can inspect the
actual stored identities and compose `handle_call` with other explicit host
bindings. Unknown bindings, class registration, movie properties and timeline
commands remain rejected by the standalone host.

## Evidence and remaining scope

The owned manager's FocusHandler constructor calls
`Selection.addListener(this)` at source offset13552 and discards the result.
The constructor initializes focus lookup arrays after that call. Its
`onSetFocus(oldFocus,newFocus,controllerIdx)` callback subsequently uses the
Scaleform-specific `getControllerFocusGroup(controllerIdx)` method, which is
outside this implementation. Registering the listener does not establish that
the complete initializer, callback or frontend has executed.

Focused tests exercise actual VM native calls, membership, duplicate handling,
Void/Boolean results, invalid arguments/receivers, budget exhaustion and duplicate
global installation. Native-client behavior has not been compared.

## Source ledger

Original implementation; no third-party broadcaster code used.

- Adobe-authored CS5 ActionScript2 language reference, proprietary documentation
  rights, mirrored [Selection reference](https://open-flash.github.io/mirrors/as2-language-reference/Selection.html):
  exact signatures, duplicate registration and removal return semantics.
- Same authorship/rights, [AsBroadcaster reference](https://open-flash.github.io/mirrors/as2-language-reference/AsBroadcaster.html):
  distinguishing its different addListener result and avoiding inappropriate reuse.
- Owner's privately pinned `MarvelGame.upk`/manager movie, proprietary game data:
  the actual call and callback dependency spans. No game data is embedded here.

Primary page snapshots, hashes and extracted contract notes are private under
`evidence/frontend-focus/docs-research/`. These documentation-backed semantics
are distinct from static native REA evidence and future runtime comparison.
