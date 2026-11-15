# Provenance and source register

Task `a3d58c579542`. Implementation is newly written Rust/Python for private
compatibility study. No retail binaries, assets, extracted data, captures or
reference implementation source are shipped in this repository or its builds.
A source being readable is not treated as permission to republish it.
No project-wide redistribution licence is selected in this initial private build.

## Format implementation and validation sources

The detailed ledgers linked here are part of this register and enumerate exact
files, licences, usage and limits for each component.

[AS2 global baseline](AS2_GLOBAL_BASELINE.md) records the private native bootstrap
inventory, deferred binding identities and later conditional host submissions.
Its source is the owner-held retail executable studied through REA; these static
observations do not establish a live initialized namespace.
[Configured localization labels](LOCALIZED_LABEL_RESOLVER.md) applies the owned
native path/hash and explicit configuration-field evidence using original Rust.
It retains one archive handle and typed resource origins; it does not install a
native callback or invent a directory/default result.

| Source | Licence / identity | Used for | Detailed file register |
| --- | --- | --- | --- |
| [MHServerEmu](https://github.com/Crypto137/MHServerEmu), read-only `/home/blizz/dev/marvel-heroes/MHServerEmu` | AGPL-3.0; observed HEAD `8fedb1a35922adfc9bff77e54f5fc3b6956073c9` | SIP fields, raw LZ4 designation, Calligraphy v11 metadata layout; source/docs studied, implementation written here | [SIP](SIP_FORMAT.md), [Calligraphy](CALLIGRAPHY_FORMAT.md) |
| Owner's server-data packages `/home/blizz/dev/marvel-heroes/Data/Game/{Calligraphy,mu_cdata}.sip` | Proprietary Gazillion data; owner-authorized private study, build unverified | Decoder comparison and metadata observations; separately hash-pinned | [Findings](../FINDINGS.md) |
| Owner's Alienware installation `/var/home/blizz/Games/banshee-library/games/marvel-heroes` | Proprietary Gazillion client and bundled dependencies; no redistribution permission inferred | Static build identity, private selected distribution pin, UPK profile and future behavior evidence | [Findings](../FINDINGS.md); private source inventory |
| [lz4_flex](https://github.com/pseitz/lz4_flex), cached 0.14.0 manifest/licence and compiled decoder | MIT | Independent byte-for-byte LZ4 validation in a private harness; not a client dependency; implementation source not used | [SIP](SIP_FORMAT.md) |

## Graphics and peer-project design references

| Source | Licence observed | Used for |
| --- | --- | --- |
| `/home/blizz/dev/benilla`: graphics status, root/app/world Cargo metadata, MIT/Apache licence files | `MIT OR Apache-2.0` | Module separation and requested/active graphics capability distinctions |
| `/home/blizz/dev/forever-client`: graphics status, enhanced-graphics, modern-ui, Cargo metadata and licence | MIT | Preset/apply behavior and optional UI boundary; no FFXI mechanics or implementation copied |
| `/home/blizz/dev/wotlk-forever/docs/{RUST_GRAPHICS_PORT_PLAN_20261109,FOREVER_GRAPHICS_REFERENCES}.md` | No blanket top-level licence found; source notes explicitly limit classic-code reuse | Shared settings versus host renderer boundary, matched-scene comparison criteria |

[Graphics plan](GRAPHICS_PLAN.md) contains the exact file-by-file ledger,
observed heads and evidence limits. The private reference hash manifest is
`evidence/options-reference-files-20261115.json` under the private project root.

## Discovery, preservation and tooling sources

| Source | Licence / status | Used for |
| --- | --- | --- |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/connection-and-recovery.md`, `references/native-and-artifacts.md`; installed `rea-agents` 6.9.0 CLI help and MCP schemas | Installed owner tooling; licence not inspected, no code copied | Required REA routing, connection recovery, evidence retention, CLI invocation |
| `/home/blizz/dev/dcuo-rust/FINDINGS.md`; names/structure under `/home/blizz/.local/share/dcuo-rust/` | Local owner-project documentation; licence not inspected | Private pins/evidence/build directory pattern and existing Alienware SSH route |
| Devbox Steam `steamapps/libraryfolders.vdf` and matching config path | Owner configuration metadata | Bounded search for installed client; no secrets or values copied into source |
| `/home/blizz/dev/mh-retail-accuracy-research.md` and targeted search results under `/home/blizz/dev/marvel-heroes/research/` | Local research; no separate licence inspected | Locate prior client/version and support notes; no implementation copied |
| `/home/blizz/dev/banshee-marvel-pack/docs/MARVEL_HEROES_TABULA_RASA.md`, `docs/channels/marvel-heroes.json`, targeted search results from `scripts/publish-marvel-heroes-channel.sh`, `docs/{ARCHITECTURE,ENB_INTEGRATION,SETTINGS,HALO_INTEGRATION}.md` and `docs/channels/README.md` | Local owner launcher project; licence not inspected | Client executable/install location and existing channel identification only; no downloads, publishing, launcher edits or launcher source copying |
| `/home/blizz/dev/mh-omega-study` filename discovery | Local owner research; licence not inspected | Look for existing UE3 study material; no implementation source found/copied |
| `/home/blizz/.local/share/devbox-coordinator/coordinator.py` report interface | Owner local operational tool; implementation not read | Authorized task status/handoff reporting only |

Search hits are discovery context, not proof of current runtime behavior. The
initial discovery inspected original-client executables as data. Later isolated
retail execution and its exact limits are recorded in
[native reference capture](NATIVE_REFERENCE_CAPTURE.md). No server data,
accounts, credentials, access settings or other project sources were changed.

## Named references not yet studied

[MHDataParser](https://github.com/Crypto137/MHDataParser) is identified by the
owner as MIT but has not been read in this increment. OpenCalligraphy and
MHPatcher/Bifrost are named future study inputs; their licences have not yet been
verified and no implementation is taken from them. UE Viewer and subsequent
compression study are recorded in the UPK component ledger when integrated.

## Integrated UE3/LZO reader

[UPK format ledger](UPK_FORMAT.md) enumerates exact primary-source paths and
versions. [UEViewer](https://github.com/gildor2/UEViewer) at commit
`a0bfb468d42be831b126632fd8a0ae6b3614f981` is MIT and was studied for UE3 summary,
name/import/export and compression-envelope fields. The Linux kernel's
[LZO format documentation](https://docs.kernel.org/staging/lzo.html) (GPL-2.0
prose) was studied for the original LZO1X decoder's grammar; no codec code was
copied. Installed `liblzo2.so.2` (GPL-2.0-or-later) was used only in a private
independent validation harness and is neither linked nor distributed. Its
copyright notice was read to verify the licence. The five proprietary sample
packages remain private. [SIP variant findings](SIP_VARIANTS.md) record the two
owner-provided package variants and attributed read-only backup hash observation.

## Prototype bodies and native import evidence

[Prototype body ledger](PROTOTYPE_FORMAT.md) adds the exact MHServerEmu
Calligraphy serializer/blueprint source locations studied (AGPL-3.0). The original
schema-checked decoder retains typed serialized values and does not copy runtime
class implementations. The independent scanner used system liblz4 1.10.0
(BSD-2-Clause library, private validation only); its copyright notice was checked.
The catalogue facade in `src/prototype_catalog.rs` is original project code over
those existing original parsers and has no additional runtime dependency.

Installed [REA](https://github.com/morluto/rea), package `rea-agents` 6.9.0 at
`/home/blizz/.local/lib/node_modules/rea-agents`, has an MIT licence verified from
`LICENSE` and `package.json`. Read-only source searches under `dist/ghidra/` and
`dist/process/` plus selected reads of `GhidraInstallation.js`,
`GhidraJavaProbe.js` and `OwnedCommand.js` diagnosed the Java probe's fixed default
5,000 ms deadline and inclusion of process ownership preparation. These files
were not edited or incorporated into the client. REA tool schemas and portable
PE import inspection were used on the private Win64 executable; exported Evidence
is private. This diagnosis does not claim that the timeout has been resolved.

[UPK object ledger](UPK_OBJECTS.md) adds exact pinned MIT UEViewer
`UnObject.cpp`, `UnObject.h`, `UnCore.h`, `UnTexture3.cpp` and `UnMaterial3.h`
references for property tags and the independent frontend object study. These
were studied without copying implementation. Selected private exported objects
and movie arrays were compared across original Rust and independent readers;
none is included in source or builds.

## Textures, movies and optional renderer

[BC format](BC_FORMAT.md), [Texture2D](TEXTURES.md), [texture cache](TEXTURE_CACHE.md),
[GFX format](GFX_FORMAT.md) and [frontend composition](FRONTEND_COMPOSITION.md)
extend this register with exact Microsoft/Adobe/Autodesk/Epic documentation,
MIT UEViewer field references and private owner-file observations. These are
format and interoperability studies, not imported implementations. The new
`frontend` bridge and CLI compose the original project readers and use those
private observations; they add no reference implementation source.

[Renderer](RENDERER.md) records exact installed primary wgpu/pollster API source
and dependency metadata read for original GPU code. The optional feature links
the registered third-party wgpu ecosystem (MIT/Apache and other declared
dependency licences); it is not claimed to be newly written library code.
No proprietary game bytes are embedded. Dependency notices accompany private
renderer builds, separately from asset evidence.

[Shape geometry](GFX_SHAPES.md) and [AVM1 decoding](AVM1_FORMAT.md) register the
Adobe-authored SWF v19 specification (copyright Adobe, study only) and exact
private owner movie evidence used for the new original bounded shape and action
readers. These parse geometry/instructions without importing another VM or
claiming scripting execution. Their source ledgers are incorporated here.

The texture-cache ledger also records MIT UEViewer `UnCoreSerialize.cpp` ranges
1036–1168 and 1531–1570 studied for bulk LZO framing. System liblzo2
(GPL-2.0-or-later) was used solely as a private independent oracle for three
owner-supplied cache ranges; it is neither copied, linked nor shipped. Full-cache
identity and rendering are separate from the recorded range/decode checks.

[Font and text reader](GFX_TEXT.md) registers Adobe SWF v19 chapters used for
original font/glyph/static/editable-text parsing, plus the exact private Startup
font movie and frontend samples. Glyph source spans are retained; font routing
and native typography are not inferred from successful parsing.

[Dependency notice collection](DEPENDENCIES.md) records Cargo-selected primary
crate metadata and exact upstream supplementary notice sources, including the
explicitly absent hexf-parse upstream notice and its separately identified CC0
declaration/standard terms. These are build dependencies and attribution records,
not proprietary game data or copied game-client implementations.

## Further frontend and native evidence

[Tessellation](GFX_TESSELLATION.md) describes the original bounded geometry
algorithm and exact inherited format sources. [Native window](WINDOW.md) adds
the pinned winit/wgpu APIs, X.Org test tooling and privately extracted official
Ubuntu runtime packages used only for isolated native presentation checks.
[Actual font resources](FRONTEND_FONTS.md) registers the newly hash-pinned
proprietary locale font movies and distribution defaults; fonts remain private.

[AVM1 execution](AVM1_VM.md) records Adobe SWF/ActionScript and primary ECMA-262
edition 3 references for the newly written bounded VM. [Button/control study](GFX_BUTTONS.md)
records Adobe format references, the cached BSD-3-Clause pypdf inspection tool,
and the actual absence of DefineButton tags in the studied frontend. Its standalone
button reader currently has synthetic coverage only and is not exposed as a
working retail-control feature.

[Prototype effective values](PROTOTYPE_VALUES.md) extends the exact read-only
AGPL MHServerEmu source ledger for parent/default behavior. The original resolver
retains unresolved runtime-assignment contexts and never substitutes server code.
[REA native alternative](REA_NATIVE_ALTERNATIVE.md) registers supported REA
Rizin/objdump capabilities, exact owner binary ranges, exported Evidence and
remaining runtime limits; no original executable was launched or patched.

[Movie library](MOVIE_LIBRARY.md) and [static scenes](GFX_SCENE.md) compose those
original readers with exact private import/export and placement observations.
The scene ledger records the inherited Adobe SWF documentation and explicit
static-frame interpretation; no outside scene graph or UI implementation is used.

[Glyph geometry](GLYPH_GEOMETRY.md) registers the Adobe-authored font/text format
references and actual private English/static-text glyph evidence used for the
original outlines and exact rational placement. Native rasterization and dynamic
text layout remain separate from geometric decoding.

[Triangle renderer](RENDERER_MESH.md) registers the exact primary wgpu and
Adobe/Autodesk sampling, gradient and clipping references used for the original
fill pipeline. Numerical policy choices and missing native rendering evidence
are kept explicit. [Localization](LOCALIZATION.md) records the exact read-only
AGPL MHServerEmu source study and private English locale/prototype files used for
the original Unicode string and UI-key bridge; no translation data is shipped.

[Plain editable text](EDIT_TEXT_LAYOUT.md) records the exact Adobe format and
TextFormat references for saved-field layout. [Static mesh geometry](STATIC_MESH.md)
records the read-only UE Viewer/UE3 study and private mesh evidence for the
original bounded reader. [Scene content](FRONTEND_SCENE_CONTENT.md) and
[initializer phases](FRONTEND_BOOTSTRAP.md) compose the existing original readers
using their inherited provenance; they include no outside runtime implementation.
[Static GPU composition](FRONTEND_PREVIEW.md) preserves those source ledgers and
records its explicit diagnostic color, texture and unsupported-effect policy.
[Scaling grids](GFX_SCALE9.md) registers the Adobe/Autodesk references and exact
private geometry observations behind the original bounded deformation helper;
native bitmap-fill paint averaging remains unresolved.
[Saved materials](MATERIALS.md) records the exact read-only UE Viewer source
files, license and private tagged-property/native-tail evidence used for the
original material dependency reader. Shader behavior is not inferred from graph
or parameter decoding.
[Saved scene objects](WORLD_SCENE.md) adds the exact pinned actor/component
property evidence and inherited package-reference source ledger.
[Selection listener state](FRONTEND_SELECTION_HOST.md) records the Adobe-authored
contracts used by the original bounded native-binding host, separately from
unverified GFx focus-event behavior.
[Focus-handler dependencies](FRONTEND_FOCUS.md) records the exact primary Adobe
contract and owned constructor/callback spans, including GFx controller extensions
that are not established by Adobe's two-argument focus-event documentation.
[Mutable display state](FRONTEND_DISPLAY_STATE.md) records Adobe MovieClip and
Autodesk cursor documentation, the actual saved MouseMove handler and original
bounded affine/property integration. It does not establish startup or native
event delivery parity.
[Saved HTML text](GFX_HTML_TEXT.md) records Adobe htmlText documentation, exact
owner formatting/token/font-linkage evidence and original bounded parsing.
Paragraph/run representation is separate from pending HTML layout and rendering.
[Native class registry](REA_CLASS_REGISTRY.md) records exact REA owner-binary
evidence for AS2 root-context ownership, key comparison and registration/removal,
separately from unresolved display action scheduling and constructor invocation.
[Frontend class bindings](FRONTEND_CLASS_REGISTRY.md) applies that native evidence
in an original bounded registry for one VM/root context. Call/definition origins
remain provenance; registration does not instantiate a class.
[Native Function calls](REA_FUNCTION_CALLS.md) records the exact owned-binary
REA Function/apply registration, receiver and empty-argument evidence, with
closure/frame boundaries kept explicit. Its selected callable-conversion and
empty-apply-list profiles now support the original bounded VM; other contracts
remain gated as recorded in `AVM1_VM.md`.
[Script Function conversion](REA_SCRIPT_FUNCTION_CONVERSION.md) binds the actual
DefineFunction2 interface and its callable getter. It preserves function-object
identity while clearing the wrapper's auxiliary frame reference; arbitrary
captured closures remain outside the admitted conversion profile.

[Native Array prototype](REA_ARRAY.md), [display action lifecycle](REA_DISPLAY_LIFECYCLE.md)
and [MovieClip callback](REA_MOVIECLIP_CONSTRUCTOR.md) register additional bounded
REA evidence from the owned executable. The callback alone does not establish
native allocation, and the lifecycle findings do not establish complete startup.
[Vista camera and browser surfaces](FRONTEND_VISTA.md) records exact saved camera
and sequence bindings and distinguishes browser replacement textures from the
unresolved Vista output. Camera math, active view and target dimensions remain
separate contracts.
[Skeletal mesh geometry](SKELETAL_MESH.md) records the exact read-only MIT UEViewer
source files, owner asset hashes and independent raw-field checks used for the
original bounded skeletal reader. Animation and skinning are separate work.
[Localization runtime trace](FRONTEND_LOCALIZATION_RUNTIME.md) records exact
owned script/native callback, export linkage, reflected aggregate and prototype-ID
formatting evidence. [Explicit-reference localization](FRONTEND_LOCALIZED_DATA.md)
applies its bounded successful plain-text profile.
[Localized prototype preparation](LOCALIZED_PROTOTYPE_FIELDS.md) records the
native parent-copy, explicit-zero assignment and postprocess evidence used for
its original bounded scalar resolver. Broader schema/default cases, fallbacks
and live callback startup remain separate contracts.
[Label lookup](LOCALIZED_LABEL_LOOKUP.md) records the configured data/asset
directory chain and native ASCII path/hash identity, including codepage and
failure boundaries. [Localization result objects](LOCALIZED_RESULT_OBJECT.md)
records fresh native Object construction, hidden metadata, ordinary setters and
the saved unpatched return-field order.
[Persistent pointer controller](FRONTEND_POINTER.md) composes the existing
original VM and display bridge, with the same Adobe/private-script provenance.
[Native scene host](NATIVE_WINDOW.md) registers the pinned winit/wgpu contracts
for persistent surface rendering, physical pointer coordinates and lifecycle.
It adds no dependency and retains the explicit native-platform validation limits.
[Saved animation data](ANIMATION.md) records the exact read-only MIT UEViewer
format study and independent original-byte comparisons for Loki's 72 sequences.
Omitted codec defaults remain explicitly caller-supplied and are not presented as
recovered retail defaults. Animation evaluation and runtime binding are separate.
[Native animation contracts](ANIMATION_RUNTIME.md) records the saved Loki scene
binding, class/default evidence and conditional codec timing/cache behavior;
unresolved final defaults and runtime masks are not supplied by inference.
[Constant-key timing](ANIMATION_TIMING.md) implements the observed conditional
index/alpha calculation with explicit inputs and cache knowledge, preserving the
native cache's omitted frame-count/sequence keys. It adds no playback clock,
interpolation, implicit cache initialization or scene-loop policy.
[Saved HTML layout](HTML_TEXT_LAYOUT.md) records exact imported-font selection,
original glyph/kerning geometry comparisons, and bounded REA evidence for FONT
color, spacing and kerning attributes. Its unsupported wrapping and alpha cases
remain explicit rather than rendered through a substitute plain-text path.
[Action scheduling](FRONTEND_ACTION_QUEUE.md) applies the owned-binary REA
cursor, token and nested-drain evidence in an original bounded queue, without
claiming a runtime executor. [Initializer submission](FRONTEND_INIT_LEDGER.md)
applies the per-instance/frame native ledger with explicit bounded-failure policy;
it is separate from actual loader triggers and bytecode execution.
[Construct/onLoad dispatch](FRONTEND_EVENT_DISPATCH.md) records exact receiver,
environment, retained source, handler and constructor ordering from REA, including
remaining display lookup, delivery-state and error-policy gaps.
[Native capture readiness](NATIVE_REFERENCE_CAPTURE.md)
records local Wine/capture-tool inspection, namespace isolation checks and
initialization of a new private Wine prefix, plus two bounded retail runs that
did not reach the frontend. [Native startup postmortem](NATIVE_STARTUP_SETTINGS.md)
ties the captured null hostname-resolution fault to the owned executable and
distinguishes the later settings verification and error-handler fault.
[Corrected hostname probe](NATIVE_HOSTNAME_PROBE.md) records a third isolated
run that reached a launcher-required message and was stopped without UI input.
[Native enumeration](REA_ENUMERATION.md) records the owned-binary string hash,
collision, growth, hidden-literal metadata and visitor evidence. Derived orders
remain distinguished from retail process observations.
[Qualified variable reads](REA_QUALIFIED_VARIABLES.md) records native scope and
member lookup for the observed class registrations; the diagnostic Plain root
does not establish attached-root lookup behavior.
[Attached MovieClip identity](ATTACHED_MOVIECLIP.md) distinguishes the display
receiver from lazy script storage and records property precedence, state and
onLoad gates without substituting generic-object behavior.
[Attached member names](ATTACHED_MEMBER_NAMES.md) records the finite native
standard-member map and exact ordinary-name admission gates, including the
distinction between a missing property and a present Undefined value.
[ExternalInterface binding](EXTERNAL_INTERFACE_BINDING.md) records deferred
allocation, member metadata and handler selection through the current caller
Environment; it does not establish preceding script-context selection or install
a native engine receiver.
[Native HTML wrapping](HTML_WRAPPING.md) records rounding and word-break evidence
while retaining unresolved translator/font-state gates; the existing diagnostic
layout is not promoted to native parity by a conditional mathematical model.
[Camera math](CAMERA_MATH.md) records explicit-input native rotation, view and
projection arithmetic plus private instruction/byte comparisons. Its original
mathematical replay does not establish the selected runtime camera or GPU output.

[Explicit movie callback routing](FRONTEND_EXTERNAL_HOST.md) applies the owned
receiver/context evidence in original Rust; supplied handler/player identities
are distinguished from automatic startup or instance discovery.

[World transforms](WORLD_TRANSFORMS.md) records owned native actor/component
math and a saved-property audit. Its class/default grammar study also used
[UELib](https://github.com/EliotVU/Unreal-Library/tree/8236c39b79e30887a487ecf9b018204e430a5ebc),
MIT, pinned commit `8236c39b79e30887a487ecf9b018204e430a5ebc`, and GNU objdump
2.46 (GPL-3.0-or-later, static analysis tool only). No reference code was copied
or executed. The detailed ledger includes exact selected UELib files, discovery
sources, licences and private source hashes.
[TextField runtime](TEXTFIELD_RUNTIME.md),
[deferred builtin values](DEFERRED_BUILTIN_VALUES.md), and
[frontend receiver association](FRONTEND_EXTERNAL_RECEIVER.md) record further
private owned-executable/package studies through REA. These distinguish static
contracts from supplied runtime state and actual retail observations.
[Standalone retail probe](NATIVE_STANDALONE_PROBE.md) records owner-documented
launch arguments, Wine/tools ledger and a bounded offline black-window
observation; it does not establish a functioning login screen.

[Native Stage](STAGE_RUNTIME.md) records the owned executable's lazy callable,
context-specific viewport state, member flags, listeners and resize behavior.
Its private REA evidence and source ledger distinguish constructor initialization
from loaded frontend state and explicitly retain unresolved runtime inputs.

[Source integrity](SOURCE_INTEGRITY.md) records the linked RustCrypto SHA-256
primitive (`sha2 = 0.10.9`, MIT OR Apache-2.0), its versioned primary documentation
and cached manifest/licence observations. Project verification glue remains
original; dependency notices are packaged separately from private source data.
[Initial frontend instance](FRONTEND_INITIAL_INSTANCE.md) records the owner's
pinned MarvelFrontEndMap and its conditional OpenMovie producer through REA.
[Instance text preparation](SCENE_TEXT_INSTANCES.md) retains explicit runtime
style/provenance inputs and unchanged saved definitions. Neither proves native
lifecycle execution. [World transform defaults](WORLD_TRANSFORM_DEFAULTS.md)
records native copy/constructor gates while preserving unresolved effective inputs.

[Attached name classification](ATTACHED_NAME_CLASSIFICATION.md) uses complete
owner-private inventories authenticated with the exact-buffer digest seam.
The inventory data remains outside git and packages; live VM state guards remain
separate from static membership.
[Reduced native trace](NATIVE_REDUCED_METADATA_TRACE.md) records actual offline
package opens, local connection metadata and unchanged private inputs.

[Localization preparation cache](LOCALIZED_LABEL_CACHE.md) records the original
instance-local immutable cache, actual resource/VM comparisons and paired private
measurements. That measurement harness also read glibc (LGPL-2.1-or-later)
`/usr/include/time.h`, `/usr/include/x86_64-linux-gnu/bits/time.h`, and
`/usr/include/x86_64-linux-gnu/bits/types/struct_timespec.h` for the process-CPU
timing ABI/constant only; exact private hashes and limits are in the linked ledger.
No glibc implementation was copied into product sources.

[Attached TextField host](TEXTFIELD_HOST.md) composes the owned-binary typed
identity and conditional text-state contracts with exact scene bindings. Its
synthetic cross-layer checks remain separate from original-script execution,
native startup and renderer integration.

[Saved startup map](FRONTEND_STARTUP_MAP.md) prepares explicit serialized
sequence properties and ordered links without inventing omitted defaults or
executing the native player lifecycle. [Targeted startup trace](NATIVE_TARGETED_METADATA_TRACE.md)
and [original client configuration](NATIVE_CLIENT_CONFIG.md) record the isolated
retail path observation and private inspection of the owner's exact file.
[Text vertical layout](TEXTFIELD_VERTICAL_LAYOUT.md) records saved action order,
native alignment and metric paths, preserving unresolved early state and public
transform semantics. [Native String methods](STRING_NATIVE_METHODS.md) records
the owned executable's boxing/constructor graph and exact ASCII uppercase map;
unsupported Unicode and mutable-graph cases remain explicit. All native evidence
and raw source inputs remain private under the paths in those studies.

[Configuration-present probe](NATIVE_CONFIG_PRESENT_PROBE.md) compares the exact
original file against its preceding absence while preserving the unsuccessful
frontend result. [Source and Environment versions](TEXTFIELD_SOURCE_ENV_VERSION.md)
distinguishes imported definition, selected MovieDef, field ancestor and captured
function identities through owned-binary REA evidence.

[Saved movie binding](SAVED_MOVIE_BINDING.md) validates exact saved Scene/library
relationships while requiring an explicit ordinary-binding runtime profile; its
facts do not observe a retail loader decision. [Complete snapshot plan](NATIVE_FULL_SNAPSHOT_PLAN.md)
records private preservation checks and the installed GNU coreutils 9.7
(GPLv3-or-later) reflink capability probe. The native String implementation uses
the existing [owned executable study](STRING_NATIVE_METHODS.md); its synthetic
checks remain distinct from execution of original login methods.
