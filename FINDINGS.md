# Retail client and implementation findings

Task `a3d58c579542`, first inspection 2026-11-15 UTC. Claims below distinguish
file observations from execution. No retail client or server login has been run.

## Target identity and private preservation

**OBSERVED:** The Alienware's existing `alienware-bazzite` SSH alias reaches the
owner-supplied machine. Read-only inventory located the installed client at
`/var/home/blizz/Games/banshee-library/games/marvel-heroes`. SSH used
`BatchMode=yes`, `StrictHostKeyChecking=yes`, `ConnectTimeout=8`. No Wine prefix,
character, account database, login/trust prompt or source file was modified.

**VERIFIED (static metadata):** The copied Win64 `MarvelHeroesOmega.exe` has
embedded file and product version **1.52.0.1700**. It is 48,544,080 bytes; SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` matches the
source-machine hash. REA identified PE32+/x86-64 and 79 resources; Evidence ID
`ev_9465227bdb062c7dd72398dc85a49335d4cad24896188dd25bf86b908c67b0a7`.

The version was decoded from REA's hashed RT_VERSION resource: file offset
46,771,896, length 668, SHA-256
`cdc4051e94dfde76c8d72de184d5628f424e1097bfa854810878e7ce0b61f1e1`.
The aligned `VS_FIXEDFILEINFO` starts at 46,771,936. This identifies embedded
metadata; it does not prove an unmodified retail installation or runtime behavior.

Private working copy (partial while transfer runs):
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/`.
The name remains candidate because copy completion and all-file verification
are separate from executable version confirmation.

**OBSERVED:** The source manifest selects **16,489 regular distribution files,
21,193,180,456 bytes**, including 15,250 UPKs and 311 texture caches. The inventory
script selects binaries, packages, localization, movies, base/default configs and
shader data. It excludes user data, download directories, patch backups and
unclassified files; no symlinks are followed. Forty-nine files inside the two
inspected subtrees are excluded. Top-level `userdata` and `Council of Ancients`
were not copied. This is a selected client distribution snapshot, not a backup
of the owner's entire installation or settings.

## Available SIP packages are not interchangeable

| Source | File | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| Alienware installation | `Data/Game/Calligraphy.sip` | 47,001,699 | `1661a4ca786c10e46aa0f9b3e15ebfc18ca8ff02ace8da9cf38de117bcacd30c` |
| Devbox server data | `Data/Game/Calligraphy.sip` | 47,001,699 | `992a502e3eb977730ed159b44a66fd09622f5afc7e59dcb191439c6686995147` |
| Both locations | `Data/Game/mu_cdata.sip` | 27,831,934 | `694f8af3f1950bac8b154e4e1ac354516f70bd3d895d6e9472eabd2df635d425` |

**OBSERVED:** Calligraphy file sizes match but hashes differ. The cause has not
yet been established. Do not treat the server copy as the pristine retail source.
The server packages were separately copied and hash-verified under
`pins/server-data-unverified-build/`; that pin's build identity remains unknown.
The devbox Steam libraries checked did not contain a Marvel client installation.

## Asset work

**VERIFIED:** The original SIP/LZ4 implementation decoded all **102,716 members**
of the two server-data packages (174,757,753 bytes) identically to an independent
cached MIT `lz4_flex` decoder. Five focused malformed-input/limit tests passed.
See `docs/SIP_FORMAT.md` for scope, limits and exact package identities.

**VERIFIED:** The original Calligraphy v11 metadata implementation parsed all
**9,602 directory, blueprint, curve and asset-type files** in the server-data
Calligraphy package without failures. Eight focused parser tests passed. The
actual type-directory signature is `TDR`, correcting a reference-doc discrepancy.
Prototype bodies, inheritance and gameplay semantics are not yet implemented.
See `docs/CALLIGRAPHY_FORMAT.md`.

**OBSERVED:** Five copied UPK samples match the source manifest. Their summary
headers declare UE3 version 868/licensee 3, engine 10897/cooker 136 and compression
flag 2 (LZO). Frontend package object decoding is being implemented; no frontend
has been rendered. See `docs/UPK_FORMAT.md` when that implementation lands.

**VERIFIED:** Preset foundation defaults to Forever, accepts Original and exposes
zero upgrades because none is finished. This is not evidence of graphics parity.

## Retained private evidence

All paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/`:

- `alienware-distribution-inventory.json`: source paths, sizes, SHA-256 for selected
  distribution files and explicit exclusions. Produced by ordinary Python/SSH.
- `server-data-pin.json`: separate source/copy hash verification.
- `retail-pe-resources.bundle.json`: complete exported REA resource Evidence.
- `retail-version.json`: bounded version-resource decode linked to that Evidence.
- `sip-real-smoke-20261115.txt`, `sip-unit-tests-20261115.txt`.
- `calligraphy-metadata-smoke.txt` and private metadata samples/harnesses.
- `options-smoke-20261115.txt`, `options-reference-files-20261115.json`.

REA MCP initially was not exposed, then connected. Its Ghidra candidate reports
an unavailable JDK in that process. A separate CLI attempt uses the already
installed JDK25 and Ghidra without changing shared configuration. The outcome of
deep analysis is recorded separately; portable PE Evidence already succeeded.

## Remaining milestones

1. Asset loader: in progress; full private copy, UPK objects/resources and
   Calligraphy prototype decoding remain.
2. Retail frontend renderer: not implemented or compared.
3. Login and character select: not implemented or tested against MHServerEmu.
4. World, movement and camera: not implemented.
5. Combat, powers, UI and full parity: not implemented.

No renderer, network or gameplay parity is claimed from the asset tests.

## First integrated build

**VERIFIED:** Cargo debug and optimized release builds compile offline without
external Rust crates on rustc 1.98.1. The integrated CLI reads both pinned archive
variants. `calligraphy verify` on the **Alienware** Calligraphy package also
passes all 9,602 supported metadata files and explicitly skips 85,186 prototype
bodies. This check does not cover the differing prototype payload.

**OBSERVED:** The SIP variant comparison found byte-identical metadata and seven
changed compressed bytes, all in one Carnival mission prototype; every other
stored member matches. The cause and effect remain under investigation; no file
was restored or changed. Detailed comparison evidence remains private.

**VERIFIED:** The integrated original UPK reader decodes the name/import/export
tables of all five pinned samples. Thirteen focused tests passed. All 322 raw
LZO blocks (38,845,307 decoded bytes) match independent installed liblzo2 output.
The frontend map has 37 exports; the frontend asset package has 606. Object
properties and asset payloads are not decoded in this increment.

**OBSERVED:** The Alienware's separately inspected game-package backup
`Data/_mhclientpatch-backup/Calligraphy.sip.20261009T031312Z.orig` hashes exactly
like the server copy (`992a502e...`). No restore was performed. This supports a
backup/current-install relationship, not an untouched-retail provenance claim.
See `docs/SIP_VARIANTS.md` and `evidence/alienware-calligraphy-backup-hash.json`.

**OBSERVED (native REA limitation):** With the correct existing JDK25 and
`GHIDRA_INSTALL_DIR`, REA recognized Ghidra 12.1.4 and its x86-64 decompiler, but
its Java probe timed out **during ownership preparation**, returning
`provider_unavailable/runtime_unavailable`. The message suggests JAVA_HOME, but
the diagnostics already name the intended JDK. Deep analysis did not start;
no functions or pseudocode were recovered. Full returned diagnostics are in
`evidence/retail-win64-inspect-jdk25.json`. No setup or shared config edits were
made. Portable REA resource inspection and exported Evidence remain valid.

**VERIFIED (first private release):** `0.1.0-dev.1` passed nine initial CLI
startup/argument/SIP/Calligraphy checks and ten final UPK/status/version checks.
Exact commands, exit codes and outputs are retained in
`evidence/build-0.1.0-dev.1-smoke.json` and
`evidence/build-0.1.0-dev.1-upk-smoke.json`. Eight initial copied files totaling
132,306,782 bytes match their remote hashes; this subset receipt does not claim
completion of the 16,489-file distribution transfer. JDK25 itself reports
25.0.5 in a direct shell probe; REA's separate ownership-preparation timeout
remains the native-analysis limitation.

## Increment 0.1.0-dev.2 — prototype bodies and UPK properties

**VERIFIED:** Schema-checked prototype decoding passes all 85,186 bodies in each
pinned Calligraphy variant. A new integrated lazy catalogue checks IDs/member
paths and supports lookups by ID, exact name and directory index. The integrated
release catalogue verified all 85,186 Alienware bodies, with 709,824 nested nodes,
731,294 groups and 1,947,742 values. Maximum embedded depth is 18. Nine focused
prototype parser tests passed. Parent-value merging and runtime property/class
binding remain unimplemented.

**OBSERVED:** The Carnival variant differs in exactly three decoded asset fields
(DesignState, DesignStateXboxOne, DesignStatePS4). Their literal asset labels are
`NotInGame` in server/backup data and `Live` in the Alienware installation. This
is a static data difference; no running-client or server effect was tested.

**VERIFIED:** The original PackageArchive reader provides checked logical/export
reads across LZO chunks. The new asset property parser reads 725 supported export
streams containing 6,442 properties across the five pinned samples; 38 exports
with unestablished preambles are explicitly skipped. Nineteen focused tests pass.
Seven extracted exports/movie arrays match the independent private probe exactly.
See `docs/UPK_OBJECTS.md` and private `evidence/upk-validation/object-checks.json`.

**OBSERVED:** Frontend data includes 582 inline Texture2D exports and 20 GFX movie
arrays, including the actual login dialog. Independent texture/GFX parsing and
renderer work continue. No reconstructed login screen is yet rendered or playable.

**OBSERVED (REA):** Static PE import inspection examined 40 DLL descriptors;
complete Evidence `ev_7136989a0b4ae1b35612e44bddcbe16bc3fbf8ad0f1245aaed529cbdb780c919`
is exported privately to `evidence/retail-pe-imports.bundle.json`. See
`docs/REA_NATIVE.md` for dependency observations and the unresolved native Java
probe deadline. A normal-priority probe reproduced the failure, without config
changes. Static import tables do not establish which APIs run.

**VERIFIED (integrated dev.2 checks):** ID/member lookups select the same Carnival
prototype; a missing ID returns an error. Four catalogue checks include the full
85,186-file structural traversal. Seven final release checks cover named/numeric
UPK export selection, malformed/missing selection, all 606 frontend asset streams,
version and Original preset. Exact outputs and binary identities are private in
`build-0.1.0-dev.2-prototype-smoke.json` and
`build-0.1.0-dev.2-object-smoke.json` under `evidence/`.

## Increment 0.1.0-dev.3 — saved frontend assets and GPU image path

**VERIFIED:** Four independently copied startup packages (46,069,713 bytes)
match the source inventory SHA-256 values. The private subset receipt is
`evidence/frontend-startup-pin.json`; the full distribution transfer is separate.
`MarvelGame.upk` contains `MarvelFrontEnd.frontend_manager` at zero-based export
17004, matching both the saved frontend map and class-default movie reference.
The exact extracted movie, offsets and hashes are private in
`evidence/frontend-startup/manifest.json`. Static configuration does not prove
which screen a running client presents.

**VERIFIED:** The original inline texture reader decodes all 582 frontend
Texture2D exports. The original movie reader parses all 20 frontend movies, and
the integrated bridge matches all 716 external image records to the independent
reference-constrained inventory. All 39 imports resolve by full object paths to
11 textures in MarvelGame. Missing and duplicated dependency inputs preserve
39 explicit unresolved/ambiguous results; outer-chain cycles are rejected.
The manager's 11 images resolve separately. See `docs/FRONTEND_READER.md` and
`evidence/frontend-binding-validation.json`.

**VERIFIED:** A private optional Vulkan renderer drew the decoded native-size
2560×1440 background. Every one of 14,745,600 GPU-readback bytes equals its
supplied RGBA input on llvmpipe. The integrated CLI repeated this check and saved
a private lossless PAM; its PNG wrapper was visually inspected. This is the
SHIELD-pattern background alone, not the composed login/frontend. No original
client display, physical GPU, runtime UI or visual parity comparison was run.
Settings, hashes and receipts are in `docs/RENDERER.md` and private
`evidence/dev3-integration-smoke.json`.

**VERIFIED:** Geometry parsing consumes all 676 actual shapes in the 20 movies
plus manager. The bounded AVM1 decoder parses 779,489 instructions in 2,863 action
roots across those movies, with no unknown opcode or push-value encoding.
These are structural results; tessellation, scripting execution and native host
semantics remain further work. See `docs/GFX_SHAPES.md`, `docs/AVM1_FORMAT.md` and
their private receipts. No runtime parity is implied by decoding instructions.

**VERIFIED:** The manifest reader preserves 47,769 texture records and 143,422
mips, including ambiguous GUIDs. Three independently pinned cache ranges contain
seven LZO blocks; all 731,056 decoded bytes match an independent liblzo2 oracle.
Existing pixel conversion handles the selected BC3/BGRA samples. Range identities
are verified separately; no whole-cache hash or native cache rendering claim is
made. See `docs/TEXTURE_CACHE.md` and `evidence/texture-cache-ranges/`.

**VERIFIED:** The original font/text reader structurally consumes 12 Font3
definitions with 3,494 glyphs/107,711 outline edges, 13 static text definitions
and 479 editable text fields across 22 private movies. Ten focused checks passed.
Font aliasing, typography/layout, HTML and rendering remain unimplemented; no
visual equivalence is claimed. See `docs/GFX_TEXT.md` and `evidence/gfx-fonts/`.

**VERIFIED (private dev.3 staging):** The optional-renderer development-profile
binary, with debug information disabled, passed the five final focused checks
in `evidence/build-0.1.0-dev.3-smoke.json`. All 154 packaged file hashes and the
packaged version/Original startup checks pass. Dependency texts cover 61 selected
Cargo crates and retain upstream notice gaps/provenance explicitly. The staged
binary SHA-256 is `c23bad1abdd002bafaa825cc0b5dc767a4c6e9003af006a2cadc0084df3ea333`;
source commit `1cf6a2e` was clean. The package contains no game assets/captures.
This is private staging, not confirmation of launcher publication. Receipt:
`evidence/build-0.1.0-dev.3-package-validation.json`.

**OBSERVED (dev.4 frontend foundations):** Saved References/ExportAssets resolve
69/71 symbols across 26 movies with an explicit English font-library route.
The two Strat2Regular exports remain absent. Login and character-select snapshots
contain 147/264 nodes with no unresolved definitions. Original triangle rendering
produces login artwork; saved character/manager composites remain black/white.
The private `evidence/frontend-preview/` records exact sampling policy, all omitted
effects/text/script work and source placement/color observations. These snapshots
are not a completed frontend or native visual comparison.

**OBSERVED:** The root persistent initializer CLI preserves 452 manager
DoInitAction records separately from its ordinary frame action. It executes 218
instructions before the unbound ASSetPropFlags call at12374; zero initializers
complete. Two focused scheduler tests pass, including refusal to retry partial
state. See `docs/FRONTEND_BOOTSTRAP.md` and its private receipts. Subsequent REA
property-flag work must preserve the unresolved target-derived default mask.

**OBSERVED:** Independent increments decode original English locale/UI keys,
19 Vista StaticMeshes (94,385 vertices,73,737 triangles), and 51 supported plain
EditText definitions (553 glyph placements). These counts are asset/source
coverage; dynamic visibility, materials, gameplay and typography parity remain
unverified. Evidence and outside-source licenses are linked from
`docs/PROVENANCE.md` and the respective format documents.

**VERIFIED (private dev.4 staging):** All nine focused CLI/startup/composition
checks pass, including exact login RGBA identity with the independent adapter
harness and two native preview frames on fresh Xvfb/llvmpipe. A case-sensitive
preset assertion was corrected and that single check rerun; implementation was
unchanged. The receipt retains this correction. All 262 packaged file hashes and
the packaged version/Original checks pass; dependency notices cover114 crates
with217 texts. Source `83e77def51cc1a7567a40b97969a648f8108fa19` was clean.
The binary SHA-256 is
`49cfb8b659c8e11239add5be3080825c609dc4d0c25ff13009cde1a8bd72c785`.
Receipts: `evidence/build-0.1.0-dev.4-smoke.json` and
`evidence/build-0.1.0-dev.4-package-validation.json`. This is private staging,
not confirmed launcher publication, owner-device testing or retail parity.

**VERIFIED (private dev.5 staging):** All nine focused final checks pass. Supported
original text yields 232 glyph fills in the saved login snapshot; the integrated
image matches the independent adapter bytes. The original manager MouseMove
handler executes 14 instructions/six host calls and moves its actual cursor to
the supplied (100,200) pixel position, with nonwhite pixels bounded by
(100,200)–(142,238). Reusable GPU resources preserve image bytes, and the final
native preview presents two frames. These are diagnostic checks, not native
retail visual or input comparisons.

**OBSERVED:** The final initializer CLI completes 52 original manager initializers
and retains exactly one Selection listener. It stops at source323687 with
unsupported super/parent preload after 8,669 instructions; it does not execute
the remaining initializers, ordinary frame actions or full startup.

All 271 packaged files verify; notices cover 114 crates and 217 texts. Clean
source commit: `477f2bb29cae43fc5e22cf69e1a11317e20736ac`. Binary SHA-256:
`8715ccb6c7c43fd538d87b583193116a65e5085e436d4eeb72e5b631592f0efc`.
Receipts: `evidence/build-0.1.0-dev.5-smoke.json` and
`evidence/build-0.1.0-dev.5-package-validation.json`. No original game data is
packaged. Private staging remains distinct from publisher-channel delivery.

**VERIFIED (private dev.6 staging, 2026-11-15 08:58 UTC):** Seven final focused
CLI checks pass. The persistent controller reproduces the prior original cursor
RGBA bytes, while the native `window-pointer --seconds 10` check passes eleven
assertions on a fresh isolated Xvfb/llvmpipe session. Four original handler
invocations execute 56 instructions/24 host calls; six frames present with one
GPU generation and one retained 16,384-byte texture. Resizing to 1000x1000 maps
surface point (250,500) to cursor (9600,12000) twips within f64 tolerance; moving
into the margin preserves it. The default standard-library build also checks.

An initial synthetic Escape injection did not close bare-Xvfb's window; its
timeout remains recorded. The final check uses the explicit native QA deadline.
Neither synthetic-key failure nor timed success establishes physical-keyboard,
owner-device or native-retail parity. A separate original-handler failure check
confirms that a partly executed action is halted without publishing its partial
snapshot or permitting retry. Fractional assertion corrections are retained.

**OBSERVED:** Constructor-super support advances the original initializer to its
actual native MovieClip call at source322430. The final CLI completes 52
initializers and 8,671 instructions, retains one Selection listener, then rejects
the unbound native callback. It does not run remaining initializers, full startup,
login or gameplay. Native allocation/category/Super proof is documented, while
additional prototype-assignment side effects remain under study before enabling
the typed receiver path.

All 278 packaged files and packaged version/Original checks pass; notices cover
114 crates/217 texts. Packaged clean source:
`db63270a727ab505237dd043692c07d3482f0239`. Binary SHA-256:
`2352a87cd90a6b11be8e009e63cbea63e2140585ba597d37c3aae674093006be`.
The tested source was `69e6327`; the only subsequent packaged change was native
validation prose, independently checked by the package receipt.
Records: `evidence/build-0.1.0-dev.6-smoke.json`,
`evidence/dev6-native-pointer-attempt2/receipt.json`,
`evidence/persistent-pointer/verification.json`, and
`evidence/build-0.1.0-dev.6-package-validation.json`. No game assets are bundled.
Private staging remains distinct from publisher-channel delivery.
