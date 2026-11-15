# Runtime text visibility in the saved login scene

Changing a submitted text draw does not guarantee a changed final pixel. The
first private original-action/GPU probe demonstrates that distinction: the
original uppercase action changes node 89's document and glyph geometry, while
the complete saved scene's resulting RGBA remains unchanged. This study keeps
that observation separate from startup visibility and from a glyph-isolation
control. It changes no product source, authored asset or panel visibility.

## Retained first GPU result

**VERIFIED retained bytes:** both 1920×1200 readbacks contain identical RGBA,
and the baseline matches the previously retained login diagnostic exactly.
The worker exited 101 at its assertion that the update must produce a visible
change. This is a failed expectation in the diagnostic probe, not evidence of
a native-client failure or proof of a text-renderer defect.

| Recorded quantity | Baseline | After the original uppercase action |
| --- | ---: | ---: |
| Node 89 submitted glyph fills | 11 | 5 |
| Node 89 submitted glyph triangles | 2,266 | 1,062 |
| All text fill draws | 303 | 297 |
| Prepared text instances | 33 | 33 |
| Changed final pixels | — | 0 |

All 147 per-node coverage records were compared; node 89 is the sole changed
coverage record. Source node IDs in that report are arena identities, not by
themselves painter-order evidence. The renderer validates the parent/child tree,
walks children in ascending saved display depth, and emits each node's fills in
that traversal order. Text glyphs retain their placement/fill order. Field clips
are applied to text and popped when the next node's draw begins.

The attempt's explicit policy uses `Effects::BaseGeometry`, whole-image bitmap
mapping, linear image encoding, shader-space color transforms, straight gradient
alpha and the saved root background. In particular it does not apply the
reported Scale9 effects. It ran on software Vulkan/llvmpipe. These are diagnostic
pipeline conditions, not a retail graphics preset.

The first attempt resolved `ordered-float` 5.5.1 while the frozen product lock
specifies 5.3.0. That dependency mismatch is retained alongside the failed
assertion. It is not assigned as the cause of the unchanged pixels. Subsequent
locked-dependency runs belong to separate receipts; the first attempt is not
rewritten or described as a pass.

## Target instance and clip

**VERIFIED source preparation:** node 89 belongs to ancestor chain
`[0,49,86,89]`. Each member is effectively visible in this structural snapshot,
has no active saved mask, and has identity world color
`multiply=(1,1,1,1), add=(0,0,0,0)`. Thus hidden state, inherited zero alpha or
a saved mask does not explain the target's lack of final pixel contribution.
These are the prepared snapshot's conditions, not a live retail observation.

Its world affine is
`[2.6346282958984375,0,0,1,16823.462829589844,12284]` in twips.
Saved EditText bounds are `x=[-40,1920]`, `y=[-40,440]`. With the supplied native
stage mapping, the clip corners span:

- x `835.9038848876953 .. 1094.0974578857422` pixels;
- y `612.2 .. 636.2` pixels.

The probe's conservative integer clip AABB is `[835,612,1095,637]`.
That box is not the actual glyph coverage. Occluder analysis must use original
triangles, transforms, paint alpha and applicable clips rather than treating
every bounding-box intersection as an overwrite.

## Later saved groups and startup code

The saved hierarchy identifies these later groups and descendants:

| Saved ancestor | Node | Selected later descendants |
| --- | ---: | --- |
| `legalBG_mc` | 103 | Shape 104 |
| `queuedLoginMessage_mc` | 105 | Shape 106 |
| `legalDoc_mc` | 111 | Shapes 112, 114, 116 |
| `compatLevelWarning_mc` | 129 | Background group 130, Shape 131; Text 132 has the authored `Destroy item?` label |

The latter text is a placeholder in the compatibility-warning subtree. It does
not establish that a destroy-item dialog is active. The four root placement
depths are 76, 78, 82 and 87 respectively; node 30's cinematic panel is depth 12.

Names identify source placements after their actual structural resolution; they
do not establish the groups' effective runtime state. The saved snapshot keeps
these groups visible before the missing startup/lifecycle execution. The
background shape/grid path is also being studied separately for Scale9; this
visibility study does not substitute a guessed native deformation.

**VERIFIED geometry and decoded alpha; INFERRED causal explanation of retained
pixels:** three later actual draws erase the target contribution under the
recorded diagnostic pipeline. Nodes **106, 112 and 116** use movie 13 bitmap 23,
resolved to package 0 (`MarvelFrontEnd_SF.upk`) Texture2D export 209, decoded
size 632×532 and exact asset path `marvelfrontend.login_tos_bg`. Each real
two-triangle union covers the entire field, with quad extent
`[644,334,1276,866]` pixels. Inverse world/bitmap mapping over that field requires
bilinear-support texels `x=191..450`, `y=277..302`, inclusive. All 6,760 decoded
alpha bytes in that support are 255. These instances have identity world color
and no masks. The selected alpha blend therefore removes prior target RGBA
contribution wherever the field can draw.

The source triangles give these post-clip glyph extents:

| Target geometry | Pixel extent `[left,top,right,bottom]` | Triangles |
| --- | --- | ---: |
| Saved text | `[845.39266336,617.871875,1084.58809638,629.3875]` | 2,266 |
| Runtime text | `[909.38531470,617.95,1020.61602807,628.60625]` | 1,062 |

Both are inside the field and the opaque covering unions. The later
glyph-intersecting nodes are 104, 106, 112, 114, 116, 119 and 131. Their roles
differ: node 104 is black with alpha 127; node 114's sampled field region has
zero alpha; node 119 has a tiny geometric overlap but no target pixel centers
in the analytic sample check. Node 131 is the final nonzero layer at those
centers, with sampled alpha approximately `0.8941..0.9059`; **116 is the last
opaque layer**. Nodes 139 and 141 have bounding boxes intersecting the field,
but their actual glyph triangles miss the target. None of these distinctions
requires fabricated rectangular geometry for a source shape or glyph.

The triangle/paint analysis is a CPU explanation of the retained GPU result,
not a new GPU rasterization or native parity test. It replays the exact retained
runtime override from the sealed original-action CPU case, labels that replay,
and preserves its conditional effective-style mapping. It does not rerun startup
or change visibility. The large opaque coverage is sufficient for the conclusion;
the analytic pixel-center details do not claim universal GPU edge-rule parity.

The exact saved/runtime planned draw indices are 294/288 for node 106,
302/296 for 112, and 304/298 for 116. The later compatibility background 131 is
321/315. These indices include actual shape fills and glyph placements, rather
than substituting source node numbers for draw order. Its `background_mc` parent
130 is local depth 1; earlier provisional depth-2 wording in another lane was
superseded by the exact scene/raw placement evidence.

**OBSERVED recovered action bodies:** the manager movie's
`MarvelLoginDialog` constructor, body `1344465+282`, contains explicit
`_visible=false` writes for these same names:

| Constructor write | Source byte offset | Matching saved ancestor |
| --- | ---: | ---: |
| `compatLevelWarning_mc` | 1344534 | 129 |
| `legalDoc_mc` | 1344656 | 111 |
| `legalBG_mc` | 1344672 | 103 |
| `queuedLoginMessage_mc` | 1344688 | 105 |

Its `Initialize` body, `1344802+1357`, also hides `cinematicsPanel_mc` at
1345235, matching node 30. These statements establish what the recovered code
requests when reached with the corresponding receiver and child bindings.
They do not establish that any given frame has executed those requests.

The conditional saved script route is explicit:

1. Login root DoAction tag 1483, timeline 0/frame 0, span `45737+428`, calls
   `MarvelUtils.setDocumentClass` at 45994, then `this.Initialize` at 46091.
2. Manager `setDocumentClass`, body `855817+34`, delegates at 855849 to the
   script function `MarvelUtils.registerClass`, body `855879+41`.
3. That helper assigns `mc.__proto__ = cls.prototype` at 855891 and invokes
   `Function(cls).apply(mc,null)` at 855918.

This script helper is distinct from native `Object.registerClass` scheduling.
The route requires root-frame execution, resolved class/helper/builtin values,
the correct root/child receiver associations and completion of preceding calls.
The class declaration has a pre-existing-global guard. The constructor's
superclass call at 1344480 precedes all four writes; Point construction and
background reads separate the first write from later ones. An optional
`_global.CLIK_loadCallback` call at 46069 occurs between the document-class call
and `Initialize`. These are real dependency/state boundaries, not an atomic
four-property initialization patch.
The uppercase text probe executes one selected original action against admitted
state; it does not run this complete login startup route. Its frame-coverage
script counters remain structural records, not a ledger of which external VM
calls actually ran.

Later recovered methods can change this state again: `ShowCompatWarning` and
`CloseCompatWarning` show/hide the warning, `OpenLegalDocView` shows the legal
panel/backdrop, and `ShowQueuedLoginMessage` shows the queued panel/backdrop.
The legal/queue close methods conditionally preserve the backdrop when the other
panel remains visible. `cinematicsClicked` toggles its panel. No invocation of
these methods is inferred from their existence or from authored placeholder text.

## Implementation boundary

The complete saved scene and its failure remain useful evidence. A separate
glyph-isolation control can retain the actual target transform, color and field
clip while checking only the selected glyph draws. Any visible change from that
control establishes that narrower graphics path; it does not establish normal
login visibility or authorize hiding the saved panels in the product.

The next lifecycle dependency is to establish and execute the original
document-class/initialization route with its real bindings, preserving any
unsupported operation as an explicit stopping point. Source code that would hide
a panel is not permission to inject that hide into the display state. Native
startup, effective formats, Scale9, transfers and retail pixel parity remain
separate claims.

## Private evidence and provenance

The new read-only study resides under
`~/.local/share/marvel-heroes-forever-rust/evidence/runtime-text-visibility/`.
`frame/receipt.json` SHA256
`969faa9408874864183833f8a0a3e8fd128024386e415ce9b3b663b8d8a1ed25`
seals copies of the first attempt's coverage, metrics, harness, dependency
comparison and exit receipt. It independently compares both RGBA payloads and
the retained baseline. `scene/path-audit/target-clip-receipt.json` SHA256
`cf77fa847663408400b1fb3a99eb7868f2fd7f89c385a5cb50b1cb3642dcb63b`
records the target's exact source hierarchy/clip/color facts.

`scene/geometry/receipt.json` SHA256
`fb03080a43352f3810d7eb6ec1e0969a641985ff10959c92da70b3b2f4e25f27`
seals 46 evidence files: original positions/indices in painter order, clipped
target polygons, actual triangle subtraction, inverse-matrix texture support,
decoded alpha, and source/field identities. The core-only exporter and bounded
analysis passed; 87 source/harness/input hashes and all three original UPKs were
verified unchanged. The initial unnecessarily broad private pixel enumerator
was stopped and retained separately; the bounded replacement and continuous
coverage/support checks passed. Analytic target centers number 1,020 for saved
geometry and 558 for runtime geometry. No GPU or native client was run by this
study, and no panel visibility was modified.

`actions/receipt.json` SHA256
`4f0d9c88baa155349a49fb60e6f3f1a886bd7383194d1c1d03076f9249cbaf2e`
verifies 13 selected original programs, 872 action records, five mapped hide
writes and four reused canonical native lifecycle records. Exact script headers,
constant pools, spans and source bytes remain in the private action evidence.
The action audit parses and checks those records without executing the scripts
or native client. All script offsets above are absolute movie-byte offsets,
not native addresses.

The study reuses the immutable original-asset CPU harness/source at
`runtime-text-preparation/actual/post-vm/source7041d19` and existing original
scene, action, renderer and lifecycle evidence. Owner-provided movie/package
bytes, extracted scripts and framebuffer captures remain proprietary/private
compatibility inputs. No new third-party implementation is incorporated.

The cached primary `wgpu-types` 27.0.1 source
(`MIT OR Apache-2.0`) was read to check the chosen
[`ALPHA_BLENDING` factors](https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.BlendState.html#associatedconstant.ALPHA_BLENDING).
The path, file hash and use are recorded in `frame/blend-source.json` and
`frame/blend-alpha-source.json`. Both destination factors are one minus source
alpha: a proved alpha-one covering draw removes earlier RGBA contribution under
this pipeline. No inference about native
retail blend state follows from that fact.
