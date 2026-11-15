# Original locale resources and frontend key lookup

The English resources are available and privately pinned. The reader loads exact
resource text and preserves missing values, variants, flags and formatting tokens.
It does not replace missing translations with authored labels.

## Resource identity and actual layout

**VERIFIED (complete file hashes):** Five files, totaling 5,730,074 bytes, were
copied read-only from the existing private client candidate into
`~/.local/share/marvel-heroes-forever-rust/pins/frontend-locale/Data/Game/Loco/`.
Every source and destination SHA-256 matches the Alienware distribution inventory;
source size, modification time and inode remained stable during the copy. The
operation did not alter the candidate transfer or access accounts/live state.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `eng.all.locale` | 72 | `05b8991566d1c4320b6949489013c6b9c465d322f5b864d0ca66f95f946ca3e3` |
| `eng.all/eng.all_3FFFFFFFFFFFFFFF.string` | 1,422,751 | `9e566ae940584f52afb1f6b013dd6ce738825efca7ad35bee5880c2b22b9ec05` |
| `eng.all/eng.all_7FFFFFFFFFFFFFFF.string` | 1,444,201 | `5bb3d29579bb3339ec416457dedd3823ff5f81dec27ce3e5401f82d39c744e30` |
| `eng.all/eng.all_BFFFFFFFFFFFFFFF.string` | 1,432,504 | `7e0dcbd7c1c9857dcec42a1328c580ba9a8429325578a32522f6089efcbf1070` |
| `eng.all/eng.all_FFFFFFFFFFFFFFFF.string` | 1,430,546 | `8f605835a8ae527d225b985007d3fc1eb32d009947270d90fa1e272cdd3c1f27` |

**OBSERVED:** `LOC` plus version byte 2 introduces four u16-length-prefixed UTF-8
strings, followed by a u8 flag count. Each flag has u16 value/mask fields and a
length-prefixed UTF-8 label. The English metadata names English/Everywhere,
directory `eng.all`, and four flags. The directory is retained as metadata.

`STR` plus version 2 introduces a u16 entry count. Each entry contains a u64 ID,
u16 variant count, u16 produced flags and a u32 absolute text offset. Additional
variants contain u64 consumed flags, u16 produced flags and a u32 offset: **14
bytes** each. Default and variant strings are NUL-terminated UTF-8. Counts of 0 or
1 have no additional variants in the reader; only positive counts were observed
in these files. Text spans exclude their terminator or length prefix.

The reference documentation's wider locale-count and variant-produced-flags
descriptions disagree with its implementation and the actual files. The reader
uses the widths confirmed by consuming all five owner files exactly.

**OBSERVED (independent scan):** 77,089 unique default IDs, 138 alternate forms
and 77,227 total strings. There are 5,187 non-ASCII strings and 6,321 empty strings.
The four default-entry counts are 19,317 / 19,159 / 19,276 / 19,337, with 34 / 27 /
42 / 35 alternates. Table ends are 309,554 / 306,928 / 309,010 / 309,888. All text
is strict UTF-8, all page bytes are covered, and no ID duplicates occur across
partitions. ID zero has an actual empty default string; a missing ID is distinct.

## Exact UI key mapping

**OBSERVED:** Keys such as `Frontend/UIButtonLogIn` identify
`Calligraphy/UI/LocalizedInfo/Frontend/UIButtonLogIn.prototype` in the owner's
Calligraphy archive. The original schema-aware prototype reader resolves its
`UILocalizedInfoPrototype.DisplayText` and `TooltipText` fields as string IDs.
No locale-ID hash or match by visible text is needed.

All 27 `Frontend/` prototypes were separately pinned under
`pins/frontend-locale/Calligraphy/UI/LocalizedInfo/Frontend/` (1,657 bytes total).
Their source archive has SHA-256
`1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c`, verified before
extraction. Every directly serialized display ID joins to the English table.

| Key | Serialized display-string ID |
| --- | --- |
| `Frontend/UIButtonLogIn` | `0x590a017f2861053f` |
| `Frontend/UILabelPassword` | `0x23e6e0df2835053a` |
| `Frontend/UIButtonRegisterNow` | `0x4b63904324e1050c` |
| `Frontend/UIButtonForgotPassword` | `0x83a14bd124f3050e` |
| `Frontend/UIButtonOptions` | `0xe3b8446724ab0508` |

The complete strings and mappings remain private in
`evidence/frontend-locale-study/frontend-key-mappings.json`, SHA-256
`f5c60afa4b8767e0e8de908e6418d653a2b38a3903de81fc2ddcbb120a823a24`.
For example, the first entry resolves to the original text “Log In”. Uppercase,
punctuation, Unicode, `$...$` tokens and markup are retained exactly.

## Interface and boundaries

`LocaleMetadata::parse` and `StringTable::parse` read bounded in-memory files;
their `_with_limits` forms accept `LocaleLimits`. `StringTable::entries()` exposes
ordered records; `get(id)` returns `Option<&StringEntry>`. Each entry retains its
default text, stored count/flags, source offsets and alternate forms. The reader
does not choose grammatical variants or interpret tokens/markup.

`LocaleCatalog::open(metadata_path, string_paths)` accepts explicit paths and
combines partitions with cross-file duplicate checks. The serialized directory
never becomes an automatic filesystem path. `new(metadata, tables)` composes
already parsed resources. `get(id)` preserves missing versus empty text.

`UiKeyBinding::read(&mut PrototypeReader, key)` resolves an exact validated
archive key through existing Calligraphy schemas. It returns `None` for an
unknown key. Present bindings preserve separately optional display/tooltip IDs
and the serialized parent reference. An absent directly serialized field remains
absent: parent-value inheritance is not implemented here. A present ID may still
be missing from the selected locale, which the catalogue reports as `None`.
Duplicate text fields and unsupported field ownership fail explicitly.

Bindings also retain `prototype_id`, obtained from the validated directory's
exact member record. It is distinct from both text IDs and the body's parent
reference. `prototype_reference_string()` uses the native successful lookup's
`0x%llX` spelling (lowercase prefix, uppercase unpadded hexadecimal), established
in [the native localization trace](FRONTEND_LOCALIZATION_RUNTIME.md). This exact
identity is needed by the authored `localizedPrototypeIdStr` consumer. It does
not turn directly serialized fields into fully resolved native values or build
an ExternalInterface result before its remaining contracts are implemented.

**VERIFIED (saved identity):** An independent walk consumes all 85,186 raw
directory records and matches the new IDs for all 27 original frontend keys.
Reference strings have the native uppercase/unpadded spelling; display/tooltip
IDs match the prior independent byte-derived table unchanged. Unknown and
case-folded archive names reject. The private fixture's initial standalone-module
path error was corrected without changing production code. Records and source
hashes are in `evidence/localized-prototype-identity/verification.json`.

Default table budgets are 64 MiB input, 65,535 entries, 255 locale flags, 256
alternate forms per entry, 65,536 total alternates, 1 MiB per string and 64 MiB
cumulative copied UTF-8 bytes. Partitions require bounded index/text offsets,
strict UTF-8 and exact text-page coverage. Identical text-span reuse is allowed;
partial overlap, gaps, trailing bytes and unterminated strings are rejected.
Catalogue loading is capped at 64 partitions, 256 MiB combined file bytes and
one million default IDs. No fallback locale, formatter, UI state or font choice
is inferred from successful parsing.

**VERIFIED:** Eight focused tests pass for Unicode/spans, variants, valid empty
text, malformed encoding/termination, offset overlap/gaps, truncated counts,
resource budgets, cross-partition collisions and invalid keys. The actual reader
matches the independent Python scan field for field for all 77,089 IDs and 138
alternates. The integrated catalogue and key bridge also resolve all 27 actual
frontend keys to the same IDs and exact English bytes. An unknown key, unknown
ID, absent tooltip field and stored blank remain distinct.

## Private evidence

All files below are under `evidence/frontend-locale-study/`:

- `pin-receipt.json`: complete locale-file copies and hashes; SHA-256
  `15cdfaa218bfa1b6eff7cd04f46653b3116bf89e4269b277dc4be391cc426dcd`.
- `locale-scan.json` and `locale-entries-private.json`: independent original
  structural scan and complete private decoded records.
- `key-assets-receipt.json`: 27 selected prototype identities; SHA-256
  `1c281ad68d6b2fa8c76aa6726df303f1bd609aff9b61f20a2b44766a281d976f`.
- `key-probe.rs`, `frontend-key-probe.txt`, `frontend-key-mappings.json`: original
  schema-aware field reads and joins to the independently decoded English data.
- `rust-python-comparison.json`: complete record/flag/span/text comparison;
  SHA-256 `0f103b6e303ba4355e2ca940d9f7288821137c26fb5e3d336056f9a54329930f`.
- `integration.rs`, `integration-tests.txt`, `integration.tsv` and
  `integration-validation.json`: final catalogue/key smoke, eight focused tests
  and implementation hashes. Validation receipt SHA-256:
  `d168f6015c6be6c302cff70375d1ab54b1dfec58b8f240f28763d0d9292f519d`.
- `source-ledger.json`, `independent-sources.json`, `combined-sources.json`:
  exact outside-source paths, revisions, hashes, read scopes and uses. The
  combined 35-file ledger has SHA-256
  `c5762ac76aa35aeaa59a54ff7dd459797b36773d7d8bcf194242e925988818d0`.

No original locale files, text tables, prototypes or captures are committed or
embedded in a build. These are static file/parser observations, not REA or retail
runtime/rendering evidence.

## Source and licence register

The MHServerEmu checkout studied was
`/home/blizz/dev/MHServerEmu`, revision
`1199f651f33977062a5dbd7937d9ea9234f7076c`. All code here is original Rust; the
following AGPL sources were studied for compatibility and were not copied.

| Source read | Licence / rights | Use |
| --- | --- | --- |
| [MHServerEmu locale documentation](https://github.com/Crypto137/MHServerEmu/blob/1199f651f33977062a5dbd7937d9ea9234f7076c/docs/GameData/Locale.md) | AGPL-3.0 | Format reference; discrepancies resolved from actual bytes. |
| Same revision, `src/MHServerEmu.Games/Locales/{Locale,LocaleManager,LocaleString,LocaleSerializer}.cs` | AGPL-3.0 | Header widths, index/variation records, UTF-8 page encoding and flag preservation. |
| Same revision, `GameData/Calligraphy/CalligraphyReader.cs`, `GameData/DataDirectory.cs`, `GameData/DataRefTypes.cs` under `src/MHServerEmu.Games/` | AGPL-3.0 | Header/version constants and ID type; only relevant sections studied. |
| Same revision, `GameData/Prototypes/{TypesPrototype,UIPrototype}.cs` under `src/MHServerEmu.Games/` | AGPL-3.0 | Translation and UI-localized field type declarations. |
| Same revision, `src/MHServerEmu.Core/Helpers/HashHelper.cs`, locale-ID command excerpts, `docs/ServerEmu/ServerCommands.md`, and root `LICENSE` | AGPL-3.0 | Candidate-hash investigation and licence identity. Hash helpers are not used for locale key resolution; commands were not executed. |
| Owner's locale files, inventory, Calligraphy archive and selected prototypes | Proprietary game data; authorized private interoperability study | Actual format, Unicode/string identities and typed frontend-key mapping. |

Exact file hashes and URLs are in the private ledger. Filename/symbol discovery
also searched the local MHServerEmu forks read-only. Existing original project
SIP, Calligraphy and prototype-catalogue readers are reused with their prior
provenance. Python standard-library utilities were used for private checks; no
new client dependency or reference implementation was introduced.
