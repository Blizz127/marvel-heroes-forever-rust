# Complete selected-distribution snapshot plan

Task `a3d58c579542`, 2026-11-15 UTC.

This records the design and harmless copy preflight, followed by the resolved
source-verification dependency. The full rsync exited zero. Root's separate
terminal report observed at 10:48:11 UTC verifies all 16,489 files and
21,193,180,456 bytes with `complete=true` and zero problems. The timestamp in
its filename is only an identifier. Source verification receipt SHA-256:
`a8994255f33d3bb5dc5da510d755d076b80028fc28d04c0363fffe0d116862c8`.

Root then authorized independent snapshot assembly and one bounded offline run.
The planning preflights used no pin bytes and launched no game. Actual assembly
and its run are separate artifacts. The preceding 453- and 454-file observations
remain intact.

The intended source is the exact 16,489-entry selected-distribution inventory,
21,193,180,456 bytes, plus the separately inspected 256-byte original
`Data/Configs/ClientConfig.xml`. The new total is **16,490 files and
21,193,180,712 bytes**. This is the complete selected distribution, not a claim
that every file in the owner install is included: the original inventory excluded
49 paths, and only the explicitly authorized configuration is added from them.

**VERIFIED, metadata:** The original inventory has 56 directories excluding its
root; adding only the original configuration produces 57 directories excluding
the root. No traversal, duplicate paths, case-folding collisions or file/directory
conflicts were found. The largest selected input is `CharTextures.tfc`,
1,373,627,963 bytes. Every prior 454-file snapshot path, size and digest is
preserved; the full plan adds 16,036 selected entries totaling 16,078,135,783 bytes.
The original source-content verification remains the separate root receipt above.

## Disk and copy feasibility

**OBSERVED:** The candidate pin and private evidence reside on the same ext4
filesystem. About 19.09 TB was free at the capability test. GNU coreutils `cp`
9.7 with `--reflink=always` failed on a generated 32-KiB file with
`Operation not supported`. No pin, game or configuration contents were used in
that test. Reflink support is therefore unavailable for this plan; no filesystem
or package change is proposed.

**VERIFIED, synthetic:** A separate original streamed-copy fixture produced an
independent inode with the expected digest. Mutating its destination preserved
the generated source; mutating that source preserved the copied destination.
The implementation used a 1-MiB maximum buffer. This proves the harmless fixture,
not integrity of any future game snapshot.

Use one serial `nice -n 19` copy/hash process, with a 1-MiB buffer and a planning
reserve of 32 GiB free space for the selected assets, clean prefix and receipts.
No hard links, recursive unfiltered directory copy or source modification is
needed. Current space is ample; source identity and hash checks remain required.

## Assembly contract

Before assembly, bind the completed root verification receipt at
`evidence/alienware-full-pin-verification-20261115T1050Z.json` and its digest to
the exact inventory digest. The report's actual schema and terminal success
must be inspected; a file merely existing or an rsync exit alone is insufficient.
All 16,489 selected file sizes and hashes must match, without missing or mismatched
inputs. No status-field names or successful result are presumed in this plan.

Copy only validated exact relative inventory paths and the one explicit original
configuration into a **new** private staging directory. Reject traversal,
non-regular files, symlinks, duplicate paths or an occupied destination. Record
case-folding conflicts if present rather than silently renaming or dropping a
source entry. During copying, verify the streamed source hash against the bound
inventory, then independently hash the destination and check size and inode
independence. Recheck source identity for a concurrent-change failure.

Only after all 16,490 entries match may the new snapshot become final, with files
`0400` and directories `0500`. Preserve the older snapshots, completed pin and
owner install. A failure leaves an explicit partial receipt and prevents launch;
it must not silently skip, repair, substitute or claim a complete snapshot.
The original configuration must retain SHA-256
`a319ef50b21d3d7e84b5b3c0894b41805c809e7d0eae728b7b4be7f1d3717c5b`.
Its values remain private.

## Runtime preparation

Retain the existing normal argument vector, a new verified clean-prefix clone,
new authenticated Xvfb, a single read-only `/game` mount, child-only offline
network/PID namespaces and the existing private loopback hostname mapping.
No UI input, new flags, endpoint edits, accounts or host service/device access
are part of the plan. Retain prompt/error stops, owned cleanup and the existing
45-second inner / 55-second outer bounds and trace caps if a run is subsequently
sequenced by root. This design does not launch the client.

Keep large preparation and verification manifests outside the guest's runtime
write mounts. Bind a separate small runtime-receipt directory at
`/capture/receipts` and scope REA's filesystem observation to that directory.
Full input verification remains an independent bounded host-side hash pass,
with summary digests linking the full manifests. This addresses the concrete
prefix-receipt digest omission in the previous capture without pretending its
missing before-hash was recovered. Record the exact observation scope and all
future truncation limits.

The full selected asset set should permit a useful comparison with the partial
runs, but it does not establish successful startup in advance. Missing unselected
inputs, runtime failures, XML consumption, rendering and behavior parity remain
questions for actual bounded evidence.

## Evidence and sources

Private planning directory: `evidence/native-full-snapshot-plan/`.
Inventory SHA-256:
`a03c7af289214495d2871956acaf4eec378a9a3868d4f06ef72f4c19d98116e0`.
Reflink feasibility receipt SHA-256:
`a953cda267859cce3ef2fb336f5a5e529c96aace321bbdd555884551a4747ccd`.
Streamed-copy fixture receipt SHA-256:
`1ae9751212fe4dbc1b632913921eae1a719fbea721491cb7aff8b6f4edbfdb23`.
Metadata-manifest plan SHA-256:
`164f5cfc72b0b5008f13693cc17eba7dfe180a6d5483676e2b32c2d7a770619b`.
Terminal verification binding SHA-256:
`3d788358cdd4100968aeee21676a8404433d5db21dedb8383ab39e134dbe2832`.
Assembly/runtime design SHA-256:
`46e502d8c6abcdb4fba4e4fa08ba8e363f0705e17b0168d2fe644a73606289fb`.

Outside sources used: owner-private distribution inventory and prior snapshot
receipts, proprietary/private compatibility evidence; installed `/usr/bin/cp`
version/help, GNU coreutils 9.7, GPLv3-or-later according to its version notice;
read-only Linux mount/resource metadata for capacity and filesystem identity.
Exact tool output stays private. Existing runtime/REA and configuration-source
provenance remains in `NATIVE_REFERENCE_CAPTURE.md` and `NATIVE_CLIENT_CONFIG.md`.
All planning and copy code is original; no external implementation was copied.
