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
| `/home/blizz/dev/marvel-heroes/research/STATUS-marvel-heroes.md` and filename inventory of `/home/blizz/dev/marvel-heroes/run/` | Owner operational notes and distribution metadata; no separate licence inspected | Locate normal startup/wake context. The dated status is historical, not a current server observation; no executable, account, configuration or live data changed |
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

[Attached focus rectangle](ATTACHED_FOCUSRECT.md) records the native exact-false
setter and mask getter. The original typed VM implementation requires supplied
raw target state and preserves unrelated bits; no fresh-state or renderer-policy
default is inferred from its focused checks.

[Runtime text preparation](RUNTIME_TEXT_PREPARATION.md) projects admitted
per-instance document text and exact format handles into explicit renderer
inputs. Native style mapping and empty-document policy remain supplied; no
authored text, default style or rendered result is substituted by the bridge.

[Complete-distribution retail probe](NATIVE_FULL_DISTRIBUTION_PROBE.md) records
the first painted original loading splash in the isolated 16,490-file snapshot,
its conservative observer stop and complete before/after private input hashes.
It does not establish entry to the login frontend or server connectivity.

The [runtime text GPU observation](RUNTIME_TEXT_PREPARATION.md) records the
failed whole-scene visibility assertion, private dependency drift and installed
Pillow 13.0.0 metadata (MIT-CMU). Pillow only encoded unchanged diagnostic RGBA;
neither that tool nor the captures are bundled in the client.

[Attached Function.apply](REA_ATTACHED_FUNCTION_APPLY.md) records the owned
binary's typed display receiver conversion and independent invocation
Environment. [Saved event execution](SAVED_EVENT_EXECUTION.md) records persistent
source-wrapper pools, owner registers/operand stacks and the null activation
fence. These contracts do not make the generic diagnostic executor a native
startup or event dispatcher.
[Persistent saved-action state](SAVED_ACTION_STATE.md) implements the recovered
state ownership and successful pool/register operations in a standalone module;
its focused synthetic checks do not establish native event execution.

[TextField format mapping](TEXTFIELD_FORMAT_MAPPING.md) follows the owned
EditText loader through fresh HTML document formatting and layout consumers.
The recovered scalar style/effect values remain conditional on that specific
runtime path; current font selection and effective layout flags are separate
inputs. Raw tables, disassembly and all extracted movie data stay private.

[Ordinary applied super calls](REA_APPLIED_SUPER.md) records receiver-derived
prototype traversal separately from construction context in the owned native
client. Its original bounded VM implementation retains the typed receiver and
independent source/Environment identities.

[Known-splash continuation](NATIVE_SPLASH_CONTINUATION.md) records the completed
same-snapshot offline run and complete post-run hashes. Its original passive
window observer studied `/usr/include/X11/Xlib.h` API signatures under the
header's Open Group permissive licence; no Xlib implementation was copied.

[Runtime text visibility](RUNTIME_TEXT_VISIBILITY.md) records exact saved draw
order, triangle coverage, decoded bitmap alpha and the original visibility
action route. Its CPU explanation preserves the failed whole-scene GPU check.
Cached `wgpu-types` 27.0.1 blend constants (MIT OR Apache-2.0) were studied to
interpret the existing diagnostic blend state, not copied into game logic.

[Saved handler table](SAVED_HANDLER_TABLE.md) implements ordered, per-owner
source-wrapper handles from the registered native event contracts. Its supported
mask profile and focused synthetic checks do not establish lifecycle delivery.

The [fresh HTML scalar-format helper](TEXTFIELD_FORMAT_MAPPING.md) now implements
the qualified owned-binary derivation in original Rust. It preserves explicit
final-font, effective-layout and current-state inputs; conditional checks do not
observe native font selection or enable unsupported formatting paths.

[Attached visibility](ATTACHED_VISIBILITY.md) registers the owned native getter,
setter, render-node effects and conditional scheduling work. A renderer Boolean
alone is not presented as that complete setter. [Scale9 rendering](SCALE9_RENDERING.md)
extends owned-binary evidence to hierarchy, region transforms and fill mapping,
while retaining final coordinate/resource-binding gaps and the older helper's
inferred status.

[Login instance inputs](LOGIN_INSTANCE_INPUTS.md) inventories all saved nodes,
bindings and required supplied runtime state without manufacturing native
defaults. [Stage VM binding](STAGE_VM_BINDING.md) records the retained native
factory/listener contracts separately from property/resize effects.
[Attached property flags](REA_ATTACHED_PROPERTY_FLAGS.md) grounds the bounded
existing-own-storage operation in owned executable evidence.

The [startup-map entry comparison](FRONTEND_STARTUP_MAP.md) adds the owner's
pinned `MarvelEntry.upk` metadata and explicit unsupported object-preamble
results. This original-reader inspection does not execute an entry-map handoff
or redistribute its payloads; the owned proprietary source remains private.

[Saved clip-action loading](SAVED_CLIP_ACTION_LOADING.md) closes the owned native
record-mask and byte-span framing path while retaining the separate timeline
handoff requirement. [Native frontend startup stages](NATIVE_FRONTEND_STARTUP_STAGE.md)
records the bitmap splash, initialization ordering and conditional request wait;
it also registers the exact Microsoft API/bitmap pages, X11 headers and private
comparison tools used. Its bitmap mismatch and observation limits remain intact.

The [native Scale9 math module](SCALE9_RENDERING.md) implements recovered point
and matrix operations in original Rust with an explicit finite IEEE arithmetic
profile. Loaded CRT/MXCSR equivalence and final renderer binding remain
unverified; the native rendering profile is not enabled by numeric checks.
The [saved-handler VM adapter](SAVED_HANDLER_TABLE.md) now reads installation
identity from the retained VM arena. Neither successful binding nor the new
bounded saved-mode executor alone proves native event eligibility or timing.

The [complete scene adopter](LOGIN_INSTANCE_INPUTS.md) composes existing original
readers and VM interfaces with explicitly supplied runtime state. Its retained
147-node proof does not claim native fresh-state defaults or startup execution.
[Attached tab state](REA_ATTACHED_TAB_ENABLED.md) grounds `tabEnabled` access in
the owned executable and preserves its shared target-status word with focus state.

[Entry-package object evidence](ENTRY_PACKAGE_OBJECTS.md) registers the owned
UObject/class serializers used for narrow Level, Model, Polys and World property
admission. WorldInfo remains with the existing actor reader; retained native
tails and successful property parsing do not establish startup behavior.

[Saved dispatch](SAVED_DISPATCH.md) composes the previously registered native
wrapper, installed-vector and persistent Environment contracts in original Rust.
The phase leaves dynamic member lookup pending and does not infer event timing.

[Saved script calls](SAVED_SCRIPT_CALLS.md) records shared operands and exact
Function2 register allocation from the owned executable. [Saved assignment](SAVED_VARIABLE_ASSIGNMENT.md)
adds the native simple-name target route; it also registers the studied Microsoft
CRT `strchr` documentation (CC BY 4.0), used only to interpret delimiter search.
[MovieClip autoSize](ATTACHED_AUTOSIZE_FALLTHROUGH.md) records the concrete
case-3 ordinary-property fallback separately from TextField layout behavior.

[Saved scene handler binding](SAVED_SCENE_HANDLERS.md) uses the owned AS2
placement-loader evidence to retain each original record's source, mask and
installation identity. Its 34-handler original-data check executes no scripts;
runtime state, delivery and constructor scheduling remain separate requirements.

[World payloads](UPK_WORLD.md) records the owned native serializers and original
map comparisons used for ordered package-local links and preserved raw records.
The parser and inspection command do not load the referenced Level or infer
meaning for the remaining native fields.

[Attached target initialization](ATTACHED_TARGET_INITIALIZATION.md) separates
owned-native constructor checkpoints from placement, publication and callback
state. [Service test entry](MHSERVEREMU_TEST_ENTRY.md) registers the read-only
launcher/server/coordinator sources used to identify normal routes and the
unresolved service hold; it does not authorize bypassing that hold. The root's
single public HTTP SiteConfig read is retained privately and is not a wake call.

The [visibility host transaction](ATTACHED_VISIBILITY_BINDING.md) implements the
conditional native-to-render-state projection in original Rust. Prepared host
updates preserve local/effective visibility separately and do not expose native
allocator or backing-node internals as verified Rust behavior. [Attached names](REA_ATTACHED_NAME.md)
requires an explicit current-name snapshot; no root name is inferred from a file.

[Level payloads](UPK_LEVEL.md) registers the owned native serializer/getter
evidence for actor, Model, ModelComponent, URL and sequence-link framing. Ordered
links and opaque suffixes remain distinct; successful parsing does not execute
the selected sequence or establish full Level dependency coverage.

The [startup selection](FRONTEND_STARTUP_MAP.md) composes those same owned
native World/Level contracts with the original inventory reader. Its CLI adds
no external source or runtime default. The visibility integration check uses
artificial script/profile data and original project code; the separate native
claim remains the pinned executable evidence. The latest
[Scale9 binding investigation](SCALE9_RENDERING.md) traces the owned tag1009
descriptor, filename resolver, engine creator and post-creation inverse matrix;
unobserved runtime inputs remain explicit and no third-party implementation was
copied.

The dev15 Linux/window dependency collection contains the same 121 registry
dependencies and 231 selected licence/notice files. Its private manifest is
`evidence/dependency-notices-dev15-linux/manifest.json`, SHA-256
`9a372a0fdc06021ddaab37464e801cd5d5b6b8df126bdfd1d16a5a29579e34c6`.
The package contains original project deliverables and these notices, without
retail executable, assets, extracted data or captures.

[Ordinary underscore names](ATTACHED_UNDERSCORE_NAMES.md) records the pinned
native getter/setter and standard-name lookup evidence used for the bounded
storage admission. Its private inventory remains outside the repository; the
added predicate exposes membership only, without embedding native name data.

The scoped DIT operational handback
`/home/blizz/scratch/dit-marvel-retained-contract-20261115/HANDBACK.txt`
(SHA-256 `008746dd02eea46c6e657543ba4d89d53b7e4f90ef96cff4968e0ab82ecd60d8`)
updates [service prerequisites](MHSERVEREMU_TEST_ENTRY.md). It is owner-provided
operational evidence; no separate publication licence was established and no
implementation was copied. Root verified the handback hash, retaining its exact
copy privately. Its underlying controller/runtime claims are attributed to
DIT's retained review; this read did not query a service or clear any hold.

[Post-saved dynamic lookup](SAVED_DYNAMIC.md) composes the existing owned native
event-dispatch contract with original VM APIs; it adds no invocation or source
defaults. The [current display projection](FRONTEND_DISPLAY_STATE.md) is original
host integration using existing Scene/DisplayState/TextFieldHost definitions and
the already registered SHA-2 dependency. Its new checks use synthetic data;
no outside implementation, new native execution or third-party source was used.

[Model and Polys prefixes](UPK_MODEL_POLYS.md) records the owned build's 868/3
serializer/bulk-array evidence, paired original-byte validation, preserved opaque
tails and original-code source ledger. No new third-party implementation source
was read or copied; serialized polygons and later Model containers remain open.

[Fresh Environment initialization](FRESH_AVM1_ENVIRONMENT.md) records constructor
checkpoints in the same owned executable, with exact source/byte receipts and
later owner/publication boundaries. [Point construction](REA_POINT_CONSTRUCTION.md)
records native deferred registration, namespace/constructor graph and the
zero-argument allocation path. These are static interoperability findings;
neither supplies current adopted-instance state or a native execution result.

The dev16 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files, with no collection issues. Private manifest:
`evidence/dependency-notices-dev16-linux/manifest.json`, SHA-256
`2c9eaee7e1cda2c5a10ac8b66bd3287f716639e18fd71af33511331505620c4a`.
The original-only private package excludes retail binaries, assets, extracted
data and captures.

[Current-frame preparation](CURRENT_SCENE.md) composes original display,
TextField, font/layout and scene-content modules. Its real-file comparison uses
the already registered private frontend, manager and font pins without script
or GPU execution. [Fresh MovieClip prototype selection](FRESH_MOVIECLIP_PROTOTYPE.md)
records the owned native cache/global lookup and later registered-class
preparation boundaries. No external implementation was copied for either
increment, and controlled adopted prototypes remain distinct from fresh native
construction.

[Scale9 texture inputs](SCALE9_TEXTURE_INPUTS.md) records the exact owned tooltip
reference/export and conditional native field layout through saved Engine/Core
declarations and native property-link routines. The source ledger identifies
all reused private files and REA records. Loaded-field preservation, active link
mode and current resource state remain separate inputs; no renderer default or
new external implementation follows from this static study.

[Frontend movie defaults](FRONTEND_MOVIE_DEFAULTS.md) records selected owned
package defaults, native tagged Boolean application and conditional property
layout. Its source ledger identifies the retained MIT Unreal-Library profile
corroboration and all private package/native inputs; no new outside implementation
was copied. The original initial MovieClip selector implements the bounded
presence distinctions already recorded in [fresh prototype evidence](FRESH_MOVIECLIP_PROTOTYPE.md)
and verifies direct allocation separately from the later storage-creating setter.

The dev17 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev17-linux/manifest.json`, SHA-256
`1163dc008501337dc8837eff041345fee8ccf98e15682ee12cec8d4e858de2ce`.

[Rectangle construction](REA_RECTANGLE_CONSTRUCTION.md) records the owned
executable's deferred factory, typed object graph, descriptors and zero-argument
allocation path used by the original bounded implementation. Its evidence ledger
retains all exact native inputs and tool provenance privately. Unsupported native
methods and computed instance properties remain explicit; synthetic checks do
not establish full startup or retail behavior parity.

The [current-frame resource association](CURRENT_SCENE.md) now retains an
original Rust ownership token across AssetSet/MovieLibrary moves and checks it
before content resolution. Its focused tests reuse constructed project fixtures;
no external implementation or new native-state assumption was used.

The dev18 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev18-linux/manifest.json`, SHA-256
`6826bccc5a3305e8abff3d3712006772f80130a8b1c7248f997431c35cefaff0`.

[UClass metadata](UPK_CLASS.md) implements the bounded original parser profile
corroborated against the retained MIT Unreal-Library sources and selected owned
package/native evidence. Its source ledger distinguishes source/synthetic-only
nonempty collection coverage from the four original class comparisons; no
third-party implementation was copied.

[Fresh root naming](FRESH_ROOT_NAME.md) records the owned executable's formatting,
interning, name-field assignment and conditional fresh MovieDef/AS2 association.
The ledger identifies retained evidence, newly verified native bytes and assumed
ordinary imported CRT contracts. No CRT DLL or native client ran for this study;
current adopted-instance names remain separate inputs.

[Loaded Scale9 texture layout](SCALE9_LOADED_LAYOUT.md) extends the existing
owned declarations/native-property evidence with the conditional Object header
extent, class Link dispatch and Texture2D serializer's version-gated writes.
Its source ledger preserves remaining archive-state, field-reorder and PostLoad
conditions. No renderer input or runtime-state default is inferred from those
static paths.

[Current-scene script rendering](CURRENT_SCENE_SCRIPT.md) joins original project
bootstrap, host, frame preparation and renderer code with the previously pinned
manager assets and explicit diagnostic associations. The private ledger freezes
source, three package inputs, harness and pixel reference; existing Mesa/wgpu
tool provenance applies. No third-party implementation or native current-state
default was added.

[Fresh target state](FRESH_TARGET_STATE.md) records exact owned constructor
stores, conditional DataDef/getter identities and ordinary allocation
assumptions through one specific owner checkpoint. Its evidence ledger retains
all selected native ranges and independent checks privately. The values do not
replace snapshots for already adopted targets or prove later preservation.

[Mouse construction](REA_MOUSE_HIDE.md) and [user-event host selection](REA_MOUSE_HOST.md)
record the owned executable's deferred graph, finite member contracts and
state-4 cache/ownership paths. Original Rust implements only the bounded graph
and an explicit rejecting host boundary. Their ledgers retain exact private
REA inputs; no installed-handler or cursor-effect assumption is added.

[Construct dynamic lookup](CONSTRUCT_DYNAMIC_LOOKUP.md) reuses the owned movie
and existing original project code in one frozen controlled replay. The ledger
preserves the supplied state and all original comparison rows; no third-party
implementation or native initialization claim was introduced.

[Scale9 geometry staging](SCALE9_GEOMETRY.md) records owned native source-path
and generated-index contracts separately. Its original implementation and
saved-source smoke do not infer triangle generation or actual provider inputs.
[Texture archive and PostLoad](SCALE9_TEXTURE_LOAD.md) extends the same owned
package/native lineage through conditional archive state and resource dispatch;
later helper and lifecycle boundaries remain explicit. Both ledgers identify
their retained source/tool inputs, with no copied external implementation.

[Saved class-default preparation](CLASS_DEFAULTS.md) composes the project's
original UClass/property readers with checked archive identities. The source
ledger records exact owned package/native evidence and the retained parser
references already registered above. The four original default payloads use an
explicit observed-file profile; unknown schemas remain unresolved. No effective
default evaluator or copied external implementation was introduced.

[ModelComponent framing](UPK_MODEL_COMPONENT.md) records owned serializer,
factory and preamble evidence with an independent comparison of all ten saved
frontend components. Its original reader preserves unknown record words and
unimplemented polymorphic branches. The ledger retains exact native/tool inputs
privately; no third-party implementation or geometry behavior was copied.

The dev19 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev19-linux/manifest.json`, SHA-256
`7b839d6c672df00331859dba2e899a83fa850bf8cf1b78c473404bfc18cbe74d`.

[Conditional source-owner completion](FRESH_TARGET_COMPLETION.md) extends
the owned constructor evidence through ledger allocation, actual prototype
selection and explicit reference-cleanup paths. Its ledger identifies exact
REA, PE-import and retained source evidence plus the ordinary imported memset
contract. No CRT DLL or client ran, and unresolved bootstrap preservation is
not replaced by an initialized-state assumption.

[Fresh root resources](FRESH_ROOT_RESOURCE.md) retains the owned root/factory
dispatch and secondary-reference cleanup proof, including the exact inherited
allocator assumptions. [Texture resource construction](SCALE9_TEXTURE_RESOURCE.md)
records the selected owner-receiving constructor and remaining concrete update
dispatch boundaries. Both extend private owner-binary evidence with original
checking scripts; no native state, renderer defaults or external implementation
was substituted for the remaining conditions.

[Explicit ModelComponent binding](MODEL_COMPONENT_BINDING.md) joins original
Level/Model/component readers through checked saved links, retained bytes and
archive identity. Its owned-package comparisons use the existing private pins;
no additional external implementation was read. [Bounded native rectangle
tessellation](SCALE9_TESSELLATION.md) records exact owner-binary scan/triangle
paths, independently checked native bytes and supplied-input oracle limits.
Neither increment supplies current scene, material or renderer state.

[Windows cursor backend](REA_CURSOR_BACKEND.md) extends the owned executable's
hardware-cursor dispatch with concrete platform interface tables, bounded
constructor/factory evidence and named USER32 import calls. Its private ledger
retains every selected REA result and original PE byte/extent check. Current
window identity, actual OS effects and the separate Mouse.hide state-4 provider
remain unproved. No native client, input API or external implementation ran or
was copied; no new dependency was added.

[Selected-fill matrix arithmetic](SCALE9_FILL_MATRIX.md) reuses the owned
native paint/bounds/helper records and original finite Scale9 math. An independent
instruction-derived scalar oracle freezes operation-order witnesses before code
comparison; the ledger distinguishes those supplied inputs from live provider
or image state. No external implementation, dependency, native execution or
renderer default was introduced.

The dev20 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev20-linux/manifest.json`, SHA-256
`c663f0fe8d2042ba6081d3041f7fcbd9ce03451b8f7c1701a2f34b80556ddea8`.

[Native global extension state](REA_GLOBAL_EXTENSIONS.md) records exact owned
getter/setter and property-order evidence. Original Rust implements an opt-in
same-context subset, preserving unknown adopted state. The original manager12
replay demonstrates the unsupported numeric input separately from earlier
Plain-global fixtures; no coerced value or startup success was substituted.

[Hardware cursor calls](REA_HARDWARE_CURSOR.md), [fresh movie default state](FRESH_MOVIE_DEFAULT_STATE.md)
and [texture manager updates](SCALE9_TEXTURE_UPDATES.md) retain exact owned
package/native producer, layout and dispatch evidence in their source ledgers.
Actual backend receivers, nested default-copy effects and later resource
lifecycle remain explicit boundaries. No new external implementation, live
service action or native runtime state was introduced.

[Selected Model node geometry](UPK_MODEL_NODE_GEOMETRY.md) records original
bounded joins between saved Node, pool, position and surface records, with
independent comparisons against both owned map exports. No triangulation or
material implementation was borrowed. [Bootstrap binding preservation](FRESH_PROTOTYPE_BINDINGS.md)
extends exact owned startup evidence through explicit cache/global/prototype
conditions; it does not supply those conditions as runtime defaults.

[Native output ownership](SCALE9_OUTPUT_RECEIVER.md),
[native options and mesh flags](SCALE9_OPTIONS.md) and
[initial texture-query checkpoints](SCALE9_TEXTURE_QUERY_CHECKPOINT.md) retain
the exact constructor, receiver, upload and resource/query records listed in
their ledgers. These original static studies keep current inputs and callback
effects explicit. No external SDK implementation or live renderer state was
substituted for the remaining evidence.

[Positive rectangle tolerance](SCALE9_TOLERANCE.md) reuses exact owned scan and
endpoint evidence, with original frozen instruction-derived witnesses. It adds
no external implementation or assumed current options. [Fresh root publication](FRESH_ROOT_PUBLICATION.md)
records conditional field/name changes, queued work and cleanup through normal
return, without applying those fields to an adopted populated Scene.

The numeric [global-extension study](REA_GLOBAL_EXTENSIONS.md) additionally
consulted Intel's [SetDenormAreZeros documentation](https://www.intel.com/content/www/us/en/docs/ipp/developer-guide-reference/2021-12/setdenormarezeros.html)
(copyright Intel; documentation only) to distinguish DAZ-controlled subnormal
behavior from the unobserved native floating-point mode. No Intel implementation
was copied. Original integer/double conversion retains that unsupported boundary.

[The selected format query](SCALE9_TEXTURE_FORMAT_QUERY.md) binds its returned
format key separately from stored image dimensions and matrix state.
[Fresh inherited default copying](FRESH_MOVIE_DEFAULT_COPY.md) closes the selected
finite property graph under explicit construction/allocator conditions. Both
reuse the owned executable/packages and original validation tools named in
their private ledgers. Neither establishes current texture state or effective
StartPaused without the remaining lifecycle evidence.

The dev21 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev21-linux/manifest.json`, SHA-256
`a80c24af0fe897a30ad9b0930ae4c36a271a975a355cb3eb3df30e4fb99db15f`.

[Missing Mouse handler diagnostics](REA_MOUSE_MISSING_HANDLER.md) reuses the
owned proprietary executable and retained constructor/import records to bind
the diagnostic, target log getter and concrete engine sink. Its source ledger
distinguishes selected callbacks from actual current handler/logger state.
No external implementation or live invocation was used.

[Rectangle AA](SCALE9_RECTANGLE_AA.md), [packed positions](SCALE9_PACKED_VERTEX_FORMAT.md),
[cursor coordinates](CURSOR_COORDINATES.md) and [Model resource records](MODEL_RENDER_RESOURCE.md)
use the exact owned native/package evidence listed in their individual ledgers.
Original Rust and independent instruction-derived comparisons cover only the
admitted supplied inputs. Proprietary SDK/game source was not copied; no new
dependency, runtime setting or actual device selection was introduced.

[Early binding construction](FRESH_BOOTSTRAP_BINDINGS.md),
[cursor visibility layout](REA_CURSOR_VISIBILITY_PROPERTY.md),
[Node consumers](NODE_RENDER_GEOMETRY.md),
[position reconstruction](SCALE9_POSITION_RECONSTRUCTION.md),
[cold root registration](FRESH_ROOT_COLD_REGISTRATION.md) and
[selected default serialization](FRESH_MOVIE_DEFAULT_SERIALIZATION.md) extend
the same owner-only static compatibility studies. Their ledgers record exact
paths, ownership/licences, selected raw evidence and conditional limits;
none establishes complete native startup or an effective default from absence.

[Original global producers](FRESH_GLOBAL_BOOTSTRAP.md) completes the bounded
pre-registrar construction interval using the owned executable and retained
REA records. Actual zero-reference cleanup is included under the stated heap,
collector and no-reentry contracts; this introduces no outside implementation.

The dev22 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev22-linux/manifest.json`, SHA-256
`f3094c061eab878708c774bd82c0f810e749a4a9e3661c820aaf597c0ce01de0`.

[Fresh Mouse state construction](REA_FRESH_MOUSE_STATE.md) and
[later state preservation](REA_MOUSE_STATE_PRESERVATION.md) join exact owner
executable, state-map and adapter evidence. The private ledgers distinguish
conditional constructor absence from the later live parent-bag chain and cache.
These original static studies add no outside game implementation, host override
or current-state assumption.

[Fresh-root composition](FRESH_ROOT_NORMAL_RETURN.md) joins the previously
registered owner-executable producer and publication records without new source
material or native execution. Its remaining resource, policy and lifecycle
conditions stay explicit.

[Context diagnostics](REA_CONTEXT_DIAGNOSTIC.md) closes the selected fixed
formatter with exact owned code/table/import evidence, an independent original
instruction-span oracle and original Rust byte preparation. Its source ledger
records the retained REA/objdump/Python/Rust tooling and proprietary-artifact
scope. No external game/SDK implementation was copied; actual logger dispatch,
variadic formatting and native encoding remain separate.

[Selected default preload](FRESH_MOVIE_DEFAULT_LOAD_LIFECYCLE.md) uses the same
owned executable/packages and retained schema/copy evidence to distinguish the
CDO-specific class serializer from ordinary UObject dispatch. Its exact-source
ledger preserves cold-reference, alternate-input and later lifecycle limits;
no effective StartPaused value was assumed or external source code copied.

[First engine-name initialization](REA_ENGINE_NAME_STATE.md) and the extended
[Mouse preservation ledger](REA_MOUSE_STATE_PRESERVATION.md) reuse the exact
owned executable and private registered native records. They close stated
forwarding, optional-task and publication intervals while retaining current
provider, notification and later initialization boundaries. No runtime state
or global handler absence was inferred from a prefix.

[Diagnostic Environment state](REA_DIAGNOSTIC_ENVIRONMENT.md) distinguishes the
stack-local policy record from the saved incoming context pointer using exact
owned interpreter/helper evidence. The ledger retains an incomplete query and
excludes it from positive coverage; no current null pointer is assumed.

[Draw mesh identity](SCALE9_DRAW_MESH_IDENTITY.md) joins the same record collection
through native upload and draw forwarding. [Model index resources](MODEL_INDEX_RESOURCE.md)
adds original integer-phase construction from explicit resolved identities,
buffer/map snapshots and ordered saved component inputs. Their ledgers record
the owned executable/packages, retained REA evidence and original independent
comparisons; no external renderer implementation, live material identity or
current lifecycle state was substituted.

[Asset-bound index preparation](MODEL_INDEX_BINDING.md) composes those original
readers with the retained AssetSet namespace and original package bytes. Its
focused production harness reuses the sealed independent fixture expectations;
no new outside implementation or native execution is involved. The private
source-replacement audit records the core API's lifetime-only change.

The [Object/Function correction](AVM1_VM.md) uses the recorded fresh-bootstrap
producer evidence and exact owned normal/raw getter bindings. It preserves
effective identities and flags in original Rust while retaining explicit
unsupported callback and metadata-mutation boundaries. The private receipt
preserves the focused harness corrections and failed assertions.

[Selected default PostLoad](FRESH_MOVIE_DEFAULT_POSTLOAD.md) reuses the owned
factory, class and serialized link evidence to bound the action CDO's direct
mutation paths. It adds no outside implementation, observed default value,
runtime invocation or hold clearance.

[Option-owner construction](SCALE9_OPTIONS_CONSTRUCTION.md) uses the same owned
executable, exact tables and retained helper/import evidence to extend the
eighteen-word store checkpoint through one conditional constructor return.
The predicate-set backend and live mesh-owner joins remain open; the private
receipt retains one provider timeout without promoting it to known completion.

The dev23 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev23-linux/manifest.json`, SHA-256
`2fa9d39d6c75fc0dee953e15889225a84595c4ea54fdb20fc28c7364c7f80adb`.

[Model vertex interpretation](MODEL_VERTEX_INTERPRETATION.md) joins exact owned
producer, declaration, backend-table and literal evidence with independent
saved-field inventories. All 96 selected coordinate fields equal the rational
source expression, while source bits and finite-sample scope remain explicit.
No external graphics enum was assigned from memory, shader arithmetic copied,
codec transferred from another mesh class or renderer conversion added.

[Primary vertex export](SCALE9_PRIMARY_VERTEX_EXPORT.md) reuses the registered
owned descriptor-copy and generated-record scan bodies. Original Rust preserves
all record fields and cursor order, with explicit bounded admission. The focused
private composite retains its unchanged-support warning and makes no runtime
source, normalization or renderer-selection claim.

The later vertex-export ledger joins independently frozen raw-record witnesses
and exact owned instruction bytes to the optimized original Rust comparison.
All source hashes, 41 private artifacts and production-library checking are
retained; no copied game data or tool output enters the implementation.

[Default input selection](FRESH_MOVIE_DEFAULT_INPUT_ROUTE.md) uses the same owned
loader/provider and saved-class evidence to distinguish the compiled zero-return
provider from the separate alternate archive. [Primary fill format](SCALE9_PRIMARY_SOURCE_FORMAT.md)
records exact selector/cache construction and distinguishes fresh format storage
from looser compatibility hits. Their remaining source/lifecycle identities stay
explicit; no new external implementation or runtime defaults were introduced.

[Function primitive callbacks](REA_FUNCTION_PRIMITIVES.md) use exact owned
callback objects, category/getter tables, active-context traversal and the
fifteen-unit literal. The original VM implementation retains proved identity
and result-slot profiles without copying native code or supplying unsupported
receiver conversions. Its private source ledger and twelve focused checks
separate synthetic execution from static instruction evidence.

[Option owner propagation](SCALE9_OPTIONS_MESH_IDENTITY.md) joins native root,
shape, compound and mesh construction stores, with concrete provider retention.
The extended engine-config and MovieDef cleanup studies keep real cache,
callback, notification and object-identity conditions explicit. All use the
previously registered owned executable; none selects current live state.

The dev24 Linux/window notice collection contains 121 registry dependencies and
231 selected licence/notice files without collection issues. Private manifest:
`evidence/dependency-notices-dev24-linux/manifest.json`, SHA-256
`80eb1d7cd1dc8327311a3359456e5bb68c1df354b88a29c043c2d9976e3e242d`.
