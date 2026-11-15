# Original saved-frontend reader

`src/frontend.rs` connects the existing original package, property, GFX and
texture readers. The source observations and licences are recorded in
[frontend composition](FRONTEND_COMPOSITION.md) and [GFX format](GFX_FORMAT.md).
No outside implementation was copied and no extra runtime dependency is used.

`AssetSet` owns explicitly supplied package archives. `read_movie(AssetLocation)`
requires a SwfMovie export with one non-indexed RawData byte-array property;
References, when present, must be a single correctly sized signed object-index
array. An absent References property remains distinct from an explicitly empty
array because class defaults have not been applied. Duplicate singleton
properties and invalid reference ranges are errors.

`bind_images` retains each local movie's external bitmap ID and logical image
dimensions. Names are compared case-insensitively within its declared Texture2D
references. A nonempty export name is preferred; otherwise the filename stem is
an identifier only. The observed dot-to-underscore fallback is explicitly marked
inferred. It is applied only when the exact name has no referenced candidate.
No file path is opened from either image name.

Positive references resolve directly in the movie's package. Negative references
resolve against full serialized outer paths and Texture2D class in supplied
archives. A cooked filename is never substituted for an object outer. Outer
chains reject cycles, out-of-range indices, excessive depth and ambiguous path
components. Multiple candidates remain ambiguous instead of selecting the first.
Missing dependencies remain structured results. Archive count is bounded to 256.

`read_texture` preserves the existing decoder's storage, color metadata and
unsupported-format checks. Binding does not decode every image or update a live
render target. The image's logical GFX dimensions remain separate from the
physical Texture2D dimensions. A frontend scene must use that distinction when
mapping bitmap fills.

The CLI exposes `frontend inspect`, `frontend verify`, `texture inspect` and,
with the optional renderer, `texture render`. The latter checks raw input versus
GPU readback and writes a private RGBA PAM artifact; it does not claim the retail
display color pipeline. No scripts, URLs, server messages or native prompts are
executed by these commands.

Focused synthetic checks exercise exact signed reference-array lengths and
external identifiers. Actual-package binding verification and build receipts
are retained privately, and successful structural binding is not runtime or
visual parity.

**VERIFIED (2026-11-15):** All 716 bindings from the 20 frontend movies match the
independent association inventory. This includes exact source reference indices,
677 local texture export indices and all 39 cross-package import locations/full
paths. Supplying no dependency preserves 39 missing imports; supplying duplicate
dependency archives preserves 39 ambiguous imports. Cyclic/out-of-range outers
are rejected. The startup manager separately resolves all 11 image records.

Private evidence: `evidence/frontend-binding-validation.json`, the exact original
harness/source and TSV beside it, and `evidence/dev3-integration-smoke.json`.
The GPU CLI check produced 14,745,600 bytes identical to its decoded 2560×1440
source. The private PAM was losslessly wrapped in PNG and visually inspected as
the SHIELD-pattern background; it is not a login screenshot or a composed UI.
