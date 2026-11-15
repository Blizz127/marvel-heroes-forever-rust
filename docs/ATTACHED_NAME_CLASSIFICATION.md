# General attached-name classification from private native inventories

**VERIFIED — bounded native evidence:** a general ordinary-name profile is
supported. On the concrete case-3 attached MovieClip class, with the original
initialized standard-member map, every nonempty NUL-free ASCII name that does
not begin with `_` and is absent from the complete 117-entry native map reaches
ordinary member handling. A new proof for each such spelling is unnecessary.

**IMPLEMENTED — original Rust:** [attached_name_profile.rs](../src/attached_name_profile.rs)
authenticates and parses a caller-supplied private inventory, classifies those
names, and separately reports original builtin-prototype membership. It embeds
no extracted native-name arrays. It does not change the VM, implement native
standard properties, perform property lookup, or declare a current property
missing merely because Rust has not modeled it.

The intended initial runtime scope remains live case-3 instances, supported
loaded versions 7..19, the established current context, and existing attachment,
storage, child, prototype, resolver, accessor and watch guards. The native map
layer itself has no version check; the greater-than-6 restriction is required
by the downstream exact-name prototype and child lookup profile.

## Why the general ordinary route is established

[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md) grounds the complete map,
concrete getter/setter admission and signed `-1` fallback. It is not merely an
inventory of the first seven names encountered by the diagnostic VM.

For any valid non-underscore name absent from that map:

- With original-node bit 29 clear, the caller's nonempty/first-character gate
  bypasses the native member lookup and reaches the ordinary route.
- With bit 29 set, exact map lookup cannot find that name and returns `-1`.
  The concrete native getter/setter and their callers reject that ID and fall
  through to ordinary handling.

Thus admitting this set does not require a whole-program assertion that no
later native operation changed the node's cached flag bits. It does require
the original map and concrete class. A standard name is conservatively returned
as `Standard(id)` and remains unsupported unless its actual native behavior is
implemented, even if some standard-ID branch could eventually fall through.

The complete standard and prototype inventories are independent. For example,
`useHandCursor` is both standard ID 31 and a stored MovieClip.prototype key;
the attached-name layer still reports it as a standard member. Prototype
membership must not override native admission.

## Interning, hashing and ASCII normalization

The native name hash `0x1400340f0` starts with 5381, scans bytes in reverse,
folds ASCII `A..Z` to lowercase, and applies wrapping
`hash = hash * 33 XOR byte`. String factories retain its lower 24 bits.
This folded hash does **not** make all property names case-insensitive.

The interner lookup `0x140096e60` first compares byte lengths, then calls the
imported `MSVCR120.dll!strncmp` thunk at `0x141c6f31e` with that length. Within
the NUL-free ASCII profile this is exact byte equality. Hash collisions and
case aliases therefore remain distinct interned nodes unless their actual
bytes match. The existing dynamic factory `0x1400972e0` returns the existing
node on a match; a new node initially contains only the 24-bit hash. Constant
factory `0x1400971e0` ORs supplied flags into an existing node or adds them to
a new constant node.

Consequently a retained name and newly supplied identical bytes passed through
the same canonical interner have the same name identity; construction does not
erase standard flags. The ordinary-name theorem is also robust to either bit-29
state on a nonstandard name. This is not permission to treat arbitrary forged
nodes or cross-interner references as interchangeable. Upstream Value-to-name
coercion and string-construction instructions retain their separate contracts.

Normalized-node creation follows
`0x140097b50 -> 0x140036750 -> 0x140072070`. The latter's bitmap and sorted
676-entry character-pair table were read through REA. A separate static
interpretation checked all 128 ASCII inputs: exactly `A..Z` change, by `+32`.
The ASCII decoder/encoder preserve the other bytes; the encoder at
`0x14006acd0` writes the resulting byte directly. The buffer helper appends
the explicit byte counts using imported `memcpy`, then the dynamic interner
provides the cached normalized node. Allocation failure and non-ASCII
normalization are outside the admitted profile.

This evidence explains the native algorithm. The Rust classifier uses exact
membership in an authenticated semantic inventory, not a copied native hash
table or its bucket order. It supplies no enumeration-order contract.

## Underscore and cached-flag routes remain distinct

Complete helper `0x14018ff00` gives original-node bit 29 precedence. That path
uses the original node and exact lookup `0x1402853c0`; an exact miss does not
retry normalized lookup.

Without bit 29, only a leading `_` can enter the alias gate. It obtains the
normalized node and requires bit 28 on that node. Helper `0x1401823b0` then
uses the **original** node's folded hash to select a bucket, but compares the
cached normalized-node pointers of the input and candidates. Candidate caches
are populated when needed. The hash's ASCII case folding is therefore
consistent with this alias lookup. No version field is read in these helpers.

All 28 table entries marked with bit 28 are lowercase ASCII, at IDs 0..21 and
110..115. `_parent` and `_lockroot` are the only underscore table entries
without that bit. Under the ordinary fresh-node state, `_X` can select `_x`
through normalization; a fresh `_PARENT` fails the bit-28 gate. Non-underscore
case variants such as `TEXT` never enter this alias route. A previously marked
original node instead takes the exact route, even for an underscore spelling.

Failure of this initial native-ID gate does not establish a final underscore
miss: the attached getter has a later underscore-specific fallback. The first
Rust profile therefore recognizes exact standard names, but rejects all other
leading-underscore names as `UnsupportedName::UnderscoreRoute`. It does not
implement an incomplete alias or `_level` grammar. Variable-path parsing is
also separate; raw member classification never splits dots, colons or slashes.

## Baseline absence versus current-runtime absence

The complete original MovieClip.prototype inventory has 44 stored names;
Object.prototype has nine. Their only mutual overlap is `constructor`, giving
52 distinct stored names. Complete native raw getters add four reserved reads:
`constructor`, `__constructor__`, `__proto__`, and `__resolve`. The union has
54 exclusions, since two reserved names are already stored.

MovieClip's complete getter `0x1401e6020` intercepts the two constructor names;
generic `0x140197000` intercepts the prototype/resolver names, then performs
ordinary own-hash lookup and follows the parent link. For any other canonical
ASCII name outside the complete stored-name union, the unchanged native own
tables cannot match. Native construction and bootstrap establish zero resolver
fields on both objects. With MovieClip.prototype linked to Object.prototype
and Object.prototype linked to null, the getter reaches `0x140197160` and
returns false without altering the output Value or calling an accessor or
resolver. Raw false is not an implicit write of Undefined.

That is a general theorem about the original chain. It is not a theorem about
all later script state. The module deliberately returns only
`BaselineMembership::{Reserved, Stored, Absent, UnsupportedName}`. In
particular, `Stored` remains authoritative when the Rust VM has not yet
implemented that native property. The current diagnostic MovieClip model's
small native binding subset is not a complete inventory.

Only the VM's existing live-state checks may turn baseline `Absent` into an
actual missing result. They must preserve:

- Own storage first, including a found Undefined value; named-child lookup
  next with complete supported child state; then prototype lookup.
- Supported script-added properties and accessors before baseline absence.
  An unsupported callback, resolver or unknown native property remains a gate.
- Exact current builtin identities, parent/constructor links, version context
  and resolver state. Untracked parent changes, resolver insertion history or
  unsupported deletion/replacement cannot be ignored.
- The distinction between read membership and writes: admitting an ordinary
  read name does not bypass native pre-write hooks, watches, accessor setters,
  read-only flags or the bound-storage receiver rules.

`prototype`, `_listeners`, and `on...` are not extra read-reserved prefixes in
these two prototype getters. Their meaning to other object kinds or setters
does not justify a read shortcut. Their upstream attached classification and
all current-state guards still apply.

## Private input format and API

The parser accepts only the version-1 format for build `1.52.0.1700`. Inputs
are capped at 64 KiB and 174 LF-terminated ASCII lines. The four header lines
are the schema/version, build, lowercase source SHA-256 and exact counts
117/44/9. Body records contain, in order:

1. 117 `standard` rows with ascending IDs 0..116, native alias-selector 0/1,
   and exact names.
2. 44 sorted `movieclip` name rows.
3. Nine sorted `object` name rows.

The parser rejects extra/truncated records, malformed fields, unsupported
schemas/builds, incorrect IDs/alias selectors, duplicate names within roles,
noncanonical ordering, source mismatch and incorrect baseline overlap/shape.
It accepts the independently meaningful standard/prototype overlap above.
The source file contains only grammar, counts and necessary semantic reserved
identifiers; no 117/44/9 native-name arrays are embedded.

`NativeNameProfile::parse_verified` receives the exact input byte buffer,
`ExpectedProfile` (trusted build, source SHA-256 and private inventory SHA-256),
and a caller-owned SHA-256 callback. The callback receives that exact buffer;
its returned digest must equal the expected inventory digest before parsing.
The source identity header must match the caller's independent expectation.
The module adds no hashing dependency and makes no claim that it measured the
actual executable. The caller must validate that source file and use a real
trusted digest implementation; a profile's self-declared hash is insufficient.

The resulting immutable profile exposes:

| API | Meaning |
| --- | --- |
| `classify_attached_name(&[u16])` | Exact standard ID, supported ordinary name route, or explicit unsupported name. |
| `native_baseline_membership(&[u16])` | Reserved read, stored-native owner role(s), original-baseline absence, or unsupported name. |
| `source_sha256()` / `inventory_sha256()` | The accepted identity receipts, for integration diagnostics. |

Name inputs are bounded to 1,024 UTF-16 code units and must be nonempty NUL-free
ASCII. No folding, trimming or path parsing occurs. The profile does not hold
mutable object state, load files, execute native code or set a VM property.
AVM1 owns any optional `Arc<NativeNameProfile>` install hook and path-module
inclusion, so this increment does not create duplicate module identities or
change central VM semantics.

The canonical private fixture is
`evidence/rea-attached-name-classification/native-name-profile.v1.tsv`, SHA-256
`902b4f15629d3ae32aa6867165d275f2ef3ea289ced3696b9bf132155fd00b91`.
It is an owner-private runtime input and must stay out of git and build
packages. Other installs/profiles need their own trusted extraction and
expected digest; the module cannot invent a replacement dataset when absent.

## Validation and evidence

Six focused standalone Rust tests cover digest-buffer binding/failure, byte and
name bounds, malformed/truncated inputs, duplicate/incomplete IDs, source/schema
substitution, baseline shape, and the separation of admission from membership.
They use generated synthetic names, not extracted tables.

A separate private smoke loaded the actual 4,226-byte profile with a real
`sha256sum` callback over its exact bytes. It checked all 117 standard IDs and
53 prototype rows, 12 previous ordinary/absent frontier names, independent
standard/prototype overlap, the underscore gate and rejection of a tampered
buffer. This tests the original Rust classifier against the retained native
inventory; it is not execution of the retail getter or a runtime-parity claim.

Private evidence under `~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `rea-attached-name-classification/combined-proof-receipt.json`: joined
  artifact/record identities, focused code checks and limits.
- `rea-attached-name-classification/final.bundle.json` and
  `buffer-final.bundle.json`: normalization, interner comparison, ASCII mapping
  data and exact buffer-copy/import paths; all claims bounded to read branches.
- `rea-attached-name-classification/ascii-fold-derived.json`: static check of
  all 128 ASCII code points against the verified native mapping.
- `rea-attached-name-classification/profile-fixture-receipt.json`,
  `profile-smoke.rs` and focused test outputs: private data provenance and Rust
  validation, with no extracted fixture included in the repository.
- `rea-attached-name-classification-map/finding-summary.json`: exact/normalized
  map branches and cached-flag precedence.
- `rea-prototype-general-classification/finding-summary.json`: complete
  baseline partition, concrete getter argument and required runtime guards.

Earlier canonical hash/interner/map/constructor inventories are reused from
[REA_ENUMERATION.md](REA_ENUMERATION.md),
[ATTACHED_MEMBER_NAMES.md](ATTACHED_MEMBER_NAMES.md) and the private receipts.
New REA sessions were exported/reset; instruction/data byte checks and focused
Rust tests passed. No original client, live server or full suite was run.

## Provenance

| Source | Licence / status | Use |
| --- | --- | --- |
| Owner's pinned `MarvelHeroesOmega.exe` and private native inventory receipts | Gazillion/Marvel proprietary artifact; original project analysis | Exact finite membership and control-flow evidence for interoperability. Raw bytes, strings and generated profile remain private. |
| REA 6.9.0, reverse-engineer-anything skill and connection/native guides | MIT toolkit and local instructions | Supported bounded evidence capture and lifecycle. |
| Rizin provider | LGPL-3.0-only; analysis tool | Selected native instructions, tables and import decoding. |
| Earlier project native studies named above | Original project analysis | Reused complete inventories, hash/interner facts and constructor/resolver guards. |

The parser, classification code, format, synthetic tests and private verification
helpers are original. No SDK source or third-party VM implementation was used.
