# Marvel Heroes Forever

Original Rust reimplementation in development, targeting Marvel Heroes Omega
1.52.0.1700 and the owner's MHServerEmu service. This is currently asset tooling,
not a playable game or a verified replacement for the retail client.

Client files, extracted data and reverse-engineering evidence remain outside
this repository under `~/.local/share/marvel-heroes-forever-rust/`.
See `FINDINGS.md` for measured evidence and `docs/PROVENANCE.md` for sources.

Build the asset tools with `nice -n 19 cargo build --offline -j2` (Rust 1.88+).
Their default build uses only the standard library. Add `--features renderer`
for optional wgpu Vulkan image/triangle rendering, or `--features window` for native
preview presentation. Required native libraries and verified runtime scope are
recorded in [window support](docs/WINDOW.md). No server connection is made.

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
prototype bodies as skipped. These SIP/Calligraphy commands never extract members
to filesystem paths.
A nonzero error exit is 2. Source hashes establish file identity separately from
successful decompression or parsing.

Private distribution preservation helpers are in `tools/`. Run the read-only
inventory on the source install, copy only that manifest's selected files, then
run `verify_pin.py` against the private copy. `package_build.py` stages the built
original executable and documentation, with hashes; it does not publish a channel.
It also accepts `--binary /absolute/path/to/mhf --profile dev` for an already
checked development build. Renderer packages require `--dependency-notices`
pointing to a complete collection from `tools/dependency_notices.py`.

UE3 metadata inspection is also available for the observed version 868/3:

```sh
mhf upk summary /private/path/MarvelFrontEndMap.upk
mhf upk inspect /private/path/MarvelFrontEnd_SF.upk
mhf upk exports /private/path/MarvelFrontEnd_SF.upk
```

This reads object tables and supported property/native payloads through bounded
LZO decoding. StaticMesh geometry can be read through the library; material/world
rendering and the playable frontend remain unimplemented.

Prototype bodies and supported asset properties now have separate commands:

```sh
mhf prototype verify /private/path/Calligraphy.sip
mhf prototype inspect /private/path/Calligraphy.sip 0xf8ffa9c3a5912250
mhf upk properties /private/path/MarvelFrontEnd_SF.upk marvel_login_dialog
mhf upk verify-objects /private/path/MarvelFrontEnd_SF.upk
```

The prototype catalogue offers lazy lookup by data ID or exact member name and
preserves typed serialized fields. The library's effective-value resolver retains
parent/default provenance and unresolved assignment contexts; runtime classes are
not implemented. UPK property parsing reports unsupported class preambles and
retains unknown array/structure payloads as spans.

Actual saved frontend movies and their image references can now be inspected:

```sh
mhf frontend verify /private/path/MarvelFrontEnd_SF.upk /private/path/MarvelGame.upk
mhf frontend inspect /private/path/MarvelGame.upk MarvelFrontEnd.frontend_manager
mhf texture inspect /private/path/MarvelFrontEnd_SF.upk frontendbg_2560x1440
mhf texture render /private/path/MarvelFrontEnd_SF.upk frontendbg_2560x1440 ~/.local/share/marvel-heroes-forever-rust/evidence/background.pam
mhf texture window /private/path/MarvelFrontEnd_SF.upk frontendbg_2560x1440
mhf frontend imports /private/path/MarvelFrontEnd_SF.upk /private/path/MarvelGame.upk /private/path/MarvelFonts_SF.upk --font-library MarvelFonts.fonts_en
```

`texture render` requires the renderer feature. It decodes mip zero, draws a
native-size quad, checks every readback byte and writes a lossless RGBA PAM file
inside the private asset root. It never overwrites an output. This displays one
saved image, not a complete login screen. Frontend image binding uses each
movie's explicit References plus qualified object paths; missing or ambiguous
dependencies remain visible. Full script initialization and live render targets
are still needed; font routes remain explicit caller selections. See [frontend composition](docs/FRONTEND_COMPOSITION.md)
and [renderer evidence](docs/RENDERER.md).

`frontend scene` takes the same package/library inputs plus a movie selector and
inspects the recovered static first-frame tree. Sprite nesting, display depth,
transforms, imported symbols and unexecuted action bindings remain explicit.
This is a structural snapshot; it is not the script-initialized game screen.

The static diagnostic can also be rendered or shown in a native window:

```sh
mhf frontend render-static /private/path/MarvelFrontEnd_SF.upk marvel_login_dialog ~/.local/share/marvel-heroes-forever-rust/evidence/login.pam /private/path/MarvelGame.upk /private/path/MarvelFonts_SF.upk --font-library MarvelFonts.fonts_en
mhf frontend window-static /private/path/MarvelFrontEnd_SF.upk marvel_login_dialog /private/path/MarvelGame.upk /private/path/MarvelFonts_SF.upk --font-library MarvelFonts.fonts_en --frames 2
mhf frontend init-probe /private/path/MarvelGame.upk MarvelFrontEnd.frontend_manager
```

Static rendering reports its explicit sampling/color policy and omitted behavior.
It does not execute scripts, implement effects or establish a working login screen.
Outputs stay private and are never overwritten. The initializer probe runs saved
DoInitAction records in a persistent bounded VM with rejecting native bindings;
it exits with an error at unsupported behavior. See
[initializer phases](docs/FRONTEND_BOOTSTRAP.md) for its exact scope.
