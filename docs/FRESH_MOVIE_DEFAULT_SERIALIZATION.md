# Selected movie-default tag serialization

The actual GFxAction_OpenMovie default record now has a bounded saved-tag and
native serializer graph. Its immediate field writes preserve StartPaused's
mask under the stated ordinary-load conditions. The post-`None` caller path
has also been identified through the UObject serializer's return.

**This does not establish an effective StartPaused value.** Reference loading,
cold class-cache initialization, populated-storage cleanup, backing I/O and
the enclosing loader lifecycle remain explicit conditions or boundaries.
No product default changed, and no native client or sequence ran.

`VERIFIED` denotes exact source bytes, bounded correspondence and the stated
range calculations. `OBSERVED` denotes saved records or static instructions.
Conditional composition is not a captured native invocation. The preceding
[copy study](FRESH_MOVIE_DEFAULT_COPY.md) covers `+0x2c8`; this study follows
serialization `+0x290` and separately follows cleanup `+0x2e0`.

## Actual saved input and scope

The selected record is GFxUI export index 146, reference 147, with exact class
GFxAction_OpenMovie. Its 2,324 bytes contain the previously admitted four-byte
NetIndex prefix and five top-level tags. The final eight-byte `None` ends at
the export boundary. No additional default tail is invented.

| Saved scope | Elements | Tags per element | Nested arrays |
| --- | ---: | ---: | --- |
| OutputLinks | 2 | 13 | Links explicitly empty in both |
| VariableLinks | 3 | 17 | LinkedVariables explicitly empty in all three |

There are 77 nested tags and 82 tags overall. Every declared field in these
five link structs is present. The stream has six tagged scopes, seven array
count words and five explicitly empty nested arrays. Absence elsewhere in
the CDO does not acquire a zero value from these observations.

The retained independent byte comparison passed 1,476 checks. It preserves
every tag, declared field identity, value span, array count and terminating
`None`. The supporting schema remains the exact 70-node graph from the copy
study; no arbitrary package is admitted by a short property-class name.

The nine object-valued occurrences are distinct:

- MoviePlayerClass has export reference 37, identifying local
  **GFxUI.GFxMoviePlayer**, not MarvelGFxFrontEnd.
- Three ExpectedType values have import reference -30, resolving through the
  exact Core.Class import to **Engine.SeqVar_Object**.
- Two LinkedOp and three CachedProperty values are explicit null references.

Six NameProperty values are FName `None`; they are not null object references.
These identities come from metadata-only target reads. No referenced class
payload was followed in this package audit, and no map-instance override is
merged into this class-default record.

## Concrete Struct serializer and admission

StructProperty table `0x141e120a0` selects `0x1407cc720` for `+0x290`.
With the admitted archive loading profile and selected descriptor flags, it
calls the existing tagged loader `0x140755f10`. The descriptor is supplied
as both schema and comparison/default-class argument; the fifth argument is
forwarded, and the sixth is zero. Incoming argument4 is not used on this route.
The saved tagged element spans have different byte lengths. Each ends at its
own `None`; the Array serializer's size-derived argument4 does not justify
splitting the encoded payload into equal byte chunks.

All five supporting ScriptStruct records have zero loaded/serialized script
lengths and the next saved u32 word equal to1. The exact ScriptStruct metadata
serializer `0x140750fa0` reads that word into descriptor `+0xe0` after its
UStruct base serializer. This joins offset 48 to the flags only under the
established 868/3, cursor-preserving file profile. The base invokes archive
and Link/preload operations; this is not a universal fixed-offset grammar.

With those loaded flags unchanged, masks `0x20`, `0x80` and `0x100` are clear.
For archive `+0x14 != 0`, `+0x18 == 0`, `+0x20 == 0` and `+0x70` bit 16 clear,
the selected values take tagged serialization. Different flags can select
binary or differential serializers, preload the descriptor or invoke its
virtual `+0x280`; those alternatives remain documented in the private receipt.
Version868 alone is insufficient to select the path.

The five saved nested arrays are empty, so their replacement data does not
invoke deeper element serializers. This does not mean their previous inherited
buffers were empty or require no cleanup.

## Exact type matching and default scope

The loader obtains the property's serialized type through virtual `+0x2e8`
at `0x140756910` and compares the returned eight-byte name with the saved tag
type at `0x14075691b`. It does not accept arbitrary Class/Object spelling
substitutions.

The exact ClassProperty getter `0x1407c0600` returns FName qword 5. The native
name-registration block copies the literal ObjectProperty into an entry,
stores encoded index `0x0a`, and calls `0x14079cca0`; that installer shifts
the encoded index right one bit and inserts the entry at index 5. Under
ordinary initialized name state, this explains the selected ClassProperty
declarations' ObjectProperty tags. Both native serializer tables select
`0x1407cc6b0`. The stream and declared classes retain their distinct identities.

Type/name matching is followed by property filter `0x1407581b0`. Four unique
selected fields carry bit 13, yielding ten saved occurrences. That bit rejects
a field only when archive dword `+0x28 != 0` and signed dword `+0x50 <= 0`.
Output.bIsActivated additionally carries bit 34, requiring archive `+0x1c == 0`.
The exact Bool/Object `+0x330` predicates return false for these selected
flags, excluding archive `+0xd0` from this profile. Other flag combinations
are not admitted by the same conclusion.

The counter is grounded independently of its presumed UE name:

- Fresh CLinkerLoad construction clears `A+0x50/+0x54` at `0x14075a53e`, where
  archive `A = L+0x1c8`. This is a construction checkpoint, not immutable state.
- UObject dispatcher `0x1407a7e90` first calls archive virtual `+0xb0`, tests
  object flag 9 and increments **dword field** `A+0x50` at `0x1407a7ec9`.
- Recursive tagged Struct calls share this archive. With stable flag 9, a valid
  nonnegative initial counter and no overflow or callback mutation, the counter
  stays positive for the selected outer default-serialization interval.
- After normal return the dispatcher decrements it at `0x1407a7f94`.

The selected saved object flags and earlier fresh route include flag 9 and
exclude the state-frame flag. These exact inputs support the conditional scope;
they do not prove an arbitrary current object or archive has those values.

## Local writes and reference boundaries

The loader computes each destination from the enclosing base, linked property
offset and fixed-array index. All selected indices are zero and dimensions
one. Checked offsets, widths and bounds remain admission conditions; native
wrapping arithmetic does not become a Rust bounds guarantee.

| Property operation | Established local contract |
| --- | --- |
| Tagged Bool | Exact Bool-class fast path ORs or clears only the declared dword mask |
| Int / Float | `0x1407cc610` requests four bytes through `0x1406d66b0`, then archive `+0x08`; byte-swapping uses four reversed one-byte requests |
| Name | `0x1407cc6a0` calls archive `+0x38`; the admitted Name reader writes one qword after package-name lookup |
| Object / Class | `0x1407cc6b0` calls archive `+0x30`; applicable selected flags exclude its optional notification |
| String | The previous bounded serializer updates its 16-byte header and separately allocated character storage |
| Array | The previous bounded serializer prepares its 16-byte header and element storage, then invokes its inner serializer |
| Struct | The admitted tagged route dispatches the exact finite saved member list |

The direct Bool `+0x290` callback was also verified, but the selected tagged
Bool operations use the fast path instead. Under full type/filter admission,
the graph contains 41 Bool updates and 46 `+0x290` calls, including five implicit
Struct calls from the outer arrays. These are conditional graph counts, not
measured invocations.

The only top-level write overlapping `[A_object+0x120, A_object+0x124)` is
bDisplayWithHudOff, whose mask `0x10` preserves StartPaused mask `0x04`.
Other top-level destinations are MoviePlayerClass `[0x110,0x118)`, OutputLinks
`[0xac,0xbc)`, VariableLinks `[0xbc,0xcc)` and ObjName `[0x6c,0x7c)`.
Nested writes stay within the selected 56-byte OutputLink or 84-byte VarLink
elements, provided their allocations are valid and disjoint from the action.

Five null object values take the already checked null-reference result. The
nonnull values retain real loader edges: export 37 to `0x140760610(L,36)`,
and import -30 to `0x140761b50(L,29)`. Neither a bounded qword destination nor
the target's metadata identity proves those calls cannot reenter or mutate the
action. No such purity assertion is made.

The shared current-property pointer is saved, replaced and restored around
normal non-Bool serialization. The Bool fast-path class cache can require
initialization. Normal-return restoration does not establish exception safety,
thread safety or the absence of target reentry through these shared states.

## Populated cleanup and backing archive

Actual saved-tag cleanup has four Array, two Struct and two String callback
contracts. Runtime invocation counts depend on prior populated counts. The two
top-level Array offsets are nonzero; nested OutputLink.Links has offset zero
and retains its native guard.

That guard consults a cached Class descriptor and the property's outer-class
chain. A matching class or a still-null cache returns without cleanup.
A cache miss can construct/link descriptors before the check. The ordinary
cleanup conclusion requires the admitted nonmatching chain and confined cache
effects. It is not silently replaced with an unconditional free operation.

On the ordinary path, Array/String writes remain in their headers and releases
affect their owned pointer-backed storage. Struct cleanup walks the linked
bit22 field list. Nested Links and LinkedVariables release their old buffers
without per-element object cleanup under their exact inner flags. Valid unique
ownership, bounded stable old counts, nonaliasing and confined allocator/free
services remain required. Replacement count zero does not establish these facts.

Ordinary CLinkerLoad raw-read slot `+0x08` forwards to its current backing
reader. Two established memory-reader identities copy bytes and advance a
cursor without local capacity/overflow guards. The buffered-reader route can
fill cached windows through I/O/decompression workers. This study preserves
those service boundaries and does not infer the current backing solely from
the package extension or compression bit.

The valid Name-reader route resolves a package name index and serialized number
before its qword destination write. Malformed-name diagnostic paths remain
separate from this successful-load profile.

## The post-None interval and remaining limit

The complete tagged loader's loading-side terminating `None` enters its
restore/return epilogue. It has no subsequent omitted-property initialization
pass. This applies to the nested scopes and the final CDO `None`.

After that return, stable clear state-frame flags exclude the dispatcher's
state-frame follow-up. The default counter is decremented, then archive
virtual `+0xb8` runs. Exact `+0xb0/+0xb8` bodies write only export-record
position dwords after obtaining the current backing cursor. For the three
established backing identities, cursor getters perform one dword read and
return. This requires valid export metadata disjoint from the action and an
admitted current backing; an unrelated replacement reader is not covered.

The UObject tail with archive `+0x1c == 0` skips its extra flags serialization
and tail-calls archive virtual `+0x28`. The exact ordinary archive slot selects
`0x14058fea0`, a three-byte return with no argument use, stores or calls. No
return value is fabricated for this void accounting operation.

Thus the local post-None path through UObject return is identified. It does
not prove completion of the enclosing preload/export-loader operation or
exclude the earlier reference/cache/I/O reentry. The preceding fresh-copy zero
checkpoint therefore remains separate from an effective loaded StartPaused
value, later instance overrides, configuration and actual action invocation.

## Validation, evidence and provenance

The original private composition checks all 82 saved tag destinations and masks.
Four negative admissions reject a substituted ClassProperty tag spelling, a
nested field outside its struct extent, a HUD mask colliding with StartPaused,
and a nonempty nested array without an admitted element stream. These are
proof-tool checks; they are not native behavior or parity tests.

All raw assets, native output and receipts remain under the private directory
`~/.local/share/marvel-heroes-forever-rust/evidence/fresh-movie-default-serialization/`.
The aggregate `receipt.json` records source hashes, component receipts and exact
checks. Native analysis used bounded target-free REA; all retained calls were
complete/untruncated and sessions were exported/reset. One helper's failed
export to an absent directory and subsequent mistaken reset are preserved in
its remediation receipt; the identical bounded read was then durably retained.

| Component receipt | SHA256 |
| --- | --- |
| `package/receipt.json` | `ff66cf170cc5ac067a323a85eb9b9b6fabfa65b7db1d79242f51c255e8651fbe` |
| `package/struct-flags/receipt.json` | `0be9f7b217c684a138a3e91fdde02d5af54785016355acbdb8e27bf6ab23002a` |
| `struct-native/receipt.json` | `1a98b465b8c988f6f16bda844aa571792492e5a2156e7c82cd5ba8ba9e863cb1` |
| `cleanup/receipt.json` | `877178b2af6898c8d49ea621be28ad5b7d9193a77411103196f52b58616b9d80` |
| `archive/receipt.json` | `dbe458655dfda7d3784c6d9a81d1d56e231b440074c65643d14b2e40c4945b6b` |
| `archive/pre-post-addendum/receipt.json` | `fd77f38db770636a0a2f94eacb1b352a870aa21f6a6341b42de47d84f7395e82` |
| `scalars/receipt.json` | `494b5ed5333d0aee4b81d5c2fc61df1e5981962372ea4e9035a95b80a502ca86` |

| Source | Licence / use |
| --- | --- |
| Owner's pinned MarvelHeroesOmega.exe, SHA256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary owner copy; exact selected serializer, filter, archive, name and caller contracts |
| Owner's pinned GFxUI.upk and Engine.upk; exact hashes in package receipts | Proprietary owner assets; saved tags, exact schema/class identities and descriptor words |
| [FRESH_MOVIE_DEFAULT_STATE.md](FRESH_MOVIE_DEFAULT_STATE.md), [FRESH_MOVIE_DEFAULT_COPY.md](FRESH_MOVIE_DEFAULT_COPY.md), [CLASS_DEFAULTS.md](CLASS_DEFAULTS.md), [UPK_CLASS.md](UPK_CLASS.md), and their private evidence | Original project research; preceding construction, copy, metadata, archive and preamble contracts |
| Retained scale9, Entry-object, current-movie-default and world-transform native evidence; exact paths/hashes in component ledgers | Original project research; complete prior methods, import declarations and owned-PE mapper reused |
| `/home/blizz/.agents/skills/reverse-engineer-anything/SKILL.md` and installed rea-agents | MIT toolkit; bounded evidence capture, export, reset and remediation |
| Installed Rizin and GNU objdump | LGPL-3.0-only and GPL-3.0-or-later; static decoding only, no implementation copied |
| Python standard library and original Rust package-reader validation wrappers | PSF and original project code; bounded extraction, independent correspondence and finite composition |

Only this original document enters git. No product source/build, full suite,
native launch, configuration change, service/network action or parity result
is claimed by this increment.
