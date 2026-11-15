# Positive tolerance on the bounded rectangle path

`gfx::native_scale9_tessellation::tessellate_rectangle` now admits an explicitly
supplied finite nonnegative relative tolerance when the native Y-coalescing
pass preserves the exact two source rows. All other restrictions in
[SCALE9_TESSELLATION.md](SCALE9_TESSELLATION.md) remain: fresh mode 1, one exact
five-point rectangle in the recovered order, styles `[0,1]`, resource style 1,
non-AA width, disabled generator postprocess and admitted finite arithmetic.

This extends the earlier zero-tolerance subset. It does not select a renderer
default, connect a native provider or enable rendering. The caller still supplies
the actual native-role option. The constructor-bit value used in a diagnostic
witness is not adopted as current runtime state.

## Source Y admission

**OBSERVED native instructions, VERIFIED bytes:** setter `1400fed20` copies the
supplied option to primary tessellator `T+0x1c` and sets `T+0x20` according to
the ordered comparison `tolerance > 0`. The separate copy to `T+0x3b4` does
not select a new branch on the retained non-AA, no-postprocess rectangle route.

Complete `140111130` stably sorts the five source indices by Y. The admitted
rectangle order becomes `[0,1,4,2,3]`. Starting with previous Y at binary32
bits `0xf149f2ca`, instructions `140111249..14011129e` compute:

1. Binary32 `distance = y - previous_y`.
2. Exact absolute Y through double conversion/masking and conversion back.
3. Binary32 `threshold = abs(y) * tolerance`.
4. Retain this index and update previous Y only if `distance > threshold`.
   Otherwise overwrite source Y with previous Y.

Equality therefore merges a row; it does not retain it. Original Rust keeps
the existing operation order and checks both intermediate results. If native
coalescing would change any original Y bits, this bounded immutable-source
adapter returns `UnsupportedBranch`. It also requires exactly two retained
indices, `[0,2]`. It does not replace the resulting degenerate or modified path
with the original rectangle.

Negative/nonfinite tolerances are unsupported profiles. Nonzero subnormal
inputs and subnormal-sensitive intermediates remain explicit errors because
the native floating-point environment has not been established. Overflow also
remains an error. Signed zero is admitted as numerical zero. Existing source,
coordinate-sentinel, slope and orientation checks remain in force.

## Why positive tolerance adds no downstream operation here

**INFERRED conditional execution from complete retained bodies:** once both Y
rows and endpoint positions survive unchanged, the existing two vertical
one-edge chains remain strictly ordered at both ends. The first scan is
`low→high`; the terminal scan is `high→high`.

| Native body | Relevant branch and selected outcome |
| --- | --- |
| `140103f60` | Adjacent chain end X stays ordered at `14010458c..140104595`; intersection insertion is skipped. Both scans retain zero intersection events. The count test at `1401047a7..1401047e2` skips the entire merge block unless there are more than one, so the `T+0x20` test and tolerance read at `140104801` / `140104855` are not reached. |
| `1400ffd20` | The first pass creates two new boundaries; the finished terminal chains create none. Style assignment and finished flags follow the previously proved route. |
| `14010ec80` | Boundary counts are old 0/new 2, then old 2/new 0. Both skip simultaneous old/new processing and its inlined interpolation read at `14010ef89`. Selected calls to `1401048a0` are `14010f0cf` and `14010f10b`, both with `R9b=1`. |
| `1401048a0` | Low-Y source-start equality and terminal finished-endpoint equality choose exact endpoint paths. Each compares source X strictly against `T+0x210`, at `1401049ae` or `14010493e`. The only tolerance read, `140104a49`, belongs to the interpolation alternative and is skipped. |
| Non-AA generation/emission | The same four positions and linked tags feed `1401115b0 → 14010d680 → 14010cbf0 → 14010c8e0 → 14010a110`. Those selected paths do not inspect the relative tolerance or its positive flag. The explicit false postprocess input still excludes its separate callback path. |

There is **no X epsilon spacing requirement for these exact endpoints**. A
one-ULP-wide rectangle can retain distinct endpoints even when its width is
less than `abs(Y) * tolerance`. Adding that interpolation criterion here would
change the selected native endpoint behavior.

The generated order, linked tags, mesh/style records and triangle order remain
the earlier conditional result: four vertices and triangles `[0,1,2]`,
`[3,2,1]`. Arbitrary contours, changed Y rows, interpolation, intersections,
AA, resource choice, allocator failure, UVs and final GPU formats remain outside
the implementation. Successful valid native allocation and the original state
conditions of the earlier proof remain assumptions of the static comparison.

## Focused validation

**VERIFIED:** the frozen isolated module compiled and all nine rectangle tests
passed. Four new tests cover supplied positive values, translated coordinates,
the strict Y threshold on positive and negative rows, immutable rejection,
one-ULP X endpoint separation, subnormal-sensitive arithmetic and overflow.
The isolated wrapper emitted one unused-method warning for the separately
implemented fill helper; no compilation or test failure occurred. A combined
product build belongs to the integrator and was not run in this lane.

Ten instruction-derived scalar witnesses were frozen before the product edit.
Seven admitted cases matched all 13 output groups each, including exact position
bits, triangle order, descriptors and packed indices. Three witnesses rejected
the native Y-merge branch. One admitted case reuses the independently decoded
saved Shape 72 coordinates under an explicitly supplied identity transform,
resource-present style 1, positive tolerance bits `0x3727c5ac` and zero AA width.
These are supplied diagnostic inputs, not an observed native invocation.

Private evidence root:
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-tolerance/`.

| Artifact | SHA-256 |
| --- | --- |
| `native-verification.json` | `64ad1c0817427ae217d484fa3e2e74e13920506039f89099b25a4d11c4920210` |
| `branch-audit/receipt.json` | `d81a20d6a1fd73104299179bafcd8a4ac0f5ec98128fa37a5a91e69d64b2d248` |
| `oracle.json` | `41f115dfd474830ecfeb1478e371b9ac2d3e86f2e9669ab8720b688b67fc7d98` |
| `checks/source-freeze.json` | `02fa7fada9c6bc02a461509b250a60ac3f33078344042eecdf1434cabe95eeb4` |
| `checks/receipt.json` | `9ae371f2fe9f71e623a2c64be442d970bea74296b6cbe99ecbe1153e2c16a872` |
| `checks/oracle-comparison.json` | `3d43a5ba03dc60d11adc4a38e8c40e88788bb6b69249e0d5315b5e7af002ff21` |

The parent byte audit matches 3,049 instruction records / 13,241 bytes across
four retained REA records. A separate downstream audit checks 4,753 records /
20,758 bytes across seven retained records; those totals overlap and are not
added. The `fed20` selection also contains adjacent `fed60` and padding, not
one 128-byte function. No new native producer call, target execution or service
request occurred. Earlier canonical records were already exported/reset.

## Evidence and provenance

All implementation and checking code is original. Native instructions and
saved proprietary data remain in the private evidence directory.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's private `MarvelHeroesOmega.exe`, build `1.52.0.1700`, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner-installed artifact; interoperability study | Exact native byte comparison and conditional control-flow/operation-order evidence |
| Retained `scale9-geometry/native/` and `scale9-tessellation/native/` REA records, individually hashed in `native-verification.json` and `branch-audit/verification.json` | Private owner-binary evidence; REA 6.9.0 / Rizin tooling previously registered | Setter, Y pass, scan, endpoint, boundary and emission branches |
| Project `SCALE9_TESSELLATION.md`, original Scale9 modules, private scan-analysis receipt and saved Shape 72 source oracle | Original project investigation; underlying assets proprietary | Existing bounded profile and exact saved-coordinate witness, not copied external implementation |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md`, `references/evidence-workflows.md`, selected `references/native-and-artifacts.md` guidance | Previously registered REA MIT tooling documentation | Reuse of complete retained Evidence, source identity and coverage handling |
| Python standard library and installed Rust compiler | PSF tooling / Rust compiler MIT or Apache-2.0 | Original byte/oracle checking scripts and isolated focused compilation |

Principal retained Evidence IDs are
`ev_66aafda720ebe6405b090d7fdce305935eae9a3dbffedae3e2bc0f8e1dc11d88`,
`ev_a729e58b24d94bd6570114d3ab7b936ec3f1bfadd8857d4a0dd79bdb8b2eaff1`,
`ev_97d601ca13cafbc89edf3e132931dbbcdb3261c1ce1c279310ae5c4b164f5d1d`
and `ev_88d6af9e2b1d0432d658714d3fbac0308e113e5a8b305a492e16d24e1ffbbbd9`.
The downstream audit records its additional selected sources and limits.
