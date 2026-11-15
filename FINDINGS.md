# Retail client and implementation findings

Task `a3d58c579542`, first inspection 2026-11-15 UTC. Claims below distinguish
file observations from execution. Earlier entries retain the state when recorded;
later dated build/probe entries supersede them. Isolated offline retail probes
have now run; no server login or playable Rust frontend has been demonstrated.

## 2026-11-15 dev13 diagnostic increment

**VERIFIED Rust build/startup:** source
`caa22506990eaffea350d70bfe6206ee08e1b19c` builds with the window feature.
Binary SHA-256 `d963ba5138f2c7310ff4761a20fa8553acdcedbc55b4238d6520d38725f704cf`
passes nine focused CLI/image checks and an isolated two-frame native-window
smoke with empty stderr. Saved login/cursor pixels remain identical to the
retained diagnostic outputs. Receipts: `evidence/build-0.1.0-dev.13-smoke.json`
and `evidence/dev13-native-html/receipt.json`. These checks do not execute full
native startup or establish playable login.

**VERIFIED Rust checks:** typed attached `Function.apply` and bounded ordinary
`super` calls are integrated. The latter retains receiver identity separately
from its prototype cursor and from construction context. Eight focused checks
include existing construction regressions; the integrated window-feature check
passes at `598b2e4`. [Applied-super evidence](docs/REA_APPLIED_SUPER.md) records
the native contract and admitted runtime profile.

[Persistent saved-action state](docs/SAVED_ACTION_STATE.md) adds explicit
per-installation constant pools and per-owner operand/register storage. Three
focused standalone state tests pass. Central saved-mode frame, activation and
dispatch integration remains outstanding; generic execution is still diagnostic.
[Saved handler bindings](docs/SAVED_HANDLER_TABLE.md) retain per-key insertion
order and shared installation identity across native Construct/Load keys. Both
focused table tests pass; table presence does not imply successful delivery.
[Native text-format mapping](docs/TEXTFIELD_FORMAT_MAPPING.md) closes the scalar
style derivation for the qualified fresh HTML path while retaining explicit
font-selection and effective runtime-state requirements.

**VERIFIED prepared geometry/alpha; INFERRED explanation of retained pixels:**
the saved login's later shape instances 106, 112 and 116 each cover the complete
node-89 field with opaque bitmap samples. Their source triangles, saved/runtime
draw order and 6,760 required alpha bytes per sampled support explain why the
updated glyph geometry produced unchanged final pixels in the retained probe.
[Visibility evidence](docs/RUNTIME_TEXT_VISIBILITY.md) also locates the original
constructor writes that hide those panels; those writes have not yet executed
through the full startup path. No visibility workaround or new GPU pass is
claimed.

**OBSERVED retail reference:** the full same-snapshot continuation remained on
the original loading splash at its 45-second deadline. Both visible windows
reported the isolated owned retail PID. All 16,490 input hashes passed the
completed post-run check, no UI input was sent and owned cleanup finished.
[Continuation evidence](docs/NATIVE_SPLASH_CONTINUATION.md) preserves the exact
scope; loading artwork does not establish a rendered login frontend.

Linux notices cover 121 dependencies and 231 notice files, manifest SHA-256
`c9525bcb587bf3326061e74d7c59bdac0f56a5e79c361a35acf61b42888ac0bb`.
This diagnostic contains no game assets and exposes zero finished upgrades.
No Rust launcher publication route has been confirmed. Gate 1 retail
behavior/visual parity is not yet verified; Gate 2 exposes no finished upgrades
yet. Private staging/readback is recorded separately below when complete.

**VERIFIED private staging:** `builds/0.1.0-dev.13` contains 354 selected
original/dependency-notice files from clean packaged commit
`d0093e5` (tested code `caa2250`). Complete file-set, size/hash and executable
version checks pass. Receipt: `evidence/build-0.1.0-dev.13-package-validation.json`.
This supersedes dev12's Rust diagnostic intake only; no retail launcher tile is
changed and no publication is claimed.

## 2026-11-15 dev12 diagnostic increment

**VERIFIED Rust execution:** tested source
`7041d197cc63a002075aa239c04985954b73158f` builds with the window feature.
Binary SHA256 `e2da8fda61e1f1e0e1e87ef2f6e4321fa624c2f7dd050877b0f1a778ad8db846`
passes nine CLI/image checks and the isolated two-frame window smoke with no
stderr. Saved scene/cursor pixels remain unchanged. Receipts:
`evidence/build-0.1.0-dev.12-smoke.json` and `dev12-native-html/receipt.json`.

Native String boxing/ASCII uppercase, exact-false `_focusrect`, checked movie
bindings and document-to-render preparation are integrated. Original localization
now obtains `Log In` from the actual catalog, executes uppercase at 94251 and
writes `LOG IN` at 94252. Six glyph placements follow from that live document;
all 32 other text instances and host/source state stay intact. Attachment,
absent services and effective format/font bindings remain controlled inputs.
The bridge also passed twenty typed rejection cases and its replacement,
cache/default and empty-document controls.

All 452 selected manager initializers complete on the controlled attached root
in 49,244 instructions, with one Selection listener and five class registrations.
The original containing-frame action then stops after 33 instructions at typed
receiver `Function.apply`. Minimal globals, child snapshot and raw native state
remain supplied. [AVM1 execution](docs/AVM1_VM.md) records the exact receipts;
this does not establish a complete native startup sequence.

**OBSERVED limit:** the first GPU composition submits updated label geometry but
changes no final pixels in the overlapping saved scene. Its visibility assertion
failed; exact occlusion and startup visibility actions are being investigated.
[Runtime text preparation](docs/RUNTIME_TEXT_PREPARATION.md) preserves that result,
the private fixture's dependency difference and its conditional style scope.

The complete 16,490-file isolated retail snapshot reached original loading
artwork and stopped on that new splash without input. All input hashes remain
unchanged. A same-snapshot/fresh-prefix comparison permits the now-known splash.
No login or server connection has been demonstrated.

Linux notices cover 121 dependencies/231 files, manifest SHA256
`79f218ee46f34e82207920473a4ebc5260b984b83a03ed71431fe44902402764`.
This original-code diagnostic bundles no game assets and exposes zero finished
upgrades. No Rust launcher publication route has been confirmed.

**VERIFIED private staging:** `builds/0.1.0-dev.12` contains 346 selected
original/dependency-notice files from clean packaged commit
`6bf3a89773b653d1bad095138ad6a36daaa2eb0a`. All file sizes/hashes, the complete
file set and executable version passed readback checks. Receipt:
`evidence/build-0.1.0-dev.12-package-validation.json`. This supersedes dev11's
Rust diagnostic intake only; the retail launcher tile is unaffected. No
publication or playable-client claim is made.

## 2026-11-15 dev11 diagnostic increment

**VERIFIED Rust execution:** tested source
`35ba47d433579e89716a4c3fee69995f5454c329` builds with the window feature.
Binary SHA256 `b3e205ae2fd77ecfc8a914cdaa8dbcd32a87107acd70e515cdd282e0e1b389bb`
passes nine focused CLI/image checks and a fresh isolated two-frame native-window
smoke with no stderr. The original saved login remains 33/33 text fields and
303 glyph fills; its pixels and the original cursor output remain identical to
the retained diagnostic outputs. Receipts are private
`evidence/build-0.1.0-dev.11-smoke.json` and `dev11-native-html/receipt.json`.

This increment adds typed attached TextFields and checked per-instance host
state, explicit imported source/Environment identities, verified private native
name inventories, lazy builtin values, cached localization preparation,
conditional Stage math, instance text preparation, SHA-256 source verification,
and the offline `frontend startup-map` command. Each component retains its
documented supported profile. The TextField host passed two synthetic cross-layer
tests; startup-map preparation passed four focused tests and the original-map
CLI smoke. Original wrapper/backend localization passed its separate frozen
integration check. These do not constitute an initialized retail frontend.

Native String uppercase and further attached initialization are subsequent
increments. The original saved login-label sequence has not yet passed through
the integrated typed host at this build checkpoint. Rendering runtime text and
native field lifecycle/format adoption remain separate gaps. Stage state is not
yet a complete VM Stage factory or listener dispatch.

**VERIFIED preservation:** the full selected client distribution now has all
16,489 files hash-checked, as recorded below. The latest completed retail probe
used the smaller 454-file snapshot and successfully checked/opened the exact
original ClientConfig, but still showed a black primary window at its 45-second
deadline. It performed no input and left no owned processes; all 454 input
hashes remained unchanged. A complete-distribution comparison is being prepared.

The Linux notice collection covers 121 dependencies/231 notice files, manifest
SHA256 `56b2785d28dcea38156ad5e76775952d7c70810e8474a17d4c7c2e9343f41be3`.
This original-code diagnostic contains no game assets, makes no server
connection, and exposes zero finished upgrades. Private staging and launcher
publication remain distinct; no Rust publication route has been confirmed.

**VERIFIED private staging:** `builds/0.1.0-dev.11` contains 341 selected
original/dependency-notice files from clean packaged commit
`50b862442df7b5edb1b7aad4b150e9b98b8d6230`. Its complete file set, sizes, hashes
and executable version passed readback verification; no game assets are bundled.
Receipt: `evidence/build-0.1.0-dev.11-package-validation.json`. This supersedes
the dev10 Rust diagnostic package for owner QA; it does not replace the retail
launcher tile and has not been published.

## 2026-11-15 dev10 diagnostic increment

**VERIFIED Rust execution:** source
`49aa8b6a60b99f5f10f43066deadbfd4879cfbd1` builds with the optional window feature.
Binary SHA256 `cec700be0be7a4ebef750a7acd337389bc187e3c0f3be3ba05614c11fba459a3`
passes eight focused CLI/image checks and a fresh isolated native-window smoke
presenting two frames, with no stderr. Saved login text remains 33/33 fields and
303 glyph fills; cursor and login pixels equal their retained diagnostic outputs.
The Plain-root initializer diagnostic remains a conditional 452-action model,
not attached retail startup. Receipts: private
`evidence/build-0.1.0-dev.10-smoke.json` and `dev10-native-html/receipt.json`.

The build adds checked attached display/source identities, active script-target
host calls, native zero-argument Object metadata, a fresh ExternalInterface
graph, bounded attached event state, resource localization and explicit
movie/player/receiver routing. Four display-state and three external-context
checks pass. The original attached cursor runs 14 instructions/six property
calls. The new composed localization route's original-wrapper validation is
separate and still in progress at this build checkpoint. World-transform math
also has independent explicit-input checks; actual Vista defaults remain gaps.

**OBSERVED retail limit:** the owner-documented standalone command remained a
black primary window through the bounded offline probe. Its original assets were
unchanged. No successful frontend, login or gameplay is claimed. The later
metadata-only trace is also bounded, and reaching its syscall cap is not retail
startup completion. Detailed private evidence is linked from the native-probe
documents.

The Linux dependency notice collection covers 114 dependencies/217 texts,
manifest SHA256 `3a48fbb2860337440711eb74d77d45c882700de3f8f2f91effc5022be518132c`.
The earlier all-platform metadata collection was incomplete and is preserved
separately; only `dependency-notices-dev10-linux` is selected for this Linux build.
This remains an original-code diagnostic, with no bundled game assets, no server
connection, and zero finished upgrades. Private staging is distinct from launcher
publication, for which no Rust delivery route has been confirmed.

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

Private working copy:
`~/.local/share/marvel-heroes-forever-rust/pins/alienware-candidate/`.
**VERIFIED at 2026-11-15 10:48:11 UTC:** the copy process exited successfully,
then every one of the 16,489 selected files matched the source size and SHA256:
21,193,180,456 bytes, zero problems. The directory name is retained so existing
evidence paths stay valid. Receipt:
`evidence/alienware-full-pin-verification-20261115T1050Z.json` (the filename is
an identifier; its `observed_at` field records the actual completion time).
This establishes the selected distribution's identity, not runtime parity or
coverage of files deliberately excluded from that inventory.
Source inventory SHA256:
`a03c7af289214495d2871956acaf4eec378a9a3868d4f06ef72f4c19d98116e0`;
verification receipt SHA256:
`a8994255f33d3bb5dc5da510d755d076b80028fc28d04c0363fffe0d116862c8`.

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

## Dev.7 integration progress

**OBSERVED (2026-11-15, original manager code in the Rust VM):** The evidenced
typed MovieClip subclass allocator advances the initializer diagnostic to 118
complete records and 18,572 instructions, retaining one Selection listener. The
next record starts at source757488 and stops at Enumerate2 source750939, because
native multi-name hash order remains gated. The native Object prototype's nine
stored keys and flags are represented; unimplemented method identities reject
on invocation. This is not full startup or a native-client execution result.

**VERIFIED (saved HTML composition):** All 33 placed login text fields now
prepare, including six HTML instances with exact imported font identities.
303 glyph fills are submitted. CLI image bytes match the independent adapter,
SHA-256 `67ca5b733220f7a71992ee8f024f75f0b35ec6da1d83804f404a15757dee6ae9`.
A fresh isolated Xvfb/llvmpipe native window presents two frames and closes at
its frame limit. Source placeholders, visibility and unsupported effects remain
explicit; this is an uninitialized diagnostic scene, not a functioning login.
Records: `evidence/dev7-integration/{initialization,login-render}.json` and
`evidence/dev7-native-html/receipt.json`.

**VERIFIED within their bounded profiles:** Nine action-queue tests and 29 VM
tests pass. The pure explicit-reference localization helper was independently
checked against all 77,089 English defaults: 70,103 preserve raw strings/source
spans and 6,986 reject unresolved formatting/sentinel cases. Actual nonzero empty
tooltips preserve true, and the stored empty zero-ID entry preserves false.
The queue has no runtime executor; the localization helper does not infer
effective prototype references or install an ExternalInterface callback.
See `evidence/dev7-integration/focused-tests.json` and
`evidence/localized-data-check/receipt.json` for scope and corrections.

**VERIFIED (final dev.7 focused checks):** All eight final CLI checks pass at
clean source `b5bfdf04044e5142820a3779f534ad2f414902c5`. The cursor remains
byte-identical to dev.6, and the login image remains byte-identical to the
independent HTML adapter after the additional native Object metadata change.
The final binary SHA-256 is
`9ae7285b644d4c80779bfb3844e8ce08833e8b0e33ae5e45960ba01908ecd3c6`.
The earlier native-window check used the same window/HTML implementation before
subsequent unrelated VM metadata, animation arithmetic and ledger additions;
its exact binary hash is retained separately. No owner-device test is implied.

The Float96 decoder now follows REA-observed sequential binary32 subtraction.
All 7,002 sampled W values match independent Python/scalar-SSE checks; precisely
1,468 changed from grouped arithmetic, while XYZ, frames, spans and bindings
remain unchanged. Five decoder tests and eight per-instance initializer-ledger
tests pass. The ledger submits ordered tags and marks before execution; partial
or unwinding submission failures quarantine the instance without replay.
Neither queue nor ledger is yet connected to the runtime executor.

Receipts: `evidence/build-0.1.0-dev.7-smoke.json`,
`evidence/dev7-integration/final-focused-tests.json`, and
`evidence/animation-runtime/w-fix/validation.json`. The window-feature executable
and standard-library test targets compile. Dependency notices cover 114 crates
and 217 texts, with no collection issues. Packaging and delivery status are
recorded separately; these checks do not establish full startup or game parity.

**VERIFIED (private dev.7 staging, 2026-11-15 09:19 UTC):** All 284 packaged file
hashes and the packaged version/Original checks pass. Packaged clean source is
`f2e35127dd82c4637365f23f1fbddb9d64209660`; its only difference from the tested
source is the preceding findings prose. Binary identity is unchanged. Receipt:
`evidence/build-0.1.0-dev.7-package-validation.json`. The artifact is under
`builds/0.1.0-dev.7` in the private project root. It contains original tooling,
documentation and dependency notices, with no retail binaries or game assets.
Publication and owner-device validation remain unconfirmed; the actionable Rust
diagnostic intake was queued for the coordinator, superseding dev.6 only.

## Dev.8 frontend runtime progress

**OBSERVED (integrated original manager, 2026-11-15):** Native ASCII hash ordering,
hidden InitObject properties, qualified class reads and the verified Function
conversion profile allow all 452 saved initializers to complete: 49,244
instructions, six host calls, one retained Selection listener and five class
registrations. This uses minimal explicit globals and an initially parentless
Plain root. Native global inventory and attached-root behavior remain incomplete;
this is not retail startup. The next original frame action stops after 31
instructions at the unimplemented MovieClip member boundary. Thirty-seven
focused VM tests pass. Receipts: `evidence/build-0.1.0-dev.8-smoke.json` and
`evidence/avm1-runtime/manager-frame-after-inits.tsv`; the earlier 447-action
receipt remains preserved at `evidence/dev8-integration/initialization-native-hash.json`.

**VERIFIED (resource preparation):** The new bounded localized-prototype resolver
preserves explicit parent copying and local zero overrides. A frozen independent
raw oracle matches all 27 selected UI keys' 54 references and origins. A separate
raw locale scan matches all 54 resulting text strings. The Help button inherits
its zero tooltip from the default layer; the other 26 explicitly store child zero.
Twelve boundary cases pass. The VM now materializes fresh native-profile Object
results with ordered ordinary setters, including supported setter reentry. Native
callback installation, configured label lookup and actual label updates remain
separate work. Receipt:
`evidence/localized-prototype-check/verification.json`; exact scope and source
ledger are in `docs/LOCALIZED_PROTOTYPE_FIELDS.md`.

**VERIFIED (dev.8 focused build/startup):** The window-enabled offline build and
eight final CLI/image checks pass. The original cursor output and saved login
HTML output retain their prior pixel identities. The exact tested binary also
presents two saved-login frames in a fresh private Xvfb window with zero stderr;
this uses llvmpipe, not an owner-device or retail comparison. The new animation
constant-key timing module passes five focused tests and its independent static
arithmetic comparison; interpolation and playback are not implemented. Test
source is `38c0570260c1ef21336e0bb8a55d8e7aa1bbb877`; binary SHA-256 is
`0c5b54295dff275370fb8c0f0904ab3e8734113cf4409dfb7a7eb70041b1fdcb`.
Receipts: `evidence/build-0.1.0-dev.8-smoke.json`,
`evidence/dev8-native-html/receipt.json`,
`evidence/avm1-runtime/host-object-fields-evidence.json` and
`evidence/animation-timing/validation.json`. Dependency notices cover 114 packages
and 217 texts without collection issues. Publication remains unconfirmed.

**VERIFIED (private dev.8 staging, 2026-11-15 09:42 UTC):** All 296 packaged file
hashes and packaged version/Original checks pass at clean source
`f4340908a55a64a0a3966be05011081296b6a970`. Only findings/provenance prose differs
from the tested source; the binary is identical. Artifact:
`builds/0.1.0-dev.8` under the private project root. Receipt:
`evidence/build-0.1.0-dev.8-package-validation.json`. The coordinator intake is
queued and supersedes dev.7's Rust diagnostic only. Staging does not confirm
publication or owner-device validation.

## Dev.9 runtime integration foundations

**VERIFIED (configured localization):** One pinned archive handle now resolves
the native-selected Globals defaults, typed UI directory Asset, ASCII label
identity and actual localized prototype membership. Four focused tests and the
independent 27-key resource comparison pass. Effective text fields preserve
parent origins and explicit zero overrides. In a separate controlled namespace,
the original selected script functions return all 27 real texts and 54 distinct
result objects through the new four-field VM conversion. That script check uses
explicit known members; native configured lookup through the full script path,
ExternalInterface installation and actual text-field lifecycle remain separate.
Receipts: `evidence/localized-label-resolver-check/receipt.json` and
`evidence/localized-vm-integration/known-members/MANIFEST.json`.

**VERIFIED (persistent continuation):** A consuming initializer handoff refuses
pending or halted phases and retains the same VM/code. Three focused tests pass.
An original-data probe completes 452 manager initializers and reaches the same
31-instruction first-frame MovieClip boundary. A separate integration then runs
the original cursor handler through the same FrontendHost: 14 instructions,
six property calls, retained Selection listener/class registry and the supplied
cursor position. These remain explicit Plain-object scene associations.
Receipts: `evidence/dev9-bootstrap-handoff/{receipt,display-host-receipt}.json`.

**VERIFIED (explicit camera arithmetic):** Original camera math generates all
16,384 table entries algorithmically and matches the frozen static oracle's
1,061 checked float words. It requires explicit pose, viewport, FOV, depth and
tangent provider. Actual camera selection, native floating-point environment,
constrained cropping and GPU comparison remain unverified. Receipt:
`evidence/camera-math/implementation-receipt.json`; its immutable
`implementation-receipt-reconciliation.json` preserves the exact historical
forward-oracle metadata bytes without changing expected outputs or source.

**VERIFIED (dev.9 focused build/startup):** The window-enabled offline build,
eight final CLI/image checks and two-frame fresh-Xvfb startup pass. Cursor and
saved-login images keep their prior pixel identities; startup emits no stderr.
Test source: `08d514cc167d1b914ade2ee2a45868c177054e22`; binary SHA-256:
`24d806e96b0b5af9ca7a8cc21a6ce4d25451614e07b010cc6597d317fbd033e1`.
Receipts: `evidence/build-0.1.0-dev.9-smoke.json` and
`evidence/dev9-native-html/receipt.json`. This remains diagnostic tooling, with
zero finished Forever upgrades and no playable-client or retail-parity claim.

**OBSERVED (retail reference):** Correcting only the private namespace's hostname
resolution advances the original client to a launcher-required Message dialog.
The run stops without input; all 453 frozen game files remain unchanged. No
missing-ClientConfig path was observed, so configuration retrieval was not
triggered. The capture route is held at that prerequisite while normal launch
requirements are studied read-only. See `docs/NATIVE_HOSTNAME_PROBE.md`.

**VERIFIED (private dev.9 staging, 2026-11-15 09:55 UTC):** All 303 packaged file
hashes and the packaged version/Original checks pass. Clean package source is
`78e023238b3e2d8846677882bc99404f15dab6c7`; only findings/provenance prose differs
from the tested source, with identical binary bytes. Artifact:
`builds/0.1.0-dev.9` under the private project root. Receipt:
`evidence/build-0.1.0-dev.9-package-validation.json`. The coordinator's queued
intake supersedes dev.8's Rust diagnostic only. Publication and owner-device
validation remain unconfirmed.

## Dev10 private staging receipt

**VERIFIED 2026-11-15 10:15 UTC:** private
`builds/0.1.0-dev.10` contains 310 original-only selected files. Every manifest
size/hash and the complete file set match; its binary reports the correct
version. Packaged source is `f205c53b1ebee392fcc06995e76049c709981887`; tested
code remains `49aa8b6a60b99f5f10f43066deadbfd4879cfbd1`. The immutable validation
receipt is `evidence/build-0.1.0-dev.10-package-validation.json`.
Coordinator intake was queued, superseding the dev9 Rust diagnostic only.
No launcher upload/publication or coordinator acknowledgement is claimed.
