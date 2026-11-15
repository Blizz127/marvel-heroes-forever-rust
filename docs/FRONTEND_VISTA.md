# Frontend Vista camera and output boundary

**VERIFIED saved linkage:** the Asgard Vista has one explicitly bound camera and
its own 16-second movement sequence. The UI's character-select and legal-document
textures belong to a separately evidenced browser pipeline. A correct camera
projection and Vista output surface have **not** yet been established, so this
document retains those inputs as explicit gaps for the next rendering step.

This is a compatibility study of the owner's pinned 1.52.0.1700 files. Export
indices below are zero-based; positive UE3 object references are one-based.
Logical package offsets address decompressed export bytes. GFX offsets address
uncompressed movie bytes. Native addresses are preferred-image Win64 addresses.
No client, browser, server, GPU or native UI was executed for this investigation.

## Saved camera identity and timeline

**VERIFIED (original package bytes):** `MarvelVista_AsgardHub.upk` export 79,
object reference 80, is `theworld.persistentlevel.cameraactor_0`, class
`CameraActor`. Its serialized span is `1878174+896`; its complete tagged-property
list follows the preserved 26-byte actor prefix.
Its class reference is import `-10`, `engine.cameraactor`; its archetype
reference is zero. That null archetype does not supply an explicit default-object
link or settle native class-default application.

| Camera property | Explicit saved value | Logical payload offset |
| --- | --- | ---: |
| Location | `(215.5206298828125, -205.3515167236328, 771.9002075195312)` | 1878974, 12 bytes |
| Rotation | signed integer words `(-8199, 5106, -28)` | 1879018, 12 bytes |
| AspectRatio | float bits `0x3FCCCCCD`, approximately 1.6 | 1878249, 4 bytes |
| bConstrainAspectRatio | false | Boolean tag at 1878200, 25 bytes |
| FOVAngle | no directly saved property | — |

These are raw scene values. This study has not assigned world-axis directions,
angle units, handedness, rotation order, view-matrix layout or a projection matrix.
The actor's saved postprocess settings also reference `luts.asgardlower_lut`
(object 1432, export 1431); rendering only its mesh placements would omit authored
postprocessing and other material/lighting work described in
[WORLD_SCENE.md](WORLD_SCENE.md) and [MATERIALS.md](MATERIALS.md).

The camera is identified by actual object references:

| Link | Saved relationship |
| --- | --- |
| SeqVar_Object 1313 | `ObjValue=80`, payload `3684657+4`, selects CameraActor 79. |
| SeqAct_Interp 1303, `Fade Once ` | `VistaCam_Asgard` links to object 1314, that SeqVar; Data selects InterpData 150. |
| SeqAct_Interp 1304, `Loop Cam & Move` | `VistaCamera` links to the same SeqVar; Data selects InterpData 151. |
| LevelLoaded event 1311 | `Loaded and Visible` explicitly links to Play on actions 1303, 1304 and Loki-animation action 1305. |
| InterpData 151 | Saves length 16 seconds and group objects 156/161, exports 155/160. |
| Director track 163 | Its time-zero cut selects the group name `vistacamera`. |
| Camera group 155 | References float-property track 166 and move track 168. |

**OBSERVED:** move track 168 saves `IMF_RelativeToInitial`. Its six subtracks
171–176 each preserve four keys, including two distinct keys at time zero.
Subtracks 172–176 explicitly declare TranslationY, TranslationZ, RotationX,
RotationY and RotationZ. Subtrack 171 omits `MoveAxis`; its title `X` is not a
substitute for the missing enum. The stored `Default__InterpTrackMoveAxis` also
omits that field. The key order, interpolation-mode words and tangents remain in
the private evidence; no duplicate keys were removed or curves evaluated.

Track 166 names `FOVAngle`, but it has no saved FloatTrack. The default-named
Engine object `Default__CameraActor`, export 8252, explicitly saves `FOVAngle=90`
(bits `0x42B40000`, payload `3500317+4`), aspect approximately 1.77778 and aspect
constraint true. The actual Vista camera overrides the latter two properties.
These are observed default-object records, not proof that the final native
camera inherits a particular FOV after all loading and updates.

The 62.10246276855469-second `ManuCam1`/`CamMover` graph is separate. Both
FrontEndMap action 26 and Asgard action 1306 (`UI Background Camera Scene`) save
empty actor-link arrays for those labels. Their names and matching duration do
not bind them to CameraActor 79. This refines the earlier broad camera-scene
observation in [FRONTEND_COMPOSITION.md](FRONTEND_COMPOSITION.md).

**Unresolved runtime linkage:** saved graph edges do not prove event timing,
which camera becomes the local view, blending, fade application, loop restart
state or the treatment of duplicate-time keys. The frontend map also opens the
manager and streams Asgard, and Asgard has another saved movie-open link. Their
complete runtime ordering has not been executed here.

## Browser textures are a separate output path

**VERIFIED saved identities:** the three render-target-named MarvelGame exports
are serialized `Texture2D` assets. Manager image bindings preserve logical image
sizes separately from the stored texture mip dimensions:

| Texture name / export | Manager bitmap ID | GFX logical size | Stored Texture2D size |
| --- | ---: | ---: | ---: |
| CharacterSelectRenderTarget / 17037 | 11 | 1280 × 720 | 2048 × 1024 |
| HelpPanelRenderTarget / 17046 | 2 | 954 × 641 | 1024 × 1024 |
| LegalDocRenderTarget / 17047 | 1 | 500 × 400 | 512 × 512 |

Login bitmap 40 resolves to the legal-document texture; character-select bitmap
28 resolves to the character-select texture. These identities do not make their
stored mip sizes camera target dimensions.

**OBSERVED original AVM1:** manager declares browser IDs LegalDoc=6, HelpPanel=7,
CharacterSelect=8. `CharacterSelectView.UpdateCharacterSelectRTPosition`
(`923168+777`) and `LoginDialog.UpdateLegalDocRTPosition` (`1349904+293`)
call `UpdateWebviewComponentBounds` with the corresponding browser ID, bounds,
scale and a boolean. `ShowCharacterSelectVideo` (`924108+110`) calls
`LoadCharacterSelectVideo` and enables the webview component; its close path
disables that component. These are saved method bodies, not executed screen state.

**OBSERVED package operands; INFERRED bounded UnrealScript call semantics:**
`CharacterSelect.RenderCharacterSelectWebviewComponent`, export 8854, supplies
browser 8, Width/Height properties and literal `CharacterSelectRenderTarget` to
`AddBrowser`; `LoginDialog.RenderLegalDocWebviewComponent`, export 9180, supplies
browser 6, 500 × 400 and literal `LegalDocRenderTarget`. Their enabled paths call
RenderBrowser and SetBrowserFocus; their disabled paths remove the browser.
`SetUpWebviewComponent`, export 6191, creates an Engine `TextureRenderTarget2D`
with supplied SizeX/SizeY, black clear color with alpha one and SRGB=false, and
assigns it through `Awesomium.AweWebViewComponent.SetTextureTarget`. `AddBrowser`
passes the target to `SetWebviewTexture`; the binding reaches
`Outer.SetExternalTexture`. Complete selected token-length agreement supports
this interpretation; two Context skip fields retain explicit uncertainty.
The saved AS render methods themselves include an empty legal-doc stub and a
trace-only character stub. Package `SetFunctionDelegates`/`ActionScriptSetFunction`
records supply the separate delegate-binding evidence; merely invoking a stub
would not reproduce browser setup. Help uses browser 7 with 954 × 641 arguments.

Thus the named saved Texture2D objects and runtime-created render targets occupy
different stages of an evidenced browser binding. This does not establish the
selected backend, color transfer, live dimensions, browser readiness or final
GPU pixels. No browser was launched or URL fetched.

Shared manager code also contains `MarvelPanel.SetupModelRender`,
`CreateModelRender`, `AdjustModelRenderPosition` and `DestroyModelRender`.
Their `RenderMarvelModelToPanel` request includes 1920 × 1080, and the matching
native-facing declarations belong to `MarvelGFxGameUI`. No active Vista call is
established by those shared methods. Their dimensions cannot be reused as a Vista
target size. The selected manager/login/character disassemblies contain no decoded
instruction reference containing `vista`; this is a bounded literal-search result.

## Selected native evidence

**OBSERVED with REA on the pinned Win64 executable:** the native name table at
`0x1429e51e0` associates `UMarvelGFxFrontEndexecLoadCharacterSelectVideo` with
`0x14137b3e0`. Its complete 246-byte wrapper dispatches through receiver vtable
slot `+0x6b0` at `0x14137b48d`.

The class factory at `0x1413b8070` supplies the literal class name
`MarvelGFxFrontEnd` and helper `0x1413bce30`. That helper installs table
`0x14228b110`; the observed `+0x6b0` entry is `0x1413bf1a0`. The complete selected
1656-byte implementation preserves browser-8 constants, including the call at
`0x1413bf64d` with `dl=8`. This links the selected static frontend implementation
to the saved video/browser interface. It does not capture a live receiver or
prove that every subclass uses this table.

The CameraActor factory at `0x140a8df10` supplies helper `0x140a917b0`, which
installs table `0x141f34118`. Separately, the native name table maps
`ACameraexecSetViewTarget` to `0x140ad7020`; that wrapper calls `0x140a9be10`.
The selected routine includes current/pending view-target state and blending
branches under an **INFERRED** field-role interpretation. No recorded call in this study connects its actor argument to Asgard
object 80. These generic camera observations do not establish the frontend's
active view or a view/projection formula.

The next named Engine metadata boundaries are `CameraActor.GetCameraView`
(export 8244, `3499469+130`, with OutPOV and DeltaTime child properties),
`Actor.CalcCamera` (674, `2930352+210`), `Camera.UpdateViewTarget` (2708,
`3083441+1401`) and `Camera.UpdateCamera` (2491, `3067938+325`). The first two
original function bodies are retained privately; their native/script body
interpretation and calling contract remain unresolved here. This provides exact
follow-up identities without assigning unnamed table slots or guessing a matrix.

Raw RIP-relative instruction bytes were used for table addresses. Some Rizin
pretty-printed symbol aliases incorrectly display unrelated exported audio names;
those aliases were not used to name camera constructors or tables. An initial
selected read used the wrong adjacent table address and ended midway through an
instruction. Its immutable Evidence is retained, but the complete corrected
wrapper/table read supersedes both selections. Automatic string-inventory warnings
about two large skipped sections remain in the receipts; selected explicit reads
are not a claim of a complete native string inventory.

## Contract and remaining implementation boundary

A source-driven loader can now preserve the exact CameraActor identity,
reference graph, initial saved fields and unmodified track records above. It can
also route the evidenced browser names and logical dimensions independently of
world-scene rendering. It must keep the following unresolved values explicit:

| Required rendering input | Established fact | Remaining evidence |
| --- | --- | --- |
| Camera instance | Asgard export 79 and saved sequence references | Native local-view selection, event order and transitions. |
| Time-varying pose | Relative-to-initial move track, six ordered subtracks | Axis 171's effective enum, angle units, transform order and curve evaluation at duplicate keys. |
| FOV | Named FOV track without saved keys; default-named object stores 90 | Effective actor defaults and native updates. |
| Aspect | Actor saves 1.6 and constraint=false | Native aspect policy with actual viewport dimensions. |
| View/projection | Raw position and rotator words retained | Axes, handedness, matrix convention, FOV interpretation, near/far planes and depth range. |
| Output | Named browser replacements are separately evidenced | Vista backbuffer/viewport ownership or any actual capture target and its dimensions. |

No SceneCapture, RenderTarget or TextureTarget identifier occurs in either map's
complete names/import/export tables. This bounded metadata absence does not rule
out runtime-created objects or references in opaque native payloads. Neither a
1920 × 1200 GFX stage nor an image mip dimension supplies the missing 3D surface
contract. No new renderer or source change accompanies this document.

## Evidence, validation and provenance

Private evidence root: `~/.local/share/marvel-heroes-forever-rust/evidence/frontend-vista/`.
The `assets/` ledger pins exact objects, properties and originals; `frontend/`
retains movie functions and browser metadata; `native/` retains REA output and
bounded byte-scan candidates. Original code and extracted data stay private.

**VERIFIED:** the asset checks covered 81 focused exports and 2657 top-level/nested
property payloads. Independent map/default checks covered 697 FrontEndMap tags
and 35 Engine-default tags. Movie checks verified 21 function headers/body spans,
three root action tags and the browser-ID assignments. Of 34 selected UnrealScript
profiles, 32 consumed their complete selected token/length profile. Frontend Start
(export 8632) and WidgetInitialized (8684) contain unsupported tokens and retain
no partial semantic claim; the two Context skip uncertainties above are separate.
Every selected property payload was compared with its original serialized span.
The native reads completed with exit zero, no output
truncation and unchanged executable digest. These are static checks, not a native
launch, rendered comparison, scheduler test or retail-parity result.

`verification.json` independently checked 808 decoded instructions / 3479 bytes
against the original executable and PE function ranges, plus 126 retained helper
file hashes. Its SHA-256 is
`ad0ec3daa5834ec52f814bdcad796ece995315d4f36d3fddc79a84d312cc2e64`.
The canonical five-record `native/native.bundle.json` has SHA-256
`20e89073d7557f2daf859e72b5877637a3494c7a7069fd2fd03cd3196946a70b`.
It includes the explicitly superseded initial selection and these claim-bearing
REA records:

| Evidence ID | Selected scope |
| --- | --- |
| `ev_91b12769355bcec2f99461cc9b1dd11aa946802779e2388ec1a698ced8bf7729` | Correct native-name tables and complete video/camera wrappers. |
| `ev_903b7e475dbbad0c3b6ba4587abcd0f3935eb1b8c6f6fba1caabf38e72715696` | CameraActor/frontend/texture class-factory ranges. |
| `ev_5924ec1504756acbe439ae2edfda0ec0f1137a33b849d5b76d9e30c0fb47dde9` | CameraActor/frontend construction helpers and selected SetViewTarget routine. |
| `ev_416753ffcb9db059d94bbb9d5ca8bf08e346403ac112eacb5b45bea819ada9dd` | Frontend table slot and complete selected character-video method range. |

No native target session was opened. The task's isolated five-record REA ledger
was exported and checkpointed before resetting it; no shared engine or other
lane's native process was changed.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Pinned `MarvelVista_AsgardHub.upk`, SHA-256 `2f45ae3e0490c54a9ec37884b463f96690c805b77cc244664719a59fe2754153` | Proprietary owner-authorized interoperability input | Camera, sequence, scene fields and bounded metadata search. |
| Pinned `MarvelFrontEndMap.upk`, SHA-256 `9264322b115aa7264f751b26f981ec62b88b91e7542d569b0f33c24a364b91f2` | Same | Startup graph and unbound camera labels. |
| Pinned `MarvelGame.upk`, `MarvelFrontEnd_SF.upk`, `Engine.upk`, and original extracted manager/login/character movies; full hashes in private ledgers | Same | Movie image IDs, browser code/metadata, class and default-named records. |
| Pinned `MarvelHeroesOmega.exe`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Same; 1.52.0.1700 Win64 | Selected REA native-name, wrapper, factory, table and function reads. |
| Original project `src/{world_scene,material,upk,texture,gfx,avm1,frontend}.rs` and relevant submodules, `docs/{WORLD_SCENE,MATERIALS,FRONTEND_COMPOSITION,FRONTEND_LOCALIZATION_RUNTIME}.md`, earlier original private inspectors | Original code/analysis with existing provenance | Reuse established decoders and identifiers; no outside implementation copied. |
| `~/.agents/skills/reverse-engineer-anything/SKILL.md`, relevant native/artifact and evidence guides, installed REA schemas/help | Toolkit workflow documentation; licence not established by this read | Scoped static analysis and immutable Evidence retention. |

No new external repository, SDK, dependency or web source was read. Parser-layout
claims remain separate from native execution. Existing source provenance is
retained in the referenced component documents and `PROVENANCE.md`.
