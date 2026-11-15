# Native AVM1 enumeration for UIObject clip parameters

Task `a3d58c579542`, private interoperability study, 2026-11-15 UTC.

This investigation recovers the native ordering used by the original
`Enumerate2` at movie offset 750,939. It changes no VM implementation and
does not execute the retail client. Native control-flow findings are marked
**VERIFIED**; original-bytecode and Rust-probe observations are marked
**OBSERVED**. The concrete key sequence is **DERIVED** from those inputs and
the recovered native algorithm, not a captured retail execution.

## Original loop and source object

**OBSERVED:** Initializer 118, `mx.core.UIComponent` at 757,488, calls
`mx.core.UIObject.mergeClipParameters` at 760,210. Its method body starts at
750,934 and enumerates parameter `p` at 750,939. Each loop iteration performs
`o[k] = p[k]`, then returns true after the null sentinel. The source is
`UIObject.prototype.clipParameters`; the destination is the initially empty
`UIComponent.prototype.clipParameters` object.

Source `InitObject` at 751,171 requests these seven assignments in stack-pop
order, all with numeric value `1` and initial property flags `0`:

`preferredWidth`, `preferredHeight`, `maxWidth`, `maxHeight`, `minWidth`,
`minHeight`, `visible`.

The original-bytecode receipt is
`evidence/avm1-runtime/enumerate2-mergeclip-private.json`. The private
instrumented Rust probe records source object 6,227, destination 6,377 and
the same seven fields in
`evidence/avm1-enumeration/creation-and-target.tsv`.
That probe predates the native Object metadata correction below; its smaller
hidden-key set is not evidence of native occupancy.

## InitObject creates three hidden own entries

**VERIFIED (static native paths):** Raw opcode `0x43` reaches the case at
`0x14020ae17`. It constructs an object using the unchanged global Object
binding through `0x1401a7c50` and shared construction helper `0x1401a74c0`,
with zero arguments, before consuming the literal pairs.

The successful native construction creates these own stored entries, in order:

| Entry | Flags | Source |
| --- | --- | --- |
| `__proto__` | `3` | Object allocator `0x1401cd330` calls `0x1401ccf40`; its initially empty hash receives the parent link through `0x14018edf0`. |
| `__constructor__` | `3` | Shared construction calls `0x14018ec90` with the original Object function reference. |
| `constructor` | `3` | Zero-argument Object callback `0x1401cd010` reuses the supplied receiver and installs public constructor metadata at `0x1401cd27f`. |

The callback returns that same object and does not remove either preceding
entry. The pair loop then reads the top value and the name immediately below
it, requires a string name, writes through the instance member setter with
flags zero, and pops both stack entries. Thus the actual literal creates ten
occupied own entries, of which seven are enumerable.

This is a proof of `InitObject` through the unchanged native Object binding.
It must not be applied indiscriminately to every internal object allocation,
automatic script-function prototype creation, or arbitrary `Object()` call.

The source's native Object.prototype parent has nine stored entries. Its
public `constructor` has flags `3`; its eight native methods have flags `1`.
Its parent is null. The private inventory
`evidence/rea-object-prototype-inventory/object-prototype-inventory.json`
therefore establishes that this unchanged native parent contributes no
enumerated names. Script modifications to those flags or keys require using
the resulting current state.

## Hash and physical iteration order

**VERIFIED (static instructions):** Property table iteration uses physical
bucket order. The table pointer is at object-interface offset `0x18`.
The table header stores occupied count at `+0`, capacity minus one at `+8`,
then 48-byte entries beginning at `+0x10`. Each entry contains a chain link,
a string-node key pointer and a stored Value. Link `-2` denotes an empty
entry; link `-1` terminates a collision chain.

The string-node hash comes from `0x1400340f0`, using seed `5381`. It scans
the original bytes from right to left, folds ASCII `A` through `Z` to
lowercase, and applies this wrapping 64-bit recurrence:

`hash = (hash * 33) XOR folded_byte`

String factories `0x1400971e0` and `0x1400972e0` retain the low 24 bits in
node field `+0x1c`. Builtin-string flags occupy higher bits. The property
table masks this field by capacity minus one; those higher flags do not
affect the 8- and 16-bucket tables here. Folding in the hash does not by
itself imply a case-insensitive property equality rule.

The original key strings are connected to that factory: raw ConstantPool
opcode `0x88` dispatches to `0x14020c632`, then parser `0x140186800`. For
each valid NUL-terminated pool string it calls `0x140097410`, which computes
the byte length and tailcalls `0x1400972e0` with the original bytes. Builtin
metadata names use `0x140097f70` and `0x1400971e0`, which share the same
hash function and low 24 bits.

New-key insertion `0x1401cb1c0` follows these rules:

1. Allocate eight buckets for the first entry.
2. Before a later insertion, grow when the existing count times five is
   strictly greater than capacity times four. The new capacity is doubled.
3. Use `hash & (capacity - 1)` as the key's home bucket. An empty home
   receives the new entry directly.
4. If occupied, find the next empty bucket by incrementing and wrapping the
   index. Move the current home occupant there. If it belonged to this home
   chain, make the new key the head and link it to the moved entry. If it
   belonged to another chain, repair that chain's previous link and give the
   new key an end-of-chain link at its own home.

The full insertion body spans `0x1401cb1c0..0x1401cb345`; its unwind metadata
splits the function into several ranges. Move/copy helpers preserve keys,
values, flags and chain links as described above.

Growth routine `0x1401a3f20..0x1401a408f` builds an empty table and reinserts
old occupied entries in ascending old bucket order through
`0x1401c2c30..0x1401c2dfd`, which uses the same collision placement rules.
It then replaces the old table. Existing-key writeback `0x1401cffd0`
updates the stored value in place without changing its bucket.

For the actual ten-entry history, capacity grows from 8 to 16 before
`minWidth` is inserted. The complete capacity-rounding helper
`0x141259450..0x141259527` selects the highest nonzero byte and uses a
256-byte lookup table to obtain its highest-set-bit index. Every nonzero
table entry was checked against that arithmetic. The caller rounds positive
requests up to a power of two, with minimum eight; doubling an existing
power-of-two capacity therefore preserves the requested doubled size.
Overflow and allocation failure remain outside this supported history.

## Filtering, prototypes and operand-stack order

**VERIFIED for the generic plain-object visitor:** Enumeration helper
`0x140208bf0` removes the receiver, pushes a null sentinel, and calls its
member visitor with options `0x0b`. Generic visitor
`0x1401924c0..0x1401926d4` scans occupied buckets in ascending index order.
It skips a stored entry whose property flags contain `1` before calling the
enumeration callback. It then traverses the parent with the same options.
The actual native instance interface `0x1425185c0` and Object.prototype
interface `0x142513cd0` both bind virtual slot `+0x40` to this visitor.

Option `0x08` prevents resolution of special raw value tag `0x0a`; the
callback receives a temporary undefined value for that case. It only uses
the member name, so enumeration does not execute a getter to obtain the
name. This is distinct from the subsequent loop's normal `GetMember`.

Callback vtable `0x14250ef20`, slot `+8`, points to `0x140188340`.
That callback directly pushes each name as a string value onto the operand
stack. It does not collect, sort or reverse a name vector. The generic
visitor and this callback contain no seen-name set or prototype-shadow
filter: eligible duplicate names from own and inherited entries are pushed
separately. A hidden own entry does not mask an enumerable inherited entry
on this path. Other native object categories need their own visitor binding.

Consequently, pushes follow own ascending buckets, then parent traversal.
The original loop pops the reverse of that entire sequence, ending at the
null sentinel. With this source's unchanged Object.prototype contributing
no names, it pops the source's seven enumerable buckets in descending order.

## Derived result for the original loop

The original private mathematical model is
`evidence/rea-enumeration-order/derive_order.py`; its receipt is
`derived-order.json`. It follows the recovered native insertion and growth
instructions and records each intermediate table. It does not load or run
native client code.

**DERIVED final table:**

| Bucket | Stored key | Low 24-bit hash | Flags | Enumeration |
| --- | --- | --- | --- | --- |
| 0 | `minHeight` | `d64890` | `0` | Included |
| 1 | `__constructor__` | `d7a28d` | `3` | Hidden; linked from bucket 13 |
| 3 | `__proto__` | `8db693` | `3` | Hidden |
| 6 | `preferredWidth` | `834b86` | `0` | Included |
| 7 | `maxWidth` | `9983d7` | `0` | Included |
| 9 | `minWidth` | `9936c9` | `0` | Included |
| 11 | `visible` | `149c0b` | `0` | Included |
| 13 | `constructor` | `469e8d` | `3` | Hidden; collision head |
| 14 | `maxHeight` | `d5ea8e` | `0` | Included |
| 15 | `preferredHeight` | `72665f` | `0` | Included |

The callback pushes:

`minHeight`, `preferredWidth`, `maxWidth`, `minWidth`, `visible`,
`maxHeight`, `preferredHeight`.

The original loop therefore consumes and copies:

`preferredHeight`, `maxHeight`, `visible`, `minWidth`, `maxWidth`,
`preferredWidth`, `minHeight`.

All copied values are `1`. Their final name-to-value mapping is independent
of this traversal order, but the destination's insertion history and later
enumerations can depend on it. This result does not justify substituting
sorted or insertion order.

## Scope and retained evidence

The target is the owner's private Win64 build 1.52.0.1700 executable,
SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The private manager movie is SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.

The joined private receipt is
`evidence/rea-enumeration-order/combined-proof-receipt.json`, SHA-256
`ccc25e98671a6d3793d585778d9b60f0e4d640996a9f1765bac08064dea6a87d`.
Its eleven new REA records all completed with exit zero, no truncation and an
unchanged target. Independent PE-section mapping matched all 1,472 decoded
instruction spans. Seven selected data reads, totaling 552 bytes, also matched
the pinned binary in `data-read-validation.json`. Unrelated partial trailing
instructions are retained but are not used as semantic evidence.

| Private evidence | Contribution |
| --- | --- |
| `rea-enumeration-order/final.bundle.json` | Four records for hashing, insertion, collision relocation, growth, rehashing, capacity rounding and builtin-name hashing. |
| `rea-enumeration-order/capacity-range.bundle.json` | Complete highest-bit helper and all 256 lookup bytes, confirming rounding beyond the motivating 8- and 16-bucket tables. |
| `rea-enumeration-callback/final.bundle.json` and `finding-summary.json` | Callback table binding, reused complete enumeration/visitor/callback bodies, filtering, prototypes, duplicate handling and operand-stack order. |
| `rea-enumeration-callback/initobject-final.bundle.json` and `initobject-finding-summary.json` | Native InitObject creation, Object allocator/callback, three hidden entries and literal pair order. |
| `rea-enumeration-callback/constructor-and-pool-final.bundle.json` and `constant-pool-finding-summary.json` | Current global Object identity for public constructor metadata and the original ConstantPool string-to-hash-factory path. |
| `rea-object-prototype-inventory/object-prototype-inventory.json` | All nine native parent keys, their hidden flags and null parent. |
| `rea-enumeration-order/derived-order.json` | Original mathematical derivation, exact final slots, pushes and pops; source script retained beside it. |

Paths in this table are relative to the private `evidence/` directory. The
joined receipt retains hashes and exact record identifiers, plus reused
allocation, metadata, raw-setter and Value-handling evidence. All enumeration
investigations were exported and reset.

The result assumes the recorded literal, successful allocation, unchanged
native Object binding and parent baseline, and no intervening mutation of
the source's stored-key set or layout. The model covers the actual ASCII,
no-deletion input. Native deletion compaction, unsupported prior capacity
history, arrays, display objects, arbitrary getter/proxy implementations,
prototype cycles and concurrent mutation are not established here.

All client bytes, bytecode, native disassembly and derived private artifacts
remain outside the repository. No runtime or gameplay parity is claimed.

## Provenance

| Source read | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned Win64 executable and existing version receipt | Proprietary Gazillion client; owner-authorized compatibility study | Bounded native instructions, tables and exact-byte receipts. No original executable execution or recovered code copied into implementation. |
| Private manager movie, `enumerate2-mergeclip-private.json`, `creation-and-target.tsv` and related AVM1 lane receipts | Proprietary owner assets and original project analysis | Original caller, literal creation sequence, loop behavior and concrete Rust-probe receiver. The probe is not a retail capture. Adobe documentation referenced by the source lane was not independently reopened here. |
| `docs/REA_NATIVE_ALTERNATIVE.md`, `docs/REA_FUNCTION_CALLS.md` and prior private REA bundles named by component receipts | Original project analysis; underlying target output remains private/proprietary | Reuse generic visitor, setters, native Object binding, constructor metadata and parent inventory without duplicating broad analysis. |
| Mandatory REA skill, connection/native guides and installed tool schemas | Previously read REA guidance; REA 6.9.0 MIT | Supported target-free static analysis and evidence preservation; no setup or shared configuration changes. |
| Rizin through REA | LGPL-3.0-only analysis tool | Bounded disassembly, table and string reads; not linked or shipped. |
| Python 3 standard library | PSF tooling | Original PE-byte validation, mathematical hash/table model and receipt assembly. |

All repository prose and the private derivation script are original work.
