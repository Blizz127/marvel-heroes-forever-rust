# Configured localization-label resolver

`frontend::localized_label` implements the bounded resource lookup established
in [LOCALIZED_LABEL_LOOKUP.md](LOCALIZED_LABEL_LOOKUP.md). It resolves the saved
Globals and typed directory Asset before deriving a label's prototype identity.
It returns a result only after exact resource membership and the existing
UILocalizedInfoPrototype scalar profile have passed.

```rust,ignore
let mut labels = ConfiguredLabels::open(calligraphy_path, Limits::default())?;
let resolved = labels.resolve("Frontend/UIButtonLogIn")?;
let data = resolved.prototype().localized_data(&explicit_locale_catalog)?;
```

The caller still selects a locale catalog. This module installs no callback,
creates no VM object and runs no startup scripts. A successful resource lookup
does not imply an initialized native frontend or retail behavior parity.

## Source and result contract

`ConfiguredLabels<R>` owns one `SipArchive<R>`; `open` uses one
`BufReader<File>`, while `from_reader` accepts another owned seekable source.
The prototype directory, blueprints, type data and subsequent prototype bodies
all come from that handle. Replacing the original pathname cannot redirect
later reads. This read-only API does not prevent another process from changing
the contents of the already-open file in place; callers must use their pinned,
unchanging source.

The exact native-selected member is `Calligraphy/Globals/Globals.defaults`.
Its required `UIGlobals` declaration must be a simple Prototype reference with
a nonzero subtype. The explicitly stored reference must resolve to that exact
blueprint, with binding `UIGlobalsPrototype`. Its required
`LocalizedInfoDirectory` declaration must be a simple Asset reference with a
nonzero subtype. The subtype selects an exact Type.directory record; the Asset
ID must have one unambiguous entry in that type. All declared type files are
checked for a second occurrence of the selected Asset ID, since native lookup
uses the global reference identity. No directory name is supplied as a default.

Both configuration objects must be their blueprint's explicit default, with
the observed directory flag value `1`, an explicit null data parent and no
schema parents/contributors. Copy numbers must be zero; declarations and stored
assignments must be unambiguous. A missing field or zero reference is an error.
Other configuration fields remain parsed source data and are not evaluated as
runtime objects. Arbitrary inheritance and native lookup fallback are rejected.
Ordinary localized child rows use observed flags `0`; their default parent rows
use observed flags `1`. These are restrictions of this tested source profile,
not a general interpretation of all directory flags.

`Configuration` retains both complete `PrototypeValues` views, the selected
`FieldOrigin` for each link, the Type.directory record/index and the selected
Asset record/index. These identify prototype IDs, source members, declarations,
layers, groups and field positions. Current source parsers do not retain the
raw byte ranges of these fields; the API does not invent those ranges.

`ResolvedLabel` owns `label`, `joined_path`, `hash_bytes` and `prototype_id`.
Its `prototype()` returns the existing `PreparedPrototype`; `configuration()`
borrows a retained shared configuration snapshot. Results remain usable after
the resolver is dropped. Output text references and their origins come from
[LOCALIZED_PROTOTYPE_FIELDS.md](LOCALIZED_PROTOTYPE_FIELDS.md)'s explicit
parent-first scalar profile. Its `localized_data` method applies the separate
present-entry/plain-text locale profile.

## Label and identity profile

Input labels and the configured directory must be ASCII without NUL. Empty
labels return an explicit unsupported error, consistent with the saved script
wrapper's guard. The native incoming conversion uses Windows code-page value
zero; this implementation does not treat arbitrary Unicode input as UTF-8 or
silently replace characters.

The resolver appends `.prototype` to the complete label, inserts `/` only when
the nonempty directory does not already end in `/` or `\`, and preserves all
other bytes. The simultaneous substitutions are `A..Z → a..z`, `/ → .`, and
`. → ?`. Backslashes remain unchanged. It computes the evidenced combined
CRC-32/Adler-32 value and wrapping decrement, then looks up that exact ID.
There is no case-insensitive filename search, separator rewriting, archive-prefix
insertion, basename search or guessed resource. Case variants work through the
native byte transformation. A nonzero hash without a resource is an error.

The exact resource must pass the existing `localized_prototype::resolve`
class/schema, parent, field and budget checks before a public result exists.
There is deliberately no public unchecked-label-to-ID success API.

## Explicit callback resource backend

`frontend::localized_callback::LocalizationCallback` combines this resolver with
an explicitly selected `LocaleCatalog`. `resolve_call` accepts the saved wrapper's
exact `GetLocalizedObjectDataByLabel` name and one nonempty NUL-free ASCII String
label, then returns the VM's fresh-object field description through
`LocalizedObjectData::into_host_result`. It does not install ExternalInterface,
select an Environment/movie, or manufacture an Unreal receiver. The integrating
host must establish those associations before routing a request here.

The backend checks request and UTF-16 input bounds before conversion or lookup.
Unsupported callbacks, argument types, labels, resources and text-formatting
paths return explicit errors. There is no invented failure object or fallback.
It retains only the last successfully resolved resource, including configuration
and field origins. Rejected requests preserve that record. Its `resolved` counter
means successful resource preparation; subsequent VM allocation and ordinary
setters may still fail and are not counted as completed script invocations.

## Budgets and errors

`Limits` exposes SIP index/member budgets, total decoded metadata bytes,
directory-record and blueprint counts, total declared Asset entries, label and
path bytes, configuration `ValueLimits`, and per-result localized prototype
`ValueLimits`. Each configuration object is additionally limited to one layer.
Metadata sizes/counts are checked before decompression or record parsing;
prototype parse budgets are derived from the corresponding `ValueLimits`,
followed by the existing cumulative value/parent accounting. Results do not
accumulate inside the resolver; callers control retained result lifetimes.

The error surface is `io::Result`. Unsupported input or an unproved profile
uses `Unsupported`; absent resources use `NotFound`; malformed or ambiguous
data and resource-budget failures use `InvalidData`. Direct file/metadata I/O
errors are preserved; the existing prototype-value adapter reports body-load
failures as decoded-source `InvalidData` errors. No failure returns a fabricated empty localization object or a
default tooltip bit. These typed rejections are an explicit restricted profile,
not an emulation of all native diagnostic/failure branches.

## Focused verification and provenance

The module's fabricated fixtures exercise configured directory selection,
typed reference links, exact membership/class checks, retained origins, null
references, configuration parents, duplicate assignments/identities, an Asset ID
appearing in another type, missing type/resource, label restrictions and resource
budgets. Independent CRC/Adler vectors and literal path distinctions are checked.
They contain no original game assets or extracted data.

Actual-data comparison uses the frozen independent 27-key oracle under private
`evidence/localized-prototype-oracle/`. The validation helper consumes the public
API without inspecting the pending resolver implementation. It compares all
representable prototype/text IDs and origins; unavailable raw byte spans are
reported separately. Private receipts reside in
`evidence/localized-label-resolver-check/` and focused build artifacts in
`evidence/localized-label-resolver-build/`.

**VERIFIED on the frozen implementation:** all four focused fabricated-fixture
tests passed. The independent actual-data run matched 27 prototype identities,
54 String references, 54 representable field origins and 16 configuration
fields; all 11 boundary/budget checks passed. Final output was byte-identical
to the corrected preliminary run. Raw field byte spans remain unavailable,
and no native runtime parity is claimed.

| Private receipt/artifact | SHA-256 |
| --- | --- |
| `localized-label-resolver-check/receipt.json` | `7b50951f202019813a9d5c320987cf83b2be968ff5dbb9005ecef5cab0152b48` |
| `localized-label-resolver-check/readback.json` | `b5a4ec6e50dc5cdf9921c94080db3798a1cf8853c1da2693ae63f7f77615798c` |
| `localized-label-resolver-check/actual.tsv` | `202f1d1900ec4137ce9927e1f58ce566e11443db72206ffa2e8e2c830d152c10` |
| `localized-label-resolver-build/receipt.json` | `b0f71a4be54c3ff079a63a2f5fbc3be1c39b1013f9875137fdeb5caf1cd56996` |
| Tested `src/frontend/localized_label.rs` | `b334333f159ad46d02cf13ea5b5b7dda2fa6b4a0f4d684e2c32f6d8d9617664a` |

The independent receipt verifies 29 artifact hashes, and the focused build
receipt retains the source/dependency hashes and standalone `rustc` commands.
The temporary wrapper supplies standalone module wiring; product frontend
module/host wiring is left to the integrator. No routine full suite was run.

The first actual open check caught an overly strict directory-flag rejection:
the owned configuration defaults carry `1`, whereas the initial implementation
required `0`. The failure receipt is retained. The implementation and fabricated
fixtures were corrected to the observed row profile before successful results
were claimed.

Implementation is original. Sources read were existing project Calligraphy,
SIP, prototype and locale APIs; the prior native contract/REA receipts and
private independent oracle; and the owner's pinned Calligraphy archive through
the focused validator. These private proprietary sources remain outside git.
No outside implementation, website, ClientConfig, account data or network
service was read. The proof's exact executable/archive identities and source
licenses are recorded in [LOCALIZED_LABEL_LOOKUP.md](LOCALIZED_LABEL_LOOKUP.md).
