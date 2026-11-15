# Definition-derived format inputs for the login label

**VERIFIED static contract:** a successful, nonempty, no-markup HTML
replacement through the retained no-service path derives fresh formats from
the attached field's definition. For the actual UISource EditText **1034**, the
numeric character and paragraph inputs below are now grounded through the
native loader, parser and fresh run storage. This is more specific than copying
saved defaults and does not resolve a live native font or prove pixel parity.

The scoped instance is diagnostic login node 89, below parent 86, with source
movie UISource (header version 11). Node 136 shares the definition but is a
different instance. The prior controlled VM fixture retains root environment
version 15 and separately supplies its selected-definition binding profile;
see [TextField host](TEXTFIELD_HOST.md) and
[source/environment identity](TEXTFIELD_SOURCE_ENV_VERSION.md). None of these
numeric IDs identifies a retail runtime object.

## Required route and state

This contract requires the actual definition to be attached to the selected
instance, ordinary successful resource/container allocation, and an input that
takes the replacement branch of `0x1401463b0`. Equal cached input with force
bit 16 clear returns before rebuilding. Public HTML enable, requested parse
mode, last input, current document, document runs and current defaults remain
separate state.

The profile explicitly excludes applicable watches, stylesheet transformation,
the root state-1 translation/formatter route, editing/variable callbacks and
image substitution. Parser flags and effective target/view state must be
supplied. The saved tag alone does not establish any of those absences. The
selected input is nonempty printable ASCII without markup, entities, CR/LF or
NUL; no claim is made here about other text. The original localized write of
`LOG IN` fits that input shape, but this study did not execute it again.

In particular, the formatter has an optional provider reached from DocView
`+0x30` at `0x14016ae31`, then `0x1401666d0`; a nonnull result selects its
alternate content path. Its runtime absence or inapplicability is part of the
explicit profile, not a property of a DefineEditText record. Missing glyphs,
fallback fonts, selection decorations and later script format mutations are
outside the mapping below.

## Saved bytes through the native definition

**OBSERVED:** the original UISource GFX has SHA-256
`49ff476cb0316f22d43b9632bd5260141587ecd32a54b4d58a5dabfb3a1548f5`.
EditText1034 is tag 37 at record span `224137+47`, body `224143+41`;
body SHA-256 is
`90eb2fb234f3dc76a3bdecf418264235806cabeb4cf9373729809d717249d786`.
The private independent decoder checks each retained field against its span.

The selected native wrapper `0x140093ff0` reads a character ID, allocates a
definition, calls constructor `0x140093f30` and reader `0x140093380`, then
registers the definition with that ID. This establishes the reader's concrete
data flow; the study does not claim that a live frontend invoked that wrapper.
Here `D` denotes its definition, `A` its text format, and `B` its paragraph
format. These analytical names do not recover C++ type names.

| Saved input | Native definition field | Derived format and renderer value |
| --- | --- | --- |
| Font height 320 twips | UI16 converted directly to f32 at `D+0x50` | `A+0x3e=320`, size present; `height_twips=320` |
| RGBA `[254,240,214,255]` | RGBA reader stores packed ARGB `0xfffef0d6` at `D+0x54` | `A+0x38` retains that value; renderer RGBA unchanged |
| Alignment selector 2 | `D+0x84=2` | B alignment bits `0x600`, native selector 3, center; renderer alignment 2 |
| Left/right margins 0 | f32 `D+0x5c/+0x60` | `B+0x16/+0x18=0`; renderer margins 0 twips |
| Indent 0 | signed UI16-sized read converted to f32 at `D+0x64` | `B+0x12=0`; renderer indent 0 twips |
| Leading 40 twips | signed i16 converted to f32 at `D+0x68` | `B+0x14=2` native pixel units; renderer leading 40 twips |
| Local font ID 270 | ID at `D+0x30`, resource pair at `D+0x20/+0x28` | Requires the selected resource/binding contract below |

`0x140141640` resets A/B and performs these assignments. Layout values use
binary32 constant bits `0x3d4ccccd`, followed by integer truncation and word
storage. The listed zeros and 40-to-2 conversion are exact for this source;
this is not a general conversion or saturation rule. The size branch also has
a specific unsigned-low32 comparison against `0x10000`; the actual 320 takes
its ordinary in-range branch. Font EM units and font-provided leading are
different quantities from the saved height and paragraph leading.

Saved flags `0xad31` become native definition flags **`0x20a`** under this
reader: multiline, read-only and has-layout. It inverts saved NoSelect into a
native selectable flag and saved UseOutlines into a device-font flag. The
saved HTML, wrapping, autosize, password, border and device-font flags are
clear. These initial mappings do not establish current mutable layout flags;
public `html=true` also does not rewrite the definition's HTML flag. Do not
reinterpret native target/view bitfields as the wire `EditFlags` value.

## Why the previously unknown effects are inactive

The new evidence distinguishes reset values, presence bits and actual storage.
It does not infer a false/zero effect from its absence in the saved record.

1. Text reset `0x1400ba990` clears the low four style bits at `A+0x40`, stores
   zero in spacing word `A+0x3c`, clears spacing-presence bit `0x2` and supplies
   underline/kerning presence bits `0x40/0x80`. Definition-derived font style
   may then set bold and italic, but does not enable underline or kerning.
   Paragraph reset `0x1400bac00` zeroes its five numeric words, including
   block-indent `B+0x10`; block-indent presence bit `0x2` remains absent.
2. HTML wrapper `0x1401648f0` clears paragraph storage. `0x14012ed30` calls
   `0x14012ce10` with requested count zero, and every successful return from
   that helper writes count zero. Parser `0x140176090` copies supplied A/B;
   its empty-URL setter `0x1400b9ce0` affects the URL string/presence, not the
   numeric fields or four style bits. With no element stack, token 3 passes
   these A/B directly to append `0x14012dc50`.
3. First paragraph creation `0x14012d3d0` zeroes its existing format and run
   metadata. `0x140127760` therefore takes the direct B factory path, rather
   than merging an old paragraph format. B copy `0x1400baf80`, table helper
   `0x1400bacc0` and equality predicate `0x1400bbb20` preserve all five numeric
   words. The equality predicate checks them even where presence is absent.
4. Empty run-array insertion `0x140127ba0` returns without creating a run.
   Iterator `0x140127f50` cannot select an old slot with count zero. Therefore
   `0x14012c530` takes the direct `0x1400bd870(A)` path, avoiding the
   existing-run overlay. A copy/equality helpers `0x1400baed0` and
   `0x1400ba0e0` retain/check spacing, size, color and style bits.
5. `0x14012b910` forwards the fresh record to `0x14012b340`. Under successful
   container growth, stores at `0x14012b3ed`, `0x14012b3f0` and
   `0x14012b403` write start 0, length N and the exact canonical A pointer.
   Thus the actual inserted characters `[0,N)` have the derived format.
   Native allocation failure and corruption behavior are not emulated here.

Retained formatter evidence closes the consumers as well:

| Runtime style input | Native consumer | Supported value |
| --- | --- | --- |
| `effects.underline` | `0x14016b8e6` reads style bit 2 | `Some(false)` |
| `effects.kerning` | `0x14016b969..973` reads style bit 3 | `Some(false)` |
| `effects.letter_spacing_twips` | `0x14016ba75..7d` skips the spacing addition when presence bit 1 is clear | `Some(0)` |
| `effects.block_indent_twips` | `0x14016acb2..cc2` adds `B+0x10` to left margin, then multiplies by 20 | `Some(0)` |

Those values apply to the exact fresh replacement route. A merge into existing
runs, another parser path, later format mutation, or a supplied custom format
does not inherit this proof. A font's kerning table does not enable kerning;
the English candidate contains 2,871 pairs, including relevant letter pairs,
while this fresh format explicitly leaves kerning disabled.

Nonempty parser completion installs resulting current defaults separately.
The renderer must bind the document's **run-format pair**, not substitute its
current defaults or last-input cache. Canonical native allocation addresses
are not the project's opaque `TextFormat`/`ParagraphFormat` identities. The
host must associate the exact pair and domain with this qualified mapping.

## Font identity and remaining runtime inputs

Native target constructor `0x140143880` receives the definition separately
from its selected MovieDef. It saves `D` at target `+0xd8` and derives the
font/resource binding table at target `+0xe8` from
`[selected MovieDef+0x38]+0x30`. Factory case 4 forwards CreateInfo's definition
and selected MovieDef separately. This is not the current script environment,
the placing movie's name, or an arbitrary global font library.

The definition reader obtains a resource pair from lookup `0x14008bd60` in
the load state's ID registry. In `0x140141640`, pair discriminator 1 resolves
its index through the target binding table; discriminator 0 uses its pointer.
A nonnull resource whose virtual category masks to `0x200` supplies font F
from `resource+0x18`. F's virtual name is copied into A. For this definition's
clear saved HTML bit, F's `+0x1c` italic/bold bits set A's corresponding style
bits. Ordinary and compact font loaders both derive those bits from the two
low saved font flags. The inspected English candidate's flags `0x8c` leave
both clear; its name containing “Bold” does not authorize synthetic bold.

If definition device-font bit 8 and F bit 6 are clear, the helper creates an
embedded-font wrapper containing F and the selected binding context's `+0x48`
object, then stores it in `A+0x30` with presence `0x800`. **This pointer is not
guaranteed to survive interning:** on a new text-format copy,
`0x1400bd870` clears that pointer and presence bit when allocator-context
`+0x80` bit 0 is set. It preserves the numeric/effect fields. Cache hits use
the previously canonical object. Later formatter font resolution at
`0x140169290` remains a distinct runtime operation; name equality alone does
not establish its result. Failed initial resource lookup also cannot be
reported as selection of the saved FontID; reset has a fallback face name.

**OBSERVED saved route:** UISource FontID270 imports `$Strat2Bold` from the
logical font library. The explicit English alias resolves it to font ID5,
Stratum2 Bold. The Russian library exports the same symbol to Cervo Medium.
The alias, language, live resource-pair substitution and final selected font
remain supplied conditions, not facts established by the EditText source.

The minimum remaining inputs for a renderer binding are consequently:

- Exact live instance, selected-definition/resource binding and opaque
  document run-format pair/domain, with the replacement/profile conditions
  above. Source/movie names alone are insufficient.
- Effective layout flags and bounds after any relevant runtime mutations;
  current public HTML/parse bits are separate. The saved bounds are
  `[-40,1920] × [-40,440]` twips, but later sizing and vertical recentering
  are separate contracts.
- An authenticated final `RuntimeFont` identity, including declared local
  import, selected font movie/character and export route. Supplying English
  is a controlled choice. It must cover retained/fallback wrapper handling,
  applicable font substitutions and availability of the required glyphs.
- A declared font metric/loading profile. Ordinary versus compact loading,
  compact nominal size and native metric/raster rounding remain the limits
  in [HTML wrapping](HTML_WRAPPING.md). The derived formatting inputs do not
  turn the current rational glyph layout into native placement parity.

For the exact qualified definition, height, RGBA, paragraph values and the
four inactive effects no longer need arbitrary defaults. Their derivation
still does not authorize automatic admission of unsupported runtime state.

## Evidence, verification and provenance

All evidence paths below are private, relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`. No native instructions,
asset bytes, font tables or captures were added to git.

- `textfield-format-mapping/saved-definition/`: original independent parser,
  source ledger, 13 exact source field spans and both font candidates.
- `textfield-format-mapping/reuse-ledger.json` and `final-branch-check.json`:
  retained evidence identities, bounded branch analysis and successful
  allocation/container conditions.
- `textfield-format-mapping/native/final.bundle.json`: six new complete REA
  records, with original full results and readable navigation files beside
  it. `native/verify.py` independently checks instruction/data bytes through
  the PE section mapping; `native/verification.json` retains the counts and
  results: 2,456 new instruction rows, 128 data bytes, 36 selected ranges and
  8,956 distinct address bytes match the pinned PE. Four reused loader and
  formatter records are checked separately. Their four incomplete tail rows
  are excluded from decoded-instruction claims and from the selected header
  semantics. Initial short captures are identified as fragments, not complete
  functions; later captures extend the relevant bodies.
- `textfield-format-mapping/receipt.json` and `MANIFEST.json`: final source,
  tool-result and analysis hashes. No native process, VM, renderer, server or
  gameplay check ran for this documentation increment.

The owned executable is
`pins/alienware-candidate/UnrealEngine3/Binaries/Win64/MarvelHeroesOmega.exe`,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
New calls were bounded, target-free REA/Rizin disassembly/data reads. Each
reported exit 0, complete untruncated output and unchanged artifact hash.
REA's provider identity and raw-mapping limitations remain in the results;
independent PE byte agreement establishes the inspected encodings, not
execution. The six-record ledger was exported and reset without an active
binary binding. No shared configuration or native prompt was used.

| Source read or reused | Licence and use |
| --- | --- |
| Owner's pinned executable, UISource GFX and English/Russian font GFX files; exact paths/hashes in the private source ledger | Proprietary owner-provided artifacts; static compatibility and saved-definition study only |
| Original project `TEXTFIELD_RUNTIME.md`, `TEXTFIELD_SOURCE_ENV_VERSION.md`, `TEXTFIELD_VERTICAL_LAYOUT.md`, `HTML_TEXT_LAYOUT.md`, `HTML_WRAPPING.md`, `RUNTIME_TEXT_PREPARATION.md` and their cited private receipts | Original project analysis; reused native identities and scoped contracts, no outside implementation |
| Original `src/frontend/scene_content.rs` and retained actual-host fixture | Original project API/context study; no product changes |
| Installed `reverse-engineer-anything/SKILL.md`, `references/evidence-workflows.md`, REA tool schemas | Installed workflow documentation; licence not separately inferred for the guides. REA tooling is MIT as recorded in `PROVENANCE.md` |
| Rizin through the registered REA provider | Analysis tool, installed header SPDX LGPL-3.0-only as recorded in `REA_NATIVE_ALTERNATIVE.md`; bundled dependency licences not audited, no implementation copied |

The earlier Adobe SWF references remain credited by the format/parser docs.
This increment uses the exact owned-binary contract for derived native state,
not an SDK implementation or an assumed browser text engine.

## Original conditional scalar helper

`frontend::textfield_format::derive_fresh_html(&EditText, FreshHtmlInput)`
returns a `runtime_text::FormatBinding` for the **actual document run pair**.
It performs no host mutation, asset lookup, format interning, setter invocation,
namespace installation or rendering. It requires the following explicit inputs:

- `definition` and `domain`: the selected source reference and the existing
  format-association domain. The field ID, text kind, bounds and source-span
  arithmetic are checked. The caller still establishes the actual Scene/VM
  lifetime and that this parsed field is the selected definition.
- `snapshot`: current document text/run pair, last input, raw target/view flags
  and definition HTML profile. The document must be nonempty printable ASCII
  without markup/entities, equal to the last input, and use the definition HTML
  run pair. HTML enable/parse bits must be set; parser bits and pending forced
  rebuild must be clear. Current defaults may have a different handle pair.
- `conditional`: explicit successful fresh replacement, successful ordinary
  allocation, selected-definition and no-later-format-mutation assertions.
  Every existing side path and the separate DocView content provider must be
  explicitly absent; unknown and present reject. These assertions are not
  derived from matching text, saved fields, invalidation bits or numeric IDs.
- `font`: the declared `RuntimeFont`, an independently supplied final selected
  `FontIdentity`, explicit final-selection assertion and a known retained or
  normalized/reselected wrapper state. The source local movie/FontID and the
  declared resolved identity are checked. Unknown wrapper/selection and
  effective bold/italic reject. Matching the local reference **does not prove
  native font selection**. A supplied English choice remains conditional.
- `layout`: explicit effective flags and bounds. Bounds must equal the parsed
  saved bounds because the renderer currently retains them. Unsupported
  font-class/device/auto-size/password/wrapping/static profiles reject. The
  recovered raw DocView wordWrap bit also must be clear, so an incompatible
  raw state cannot be hidden by the supplied flag value. Accepted flags are
  copied exactly; no public HTML or native target bits are cast into them.
- Bounded, nonempty `provenance`, retained with a conditional-mapping prefix.

The source requires an explicit FontID, height, RGBA and layout. Missing fields
do not select native reset defaults. This increment excludes authored HTML,
FontClass/device-font/was-static definitions and nonempty variable names. It
derives exact height and RGBA, maps left/right/center alignment, and supplies
the four proven inactive effects. It never takes these values from current
defaults or accepts arbitrary caller-provided effect overrides.

Paragraph conversion uses the actual binary32 multiplier `0x3d4ccccd` and
truncation, then converts the native integer pixel value back to twips. As an
explicit bounded implementation policy, a source value that changes through
that conversion returns `Error::Quantization`; it is not silently rounded.
Thus admitted margins are multiples of 20 through 65,520, native nonnegative
indent through 32,760, and signed leading from -32,760 through 32,760. The
native reader interprets the serialized indent word as signed; a negative
value rejects because the current renderer's indent is unsigned. Justified
and unknown alignments reject. Height is an exact UInt16 input, including zero;
zero-height rendering behavior is left to the existing layout API.

All unsupported checks precede binding construction. Text inspection is
bounded by `MAX_TEXT_BYTES`; the only new allocation is provenance, bounded
including its prefix by `MAX_RUNTIME_PROVENANCE_BYTES` and using `try_reserve`.
Errors leave the input field and snapshot unchanged. No external access,
recursion or mutation occurs. The downstream renderer must still verify the
font route, decoded font, glyphs and layout support. Font loading/rounding and
native pixel-parity limits above continue to apply.

Focused validation is retained privately in `textfield-format-helper/`.
Seven module tests cover run-versus-default identity, native flag separation,
every side-path/condition gate, stale or malformed documents, font selection,
bounds, signed indent/quantization and allocation-size admission. An original
independent numeric oracle checks 33 hand cases and the complete serialized
16-bit numeric domains; it distinguishes native conversion from the helper's
stricter no-quantization/unsigned-indent policy. No full project suite or native
client was run. The exact actual-source probe and final build/source receipts
are recorded with that private evidence; they are controlled composition,
not discovery of a retail font, locale or runtime profile.

The actual-source probe checks 13 fields of the unchanged UISource definition,
the explicit English candidate and all six supplied label codepoints. One
positive mapping produces the native-derived 320-twip size, saved RGBA, center
alignment, 40-twip leading and four inactive effects. Four negative cases reject
a wrong final font, missing final-selection assertion, changed bounds and
unknown wrapper state. Field, snapshot and input files remain unchanged.
`LOG IN` is supplied post-replacement state in this probe; no saved script or
VM is executed. The earlier original-script evidence remains a separate check.

## Original-script to helper to CPU composition

**VERIFIED controlled composition:** a separate private run on frozen root
`466c18f3e377abd2a63f1831fb989b8f99e24189` replaces the previous manual
`RuntimeTextStyle`/`FormatBinding` construction with `derive_fresh_html` applied
to the **live post-script host snapshot**. The original selected caller,
declarations, constant pools, wrappers, source files and scene recipe remain
unchanged. The previous `7041d19` fixture and its sealed results were preserved.

The unchanged `updateLocaleTextFromLabel` route executes original
`updateLabelText` body `94114+178`, including the true uppercase branch through
the native-derived String method at source offset 94251. The actual
ConfiguredLabels/English LocaleCatalog backend and controller cache retain
`Log In`; the original HTML setter at 94201 and htmlText setter at 94252 produce
the live document `LOG IN`. This run executes **142 instructions and 3 host
calls**. The controller's supplied vertical-center flag remains false, so the
unchanged recenter method takes its existing supported branch. No additional
class initialization or native lifecycle claim is introduced.

The bridge passes the live snapshot and its admitted side-path profile into
the helper. The run pair differs from current defaults and remains the selected
pair. The helper derives the source's 320-twip size, RGBA, paragraph values and
inactive effects; the fixture no longer builds those style fields manually.
Font import routing is checked through the actual library, while final font
`22:5`, retained wrapper, effective flags `0xad31`, bounds, absent content
provider and fresh/no-later-mutation conditions remain **explicit conditional
inputs**. Neither the alias nor this successful composition observes native
font, locale, wrapper or effective-state selection.

The resulting override produces **six CPU glyph placements** for node 89.
All **32 other text instances**, including shared-definition neighbor 136,
retain their prior prepared fields, layouts, issues and glyph geometry. Host
snapshots, statistics, traces, storage and pending invalidation remain unchanged
by helper/CPU preparation. All saved Scene nodes, action metadata and movie
bytes are preserved. The original post-call checks also preserve the nonempty
Selection listener and class-registry controls. No visibility setup, GPU draw,
native process or server operation was added to this run.

Private evidence is under `textfield-format-post-vm/`. Its source archive has
209 files matching the exact commit. The four copied caller/script/scene files
are byte-identical; only the private `vm_bridge.rs` imports, format construction,
output destination and diagnostics change. The execution receipt checks 217
source/harness/lock/metadata files before and after. The separate audit checks
the prior sealed files and original declaration/pool dependencies, and requires
the final case-completion marker after the CPU success marker.

The CPU build uses `nice -n 19 cargo build --offline --locked -j2`. All ten
external dependency records exactly match the frozen root lockfile. Copying
the complete root lock plus the private package initially caused a preserved
`--locked` rejection; the accepted fixture lock contains the exact recursive
CPU dependency subset, with no unlocked resolution or version/checksum update.
The earlier GPU attempt and its dependency-drift records remain untouched.
`run-receipt.json`, `audit.json`, `receipt.json` and `MANIFEST.json` retain the
commands, exit codes, outputs, assertions, source hashes and limits. Both the
build and the complete fixture exited zero; no full test suite was run.
