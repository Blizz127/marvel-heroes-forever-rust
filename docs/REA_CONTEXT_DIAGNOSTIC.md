# Context diagnostic byte formatting

**VERIFIED selected static route:** the nonnull-context, nonnull-log,
formatting-enabled branch of `0x140186d10` formats the message and context
bytes in that order. It removes one terminal LF from the nonempty message,
joins the two arguments with ` : `, adds LF, copies at most 256 bytes and
terminates at byte `min(full_length, 255)`. This can truncate within a multibyte
encoding. It does not interpret the arguments as UTF-8 or expand percent signs.

Original Rust implements that byte result in
`frontend::diagnostic_message::format_with_context`. The result exposes only
the initialized visible prefix and its NUL, plus the full formatted length.
It does not expose or initialize a claimed native stack-buffer tail.

This completes the fixed formatter branch left open in
[the diagnostic dispatcher](REA_MOUSE_MISSING_HANDLER.md). It does not prove
the current Environment's context, a missing Mouse handler, a current log sink,
or the subsequent variadic logger's output. No Host default or OS operation
changes. No original client, original script, service or native callback ran.

## Actual inputs and formatter initialization

The existing complete dispatcher uses optional context D at `E+0x138`, log
pointer `D+8`, formatting flag byte `D+0x12`, and the C-string pointer at
`D+0x18`. These are selected field roles, not recovered public SDK names.
With the flag nonzero it computes the message's C-string length and reads the
last byte, removing exactly one LF if present. An empty message would read
before its start and is outside the admitted Rust profile. The actual fixed
missing-handler message is nonempty; no empty native behavior is invented.

The caller creates formatter F on its stack and supplies destination mode 2,
the 256-byte buffer and capacity 256. Complete constructor
`082b60..082be5` sets table `142504478`, word `F+8=0x0025`, word
`F+0xa=0`, word `F+0xc=0xffff`, count `F+0x40=0`, and null overflow
storage. Consequently the argument-selection byte `F+9` is zero.
Other unspecified padding and inactive bytes are not treated as initialized.

Its local argument arena starts at aligned `F+0x1e8`, within the formatter's
stack storage. Under the actual Win64 stack alignment, the two 40-byte
argument objects occupy `F+0x1e8` and `F+0x210`; the final arena cursor is
`F+0x238` and remaining capacity is `0x1b0`. Both use inline storage.
No argument heap allocation or overflow record allocation is selected.

## Fixed template and argument binding

Exact literal bytes at `14250f9a0` are `{0} : {1}\n` followed by NUL.
Complete parser `082dd0..0830da` creates four inline records:

| Record | Parsed kind | Selected bytes or argument |
| --- | --- | --- |
| 0 | 1 | Argument index 0, empty format suffix |
| 1 | 0 | Literal three bytes ` : ` |
| 2 | 1 | Argument index 1, empty format suffix |
| 3 | 0 | Literal LF |

The selected imports classify ASCII digits 0 and 1 and convert them to those
indices. Numeric placeholders do **not** increment `F+0xa`; that field stays
zero. It is not the number of argument placeholders. The parser's selected
direct stores and the bounded literal append helper `082c20..082cf3`
produce count 4; only the first 16 inline record slots are available but just
four are used here.

The first builder, complete `184cd0..184dbb`, receives the caller's message
pointer/length view. The second, complete `0ace40..0acf2b`, receives the
address of `D+0x18` and loads its nullable C-string pointer. Complete
constructors `080250..080275` and `0801f0..080247` retain those byte views
with common argument table `142504150`. A null context pointer has length
zero; no filename, Unicode string object or inferred source identity is used.

Complete selector `080bc0..080c52` and binder `080c60..080cfe` establish:

- Argument 0 selects record 0 and changes it from kind 1 to kind 2. Its next
  scan reaches argument 1 and returns no further match. The builder advances
  byte `F+9` to 1.
- Argument 1 selects record 2, also changing it to kind 2. The next scan
  reaches the end; `F+9` becomes 2 and `F+0xc` becomes 4.
- Both format suffix lengths are zero, so the binder skips argument virtual
  `+0x10`. No arbitrary argument-specific format parser runs.

Complete finalizer leaf `081dc0..081dd5` sees `F+0xa=0` and jumps directly
to `081450`. The separately captured recursive `081c90` route is excluded
by this concrete template and argument state.

## Length, copy and cleanup

`081450..08160c` sums the literal spans and argument lengths. For each kind-2
argument it calls `080d00`'s selected zero-type branch: common table `+0x30`
resolves to `5b93b0`, whose complete leaf clears AL and returns. The branch
therefore skips the intervening generic type/neighbor formatting logic and
goes to `081427`. Since the new argument byte `+0x10` is zero, it calls
table `+0x18 = 07fba0`, which sets that byte to one and returns. The whole
generic `080d00` body is not claimed covered; its selected entry and tail are.

Argument virtual `+0x28 = 16d760` returns the retained length. Virtual
`+0x20 = 07fbb0` returns the retained pointer and length as a view. Both
complete leaves have no callback or conversion. Thus the full length is the
sum of the trimmed message, three separator bytes, context and one LF.

Mode 2 calls formatter table `+8 = 081610`. Its complete body copies those
four spans in record order, using imported `memcpy`, and reduces the remaining
capacity after each copy. It stops when capacity is exhausted or records end.
`081450` then writes NUL at `min(full_length, 255)`. At full length 256 it
first copies 256 bytes and then overwrites the last copied byte with NUL.
The Rust API returns the equivalent initialized prefix, not the unobservable
intermediate overwritten byte or untouched tail.

A null-context argument remains a nonnull inline argument object with data
pointer null and length zero. When capacity remains, the output helper still
calls `memcpy` with that zero count; it does not skip the call. Ordinary
zero-length import behavior is an explicit native-profile condition.

Complete formatter cleanup `082820..082904` destroys the two kind-2 objects
with virtual destructor argument zero. Concrete `07fb20..07fb46` only
changes the argument's table on this route; its conditional free is skipped.
Both objects are inside the formatter arena, and overflow storage is null,
so the formatter's external free branches are also skipped.

## Rust scope and validation

The API takes a nonempty `CStr` message and optional `CStr` context. It preserves
arbitrary non-NUL bytes, percent sequences and CR, removes only one trailing
LF, and truncates by bytes. Empty messages and unsupported address-space
length overflow are explicit admission errors, not native error semantics.

This function prepares the selected buffer only. The subsequent engine sink
can treat that buffer as a variadic format string; its percent processing,
code-page conversion and output-device effects remain separate. The pure byte
result cannot be used as evidence that a logger or `Mouse.hide` ran.

An independent original instruction-span model froze fifteen witnesses before
the product implementation. They cover terminal LF cases, null/empty context,
full lengths 254/255/256/257, long context, non-ASCII bytes and a split UTF-8
sequence. Its retained copy-call traces distinguish raw copied bytes from the
final initialized C string. Product comparison and focused-test receipts are
under the private `product/` directory.

All fifteen product comparisons and three focused tests pass. Original module
SHA-256: `dc50df21e2ed700ace17b27661eff5cf217b430fdb1de482a7e5d8569930b362`.
The independent derivation receipt is
`template-derivation/receipt.json`, SHA-256
`96677b59c570d1f49669ed1aed9de17c107742136e734f901d407151a582a919`;
the frozen fifteen-case oracle SHA-256 is
`8acb11705a9d9767252a84824883fb44a7210f9cd2582a24d0c8dc317ecf526e`.
These checks establish the supplied-input byte result, not actual startup or
delivery to a log device.

## Evidence and provenance

Private root: `evidence/mouse-prefixed-diagnostic/`. Four new REA records are
retained inline and in `final.bundle.json`; export/reset leaves no target.
All complete with exit0, unchanged executable and untruncated output. Rizin's
automatic whole-region string scan emits size warnings; they are retained and
are not evidence of complete string inventory. Exact literals are read directly.

The original PE validator checks 23 complete bodies plus the selected
`080d00` entry/tail, totaling 957 distinct instruction rows / 3,515 bytes,
selected table slots, literal bytes and four exact import thunks/names.
`verification.json` SHA-256:
`3b645937f654003f5518ee6b5e2d5a27a127158a1ea1ff264a151264abe40d69`.
The first audit's incorrect empty-stderr assumption is recorded separately;
the completed REA calls were not repeated to hide their warnings.

Initial short reads of unwind fragments were extended to complete selected
bodies. Neighboring instructions and padding remain retained observations and
are excluded from the declared semantic ranges. The three classification/
conversion imports are `MSVCR120.dll!isspace`, `isdigit`, and `atoi`; the
output import is `memcpy`. Loaded runtime IAT values were not observed.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owned Win64 `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner installation; private interoperability study | Formatter, argument, copy and cleanup contracts |
| `mouse-missing-handler/rea-diagnostic.json`, retained PE import evidence and project ledgers | Original private project evidence from the same owned artifact | Exact caller, template and import identities |
| REA 6.9.0, installed skill/native guidance, Rizin and GNU objdump | Previously registered MIT / LGPL-3.0 / GPL tooling | Bounded static reads, navigation and canonical export/reset |
| Python and Rust standard libraries/toolchains | Previously registered PSF / MIT-or-Apache-2.0 tooling | Original validators, finite oracle, byte implementation and focused checks |

No external game or SDK implementation was copied. Raw evidence, owner
artifacts and oracle inputs remain private and are excluded from builds/git.
