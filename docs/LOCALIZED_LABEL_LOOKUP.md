# Configured localization label lookup

**OBSERVED in the owner's build 1.52.0.1700:** localization core
`0x1413b7010` obtains its directory through the configured Globals resource,
then constructs and hashes a prototype path. The stored directory is exactly
`UI/LocalizedInfo`. It is an Asset reference resolved through the owned type
data, not a hardcoded default inferred from a filename.

**VERIFIED by an original static-byte probe:** the recovered ASCII transform
and hash reproduce all 27 previously frozen frontend prototype IDs, including
`Frontend/UIButtonLogIn` → `0xee354851ea1813f3`. This is static compatibility
evidence. No retail process, VM, native callback or startup sequence was run.

## Globals selection and stored source identity

The native setup function `0x14169a7b0..0x14169b163` passes the literal
`Globals/Globals.defaults` at `0x14240ac90` to name lookup `0x1413e98b0`.
At `0x14169ab15` it stores the result in its context at `+0x390`, then tests
against the invalid-reference sentinel. The failed branch reports a diagnostic
and returns failure. This establishes the selected resource name on that
successful initialization path; it does not establish that initialization ran
in a particular session. No ClientConfig file was read or copied.

The lookup visits its linked name records and copies the matching record's
reference. Its comparison helper's broader matching rules were not recovered;
the exact owned Globals name is the supported source identity here.

The localization core uses global context pointer `0x142c5c0e8` and calls
`0x14169a640`. That getter lazily resolves context `+0x390` through
`0x141563ad0`, checks/binds a GlobalsPrototype through `0x141657390` and
`0x14162ada0`, and caches the object at `+0x398`. A missing object reports a
diagnostic and returns null. A present object returns its `UIGlobals` field
at `+0x438`. The core then addresses `UIGlobals + 0x1c0`.

The generated metadata independently names these fields:

| Field | Native descriptor | Native type | Object offset / width |
| --- | --- | --- | --- |
| GlobalsPrototype.UIGlobals | `0x142aac820` | 33 | `0x438 / 8` |
| UIGlobalsPrototype.LocalizedInfoDirectory | `0x142a9b300` | 15 | `0x1c0 / 8` |

The independent raw SIP cursor established the corresponding stored chain:

| Source | Stored field / value | Value bytes within uncompressed member |
| --- | --- | --- |
| `Calligraphy/Globals/Globals.defaults`, ID `0xbb591dc280950a0a` | Simple Prototype `UIGlobals`, field ID `0x6271b98965f010b1`, value `0x4e5292ae96cc0ae8` | `[2277,2285)` |
| `Calligraphy/Globals/UIGlobals.defaults`, ID `0x4e5292ae96cc0ae8` | Simple Asset `LocalizedInfoDirectory`, field ID `0x8dd15c2197461705`, value `0xde037edeac771053` | `[69,77)` |
| `Calligraphy/UI/Types/UIDirectory.type` | Asset `0xde037edeac771053`, GUID `0x8aa2eb901dcc0450`, name `UI/LocalizedInfo` | ID `[6,14)`, name `[25,41)` |

Both selected defaults explicitly encode null parents, and their blueprints
have no parents or contributors. Each blueprint has exactly one Prototype
directory row in this archive. `LocalizedInfoDirectory` has Asset subtype
`0x50657df181d40a23`; Type.directory record 213, bytes `[11707,11751)`, binds
that subtype to `UI/Types/UIDirectory.type`. The type file contains one entry.
The saved Prototype-valued field and the native resolved pointer are different
representations; the table does not imply that a serialized ID is a pointer.

## Directory and label operations

The complete selected functions support this order:

1. The core rejects a null UIGlobals pointer and an invalid directory reference
   through separate diagnostic branches.
2. `0x141343850..0x1413438b5` finds the directory Asset reference in the context's
   buckets at `+0x110/+0x118`, comparing exact eight-byte keys at node `+0x10`.
   A hit returns the stored string object at node `+0x18`. A miss returns the
   context's fallback string object at `+0x448`; its content is not established
   here. The core does not add a separate membership check after this call.
3. The input is an already-marshaled FString. The core selects its UTF-16
   buffer, using the shared empty buffer when the FString count is zero.
   Helper `0x14070b140..0x14070b20d` calls the imported `KERNEL32!lstrlenW`,
   then `KERNEL32!WideCharToMultiByte` with numeric code page **0**, flags **0**,
   input count `length + 1`, and output capacity `2 * (length + 1)`. Its return
   count is not checked in the selected helper. This is not an explicit UTF-8
   conversion. Host code-page behavior, conversion failures and non-ASCII labels
   remain unsupported by this contract.
4. The core formats the fixed narrow literal `%s.prototype`, then calls
   `0x1405a6d70..0x1405a6eff` to join it to the directory. The join copies an
   empty-side counterpart; with two nonempty strings, it copies the directory,
   appends `/` unless the directory already ends in `/` or `\`, then appends
   the entire second string. Suffix helper `0x1405a9150` compares one last
   byte. The selected join does not trim a leading separator, collapse `..`,
   remove an extension, or replace existing backslashes.
   The separately retained ordinary append branch at `0x1405a1f40` copies the
   supplied byte count with imported `MSVCR120!memcpy` to the current string's
   end and writes a terminator. The formatting helper uses the imported
   `_vsnprintf_s` with the fixed format; it does not interpret the label as a
   format string. Allocation failures and aliasing branches are outside the
   successful separate-buffer profile.
5. `0x141695d30..0x141695e43` copies the joined narrow string, transforms it
   through `0x141653a60`, and hashes it through `0x1405a8870`. It returns the
   computed reference directly. It does **not** search the Prototype directory
   for a matching filename at this point.
6. The core tests the computed reference against its invalid sentinel, loads
   the resource through `0x14158e290`, and checks it as UILocalizedInfoPrototype
   through `0x1413a9e20`. Unknown labels can therefore produce a nonzero hash
   and fail at resource loading. A hash alone is not evidence of membership.

## Exact supported ASCII identity

For ASCII, character helper `0x1405a5a60` and its 45-entry switch table apply
these byte substitutions once:

| Input | Output |
| --- | --- |
| `A` through `Z` | `a` through `z` |
| `/` | `.` |
| `.` | `?` |
| Other ASCII bytes, including `\` | Unchanged |

These are simultaneous character mappings: a slash becomes a dot, not a
question mark. A second helper, `0x1405a9eb0`, calls imported
`MSVCR120!tolower`; all uppercase ASCII has already been removed for these
inputs. The selected non-ASCII branches and CRT locale behavior are not a
general Unicode case-folding contract.

The hash consumes the resulting bytes without their NUL terminator. It combines
CRC-32 and Adler-32, then subtracts one with 64-bit wrapping:

```text
reference = ((u64(crc32(bytes)) << 32) | u64(adler32(bytes))) - 1
```

The actual 256-entry CRC table at `0x14255b050` exactly matches reflected
polynomial `0xedb88320`; the code starts at `0xffffffff` and complements the
result. The Adler accumulator starts at `(1,0)` and reduces modulo 65521.
Selected instructions explicitly combine CRC in the upper half, Adler in the
lower half, and decrement the combined value. The private probe independently
compared both components against Python's installed zlib results.

For the login label:

```text
joined: UI/LocalizedInfo/Frontend/UIButtonLogIn.prototype
hashed: ui.localizedinfo.frontend.uibuttonlogin?prototype
ID:     0xee354851ea1813f3
```

The owned Prototype.directory stores a backslash-separated archive path and
the same ID. Its stored path spelling is not the callback's hash input.
`Calligraphy/` is the package-member prefix and is not prepended here. The
private probe confirms that adding that prefix, changing the directory slash
to a backslash, adding a leading label slash, or inserting `./` changes the
computed login identity. Those four checks exercise the recovered arithmetic,
not actual native calls or all possible malformed paths.

## Original host implementation boundary

A bounded first implementation can resolve the exact native-selected Globals
source, require the two supported stored fields and their declared types,
resolve the directory Asset through the exact type table, and derive references
for NUL-free ASCII labels. Retain the label, directory reference and source
spans, joined bytes, computed reference, exact resource membership and class
validation. Feed only a successfully resolved UILocalizedInfoPrototype into the
separately proved field/locale resolver. Preserve explicit failures for missing
configuration, unknown type/asset, unsupported input and missing resources.

Such rejection is an explicit restricted implementation profile, not a claim
that native code rejects every corresponding input. The saved script wrapper's
empty/falsy-label guard, callback argument marshaling, arbitrary argument
coercions and lifecycle timing remain as documented in
[FRONTEND_LOCALIZATION_RUNTIME.md](FRONTEND_LOCALIZATION_RUNTIME.md).

No safe false/default result was recovered for all native failures: the earlier
core evidence still shows failure branches copying a tooltip bit from a local
temporary without an explicit preceding initialization. Directory-map fallback
contents, arbitrary schema inheritance, name-map comparison behavior, hash
collisions and Unicode/code-page cases remain unresolved. This work does not
activate ExternalInterface or establish an initialized frontend.

## Evidence, verification and provenance

All raw files, probes and extracted bytes remain private under
`~/.local/share/marvel-heroes-forever-rust/evidence/localized-label-lookup/`.

| Private evidence | SHA-256 |
| --- | --- |
| `native.bundle.json` | `e1071ca51de55127f313ab51d79f3328ce69a395cb2d64d8439466705c6145b8` |
| `string-helpers.bundle.json` | `31f20a8947534338c256dfc710db83bdad1f3fd4532948e1b037700a0ad9443f` |
| `string-helpers-verification.json` | `da450e43198395396307d9730013026846bfc0ac4c30cbd7d452873e7dc3481a` |
| `data/receipt.json` | `cdfdca072c930103d8dee5f83f5920150876fc4d252dfa6248d315e0f382b675` |
| `native-globals/receipt.json` | `f9298918f47de1465cd6bf22f7ba7857f19fda41fb086909f2d42319344abfe2` |
| `native-globals/imports-followup/receipt.json` | `8d768cfdd2b646d6273d0f77de6b84d17d09454d8f2aca608df42007e58a4211` |
| `path/findings.json` | `74449193591787ffbdb0e67e1dca2c6303e824a154400f2fdd6dc79e6aba7fdc` |
| `path/verification.json` | `beae167cac33223d492adce060ebec77416cffe777b97001aa73efa51e612243` |
| `path/receipt.json` | `12ebe6fe6b45989fd9d5c5ae3fe4f4494a318d21202d79edd3652f75fcbfe128` |

The new REA records are
`ev_b81f6a916413c935be66d4038ecea8f9f20df8197ccab9645e262472fa7ca1fa`
(path operations/hash),
`ev_7d9ca21ac644ff7e6a28b2668ed6924f8ed4f8265b253d0b4ff66d714e080915`
(Globals source/getter/fields/conversion), and
`ev_d62b7de6adee929d08e685c9403eadf6bd05d25283d486db5a6d361758d04f02`
(field binding and import bytes). All completed with exit 0, no output
truncation, and unchanged artifact SHA. Rizin warned that broad string-search
regions exceeded its limit; no claim here depends on that omitted search.
The original verifier matched all 1,754 selected decoded instruction byte
sequences back to the owned PE. The exported ledger retained no active target.
The final string-callee record,
`ev_a0f46ed1bb9742d2b7e47aaf46cd8e6c0e78c4b0af5e2d8f57ec7806d824e906`,
has the same completion guarantees and another 330 verified instruction byte
sequences. Its separate receipt identifies the two CRT imports. A final
`support-readback.json` rechecked all 127 retained supporting artifacts against
the three helper receipts.

The complete core remains in the earlier
`frontend-localization-runtime/native/final.bundle.json`, record
`ev_f3364344183c0a8c2f6f2ee3ea4889e9cd13e13e46fead263f88b488a94f3a12`.
Raw data receipts cover nine selected SIP members. The identity probe used the
previously frozen independent 27-key oracle, not pending product code, and
matched all 27 exact IDs plus 27 ASCII case variants. These checks do not
validate runtime callback behavior.

Sources read: the owner's proprietary Win64 executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and owned `Data/Game/Calligraphy.sip`, SHA-256
`1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c`,
both beneath the private `pins/alienware-candidate/` tree; the project's
original private PE/cursor helpers and frozen oracle; prior project findings;
and the installed REA skill/selected Rizin output. No outside implementation,
website, ClientConfig, account data or network service was read. This document
is the only repository change; all implementation-oriented probes are original.
