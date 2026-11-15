# Work status

Task `a3d58c579542` remains **working**. The goal is not complete. No retail
visual/gameplay/network parity claim is made.

## Current original build

`0.1.0-dev.1`: asset tools with SIP index/LZ4 reads, Calligraphy metadata,
UE3 868/3 name/import/export tables and LZO; Forever/Original preset foundation.
No renderer or server connection. Release build and focused CLI smokes passed.
Private build staging path:
`~/.local/share/marvel-heroes-forever-rust/builds/0.1.0-dev.1/`.
Publishing through the owner launcher remains a coordinator/publisher handoff;
this repository has no existing Rust-client private channel configuration.

## Active independent work

- Full selected Alienware distribution copy continues into `pins/alienware-candidate`.
  Source inventory is complete; all-file local hash verification must follow.
  Do not rename or modify the pin while its transfer is writing.
- Calligraphy helper implements bounded full prototype parsing in isolated
  `worktrees/calligraphy`, disjoint `src/calligraphy/prototype.rs`.
- UPK helper implements logical payload ranges and tagged object-property parsing
  in isolated `worktrees/upk`, `src/upk/objects.rs` plus coordinated UPK hooks.
- Root integrates each finished increment, advances version, builds/smokes and
  stages original-only artifacts. No pushes or remote configured.

## Next concrete steps

1. Integrate prototype and object payload decoders with real source-pinned tests.
2. Identify actual frontend textures, UI payloads and dependencies; implement
   the missing required asset readers before claiming milestone 1 complete.
3. Implement asset-driven frontend rendering and compare to the owner's client.
4. Follow the ordered login/character/world/combat milestones.

REA portable resource evidence identifies embedded executable build 1.52.0.1700.
Native Ghidra analysis remains unavailable because REA's Java readiness probe
reports a timeout during process ownership preparation despite correct installed
JDK25/Ghidra paths. Do not run setup or change shared configs; retain the exact
failure and continue supported REA/asset work. See FINDINGS.md and private evidence.

Server data and Alienware Calligraphy variants differ in one prototype; the
Alienware backup hash equals the server copy. Preserve both; do not overwrite or
restore the live install. Character/accounts/server state remain untouched.
