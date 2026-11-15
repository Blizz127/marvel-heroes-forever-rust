# Cargo dependency notices

`tools/dependency_notices.py` assembles notice texts for the selected normal and
build dependency graph in an existing Cargo metadata file. It runs offline,
uses Python's standard library, and copies source notices byte for byte. The
collector does not interpret licences or certify legal compliance. Its
`complete` field means that every selected dependency had a licence declaration
and at least one collected notice text, and that no collection check failed.

## Invocation

Generate metadata for the actual features and target of the build, then use a
new private output directory. The parent directory must already exist.

```sh
cargo metadata --offline --format-version 1 --features renderer \
  --filter-platform x86_64-unknown-linux-gnu > /absolute/private/metadata.json
python3 tools/dependency_notices.py \
  --metadata /absolute/private/metadata.json \
  --target x86_64-unknown-linux-gnu \
  --output /absolute/private/dependency-notices \
  --supplements /absolute/private/supplements/manifest.json
```

`--supplements` is optional. `--target` records the caller's filter declaration;
the collector does not reevaluate Cargo target expressions and cannot recover
the invocation from metadata. The metadata must be generated for the desired
build. Normal and build edges are traversed from `resolve.root`, or the default
workspace members for a virtual workspace. Development-only edges are excluded.
Build dependencies/procedural macros remain included conservatively; the graph
is not a linker trace and does not identify which code reached the executable.
The standard library, operating-system libraries and GPU drivers are outside
this Cargo metadata inventory.

Exit statuses: `0` is collection complete, `1` is a written manifest with package
issues, and `2` is an invalid input or a top-level I/O failure. Existing output
directories are refused without overwriting. An interrupted or failed collection
may leave a partial private directory; a consumer must require a parseable
`manifest.json` with `complete=true` and `issues=[]` before packaging it.

## Collected inputs and path handling

For each dependency the collector retains the Cargo package ID, name, exact
version, source, selected features, authors, repository and the unmodified
`license` and `license_file` declarations. It collects regular files matching
root `LICENSE*`, `COPYING*` and `NOTICE*`, case-insensitively, and the explicit
`license_file`. Matching directories are walked with bounded depth and count.
An explicit licence file must remain inside its crate. Paths with traversal,
symlink components, special files or unsafe archive-relative names are rejected.
Reads are size-bounded and reject files changed during the read. New output is
private, with a mode-0700 root and mode-0600 files.

`manifest.json` records each copied relative path, byte length and SHA-256, plus
its source path or supplement provenance. It also records metadata and package
Cargo manifest hashes. The program prints the final manifest's SHA-256; that
hash is not embedded in the manifest itself. A missing notice is an issue even
when the package declares a recognised SPDX expression. No copyright/NOTICE text is reconstructed. Official licence terms can be
provided as a separately identified supplement, never as an invented
crate-authored notice.

## Exact-revision supplements

Some published crate archives omit their repository licence files. A separate
source-recovery step may supply the actual texts from the exact source revision.
The supplement manifest has this shape:

```json
{
  "packages": [{
    "name": "example",
    "version": "1.2.3",
    "repository": "https://github.com/owner/project",
    "revision": "0123456789012345678901234567890123456789",
    "files": [{
      "path": "example/LICENSE-MIT",
      "url": "https://raw.githubusercontent.com/owner/project/0123456789012345678901234567890123456789/LICENSE-MIT",
      "sha256": "the exact file SHA-256 as 64 lower-case hexadecimal digits"
    }]
  }]
}
```

The optional `package_id` narrows a supplement to one Cargo package ID.
Files may additionally identify `kind` as `upstream-notice` (default),
`upstream-declaration`, `official-license-text`, or
`upstream-component-notice`. Upstream files may specify
`source_revision`, which must match the package revision. Official terms must
use `source_revision=null`; their source remains their own URL. A package with
`upstream_notice_absent=true` must include both its exact upstream declaration
and separately identified official terms. The output preserves that absence;
collection completeness does not invent a missing upstream notice.

A component notice may have a different exact revision when a saved upstream
Git tree binds it through a gitlink. Its `source_repository` and
`source_revision` remain separate from the crate repository/revision.
`revision_basis` supplies `kind="gitlink"`, the exact `parent_revision`,
component `path`, and the relative `evidence_path`, SHA-256 and primary
`evidence_url`. The collector checks the saved tree entry mode `160000`,
type `commit`, path and revision; the evidence URL must select the parent
revision. This preserves a dependency's bundled-source notice without
mislabeling it as a file from the parent Git revision.

File paths are relative to the supplement manifest's directory. The collector
checks their hashes, requires HTTPS provenance URLs without credentials, matches
the repository against Cargo metadata, and matches the exact 40-digit revision
against the installed `.cargo_vcs_info.json`. It copies all supplied texts,
retaining their URL/revision and the VCS metadata hash. It does not fetch URLs
or independently prove that bytes originated at a URL; the source-recovery
record supplies that observation. Missing registry texts remain explicit as
`registry_notice_absent=true` after a valid supplement fills the collection gap.

## Observed renderer dependency snapshot

**OBSERVED, 2026-11-15:** the project `0.1.0-dev.3` metadata generated for
`--features renderer --filter-platform x86_64-unknown-linux-gnu` contains
61 selected dependencies. The initial installed-only run collected 108 notice
files and correctly returned exit 1 for seven packages with no notice texts.
The table records declarations as supplied by installed Cargo metadata; an
expression is not interpreted as a selected licence choice.

| Crate | Version | Declared licence | Installed root notices |
| --- | --- | --- | --- |
| `arrayvec` | `0.7.8` | MIT OR Apache-2.0 | 2 files |
| `ash` | `0.38.0+1.3.281` | MIT OR Apache-2.0 | 2 files |
| `autocfg` | `1.5.1` | Apache-2.0 OR MIT | 2 files |
| `bit-set` | `0.8.0` | Apache-2.0 OR MIT | 2 files |
| `bit-vec` | `0.8.0` | Apache-2.0 OR MIT | 2 files |
| `bitflags` | `2.13.2` | MIT OR Apache-2.0 | 2 files |
| `bytemuck` | `1.25.2` | Zlib OR Apache-2.0 OR MIT | 3 files |
| `bytemuck_derive` | `1.12.1` | Zlib OR Apache-2.0 OR MIT | 3 files |
| `cfg-if` | `1.0.5` | MIT OR Apache-2.0 | 2 files |
| `cfg_aliases` | `0.2.2` | MIT | 2 files |
| `codespan-reporting` | `0.12.0` | Apache-2.0 | 1 files |
| `document-features` | `0.2.12` | MIT OR Apache-2.0 | 2 files |
| `equivalent` | `1.0.3` | Apache-2.0 OR MIT | 2 files |
| `foldhash` | `0.1.5` | Zlib | 1 files |
| `foldhash` | `0.2.0` | Zlib | 1 files |
| `gpu-alloc` | `0.6.2` | MIT OR Apache-2.0 | Absent; exact-revision supplement required |
| `gpu-alloc-types` | `0.3.1` | MIT OR Apache-2.0 | Absent; exact-revision supplement required |
| `gpu-descriptor` | `0.3.2` | MIT OR Apache-2.0 | Absent; exact-revision supplement required |
| `gpu-descriptor-types` | `0.2.0` | MIT OR Apache-2.0 | Absent; exact-revision supplement required |
| `half` | `2.7.1` | MIT OR Apache-2.0 | 2 files |
| `hashbrown` | `0.15.5` | MIT OR Apache-2.0 | 2 files |
| `hashbrown` | `0.16.1` | MIT OR Apache-2.0 | 2 files |
| `hashbrown` | `0.17.1` | MIT OR Apache-2.0 | 2 files |
| `hexf-parse` | `0.2.1` | CC0-1.0 | Absent; exact-revision supplement required |
| `indexmap` | `2.14.2` | Apache-2.0 OR MIT | 2 files |
| `libc` | `0.2.191` | MIT OR Apache-2.0 | 2 files |
| `libloading` | `0.8.9` | ISC | 1 files |
| `libm` | `0.2.16` | MIT | 1 files |
| `litrs` | `1.0.1` | MIT OR Apache-2.0 | 2 files |
| `lock_api` | `0.4.14` | MIT OR Apache-2.0 | 2 files |
| `log` | `0.4.35` | MIT OR Apache-2.0 | 2 files |
| `naga` | `27.0.3` | MIT OR Apache-2.0 | 2 files |
| `num-traits` | `0.2.19` | MIT OR Apache-2.0 | 2 files |
| `once_cell` | `1.21.4` | MIT OR Apache-2.0 | 2 files |
| `ordered-float` | `5.3.0` | MIT | 1 files |
| `parking_lot` | `0.12.5` | MIT OR Apache-2.0 | 2 files |
| `parking_lot_core` | `0.9.12` | MIT OR Apache-2.0 | 2 files |
| `pollster` | `0.4.0` | Apache-2.0/MIT | 2 files |
| `proc-macro2` | `1.0.107` | MIT OR Apache-2.0 | 2 files |
| `profiling` | `1.0.18` | MIT OR Apache-2.0 | Absent; exact-revision supplement required |
| `quote` | `1.0.47` | MIT OR Apache-2.0 | 2 files |
| `raw-window-handle` | `0.6.2` | MIT OR Apache-2.0 OR Zlib | 3 files |
| `renderdoc-sys` | `1.1.0` | MIT OR Apache-2.0 | 2 files |
| `rustc-hash` | `1.1.0` | Apache-2.0/MIT | 2 files |
| `scopeguard` | `1.2.0` | MIT OR Apache-2.0 | 2 files |
| `smallvec` | `1.16.5` | MIT OR Apache-2.0 | 2 files |
| `spirv` | `0.3.0+sdk-1.3.268.0` | Apache-2.0 | Absent; exact-revision supplement required |
| `static_assertions` | `1.1.0` | MIT OR Apache-2.0 | 2 files |
| `syn` | `2.0.119` | MIT OR Apache-2.0 | 2 files |
| `syn` | `3.0.10` | MIT OR Apache-2.0 | 2 files |
| `thiserror` | `2.0.21` | MIT OR Apache-2.0 | 2 files |
| `thiserror-impl` | `2.0.21` | MIT OR Apache-2.0 | 2 files |
| `unicode-ident` | `1.0.27` | (MIT OR Apache-2.0) AND Unicode-3.0 | 3 files |
| `unicode-width` | `0.2.3` | MIT OR Apache-2.0 | 2 files |
| `wgpu` | `27.0.1` | MIT OR Apache-2.0 | 2 files |
| `wgpu-core` | `27.0.3` | MIT OR Apache-2.0 | 2 files |
| `wgpu-core-deps-windows-linux-android` | `27.0.0` | MIT OR Apache-2.0 | 2 files |
| `wgpu-hal` | `27.0.4` | MIT OR Apache-2.0 | 2 files |
| `wgpu-types` | `27.0.1` | MIT OR Apache-2.0 | 2 files |
| `zerocopy` | `0.8.63` | BSD-2-Clause OR Apache-2.0 OR MIT | 3 files |
| `zerocopy-derive` | `0.8.63` | BSD-2-Clause OR Apache-2.0 OR MIT | 3 files |

## Verification and source provenance

**VERIFIED:** the actual installed-only collection completed with 61 dependency
records, 108 byte-preserving notices, seven explicit missing-text issues and
exit 1. A separate readback checked the size and SHA-256 of every copied notice.
Focused temporary fixtures checked normal/build reachability, dev-only exclusion,
case-insensitive filenames, preserved author metadata, refusal to overwrite an
existing output, traversal rejection and symlink rejection. Supplemental
fixtures also checked exact revision matching, separate official-term origin,
retained upstream absence, and rejection of wrong file hashes/revisions. All passed.
No full test suite, legal audit or network acquisition was run by this tool.

Private evidence:

- `~/.local/share/marvel-heroes-forever-rust/evidence/dev3-cargo-metadata.json`
- `~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notices-installed-smoke/manifest.json`
- `~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notices-checks/result.json`
- `~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notices-supplement-checks/result.json`

All external files read by this collection are installed primary crate manifests
and notice texts under
`/home/blizz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`, plus the
selected crates' `.cargo_vcs_info.json` for supplemental revision checks.
The output manifest enumerates exact source paths, hashes and crate licence
declarations for this source ledger; the table above identifies each crate and
version. They were read solely to preserve attribution and licence notices,
not to copy implementation code. Supplements are separately supplied evidence,
with their primary URLs/revisions/hashes retained for every copied file.
No proprietary client binary, asset, capture or account file is an input.

Metadata SHA-256: `8e5bd9985cc80c58fe7490fe7120e9d255ba2d767c55d10763b0c2a8953de933`.

## Complete dev.3 collection

**VERIFIED, 2026-11-15:** with the supplied source-recovery manifest, the final
collector returned exit 0, `complete=true`, `issues=[]`, 61 packages and 127
texts: 108 installed texts and 19 supplemental texts. Readback verified every
file's length and SHA-256 and confirmed that no unlisted files were present.
Focused component fixtures accepted the exact saved gitlink and rejected a
different component revision even when the altered evidence file hash matched.

Ready directory:
`~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notices-dev3/`.
Manifest SHA-256:
`a148a70107840e7143ac45d64645ea5fe5095ed2439d09a718c8d8fa7b73c283`.
Readback receipt:
`~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notices-dev3-verification.json`.

The supplement source manifest is
`~/.local/share/marvel-heroes-forever-rust/evidence/dependency-notice-supplements/manifest.json`,
SHA-256 `627ad5a84bcd696c6bf2503fedad6e3760575c62c6ae909df3234e550b13bb42`.
It identifies the exact primary repository URLs and revisions for the four GPU
allocation/descriptor crates, profiling, hexf-parse and spirv. Its full package
observations, per-file provenance and exceptions survive in the output manifest.

The seven registry omissions remain explicit. The hexf source had no standalone
upstream notice: its exact CC0 declaration and the Creative Commons official
terms are recorded separately, with the standard's own repository/revision.
The spirv supplement also conservatively retains two Khronos notices from the
SPIRV-Headers revision named by the upstream gitlink. One supplied notice is the
copyright string array from the grammar JSON joined with LF; the declared
transformation, original source SHA-256 and Git blob identity remain attached.
This is explicit attribution collection, not a finding that a further legal
obligation exists or a legal-compliance certification.
