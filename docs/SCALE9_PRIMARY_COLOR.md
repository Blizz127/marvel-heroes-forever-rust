# Native primary style-color producer and cache

This study follows the color word written by the ordinary primary output
routine `bff70`. It uses bounded static REA observations of the private owned
build 1.52.0.1700 executable, SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
It does not assign a default bitmap color or infer channel order. Addresses
below omit the common `0x140000000` code base; offsets and masks are hexadecimal.

## Concrete callback and distinct source types

**OBSERVED:** provider constructor `c9740` retains incoming RDX and stores it
at full-provider `+58`, denoted Q. Writer `bff70` invokes Q's virtual `+20`
with a style ID and a pointer to a 16-byte retained result. This is Q's table,
not the separate provider interface at full-provider `+10`.

Two concrete tables have separately retained, complete 76-byte callbacks:

| Q table | Color callback at `+20` | Style-array field | Fill-count getter |
| --- | --- | --- | --- |
| `1424feaf0` | `c7d80` | `Q+18` | Not needed for this callback proof |
| `1424ffd90` | `e9180` | `Q+20` | `5dee30`, returns dword `Q+28` |

The second table's `+18` getter `5dee80` returns dword `Q+2c`, separately
bound to the line count by the saved reader. The two callbacks perform the
same operations using their respective array fields, in order:

1. Compute a 32-bit `styleID - 1`, zero-extend it, and multiply by 16.
2. Add the actual source type's style-array pointer to obtain the style record.
3. Copy the record's first dword to result `+0`.
4. Retain the record's nonnull pointer at `+8` through `031ce0`.
5. Release the result's previous nonnull pointer at `+8` through `031cf0`.
6. Reread the record pointer at `+8` and copy it to result `+8`.

No instruction writes result bytes `+4..+7`. The color store precedes the
reference callbacks, while the final resource-pointer read follows them.
Consequently a model that assumes immutable records or inert resource callbacks
must state those conditions; the callback itself is not a pure scalar getter.

The body contains no array-count or style-zero check. Style zero computes
`array + 0xffffffff0`, not `array - 16`, because the subtraction is 32-bit
and zero-extended before the shift. An original bounded reader would reject
zero and out-of-range IDs rather than emulate invalid memory accesses.

## Established constructor route and color-source limit

**OBSERVED:** the complete `0eb590` path allocates Q, installs `1424feaf0`,
and initializes the style-array pointer/count fields at `+18/+20`. It appends
a 16-byte style record, then passes that same Q as argument 2 to `c9740` on
the selected successful construction path. This binds the callback to an
actual provider source type, without assuming a neighboring table is equivalent.

The append sequence copies a dword from its local stack `+78` to record `+0`
and its prepared resource pointer to record `+8`. No explicit initialization
of that local color dword is established by the complete retained routine.
Its allocation/resource callbacks remain real external calls and are not
assumed unable to write reachable memory. This route therefore establishes the
color word's storage/readback, but not a fixed value. It is insufficient evidence
for white, zero, a specific alpha, or a channel-order convention.

The saved-definition constructor `0e8a50` separately stores its incoming Q at
definition `+28` and passes that pointer directly into `c9740`. The selected
clone/bind route `0e83c0` instead obtains Q from definition `+28` virtual `+80`,
invokes Q's `+88`, and passes the returned Q into `c9740`. These argument
contracts are retained separately from the explicit `0eb590` construction;
the clone and binding callbacks' transformations are not covered here.

## Saved-shape construction

**OBSERVED:** loader `06eec0` reads a saved 16-bit resource ID, allocates a
`0x50`-byte Q, initializes its array/count fields, and installs table
`1424ffd90`. It calls that table's `+78` reader, `0eb320`, with the original
load process, incoming tag ID, calculated remaining length, and a fifth
argument true. It then calls `0e8a50(newDefinition, Q)` and publishes that
definition through `08de00` on the selected `LoadProcess+3b8 == 0` branch.
Thus the Q retained at ordinary provider `+58` is concretely associated with
the saved reader and `e9180`; it is not inferred from the float-shape layout.

The other retained `0e8a50` caller, `08fc90`, constructs reserved IDs and a
different Q. It is not used to infer saved-style defaults. Actual dispatch to
the saved loader, later cloning/binding, and the particular live drawable must
remain distinct from this constructor dataflow.

**OBSERVED:** `0eb320` calls `0ea970`, whose fill-array path calls `0ea5a0`
and single-fill reader `0e9610`. The wrapper transfers temporary 16-byte fill
records to `Q+20`, preserving dword `+0` and retained pointer `+8`, and stores
the fill and line counts at `Q+28/+2c` on parser success. On parser failure the
wrapper cleans temporary arrays and returns false; outer `06eec0` does not
test that result. This study therefore requires the successful parse branch
and does not claim malformed-stream recovery. Array growth explicitly initializes new
record pointers at `+8`; it does not establish a color-dword initialization.
The allocator remains an unresolved callee, so this is not a claim about all
possible allocator or callback effects.

For saved solid type zero, `0e9610` calls `090730` and explicitly stores its
result dword into the fill record. `090730` uses the original tag argument:
signed tag value at most `0x16` selects `085840`, otherwise it selects `0858f0`.
The former reads three successive bytes into output bytes 2, 1 and 0, then
writes `0xff` to byte 3. The latter calls the former and replaces byte 3 with
the next source byte. Thus the qualified RGB(A) path yields numeric word
`0xAARRGGBB`, with alpha `0xff` for RGB. These readers perform no premultiplication
or other channel arithmetic. This byte contract is specific to the proved
reader, not an inferred convention for every resource or output shader.

For a fill type with bit `0x40` and without the earlier gradient bit `0x10`,
the bitmap branch builds the complex-fill pointer and reads the saved bitmap
ID and matrix. The ordinary successful resource route has no established
write of a fixed fill color dword. Its `resource+44 = 0xffffffff` store belongs
to the pointed resource descriptor; that is not the fill's color dword.

The branch at `e9ddb/e9dde` writes the separate literal `0xffa7dd39` to the fill
color when lookup fails, its returned kind is nonzero, or its resource is
null. A kind-one result also records the deferred index in resource `+44`.
This literal is not a universal bitmap default. The alternate nonnull resource
route that fails the type-mask comparison does not take that same literal
store. Logging, lookup, allocation and reference callbacks remain explicit
dependencies; no blanket purity or aliasing rule is assumed.

## Later binding and color preservation

**OBSERVED:** saved Q `+80` wrapper `0ea360` allocates a new `0x50`-byte object
and calls `0ea2c0`. That constructor installs the same saved table and forwards
the old fill count, old fill-array pointer, line count and following line-array
pointer to `0e91f0`. The latter copies each fill's color dword unchanged before
retaining its resource pointer; nonnull resources are subsequently cloned through
`0e8fc0`. No color conversion or fixed color replacement appears in those direct
copy instructions. Allocation, the resource clone, references and alias carry
remain conditions; `0e8fc0` is not expanded here.

**OBSERVED:** the concrete saved table's `+88` function `0e87b0` iterates the
fill records from `Q+20` using count `Q+28`, then the separate line records.
For fills it skips a null resource pointer or resource index `+44` equal to
`0xffffffff`. Otherwise it resolves the current binding entry, either from
the admitted direct table or through `0c9aa0`.

With a nonnull result whose virtual `+10` type satisfies
`(type & 0xff00) == 0x100`, the direct successful branch replaces resource
`+10`, clears its index `+44` to `0xffffffff`, and leaves the fill color dword
untouched by its own instructions. The failure branch releases and clears the
fill's resource pointer, then writes `0xffa7dd39` to the fill color. Therefore
this direct successful branch does not turn an earlier deferred color into
white. Preservation across the intervening resource callbacks still requires
the actual type, alias and lifetime conditions; this is not a proof of arbitrary
callback purity or of the final drawn color.

## Cache lifetime inside the writer

**OBSERVED:** after successful receiver begin, `c04d2..c04e9` initializes both
cached style IDs to zero once per `bff70` invocation. It explicitly clears the
two retained result **resource pointers**. It does not explicitly initialize
the two retained color dwords. A first selected style ID zero therefore matches
the initial ID and bypasses Q, leaving the color word unestablished by this
body. This is not evidence for an implicit zero color.

The ordinary single-style path always uses cache 0. Vertex bit `0x20` selects
which input style slot supplies the ID; it does not select a different cache.
On an ID miss, the callback fills cache 0's result, optional morph work follows,
and the selected ID becomes cache 0's new ID. On a hit, the stored color is
reused. The `0x10` dual-style branch uses both caches. The `0x8000` bypass
writes the separate vertex color zero and leaves both cached results unchanged.

The cache survives all of these boundaries within one writer call:

- Consecutive vertices.
- Successive at-most-256-vertex export chunks.
- Changes of output descriptor and the intervening triangle-index export.

The IDs are spilled before index export and restored at `c08fb/c0900`; the
index-export use of the same registers is not a cache reset. The resource
pointers remain retained until after receiver finish at `c0944`, with result 1
then result 0 released at `c0950/c095f` when nonnull.

If a later nonzero-to-zero transition actually invokes Q, its result is a
different case from the initial zero-ID hit. The concrete array callback still
has the invalid zero-index arithmetic above; no safe table lookup or universal
style-zero value is invented. The conservative zero-style rejection in
[SCALE9_PRIMARY_PACKING.md](SCALE9_PRIMARY_PACKING.md) remains appropriate for
that bounded component.

The native cache stores color and resource state, not only an ID-to-word map.
Any future resolver must retain caller identity, the selected table, resource
effects, and the invocation boundary. Resetting at each descriptor/chunk, or
using independent caches for the two single-style input slots, would change
the recovered behavior.

The following hand-derived trace illustrates the single-style contract. `A`
and `B` are different nonzero IDs, with no morph or override branch admitted.
These are instruction-derived expectations, not executed retail observations.

| Step in one invocation | Cache-0 effect |
| --- | --- |
| First `A` | Miss; call Q and save its current result |
| Next chunk, `A` through the other input style slot | Hit; reuse the same result |
| Next descriptor, `A` | Hit; descriptor/index processing did not clear it |
| `B` | Miss; replace retained result and ID |
| `A` again | Miss; call Q again, without assuming the earlier `A` word |
| Bypass flag `0x8000` | Leave cache and retained resource unchanged |
| New writer invocation, `A` | Fresh zero ID; miss again |

## Scope

Saved solid-byte mapping, saved record transfer, concrete callback readback and
writer cache lifetime are now connected conditionally. Actual tag dispatch,
selected live drawable, allocation/resource effects, callback field preservation
and observable shader use remain separate claims. A narrow read-only search for
absolute pointer references to `06eec0` found none; writable/relative tables and
runtime registration were outside that search, so no absence is inferred.

No new runtime resolver is added: deriving a fixed word for the selected ordinary
bitmap route would still require unsupported assumptions. The existing explicit
color-word input remains necessary, and must describe the same Q and one native
writer invocation across all chunks and descriptors. A fresh-cache-per-chunk
implementation would violate the proved lifetime even when every word came from
the correct array.

This evidence does not prove that the unspecified color bits from one
constructor are visible, invisible,
white, or otherwise interchangeable at draw time. It also does not loosen the
exact selected-format/stride admission of the record writer.

All client bytes and captures remain private under
`evidence/scale9-primary-color/`. The only outside source is the proprietary
owner-provided executable, studied for interoperability. Earlier project
evidence and original readers are reused with exact source identities; no
third-party implementation is copied. No client execution, service change,
GPU draw, renderer enablement, or runtime parity claim is made.

## Evidence index and checks

Private paths below are relative to
`~/.local/share/marvel-heroes-forever-rust/evidence/scale9-primary-color/`.

| Evidence | Supported boundary |
| --- | --- |
| `native/rea-shape-data-table.json` | Concrete float-shape callback table |
| `native/rea-provider-source-callers.json` | `e83c0/e8a50/e3f90` source forwarding |
| `native/rea-color-callback-and-source.json` | Complete float callback and `eb590` construction |
| `native/rea-definition-loader-candidates.json` | `06eec0` saved-loader construction, distinct reserved-ID route |
| `native/rea-saved-shape-table.json` | Saved Q callback/reader/clone/bind slots |
| `native/rea-saved-shape-color-getter.json` | Complete `e9180`, exact two count-getter leaves |
| `native/rea-saved-shape-bind-clone.json` | Complete `e87b0` and `ea360` |
| `native/rea-saved-shape-clone-copy.json` | Complete `ea2c0` source forwarding |
| `native/rea-saved-shape-record-copy.json` | Complete `e91f0` direct copy and resource-clone boundary |
| `saved-style-loader/rea-saved-style-reader.json` | Complete `eb320` saved wrapper |
| `saved-style-loader/rea-shape-style-parser.json` | Complete `ea970` style parser |
| `saved-style-loader/rea-fill-array-reader.json` | Complete `ea5a0` array reader |
| `saved-style-loader/rea-single-fill-reader.json` | Complete `e9610` solid/bitmap branches |
| `saved-style-loader/rea-color-byte-readers.json` | Exact `090730/085840/0858f0` color bytes and `ea3b0` growth |
| `reuse/cache-lifetime.json` | Existing complete `bff70` cache/readback and call profile |
| `saved-source-reuse/receipt.json` | Existing complete `c9740` source retention; distinct `122130+88` |

The nine main records are retained in `native/final.bundle.json` (six) and
`native/bind-clone.bundle.json` (three). Their SHA256 values are respectively
`a31e5fec605427ebc0edc76e91d74c90b8cac580c773ddc4593db74c9128e6bf` and
`f697cc45f3b2dcdd676e82c6e222918e02979c298f259eb9772b9f75f214e8b0`.
`navigation/audit.json` verifies complete, untruncated exit-zero results,
unchanged subject hash, all 5,107 decoded code bytes in 1,318 instruction rows,
and 248 explicit table bytes. It reports no errors, warnings or invalid rows.
Full chained unwind families were used for split functions; the two four-byte
leaf getters have no unwind entries and are identified as exact complete leaf
bodies rather than arbitrary metadata gaps.

The five saved-reader records are retained separately in
`saved-style-loader/final.bundle.json`. Their independent byte audit,
`saved-style-loader/verification.json`, verifies 6,089 code bytes in 1,573
instructions and the same complete/exit-zero/unchanged-artifact conditions.
Together the new evidence comprises 14 records, 11,196 code bytes, 2,891
instruction rows and 248 table bytes. The consolidated `receipt.json` and
`MANIFEST.json` retain exact Evidence IDs, file hashes, source licences and the
remaining limits. Both target-free ledgers were exported; the main ledger was
reset after each export. No native analysis target was opened.

The consolidated receipt SHA256 is
`320d325505615b132b72f1d511dbcd7e2bddcec0dc41a982a17bbcf4d03c12b6`;
the 76-file manifest SHA256 is
`b6004ae7994a95d8967d60c79fd77f5d9250bfef108adeefe0541065646fbff6`.
No Rust build/test was needed for this documentation-only increment. The
checks above verify retained static evidence; they do not claim executed
color resolution or output parity.
