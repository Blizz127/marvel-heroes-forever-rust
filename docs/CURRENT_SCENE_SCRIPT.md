# Current-scene preparation after the original manager MouseMove

**VERIFIED controlled integration:** on frozen source
`33ab978b086f6bba1a811fbd689f6b273151d042`, the same FrontendHost completes
452 original manager initializers, receives the original MouseMove handler,
and supplies its resulting display state to `current_scene::prepare`.
The resulting headless frame matches every RGBA byte of the retained dev.17
cursor reference. This proves this composition and handoff; it does not prove
native startup, complete event scheduling or retail rendering parity.

The original private harness is
`evidence/current-scene-script/harness/main.rs`. It compiled and ran successfully
on the first attempt. It uses the checked `InitSession::into_initialized`
handoff and retains the same VM and FrontendHost throughout. It does not use
PointerController's fresh diagnostic VM for the post-initialization step.

## Explicit source and execution profile

The AssetSet contains the owner's pinned `MarvelFrontEnd_SF.upk`, startup
`MarvelGame.upk` and `MarvelFonts_SF.upk`. MovieLibrary receives the explicit
English inspection alias `MarvelFonts.gfxfontlib -> MarvelFonts.fonts_en`.
This is a diagnostic input, not a selected runtime locale. The same AssetSet,
library entries and original `scene.movies` references serve both preparations.

The selected source is `MarvelFrontEnd.frontend_manager`, library ID 20 in
this load, GFX version 15, 2,055,748 bytes, SHA-256
`04d6748bc4ed845a7692a180d57b0fcd327fbbde9d84afacc461d55ca6ab8da4`.
Its structural scene has six nodes. Exactly one saved MouseMove handler is
selected: owner node 4, original byte span `7096+77`.

The VM root and a newly allocated **Plain diagnostic object** are explicitly
associated with the scene root and handler owner. No attached target flags,
native visibility defaults or native MovieClip construction are supplied.
The pointer is explicitly `(100,200)` stage pixels, represented as
`(2000,4000)` twips. The unchanged handler is parsed at its actual source
offset and invoked once with that diagnostic context.

The initializer outcome records 452 initializers, 49,244 instructions and six
host calls. This executes the initializer streams, including their ordinary
object/function operations; it does not separately dispatch the saved root
frame action, frontend display-constructor/Construct phase or a general event
schedule. The executed harness's shorthand “no constructors” refers to that
absent separately scheduled display phase, not to a prohibition on calls made
inside initializer bytecode.

The MouseMove execution then records 14 instructions and six property calls:

| Source offset | Operation | Pixel-space result |
| --- | --- | --- |
| 7127 | Read X | -40 |
| 7138 | Read MouseX | 140 |
| 7140 | Write X | -40 → 100 |
| 7158 | Read Y | -40 |
| 7169 | Read MouseY | 240 |
| 7171 | Write Y | -40 → 200 |

All six traces retain the same context, node and pointer sample. The cursor's
local translation becomes `(2000,4000)` twips. Only nodes 4 and 5—the cursor
and its geometry child—change world transforms. Display statistics change
from zero to six property calls, four recomputed nodes and revision two.
The same Selection listener and all five class registrations, including their
identities, origins and statistics, remain unchanged.

## Current-frame and metadata correspondence

A pre-handler CurrentScene is retained before running the handler. Afterward,
the harness clones that PreparedScene and overlays **only** each instance's
current world matrix and effective visibility from DisplayState. The complete
Debug representation of this expected value equals the newly prepared result.
The earlier owned frame retains its original digest.

This comparison covers all retained public fields: source/tree/placement
metadata, masks, colors, raw saved transforms, definition identities, geometry
and indices, cache entries, image/texture bytes, preparation issues, actions,
opaque records, unbound action tags and preparation options. Every immutable
DisplayNode source also matches its original SceneNode. Raw placement data is
not rewritten to disguise the current transform change.

Both calls to `current_scene::prepare` leave display statistics and traces
unchanged. The later result records display revision two and the exact
post-handler statistics. No TextFieldHost is attached, field/format bindings
are empty and `CurrentScene.text` is None. No runtime text, localization or
field-format state is inferred.

The prepared result contains one mesh, one image and one texture, two triangles,
16,384 decoded RGBA bytes, 461 source action bindings, five opaque records and
14 unbound action tags. Its 166,357-byte complete Debug representation has
SHA-256 `ec22a25c5c3680db4f86c829928b7bc3aa3f7fd4b7f47ba396faa21e916d3043`.
The earlier frame digest is
`c715dc607f91e16f80bb11bb66b50ace70a9a64cc3ffd41c82a510cb3854ad9f`.

Source action lists and action-related limitation records remain source
metadata. They are not an execution ledger and are not removed after this
separate execution. Preview's legacy “unexecuted” count aliases likewise do
not negate the separately recorded initializer/handler execution.

## Headless rendering and reference comparison

The new PreparedScene is passed directly to the existing preview renderer.
The stage is `[0,38400] × [0,24000]` twips, mapped to 1920×1200 pixels at
0.05 pixels/twip, without resizing or an offset. The saved root background is
opaque white. Explicit policy matches the dev.17 reference:

- `SavedRootTagOrTransparent`, linear `Rgba8Unorm` output;
- shader-space color transforms, sRGB gradient transfer and straight gradient
  alpha;
- `WholeImageReplacement`, no guessed encoding for unknown textures, and
  `BaseGeometry` for unimplemented effects.

These remain diagnostic choices; whole-image bitmap replacement and omitted
effects are not asserted retail behavior. The run uses the headless Vulkan
llvmpipe adapter, LLVM 21.1.8 / Mesa 26.0.8-1ubuntu0.3. It submits one fill,
two triangles and one texture. Existing coverage/limitation records are retained.

All **9,216,000 RGBA bytes** match
`evidence/dev17-final-cursor.pam`. Both payloads have SHA-256
`3f62ef7d92afd118bab614d612cc93c4a9f0b03f2b775dd0735795c0a43a9389`.
Alpha is 255 throughout; 761 pixels differ from white, with inclusive bounds
`[100,200]..[142,238]`. The independent check parses both PAM headers and
compares their complete payloads without conversion, flipping or unpremultiplying.

The historical reference used a single startup package, no font alias, and a
fresh pointer-probe VM **without** the 452 initializer phase. The matching
source manager, final geometry and explicit render policy permit this pixel
comparison; the two executions do not have equivalent lifecycle provenance.
The successful result is not a retail screenshot comparison or a complete
initialized frontend claim.

## Reproducibility and provenance

All evidence is private under `evidence/current-scene-script/`:

- `source-input-freeze.json` seals all 258 archived source files and the three
  package inputs at the exact source commit; readback confirms them unchanged.
- `harness-freeze-attempt1.json`, build/run logs and `run-attempt1.json` retain
  the executed original harness, selected lockfile and executable identity.
- `cpu-report.txt`, `property-traces.txt`, `gpu-report.txt` and
  `pixel-comparison.txt` retain results and limitations.
- `output-verification.json` and original `check_outputs.py` independently
  validate the six exact trace records, input/source hashes, image payloads
  and cursor bounds. No failed run was discarded or retried.
- `api-review/receipt.json` and `reference-review/receipt.json` separately
  retain the pre-execution API and historical-reference reviews.

The focused offline build used two compiler jobs at niceness 19 with the
existing shared renderer dependency cache. No full suite, native client,
window, server, account operation or network operation ran. No product source
changed for this check. The native/service/access restrictions remain intact.

Sources are this project's original frozen Rust code and the earlier original
private harness patterns `dev9-bootstrap-handoff/src/bin/display-host.rs` and
`current-scene/check2/original_login.rs`, plus the owner's proprietary pinned
packages and private dev.17 diagnostic reference. No external implementation
or new dependency was introduced. Existing REA-based native findings are not
promoted into additional native-state assumptions by this integration test;
no new REA operation was needed or performed.
