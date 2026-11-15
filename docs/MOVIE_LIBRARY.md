# Saved movie symbol library

`frontend::library` is original dependency-free linkage code over this project's
package and GFX readers. It opens only caller-supplied package archives and never
opens a URL from a movie. The source observations are registered in
[frontend composition](FRONTEND_COMPOSITION.md), [font resources](FRONTEND_FONTS.md)
and [native REA findings](REA_NATIVE_ALTERNATIVE.md). No reference implementation
was read or copied for this bridge.

`MovieLibrary::load` selects SwfMovie exports from explicit archive indices and
keeps stable movie IDs, full UE3 object paths, exact export locations and symbol
tables. Positive saved References retain their exact source-package identity.
Imported references use qualified object paths. Missing or duplicate candidates
remain structured resolution results. ExportAssets symbols are case-sensitive;
only the observed UE3 object-path comparisons fold ASCII case.

Saved relative imports such as `../MarvelHUD/!UISource.swf` are normalized as
logical identifiers, constrained by the source movie's SwfMovie References. The
actual cinematics panel also uses the extensionless identifier `marvel_assets`;
that spelling is admitted under the same reference constraint. Scheme URLs,
queries, fragments, absolute paths and logical-root escapes are unsupported.
No normalized value is used as a filesystem path.

An explicit `MovieAlias` can supply an application-level route separately from
saved References. The CLI's `--font-library MarvelFonts.fonts_en` selects an
English inspection resource for `MarvelFonts.gfxfontlib`; it is not automatic
locale detection. Russian and English export several identical symbol names,
so a global first-symbol match would be ambiguous and is never used. Native REA
evidence establishes locale-dependent selection and named-library construction,
while complete relative-import resolution and configuration-to-selector mapping
remain unverified.

`ImportResolver` integration checks that each scene request exactly matches its
saved source movie, local dictionary ID, URL and symbol. It returns only the
previously resolved target movie/character identity. Import cycles and missing
targets remain the scene builder's explicit structural conditions.

Default loading budgets are 256 movies, 256 MiB of serialized movie exports,
250,000 total import/export symbol records and 256 explicit aliases. The loader
checks budgets before admitting each export or symbol group. It rejects duplicate
selected archive indices and duplicate alias keys. It does not apply class
defaults, execute initializers, fetch dependencies, or select fallback fonts.

## Observations and validation

**VERIFIED:** The first integrated actual-file check loaded 26 movies from the
frontend, MarvelGame and locale-font packages. Without a font route, 40/71 imported
symbols resolved. With explicit English or Russian selection, 68/71 resolved,
including all 16 login and 13 character-select imports. That check identified
the extensionless cinematics import, which led to the additional supported
identifier form described above. Its subsequent verification is recorded in the
private follow-up receipts rather than assumed from the patch.

The other two unresolved symbols are `$Strat2Regular` in the gameplay/video
options movies. Neither selected locale's saved ExportAssets nor the currently
recovered native five-alias mapping supplies that symbol. No substitute is
invented. Successful login symbol linkage still does not execute its UI scripts
or prove native font/fallback behavior.

Private receipts: `evidence/frontend-import-library-smoke.json` and subsequent
scene/library checks. The focused logical-identifier test covers parent-relative,
same-package and font-library names, extensionless imports and rejected external
URL/escape forms. The renderer's two imported bitmap fills resolve through the
existing `MarvelHUD.!UISource` movie inside the frontend package, illustrating
why an outer package name must not be guessed from a cooked filename.

**VERIFIED (follow-up):** The extensionless reference resolves its real
MarvelScrollBar symbol, bringing the explicit English library to **69/71**.
Only the two `$Strat2Regular` symbols remain missing. Integrated scene inspection
now follows this same resolver; exact output and binary hashes are private in
`evidence/frontend-library-scene-smoke.json`.
