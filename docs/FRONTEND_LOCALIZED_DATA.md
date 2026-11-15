# Successful localized result from explicit references

`frontend::localized_data::resolve` constructs the four fields observed in the
native localization result from a parsed locale catalog and explicitly supplied
effective prototype, display-string and tooltip-string references. It is a pure
data operation, separate from an ExternalInterface callback or VM object.

**OBSERVED:** The native profile and evidence are recorded in
[the localization runtime trace](FRONTEND_LOCALIZATION_RUNTIME.md). The callback
uses selector UINT64_MAX, which selects the standard loader's default entry
before any alternates. In the established no-marker path it preserves the text
and converts validated UTF-8 to UTF-16. The prototype identity is formatted as
`0x` followed by uppercase hexadecimal digits without padding. `hasTooltip`
tests the tooltip reference, including when a nonzero reference selects empty
text; it does not test text length.

The original Rust result preserves that profile:

| Returned field | Source |
| --- | --- |
| `localized_prototype_id_str` | Explicit prototype reference with native spelling |
| `localized_text` | Default entry for the explicit display-string reference |
| `localized_tooltip_text` | Default entry for the explicit tooltip-string reference |
| `has_tooltip` | Explicit tooltip-string reference is nonzero |

Each string also retains its catalog partition and original entry/text byte
spans as provenance. Those metadata fields are not ActionScript properties.
Strings retain their validated UTF-8 without normalization; `encode_utf16`
provides the lossless conversion at a future VM seam.

The resolver rejects zero prototype identity, absent string IDs, `#` formatter
markers and the native missing-translation sentinel. It does not synthesize
empty strings, select an alternate form, apply a fallback locale or interpret
formatting. A zero tooltip reference is still a string lookup: this code does
not invent its native fallback behavior when that entry is absent.

**Boundary:** Directly serialized optional
`UiKeyBinding` fields do not establish effective native prototype values.
Deserialization may apply inherited or class defaults. There is no implicit
conversion from those optional fields to this module's `References`; callers
must resolve that separate contract first. The separately evidenced
[localized prototype preparation](LOCALIZED_PROTOTYPE_FIELDS.md) now supplies
references for its bounded same-blueprint scalar profile. The runtime callback, native receiver,
label-path lookup, object allocation and failure-return semantics remain outside
this module. A successful pure lookup does not establish frontend execution or
wire/behavior parity.

## Focused verification

**VERIFIED against original stored locale bytes:** A private dependency-free
harness checked all 77,089 default entries in the four English partitions.
70,103 results preserved their raw text bytes, source partition/entry/text spans
and UTF-16 roundtrip; 5,111 of those contained non-ASCII characters. The remaining
6,986 entries correctly rejected the scoped unsupported marker paths: 6,983 with
`#` and three with the actual missing-translation sentinel.

6,316 nonzero references to empty default text produced `has_tooltip=true`.
The actual stored zero-ID entry is empty and produced false. 73 supported
entries with alternates selected their base string; an independent raw case
confirmed a distinct alternate was not substituted. Eight independently selected
raw cases also verified identity spelling, both string slots' marker rejection,
missing-ID errors and zero-prototype rejection. No supplementary-plane character
occurs in the 77,227 stored strings, so that character class was not exercised by
this original-data check.

These checks supply explicit reference values; they do not infer effective
prototype fields or invoke the native/VM callback. The frozen source and input
hashes are in private `evidence/localized-data-check/receipt.json`, SHA-256
`abf813937c5d5d149dd910d96234c62eca354f5f615cb7e3ef73375207a66d45`.

## Provenance

The implementation uses original Rust and the standard library. Its native
contract comes from REA study of Blizz's owned build 1.52.0.1700, registered in
the linked runtime trace, notably private
`evidence/frontend-localization-runtime/aggregate-followup/resource-profile/verification.json`
(SHA-256 `71936d5868d94d6dbe6a2eff31d45dfaba11849d3a12446e55074f1f7fd59ff5`).
Locale parsing retains the source ledger in [localization](LOCALIZATION.md).
No extracted translation, native code or game asset is included in the repo or
private Rust build packages.
