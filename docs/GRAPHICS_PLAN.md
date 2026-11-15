# Graphics and options foundation

Task `a3d58c579542`, 2026-11-15. This is an original implementation and a
source-document study, not retail runtime or REA evidence. No source code,
shaders, native tables, game art or capture contents were copied.

## Implemented scope

**OBSERVED:** `src/options.rs` provides the `original` and `forever` preset
names, rejects unknown names, defaults new session options to Forever, and
switches the complete preset through `ClientOptions::set_preset`. The catalogue
of finished upgrades is empty. Both selections therefore request zero upgrades.
The type does not read or modify saved preferences, start a renderer, or claim
that the current client matches retail. CLI/menu integration belongs to the host.

The owner's current Marvel instruction controls these defaults. Older peer
project documents sometimes specify Original or enhancement-off defaults; those
are observations about those projects, not Marvel's policy.

When a real upgrade is added, its same change must provide its independent off
switch, an honest capability result and a finished implementation. The Forever
preset selects every finished upgrade; Original selects none. A requested option
and the effect actually enabled on a device must remain distinguishable.
Unfinished DLSS, FSR, AA, HDR, shadows, draw-distance, scaling, UI and bot features
are absent from the player-facing catalogue.

## Architecture direction

**OBSERVED (peer source metadata):** Benilla separates file/asset formats,
world rendering, protocol and application/UI into crates; its world crate does
not depend on the protocol crate. The inspected metadata names Bevy 0.18.1 and
wgpu 27. These are the local reference's pins, not a dependency decision or a
claim about current upstream versions.

**OBSERVED (peer documentation):** WotLK's shared-source section describes
shared preset selection as pure logic, with host-specific render graphs, device
handles, material classes, color stages and coordinate adapters. It also warns
that a classic/native tree is not a blanket MIT/Apache source drop. The Forever
client's graphics status separates requested settings from active capabilities
and separates software-rendering checks from actual hardware observations.

**INFERRED (proposed Marvel design):** Keep package parsing, Calligraphy models,
network state and rendering separate. Follow the documented pure-settings seam
rather than importing another game's engine state. A small wgpu-based host is a
candidate for the original frontend; choose its actual dependencies when the
first asset-driven scene is ready. The reference does not establish Marvel's
coordinate, gamma, material or UI contracts; those require Marvel evidence.

## Ordered work and evidence

1. Complete the asset milestone: bounded SIP reads, Calligraphy data and the
   relevant UE3 package/object decoders. Identify frontend objects and their
   dependencies from the owner's pinned build and retained REA evidence.
2. Render an original frontend from those assets. Recover layout, text, focus,
   input behavior and scene composition from the matching retail build. Retain
   source/build/settings with each comparison; a drawn window alone proves
   neither asset correctness nor retail frontend parity.
3. Continue the requested login, character-select, world and combat milestones
   using the original presentation. Preserve game-specific material, camera,
   animation and UI behavior rather than substituting WoW or FFXI behavior.
4. Add finished enhancements in small increments. Keep the world rendering size
   separate from UI layout; make resolution/UI scale explicit when implemented.
   Add spatial AA/upscaling before temporal effects if that fits the actual
   renderer. Temporal effects need validated motion vectors, depth, history
   resets and UI composition. HDR scene precision and HDR display output are
   separate features. DLSS needs its actual SDK/device integration; a name or
   selection flag does not implement it.
5. Check each changed effect with a focused build and startup/feature smoke.
   Compare a matched original/effect-off scene and effect-on scene where useful,
   including motion when history is involved. Report the backend, device,
   resolution, options and unrun hardware checks. Shader tests or software
   readbacks do not establish whole-client retail parity.

**OBSERVED (peer documents only):** Benilla reports a software-Vulkan login
startup and explicitly leaves whole-client comparison and physical GPU checks
open. Forever reports selected offline renders plus limited owner-device DLSS
logs, with HDR display work still unobserved. These reports inform the evidence
boundaries above; none verifies the Marvel renderer.

## Source and licence ledger

These are all outside project files read for this increment. Paths identify
local study inputs. The listed repository heads are context only: documents
may contain local edits, so HEAD is not claimed to pin their bytes.

| Source paths | Licence observed | Use |
| --- | --- | --- |
| `/home/blizz/dev/benilla/docs/GRAPHICS-STATUS.md` | Repository `MIT OR Apache-2.0`; no separate document notice inspected | Graphics feature/capability distinctions and explicit runtime evidence limits |
| `/home/blizz/dev/benilla/Cargo.toml` | Declares `MIT OR Apache-2.0` | Workspace split, local Bevy/wgpu pins and build-profile structure |
| `/home/blizz/dev/benilla/crates/benilla-world/Cargo.toml` | Inherits workspace licence | Renderer/protocol separation; no implementation copied |
| `/home/blizz/dev/benilla/crates/benilla-app/Cargo.toml` | Inherits workspace licence | Host/UI dependency boundary; no implementation copied |
| `/home/blizz/dev/benilla/LICENSE-MIT` and `LICENSE-APACHE` | MIT (2026 Sam) and Apache-2.0 texts | Confirm the workspace licence declarations |
| `/home/blizz/dev/forever-client/docs/GRAPHICS-STATUS.md` | Repository MIT; no separate document notice inspected | Requested versus active settings, Original selection, evidence limits |
| `/home/blizz/dev/forever-client/docs/enhanced-graphics.md` | Repository MIT; no separate document notice inspected | Apply/revert behavior and original-asset preservation as design references |
| `/home/blizz/dev/forever-client/docs/modern-ui.md` | Repository MIT; no separate document notice inspected | Separate optional interface layer and input/focus behavior; FFXI-specific mechanics not adopted |
| `/home/blizz/dev/forever-client/Cargo.toml` | Repository MIT | Iterative build-profile organization only |
| `/home/blizz/dev/forever-client/LICENSE` | MIT, copyright 2026 Lenon Kitchens | Confirm repository licence text |
| `/home/blizz/dev/wotlk-forever/docs/RUST_GRAPHICS_PORT_PLAN_20261109.md` | No top-level licence found; document explicitly says classic sources are not a blanket MIT/Apache drop | Read-only roadmap and shared pure-logic/host-adapter boundary; no native/decompiled code, captured tables or shader contents read/copied |
| `/home/blizz/dev/wotlk-forever/docs/FOREVER_GRAPHICS_REFERENCES.md` | No top-level licence found; no separate document notice inspected | Matched-scene/motion comparisons and distinction between visible targets and algorithm proof; linked screenshots and videos not opened |

Observed heads: Benilla `a047d1028b7dc4907d5432c68c82769784273b0b`;
Forever client `a31e144a21290663bd48f7423f5d24ecaa137334`;
WotLK classic `ec6919baf9580f2c4938f20c941ee64273e5a1e8`.
No linked external pages, proprietary reference source or shared renderer source
files were read in this options-only increment. The references are already
local; no packages, SDKs or images were downloaded.

## Focused verification

**VERIFIED (standalone module, 2026-11-15):** `rustfmt --check` and
`git diff --check` passed. `rustc 1.98.1`, edition 2021, compiled the module as
an independent library at nice level 19 with no diagnostics. A private API
smoke compiled and exited 0, reporting default Forever with zero upgrades,
switch to Original with zero upgrades, restoration to Forever with zero
upgrades, and rejection of the unknown preset `ultra`.

The smoke transcript is
`~/.local/share/marvel-heroes-forever-rust/evidence/options-smoke-20261115.txt`.
The reference-file size/hash inventory is
`~/.local/share/marvel-heroes-forever-rust/evidence/options-reference-files-20261115.json`.
These are shell/source observations, not REA Evidence. Integrated Cargo/CLI,
renderer startup, saved-preference persistence, retail comparison and GPU checks
were not run in this helper's scope.
