# Native HTML wrapping investigation

This increment changes no Rust source and does not relax the existing
`NeedsWrapping` failures. The ordinary ASCII wrapping path is recovered from
Blizz's pinned executable. Its source membership and width rules differ from
trimming text into words or requiring a boundary space to fit.

**VERIFIED static evidence:** named word-wrap accessors, dirty reformatting,
paragraph formatting, ASCII break classification, the strict overflow test,
saved-boundary restoration, line emission, and line-query storage are linked
below. Returned instruction bytes are checked against the original PE file.
This is not a controlled execution of the retail client or a pixel comparison.

An implementation must qualify the ordinary path: an optional callback can
replace its chosen break and insert a hyphen. The exact original frontend
callback state and native font conversion are tracked below rather than
assumed from saved HTML. No browser CSS or host text engine is a reference.

## Actual inputs

The existing placed failures are character-select definitions 246 and 262,
nodes 88 and 84 respectively. Both currently have composed alpha zero in the
saved scene. Definition 163 is another serialized wrapping case, but is absent
from that first-frame placement/error receipt. Absence there does not establish
runtime nonuse.

| Definition | Body scalars | Ordinary spaces | Final spaces | Size, twips | Available width, twips |
|---|---:|---:|---:|---:|---:|
| 163 | 163 | 24 | 0 | 400 | 9,920 |
| 246 | 233 | 34 | 0 | 360 | 9,251 |
| 262 | 242 | 36 | 1 | 360 | 9,920 |

All three have one P/FONT, left alignment, multiline and word-wrap enabled,
zero margins/indent/letter spacing, explicit leading 40 twips and kerning 1.
They resolve `$Strat2Medium` to the exact English library font 4 through the
previously verified explicit host route. All 638 body scalars are ASCII;
punctuation is limited to commas and periods. Spaces are isolated U+0020, with
no leading spaces, tabs, NBSP, explicit breaks or soft hyphens. Every word fits
individually, with maximum length 12. The 22 used kerning pairs are internal to
words; none touches a space.

These originals discriminate useful rules: requiring each ending separator to
fit can produce five lines for 246 where a separator excluded from the fit
produces four. Definition 262 distinguishes keeping its final space from
trimming it. Those initial policy comparisons were explicitly hypothetical;
the executable trace, not their convenient output, selects the contract.

## Verified native chain

| Stage | Native location and observation |
|---|---|
| Named wordWrap accessors | Thunks `0x1402c80e0`/`0x1402c8150`; static initializers `0x141d8c9f0`/`0x141d8ca20` bind targets `0x1402c2ff0`/`0x1402c49f0`. The getter reads DocView byte `+0x149`, bit `0x08`. |
| Enable/disable | `0x1401646e0`/`0x140164700` change that bit and dirty bit `+0x14b & 2`. The text-field construction path at `0x140143880` calls the enable helper from its definition flag. The source wire-to-internal flag conversion is not reconstructed here. |
| Query/reformat | AVM1 `getLineMetrics` at `0x1401d2540` calls `0x140165140`; line offset/text wrappers lead to `0x140165020`/`0x140166420`. These call `0x14016bbf0` when `+0x14b & 3` is set. |
| Content rectangle | `0x14016bbf0` adds/subtracts the byte-verified 40.0 constant on each side of the field rectangle. |
| Paragraph formatting | Fresh paragraphs call `0x14016a9d0`, initialized by `0x14016a510`; the available float width is the inner rectangle's right minus left. |
| Break opportunity | `0x1401645a0`, its range helper `0x140164500`, and exclusion lookup `0x140173fb0`. |
| Overflow/restore | `0x14016a2d0`; saved state copier `0x140167db0`; buffer rewind `0x14014f8f0` and discarded-entry cleanup `0x14014f860`. |
| Line emission | `0x140165490`; distinguishes advanced width from last-nonwhitespace width. |

`F` below denotes the temporary formatter, not a public runtime class proposal.
The field names are analytical labels inferred from the verified data flow.
No native object layout is copied into a Rust runtime object.

## Ordinary ASCII contract

This describes mode `F+0x4a0 == 1`, word-wrap enabled, ordinary embedded text,
and no successful custom break callback. Images, fallback glyphs, tabs, complex
shaping, justification, and overlong words require separate evidence.

1. Before processing the current character, the formatter asks whether the
   preceding/current pair is a break opportunity. For the actual ASCII corpus,
   the only opportunities are immediately **after** U+0020. It saves the whole
   current state at that point, including the already committed separator's
   glyph/source counts and the current iterator position.
2. The previous advance and pair adjustment are combined in binary32, then
   rounded to an integer by widening to binary64, adding 0.5 for positive
   values or subtracting 0.5 otherwise, and truncating toward zero. The same
   signed-nearest rounding is applied separately to the current advance.
   This is per-glyph accumulation; rounding the whole exact line sum is a
   different operation. The constant at `0x141de2da8` is exactly binary64 0.5.
3. The ordinary whitespace classification explicitly includes zero, tab, CR,
   U+0020 and U+3000, with NBSP tracked separately. The overflow test is skipped
   for this whitespace class. For actual U+0020 this means an ending space
   does not itself force a wrap.
4. For a nonspace character, the tentative integer right edge includes the
   current rounded advance and paragraph margins/indent. It is compared with
   the float available width using **strict greater-than**. Equality fits.
5. When it exceeds the width, the optional custom handler is consulted first.
   If that does not take over and a saved opportunity is valid, the formatter
   restores the latest opportunity and discards subsequently staged glyphs.
   The separator remains a member of the preceding line's source range.
6. Before emitting the wrapped line, it marks the retained last glyph with
   flag `0x08`, clears the pending advance/nominal extent and previous-glyph
   pointer, then emits. It resets kerning/previous-character state for the new
   line. Processing resumes at the restored current word without consuming
   that character twice.

For the traced normal branch the exact fit operands are
`float(F[0x6e8] + F[0x6e4] + F[0xec] + F[0xe8])` and
`F[0x490] - float(F[0xf0])`: tentative pen, current fit extent, left/block
margin, indent, available width and right margin respectively. The current fit
extent is zero for ordinary whitespace. The branch is taken only when the
first operand is strictly greater. These analytical offsets document the
data flow; they are not a proposed public API.

For ASCII, `0x140164500` is false. The exclusion table at `0x142adc710` has no
ASCII entries other than a zero sentinel; both applicable exclusion queries
return false for the corpus. The table is **not fully sorted**: it contains an
out-of-order Unicode pair and a final zero. This investigation does not promote
its binary search into a general Unicode line-breaking implementation. The
larger native classifier also mentions hyphen, tab, CR and East Asian ranges;
the saved inputs do not justify enabling those branches in this increment.

## Separator, width and source preservation

The saved snapshot directly retains the already committed source count,
glyph count, last-glyph pointer and iterator. Buffer rewind restores those
counts. Cleanup begins at the first discarded glyph and advances forward;
it does not rewrite the retained separator slot. Consequently the separator
slot can retain an advance previously written while the now-discarded word
was tentatively processed. It is incorrect to claim its stored advance is
always zero.

Soft-break line emission nevertheless clears the pending formatter values
before finalizing width. For left alignment, its reported advanced width
therefore excludes the final boundary space's pending advance. This is a
**width rule**, not permission to remove that scalar, its glyph identity or its
source span. Preserve a separate soft-break boundary/source designation rather
than trimming the source into an unrelated new string.

Ordinary terminal emission takes a different path: it keeps the pending values,
writes the final slot normally, and adds its final advance. Thus an ordinary
left-aligned terminal space is retained in the advanced width. Right/center
alignment uses the separately tracked last-nonwhitespace width; these wider
alignment cases should not be inferred from the three left-aligned originals.
Paragraph terminator insertion by the HTML parser is a separate step and is
not conflated with the original markup's body-space spans.

Native line records have compact/full representations. Compact stores the
start in the low 24 bits of dword `+0x08`, and source length in byte `+0x0b`;
full stores start at dword `+0x08` and length at dword `+0x20`. `getLineText`
returns a source-buffer address with a two-byte stride and the stored length;
its query bodies do not trim, scan or manufacture text. General UTF-16 unit
interpretation across its lookup helper remains inferred, while actual ASCII
characters each occupy one source unit. Source markup offsets and decoded
body offsets must remain distinct in the original HTML representation.

The exact query metrics are width, height, ascent, descent, x and leading.
The AVM1 wrapper scales integer twip values by binary64 0.05. The core computes
descent from height minus ascent through binary32, then truncates to integer.
The compact missing-position sentinel is returned through EAX, while invalid
line lookup returns 64-bit all-ones; these are not interchangeable raw values.

## Leading and placement precision

The emitter uses rounded ascent-plus-descent for height. Explicit paragraph
leading is read from the format and multiplied by 20; otherwise a font-derived
leading value is available. The line step adds its integer height and leading.
The saved 40-twip leading is explicit input, but its complete native loader path
is not claimed from serialization alone.

The previously committed HTML module deliberately used an exact rational Flash
metric model. Native per-advance rounding now supplies evidence that this model
is not native placement parity. Any wrapping implementation must make this
difference explicit and avoid mixing native integer fit widths with unchanged
rational glyph pens while claiming an exact native contract.

## Font normalization and selection

The native formatter expects metrics in a 1,024-unit EM and applies its
size-dependent binary32 multiplier before per-character integer rounding.
REA traced two DefineFont3 loader branches in the original executable:

| Path | Loader | Advance/pair virtual targets |
|---|---|---|
| Ordinary | `0x1400d9000` | `0x1400cec60` / `0x1400d73c0` |
| Compact | `0x1400d5450` | `0x1400d0750` / `0x1400d07e0` |

For an ordinary font with saved layout and a valid glyph, the loader multiplies
the raw advance by binary32 `0.05` and stores the binary32 result. The observed
advance read is unsigned u16. Kerning adjustments are signed i16, multiplied
by the same constant; an absent pair yields zero. The getters introduce no
additional scale. Missing glyph/table fallback is outside this contract.

The compact loader instead truncates the normalized value to an integer,
multiplies it by a nominal-size integer, and uses truncating division by 1,024
before passing it to a packed writer. Its getters normalize packed values
back using binary32 `value * 1024 / nominal_size`. Exact packed writer/lookup
details and nominal-size transfer are still unresolved. This extra stage can
change metrics and must not be silently replaced with ordinary font metrics.

The selector is `0x14006d0f0`. It permits compact loading when optional metadata
reached through context `+0x38`, then `+0x80`, is absent or lacks bit `0x10` at
its `+0x1c`; it also requires a nonnull packing-state pointer reached through
context `+0x18`, then `+0x10`, then `+0x40`. The optional metadata's zero first
dword is treated as absent. Otherwise the selector uses the ordinary path.
**UNRESOLVED:** which path the frontend English font library actually takes,
and its nominal size if compact. Pointer roles are analytical names.

The helper independently matched all 756 saved advances and 5,961 kerning
records in English font IDs 4 and 5 to original bytes. All are multiples of 20;
the actual native binary32 `0.05` multiplication produces exact integral
1,024-unit metrics for these values. This closes their initial normalization
question, but not the compact path or runtime selection. Its full conditional
contract and getter-call limitation are retained in the private font receipt.

## Custom-break qualification

The remaining callback branch is concrete:

- Paragraph setup `0x140169790` sets `F+0x4ac` when the DocView listener at
  `+0x20` exists and its byte `+0x10` has bit 0 set.
- Text-field construction `0x140143880` creates the listener at field `+0x130`,
  with vtable `0x14250a720` and initial flags `0x0e`. It requests state ID 1 at
  `0x140143960`; if that object exists and its dword `+0x18` is nonzero, it sets
  listener bit 0. The listener is installed in DocView at `0x140143ab9`.
- Overflow helper `0x140169920` can invoke the listener's virtual slot `+0x08`.
  That slot resolves to `0x14013efb0`, which requests state ID 1 again, converts
  its width/advance arguments from twips using binary32 0.05, then invokes the
  state object's virtual slot `+0x18` at `0x14013f0a9`.
- On a true result it copies back the selected break position and hyphen flag.
  That result can change the formatting mode and insert a hyphen. On missing
  state or a false result the delegate returns false.

**UNRESOLVED:** the concrete state-ID-1 instance, capability value and method
used by these original frontend movies. The initial `0x0e` flags alone do not
prove the custom path is disabled. Neither English source text nor its absence
of literal hyphens proves that the callback cannot choose another break.
State ID 1 is an observed numeric selector; describing it as a translator is
an analytical role, not a recovered runtime type name.

## Conditional simulation, not native expected results

An original private simulation applies the ordinary branch to the exact saved
body/source records, under the metric and iterator assumptions in its receipt.
A separate word-based calculation matches all 11 ranges and widths. These are
**VERIFIED simulation-consistency checks**, not outputs observed from retail.
They must not be installed as native golden tests before qualification.

| Definition | Decoded body ranges → advanced width, twips |
|---|---|
| 163 | `[0,50)` → 8,437; `[50,107)` → 9,469; `[107,163)` → 9,094 |
| 246 | `[0,61)` → 9,211; `[61,118)` → 8,378; `[118,180)` → 9,005; `[180,233)` → 8,188 |
| 262 | `[0,67)` → 9,848; `[67,128)` → 9,076; `[128,186)` → 8,603; `[186,242)` → 8,466 |

All 638 original body units occur exactly once. Each soft-wrap separator remains
in the preceding range; 262's terminal space contributes 78 twips under the
assumed font path. Three boundaries overflow at the first character following
the separator. Their retained slot has no preceding finalization write in this
model, so its advance is explicitly unknown. Other retained separator slots
can retain a previously written advance even though final line width excludes
the pending separator. Downstream consumption of flag `0x08` is not recovered.

## Remaining qualification and implementation hold

Before replacing `NeedsWrapping`, select the callback and font-loading profiles
from evidence, and establish the paragraph iterator's injected terminal-unit
handling. General Unicode indices, source-terminal metrics, overlong words and
downstream soft-break glyph flags remain outside the verified path. Full
baseline/leading parity also needs the native font metrics, not only line-fit
widths. No runtime, raster or native-layout comparison was run.

The assigned scope permits documentation and private evidence only until that
native profile is established. This increment changes no source and makes no
font, wrapping, visibility or alpha fallback. Existing source HTML and typed
wrapping errors remain available.

## Evidence and provenance

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/`.

- `html-wrapping-study/full-records.json` preserves all 638 source scalars,
  spans, metrics and 22 pair adjustments. `sources.json` records exact inputs.
- `html-wrapping-study/source-free-summary.json`, SHA-256
  `82f78ee4a56fda5586d6aef557ac4cd19ea4506a97aebe6bf4f38e5bb493c121`,
  records the corpus and distinguishes hypothetical policy comparisons.
- `html-wrapping-study/helper-native-lines.json`, SHA-256
  `d340512db91507d1675b6f453283f570c21f16f5759e464e745a4638a2f9159f`,
  independently verifies query data flow and its limitations.
- `html-wrapping-study/helper-wrap-state.md` and accompanying byte receipt
  independently check snapshot, source membership and emission behavior.
- `html-wrapping-study/native*.json` retain complete REA results. Corresponding
  assembly and byte-check receipts keep raw evidence separate from analytical
  field names. `classifier-table-check.json` preserves its ordering caveats.
- `html-wrapping-study/helper-conditional-wrap.json`, SHA-256
  `03ef580b1ae2e587ba5498d3ed77607b66e61158f40de93af4e03a1d686b899e`,
  preserves the conditional model; `helper-conditional-wrap-consistency.json`
  records its independent 11/11 range/width comparison.
- `html-wrapping-study/rea-evidence.json` exports 15 records: the 12 calls in
  this investigation and three preceding HTML-format calls. All 12 current
  calls report exit 0, complete output, no truncation and unchanged artifact
  digest. Independent checks match 5,562 returned instruction encodings,
  including overlapping ranges. Byte equality is not instruction alignment or
  proof of complete function coverage: the unaligned native8 part 4 and tails
  outside cited function ranges are excluded from semantic claims.
  Bundle SHA-256 is
  `bda873e50b3aeeb03801d3437a0ad0e52a547b680a307e9c0735c8e2f73509c0`.
- `html-wrapping-font/FINDINGS.md`, SHA-256
  `8574f645bd8b72eb2a91fd4cf22035ed526d386440c711b6af876380d50ddc79`,
  and `MANIFEST.json` retain the font branch, exact conversions and source
  checks. Its separate six-record `rea-evidence.json` has SHA-256
  `c7d7cee526c3d252cb90d0b70a96cdcef5a7bf7b0e716e85fd03a572970eb71f`.
- `html-wrapping-study/verify_receipt.py` and `FINAL-RECEIPT.json` record a
  fresh owner-binary digest, all 12 independent instruction-byte comparisons,
  bundle membership and all 32 font-manifest file hashes. No Rust build or
  gameplay check is appropriate to this documentation-only change.

The font helper's getter call reports `cleanup_incomplete`: REA's ownership
scan could not read an unrelated `sshd-session` process environment. Exported
Evidence retains complete, untruncated exit-0 output but lacks REA's post-run
digest. Its instruction encodings and a separate current executable hash were
checked independently; neither establishes provider cleanup. The helper ran
`rea doctor`, preserved its existing optional-engine/configuration warnings,
and did not retry the call, reset, kill processes or modify shared settings.

All native calls use the owner-pinned executable under
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It is proprietary owner-provided material, used only for compatibility study.
No binary, assembly listing, extracted body text, font table or capture enters
git. Outside sources read or reused for this increment are recorded here for
the integrator's provenance ledger:

| Source | Licence and use |
|---|---|
| Owner's pinned `MarvelHeroesOmega.exe` at the path and SHA above | Proprietary owner-provided artifact; static native wrapping, font loading and numeric metric evidence. |
| Private `evidence/upk-rust-samples/marvel_character_select.gfx`, SHA `a12781e8109d682a17dcfb11e622700f6ce33522a989e17b42b2a01137eb2034` | Proprietary original movie; saved definitions and source spans, through project receipts with original byte checks. |
| Private `evidence/frontend-fonts/MarvelFonts_SF-fonts_en.gfx`, SHA `330b919bcbabe8612db9c2140a8f327a663362fd7e1aaa28a215c702cae7032a` | Proprietary original movie; exact advance/kerning values checked directly against source bytes. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/native-and-artifacts.md`, `references/connection-and-recovery.md` | Installed REA workflow documentation, licence not independently determined; tool selection, evidence preservation and failure handling. |

All remaining references are this project's original source or private
analytical receipts, listed in `html-wrapping-study/sources.json` and the font
receipt. The earlier Adobe references remain in `HTML_TEXT_LAYOUT.md`; this
investigation does not derive native wrapping from browser behavior or an
outside text-engine implementation. No native application, debugger inferior,
server login, Ghidra binding, configuration change or approval prompt was used.
