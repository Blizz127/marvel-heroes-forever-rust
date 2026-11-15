# Exact owner-install ClientConfig inspection

Task `a3d58c579542`, 2026-11-15 UTC.

The [targeted runtime trace](NATIVE_TARGETED_METADATA_TRACE.md) observed the
exact `ClientConfig.xml` path and its parent returning `ENOENT`. After owned
cleanup and verification of all 453 frozen inputs, root confirmed that this
met the standing condition for read-only retrieval of the owner's exact file.
No earlier frozen snapshot or source install was changed.

**OBSERVED:** Strict host-key checking and batch-mode SSH retrieved only
`/var/home/blizz/Games/banshee-library/games/marvel-heroes/Data/Configs/ClientConfig.xml`
from the existing `alienware-bazzite` owner install. The read-only source was a
regular nonsymlink file; two bounded reads matched and its inode, size and
modification/change metadata stayed unchanged. SSH exited zero without an
interactive prompt. The independent private copy is 256 bytes, SHA-256:
`a319ef50b21d3d7e84b5b3c0894b41805c809e7d0eae728b7b4be7f1d3717c5b`.

The XML has a `config` root and these three declared properties. Property
identifiers are held in `name` attributes; settings values are in separate
`value` attributes and are not reproduced here.

| Declared type | Field name |
| --- | --- |
| `str` | `SiteConfigLocation` |
| `str` | `SiteConfigLocationBackup` |
| `bool` | `KioskMode` |

No account, credential or user-identity property was found. The two location
fields contain no URI user information, query parameters, fragments or line
breaks. No DTD or entity declaration was present. These concrete checks are not
a universal secret-detection guarantee. All settings values, raw XML and the
retrieval response remain private. No native execution followed retrieval or
inspection before this report to root.

This is the exact **owner-install** configuration, not proof of historical
Gazillion default settings or an unmodified Steam distribution configuration.
Its historical origin is unverified. It was absent from the earlier selected
working snapshot and excluded from the original distribution inventory; this
observation does not alter those earlier receipts.

Root subsequently authorized a new immutable snapshot that preserves all 453
previous input hashes and adds only this exact original file as input 454.
That new snapshot and its separately bounded offline observation must retain
the normal arguments, clean prefix, no-egress namespace, prompt stop and
owned-process cleanup. Prior snapshots remain unchanged. No generated
replacement settings, endpoint edits, flags or UI input are part of this change.

Private evidence: `evidence/clientconfig-inspection/`.
Schema receipt SHA-256:
`c1383fd5f5e9884b071967d49ae604863445917d2d1e2e2b5379ecb19a7bc2ed`.
Property-name receipt SHA-256:
`fb0e57d4508f5bf2514acde978116269b9b45e2022b761b1a5dd35a074d70414`.
Location-field indicator receipt SHA-256:
`333ff31e4ffef6d573d317489cc71db662e23d64c43c7c1b0fd71141b3356f34`.
The read-only retrieval receipt binds the source path, repeated hashes,
strict-SSH options and prior post-run integrity receipt without exposing values.

Source/provenance: the exact owner-install path above, copied privately for
compatibility study. No open-source licence is asserted for this private
configuration, and it is not redistributed. Original Python standard-library
inspection code records names/types and credential indicators without copying
external implementation. Existing SSH/runtime provenance and REA study receipts
remain those in `NATIVE_REFERENCE_CAPTURE.md` and `NATIVE_STARTUP_SETTINGS.md`.
