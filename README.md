# Marvel Heroes Forever

Original Rust reimplementation in development, targeting Marvel Heroes Omega
1.52.0.1700 and the owner's MHServerEmu service. This is currently asset tooling,
not a playable game or a verified replacement for the retail client.

Client files, extracted data and reverse-engineering evidence remain outside
this repository under `~/.local/share/marvel-heroes-forever-rust/`.
See `FINDINGS.md` for measured evidence and `docs/PROVENANCE.md` for sources.

Build locally with `nice -n 19 cargo build --offline -j2`. No external Rust
crate dependencies are currently required. No server connection is made by
the asset tools.

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

This reads object tables through bounded LZO decoding. Object payloads, textures,
meshes, Scaleform UI and rendering remain unimplemented.

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
