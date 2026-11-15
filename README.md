# Marvel Heroes Forever

Original Rust reimplementation in development, targeting Marvel Heroes Omega
1.52.0.1700 and the owner's MHServerEmu service. This is currently asset tooling,
not a playable game or a verified replacement for the retail client.

Client files, extracted data and reverse-engineering evidence remain outside
this repository under `~/.local/share/marvel-heroes-forever-rust/`.
See `FINDINGS.md` for measured evidence and `docs/PROVENANCE.md` for sources.

Build the asset tools with `nice -n 19 cargo build --offline -j2` (Rust 1.88+).
Their default build uses only the standard library. Add `--features renderer`
for the optional wgpu Vulkan image renderer. No server connection is made.

The default preset is Forever; Original is one switch away. Only completed
upgrades can be exposed. No graphics upgrades are implemented yet.

Examples (use your private pin paths):

```sh
mhf status --preset original
mhf sip inspect /private/path/Calligraphy.sip
mhf sip list /private/path/Calligraphy.sip .directory
mhf sip verify /private/path/mu_cdata.sip
mhf calligraphy inspect /private/path/Calligraphy.sip Calligraphy/Type.directory
mhf calligraphy verify /private/path/Calligraphy.sip
```

`calligraphy verify` checks supported metadata members and reports the remaining
prototype bodies as skipped. The tools never extract members to filesystem paths.
A nonzero error exit is 2. Source hashes establish file identity separately from
successful decompression or parsing.

Private distribution preservation helpers are in `tools/`. Run the read-only
inventory on the source install, copy only that manifest's selected files, then
run `verify_pin.py` against the private copy. `package_build.py` stages the built
original executable and documentation, with hashes; it does not publish a channel.

UE3 metadata inspection is also available for the observed version 868/3:

```sh
mhf upk summary /private/path/MarvelFrontEndMap.upk
mhf upk inspect /private/path/MarvelFrontEnd_SF.upk
mhf upk exports /private/path/MarvelFrontEnd_SF.upk
```

This reads object tables and supported property/native payloads through bounded
LZO decoding. Mesh rendering and the playable frontend remain unimplemented.

Prototype bodies and supported asset properties now have separate commands:

```sh
mhf prototype verify /private/path/Calligraphy.sip
mhf prototype inspect /private/path/Calligraphy.sip 0xf8ffa9c3a5912250
mhf upk properties /private/path/MarvelFrontEnd_SF.upk marvel_login_dialog
mhf upk verify-objects /private/path/MarvelFrontEnd_SF.upk
```

The prototype catalogue offers lazy lookup by data ID or exact member name and
preserves typed serialized fields. Parent-value merging and runtime classes are
not implemented. UPK property parsing reports unsupported class preambles and
retains unknown array/structure payloads as spans.

Actual saved frontend movies and their image references can now be inspected:

```sh
mhf frontend verify /private/path/MarvelFrontEnd_SF.upk /private/path/MarvelGame.upk
mhf frontend inspect /private/path/MarvelGame.upk MarvelFrontEnd.frontend_manager
mhf texture inspect /private/path/MarvelFrontEnd_SF.upk frontendbg_2560x1440
mhf texture render /private/path/MarvelFrontEnd_SF.upk frontendbg_2560x1440 ~/.local/share/marvel-heroes-forever-rust/evidence/background.pam
```

`texture render` requires the renderer feature. It decodes mip zero, draws a
native-size quad, checks every readback byte and writes a lossless RGBA PAM file
inside the private asset root. It never overwrites an output. This displays one
saved image, not a complete login screen. Frontend image binding uses each
movie's explicit References plus qualified object paths; missing or ambiguous
dependencies remain visible. Script execution, font routing and live render
targets are still needed. See [frontend composition](docs/FRONTEND_COMPOSITION.md)
and [renderer evidence](docs/RENDERER.md).
