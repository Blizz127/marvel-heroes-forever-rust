# Model draw receiver: bounded search and concrete alternate links

**OBSERVED frontier:** the bounded investigation did not identify the concrete
receiver supplied as `0x140bfabc0`'s incoming RDX. That value, called `D` here,
is the receiver whose virtual `+0x38` consumes the Model record in
[MODEL_DRAW_CONSUMER.md](MODEL_DRAW_CONSUMER.md). It remains distinct from the
constructed Model proxy `P` and the global graphics backend.

Two concrete links did advance: the typed component's conditional callback on
its constructed proxy, and the saved element conditions governing the wrapper
referent used by the normal draw record. Neither supplies D, invokes the
selected Model's bound state, or identifies a compiled shader record. All work
is static and private; no native client or renderer was executed.

## Exact receiver search boundary

The new navigation checked exact relative E8/E9 target candidates in the owned
executable's `.text` for `0x140bfabc0` and `0x140bf8ad0`. Both sets were empty.
This describes those exact encodings/targets and the inspected section, not
absence of indirect calls, aliases, alternate code sections or a caller.
The already verified sole direct constructor candidate
`0x140bf8b06 -> 0x140bed200` was reused without repeating that search.

The qword preceding proxy table `0x141f9bc98` points into code; it does not
provide an established RTTI/name route. The named table slot `+0x18` still
points to `0x140bfabc0`. No broad scan of arbitrary virtual-slot calls was
performed after these bounded candidates were exhausted.

A nearby value must not be substituted for the missing target: the proxy's
own table `+0x38` points to `0x140bfb470`, but `ceb180` uses D's table. There
is no retained equality D = P. Likewise, the retained native UI renderer,
resource-command paths and authored Rust renderers do not establish that their
receivers are D. The named startup/scene evidence contains saved graphs and
conditional engine/resource operations, not this missing invocation.

## Concrete typed component callback

**VERIFIED static link:** the existing Engine.ModelComponent table
`0x141f9b2a8`, slot `+0x460`, selects proxy factory `0x140bf8ad0`. It allocates
and constructs P, then returns P. Before returning, it calls the original
component's virtual `+0x520(component, P)` only when global DWORD
`0x142aee944` is nonzero and P is nonnull. This is a typed forwarding call;
registration or draw submission does not follow from its existence.

The exact slot at `0x141f9b7c8` selects `0x140c0ebc0`, a complete 688-byte
body. On its ordinary returning path it:

- Writes `P+0xc0 = 2` and invokes P's virtual `+0xb0` with argument 1.
- Resets the count at `P+0xec`; when capacity at `P+0xf0` is nonzero, clears
  that capacity and invokes the storage helper on `P+0xe4` as applicable.
- Continues its wrapper loop only when the 32-bit subtraction result
  `Model.u32+0x4ec - Model.u32+0x518`, interpreted as signed, is positive,
  and P's wrapper count is
  positive. These raw words are not given saved-format or semantic names.
- For an eligible wrapper with a nonnull original element and positive node
  count, reads the first u16 from the element's native node array. It examines
  collection entries exposed by opaque iterator helpers around Model `+0x4e4`
  and tests their u32 lists for equality to that node value.
- Supplies a selected entry's raw words `+0xec/+0xf0`, or explicit zeros if
  no entry was selected, to `0x140bf3050(P, wrapper_index, word0, word1)`.

Exact iterator, branch and diagnostic anchors are retained privately. The
iterator/storage/virtual helper implementations and invalid-state behavior are
not reproduced; this is not a complete Model `+0x4e4` container decoder or a
claim of its current contents. Its identity is not inferred from another
container merely because offsets or record sizes appear similar.

The complete 177-byte `0x140bf3050` grows the `P+0xe4` array when necessary
through opaque helper `0x140fcad70`. It converts each supplied signed word with
`CVTDQ2PS` and stores the resulting two low float words as one eight-byte
record at `array + wrapper_index*8`. Exact rounding state, exception behavior,
allocation failure and invalid-index diagnostic fallthrough remain unproved;
no product float conversion is derived.

The already retained **dynamic** branch of `bfabc0` reads this numerical pair
into `P+0xb8` at `0x140bfae00..0x140bfae0b`, or writes zero when the index is
outside the count. This closes a metadata producer/consumer relationship.
It does not make the pair a pointer, construct D, or establish that the selected
FrontEnd uses this callback or dynamic branch. Model `+0x4e4` and the source
words remain unbound to the selected saved package fields.

## Conditional wrapper referent from saved material data

The complete retained wrapper constructor `0x140becd50` stores the original
element pointer at wrapper `+0x10`. It initially copies `element+8` to wrapper
`+8`, but the result is conditional:

1. If the subsequently tested `element+8` is null, substitute the fallback
   `qword[qword[0x142c31378]+0x160]`.
2. Otherwise, if either qword `element+0x30` or u32 `element+0x28` is nonzero,
   call the referent's virtual `+0x320(referent, 6, 0)`. A zero return selects
   that same fallback; a nonzero return retains the initial referent.
3. A nonnull referent with both fields zero skips that virtual probe and
   retains the initially copied value.

These branches require stable source fields for the usual value-equality
interpretation; separate loads do not prove concurrent stability. The fallback
global, virtual result and resolved pointer identity are not inferred from a
package reference. An unconditional wrapper+8 = resolved material assertion
would therefore be wrong.

**VERIFIED fresh-load link, OBSERVED saved inputs:** all ten retained FrontEnd
component elements have zero in the relevant serialized header and array count.
The exact ordinary fresh-load path initializes element `+0x30` null, and the
zero-header helper leaves that null value before storing it back. The native
array loader called with element `+0x20` clears its count word at `+8`; a
serialized zero count bypasses append, establishing element u32 `+0x28 = 0`.
These are specific inspected load paths, not generic array-layout assumptions.

Consequently, **if** each saved material reference successfully resolves to a
nonnull `element+8` pointer and these live fields remain unchanged until wrapper
construction, those selected elements retain that pointer in wrapper `+8`
without the `+0x320` probe. This does not establish that resolution or stability
actually occurs. The saved references remain imported Engine.Material `-33`
(`removesurfacematerial`) and `-32` (`defaultmaterial`), distinct from native
pointers and from the fallback referent.

Constructor `0x140bed200` later calls concrete helper `0x140913c10` on wrapper
`+8` and an output temporary, and accumulates returned low flag bits. The normal
draw path separately calls that referent's virtual `+0x2f8` with RDX and R8D zero
and places the return value in draw record `+0x10c`. These remain precise next
anchors. Neither method's semantic name, shader variant nor returned runtime
object is established here.

## Product and next investigation boundary

Existing ModelComponent/Model/index APIs retain the actual saved references,
raw fields and ordered node data. Explicit index snapshot calculations remain
separate from native pointer resolution, the fallback global and proxy metadata.
No new decoder or constructor is added: the new callback's collection and
process pointers lack a complete saved-data contract, and the missing draw
receiver cannot be manufactured from the serialized graph.

A direct D binding still requires an actual proxy invocation with known incoming
RDX or a grounded D constructor and `+0x38` implementation. The bounded failed
candidates above should be reused, rather than searched repeatedly. Independently,
the concrete wrapper helper `0x140913c10`, referent virtual `+0x2f8/+0x320`
and exact imported-material resolution are now explicit upstream targets for
the material/runtime boundary. Shader bytes require a material/variant →
handle → owned bounded file-record association; no arbitrary shader sample is
selected.

## Verification and provenance

All underlying target bytes and study artifacts remain under the private root
`~/.local/share/marvel-heroes-forever-rust/evidence/model-draw-receiver/`.
The owned 1.52.0.1700 executable SHA-256 is
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.

One fresh target-free REA/Rizin record independently agrees with the owned PE
on both complete callback bodies: **865 code bytes / 210 instruction rows**,
plus three exact eight-byte table cells. Evidence ID:
`ev_50d452fba310dc841a10bc64cb53ae47b23781e4be0be1b02b0e9370eaad7ea1`.
The exported bundle SHA-256 is
`f372f8d63064c1d5266050faa531030f90972a3a931b929bfade181c4802199f`.
The command completed untruncated with unchanged target hash. REA cleanup
records no open target, pending cleanup or retained records.

| Private component | Receipt SHA-256 | Scope |
| --- | --- | --- |
| `native-route/receipt.json` | `ceea3f275106ac0f89d7a06711a1af91adec9a68a52fe04f991f55f9d3646938` | Exact-target navigation, typed callback, numeric writer and prior dynamic consumer |
| `retained-anchors/receipt.json` | `d2578fc7b55fe527ff35520bb7bfa76f0156f2eae0a9cf3580f0ca1cfc72ed01` | Named existing startup/renderer/proxy alternatives, with receiver identities kept separate |
| `wrapper-source/receipt.json` | `35eb588f1659c36daa60759360efa315c5c4553fea1a1a7d837d1ceccec322a6` | Conditional wrapper construction, ten saved inputs and exact fresh-load zero mappings |

The wrapper audit independently checked 1,053 retained bytes / 277 instruction
rows from its two complete earlier bodies; it reused their original PE/REA
identities. The proxy-return audit checked 109 retained bytes / 30 rows with
its prior class/REA seal. These are distinct from the fresh callback checks.
No new shader/package extraction, product test/build, native runtime or service
operation was performed. Focused document whitespace and local-link checks
passed; only this document is added.

Original project source was frozen at
`c0e6050dc44f85d05f0c058fccd1eabcbf01b2af`. The parent `sources.json` and
component ledgers record exact paths, hashes, purpose and licence categories.
The owned retail executable is proprietary; existing original private helpers
and study receipts were reused read-only. No outside implementation, SDK
source or generated conversion code was introduced. GNU objdump and REA/Rizin
were used as analysis tools only. Previously studied API sources remain
attributed in their earlier ledgers; no new API identity is inferred here.
