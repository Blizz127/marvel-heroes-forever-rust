# Marvel Heroes Forever (Rust client) - project instructions

Task: Devbox task a3d58c579542 (project key `claude-marvel-heroes-forever-rust`, session `codex-marvel-heroes-forever-rust`). Full standing brief: ~/scratch/model-swap-20261110/brief-marvel-heroes-forever-rust-20261115-0000cst.txt

- Goal: 1:1 Rust reimplementation of the Marvel Heroes final 2017 PC client (build 1.52.0.1700, the Omega Steam client MHServerEmu fully supports), loading Blizz's own game files and connecting to MHServerEmu at mh.bansheerealm.com. Forever preset (all finished upgrades) ON by default, Original/retail one switch away.
- REA is mandatory (Blizz, 12:01 AM CST Nov 15 2026): use the `reverse-engineer-anything` skill on the 1.52.0.1700 client. Skill: ~/.agents/skills/reverse-engineer-anything/SKILL.md (Codex lists it under skill root r1 = ~/.agents/skills). REA MCP server `rea` is registered in ~/.codex/config.toml (Ghidra 12.1.4 at ~/re-studio/ghidra_12.1.4_PUBLIC). Do not run `rea setup` or edit shared client configs. Keep REA Evidence under ~/.local/share/marvel-heroes-forever-rust/evidence/ and cite it in docs.
- Original code only. Record every outside source (MHServerEmu AGPL, MHDataParser MIT, OpenCalligraphy, MHPatcher/Bifrost, UE Viewer, Benilla/Forever graphics layer) in docs/PROVENANCE.md with licence.
- Client binaries/assets/captures never enter git; they live under ~/.local/share/marvel-heroes-forever-rust/. No remote, no pushes without Blizz's word. Do not modify ~/dev/marvel-heroes*, ~/dev/MHServerEmu*, ~/dev/banshee-marvel-pack, ~/dev/mh-omega-study, or the launcher worktrees (Banshee Client Bot owns the launcher tile).
- Never press native prompts; label claims VERIFIED/OBSERVED/INFERRED.
