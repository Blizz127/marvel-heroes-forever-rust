# Saved class-default provenance preparation

`frontend::class_defaults::prepare` loads selected class records, follows their
bounded parent chains and retains each linked saved default separately. It
reads explicit CDO tags only for the four established GFx saved-file schemas.
It does not calculate effective values, apply configuration, execute scripts,
instantiate objects or combine the action and movie-player defaults.

The diagnostic command `mhf upk class-defaults <package.upk> <logical-package-name>
<1-based-class-index>` prepares one selected class and its local parents. The
logical name is explicit, and external imports stay unresolved. Output lists
saved property counts and unresolved chain endpoints without applying defaults.

## Explicit inputs and retained identity

```rust
prepare(&assets, &selected_classes, &package_names, &import_bindings, limits)?
```

`AssetLocation` selects an exact archive index and export index. `PackageName`
declares the archive's logical namespace; it is not inferred from a filename.
`ImportBinding` explicitly maps a source archive/import index to a target
archive/export index. All indices are zero-based. Negative package object
references remain signed one-based values in `Link.reference`.

Bindings must agree with the complete source import object path and declared
class identity. The target's qualified path uses its caller-declared package
namespace and serialized outer chain. Duplicate namespaces or duplicate source
bindings are rejected. Names are compared with the existing project's ASCII
case folding; original spelling remains in the records. There is no short-name
search, suffix matching, automatic package scan or filesystem/network import.
An available archive does not resolve an import without an explicit binding.

The result retains the AssetSet's shared identity token. `matches_assets`
accepts that same archive-index namespace after a move; reopening identical
files produces another namespace and fails the identity check. Selected roots
retain their order. Each chain lists the selected class first, then successive
parents; this is provenance order, not a default-application schedule. Shared
ancestors are read once into the record map.

Each class record preserves its `ClassObject`, original serialized bytes and
qualified identity. Its superclass and terminal default links retain exact
references, locations, source field spans and logical export base. A default's
archetype link explicitly identifies export-table metadata; no fictitious
payload offset is assigned. Imported and unbound links remain distinguishable
from null or local links.

For a locally or explicitly resolved default, the target's class must resolve
to the exact selected class location. Names alone cannot establish this join.
The superclass chain and CDO archetype links remain independent provenance:
the reader never substitutes a parent default or applies an archetype.

`SavedDefault` preserves its complete tagged list, raw bytes, NetIndex and any
uninterpreted trailing bytes. `tag_origin(index)` identifies the declaring class,
source CDO, logical export base, tag, payload and encoded value spans. Boolean
origins point to the inline tag byte even though their payload is empty.
Duplicate/indexed tags, arrays, structs and unknown property types retain the
existing tagged reader's representation. Object-valued properties remain raw
saved references; their resources are not eagerly loaded.

## Narrow CDO file-profile admission

**OBSERVED saved-file grammar, corroborated by VERIFIED static native routing:**
the admitted profile is version 868/licensee 3, ordinary little-endian package
input, the exact class/super/archetype identities below, raw CDO flags
`0x0007000400000200`, and NetIndex -1 before the tagged list. This is exposed as
`DefaultPreamble::ObservedGfx8683File`, not an assertion about a live archive.

| Class | Required declared superclass | Required CDO archetype |
| --- | --- | --- |
| GFxUI.GFxAction_OpenMovie | Engine.SequenceAction | Engine.Default__SequenceAction |
| GFxUI.GFxMoviePlayer | Core.Object | Core.Default__Object |
| MarvelGame.MarvelGFxMoviePlayer | GFxUI.GFxMoviePlayer | GFxUI.Default__GFxMoviePlayer |
| MarvelGame.MarvelGFxFrontEnd | MarvelGame.MarvelGFxMoviePlayer | MarvelGame.Default__MarvelGFxMoviePlayer |

Classes must be package-root class exports. A valid imported superclass or
archetype identity can be inspected without loading its target; its unresolved
resource state is preserved. CDO selection itself always uses the parsed
terminal class-default reference and exact target class identity, never a
Default__ name-prefix scan.

Native class builders and constructor tables establish the four identities.
GFxAction_OpenMovie's table `0x1421c1e50`, slot +0x68, points to UObject
serializer `0x1407a7720`. GFxMoviePlayer, MarvelGFxMoviePlayer and FrontEnd
tables (`0x1421c0320`, `0x14228ab30`, `0x14228b110`) point to `0x1412c0bf0`,
which is exactly a five-byte tail jump to that same serializer. There are no
class-specific prefix operations in that override. All four optional-prefix
predicate slots +0x168 point to the proved false leaf `0x1405ac2d0`; the
observed flags clear the native state-frame mask.

The common serializer's four-byte NetIndex branch requires archive +0x14
nonzero, +0x1c/+0x20 zero and +0x70 bit0x1000 clear. The retained ordinary-loader
study establishes archive A=L+0x1c8, constructor `0x14075a490` zeroing those
mode fields and endian field +0x4c, followed by fresh backing initialization
`0x140762900` setting A+0x14=1 at `0x1407635a2`. The ordinary package magic
does not take the swapped-endian branch. Original CDO bytes then corroborate
NetIndex -1 and a complete tagged stream starting at offset 4.

These are conditional native checkpoints and a narrow file-format profile.
They do not prove that every later runtime callback preserves archive modes or
cursor state. The mode with +0x70 bit0x1000 set omits NetIndex. This module
therefore does not claim a universal UObject/CDO preamble or runtime serializer.
No original handler or native game execution was needed for this admission.

Unsupported schemas, archetypes, state-frame flags, other object flags or
NetIndex values produce `DefaultBody::Unresolved` with a typed issue and the
retained bytes. Missing defaults/imports or unresolved class identities also
remain typed. An unsupported class payload remains a class record with its
reason and bytes. Malformed fields, invalid references or a bound default with
the wrong exact class return an error. Unsupported property-tag layouts are
retained as unresolved; malformed tag data is rejected.

Parent chains terminate explicitly at null, missing import, unsupported class
or a detected cycle. A cycle is not silently removed or interpreted as a valid
inheritance hierarchy. Neither a successfully prepared graph nor a fully
parsed CDO establishes default application or actual construction.

## Bounds and actual inputs

Only selected classes, their parent classes and their resolved CDO payloads are
read. Defaults: 128 classes/selected roots, depth 32, 4,096 import bindings,
256 namespace declarations, outer depth 128, 4 MiB charged identity data,
8 MiB serialized payloads, 1 MiB per CDO and 65,536 explicit properties in
aggregate. Class parsing also retains `upk::class::Limits` and the package
reader's hard export cap. Payload sizes are charged before reads. Property
parsing retains its existing per-list bound before aggregate accounting.

The actual intake uses archive 0 GFxUI and archive 1 MarvelGame. The only two
cross-package bindings needed for the available player ancestry are:

| Source import | Target export |
| --- | --- |
| MarvelGame / 329, GFxUI.GFxMoviePlayer | GFxUI / 36 |
| MarvelGame / 726, GFxUI.Default__GFxMoviePlayer | GFxUI / 37 |

The four class/default pairs are GFxUI 36→37 and 145→146, and MarvelGame
8538→8539 and 8764→8765. Their explicit tag counts are 7, 5, 5 and 7:
24 top-level tags in total, including nine Booleans. Four Core/Engine
superclass/archetype edges remain unresolved in the two-archive intake.

The FrontEnd player ancestry has 19 property occurrences, including base/derived
HUD-off and Priority tags with distinct origins. They remain 19 occurrences,
not an effective 17-name map. OpenMovie belongs to the SequenceAction chain;
its MoviePlayerClass property does not make it a movie-player subclass.

## Evidence and provenance

Four focused synthetic tests pass with a warning-free library test compile.
They verify Boolean tag origins with an empty payload, missing parents despite
an available archive, exact explicit binding, duplicate/wrong bindings,
namespace identity across moves versus reopen, cycles, unsupported CDO flags
and the intake/byte/property/identity budgets. Fixtures are original generated
packages with no retail bytes. The other 546 compiled tests were filtered out.
The focused receipt is `check-2/receipt.json`, SHA256
`601c349748f42b5894ba21ada9fd2b737af5ade2fb7f7fad98debe0f05390fb8`.

The original-package comparison also passes: four classes, four CDOs, all
24 tag origins, 24 graph links across bound/unbound intake cases, eight parent
chains and eight exact body hashes (10,256 bytes) match the independent oracle.
Same-namespace identity succeeds, a reopened identical AssetSet fails that
identity check, and a wrong import binding returns InvalidData. Both packages,
the harness and 103 original/snapshot source files remain unchanged. The
oracle lacks the import class-package field, so that field is observed and
checked for product consistency rather than independently compared here.

The composition receipt is `retail-check/receipt.json`, SHA256
`b0a4ef72b70053f1c9565b42040fa0360af2ca52df20d1d8a22bccd9131b8ffb`;
`retail-check/output/comparison.json` has SHA256
`995c3197d055c54e99436e077d7d9625a5b23b7cc7eeb2ed549d38b85d82fa14`.
The exact tested module SHA256 is
`a21ae4f10080ad1621b59ffdd42204d23c46a4eb9dcdd4c820d1e3e0226adf38`.

Private evidence is under
`~/.local/share/marvel-heroes-forever-rust/evidence/class-defaults/`:

- `retail-oracle/oracle.json`, SHA256
  `3251e90c66fe40b4e33810b1c4207307fc0a40a5f3207a9f1d76abfb9343c574`,
  independently records the four class/default nodes, 24 tag origins, raw
  references and expected unresolved edges from retained package evidence.
- `preamble/receipt.json`, SHA256
  `90a00f9dba9d0b674b6c9cab47870bc1a45ed95e69b184034ab9e81a2f6445b5`,
  binds exact native identities, four payload hashes and the new bounded REA
  record `ev_cb5cf9e7d372604f81e84569bd61b0127b58765911422a283682ec3609ce9acf`.
  Its disassembled thunk displacement, opcode and jump field agree; inconsistent
  Rizin ESIL is excluded. The one-record investigation was exported/reset.
- `preamble/archive-addendum-receipt.json`, SHA256
  `e9389de6d7a8a7fe43dd9ea5bfb794e4e25fcb4b1bc8c88aeafa201494b01ba3`,
  rechecks the 899-byte constructor and 3,538-byte fresh-loader stage from
  `scale9-texture-load/rea-archive-identity-loading.json`, record
  `ev_9e0083a65f2853e77fe0d86f918d23a86b86e19777744c764828719c817fde5d`.

Sources are the owner's proprietary build 1.52.0.1700 executable (SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`),
GFxUI.upk (`dca575a8e84f16dd1cec7e2fd4afd4c98bf7070a5a853ebb84bcfd57f3951199`)
and MarvelGame.upk (`d9c98071ecbbf5d2b35936fa52283b804c45acccaa16faf232561c796ee9cdc6`),
plus original project code and retained evidence. Raw exports and disassembly
remain private. Reused original studies are UPK_CLASS, FRONTEND_MOVIE_DEFAULTS,
FRONTEND_INITIAL_INSTANCE, FRONTEND_EXTERNAL_RECEIVER and ENTRY_PACKAGE_OBJECTS;
their external-source provenance is retained, with no external implementation
copied or newly read for this adapter.

The REA skill and native guidance are the installed MIT toolkit instructions;
Rizin LGPL-3.0-only tooling through REA produced static evidence. Rust compiler/standard
library (MIT/Apache-2.0) and Python standard library (PSF) support original code,
fixtures and comparisons. No dependency was added. No accounts, configuration,
native client, GPU, server, network or source-install mutation was involved.
