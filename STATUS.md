# Work status

Task `a3d58c579542` remains **working**. No retail visual/gameplay/network parity
claim is made. Milestone 1 is in progress; milestones 2–5 remain incomplete.

## Current original build

`0.1.0-dev.3` candidate: the prior SIP/Calligraphy/UE3 readers plus inline texture
decoding, external cache metadata/LZO payload reads, GFX structures/shapes/text,
AVM1 structural decoding, saved-movie image binding and optional Vulkan image
inspection. The integrated image-binding and native-size offscreen checks pass.
Scripts and the full frontend are not running; no server connection is made.

Private staging directory:
`~/.local/share/marvel-heroes-forever-rust/builds/0.1.0-dev.2/`.
This is the previous staged build while the dev.3 dependency notice collection
and final packaging finish. Publishing through the owner launcher is a
coordinator/publisher handoff; no Rust-client private channel is configured here.

## Active independent work

- Full selected Alienware distribution transfer continues into
  `pins/alienware-candidate`. Source manifest is complete; all-file local hash
  verification must follow. Do not rename or modify this destination while its
  transfer is writing.
- Shape tessellation: actual decoded fill geometry, curves and winding handling.
- AVM1 execution core: bounded state and explicit native host boundary.
- Native window preview: compiled; a fresh isolated Xvfb test exposed missing
  X11 runtime library support before creating a window. Private runtime discovery
  continues; no successful presentation is yet claimed.
- Font routing: saved frontend imports/aliases need an actual dependency match.
- Prototype effective values: default/parent inheritance remains to be resolved.
- REA native alternative: bounded supported Rizin/objdump tools are being checked
  after the separate Ghidra prerequisite failure.

## Next concrete work

The actual texture/GFX paths and bitmap binding are integrated. Implement the
frontend scene/runtime and render it, retaining masks/text/script behavior.
Continue ordered
login/character/world/combat milestones after asset/frontend behavior is ready.
Calligraphy parent-value merging, runtime classes and property assignment are
still missing despite complete structural prototype decoding.

## Evidence boundaries and blockers

REA portable resource evidence identifies embedded executable build 1.52.0.1700.
Its import inspection is also retained. Ghidra is installed, but REA's Java
prerequisite process times out during ownership preparation under both tested
priorities. Direct JDK25 execution works. Public settings expose no Java-probe
deadline control; no setup/config/toolkit edits were made. See `docs/REA_NATIVE.md`.

Server/backup and Alienware Calligraphy variants differ in three design-state
fields of one Carnival prototype. Preserve both. No restore, account access,
server wake/login, source-install edit, push or launcher-worktree edit occurred.
