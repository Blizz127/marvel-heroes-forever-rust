# Work status

Task `a3d58c579542` remains **working**. No retail visual/gameplay/network parity
claim is made. Milestone 1 is in progress; milestones 2–5 remain incomplete.

## Current original build

`0.1.0-dev.2`: SIP/LZ4 reads, Calligraphy metadata and complete structural
prototype decoding with lazy catalogue lookup, UE3 868/3 tables, LZO, checked
serialized export reads and supported asset property tags. No renderer or server
connection. Release build and focused catalogue/property CLI checks passed.

Private staging directory:
`~/.local/share/marvel-heroes-forever-rust/builds/0.1.0-dev.2/`.
Build 1 remains available. Publishing through the owner launcher is a
coordinator/publisher handoff; no Rust-client private channel is configured here.

## Active independent work

- Full selected Alienware distribution transfer continues into
  `pins/alienware-candidate`. Source manifest is complete; all-file local hash
  verification must follow. Do not rename or modify this destination while its
  transfer is writing.
- Texture helper: original inline Texture2D/mipmap/BC decode.
- GFX helper: actual movie headers/dictionaries/timeline/place records, with
  scripting and nested instantiation explicitly outside its first static pass.
- Renderer helper: wgpu image-quad upload/render/readback with private original
  texture validation. No mock UI or frontend parity claim.
- Texture-cache helper: actual manifest GUID/mip/name lookup; preserve ambiguous
  GUID mappings rather than silently choosing one.
- UPK helper: frontend startup/composition research. The startup map opens
  `frontend_manager`, not the login dialog directly. A small independent
  `pins/frontend-startup` copy requests shared Startup/GFxUI/MarvelGame/vista
  packages to resolve that dependency without waiting for the full transfer.

## Next concrete work

Integrate the actual texture and GFX paths, bind bitmap identifiers to original
assets, implement the frontend scene/runtime and render it. Continue ordered
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
