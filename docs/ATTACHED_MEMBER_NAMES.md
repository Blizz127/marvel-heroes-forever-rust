# Native attached-MovieClip member-name admission

**VERIFIED — bounded static evidence:** `gfx`, `Function`, `disabled`,
`enableInitCallback`, `localTextLabel`, `button`, and `localizedText_tf` all
reach the ordinary member route on the concrete attached source MovieClip
class with the original initialized standard-member map. This admits their
names; it does not decide whether a particular access finds a value, invokes an
accessor, encounters an unsupported child type, or may perform an assignment.

The standard-member map is a complete finite initialization table with **117
entries**, IDs 0 through 116. None of the seven requested names occurs in it.
The raw inventory and strings remain private. A test such as “does not begin
with underscore” is insufficient: native standard members also include
`enabled`, `tabEnabled`, `transform`, and `scale9Grid`.

The compatible implementation boundary is the original case-3 attached class,
loaded version greater than 6, the original standard-name map, and exact
supported names. Keep all other constraints in
[the attached MovieClip study](ATTACHED_MOVIECLIP.md): member precedence,
complete child state, prototype behavior, pre-write hooks, read-only flags,
accessors, resolvers, watches, and live target identity.

No original executable was run. These are static routing facts, not a retail
startup or gameplay-parity result.

## Target, evidence, and finite table

The owner's pinned executable is private build `1.52.0.1700`, Win64,
SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
All addresses below are preferred-image virtual addresses. Evidence paths are
relative to `~/.local/share/marvel-heroes-forever-rust/`.

The finite table begins at `0x142adcaa0`, uses 16-byte rows, and ends at the
null-name sentinel `0x142add1f0`. Each row holds a string pointer at `+0`, an ID
whose low byte is used at `+8`, and a classification selector at `+0xc`.
All 117 live IDs are contiguous and unique in the range 0..116; the sentinel
ID field is `0xffffffff`.

The complete private inventory, row addresses, exact names, IDs, selectors,
and byte-check receipts are in
`evidence/rea-attached-name-map/finding-summary.json`. Its canonical REA bundle
is `evidence/rea-attached-name-map/final.bundle.json`. This is a complete table
walk, not a search that inferred absence from a few strings.

Initializer `0x1401954c0..0x1401955b1` receives the global context and populates
its map at `+0x70`. Bootstrap `0x14020fee7..0x14020fef3` prepares that map and
calls the initializer. The loop advances by 16 until the null-name sentinel.
For every row it supplies bit 29 (`0x20000000`) to the constant string interner
`0x1400971e0`. It additionally supplies bit 28 (`0x10000000`) when the row's
selector is nonzero. That selector is set exactly for IDs 0..21 and 110..115.

The initializer passes the resulting interned node and ID byte to
`0x140187f10`. The complete insertion helper retains the node and stores this
pair in the context map's 24-byte buckets. The node classification and the ID
map are related but distinct state; a cached name flag alone is not an ID.

## What the string flags mean on this route

The flag word is `StringNode + 0x1c`; its lower 24 bits contain the hash.

| Bit | Observed producer/use relevant here |
| --- | --- |
| 27 | Cached agreement between byte count and character count, used by the length/character helpers. |
| 28 | Selected by the standard-member table; an underscore-name gate checks this bit on its normalized node. |
| 29 | Set for every standard-table name; the attached caller uses it to request native member-ID lookup. |
| 30 | Added when the constant interner creates a new node referring to constant bytes. |
| 31 | Supplied by the separate builtin-string initialization loop; it is not equivalent to standard-member bit 29. |

The complete constant interner `0x1400971e0..0x1400972dc` ORs its supplied flags
into an existing node at `0x140097255`. A newly created node receives the
24-bit hash, supplied flags, and bit 30. The dynamic interner
`0x1400972e0..0x14009740a` creates a node with only the lower 24-bit hash;
an existing interned node retains its existing flags.

Builtin-string construction `0x140181020 -> 0x140097f70` walks a separate
156-entry array and supplies `0x80000000` to the constant interner. In
particular, being the builtin name `Function` does not make a node a standard
MovieClip member. This does not claim that every later flag writer in the
program has been found. The admission proof below tolerates either state of
bit 29 for each requested name.

## Exact caller gate

The attached normal getter and raw getter converge on `0x1401ddcd0`.
The attached normal setter is `0x1401aa130`; its raw setter is
`0x1401eb4d0`. All three use the same initial classification:

1. If the original name node has bit 29, request a native member ID.
2. Otherwise, require a nonempty name beginning with `_`, obtain its normalized
   node through `0x140098060`, and request a native ID only when that node has
   bit 28.
3. Otherwise, continue to the ordinary-name branch.

For the requested ASCII names, the length/first-character helpers are also
grounded: `0x140096820` either uses the cached byte count or counts characters
through `0x14006b480`; that function counts each positive ASCII byte once.
`0x140096870`, index zero, either reads the first byte directly or calls
`0x14006a960`, whose positive-ASCII branch returns that same first byte.
All seven names are nonempty and their first byte is not `_`. Their spelling
therefore bypasses the normalized-underscore branch when bit 29 is clear.

`0x140098060` obtains or initializes the node's cached normalized form through
`0x140097b50`. This study does not generalize that normalization into a new
Unicode or older-version case policy; the requested names do not need that
branch. An unknown underscore name remains an explicit implementation gate.

## A forced native lookup still falls through for these names

The lookup helper `0x14018ff00..0x140190087` obtains the context through owner
virtual `+0xb8`, then uses its map at `+0x70`. With original node bit 29 set,
it uses the original interned node and exact lookup `0x1402853c0`; that lookup
compares interned pointers in the bucket chain. A successful table entry
provides its ID byte. Every miss returns **signed -1**, from an initial byte
`0xff` followed by sign extension.

Because all seven names are absent from the complete original map, forcing
this lookup with a preexisting bit 29 still returns -1. The concrete attached
class handles that result as follows:

| Route | Verified -1 path |
| --- | --- |
| Getter | Primary table `0x142517670`, slot `+0x118`, calls `0x1401dd870`, which calls base helper `0x1401dce90`. With the caller's mode zero, `0x1401dcee1` detects -1 and returns false. The derived unsigned range test also rejects -1 and returns false at `0x1401dd9c3`. |
| Setter | Primary slot `+0x110` calls `0x1401c3e50`. Its signed comparison sends -1 to base helper `0x1401a8f20`, whose mode-zero check at `0x1401a8f60` returns false for -1. The derived range rejects it and returns false after temporary-value cleanup at `0x1401c43ba`. |

The getter caller's later native-ID switch rejects -1 at
`0x1401ddd95..0x1401ddd9b`. The normal setter's extra native-ID switch rejects
it at `0x1401aa20f..0x1401aa215`. The raw setter continues immediately when its
native setter returns false. These routes then reach ordinary handling.

The getter and normal setter also recognize the exact builtin `__proto__`
outside this table gate. None of the requested names matches it. The later
raw storage setter and pre-write hook have their own reserved names and
effects, described in the attached MovieClip study; ordinary name admission
does not bypass those hooks.

| Requested exact name | Initialized native ID | Admitted name route |
| --- | --- | --- |
| `gfx` | none / lookup -1 | ordinary |
| `Function` | none / lookup -1 | ordinary |
| `disabled` | none / lookup -1 | ordinary |
| `enableInitCallback` | none / lookup -1 | ordinary |
| `localTextLabel` | none / lookup -1 | ordinary |
| `button` | none / lookup -1 | ordinary |
| `localizedText_tf` | none / lookup -1 | ordinary |

This proof is independent of the historical order in which a requested node
was interned. It still requires the original initialized map and the concrete
attached class; it is not a claim about an arbitrarily replaced host map or
another native display class's virtual methods.

## Ordinary name admission is not a missing-value shortcut

Reads retain the native order: own storage, named child, display prototype.
The value may be present even if it is Undefined. An ordinary name may name a
TextField or another unsupported child class; admitting `localizedText_tf`
does not implement that child's text properties. Writes use the established
bound-storage path and cannot substitute the named display child as the write
destination.

For the exact `Function` lookup, the original unmodified builtin prototype
chain can be checked separately. MovieClip.prototype's getter `0x1401e6020`
special-cases only `__constructor__` and `constructor` before using generic
`0x140197000`; `Function` matches neither. Generic lookup's reserved checks
are `__proto__` and `__resolve`, also different. The complete native 44-key
MovieClip.prototype and 9-key Object.prototype inventories contain no
`Function` member, no `__resolve` entry, and no accessor entries. Their native
constructors initialize resolver fields `+0x40/+0x48/+0x50` to zero.

With that unchanged chain, context greater than 6, and Object.prototype's
null parent, the generic search reaches `0x140197160` and returns false without
calling an accessor or resolver or changing the caller's output Value. Raw
false does not itself mean that an Undefined value was written. The separate
private prototype-miss receipt
binds these inventories, constructors, and getter paths. Later script changes
must be tracked: a new `Function`, resolver, accessor, or parent relationship
invalidates this unchanged-chain conclusion. Missing a key in an incomplete
Rust prototype model is not an adequate substitute.

## Evidence, validation, and remaining boundaries

New canonical bundles and receipts:

- `evidence/rea-attached-member-names/final.bundle.json`: interner flags,
  string gates, ASCII helpers, concrete native virtual slots, and the selected
  unmapped-ID get/set branches.
- `evidence/rea-attached-member-names/combined-proof-receipt.json`: joined
  hashes, instruction/data verification, selected name results, and limits.
- `evidence/rea-attached-name-map/final.bundle.json` and
  `finding-summary.json`: complete 117-entry table, initializer, insertion,
  exact lookup, and miss-ID proof.
- `evidence/rea-prototype-function-miss/finding-summary.json`: unchanged
  builtin prototype miss proof, reusing canonical inventories and native
  constructor/getter receipts.

The complete attached caller functions are reused from
`evidence/rea-event-dispatch/complete.bundle.json`, record
`ev_90b1d7e29f962fe20773e92738557b9a70ac866cca42f75a690623168b6c3c54`, and
`evidence/rea-attached-movieclip/complete.bundle.json`, record
`ev_a256724d7b49eca13a8a4ce045d955da2247806b89f3572d0633987970a1a973`.
Selected new ranges deliberately stop after the relevant return branches;
trailing partial instructions are not used as complete-function evidence.

All new REA calls were bounded, target-free Rizin queries. Instruction bytes,
table bytes, and literal strings were checked independently against the pinned
PE: nine new records matched 1,097 instruction spans, 5,261 data bytes, and
117 literal strings. The separate prototype-miss study validated 2,298 spans
across 11 reused records. No shared configuration, target state, original assets, or native process
was modified. Focused documentation validation used `git diff --check`.

Remaining gates include arbitrary standard-member IDs, underscores and
normalization, non-ASCII names, older contexts, replacement native maps,
different display classes, property callbacks, incomplete/ambiguous child
snapshots, target retirement, and untracked prototype mutations. These are
limits of the admitted Rust profile, not claims that retail rejects those
operations.

## Provenance

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe` | Gazillion/Marvel proprietary; private owner installation | Native name classification, finite registration table, interner and ID routing for interoperability; raw bytes, strings and extraction remain private. |
| REA 6.9.0 and local `reverse-engineer-anything` skill, connection/recovery and native-artifact guides | MIT toolkit and local usage instructions | Required bounded native workflow and Evidence preservation. |
| Rizin 0.9.1 through REA | LGPL-3.0-only | Selected instruction and data decoding. |
| Existing project attached-object, event, prototype-inventory and constructor receipts cited above | Original project analysis | Concrete caller/class binding and unchanged builtin-chain proof. |

No SDK source or third-party VM implementation supplied these semantics.
