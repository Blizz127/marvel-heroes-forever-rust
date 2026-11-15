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
| `localizedTooltipText` | A fourth field declared by the native return struct; separately consumed by `MarvelPanel.GetLocalizedTooltipLabel` and a console-gated path in `DifficultyNode.SetData`. The manager label-refresh method does not read it. |
| `hasTooltip` | Cached verbatim at 93007. |

An absent result clears the text cache to null and `hasTooltip` to false, then
still runs `updateLabelText`. The script's handling of absence does not prove
which missing-resource cases make the native callback return absence.

The separate `MarvelGlobals.GetLocalizedStringByLabel` helper explicitly
converts a returned `localizedText` to a string. That conversion must not be
silently inserted into the object-returning callback path.

## Native dispatch and aggregate conversion

**OBSERVED with REA on the owner's pinned executable:** the stored native-name
table associates `UMarvelGameEngineexecGetLocalizedObjectDataByLabel` with
`0x14135e540`, and `UMarvelGFxMoviePlayerexecGetLocalizedObjectDataByLabel` with
`0x14136e490`. The first wrapper calls `0x1413b7010`. The second uses a virtual
call at slot `+0x538`; static tables contain implementation `0x1414e51c0`, which
also calls `0x1413b7010`. The concrete live receiver/vtable was not captured.

**OBSERVED (owned package metadata):** `MarvelGame.upk` contains ScriptStruct
`MarvelGameEngine.LocalizedObjectData`, export index 5709 / object reference
5710, logical byte span `2125413+169`. Both lookup UFunctions declare a
`localTextLabel` StrProperty argument and a ReturnValue StructProperty referring
to this same struct. Its fields are `localizedPrototypeIdStr`, `localizedText`,
`localizedTooltipText` (StrProperty), and `hasTooltip` (BoolProperty). A complete
tagged default list at logical `2125465+117` declares three empty strings and
false. Those are serialized defaults, not proof of native failure-path values.

The AS2 native method table at `0x14250e210` pairs the literal `call` with
`0x1401c55d0`; its constructor installs that table at `0x1401f2265`. The call
routine reads the movie's handler at `+0x238` and invokes handler vtable slot
`+0x08` at `0x1401c57aa`. The engine's state-installation path at
`0x14125e117..0x14125e16b` creates state type 6 with vtable `0x1421c3348`, whose
`+0x08` entry is `0x141264c20`. A selected movie update path requests ten states
using the list at `0x1424feda8` (`4,5,6,3,2,17,20,19,1,29`); it stores the
third returned state in movie `+0x238` at `0x140063b6b`. This closes the static
connection between the installed type-6 state and the AS2 call routine's cached
handler. A live movie's selected handler/receiver has not been captured.

**OBSERVED with REA:** generic dispatch routine `0x141264c20` resolves the
callback name on a selected engine-side receiver, zeroes the reflected function's
parameter buffer, converts supplied arguments at `0x141264e70`, invokes the
function at `0x141264ef2`, and locates its ReturnValue property at `0x141264efb`.
It calls the output converter `0x141267200` at `0x141264f2e` and supplies a
non-undefined result to the movie at `0x141264f41`. Null receivers, rejected
receiver state, absent names/functions and names beginning with `_` take paths
that do not set a result. These are native control-flow observations, not a
captured invocation. The AS2 call routine resets its result to undefined before
calling the handler, then reads the movie's stored result afterward.

The selected engine dispatcher gets its receiver from movie user data at
`+0x124`. The two complete package tables include the GFxUI ExternalInterface
field and its setter. A scan of all 3,563 MarvelGame and 172 GFxUI Function
exports found no direct initialization caller: the sole relevant code reference
is the setter itself. This bounds the negative result to stored references and
literal names in these two packages. Native initialization, other packages and
indirect calls remain possible; there is no invented default receiver.

The selected input converter at `0x1412668b0` has explicit narrow-string and
wide-string branches for string properties (`0x141266c85..0x141266d4d`). It is
not evidence for applying general AS ToString to every incoming value. The
native argument loop consumes ordinary parameters in reflected order, stops
at the end of those parameters, and leaves missing arguments in the zeroed
parameter buffer. The saved MarvelGlobals wrapper supplies exactly one string
argument after the callback name.

The output converter's BoolProperty, StrProperty and StructProperty class
identities were checked through their native class builders and corresponding
UTF-16 class-name literals:

- The StructProperty branch creates an object at `0x1412677ad`, walks the
  reflected struct's child properties, reads each value using that property's
  runtime offset, and recursively converts it at `0x14126785c`.
- It obtains each child's actual reflected name at `0x14126786e`, converts the
  name, then supplies name/value to the object-interface operation at
  `0x1412678e7`, vtable slot `+0x28`. Its member-write role is inferred from this
  argument/data flow; no internal AS2 object-interface slot names are borrowed.
- StrProperty output uses a wide-string value (native type tag 7). BoolProperty
  output tests its declared bitmask and produces a boolean (type tag 2).
- The metadata does not serialize each runtime UProperty offset. The physical
  native aggregate's three 16-byte string slots followed by a boolean match
  the declared field order; that offset/name association remains a structural
  inference rather than a captured linked UProperty table.

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
  bit from whether the second reference is nonzero (`0x1413b7718` onward),
  independently of whether the resolved tooltip text is empty.
  The returned native aggregate has three string-like slots and that bit.
  The generic converter and declared struct explain the four AS member names;
  their exact native runtime offsets retain the qualification above.

**INFERRED:** the configured directory plus label/suffix path corresponds to
the exact `UiKeyBinding` lookup already documented in [LOCALIZATION.md](LOCALIZATION.md).
The existing resource reader resolves `Frontend/UIButtonLogIn` to display ID
`0x590a017f2861053f`, whose English text is `Log In`. Its current scope does not
include inherited prototype fields, marker formatting, locale fallback, or full
native failure behavior. The saved `forceAllCaps` setting belongs to the script
path; it is not a reason to rewrite the stored locale text.

## Bounded native resource and plain-string profile

**OBSERVED with REA:** the generated class registration at `0x1416e6280`
installs four descriptors from `0x1429fa160`, with stride `0x98`. The record
named `DisplayText` contains the offset/width pair `0x58,8`; `TooltipText`
contains `0x70,8`. The other two records associate TooltipFont and TooltipStyle
with `0x60` and `0x68`. Registration uses the same type descriptor as the
selected UILocalizedInfoPrototype type check. The offset interpretation is
supported by the independent callback loads; a generic descriptor-offset
consumer was not reconstructed; the table and callback provide independent
observations of the same offsets.

The callback obtains its prototype through `0x14158e290`. That routine returns
an existing entry's object at `+0x30`, or enters a lazy load path and caches the
result. The ordinary loader `0x141605200` creates an object, then invokes the
selected handler's virtual slot zero with that object, its entry reference and
input stream. Its alternate stream path also converges on this loader.
`0x14160ebf0` requests a cleared allocation; `0x14064d0b0` calls the imported
`memset` with zero and the pool's block size before the constructor callback.
The selected derived constructor explicitly clears font/style fields, without
writing either text-reference slot. These reads establish allocation clearing,
not effective text defaults after the serializer. Parent application, terminal
missing-field treatment and postprocessing still cross an unresolved boundary.
The existing explicit-parent resource view in
[PROTOTYPE_VALUES.md](PROTOTYPE_VALUES.md) has its own source and data evidence;
this investigation does not upgrade that view to native-runtime parity.

**OBSERVED with REA:** both text references enter thunk `0x1418b1130`, which
forces selector `UINT64_MAX` before calling `0x1418c4840`. The selected catalog
loader initializes the base node's consumed mask to `UINT64_MAX` at
`0x1418a6c29`. Its alternate-form loop reads an eight-byte consumed mask,
two-byte produced flags and four-byte text offset, then creates separate linked
nodes through `0x1418a80a0`; it preserves the base mask. The selector compares
against that base mask before traversing alternates. Consequently this callback
uses the stored **default text** for records created by that observed loading
path, including records that have alternate forms. This is not a general
variant-selection rule for other callers.

The active catalog comes from the global resource context, not a locale
argument to this callback. Lookup `0x1418bb160` also contains mode-dependent
fallback for absent entries and for text containing the exact marker
`!!Missing translation:`. Its missing-entry path can construct an invalid-ID
marker. The numeric mode values, their correspondence to English or other
languages, live selection and complete fallback order remain unverified.
Choosing an arbitrary first locale or returning an empty string for a missing
ID is not supported by this evidence.

After selection, `0x14187e820` scans for `#`. If none occurs, its marker
collection stays empty and it skips the text writeback, preserving the selected
bytes. Marker branches, including nested/reference-like forms, require further
work. The following bridge calls the imported `MultiByteToWideChar` with code
page `65001`, flags `8`, and a NUL-terminated input length. No malformed input was
executed; a strict profile can require valid UTF-8 instead of guessing its
failure result.

A future pure resource resolver can therefore expose this bounded contract:

- Take an exact resolved prototype reference, explicit effective DisplayText and
  TooltipText references with their origins, and an explicitly selected catalog.
  Unresolved fields or inheritance must remain explicit.
- Require present default entries with valid UTF-8, no `#`, and no missing
  translation marker. A stored zero-ID entry is distinct from an absent ID.
- Format the prototype reference with `0x%llX`, preserve the selected display
  and tooltip text, and compute `hasTooltip` from `TooltipText != 0`.
- Return typed unsupported/missing-resource outcomes outside that scope. Do not
  uppercase text, substitute authored placeholders or infer script state here.

This contract prepares native-shaped resource data. It does not install
ExternalInterface, construct AS objects, execute the saved label setters or
prove that retail startup reached the callback.

## Failure paths and limits of the return contract

The script wrapper's empty/falsy guard returns undefined before entering the
host. It is separate from an invalid nonempty resource key. Native dispatch
initializes its parameter buffer, but that does not initialize every temporary
inside the engine's lookup implementation.

**OBSERVED:** the complete selected `0x1413b7010` routine has early branches for
missing UI globals (`0x1413b712c`), an invalid localized-info directory
(`0x1413b71fa`), an unresolved prototype reference (`0x1413b744f`), and a missing
or incompatible prototype object (`0x1413b75c8` / `0x1413b75dc`). They pass
diagnostics to shared handlers and, if those handlers return, copy the temporary
aggregate into the result. The three temporary strings are empty-initialized.

The temporary boolean source at stack `+0x70` has no explicit initializing write
before those early branches in the selected routine. The success path writes
it at `0x1413b772d`; earlier failure paths read its low bit at `0x1413b719e`,
`0x1413b726c`, `0x1413b74e1` or `0x1413b78ce`. The shared diagnostic handler
contains runtime-configured callbacks; no failure was executed. Consequently,
neither a guaranteed absent result nor `hasTooltip=false` is established for an
invalid native resource lookup. A Rust implementation should keep this outside
a verified callback profile rather than silently substitute the package's false
default or fabricate a tooltip.

No retail process, login, account, server or native UI was used. Actual startup,
live native receiver selection, effective inherited prototype fields, marker
formatting, locale fallback, diagnostic-handler policy, failure-stack contents,
HTML layout and rendered parity remain unverified. Arbitrary non-string label values also require the
input-conversion qualification; the authored login label is a string.

## Locale changes and other text mechanisms

Manager `MarvelUIManager.Initialize` (`169101+271`) invokes
`UpdateCurrentLocale` at 169210 and registers `LocaleReset` at 169234.
`handleEvent_LocaleReset` (`179588+134`) updates the locale and calls
`AcquireLocalizedData()` on each non-null registered object at 179704.

The expanded scan found `AcquireLocalizedData` in three **alternative
LocalizedObject definitions**, in the video, gameplay and audio options movies:

| Movie | Class initializer | Acquire body / ExternalInterface call | Update callback body |
| --- | --- | --- | --- |
| Video options | `113497+3061` | `115998+85` / 116081 | `115014+139` |
| Gameplay options | `114201+3061` | `116702+85` / 116785 | `115718+139` |
| Audio options | `85022+3061` | `87523+85` / 87606 | `86539+139` |

Those 3,061-byte class blocks are byte-identical. They call
`ExternalInterface.call("GetLocalizedDataForObject", this, m_localTextLabel)`
after checking the label against undefined and empty string. Their
`UpdateLocalizedPrototypeData(localizedData)` method consumes the prototype ID,
display text and tooltip flag and writes **plain `text`**, optionally uppercased.
Each whole class definition is skipped when `_global.LocalizedObject` is already
truthy. They therefore do not automatically patch manager's different class or
provide an alias for its differently named refresh method. Which global class
exists depends on actual initialization order.

**OBSERVED bytes; INFERRED selected UnrealScript token semantics:** package
function `MarvelGFxMoviePlayer.GetLocalizedDataForObject`, export 8434, contains
a 53-byte serialized script at logical 2352912. The bounded token interpretation
assigns the engine lookup result to `newLocalizedObjectData`, then calls
`callUpdateLocalizedPrototypeData(data, localizedObject)`. That wrapper, export
8430, has 52 serialized script bytes at logical 2352669 and explicitly invokes
`GFxUI.GFxObject.ActionScriptVoid` on `selectionSlot` with the method-name string
`UpdateLocalizedPrototypeData`. Exact object/name references, serialized lengths,
context skip counts and 64-bit pointer expansion were checked; a complete native
UnrealScript opcode contract was not reconstructed here.

**OBSERVED with REA:** `UGFxObjectexecActionScriptVoid` at `0x141282c90` calls
`0x14125c390`. The shared bridge obtains the invoking function from the supplied
frame and walks its parameter properties, accepting flags with
`flags & 0x480 == 0x80`. It reads each value from frame locals plus the property's
runtime offset, converts it at `0x14125c643`, and supplies the method name,
converted arguments and count to the object-interface call at `0x14125c736`.
Thus the wrapper's data can be forwarded from its frame even though only the
callback name is an explicit ActionScriptVoid operand. Both `Data` (`0x400080`)
and `selectionSlot` (`0x80`) satisfy the native mask; the observed serialized
child chain orders Data first, selectionSlot second. The native bridge
consequently supports a second forwarded argument as well; the saved AS method
declares one formal parameter. Do not assume that extra arguments are absent
merely from that declaration. The actual runtime child-chain order was not
captured.

The expanded search covered 52 saved input paths with 27 distinct hashes,
including every DoAction/DoInitAction and PlaceObject ClipAction, and recursively
decoded 20,302 functions without errors. Its 14 direct AcquireLocalizedData
Push uses comprise three definitions, three configUI calls, three VendorPanel
calls and five locale-reset calls. No direct alias/injection or ClipAction use
was found. Arbitrary computed names, native mutation and runtime reachability
remain outside that bounded negative result.

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
- `textfield/acquire-followup/receipt.json`: expanded 27-movie action and clip
  scan, exact alternative class bodies, guards and tooltip consumers. SHA-256:
  `c42dd386f851e7ca396c0d0b127d4e7f5bf1bcd7aa3194029d423c2af4dbbc48`.
- `aggregate-followup/metadata/receipt.json`: two owned UE3 packages, complete
  name/import/export tables, 25 selected exports, exact typed defaults and five
  bounded token interpretations. SHA-256:
  `c8c59df9d68b118a7acdbb0f1484e1b60e23f4c1c89f01e7e6a09489a5eefd92`.
- `aggregate-followup/metadata/native-linkage/receipt.json`: complete 3,735-function
  package reference scan and selected state-cache candidates; 68 retained files.
  SHA-256: `9cd3df726876a2d5dc8052adbbc1f43e925b18eef043bb6bb5d35fe094903f36`.
- `aggregate-followup/native/final.bundle.json`: five additional canonical REA
  records for diagnostics, input/output conversion, ActionScriptVoid, AS2 call,
  handler installation and the movie's state cache. SHA-256:
  `ee2dc52405ce5879a69c7e1ff6ebf97f3202b6bd647694916242c9ee342ceb67`.
- `aggregate-followup/verification.json`: new Evidence IDs, completion/readback
  checks, all 13 native evidence/candidate files and the five immutable previous
  or helper receipts. SHA-256:
  `08b1cf2d396166c86686a9926dd96a03b91f0efceff27f61df6d35bca4331f5e`.
  All earlier receipts remain unchanged.
- `aggregate-followup/{prototype-fields,prototype-resolution,locale-format}/receipt.json`:
  bounded field registration, cache/loader/allocator and locale loader/formatter
  candidate records, respectively. Their SHA-256 values are
  `1b7af4a6f60a40b773318a7bc36f81d59ca69eda26ef0e43ae1c146648cf4daa`,
  `d16ee43396659a07dcc28b1adbaa3bfbe8eba49234d9b68400656b6a540898bf`, and
  `f4aad27612b3441351d11318178b6d6f691129f2e0894679f4e8d76d5a2ca816`.
- `aggregate-followup/resource-profile/native.bundle.json`: four additional
  canonical REA records, including selected loader/variant/default-node paths,
  field registration, cleared allocation, formatter and import identities.
  SHA-256: `dd88d461b0b8c52eacebc832d88528fc9ab5fce7e6f82ccee8d5e8ac8b43d5b9`.
- `aggregate-followup/resource-profile/verification.json`: Evidence IDs, commands,
  unchanged input, complete selected JSON outputs and helper receipt readback.
  SHA-256: `71936d5868d94d6dbe6a2eff31d45dfaba11849d3a12446e55074f1f7fd59ff5`.

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

**VERIFIED (expanded static checks):** the Acquire scan accepted 36,115 tags,
2,531 action tags and 332 ClipActions, decoded 2,863 root programs / 20,302
functions with zero decode errors, and read back all 52 input aliases plus 97
retained files. The package helper verified both source hashes and 47 retained
files. Its selected bytecode profiles consumed every chosen serialized byte;
matching loaded-size and context-skip arithmetic supports, but does not upgrade,
the explicitly inferred UnrealScript token semantics. The five additional REA
commands completed with exit 0, untruncated output and unchanged executable
hash. The 82-byte state-install selection's partial final instruction was
superseded by an 84-byte read; the movie-entry selection has a later partial
tail outside the claimed `this` assignment. These boundary details are recorded
in the follow-up receipt and do not establish complete whole-program analysis.
The four resource-profile commands also completed with exit 0, untruncated
output, unchanged executable hash and no invalid decoded instructions in their
selected ranges. Their receipt verified 48 field, 37 prototype-resolution and
51 locale-format retained files, alongside the unchanged earlier receipts.
Full native functions and selected chained-unwind fragments are distinguished
in the candidate records; a completed selected read is not a complete serializer
or fallback-policy reconstruction.

## Sources and rights

| Source read | Licence / rights | Use |
| --- | --- | --- |
| Owner's `MarvelGame-frontend_manager.gfx`, `marvel_login_dialog.gfx`, and `!uisource.gfx` at the private paths recorded in the receipt | Proprietary retail data; authorized private interoperability study | Exact saved class definitions, methods, key, import/export association and field placement. |
| Expanded private GFX/SWF inventory: 52 paths / 27 unique hashes, including frontend options and font movies | Proprietary retail data; authorized private interoperability study | Complete selected action/clip search for AcquireLocalizedData, conditional class definitions and tooltip consumers. Exact aliases and hashes remain in the follow-up receipt. |
| Pinned `frontend-startup/UnrealEngine3/MarvelGame/CookedPCConsole/{MarvelGame,GFxUI}.upk` | Proprietary retail data; authorized private interoperability study | Native return-struct field identities/defaults, function/property references and selected wrapper script bytes. SHA-256: MarvelGame `d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6`; GFxUI `dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199`. |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owned 1.52.0.1700 client; authorized private interoperability study | Selected native callback wrappers, resource lookup, reference formatting and static table entries through REA. |
| Original project `src/avm1.rs`, `src/avm1/`, `src/gfx.rs`, `src/gfx/`, `src/upk.rs`, `src/upk/`, `src/localization.rs`, saved original decoder helpers, `docs/LOCALIZATION.md`, `docs/PROTOTYPE_VALUES.md`, and prior private PE-scanner helper | Original project code and analysis with their existing provenance | Reused privately for decoding, bounded address discovery, and comparison with the existing resource bridge. No external implementation copied. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` and registered REA tool schemas | Toolkit/workflow instructions; toolkit source licence not established by this read | Evidence handling, selected read-only analysis and canonical export. |

No new outside source repository, web page, dependency or client implementation
was read or copied for this investigation. The exact source/hash ledger is
private; the integrator can merge this source register into `PROVENANCE.md`.
