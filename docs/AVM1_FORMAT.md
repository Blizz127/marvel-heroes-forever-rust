# AVM1 structural decoder

`src/avm1.rs` is original, dependency-free Rust. It decodes action records and
nested code; it does not execute ActionScript, construct controls, open URLs,
call native functions, or establish frontend behavior parity.

## API

- `Program::parse(bytes)` reads a standalone action stream.
- `Program::parse_at(bytes, source_offset)` preserves absolute movie offsets.
- `Program::parse_with_limits(bytes, source_offset, limits)` sets resource caps.
- `Program::instruction_at(offset)` finds an exact decoded boundary, including
  nested bodies. This is source lookup, not permission to cross function scopes.
- `Program::coverage()` recursively reports opcodes, unknown operations, block
  nesting, unparsed trailing bytes, and branch destinations.

For an existing `gfx::TagData::Actions` or `gfx::ClipAction` range, pass
`span.bytes(movie.bytes())?` and `span.offset`. This module has no dependency on
`gfx`, so it also accepts button action streams once their envelopes are decoded.

An `Instruction` separates its header/payload `record`, declared `payload`, and
whole `span` including nested bodies. `Operands` has typed forms for all long
opcodes defined in Adobe's action specification. Short actions expose their
opcode/name with `Operands::None`. Unknown opcodes retain their bounded payload
span and `Operands::Unknown`; no invented semantics are attached.

`ByteString` preserves bytes and offers fallible UTF-8 interpretation. Numeric
literals preserve their bits, including NaNs, negative zero and the unsigned
32-bit storage of integer literals. Boolean bytes are also preserved without
normalization. An unknown Push type retains the remaining payload as one opaque
span, since its value length is unavailable. Decoding resumes at the next action.

`Literal::constant(pool)` resolves an index only against the caller's explicitly
supplied pool. The decoder does not assign a pool based on sequential source
order: branches and function lifetimes affect which pool is active. Out-of-range
indices return `None`; they do not cause an unchecked lookup.

Each `Branch` retains its signed displacement, absolute destination and local
boundary classification. Exact block ends are distinguished from instruction
boundaries, mid-record/trailing positions and exits from the immediate block.
Those classifications are observations, not an execution validator. In
particular, a TryBody may jump past CatchBody to its enclosing stream.

`Program` includes an End action if present and preserves any later bytes in
`trailing`. Sized nested blocks need not end with End. The root decoder does not
require a terminator because it also accepts sized substreams; callers inspecting
DoAction/DoInitAction/ClipAction must check `terminated` themselves.

## Owned-file observations

**OBSERVED:** The private smoke reads all 20 extracted frontend movies from
`MarvelFrontEnd_SF.upk`, plus the actual `frontend_manager` movie extracted from
`MarvelGame.upk`. Every available DoAction, DoInitAction and placement ClipAction
span in these 21 files parses with the default limits.

| Scope | Root action streams | Recursively decoded instructions |
| --- | ---: | ---: |
| Twenty frontend movies | 2,391 | 447,888 |
| `frontend_manager` | 472 | 331,601 |
| Combined | 2,863 | 779,489 |

The combined roots comprise 1,210 DoAction, 1,321 DoInitAction and 332 ClipAction
streams. There are 23,255 code blocks, 20,302 function definitions and 30 Try
constructs. Maximum nested depth is three. No With operation occurs in this
sample; its decoder currently has synthetic validation only.

**OBSERVED:** The instruction inventory contains 62 distinct documented opcodes,
zero unknown opcodes, zero unknown Push types, zero trailing bytes and zero
unterminated root streams. Of 39,863 branches, zero land inside a record or in
trailing bytes within their immediate block; 30 leave their immediate TryBody.
These are bytecode structure measurements, not a claim that any opcode executes.
Button-definition bodies remain opaque in `gfx`; this inventory does not include
unextracted button action streams or other client packages.

**OBSERVED:** Function and Try action lengths in these samples cover metadata;
nested bodies follow those declared lengths. An independent private scan of the
20 frontend movies agrees with that framing. The observed 64-bit Push literal
encoding stores the high 32-bit word first, with each word little-endian. The
independent scan identifies ordinary values such as 0.75 and 1.2 using that
interpretation; interpreting all eight bytes as plain little-endian produces
subnormal values. The decoder explicitly exchanges the words. This measured
encoding is recorded separately because Adobe's action table's little-endian
description does not make that word order clear.

Evidence, reproducible harnesses and all input hashes remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `avm1-coverage.rs`, `avm1-coverage.tsv`, `avm1-coverage.json`: original Rust
  decoder coverage, input hashes and module hash.
- `avm1-reference/inspect-owned.py`, `avm1-reference/owned-observations.json`:
  independent exploratory framing/branch/double observations for 20 movies.
- `avm1-unit-tests.txt`: focused synthetic validation.

No client bytecode, literal tables, functions, assets or reference-document
copies are committed. The tests are invented format examples.

## Bounds and validation

Defaults are 64 MiB input, one million instructions, 64 nested levels, 65,536
bytes per string, 32 MiB cumulative string bytes, and one million combined pushed
values/pool entries/function parameters. Instruction, string and value counters
apply across all nested blocks in one parse. Configured nesting greater than 256
is rejected. No declared count allocates an unchecked capacity. Source spans,
record lengths and nested sizes are checked before slicing.

Truncated metadata, incomplete bodies and extra bytes in known operands return
`InvalidData` with a source offset where available. Raw flag values are retained,
including reserved bits; this is not a SWF semantic-conformance validator.

**VERIFIED:** Eleven focused tests pass: numeric/string/pool handling,
Function/Function2/With/Try framing, nested absolute boundaries, cross-Try
branches, opaque extensions, truncations, incorrect lengths and resource limits.
The standalone Rust module and private movie harness compile and run. No native
retail comparison, script execution, stack semantics, display-list mutation,
callback invocation or login test has been performed by this module.

## Source and licence ledger

| Source | Licence/rights | Use |
| --- | --- | --- |
| [Adobe SWF File Format Specification v19 (2012), mirrored original PDF](https://open-flash.github.io/mirrors/swf-spec-19.pdf) | © 2006–2012 Adobe Systems Incorporated, all rights reserved | Primary published record framing, opcode names and operand layouts. Printed pages 64–70, 80–83, 88, 92–93, 100, 107 and 112–116. Study only; no source implementation used. |
| Owner's private `MarvelFrontEnd_SF.upk` and `MarvelGame.upk` extracted GFX RawData | Proprietary retail material, private compatibility study | Actual body-length, double-word-order and instruction coverage evidence; paths and SHA-256 values are in the private manifests above. |

All implementation and tests were written here. No external VM/decompiler source
was used. The integrating lead should add these entries to `docs/PROVENANCE.md`;
this lane owns only this document and `src/avm1.rs`.
