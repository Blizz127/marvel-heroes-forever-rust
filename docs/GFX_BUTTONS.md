# Button records and observed frontend controls

## Actual scope comes first

**OBSERVED:** None of the 20 pinned frontend movies or
`MarvelGame-frontend_manager.gfx` contains DefineButton (7), DefineButtonSound
(17), DefineButtonCxform (23), or DefineButton2 (34). Independent Python scanning
and the original Rust movie reader agree on 35,990 tags. The independent scan
consumes 1,873 sprite definitions and all 1,894 root/sprite streams exactly.
Versions 10, 11 and 15 occur. This corpus provides **no actual button-record
grammar validation**; successful movie parsing does not change that limit.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/gfx-buttons-study/`.
`button-scan.json` contains every input path/hash and per-movie count; its SHA-256
is `7f8883db167d4f5e3ec98d0a9c2d5b8b1c01d5d40ee76d51603fd15a71819825`.
`scan_buttons.py`, `FINDINGS.txt` and `sources.json` retain the independent method,
format notes and source ledger. The Rust inventory is `rust-smoke.tsv`.

**OBSERVED (existing original AVM1 decoder, no execution):** The movies instead
contain 332 serialized clip-action handlers: 331 with Construct flag `0x40000`,
and one with MouseMove flag `0x8`. There are no serialized Press, Release,
ReleaseOutside, RollOver, RollOut, DragOver or DragOut handlers in this sample.
The MouseMove handler belongs to `cursor_mc` in the manager movie, tag index 375,
absolute action span 7,096 + 77 bytes.

Named placement occurrences include `hitArea` (18), `hitArea_mc` (13) and
`m_hitarea` (1). Frame-label occurrences include `up` (153), `over` (153), `down`
(149) and `disabled` (170). These counts are serialized occurrences, not unique
runtime instances. Literal strings and export names include `MarvelButton` and
`gfx.controls.Button`; pools also contain names such as `onPress`, `onRelease`,
`onRollOver` and `onRollOut`. Their presence does not establish that execution
assigns or invokes those handlers.

For a concrete scene reference, `marvel_login_dialog` places `loginButton_mc`
with character ID 170 on timeline 219 at tag index 1345, body span 39,192 + 497.
The same timeline contains `verifyEmailButton_mc`, `registerNowButton_mc` and
`forgotPasswordButton_mc`. The source spans, timeline IDs and names are in
`controls-records.json`; the reproducible original harness is `controls.rs` and
its output is `controls.tsv`. All action/clip streams decode to 23,255 blocks and
779,489 instructions with no unknown opcodes or trailing bytes. This is static
decode coverage, not runtime/control parity.

**INFERRED:** The observed sprite timelines and class references make script
registration and MovieClip event hosting the next control-integration work.
They do not justify inventing DefineButton nodes, hit-test behavior, active
states or handler reachability. The scene and event-host lanes receive these
source references separately.

## Retained structural reader

`src/gfx/buttons.rs` was already implemented when the zero-button inventory
completed. It is retained as an original, bounded reader with synthetic coverage;
it is not a dependency gate for rendering these frontend movies.

`Button::parse(tag_code, body, body_offset, swf_version)` reads tags 7 and 34.
It retains the identifier, tracking flags, declared action offset, display
records and action spans. Each `ButtonRecord` exposes character ID, depth,
matrix, optional color transform, filter payload spans and raw blend mode.
`in_state(ButtonState::{Up, Over, Down, HitTest})` reads its state mask without
choosing a runtime state. All source spans address the complete original movie.

`ButtonAction` distinguishes the legacy release action from condition records.
It preserves the declared size, raw `ButtonConditions` word and complete action
span including its final zero. Conditions expose individual transitions and a
separate key code. Record offsets delimit bytecode; the reader does not scan for
the first zero byte inside operands. It checks the final terminator but leaves
instruction and nested-function validation to `avm1::Program::parse_at`.

`ButtonCompanion::parse(tag_code, body, body_offset, swf_version)` reads RGB color
companions (23) and four sound-transition descriptors (17). Sound metadata retains
zero IDs, optional style fields, loop counts and envelope points. No sound is
opened or played, and no companion is automatically attached to a character.

Bodies are limited to 8 MiB, display records to 65,536, condition records to
65,536 and filters to 65,536 per definition. Source-offset overflow, truncated
fields, invalid relative offsets, missing terminators, reserved structural flags
and trailing bytes outside action blocks fail. Unsupported tag/version/effect
profiles return errors. Empty display arrays and otherwise uninterpreted numeric
values remain structural data; this is not a complete SWF semantic validator.

**VERIFIED (synthetic only):** Five focused tests cover signed transforms,
state masks, filter spans, blend metadata, conditional/key bits, legacy versus
modern actions, color/sound companions, every truncation of representative
records, malformed offsets/flags, version checks and bounds. A separate invented
button action span passes to the original AVM1 decoder at the correct absolute
offset. `focused-tests.txt`, `check.rs` and `rust-smoke.tsv` record these checks.
No event dispatch, drawing, audio playback, script execution or retail comparison
was performed by this reader.

## Source and licence register

| Source read | Licence / rights | Use |
| --- | --- | --- |
| Adobe-authored [SWF File Format Specification v19](https://open-flash.github.io/mirrors/swf-spec-19.pdf), button chapter printed pages 195–200, sound style pages 180–181, clip-event pages 48–50, and framing/basic types | Copyright 2006–2012 Adobe, all rights reserved; documentation study only | Published button/condition/companion framing and clip-event flag interpretation. No external parser or VM implementation copied. |
| Cached matching PDF, `evidence/gfx-fonts/adobe-swf-spec-19.pdf` | Same Adobe document; SHA-256 `6a1625b0929a86329021d365ba079c7e1f51502dfb4ea7d92ec9947841625cdd` | Independent local specification reading. Exact physical page list is in `sources.json`. |
| Existing cached pypdf 7.0.0, `/home/blizz/.cache/uv/archive-v0/DqHvabedjSAVs2kC/pypdf`; licence in adjacent dist-info directory | BSD-3-Clause | Private PDF text extraction utility only; no source read/copied or client dependency added. |
| Twenty `evidence/gfx-samples/*.gfx` movies and `evidence/frontend-startup/MarvelGame-frontend_manager.gfx` | Proprietary owner game assets; private interoperability study | Exact tag inventory, named placements/labels, clip flags and static action references. No source movie, bytecode or extracted data enters git/builds. |

The original project `gfx` matrix/color/filter helpers and `avm1` decoder are
reused; their existing provenance records apply. Search discovery returned
third-party summaries and parser type names, but no external parser source was
opened or used. The integrator should link this ledger from `docs/PROVENANCE.md`.
