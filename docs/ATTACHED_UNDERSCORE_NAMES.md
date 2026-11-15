# Bounded attached MovieClip underscore names

**VERIFIED static native evidence:** `_bMouseOver` and `_bShowTooltip` reach
ordinary member handling on the concrete case-3 attached MovieClip with the
original initialized native-name map. Native classification does not lowercase
their stored keys. For supported versions 7..19, ordinary own-property equality
uses their original interned spelling.

This supports the conservative general admission subset below. It does not
make every underscore name ordinary, enable native aliases, or turn an
incomplete Rust prototype into proof that a property is missing. All existing
live-target, context, storage, child, prototype, accessor, resolver, watch,
callback, property-flag and allocation guards remain necessary.

## Original manager frontier

**OBSERVED controlled run:** frozen `7bb0f19` completes 452 initializers and
reaches **SetMember 60368**, CodeId 15, after 194 frame instructions. Original
Push 60359 supplies `[r1, "_bMouseOver", false]` in the `MarvelObject`
constructor, definition 60217, body `60228+160`, flags `0x19`. Register 1 is
preloaded `this`. The following unexecuted source writes `tooltip_mc=null` at
60377 and `_bShowTooltip=true` at 60387. Those later instructions are source
observations, not completed effects from this diagnostic.

The owned manager movie SHA-256 is
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Private receipts under `evidence/avm1-attached-manager-frame-action-increment7/`:

- `receipt.json`: `0b94d603e4a52a3cfec4bf8b85287af40d7015f796ef6573200a1dfcec843bc8`.
- `static-context/receipt.json`: `da524e1888a31db89a75874f91fbe4500cc2df1fc8372c0ae1b9533fcea8cc36`.

The diagnostic retains the explicitly synthetic current root-name input from
the prior probe. It establishes neither native startup state nor retail parity.

## Standard-name classification and alias flags

The complete authenticated 117-entry map and exact ASCII normalization are
grounded in [ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md) and
[ATTACHED_NAME_CLASSIFICATION.md](ATTACHED_NAME_CLASSIFICATION.md). Both original
spellings and their ASCII-lowercase forms are absent from that complete map.

Normal setter `0x1401aa130` first saves the original key-reference in R15 at
`0x1401aa167`. If its original node has bit 29, helper `0x14018ff00` performs
exact map lookup. Otherwise a leading underscore invokes normalized-node
creation/lookup through `0x140098060`; normalized-node bit 28 determines whether
native-ID lookup is attempted. This normalized result is temporary. The original
R15 remains the key passed to lookup and eventually to storage at
**`0x1401aa742`**.

Complete `0x14018ff00..0x140190087` gives original bit 29 precedence: that
route performs exact node lookup and does not retry aliases after a miss.
Without that bit, the alias route requires normalized bit 28. Complete
`0x1401823b0` compares normalized input/candidate node identities using the
folded hash bucket. For NUL-free ASCII, the verified normalizer changes only
`A..Z` to `a..z`. Therefore absence of **both exact and lowercase spellings
from the complete map** proves a miss on either route, even if those cached
flag bits cause a lookup attempt. The concrete -1 rejection paths then reach
ordinary handling.

This is stronger than assuming that a new node always has certain flags.
It also explains why the conservative subset below excludes every folded
standard-name candidate. Only selectors 1, at IDs 0..21 and 110..115, receive
native alias bit 28 during initialization. `_parent` and `_lockroot` have
selector 0. A nonexact lowercase match must not automatically become
`Standard(id)`: fresh `_X` and `_PARENT` have different native gates, and an
already marked original node takes the exact route instead. This increment
enables neither class of aliases.

Normalization can allocate or populate a cached node. These reference/cache
effects are distinct from changing the ordinary key. The proof does not claim
native allocation-count parity or eliminate allocation-failure boundaries.

## Ordinary spelling is preserved

Bound-storage normal and raw wrappers retain the original key around their
pre-write hooks. Generic normal setter `0x1401a1020` passes that key to
**`0x14018e0c0`** with a Boolean derived from current Environment version >6.
The newly recovered complete helper `0x14018e0c0..0x14018e179` uses the original
node and exact lookup **`0x140181fd0` at `0x14018e0f6`** on that branch.
Its normalized lookup is confined to the other branch.

Raw setter `0x1401e3b20` independently checks its StringContext version and
uses exact lookup at `0x1401e3cc8` when >6. Existing entries retain their key;
new and collision insertions store the original node, not its normalized cache.
The generic getter `0x140197000..0x140197328` likewise compares the stored node
with the original node at `0x1401970cf` in its >6 path. A stored Boolean false
returns found=true. The hash folds ASCII case for bucket selection, but that
does not make differently spelled keys equal.

Attached reads preserve own storage first, named child next and prototype
lookup afterward. Their original key remains in R14 through native-name
classification and those probes. Unknown native properties, unsupported child
classes, callbacks or changed prototype state remain gates. An original-baseline
absence classification is not by itself a current-runtime missing result.

## The later underscore fallback

After ordinary own/child/prototype attempts fail, attached getter
`0x1401ddcd0` checks for a leading underscore at `0x1401de354..0x1401de369`.
It then calls **`0x140194ba0`** with the attached owner's embedded Environment.
Complete `0x140194ba0..0x140194d1d` has three relevant alternatives:

1. For version >6, compare the original node exactly with builtin `_root`,
   then `_global`. Their builtin offsets are `+0x338` and `+0x330`, grounded
   in the established builtin-array base `+0x138`, literal table
   `0x142add2e0`, and exact entries `0x142add4e0`/`0x142add4d8`.
2. Otherwise pass the original bytes to **`0x14017ac10`**, with its
   case-sensitive selector true for version >6.
3. If the parser rejects, or leaves a non-NUL suffix, publish fallback state
   byte 0 without writing the output Value. A recognized level can instead
   return found state 1 or absent-level state 2. The attached caller reports
   success only for state 1.

Complete `0x14017ac10..0x14017acee` requires exact lowercase **`_level`** in
the >6 underscore branch, followed immediately by an ASCII digit `0..9` at
index 6. It then calls a numeric conversion thunk and supplies its end pointer.
The wrapper requires that pointer to address NUL. Older versions also accept
case variants of the prefix, but are outside this profile. Bare digit parsing
exists in the helper; the attached late fallback reaches it only for names
whose first byte was already proved to be `_`.

No numeric-conversion, overflow or level-object implementation is inferred
here. The conservative exclusion is **every exact lowercase `_level` prefix
followed immediately by a digit, regardless of the remaining suffix**. Thus
`_level0suffix` stays gated even though the native wrapper can reject trailing
text. `_level`, `_levelX` and `_Level0` are not candidates in version >6.
This is a deliberate superset of the unresolved level route, not a claim that
all excluded spellings return a level.

## Conservative general admission contract

**INFERRED implementation boundary from the verified routes:** retain the
existing nonempty, bounded printable ASCII profile, bytes **`0x21..0x7e`**,
with the existing 1,024-code-unit limit. It is a valid narrower subset of the
NUL-free ASCII proof. Admit an underscore spelling to ordinary handling only
when all of the following hold:

- Neither its exact spelling nor its ASCII-lowercase spelling occurs in the
  authenticated complete 117-entry standard map.
- It is neither exact `_root` nor exact `_global`.
- It is not a reserved metadata name: `constructor`, `__constructor__`,
  `__proto__` or `__resolve`. Existing typed metadata handling remains separate.
- It is not exact lowercase `_level` followed immediately by an ASCII digit.

The rules admit both original manager names without a per-spelling whitelist.
They preserve original spelling and enable no native alias. Do not trim names,
fold the stored key or split dots/colons/slashes during raw member admission.
Controls, spaces, NUL, non-ASCII, oversized/empty names and older contexts stay
outside this chosen subset. All existing property-operation guards still run.

## Evidence, validation and provenance

The pinned Win64 `1.52.0.1700` executable SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
Private `evidence/rea-attached-underscore-names/complete.bundle.json` contains:

| New record | Scope |
| --- | --- |
| `ev_25644a99b1545da7128c0a2ee2a67d2ef06d2f3cd0d1bfbfc9fac2349ae2ee22` | Complete late underscore fallback |
| `ev_bfef7dba6d41f1d1c47477bed1f0af27dd46e59397c855a47699ddd6b53ffd4f` | Complete level-prefix parser |
| `ev_4530bea687c7dddef47a2c497f0d975dfcbfe615d84841cbc2adbb83d34c6a1a` | Complete normal own-hash lookup and exact builtin literals |
| `ev_1c1ca9ed1f7830dc946cb9e620f661503817486032c9ecbfc94be7b5e51e4635` | Correctly mapped `.data` builtin pointer entries |

The first builtin pointer read used the `.text`/`.rdata` raw-image displacement
for a `.data` address and is explicitly excluded; the fourth record supersedes
it using the PE's correct `.data` displacement. Original function/literal
reads in that earlier record remain valid. This correction is retained in the
private receipt rather than silently treating those bytes as the requested VA.

The original private checker matched **2,318 instruction spans, 1,904 table
bytes and 14 literal bytes** to the pinned PE, including retained caller,
classification, storage and generic-getter evidence. It also authenticated the
private profile, exact original Push/SetMember bytes and both controlled/static
receipts. Verification SHA-256:
`5800a95ec8178393dc172365a0afca6bf9e74a2ab0c1e6b5b881b4c46f21103c`.
Complete bundle SHA-256:
`058957fa0e23478ff26185c288acbe6984b93bfd9eec97d74953848771b2a349`.
Finding summary SHA-256:
`e0d371076be03d55663019bbd8d2b20886c269d0cb3116c3f542b7c3d4df9fb3`.

The retained helper review `ordinary-own-key-review.json` records complete
forwarding/insertion evidence IDs. Its then-open `18e0c0` gap is closed by the
new record above. The existing verified ASCII-normalization and baseline
prototype receipts retain their own scope; no full lookup conclusion is
substituted for those current-state requirements.

All four new REA calls completed untruncated with exit zero and unchanged
target hash, then were exported/reset with no retained target. No native client,
callback or original-script completion was executed or claimed by this study.
Sources read: proprietary owned executable/movie for private interoperability;
original local member-name/classification, attached-object, allocation, builtin
binding and controlled-probe research; REA skill/toolkit (MIT), Rizin
(LGPL-3.0-only), and Python standard library (PSF) for original byte validation.
No outside implementation was copied. Native inventories and extracted bytes
remain private outside git and build packages.
