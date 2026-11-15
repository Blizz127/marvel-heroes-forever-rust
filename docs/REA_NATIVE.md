# REA native evidence

The executable is the private Win64 candidate identified in `FINDINGS.md`:
48,544,080 bytes, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
embedded version 1.52.0.1700. No executable was launched by these inspections.

## Portable import inspection

**OBSERVED (REA):** `inspect_pe_imports` reports complete coverage of the declared
import/delay-import directories: 40 DLL descriptors, 870 ordinary function rows
and 12 delayed rows. Evidence ID:
`ev_7136989a0b4ae1b35612e44bddcbe16bc3fbf8ad0f1245aaed529cbdb780c919`.
Full private bundle: `evidence/retail-pe-imports.bundle.json`.

The ordinary import table names D3D9, D3DCompiler/D3DX, libcef, Bink2, Vorbis,
Steam API, input libraries and Windows networking interfaces. D3D11, DXGI,
OpenGL and PhysXLoader appear in the delayed table. These are declared static
dependencies, not proof of which renderer, UI path, networking mode or physics
feature runs. The table does not reveal APIs resolved dynamically.

## Ghidra readiness limitation

**OBSERVED:** REA 6.9.0 recognizes the installed Ghidra 12.1.4 and matching native
decompiler when the CLI receives the already-existing `GHIDRA_INSTALL_DIR` and
`JAVA_HOME` paths. Its prerequisite Java observation returned `ETIMEDOUT` during
process ownership preparation. The direct installed Java probe reports JDK
25.0.5 and succeeds. No target import or decompilation occurred in that failure.

**OBSERVED (installed toolkit source):** `GhidraJavaProbe.js` passes a default
5,000 ms command deadline. `GhidraInstallation.js` does not supply an alternate
probe deadline in the inspected caller. `OwnedCommand.js` starts that deadline
before process launch/ownership preparation. The public analysis-settings schema
has a Ghidra startup timeout, but no Java-probe deadline setting. Changing the
unrelated startup timeout is therefore not a demonstrated remedy.

**INFERRED:** CPU pressure and low-priority scheduling may contribute, but the
source read does not establish the root cause. A normal-priority readiness-only
probe is recorded separately to test that specific hypothesis. No toolkit code,
shared client registrations or host access configuration has been changed.
No `rea setup` was run. Native engine availability remains a separate limitation
from the completed portable inspections.

Provenance: installed MIT REA 6.9.0 source/readme/schema details are listed in
`PROVENANCE.md`. All complete results and diagnostics stay under the private
project evidence directory, not in builds or git.

**OBSERVED (follow-up):** The explicit normal-priority `rea doctor --provider
ghidra --json` attempt returned the same ownership-preparation timeout. Its
scope checks pass installation, version, architecture, headless launcher and
native decompiler, then fail Java readiness. The priority hypothesis did not
resolve the failure. Receipt:
`evidence/rea-ghidra-normal-priority-doctor.json`. No unchanged retry is planned
until there is new evidence or a supported repair from the tooling owner.
