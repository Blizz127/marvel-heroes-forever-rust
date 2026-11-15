# Frontend localization runtime boundary

**OBSERVED:** The saved login button does not obtain its displayed label by
matching authored placeholder text. Its Construct action supplies
`Frontend/UIButtonLogIn`; manager classes pass that key to
`flash.external.ExternalInterface.call("GetLocalizedObjectDataByLabel", label)`.
The returned object feeds `LocalizedObject`, which writes an HTML-capable text
field. This establishes a concrete script/host boundary for a future runtime.
It does not establish that the Rust client has executed that startup path.

All offsets below are absolute offsets in the corresponding privately saved,
uncompressed GFX movie. `offset+length` denotes a byte span, not an instruction
count. Native addresses are preferred-image virtual addresses in the pinned
1.52.0.1700 Win64 executable. No proprietary script, disassembly, asset or
extracted text is committed with this document.

## Exact login control identity

**OBSERVED (original parser, actual owner files):**

| Link | Saved evidence |
| --- | --- |
| Login placement | `marvel_login_dialog`, timeline 219, frame 0, depth 39, `loginButton_mc`, local character 170; tag 1345 body `39192+497`. |
| Declared import | Login ImportAssets2 tag 970, record offset 26448, body `26454+52`: URL `../MarvelHUD/!UISource.swf`, symbol `Btn_Stndrd_Primary`, local ID 170. |
| Exact source export | `!UISource` ExportAssets tag 10925, body `225229+23`: `Btn_Stndrd_Primary` maps to sprite 1039. |
| Definition | UISource DefineSprite tag 10852, body `224440+772`, sprite 1039, 40 frames. |
| Script association | UISource DoInitAction tag 13301, sprite 1039, actions `353718+76`; call at 353791 registers the exact symbol with `MarvelButton`. Registration alone does not construct an instance. |
| Text child | Sprite 1039's saved frame 0 places `localizedText_tf` at depth 5, EditText character 1034. Its matrix translates by `(100,120)` twips; bounds are `[-40,1920] × [-40,440]`. Three saved filters remain relevant to rendering. |

The login Construct handler is `39236+449`, event flag `0x40000`. Its constant
pool contains the actual key at `39390+22`. It assigns `forceAllCaps=true`,
`label=""`, `localTextLabel="Frontend/UIButtonLogIn"`, and other authored
properties. The `localTextLabel` assignment is at 39585. It does not itself call
the native localization service. The eventual effect of a SetVariable depends
on the instantiated object, inherited accessor and native scheduling.

The manager defines `MarvelButton` in DoInitAction tag 467, sprite 55, exported
as `__Packages.MarvelButton`, actions `150448+8677`. Its constructor is
`152794+37`; the class extends `LocalizedObject` at 152854. `LocalizedObject`
is manager sprite 51, exported as `__Packages.LocalizedObject`, with actions
`90399+4390` and constructor `92017+108`. These identities come from explicit
tags and decoded instructions, not filenames or visible text.

## Script calls and label timing

**OBSERVED static instructions; conditional runtime reachability remains
conditional on class construction and event dispatch:**

| Owner / method | Function body | Relevant behavior / call offsets |
| --- | --- | --- |
| `gfx.core.UIComponent.onLoad` | `8737+18` | Calls `this.onLoadImpl` at 8753. |
| `UIComponent.onLoadImpl` | `8774+557` | Returns when already initialized; otherwise marks initialized at 8878, invokes virtual `configUI` at 8895, dispatches `init` at 8922, and calls `validateNow` at 8940. |
| `MarvelButton.configUI` | `155273+439` | Resolves its text field, configures sizing/constraints, calls `super.configUI` at 155482, installs handlers, then calls `updateAfterStateChange` at 155710. |
| `LocalizedObject.configUI` | `94311+183` | Calls its superclass, invokes `updateLocaleTextFromLabel` at 94457, then registers with `MarvelUIManager.instance.AddLocalizedObject` at 94492. |
| `LocalizedObject.__set__localTextLabel` | `92400+29` | Stores `m_localTextLabel` at 92409 and returns its getter. It does not refresh text. |
| `LocalizedObject.ForceSetLocalTextLabel` | `93076+28` | Stores the key at 93085, then refreshes at 93102. |
| `LocalizedObject.updateLocaleTextFromLabel` | `92910+140` | Calls `MarvelGlobals.GetLocalizedObjectDataByLabel(m_localTextLabel)` at 92934; consumes the result, then calls `updateLabelText` at 93048. |
| `LocalizedObject.getLabelText` | `93985+110` | Prefers a nonempty localized cache, then a nonempty override; otherwise returns null. |
| `LocalizedObject.updateLabelText` | `94114+178` | Requires `localizedText_tf`; sets `html=true` at 94201 and `htmlText` at 94252, optionally calling `toUpperCase` at 94251, then recenters at 94269. The empty path clears `htmlText` at 94291. |
| `MarvelButton.updateAfterStateChange` | `156969+358` | Advances state frames and validates, reapplies the override label at 157170, then retrieves and force-reapplies the localization key at 157188/157201. |
| `MarvelButton.__set__disabled` | `154085+155` | On a changed state, calls the superclass, sets the button state, reapplies override text, then force-reapplies the localization key at 154206/154219. |
| `MarvelButton.getTextField` | `158547+41` | Returns `localizedText_tf` when defined; otherwise returns `textField`. |

`MarvelButton.__set__label` (`153630+373`) is a distinct override path. It
records `m_overrideText`, applies the all-caps flag, gets the field, and chooses
`htmlText` or `text` according to the field's `html` value. Localization has
priority in `LocalizedObject.getLabelText`; a generic replacement of every
authored EditText initial string would bypass that behavior.

`LocalizedObject`'s constructor calls `ClearLocalTextData` without arguments.
That method clears a present field's plain `text`, but clears the localization
key only when its explicit `bClearLocalTextLabel` argument is truthy. This does
not by itself prove when the Construct handler's assignments occur relative to
native construction or `onLoad`.

## Exact external callback contract seen by script

`MarvelGlobals.GetLocalizedObjectDataByLabel` has definition offset 110231,
body `110249+64`, and inherited constant pool 100257. The ExternalInterface
CallMethod instruction is at 110311. Its effective argument order is the literal
callback name followed by the original label value.

The wrapper calls `MarvelGlobals.IsEmptyString` (`108867+44`). That helper
accepts falsy values or values comparing equal to the empty string with AVM1
Equals2. The wrapper returns undefined for that guard; otherwise it forwards
the argument without an explicit ToString. The actual login argument is a
string. This evidence does not justify a universal strict-string native
argument contract or guessed coercion behavior for other value types.

`updateLocaleTextFromLabel` treats a successful response as an object:

| Property | Script consumer |
| --- | --- |
| `localizedPrototypeIdStr` | Passed to `_prototypeData.__set__prototypeId` at 92980. It is a prototype reference, separate from a display-string ID. |
| `localizedText` | Cached verbatim in `m_localTextCache` at 92994. |
| `hasTooltip` | Cached verbatim at 93007. |

An absent result clears the text cache to null and `hasTooltip` to false, then
still runs `updateLabelText`. The script's handling of absence does not prove
which missing-resource cases make the native callback return absence.

The separate `MarvelGlobals.GetLocalizedStringByLabel` helper explicitly
converts a returned `localizedText` to a string. That conversion must not be
silently inserted into the object-returning callback path.

## Native evidence and remaining dispatch gap

**OBSERVED with REA on the owner's pinned executable:** the stored native-name
table associates `UMarvelGameEngineexecGetLocalizedObjectDataByLabel` with
`0x14135e540`, and `UMarvelGFxMoviePlayerexecGetLocalizedObjectDataByLabel` with
`0x14136e490`. The first wrapper calls `0x1413b7010`. The second uses a virtual
call at slot `+0x538`; static tables contain a possible implementation
`0x1414e51c0`, which also calls `0x1413b7010`. The concrete receiver/vtable and
the generic ExternalInterface-to-Unreal-function dispatch were not observed
executing. A matching native function name is not a captured end-to-end call.

The selected engine routine at `0x1413b7010` provides stronger resource evidence
than a guessed key hash:

- It checks configured UI globals and the localized-info directory; an embedded
  diagnostic explicitly names `UIGlobals.LocalizedInfoDirectory`.
- It formats the input label with `%s.prototype` at `0x1413b73e1`, combines it
  with the configured path at `0x1413b7419`, then resolves a prototype reference
  at `0x1413b7443`. Diagnostics name `UILocalizedInfoPrototype`.
- On its successful type-check path, it passes that prototype reference to
  helper `0x1413b9f20` at `0x1413b75ed`. The helper formats with `0x%llX`:
  lowercase `0x` prefix, uppercase hexadecimal digits, no explicit width.
  This is not the display-string identifier or a decimal representation.
- It obtains two resource values from offsets `+0x58` and `+0x70` of the
  resolved object, performs two text-resolution paths, and derives a boolean
  bit from whether the second reference is nonzero (`0x1413b7718` onward).
  The returned native aggregate has three string-like slots and that bit.
  Assigning every aggregate offset to an AS property still requires the native
  struct/ExternalInterface conversion metadata; the script directly establishes
  only the three property names above.

**INFERRED:** the configured directory plus label/suffix path corresponds to
the exact `UiKeyBinding` lookup already documented in [LOCALIZATION.md](LOCALIZATION.md).
The existing resource reader resolves `Frontend/UIButtonLogIn` to display ID
`0x590a017f2861053f`, whose English text is `Log In`. Its current scope does not
include inherited prototype fields, locale formatting, or full native failure
behavior. The saved `forceAllCaps` setting belongs to the script path; it is not
a reason to rewrite the stored locale text.

No retail process, login, account, server or native UI was used. These are
static native and saved-script observations. Generic callback routing,
AS/native value conversion, concrete startup scheduling, child readiness,
prototype inheritance, native missing-resource results, text formatting/token
substitution, HTML layout and rendered parity remain outside this receipt.

## Locale changes and other text mechanisms

Manager `MarvelUIManager.Initialize` (`169101+271`) invokes
`UpdateCurrentLocale` at 169210 and registers `LocaleReset` at 169234.
`handleEvent_LocaleReset` (`179588+134`) updates the locale and calls
`AcquireLocalizedData()` on each non-null registered object at 179704.

No direct definition binding for `AcquireLocalizedData` was found among the
7,738 manager and 584 login function definitions decoded from DoAction and
DoInitAction streams. This is a bounded negative result: dynamic injection,
other movies and ClipAction bodies were not included in that search. It is not
evidence for treating that method as an alias for `updateLocaleTextFromLabel`.

The saved `gfx.utils.Locale.getTranslatedString` (`52086+6`) simply returns
its input. Selected DataBindTextField event/setter methods handle event values
and ordinary field text; no MarvelGlobals localization lookup was observed in
those methods. These separate mechanisms do not establish a global TextField
localization hook.

## Verification and private evidence

Private root:
`~/.local/share/marvel-heroes-forever-rust/evidence/frontend-localization-runtime/`.

- `button-dump.rs`, `manager-button-disassembly.txt` and
  `login-button-disassembly.txt`: original project decoders, exact action/function
  offsets, inherited lexical pools and authored Construct assignments.
- `textfield/function-spans.json`, `findings.json`, `uisource-linkage.json` and
  `receipt.json`: independently decoded method spans and exact import/export,
  registration and text-child identity. Receipt SHA-256:
  `7e10cadf1751289e9cb235d396ed89cfcf7ea4a352e52b056f04b503f372cc61`.
- `native/candidates.json`, `followup-candidates.json` and
  `vtable-reference-candidates.json`: address-discovery byte/PE scans. Instruction
  and slot claims above use the subsequent selected REA reads, not pattern
  matches alone. The proposed RTTI association in the candidate scan did not
  identify a class and is not used as proof of one.
- `native/final.bundle.json`: canonical REA Evidence, with immutable IDs,
  selected commands, input hash, completion and limitation metadata. SHA-256:
  `8eb78e279696ad48bddb2eaac10acb0e8687314ac84c8863fc98ca391d69de8d`.
- `verification.json`: retained file hashes, original parser-source identities,
  native Evidence IDs, selected-output checks and explicit unresolved boundaries.
  SHA-256: `fcf1b221fe1eb316bf1b23743beb566d98b4752d349fc26f0be4fbd4f4144dfe`.

**VERIFIED (focused static checks):** original Rust decoder harnesses compiled
and parsed manager/login/UISource with 2,173 / 1,654 / 13,326 tags and enumerated
7,738 / 584 / 562 nested function definitions. Selected function spans stayed
within the owned input and their hashes were retained. The helper receipt read
back all three inputs and 25 retained files; the combined receipt read back 49
retained files, 28 focused method spans and 113 button/UIComponent function
spans. Four REA commands completed with exit 0,
untruncated output and unchanged executable SHA-256. This does not mean every
callee or all strings were analyzed: Rizin's automatic string scan skipped two
large sections, while the selected explicit reads were retained. Some helper
disassembly selections are partial byte ranges; no full-helper semantic proof
is claimed for them. No VM startup, native callback execution, localization
rendering or retail parity test was run for this investigation.

## Sources and rights

| Source read | Licence / rights | Use |
| --- | --- | --- |
| Owner's `MarvelGame-frontend_manager.gfx`, `marvel_login_dialog.gfx`, and `!uisource.gfx` at the private paths recorded in the receipt | Proprietary retail data; authorized private interoperability study | Exact saved class definitions, methods, key, import/export association and field placement. |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owned 1.52.0.1700 client; authorized private interoperability study | Selected native callback wrappers, resource lookup, reference formatting and static table entries through REA. |
| Original project `src/avm1.rs`, `src/avm1/`, `src/gfx.rs`, `src/gfx/`, saved original decoder helpers, `docs/LOCALIZATION.md`, and prior private PE-scanner helper | Original project code and analysis with their existing provenance | Reused privately for decoding, bounded address discovery, and comparison with the existing resource bridge. No external implementation copied. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and registered REA tool schemas | Toolkit/workflow instructions; toolkit source licence not established by this read | Evidence handling, selected read-only analysis and canonical export. |

No new outside source repository, web page, dependency or client implementation
was read or copied for this investigation. The exact source/hash ledger is
private; the integrator can merge this source register into `PROVENANCE.md`.
