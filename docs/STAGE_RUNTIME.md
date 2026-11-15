# AS2 Stage native contract

**VERIFIED, bounded static evidence:** Stage is a lazily created native callable
associated with one runtime context and its MovieImpl. It is not an ordinary
object containing a window width and height. Its dimension properties, extended
rectangle properties and listener methods have different receiver/state paths.

Target: private build 1.52.0.1700 `MarvelHeroesOmega.exe`, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
No native client, native AS2 method or Rust Stage implementation was executed in
this study. This document is an implementation contract, not a parity result.

## Lazy identity and fresh object graph

The bootstrap and generic resolver are established in
[AS2_GLOBAL_BASELINE.md](AS2_GLOBAL_BASELINE.md) and
[EXTERNAL_INTERFACE_BINDING.md](EXTERNAL_INTERFACE_BINDING.md). Stage's wrapper
is `0x140201c00`, factory `0x1402014a0`, builtin/prototype-cache index 18, and
builtin-name offset `+0x1c8`. Bootstrap stores a tag-11 name token on the global
object; its final member flags are 1. Registration does not invoke the factory.

Receiver/callee conversion resolves the token through the **current Env's
context at `Env+0xe8`**, using that context's factory registry at `+0x20`.
Successful repeated resolution returns the cached callable. A retained token
does not carry a frozen runtime context. Raw member copying alone does not
materialize it; null factory results remain retryable.

On fresh successful allocation the factory creates:

- A `0x78`-byte callable through `0x140200dc0`, with primary table
  `0x1425171b0`, interface `0x142517100`, category 23. Its linked parent is the
  cached Function prototype, index 7. Its `+0x70` field retains
  `context[+0x28]`, the MovieImpl. Callback `0x1401894f0` clears its result and
  writes tag 1, **null**; this does not establish every `new Stage` path.
- A separate `0xa8`-byte prototype through `0x1401e6f80`, parented to cached
  Object prototype, index 1. Its final primary table is `0x1425164d0`, interface
  `0x142513ad0`, category 24, and metadata interface `+0x68` is
  `0x142515998`. The constructor reference and callable's `prototype` link are
  installed by the shared constructor/prototype machinery. The factory caches
  this prototype under index 18.

The factory publishes tag 8 on `context[+0x60].Stage`, submitting flags 0.
**The existing bootstrap property's flags 1 survive.** Global interface slot
`+0x50` selects `0x1401e3b20`; its existing-entry branch copies only Value
tag/payload through `0x14018d9d0`, preserving the member flag byte at Value `+1`.
The supplied flags are used on the distinct insertion branch. Deleting or
altering Stage before resolution is a different starting condition.

The fresh explicit own-member ledger is:

| Object | Own name | Initial stored value / meaning | Flags |
| --- | --- | --- | --- |
| Stage callable | `__proto__` | Tag-10 metadata; linked parent is cached Function prototype | 3 |
| Stage callable | `addListener` | Native callable `0x1401a8e10` | 1 |
| Stage callable | `broadcastMessage` | Native callable `0x1401fc0c0` | 1 |
| Stage callable | `removeListener` | Native callable `0x1401a8e90` | 1 |
| Stage callable | `_listeners` | Fresh empty Array, parented to cached Array prototype | 1 |
| Stage callable | `width`, `height` | Tag-10 markers; special getter computes integers | 0 |
| Stage callable | `scaleMode`, `align` | Tag-10 markers; special getter computes strings | 0 |
| Stage callable | `showMenu` | Boolean true | 0 |
| Stage callable | `prototype` | Reference to the separate Stage prototype | 3 |
| Stage prototype | `__proto__` | Tag-10 metadata; linked parent is cached Object prototype | 3 |
| Stage prototype | `constructor` | Tag-10 metadata; getter resolves the stored Stage constructor | 3 |
| Stage prototype | `addListener`, `removeListener`, `broadcastMessage` | Separate native functions using `0x14018d3a0`; each clears result to undefined | 1 |

Thus there are 11 explicit callable own keys and five prototype own keys.
`onResize`, `visibleRect`, `safeRect` and `originalRect` are not added by this
factory as own data properties. The three callable methods come from terminated
table `0x14250d870`; the prototype's distinct table is `0x14250eb30`.
Property flags, Value tags and function-reference flags are separate. This
inventory does not imply that all methods exposed by prototype inheritance are
implemented, or that `showMenu`'s downstream native effect has been recovered.

## Which movie supplies each property

Let `S` be the Stage object, `I=S+0x20` its interface, `E` the calling Env, and
`M=S[+0x70]` its retained MovieImpl. Existing root/context evidence is in
[REA_CLASS_REGISTRY.md](REA_CLASS_REGISTRY.md).

| Operation | State source |
| --- | --- |
| `width`, `height`, `scaleMode`, `align` reads | Retained `M=I[+0x50]` |
| `scaleMode`, `align` writes | Same retained M; conversions and name rules use calling E |
| Extended rectangle availability | Calling `E[+0xe8][+0x68] == 1` |
| Extended rectangle geometry | Calling target/root chain `E[+0xe0][+0x20][+0x10]` |
| Listener collection | Actual callable method's `this` interface, through raw `_listeners` lookup |

These routes can differ when a Stage reference crosses runtime contexts. A
definition's filename, static SWF bounds or current physical surface cannot
replace either native ownership path.

The ordinary Stage getter `0x1401bdde0` first handles the extension properties
when the caller's extensions state equals 1, then delegates to raw getter
`0x140197420`. Above movie version 6, property-name comparisons use exact
interned names; the older branches use the native normalized-name cache.
Fresh extensions state is separately established as zero in the global
baseline. Neither this study nor a saved script reference proves that state
was enabled for the running manager.

## Dimensions and rectangles

Raw getter `0x140197420` reads the four float32 endpoints at
`M+0xf0/+0xf4/+0xf8/+0xfc`. Width is the float32 subtraction `x1-x0`, multiplied
in float32 by the pinned constant `0.05`, then converted with `CVTTSS2SI` to a
signed integer Value, tag 4. Height uses `y1-y0` through the same conversion.
Do not replace this with double arithmetic, rounding to nearest, or direct
surface pixel dimensions. Exceptional floating values retain the native
instruction semantics; no synthetic edge-case execution is claimed here.

Raw setter `0x1401e43f0` recognizes width and height and returns true **without
writing them**. The constructor explicitly calls the base setter to install
their initial markers. Their flags 0 therefore do not make them ordinary
writable dimensions.

With extensions enabled, the normal getter implements:

| Property | Source on the caller-associated MovieImpl |
| --- | --- |
| `visibleRect` | Virtual `+0x90`, concrete `0x140062c40`: each effective endpoint at `+0xf0..+0xfc` multiplied by float32 0.05 |
| `safeRect` | Virtual `+0x98`, concrete `0x140062c90`: four floats at `+0x100..+0x10c`; if either ordered positive extent check fails, falls back to visibleRect |
| `originalRect` | Movie virtual `+0x08`, concrete `0x140054400`, returns `Movie[+0x48]`; that definition's virtual `+0x50` supplies the rectangle |

Helper `0x1401bdd00` creates/resolves a Rectangle through the caller's
`flash.geom` context namespace and builtin `Rectangle` name, converts the four
float endpoints to doubles, and uses `0x1401929c0` to submit `x=x0`, `y=y0`,
`width=x1-x0`, `height=y1-y0`, with flags 0. This is a fresh result construction
path, not a shared mutable rectangle alias. The public Rectangle constructor
lookup can be changed by script; arbitrary replacement/allocation failures are
outside the successful native Rectangle path. The concrete definition's
virtual `+0x50` body was not separately recovered here, so this document does
not substitute a guessed file rectangle for that call.

### Effective rectangle inputs

MovieImpl virtual `+0x60`, `0x14005d380`, accepts a 0x34-byte descriptor at
`Movie+0xa0` and calls calculator `0x14005ae60`. The calculator has distinct
inputs: signed integer origin/extent fields `+0xa8/+0xac/+0xb0/+0xb4`, float
factors `+0xcc` and `+0xd0`, mode/alignment `+0xe8/+0xec`, and definition size
reached through `Movie[+0x48][+0x38][+0x18][+0x40]`, subtracting float endpoints
at `+0x60/+0x64/+0x68/+0x6c`. It does not read descriptor `+0xa0/+0xa4`.
Their physical-buffer meaning is not independently proven here.

For ordinary finite valid geometry, the verified mode branches are:

- `noScale` derives extents from the descriptor extent and the two float
  factors, then uses alignment to position them relative to definition size.
  Centering truncates the offset toward zero to whole pixels before converting
  it back to twips. It is not floor rounding.
- `exactFit` uses effective endpoints `(0,0,definition_width,definition_height)`.
- `showAll` and `noBorder` choose different aspect-ratio comparison branches,
  preserving one definition extent and deriving the other from the descriptor
  aspect and horizontal factor. Their effective rectangle is centered in this
  calculator; the alignment enum is read only by its `noScale` branch.

Private `viewport-contract.json` preserves the formulas, inputs, alignment
table and arithmetic-order limits. Descriptor width times 20 is only an
algebraic simplification: the native code subtracts separately converted
origin/end values after 32-bit addition. Endpoints, fractional dimensions,
zero/null paths and float32 ordering must remain explicit in an implementation.
The subsequent matrix helper `0x1400588b0` consumes the effective rectangle and
descriptor extents separately. No physical surface/DPI policy is inferred.

## Scale mode, alignment and initialization

Stage setter `0x140201060` converts the supplied Value to a string using the
calling environment. It compares scale-mode strings through the native
normalized-name cache and selects these enums; unmatched strings select 1:

| Movie mode `+0xe8` | Getter string |
| --- | --- |
| 0 | `noScale` |
| 1, or other unrecognized enum | `showAll` |
| 2 | `exactFit` |
| 3 | `noBorder` |

It always calls retained Movie virtual `+0x70` (`0x140053ef0`), which writes
the mode and resubmits the current descriptor for recomputation. If the old
mode differs and the new mode is 0, Stage additionally calls its resize
broadcaster `0x1401f69b0` directly. Callback scheduling elsewhere is separate.

Alignment conversion passes through `0x1400978f0` / `0x140036660` and the
native character-mapping helper. The retained mapping tables verify ASCII
`t/l/b/r` become uppercase. The setter examines at most the first two
characters; remaining characters do not participate in its selection.

| Accepted first characters after normalization | Enum `+0xec` | Getter string |
| --- | --- | --- |
| Empty, C, or unrecognized first character | 0 | Empty string |
| T with second character other than L/R | 1 | `T` |
| B with second character other than L/R | 2 | `B` |
| L with second character other than T/B | 3 | `L` |
| R with second character other than T/B | 4 | `R` |
| TL or LT | 5 | `LT` |
| TR or RT | 6 | `TR` |
| BL or LB | 7 | `LB` |
| BR or RB | 8 | `RB` |

The setter calls retained Movie virtual `+0x80` (`0x140053150`), which writes
the alignment and resubmits the descriptor. The getter uses virtual `+0x88`
(`0x140062a50`); invalid enums also return the empty string.

**VERIFIED initialization store, not a loaded frontend default:** MovieImpl
constructor `0x140065080` writes **qword `[Movie+0xe8]=1`** at `0x140065765`.
Consequently its initial mode is 1 (`showAll`) and alignment is 0 (getter empty
string). It initially clears the effective rectangle and sets descriptor width
and height to 1 and factors `+0xcc/+0xd0` to 1. These constructor stores precede
definition installation, host viewport setup and script writes. They do not
justify a 1×1 Stage viewport, a zero-sized loaded manager, or `showAll` as the
manager's eventual setting.

## Listener methods and resize delivery

`0x1401dc5b0` allocates the fresh `_listeners` Array. The callable methods use
the supplied `this` interface; they are not hard-coded to a global Stage list.

- `addListener`, given at least one argument, converts the first argument to
  an interface using the calling Env, then calls `0x14019c2f0`. That helper
  raw-reads `_listeners`, requires an Array, compares interface identity in
  ascending order to suppress duplicates, and appends an interface-backed
  Value if absent. The public callback discards the helper's Boolean and
  returns **true**, including a duplicate or rejected null/non-object input.
- `removeListener`, given at least one argument, similarly converts it and
  calls `0x14019c480`. That helper scans the Array backward, removes one matching
  interface, and returns true; no matching usable list/interface returns false.
  The public callback returns that Boolean.
- `broadcastMessage`, given at least one argument, converts the first argument
  to the event-name string, passes the remaining arguments to `0x1401dc6a0`,
  and clears its result to undefined. The helper copies the listener Values
  into a separate Array through `0x14018f500`, then visits that copy in order.
  For each usable interface it raw-reads the named method, accepts tag 8 or
  resolves tag 11 with the calling Env, and submits the function with that
  listener as `this`. A listener-list mutation does not replace the copied
  iteration set; method lookup still occurs at visit time.

All three public callbacks return without writing their result when passed
zero arguments. This is separate from the caller's initial result state.
Allocation-failure and arbitrary custom `_listeners` behavior are not flattened
into success stubs. Array append/removal primitives reuse the existing Array
contract rather than establishing every sparse-array mutation here.

Resize helper `0x1401f69b0` broadcasts `onResize`. With caller extensions state
1, it constructs a Rectangle from the **caller-associated visible rectangle**
and passes one argument; otherwise it passes zero arguments. This helper does
not manufacture an own `onResize` property on Stage.

The viewport setter invokes AS2-root virtual `+0x98` when effective endpoints
change, and under additional mode-dependent descriptor-change conditions.
Concrete root handler `0x1402003e0` locates level zero and submits a queue entry
of kind 4/priority argument 4 carrying callback `0x1401f6c50`, if the target and
allocation exist. At callback execution it raw-reads global Stage from that
FnCall's Env and **explicitly skips tag 11**. Therefore a queued resize does
not create previously unresolved Stage. A usable resolved object goes to
`0x1401f69b0`. Full queue timing, coalescing and drain behavior remain in the
separate lifecycle study; this document does not equate queue insertion with
callback execution.

## Actual saved frontend requirements

**OBSERVED in the pinned manager movie:** 94 direct Stage references comprise
42 width reads, 35 height reads, five originalRect reads, three visibleRect
reads, seven addListener calls, one align write and one scaleMode write.
The bounded UIComponent initializer/method scan found no direct Stage access;
the derived layout/manager code supplies the concrete uses.

`gfx.layout.StagePanel.configUI` (body offset 2053154, length 859) reads
originalRect, calls `Stage.addListener(this)` at 2053207, writes alignment
`C` at 2053222, and copies `this.scaleMode` at 2053239. Its prototype assignment
at 2055150 supplies `noScale`. That is a **script default**, distinct from
MovieImpl initialization. StagePanel draw/alignment also require visibleRect
and originalRect. MarvelUIManager.Initialize registers at 169119; its onResize
body at 169388, length 32, calls the separate ExternalInterface
`CalculateUIScaling` bridge. Script references alone do not prove native
extensions state, registered callbacks, or their execution order.

## Evidence, validation and provenance

Private root: `~/.local/share/marvel-heroes-forever-rust/evidence/stage-runtime/`.
No original disassembly, binary bytes or extracted script bodies enter git.

- `final2.bundle.json`, SHA-256
  `c048ece78f4c7dd48d3ab585ddf3e69b3538952cf2c79131b9437ab0ea5c1d83`,
  preserves all REA records, including unsuccessful discovery receipts.
  `verification.json` and original `verify.py` index selected evidence and
  exclusions. Twelve new positive records completed with exit 0, no truncation
  or source mutation. One PE-loader timeout and one malformed map command are
  preserved and excluded. Raw `.data` reads made with the wrong section base
  are also explicitly excluded; corrected PE-mapped reads establish names.
- The checker independently compared 3,931 selected new instruction records
  (3,600 unique addresses), 2,609 reused instruction records and 52 PE-mapped
  byte arrays with the pinned file. All selected code intervals have contiguous
  checked coverage. Counts include overlapping observations and are not tests
  of runtime behavior. Reused PE-loader warnings about string-scan limits are
  retained; selected bytes still match the file.
- `script-uses/receipt.json`, SHA-256
  `ebd053895a7b8fd5d0678a6331b1f5286e985ca9b03e40fbe4c734e08f951bd6`,
  checks the movie hash, 331,498 instruction opcode positions, 47 selected
  function headers and 1,732 selected Push payloads. Private source windows and
  exact sites accompany it; this is static script evidence.
- `script-uses/viewport-candidates/` retains independent bounded discovery.
  Its selected native spans were subsequently captured in canonical REA
  records `ev_6337f7dc25de8bbceed804810fcc004074c9487a36943150b05030593ed73f90`
  and `ev_1fc4881c798d8e2f85777d04f8d9fd8d5862cd78b741eb9b40ed5f374516e684`.
- Fresh graph/method and property evidence is in `ev_7cb2da...`, `ev_59aadd...`,
  `ev_21722c...`, `ev_eac588...`, `ev_c3407e...`; corrected builtin names in
  `ev_3cbeac...` and `ev_ce1838...`; normalization in `ev_2b62ad...`; listener
  copy/callback in `ev_a82e3e...`; exact existing-entry flags and width/height
  write behavior in `ev_4a63ba...`. Full IDs and commands are in the verifier.

| Source | Licence/status | Use |
| --- | --- | --- |
| Owner's pinned retail executable and private `MarvelGame-frontend_manager.gfx` | Proprietary Gazillion/Marvel/Scaleform installation; private interoperability study | Native Stage graph, properties, viewport arithmetic, listener/resize paths, and exact saved script requirements |
| REA skill, native/artifact, recovery and evidence workflow references under `~/.agents/skills/reverse-engineer-anything/` | REA MIT toolkit documentation | Target-free collection, retained limitations and private evidence lifecycle |
| REA 6.9.0 Rizin provider; GNU objdump for bounded candidate discovery | Rizin LGPL-3.0-only; GNU Binutils GPL-3.0-or-later tools; neither linked into the client | Static decoding with independent original-byte verification |
| Original project global baseline, ExternalInterface, Array, class-registry, event/lifecycle, display and GFX documentation; cited private receipts; original AVM1 parser/diagnostic outputs | Original project analysis/code; underlying retail artifacts remain private | Reuse proven generic constructors, lazy resolution, context ownership, names, and script decoding; no external VM/SDK implementation imported |

REA was used only in target-free read-only mode; no shared configuration or
active engine was changed. The evidence session was exported and reset. This
docs-only study adds no Stage stub and claims no build, live frontend, network
or retail behavior validation.

## Bounded Rust geometry and retained movie state

`src/frontend/stage_state.rs` implements an original, standalone calculation
and state boundary. Its module hook and VM integration belong to the integrator.
It supplies no native-object factory, global binding, listener storage, host
callback, renderer transform or event scheduler.

**VERIFIED against the instruction-derived oracle within the bounds below:**
the four scale modes and nine alignments produce matching effective endpoint
bits and integer width/height. Scalar operations retain the recovered f32 order,
including converting both signed viewport endpoints before multiplying by 20
and subtracting. Replacing that sequence with `width * 20` is observably wrong:
the synthetic `left=16777217, width=333` case produces a Stage width of 334 with
unit factors and `noScale`. This is a numerical fixture, not an observed host
viewport or display resolution.

The API requires explicit caller-owned `MovieIdentity`, definition endpoints,
descriptor origin/extent, factors `+0xcc/+0xd0`, scale mode and alignment. No
`Default` supplies any of them. The factor names retain native offsets because
their association with physical surfaces or DPI has not been established.
`from_inputs` computes the effective rectangle; `from_snapshot` instead retains
the caller's explicit effective rectangle after validating admitted inputs and
getter arithmetic. The snapshot need not equal a fresh calculation. Width and
height read this retained state and use separate f32 subtraction/multiplication
before `cvttss2si`. The identity is immutable and never replaced by a getter's
caller context. Extended `visibleRect`/`originalRect` object getters remain out
of scope because their movie/context associations and allocations are separate.

`recompute` accepts new explicit descriptor/definition state. ASCII setters
recompute even when their parsed enum is unchanged. Changes are committed only
after all admitted arithmetic succeeds; an error leaves the previous state
intact. This transactional rejection is the Rust boundary for unsupported
inputs, not a claim that the native client reports these errors. The returned
`StageChange.geometry_changed` compares endpoints numerically, so signed zeros
compare equal. It does not summarize the native viewport setter's additional
mode/descriptor event conditions.

Changing scale mode to `noScale` from a different supported mode returns
`NoScaleResizeRequest` with the retained movie identity even if endpoints did
not change. Repeating `noScale`, changing alignment, and explicit recomputation
return no such direct request. Nothing executes or queues listeners. Integrators
must preserve the distinct caller Env used by extension resize arguments and
must not treat this request as a ready-made Rectangle or completed event.

Actual strings are admitted as ASCII without embedded NUL. Scale mode uses
case-insensitive exact matching for `noScale`, `exactFit`, and `noBorder`;
unknown strings, including empty strings and names with trailing spaces,
select `showAll`. Alignment folds ASCII to uppercase, examines only the first
two bytes, and keeps the unknown-to-center mapping and native getter spelling
documented above. Unicode normalization, embedded NUL and VM Value-to-string
conversion are explicit unsupported boundaries rather than substitutes.

The admitted geometry requires a present definition, finite normal-or-zero
endpoints, positive f32 definition extents, positive descriptor integer extents,
positive finite normal factors and checked i32 origin-plus-extent arithmetic.
Every evaluated geometry intermediate must be finite and normal or zero;
ratios, computed extents and final effective extents must remain positive.
The center path must fit its pixel conversion and subsequent signed integer
multiplication by 20. Unused center paths are not evaluated. Missing definition,
invalid native enums, subnormal/nonfinite geometry, collapsed extents and
integer overflow return typed errors. These bounds intentionally omit native
missing-definition preservation, invalid-alignment reuse and wrapping overflow.

The standalone `cvttss2si` helper covers every f32 numeric input: it truncates
representable values toward zero and returns signed `MIN` for NaN, infinities
and out-of-range values. A valid conversion of negative 2^31 has the same
numeric result. This differs from Rust's ordinary saturating cast. Effective
snapshot dimensions may exceed integer range and therefore return `MIN` from
the getter. Floating-point exception flags/traps and altered rounding modes
are not modeled; calculations assume the ordinary nearest/ties-even profile.
No physical surface size, native frontend state or loaded script default is
inferred from these bounds.

### Additional evidence and focused validation

The ASCII lowercase cache path is separately **VERIFIED statically**:
`0x140097b50` calls string transformation `0x140036750`, which calls mapping
helper `0x140072070`, then stores the interned result at original string `+0x10`.
The independent checker validates the ASCII bitmap and pair-table lookup for
all 128 code points: A–Z become a–z; the other ASCII code points map unchanged.
This extends the earlier normalized-cache evidence without making a Unicode
or embedded-NUL claim.

- Private `scale-normalization.bundle.json`, SHA-256
  `c6331f5a23f6d4ed63532837285bdb826ac3a1f72516cfe0e5a4e79b78a0f431`,
  contains REA record
  `ev_f9904ff0f9bb4804fab03a1056440999ce5d5f708b1eb9e6231815625facc0b0`.
  Original `verify-scale-normalization.py` checks 139 contiguous decoded
  instructions and 2,800 data bytes against independently mapped PE offsets.
  The result has exit 0, no truncation, no stderr and no artifact mutation.
  `scale-normalization-verification.json` retains its detailed checks. The
  target-free REA session was exported and reset with no retained target.
- Private `math-oracle/oracle.py` is an original register-machine interpreter
  driven directly by the canonical calculator instruction record, with exact
  rational ties-even f32 rounding. It independently verifies 1,829 original
  bytes across 344 instructions and stops after effective endpoint writes;
  it neither translates the Rust formulas nor launches target code.
  Oracle receipt SHA-256:
  `a21b5475ff91c32c5ea2dc7c620a1832e02e8fa7a89a1b67edc8e0447020efc8`.
- Private `math-oracle/vectors.json`, SHA-256
  `1d9a0f25082be3aa79bb82b5df4b18bfb6c5ea2759d66a6717176e548cb6f64a`,
  contains 360 geometry cases: eight synthetic inputs and two combinations of
  the owner's saved manager definition rectangle with explicitly synthetic
  viewports, each across four modes and nine alignments. Extracted definition
  inputs and instruction data remain private. It also contains 25 standalone
  CVTT boundary cases, including signed zero, subnormals, finite limits,
  infinities and NaNs.
- Private `math-oracle/check-rust.py` generates a private standalone runner
  importing this module. With Rust 1.98.1, both ordinary and `-O` builds passed
  all 360 exact endpoint-bit/dimension comparisons and all 25 conversion cases.
  `math-oracle/rust-verification.json` records module, runner, oracle, fixture
  and executable hashes and commands' exit results. Nine focused module tests
  pass, covering state retention, same-mode recomputation, conditional resize
  requests, alignment/string mappings, rejected geometry and atomic failures.

This addition uses only the original project analysis, the owner's private
artifacts, REA and GNU objdump already listed in the provenance table above.
The private oracle/checkers and Rust implementation are original code. The
focused standalone checks establish this bounded arithmetic/state behavior;
no full client suite, native execution, live viewport, VM, rendered frontend,
network or listener execution was tested.
