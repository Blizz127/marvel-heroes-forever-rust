# Serialized clip actions and retained saved handlers

**VERIFIED, bounded static native evidence:** on the recovered AS2
PlaceObject2/3 route, each serialized record's EventFlags are copied unchanged
into its retained event mask. ActionRecordSize includes the optional KeyPress
byte; that byte is stored separately and the remaining bytes are copied exactly
into the saved action source. The envelope's AllEventFlags is skipped and is
not the mask used to select the installed function's execution mode.

This closes the mask/source conversion previously left open in
[FRONTEND_EVENT_DISPATCH.md](FRONTEND_EVENT_DISPATCH.md). It does not execute
the original client, install a Rust handler, prove event delivery or establish
all timeline/import routes. Saved invocation state remains the separate
contract in [SAVED_EVENT_EXECUTION.md](SAVED_EVENT_EXECUTION.md).

The owner-private build `1.52.0.1700` Win64 executable has SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses below refer to that image. Supported conclusions assume the
original tables, the concrete AS2 support route, successful allocations and
complete, well-formed input spans.

## Native placement admission and cursor

The tag dispatch table binds code 26 to `0x14006f2e0` and code 70 to
`0x14006f460`. Both select the external stream at LoadProcess `+0x418` when
present, otherwise its embedded stream at `+0x50`. Predicate
`0x1400defe0` peeks the first tag-body byte, restores the stream position and
returns its high bit: HasClipActions.

When this bit is set and DataState byte `+0xd0` bit `0x08` is clear, the
loaders require the AS2 support pointer. An absent pointer diagnoses missing
AS2 support and skips the tag. This study does not assign a public name to
that state bit. AS2 support virtual slots `+0x38/+0x40` select these concrete
tag types:

| Tag profile | Allocator / primary table | Unpack / clip-reader slots |
| --- | --- | --- |
| PlaceObject2, loaded header version at least 6 | `0x140179bc0` / `0x14250b7e0` | `+0x40` supplies profile 6 to `0x1400df050`; `+0x58` is `0x140179050` |
| PlaceObject2, loaded header version below 6 | Same allocator / `0x14250ba70` | `+0x40` supplies profile 3; `+0x58` is wrapper `0x140179ba0`, supplying profile 3 to the same reader |
| PlaceObject3 | `0x140178e90` / `0x14250b840` | `+0x40` is `0x1400e04d0`; `+0x58` is `0x140179240` |

The loader first retains the tag payload. For a tag with clip actions it
reserves an additional eight bytes and inserts a cache slot immediately after
the first flags byte. The later unpacker sets its cursor base to tag `+0x08`
and starts at byte position 9, thereby skipping this inserted slot. A tag
without clip actions starts at position 1. This cache gap is native storage,
not eight bytes from the original file.

Both unpackers consume depth and the optional fields selected by the first
flags byte: character ID, matrix, color transform, ratio, name and clip depth,
in that order. The matrix/color helpers receive the same cursor; their bit
algorithms are not expanded by this investigation. PlaceObject3 additionally
reads its second flags byte, consumes an explicit class name for bit `0x08`
before the common optional fields, then handles filters (`0x01`), blend byte
(`0x02`), cache byte (`0x04`) and visibility byte (`0x20`). Only first-byte
bit `0x80` invokes the tag's clip-reader virtual slot `+0x58`; otherwise the
unpacked event-list field at output `+0x70` is null.

**Explicit PlaceObject3 boundary:** the recovered complete unpacker does not
consume an implicit class name for HasImage bit `0x10` and does not consume an
opaque-background field for bit `0x40`. The existing Rust format parser has
broader branches for those cases. Their native parity is not established here.
All 29 PlaceObject3 lists in the checked private corpus use second flags
`0x01` or `0x02`, so these excluded cases do not affect that corpus.

## Envelope, record order and event identity

The clip readers byte-align the cursor if necessary. PlaceObject2 skips the
two-byte Reserved field, then skips either four bytes of AllEventFlags for
profile at least 6 or two for the older profile. Its following record masks
use that same 32-bit/16-bit width. PlaceObject3 skips six bytes and always
reads 32-bit record masks. All multibyte record values here are little-endian.

The native readers do not validate Reserved as zero, compare AllEventFlags
with record masks or use it as an event descriptor. A record mask of zero
terminates the list immediately; no ActionRecordSize follows that terminating
mask. Each nonzero mask allocates a 32-byte event object and calls
`0x140183820(event, cursor, mask)`. The newly read record is appended at the
next list index. There is no record deduplication or sorting. The parsed list
is cached at tag `+0x09` through `0x1400defc0` and returned at output `+0x70`.
Later unpacking of that retained tag can return the same cached list.

At `0x14018383c`, the shared record reader writes its mask argument directly
to event DWORD `+0x00`. No bit permutation, event-name lookup or alternate
mask conversion intervenes. Thus these serialized values have the following
retained native values on this route:

| Serialized record flag | Retained native mask | Installed raw mode for that exact mask |
| --- | --- | ---: |
| Construct `0x40000` | `0x40000` | 4 |
| Load `0x00001` | `0x00001` | 4 |
| MouseMove `0x00008` | `0x00008` | 3 |

These mode numbers come from the separately verified installer, not the
loader. Load is statically covered by the direct-copy rule, but no Load record
is present in the checked 332-handler corpus. MouseMove delivery arguments,
eligibility and event execution are outside this study.

The reader also initializes QWORD event `+0x04` to zero, byte `+0x0c` to
zero and DWORD `+0x10` to `0xff00`. The optional key can subsequently replace
DWORD `+0x08`. Padding at `+0x0d..+0x0f` is not established as initialized;
these descriptors must not be compared as an assumed fully zeroed byte array.

## Size, key and exact source span

After a nonzero mask, `0x140183820..0x140183982` byte-aligns and reads the
four-byte ActionRecordSize. If the retained mask has bit `0x20000`, it consumes
one following byte, zero-extends that byte into event DWORD `+0x08`, and
subtracts one from the requested source length. It performs no key-code
translation in this reader. The key is descriptor data, not the first opcode.
Older 16-bit masks cannot contain that bit.

The function allocates the source object, whose final table is
`0x14250f8f8`, stores it at event `+0x18` and calls complete source reader
`0x140179f70..0x140179ff7` with the adjusted length. That leaf:

1. Byte-aligns the cursor, allocates exactly the requested number of bytes and
   stores the new byte pointer at source `+0x10`.
2. Calls the bound `MSVCR120.dll!memcpy` thunk `0x141c6ed86` with the cursor's
   current byte pointer and that exact length.
3. Stores the same requested length at source DWORD `+0x18` and advances the
   cursor by exactly that many bytes.

There is no ActionEnd search, bytecode validation, content-dependent trimming
or added terminator. An ActionEnd already present is copied; any remaining
declared bytes after it are copied too. The caller has a generic length
reconciliation branch, but this concrete leaf returns exactly the requested
length, so that branch makes no adjustment on the admitted successful path.

**INFERRED original Rust representation from that verified copy:** for a
record starting at original file offset `R`, let `W` be the event-mask width,
`N` its ActionRecordSize and `K` be 1 when its mask has `0x20000`, otherwise 0.
The original source span is:

```text
start  = R + W + 4 + K
length = N - K
next record = R + W + 4 + N
```

These formulas apply after correct placement/envelope traversal and byte
alignment. Preserve the original movie/resource and tag/record identity with
that span. The native source object owns a copy indexed from zero; it does not
store the original file offset. The Rust provenance mapping must therefore
remain explicit rather than labeling this source as the receiving instance's
constructor or current frame action.

A safe Rust parser must check size at least `K`, checked arithmetic, complete
source availability and resource limits before forming the span. The native
reader's unchecked KeyPress subtraction and copy are not a requirement to
reproduce unsafe behavior for malformed input. No malformed-size, allocation
failure, truncated-input or modified-import behavior is admitted by this proof.

## Installer admission and aggregate-mask meaning

The retained construction producer `0x140203920` visits its supplied event
objects in order and calls `0x140200570`. As documented in
FRONTEND_EVENT_DISPATCH, that installer rejects a null source, source length
below one, or a zero first source byte. For admitted data it creates a source
wrapper and ScriptFunction retaining this same source with local start 0 and
its full stored length. The loader's exact copy and this retained pointer join
the serialized action span to the installed function source.

The installer chooses raw mode 4 only when the **original per-record mask**
equals `1`, `4`, `0x200` or `0x40000`; every other mask uses mode 3. Splitting a
combined record mask into map keys happens afterward and shares the same
callable/source wrapper across those keys. Header AllEventFlags, the union of
several separate records, and a single delivered bit must not replace the
per-record mask for this decision.

There are three different zero-related rules:

- A zero EventFlags field terminates the serialized record list.
- An ActionEnd opcode inside a record remains part of the exact copied span;
  interpreter behavior is separate from loader framing.
- A zero first action byte causes the installer to skip that saved record,
  even when its mask and declared length are nonzero.

This investigation binds tag unpacking to the retained event/source format and
joins that format to the existing installer. It does not independently trace
every timeline consumer between unpacked output `+0x70` and the construction
producer, all imported-resource ownership changes or every replacement path.
Those live-instance admission conditions must remain explicit in integration.

## Checks against the owner's decoded movies

**OBSERVED, original-byte inspection:** the frozen private login input probe
contains 332 serialized handlers in 331 lists across its 26 decoded movies:
331 Construct records and one MouseMove record. The lists comprise 302
PlaceObject2 and 29 PlaceObject3 tags. The one two-record list is the manager
cursor placement; its separate Construct and MouseMove records must retain
separate masks and sources despite their shared header union.

The independent checker verifies the decoded movie hashes, bounded tag and
optional-field traversal, envelope positions, mask width, each record mask and
size, exact action span/hash, and the final zero-mask terminator. It also records
the installer's first-byte admission and derived mode for every record. This is
a static corpus check, not native execution or a successful event delivery.
All 8,786 field/hash checks pass. Every handler uses a four-byte event mask;
all 332 sources pass the first-byte gate. All Reserved values are zero, all
header masks equal their record unions, and each zero-mask terminator ends its
tag exactly. These are observed asset properties, not native validation rules.

The frozen 147-node login Scene inventory associates only 34 of these handlers
with instances, all Construct from placing movie 13. This uses the probe's
explicit static profile assumptions; it establishes no native or VM object
binding. The manager cursor MouseMove belongs to movie 20, outside that Scene's
registered movies `[5, 13, 19]`; it must not be installed merely because it
appears in the broader loaded-resource inventory. Two exact private examples
retained by the corpus receipt are:

| Original record | Action span | SHA-256 |
| --- | --- | --- |
| Login button Construct, movie 13 / tag 1345, record 39228 | `39236 + 449` | `f68fe1b85573841b9c5a18018092b9e8e0b0ab34720a3ed0ee422cbce396f431` |
| Manager cursor MouseMove, movie 20 / tag 375, record 7088 | `7096 + 77` | `b81239a0272a1f40972fafeac96d075d36d91bf5528c649e4b305f2e681fe885` |

No KeyPress example exists in this corpus. Its byte-consumption rule is grounded
in the complete native reader, not a claimed original-asset or runtime test.
The header Reserved/AllEventFlags observations for well-formed assets do not
authorize a native equality-validation claim: the native reader skips them.

## Evidence and provenance

All original bytes, native output and private checks remain under
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-clipaction-loading/combined-proof-receipt.json` binds the component
  receipts, corpus validation, pinned image and original verification scripts;
  its SHA-256 is
  `9e79049fbd3ff23dff71daf7e13d0f0f2eea514178537a54097e795392ecc18b`.
- `rea-clipaction-loading/record.bundle.json`, Evidence
  `ev_156abd518ddc1de8ec36523624a8389fffb84c8113d746b70c40fa2dbe5af0df`,
  contains complete shared record reader `0x140183820`.
- `rea-clipaction-placeobject/finding-summary.json`, SHA-256
  `bd1b829183f63354d738f99c3361e1f71ee042aa7f32e1c6c48ac176c309a95d`,
  binds eight REA records for dispatch, support/tag tables, versioned unpacking,
  cache and envelope readers.
- `rea-clipaction-source-span/finding-summary.json`, SHA-256
  `fece98d13093a59a83ab84aa789b2cb8e887778c6c590f74f203c674b7d506cc`,
  binds exact source allocation/copy and retained installer use.
- `rea-clipaction-loading/native-byte-check.json` validates completion, exit 0,
  untruncated output and unchanged target hashes for all 12 new REA records. It
  independently matches 1,284 decoded spans, 328 table bytes and four literals
  to the pinned PE. These counts include explicitly excluded discovery reads;
  byte correspondence alone is not a semantic-coverage claim.
- `rea-clipaction-loading/corpus-check.json` binds the frozen
  `login-instance-inputs/probe/clip-wire-records.json`, original decoded movie
  hashes and independent bounded source-span checks; its SHA-256 is
  `323d00c0bcca54fbbc1ce4a7f19a0d7503e51880b5f564ddba598c44eae70605`.
- Retained `rea-event-dispatch/complete.bundle.json`, Evidence
  `ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54`,
  supplies installer/wrapper/splitter semantics; retained
  `rea-attached-name-classification/buffer-final.bundle.json`, Evidence
  `ev_27687c7d49830dbb9858dbfb53796f6ffb93565413c87c09c74eeb8116833e02`,
  identifies the PE-mode memcpy thunk.

The three REA investigations exported their records and reset their ledgers;
none retained an active binary target. No Ghidra setup, shared configuration
change, original-client launch, server action or source-byte publication occurred.

| Source read | Licence / ownership | Use |
| --- | --- | --- |
| Owner-private pinned `MarvelHeroesOmega.exe`, build 1.52.0.1700 | Proprietary; compatibility/interoperability study only | Bounded static instructions, table bindings and byte validation |
| Owner-private decoded GFX movies and frozen login inventory under `evidence/login-instance-inputs/probe/` | Proprietary original assets; private derived project evidence | Exact serialized framing, source spans and corpus admission |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, connection/native reference guides and REA 6.9.0 | MIT tooling/documentation | Supported static workflow and Evidence lifecycle |
| Rizin 0.9.1; GNU objdump; Python standard library | LGPL-3.0-only; GPL-3.0 tooling; PSF | Canonical decode, candidate navigation and original verification scripts; not linked or shipped |
| Project `src/gfx.rs`, `docs/GFX_FORMAT.md`, `docs/FRONTEND_EVENT_DISPATCH.md`, `docs/SAVED_EVENT_EXECUTION.md`, `docs/REA_NATIVE_ALTERNATIVE.md` and retained native/movie-header receipts named in component manifests | Original project work; underlying native output remains private | Existing format labels, bounded parser comparison and reuse of proven source/installer/version relationships |

All explanation and checking code are original. No SDK implementation, extracted
native code or asset payload is added to the repository.
