# Saved vertical centering and attached TextField alignment

**OBSERVED saved script; VERIFIED static native paths.** The login button's
Construct action explicitly assigns `verticalCenter=false`. Its unchanged
`LocalizedObject.recenterText` consequently needs no vertical-alignment write
when that value has reached the instance. Applying the saved assignment through
the real class/lifecycle path is still a separate obligation. A controlled
fixture that supplies `m_VerticalCenter=false` does not establish that lifecycle.

There is an earlier, distinct geometry path: Construct assigns `label=""`
before assigning `verticalCenter=false`. The inherited flag's value at that
earlier point is not established by the later assignment. These findings do
not justify either an invented true branch or an invented initial false value.
No native client, VM fixture, renderer, or live login was executed for this study.

## Saved instance and action order

The scene receipt identifies text node **89**, parent **86**, at
`24:loginContainer_mc/39:loginButton_mc/5:localizedText_tf`. The field is saved
UISource EditText **1034**; its bounds, flags, font and initial placeholder remain
as recorded in [TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md). The UISource movie
has version 11; the manager script has version 15. This study does not remove
that version distinction from AVM execution requirements.

Within the original 449-byte login Construct body at absolute offset **39236**:

| Absolute action offset | Saved assignment |
| --- | --- |
| 39553 | `forceAllCaps=true` |
| 39577 | `label=""` |
| 39585 | `localTextLabel="Frontend/UIButtonLogIn"` |
| 39675 | `verticalCenter=false` |

The Boolean payload for the last assignment is an actual Push Boolean zero in
the saved bytes, followed by SetVariable. It is not inferred from absence.
SetVariable still depends on its current scope/target and descriptor routing.

The complete scoped class evidence supplies a concrete inherited route:

* `MarvelButton` executes Extends with `LocalizedObject` at **152854**; its
  constructor calls zero-argument `super` at **152809**.
* `LocalizedObject` binds its prototype to register 2 at **92149–92161**.
  Its `addProperty` sequence at **94721+31**, CallMethod **94751**, connects
  `verticalCenter` to getter body **92617+9** and setter body **92569+29**.
* The setter writes its supplied argument to `this.m_VerticalCenter` at
  **92578**. It performs no coercion, conditional branch, recenter call or Y
  restoration. The getter reads that member.

The LocalizedObject initializer is guarded by the existing global class value;
the descriptor installation itself is conditional. MarvelButton's nine explicit
`addProperty` registrations do not introduce its own verticalCenter descriptor.
This supports the inherited path when those initializers/prototypes apply; it
does not prove arbitrary runtime replacement or existing-global state absent.

The complete LocalizedObject initializer **90399+4390** (158 outer actions,
30 SetMember sites), MarvelButton initializer **150448+8677** (255 outer
actions, 53 SetMember sites), and their constructors contain no explicit direct
initial store to `m_VerticalCenter`. This is a bounded absence finding, not a
default value. Earlier state, prototype values and lifecycle order remain inputs.

## The two different script paths

`LocalizedObject.recenterText` has declaration **94502+63**, unchanged body
**94513+52**, inherited constant pool **90399**, two registers, flags `0x29`,
and no parameters. Its body first reads `this.m_VerticalCenter`. A false value
short-circuits before reading `localizedText_tf`, comparing it with undefined,
or assigning a property. Otherwise it checks that the field is not undefined
using Equals2 at **94541**, then writes `verticalAlign="center"` at **94564**.
It performs no height query, subtraction or `_y` assignment.

`updateLabelText` body **94114+178** calls recenterText at **94269**. Its source
is retained unchanged by the separate frontend fixture lane. A successful false
branch fixture establishes only that supplied branch profile, not native
verticalAlign support or saved Construct application.

By contrast, `MarvelButton.__set__label` has declaration **153608+395** and body
**153630+373**. It calls `getTextField` at **153709**, retains the result in
register 2, and guards an undefined field before the text work. The separate
`m_VerticalCenter` / background gate is **153878+36**; its true body is
**153914+70**, ending with SetMember **153983**. For numeric resolved operands,
the saved operation order is:

```text
field._y = ((background_mc._y + background_mc._height / 2)
            - field.textHeight / 2) - 3
```

This is a representation of the captured AVM operations, not permission to
replace them with host arithmetic: member reads, Add2, Divide, Subtract and
SetMember retain their VM semantics. The scalar 3 is saved explicitly, but this
receipt alone does not establish its physical units. Empty string is not
undefined, so the early saved label assignment cannot be ruled out of this
path merely because the label is empty. The later flag setter does not undo a
prior position change.

## Native alignment identity and dispatch

Use the independently recovered attached TextField identity from
[TEXTFIELD_RUNTIME.md](TEXTFIELD_RUNTIME.md): target `T`, owner `O=T+0x148`,
AVM interface at `O+8` with category 4, and text view `V=[T+0xe0]`. This is not
the MovieClip category and not unbound TextField script storage. The attached
environment is obtained from its parent chain by `0x14017a130`; raw bridges
`0x140203490` / `0x1402034f0` forward into the normal setter/getter.

The canonical standard-member table maps `verticalAlign` to **ID 77** with
bit-29 classification set and bit 28 clear. Its row at **0x142adcf70** has
string pointer **0x14250d050**, ID byte `0x4d`, and zero extra selector. Native
name lookup is `0x14018ff00`.

Both native ID77 branches require byte **`[[Env+0xe8]+0x68] == 1`**. This is
the context extension flag associated with the recovered gfxExtensions state;
the study does not establish its value for an executing login. A runtime
contract must supply or prove it.

Setter `0x1401eb7d0` copies the Value. For this ID and a non-null Env, the
pre-switch path may create/reuse lazy storage via `0x1401eb730` and invoke a
watch which can replace the copied Value. The generic standard setter is tried
before the TextField-specific branch. The simple observed string profile
requires the relevant hook/watch conditions to be satisfied, not discarded.

Getter `0x140195640` instead dispatches directly through its own ID25–109
switch. Generic/ordinary fallback is later. In particular, the generic-helper
ordering described for setters must not be generalized to getters.

## Exact native ID77 behavior

Setter case **0x1401eccdd** uses the copied Value's ToString result. The actual
saved `"center"` string exercises the known string-tag path; arbitrary coercion
and embedded NUL are outside this contract. The native temporary string copy
at `0x1400360c0` uses a byte length and terminator, without a case-folding step.
Comparisons include the terminating NUL.

| Exact input | Mode bits 2–3 of `u8[V+0x148]` |
| --- | --- |
| `none` | 0 |
| `top` | 1 |
| `bottom` | 2 |
| `center` | 3 |

A recognized input changes only those mode bits and ORs **`V+0x14b` with 1**
at **0x1401ecdc0**, even when the selected mode was already equal. An unknown
string skips both writes. Both recognized and unknown strings still call
**`0x14013d3d0(T)` at 0x1401ecdcb**, then return success on this enabled branch.
That helper obtains an object through `0x14009a760` and tail-calls
`0x14014ea10`, whose previously retained path marks/enqueues display work.
This is invalidation evidence, not completed layout or rendering.

The case contains no direct `_y` write or text reparse call, and does not set
the text-replacement target flag `T+0xfc` bit 13. Vertical layout dirtiness must
remain distinct from last-input/document replacement and its parse mode.

Getter case **0x1401961f2** extracts `(u8[V+0x148] >> 2) & 3` and returns the
corresponding interned string above. Its common return/cleanup path succeeds;
it does not then consult ordinary storage. Thus an enabled native getter is
not shadowed by an ordinary `verticalAlign` member.

When the extension gate is disabled, the setter delegates to the common
attached setter **0x1401aa130** through **0x1401ed30e**. The getter goes to
**0x1401964ef**: when lazy storage exists, it returns that storage getter's
result; when storage is absent, it uses the relevant
prototype/fallback path. It does not create lazy storage there. Neither an
automatic `none` result nor successful ordinary assignment is inferred without
that fallback's actual supplied state.

## Native metric and layout dependency

Standard IDs **41/42** are `textWidth` / `textHeight`. Getter case
**0x140195748** for textHeight calls **0x14013d7d0(T)**, which calls
**0x14016cff0(V)**. If `V+0x14b & 3` is nonzero, the latter calls formatter
**0x14016bbf0**, then clears those two dirty bits. It returns
`f32(u32[V+0xc4])`, or positive zero for zero. The wrapper multiplies in f32 by
the exact literal **0x3d4ccccd** (`0.05000000074505806`) and widens the rounded
product to f64. It does not perform an additional integer rounding. Width uses
the corresponding `V+0xc0` metric through **0x14013d7a0 → 0x14016cfb0**.
The native textHeight query can therefore trigger formatting; it is not a
passive lookup of the saved field's bounding rectangle.

The retained constructor at **0x140143b74–0x140143b85** passes `D+0x40`, where
`D=[T+0xd8]`, to rectangle setter **0x1401671f0(V,rectangle,1)**. That setter
copies the four f32 values into `V+0xb0/b4/b8/bc`. Its unchanged ordered-equal
rectangle path returns early; unordered comparisons take the changed path.
On change it initializes the inner rectangle `V+0x80/84/88/8c` by adding or
subtracting the exact f32 inset **40.0**. Its later invalidation depends on
the supplied update selector, dimensions and existing view bits. This locates
the native definition source; mapping that definition to decoded SWF bounds
and later transform operations still needs its own proof.

The formatter **0x14016bbf0..0x14016cf33** has several layout paths. The
ordinary vertical placement at **0x14016c5e9..0x14016c7c4** is skipped if an
earlier branch adjusted the rectangle, the local effective mode is zero, or
`V+0x149` bit 1 is set. The earlier optional scaling branch can change a zero
local mode to 3 at **0x14016c48f** and can change the metric; a direct read of
the two mode bits alone does not establish this branch's entire input state.
In its full-metric branch:

```text
top       = f32[V+0xb4] + 40.0f32
bottom    = f32[V+0xbc] - 40.0f32
available = bottom - top
metric    = f32(u32[V+0xc4])
```

If `available > metric` in the observed SSE comparison, mode 3 computes
`(available * 0.5f32) - (metric * 0.5f32) + top`, preserving the separate f32
multiplications/subtraction/addition, and stores the result to **`V+0x84` at
0x14016c6cb**. There is no integer-rounding instruction in this selected write.
Mode 2 uses `bottom - metric`; mode 1 restores the inset rectangle.

`available <= metric` **or unordered** branches to a different path. For
modes 2/3 it traverses existing line-like records starting from the result of
`0x1401652d0`, accumulates fitting record metrics (with distinct compact and
full layouts and signed leading), and uses that sum for placement. A separate
earlier rectangle-adjustment path also exists. These paths prevent treating
verticalAlign as an unconditional `(height-textHeight)/2` operation. Full
line records, formatting, scrolling, autosize and transforms are not recovered
by this narrow study.

## Implementation boundary

The present evidence can drive an explicit native-state contract for the
enabled alignment string setter/getter and dirty flag, with supplied attached
identity, Env, watch/hook profile and current view state. It does not authorize
an implicit initial mode or translation of a native format handle to a renderer
style. Existing TextField replacement state and per-instance prepared text
remain separate concerns.

For actual login, preserve the saved false assignment and prove its descriptor
application at the correct lifecycle point. The earlier label setter remains
conditional on its then-current flag/background/field values. Its public
MovieClip `_y`/`_height` reads and attached TextField `_y` write, including
transform units and coercions, are not proven by the view-layout arithmetic
above. No new renderer/VM hook or geometry fallback is implemented here.

## Evidence, validation and provenance

All raw bytes, extraction outputs and native instructions remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/`. No game payload is committed.
The source ledger for this increment is:

| Source | Licence / use |
| --- | --- |
| `pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe` | Owner's proprietary client, compatibility study only. Build 1.52.0.1700, 48,544,080 bytes, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`. Native identity, members, invalidation, metrics and layout. |
| `evidence/frontend-startup/MarvelGame-frontend_manager.gfx` | Owner's proprietary saved movie, private study. 2,055,748 bytes, SHA256 `04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`. Exact functions, Construct order and scoped class initialization. |
| `evidence/textfield-host-validation/{scene,script}/` | Original project evidence derived from owner movies; instance identity, unchanged function extraction and constant pool. UISource SHA256 `49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5`. |
| `evidence/{textfield-runtime,rea-attached-name-map}/` and the linked project docs | Original prior project analysis of the same owned executable; category/Env/default/storage proof and full standard-name table. No external implementation source. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md` v62 and `references/connection-and-recovery.md` | Installed REA workflow instructions, used for bounded target-free Rizin capture, immutable Evidence and cleanup recovery. No toolkit implementation source copied. |

New REA records are saved in `evidence/textfield-vertical/complete.bundle.json`:
the 33-record bundle has SHA256
`8ef653dc667cf0528001009e8c14c05f12666dc82eabf5112442570abaa8cbdb`.

| Record | Scope |
| --- | --- |
| `ev_97da4b4792bbd84d062fd32196c6a9e253b994bfa28f8881f6e5ac845274a766` | Dirty helper, temporary string copy, exact literals and getter/setter dispatch entries. |
| `ev_c7fe30d990afcebf19ba46133d8b3c70283c9524178b5af61a310f12d34570fd` | Correctly mapped standard-member data row. |
| `ev_5a0f4002697ec02a9029471b98100dd31c7c21cc90f89430033f3c1c475c34b7` | Rectangle setter and formatter, 1,358 instructions / 5,402 bytes. |
| `ev_2628f0a8aa938f290cd8ace14be8ade69a4d3a23f52f775532cb762d8535fac5` | Width/height wrappers and exact 40.0 / 0.5 constants. |
| `ev_f42862b0907dc6b66c029dd308b3e1b9e5c82dfa10abbb34737a5e4c0b883995` | Metric-source functions, 0.05 multiplier and IDs41/42 dispatch entries. |

The first record's data-row array used the text-section raw mapping for a data
section address. It is explicitly excluded: the `.data` mapping delta is
`0x1800`, whereas `.text/.rdata` use `0xc00`. The second record corrects that
one read. The original output is preserved; no row claim uses the wrong bytes.

The formatter capture completed but REA reported cleanup incomplete while
checking process ownership. Its record was recovered from the retained bundle
without replay. It has no provider post-run digest; that limitation is retained.
The exact resource cleanup retry succeeded. Independent validation then
byte-matched every instruction to the complete hash-verified executable and
checked the file's hash/stat stability before and after. The other four new
records completed untruncated with unchanged artifact digests. No configuration,
access, process-killing or native-execution workaround was used.

Focused receipts in `evidence/textfield-vertical/` include:

* `saved-path/vertical-path-verification.json`: original source hashes, exact
  function/Construct spans, inherited descriptor route and scoped default search;
  SHA256 `ac5e7ca0b8dd69bc98a8ff1846d09fdcc3725d201243fba1c0fe4a6103febf97`.
* `candidates/member-receipt.json`: independently byte-checked member paths,
  direct data, enabled return precedence and excluded/corrected row.
* `candidates/retained-layout-validation.json`: both complete captured layout
  ranges and four canonical metric/helper ranges compared to the pinned PE:
  **1,412 instructions / 5,600 bytes**, plus four metric/constant data arrays.
* `candidates/helper-independent/helper-and-constants.json` plus the canonical
  metric captures: independent wrapper/constant cross-check.

These are static source/capture checks. No runtime behavior, pixels, native
startup, VM execution or wire parity is claimed.

The aggregate receipt is `evidence/textfield-vertical/verification.json`, SHA256
`d9e159c19d0a29ff36fb4881ee24f32199b59a0c838b363ec53c6b4b87d572c5`.
