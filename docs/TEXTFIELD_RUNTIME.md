# Attached TextField storage and text replacement

**VERIFIED static native paths; OBSERVED saved field data.** An attached
TextField owns a display target, a distinct AVM interface, lazy script storage,
a last-input string, and a styled document. `text` and `htmlText` are native
properties, not interchangeable ordinary string members. This study supplies a
conditional contract for the login label; it does not execute the client or
establish startup, layout, or rendering parity.

The saved script route is
[LocalizedObject.updateLabelText](FRONTEND_LOCALIZATION_RUNTIME.md): manager
body `94114+178` writes `html=true` at 94201, then `htmlText` at 94252, with an
optional uppercase call immediately before that write. Its empty-label branch
also writes `htmlText`. Attachment, class initialization, event order, and
ExternalInterface return values remain separate dependencies.

## The actual saved label

UISource sprite 1039 places EditText 1034 as `localizedText_tf` at depth 5.
The complete DefineEditText body is absolute span `224143+41`; the six-byte
extended tag header starts at 224137. Independent bounded parsing consumes the
entire body and establishes:

| Field | Saved value |
| --- | --- |
| Variable name | Empty; terminator at 224171 |
| Initial text | `*BTN_LABEL*`, starting at 224172 |
| Wire flags | `0xad31`: HTML false; multiline, read-only, no-select, use-outlines true; word-wrap false |
| Font | ID 270, height 320 twips / 16 pixels |
| Color | RGBA `fe f0 d6 ff` |
| Layout | Alignment 2 (center); margins and indent zero; leading 40 twips |
| Bounds | `[-40,1920] × [-40,440]` twips |

The existing placement receipt supplies translation `(100,120)` twips and
three filters. Those are not discarded by changing text. Saved SWF flags are
not the same bit positions as native definition/target flags below. This study
does not infer that mapping merely from equal-looking values.

The empty variable name is useful: the native variable-binding notification
below is a no-op for an attached owner retaining that empty definition name.
The authoring placeholder is not a localization key and must not be replaced
globally. The saved script supplies `Frontend/UIButtonLogIn`.

## Attached and unbound identities

All addresses are preferred-image virtual addresses in the pinned x64 client.
Let `T` be the attached display target, `O` its AVM owner, `V=[T+0xe0]` the text
view, `C=[V+0x10]` its styled document, and `D=[T+0xd8]` the source definition.

Source factory `0x140178440`, selector 4, allocates `0x1b0` bytes and calls
`0x140143880`. Its successful return is the allocation pointer `T`. It then
constructs `O=T+0x148` through `0x140204a40`. Binding helper `0x140099640`
stores `(O-T+3)>>2` into target byte `+0x6d`, yielding `0x52` here. This proof
uses the TextField branch, not the MovieClip branch's class identity.

| Native field | Observed role |
| --- | --- |
| `T+0` | Primary table `0x14250a800` |
| `T+0xd8` | Source definition `D` |
| `T+0xe0` | Text view `V` |
| `T+0xfc` | Target state bits described below |
| `T+0x100` | Separate tagged-string storage for the last assigned input |
| `T+0x108` | Optional dynamic extensions, initially null |
| `O+0` | Primary table `0x142517970` |
| `O+8` | AVM interface `0x1425178c0`; category virtual returns **4** |
| `O+0x18` | Current display prototype, initially selected dynamically with builtin selector 10 |
| `O+0x20` | Target `T` |
| `O+0x30` | Text-specific interface table `0x1425177b0` |
| `O+0x38` | Interned definition variable name |
| `O+0x40` | Variable-binding Value cache, initially tag zero |
| `O+0x60` | Lazy ordinary script-storage object, initially null |
| `C+0x30`, `C+0x38` | Current paragraph-format and text-format defaults |

Category numbers here are internal virtual results, not script `typeof`
values. Environment helper `0x14017a130` walks parent targets until the
appropriate parent marker, then returns that parent's environment. The attached
TextField does not embed the MovieClip environment layout.

The lazy path `0x1401eb730` allocates `0x78` bytes and calls `0x1401deb20`.
Storage has primary/interface tables `0x142512ca0` / `0x142512cf0`, category
**13**, a target-reference token at `+0x68`, and the owner's current prototype
at `+0x30`. Existing storage is reused. The token is not itself proof that all
public display handles can be represented as an ordinary object reference.

By contrast, the deferred global TextField factory `0x14020de90` creates a
callable with callback `0x14058fea0` (a no-op return), a separate prototype, and
allocator `0x1401dec40`. That allocator uses `0x1401deab0` to create unbound
category-13 storage with a zero target token and no attached text document.
Merely constructing or registering this builtin does not create `T`, `O`, or
the saved label. The global registration evidence is in
[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md).

## Member routing and bounded inputs

The verified standard-member table maps `text` to ID **40**, `html` to **45**,
and `htmlText` to **46**. Each has the table's bit-29 classification and not
bit 28. Attached setter `0x1401eb7d0` uses the canonical name lookup
`0x14018ff00`; getter `0x140195640` uses the corresponding IDs. The setter first calls a generic
standard-property helper, which declines these IDs before the TextField switch.
The getter routes directly into its own ID switch; its generic getter is in
the out-of-range/default path.

For these setters, a non-null environment can first cause lazy storage creation
and invoke its watch path. A watch can replace the copied Value consumed by the
native property. The simple profile therefore requires no applicable watch;
it must not silently skip a present one. The TextField raw bridges
`0x140203490` / `0x1402034f0` obtain the attached environment and forward to the
normal setter/getter. They do not turn these native members into raw map entries.

This study admits actual Boolean input for `html` and actual string input for
`text`/`htmlText`. ToBool `0x14017d730` reads the Boolean payload for tag 2.
ToString `0x1401a2920` preserves/retains the existing string node for tag 5;
the selected string branch is `0x1401a2988`. Version-dependent undefined/null
handling and arbitrary coercion are outside the contract. The native selected
path passes a NUL-terminated byte string, so embedded NUL is also outside the
simple profile rather than silently treated as an unrestricted Rust string.

| Property | Setter | Getter |
| --- | --- | --- |
| `html` | At `0x1401eb90b`, set/clear target `+0xfc` bit 1 from the Boolean. No text parse or dirty call in this branch. | At `0x14019569f`, return bit 1 as a Boolean. |
| `htmlText` | At `0x1401eb938`, obtain the string and call `0x1401463b0(T,input,bit1,true)`. | At `0x1401956c1`, call `0x14013dbd0` in HTML mode. |
| `text` | At `0x1401eb9ba`, first call selection/edit helper `0x14013dff0(T,true,false)`, then `0x1401463b0(T,input,false,true)`. | At `0x1401956f2`, call `0x14013dbd0` in plain mode. |

Writing `text` does not itself clear `html`. The selected setter returns
success after cleanup even when the shared replacement helper returns false
for unchanged input; do not expose that helper result as setter failure.

## Last input, parse mode, and invalidation

The shared helper `0x1401463b0` first compares the incoming C string byte for
byte with `T+0x100`. If equal and target bit 16 is clear, it returns **before**
changing parse mode, reparsing, or marking the text dirty. Changing `html`
alone does not bypass this guard. For example, with equal input and bit 16
clear, enabling HTML and writing the same `htmlText` does not establish that
the existing plain document was reparsed. The preceding `text` edit/selection
helper is a separate possible side effect even if the core later returns early.

On the replacement path, the helper:

1. Clears bit 16. An attached stylesheet can force HTML enable and HTML parsing.
2. Assigns the input to `T+0x100` through `0x140034e30` and its string helper.
3. Sets/clears target bit **12** for the requested parse mode. This is distinct
   from the public HTML-enable bit **1**.
4. Checks a root translation/formatter service (state selector 1) when target
   bit 3 is clear. A present service has extra transformation branches; the
   minimal profile requires that gate to be absent or bypassed explicitly.
5. Takes the plain or HTML document-replacement path below.
6. May clamp a live editing view and notify a nonempty variable binding. The
   notification `0x140204fe0` reads the resulting plain text into `O+0x40` and
   invokes the variable-write path; an empty `O+0x38` avoids it.
7. Sets target bit **13** and marks the render-side object dirty through
   `0x14014ea10`. The view replacement wrappers also invalidate via view
   virtual `+8` with mask `0x106`.

The parser flag supplied from target bit 4 and a separate view flag are
explicit inputs. No generic whitespace, wrapping, translation, editor,
stylesheet, or image policy is invented for them. Dirtying proves an update
request, not completion of layout or drawing.

## Replacement and style source

Plain wrapper `0x1401647e0` calls `0x14012ef50`. That clears paragraph storage
through `0x14012ed30`, preserving `C+0x30` / `C+0x38`, then passes those
**current document defaults** to the append path `0x14012d930`. The append
path decodes input, stores 16-bit text units, and applies the supplied format
profiles. It has explicit CR/LF handling; general newline and malformed-Unicode
behavior are not part of the simple label profile.

HTML wrapper `0x1401648f0` also clears the document. It calls
`0x140178220`, which converts input to 16-bit units and invokes parser
`0x140176090` with the supplied format profiles, parser flags, and optional
stylesheet/image state. In the no-formatter branch the profiles come freshly
from `0x140141640(T,A,B)`, not from copying the old document's runs. Empty
input reaches the parser's empty-document path; the outer wrapper still
invalidates even though the inner parser returns false.

The definition helper resets two temporary profiles and derives their fields
from `D` and resource lookups. Direct numeric evidence includes:

| Definition source | Temporary profile destination |
| --- | --- |
| `D+0x50`, f32 | A size field `+0x3e`, with presence bit 3 at `+0x42` |
| `D+0x54`, u32 | A color field `+0x38`, with presence bit 0 |
| `D+0x84`, selectors 0/1/2/3 | B alignment bits `0x600` become 0/`0x200`/`0x600`/`0x400`, presence bit 0 |
| `D+0x5c/+0x60/+0x64/+0x68`, if definition bit 9 | Four B numeric fields multiplied in f32 by exact constant bits `0x3d4ccccd`, converted/truncated to 16 bits, with separate presence bits |

These are native definition fields; source-unit conversion is a separate
loader contract. In particular the size conversion compares unsigned low32
against `0x10000` before choosing low16 or `0xffff`; it is not generic
saturation. Font/name/resource resolution enriches A through additional calls.
The saved font ID by itself does not prove a live resource lookup succeeded.

Target construction calls initialization bridge `0x140141c20`. It combines
profiles before storing persistent defaults through `0x14012b630` and
`0x14012b810`, at `C+0x38` and `C+0x30` respectively:

- Definition bit 11 clear: helper-derived A/B overlay existing defaults where
  the helper's presence bits are set.
- Definition bit 11 set: existing defaults overlay fresh reset profiles.

The combine functions `0x1400bb030` / `0x1400bbbf0` copy the base then apply
only fields marked present by the overlay. Reset zeroes in absent paragraph
fields do not erase older defaults. Persistent format factories and a
conditional A normalization remain separate callees; no complete font-object
or format-interning implementation is inferred here.

This distinction matters to a runtime bridge: preserving only saved authoring
style is insufficient after format mutations, and using the current plain-text
defaults for every HTML assignment is not the observed caller contract.

## Readback is not the assignment cache

Plain getter mode always reads `V`'s document through `0x1401648a0` and
`0x140127a50` / `0x1401274f0`. It concatenates stored paragraph text, omitting
each paragraph's final NUL unit if present, and appends through `0x140034a40`.
That path uses explicit-count conversion `0x14006aef0`; ASCII units produce
the same one-byte values. It does not return `T+0x100` or insert separators
independently of the stored paragraph contents.

HTML getter mode `0x14013dbd0` selects:

| State | Readback source |
| --- | --- |
| Attached stylesheet gate true | Intern/return cached last-input string `T+0x100` |
| No stylesheet, public HTML bit 1 true | Serialize the current styled document through `0x1401648d0`, `0x14012b160`, `0x140128540` |
| No stylesheet, HTML bit 1 false | Same plain document readback as `text` |

The stylesheet gate `0x14017c4b0` checks optional `T+0x108` and its `+0x50`
pointer; the companion payload getter `0x14017c4e0` returns that object's
`+0x68` address. A fresh target initializes the extension pointer null. Later
stylesheet attachment is not proven absent merely by this constructor.

The serializer emits paragraph formatting even for a plain-looking nonempty
run. Its observed paragraph shape contains `TEXTFORMAT`, `P ALIGN=...`, and,
for a non-null normal run format, `FONT` with a mandatory KERNING attribute.
Optional attributes depend on format presence bits. A null run format skips
the FONT wrapper. The exact output for the saved field cannot be declared from
the assigned string alone. Six explicit unit escapes are verified: quotation
mark, ampersand, apostrophe, less-than, greater-than, and nonbreaking space.
The ordinary-unit encoder `0x14006acd0` and downstream Boolean-format expansion
for `KERNING` remain unexpanded dependencies; this study does **not** prescribe
a byte-exact HTML string or claim full HTML readback support.

## Conditional implementation seam and remaining boundaries

**INFERRED implementation design from the observed paths:** retain an attached
TextField identity linked to its exact saved display instance and ordinary
storage. Keep HTML enable, last parse mode, force-rebuild state, last-input
bytes, current document text/runs, and current default profiles separately.
Resolve the three native member IDs before using an ordinary member fallback.
A replacement result can update the existing saved-text layout input and
request relayout while retaining placement, source style/resource identities,
and filters. None of those data changes establishes rendered parity.

The native evidence supports a small explicit string profile: live attached
receiver, actual Boolean/string inputs, no applicable watch, no active
translation/stylesheet/edit-binding side path, successful allocation and
valid format/resource state. A no-markup ASCII label with no CR/LF/NUL and
whitespace-condensation disabled follows the parser's token-3 text route:
`0x140175680 -> 0x140175dc0 -> 0x14012dc50`. The text run's units are retained;
the plain getter encodes them back to the same ASCII bytes. This is a static
conditional result, not an observed runtime execution of `LOG IN`.

The saved empty variable name closes one gate. It does not close watches,
resource lookup, renderer state, definition-to-native flag mapping, lifecycle,
or runtime service presence. Coercion outside string/Boolean inputs, Unicode
edge cases, rich HTML/images/entities, exact HTML serialization, translation,
selection/caret/focus behavior, sizing/wrapping, filters, format mutation APIs,
prototype mutation, target retirement, and construction/event order must remain
explicit unsupported or separately evidenced branches. Do not use a generic
AS2 library or literal-echo `htmlText` getter to fill these gaps.

## Evidence, validation, and source ledger

Private root: `~/.local/share/marvel-heroes-forever-rust/`. No binary, extracted
script, field payload, disassembly, or evidence bundle is committed.

| Source | Licence/status | Use |
| --- | --- | --- |
| `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | Owner's proprietary Gazillion/Marvel installation; private compatibility study | Attached/unbound identity, member switches, replacement, defaults, parser/readback paths |
| `evidence/gfx-samples/!uisource.gfx` | Same proprietary installation | Complete saved EditText1034 body and its linkage |
| Existing project `FRONTEND_LOCALIZATION_RUNTIME.md`, `ATTACHED_MOVIECLIP.md`, `AS2_GLOBAL_BASELINE.md`, `src/gfx/text.rs` | Original project analysis/code; underlying evidence private | Exact script use, distinguish MovieClip, global registration, independent saved-field comparison |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` | REA MIT toolkit | Evidence handling and target-free bounded analysis workflow |
| REA / Rizin provider | MIT / LGPL-3.0-only tooling; no provider code linked | Static disassembly and exact data observations |

Executable SHA256:
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`
(48,544,080 bytes). UISource SHA256:
`49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5`.
Its selected 41-byte body SHA256:
`90eb2fb234f3dc76a3bdecf418264235806cabeb4cf9373729809d717249d786`.

New records reside under `evidence/textfield-runtime/`:

| Record | Canonical REA ID |
| --- | --- |
| `rea-textfield-factory.json` | `ev_1055066c6117b43a252d506206dab00b100091cea6e467b314f84d02e834c27d` |
| `rea-unbound-allocator-prototype-prefix.json` | `ev_494f5a132ee13eb2ad9766078ed4bc4ca8bc6a1dd4bfa3b68d75a8031ead8b1c` |
| `rea-attached-identity-member-bridges.json` | `ev_f6cd6a8ed0bc6c5efabb5e731463f80f637d548331595f05fffe3e6802ce75b9` |
| `rea-attached-text-accessors.json` | `ev_469be6a85ccea04c1d887b7a7429a9d5f59c5de876d940d1a0f4f62de1fafda4` |
| `rea-text-write-core-and-storage.json` | `ev_31a99dac0759d9222bdb842784d1f2491bed367a9923f88582564be1dfc9c2ed` |
| `rea-readback-style-and-parse-entry.json` | `ev_f0464d9f9261752a01b6d6a994e2ae118088939ae80905bef3b79c5a0318be1f` |
| `rea-target-constructor-document-paths.json` | `ev_51d87eaf030212bafc0535836eb6bd96c29e46626bea63bdae48ac7b1f1376fb` |
| `rea-default-reset-plain-append-readback.json` | `ev_b919e49a791f4bad1ed48a725355fc6e04ae15714c19764a8c2892767a503a6f` |
| `rea-html-serializer.json` | `ev_04ea58c22d7fc33642ef6f9167ca8eaaf63959d386a54003fa39073ccda9b177` |
| `rea-html-parser-and-plain-readback.json` | `ev_14c8ee40054e0290833032a6888843f4c4f5ba422e5972485214d9d2e506e1fa` |
| `rea-default-merges-ascii-format.json` | `ev_a562ce0b469ba065ffad2b5e01d2364bf450c4ea9f7d6c2d3ec6a8905c2ad7b1` |
| `rea-plain-html-run-and-encoding.json` | `ev_8c624804103d91bec4218d38bc080acb51d57604f2df474f0db8287f3ac116df` |

`complete.bundle.json` contains these 12 records plus 14 earlier animation
records retained in this session; export SHA256 is
`0474dcfd507c1cc68d0864cf5e1a9928d23f9db690f89ef84047ad53f41152a8`.
The original verifier checks all 12 new records complete, exit-zero,
untruncated, and unchanged; it compares **78 ranges / 50,527 observed bytes
including overlap** against PE section mappings independently of Rizin's raw
base. `verification.json` SHA256:
`3c39a2d6263c993b0975539150dd489bcda7c81ce9566e95c6ea913fcdaa62a2`.
This is byte consistency, not a count of executed or semantically proven
instructions. Embedded switch tables, padding, neighboring bodies, and
incomplete prefix tails are not treated as code proof.

Focused independent receipts include `saved-field-independent/saved-field-1034.json`
(SHA256 `5147b40aa19e899925fe9e5b9e32eefd477770405709a65e11e4b46fd93da742`),
`style-independent/style-receipt.json`
(`e1d6a63a7ee15c325dcec90873c106581f476354f5ceb1dd89e97f5fa165d76c`),
and the `serializer-independent/` contract, escape, and final helper-boundary
receipts. The style review checks 12 full ranges / 7,891 bytes; serializer
review checks 7,811 bytes while separating 155 bytes of trailing switch data.

Reused source-factory records are
`ev_63afb28a4e5a8587d3d55625096a552ed22a947f9c384cac9c1ace7f5dfa051c`
and `ev_6c4d08c2c661c182c698f0e1a41179cf6ef863808887629e100ede2e0dd81409`
in `evidence/rea-asflags-followup-version-proof/final.bundle.json`; the
constructor-return gap in that earlier slice is closed by the new full
`0x140143880` capture. Exact name/ID table evidence is
`ev_06752c5af92f95580a84cd9200320202857cc0052006fab5a6a5833224f08650`
and `ev_89cb16c8abed146a0071376ee2eb978c82bb47d08a2504144dea383b87a6726d`
under `evidence/rea-attached-name-map/`; lookup is
`ev_a7b803905b375352deb3ec300246f684190dbb8db9efe96f7645677f3c9e2330`.
Private candidate/review files record the exact reused inputs and their hashes.

No native execution, live UI test, source implementation port, shared
configuration edit, or broad test suite was performed for this docs-only
increment. No Ghidra target was opened; the bounded target-free Rizin calls
required no target close or shared-session changes.

## Conditional replacement-state implementation

`src/frontend/textfield_state.rs` adds an original, standalone state primitive.
It has no VM, host, renderer, display-state, or module-registration hook. It does
not instantiate the saved label or manufacture native initial values.

The caller supplies `AttachedIdentity` (association domain, scene node, AVM
owner), `Profile`, `Snapshot`, and `Limits` to
`TextFieldState::from_snapshot`. The snapshot explicitly carries both raw native
flag fields, separate last-input/document text, the document's uniform run
profile, current default profiles, definition-derived HTML profiles, and pending
invalidation. The association domain and nonzero typed format handles are
caller-local identifiers, not native pointers. Non-null format handles are this
module's restricted profile; they are not asserted to be universal native rules.
The module cannot establish the truth of a caller's identity/resource evidence.

Saved `EditText`, `FieldDefaults`, and the existing HTML declarations preserve
authoring data; they do not represent the native current/default profile pair or
format-factory results. The new module therefore retains opaque `TextFormat` and
`ParagraphFormat` handles instead of guessing that conversion. The caller must
bind those handles to validated effective profiles in the same domain. HTML
input run format and defaults after a nonempty parse are supplied separately:
the native default factory can normalize its input, so these handles need not
have the same identity or value. No automatic renderer/font resolution occurs.

Admission requires every explicit side-path observation to be `Absent`:
watches, translation service, stylesheet, editor, variable binding, and image
callbacks. `Present` and `Unknown` are errors. This is intentionally a restricted
profile even where a particular operation could avoid one of those branches.
Native target `+0xfc` bit 4 and view `+0x149` bit 2 must both be clear. They are
not inferred from SWF flags. Re-admit a snapshot if external activity changes
these conditions; the module is a closed conditional state, not a live observer.

Accepted text is empty or printable ASCII (`0x20..0x7e`) without `<` or `&`.
NUL, CR/LF, tabs, other controls, non-ASCII, markup, and entities are rejected
with a byte offset. Both initial string stores are checked independently. The
single uniform document requires a format for nonempty text and no character
run for empty text. That projection does not assert the native empty-paragraph
representation. Configurable byte limits cannot exceed 1 MiB per string. There
is no retained operation log or unbounded run list.

| API | Conditional effect |
| --- | --- |
| `set_html(bool)` | Change only public bit 1; return the old/new Boolean. No parse or invalidation. |
| `set_text(&str)` | Request plain replacement without clearing public HTML enable. |
| `set_html_text(&str)` | Request HTML or plain replacement according to public bit 1. |
| `text()` | Return the current document text, independent of last input. |
| `html_text()` | Return document text when HTML is disabled; otherwise return `HtmlSerializationUnproved`, including empty documents. |
| `snapshot()`, `parse_mode()`, `force_rebuild()` | Inspect retained state without performing native calls. |

The same-input / clear-force guard precedes mode, profile, or invalidation
changes. It returns `Replacement::Unchanged`, a successful native-core no-op,
not a failed property assignment. A changed or forced replacement allocates
both bounded strings before mutation; input, limit, or allocation errors leave
the snapshot unchanged. Replacement clears bit 16, selects bit 12, sets bit 13,
ORs `0x106` into pending view invalidation, and requests render dirtying. Other
target/view bits and prior pending requests are preserved. Invalidation is an
explicit request for a future host, not proof of rendered output.

### Follow-up: HTML completion updates current defaults

Implementation tracing resolved a distinction left unstated in the earlier
style-source section. In the retained nonempty, no-markup token-3 route,
`0x140176090` leaves the HTML element stack depth zero. Completion at
`0x140177ece` supplies local A/B to `0x14012b630` / `0x14012b810`, with calls at
`0x140177ed5` / `0x140177ee1`. These install the resulting current defaults.
Consequently a later plain replacement uses those new defaults, not the defaults
from before the HTML write.

Empty HTML instead calls `0x14012e010` at `0x1401760bc` and returns before the
default-store tail. The new narrow finalizer capture checks its paragraph
creation/final-NUL route: it takes current `C+0x30` / `C+0x38` as inputs through
`0x14012d3d0` / `0x14012cad0`, rather than taking the definition's supplied HTML
profiles. The module therefore preserves current defaults for empty HTML and
plain replacement, and installs the explicitly supplied
`DefinitionHtmlProfile::defaults_after_nonempty` only for a nonempty HTML
replacement. Ordinary successful allocation/format-factory behavior remains a
caller precondition; native heap identity is not modeled.

The two added canonical REA records are private under
`evidence/textfield-state/`:

- `rea-empty-document-finalizer.json`,
  `ev_27b32e521547ebdfbc3e5e0e72a8478e5b1c46d1f7b13f8ae2016b73a5ea2b95`.
- `rea-finalizer-paragraph-helpers.json`,
  `ev_3240e38e4bf52103e6341fa9e9d4c3f34ed759508fbb58dfe70f412928dd861c`.

`native-finalizer-verification.json` checks all 848 bytes / 250 instructions
against the same pinned executable's PE mapping, with complete, exit-zero,
untruncated, unchanged receipts. `complete.bundle.json` in this directory
contains these two records plus the 26 already retained records. No target was
executed or opened, and no shared configuration changed. The source licence and
workflow ledger above applies; the additional existing-project reads were
`src/frontend/display_state.rs`, `scene_content.rs`, `src/gfx/html_text.rs`,
`edit_layout.rs`, `html_layout.rs`, `glyphs.rs`, and `scene.rs`, solely to assess
compatible API and style representations. No outside implementation was read.

Focused validation passed **six module tests** and a standalone library compile
with warnings denied. Tests cover independent last-input/document storage,
HTML toggle followed by unchanged input, forced rebuild, plain replacement
while HTML remains enabled, distinct input/result format handles, HTML default
updates followed by plain replacement, empty clears, unknown/present gates,
unsupported inputs, exact byte limits, and unchanged snapshots on rejection.

Separately, `evidence/textfield-state/conditional-cases.json` contains **13**
literal native-derived transition cases authored without inspecting the module.
The original adapter `check_cases.py` checks their raw flags, both string stores,
run/default profiles, new invalidation events, readback boundary, and identity;
all 13 cases / 130 checks pass. `conditional-validation.json` pins the oracle,
adapter, harness, and module hashes. These are conditional model checks using
explicit supplied state, not evidence that the saved frontend has executed or
that these flags are its live initial values. No broad suite, startup execution,
layout comparison, or native runtime test was run for this increment.
