# Primary generated-vertex export

`gfx::native_primary_export::VertexExportCursor` implements the selected native
descriptor copy/reset and generated-vertex scan. It bridges the existing
generated records to the later packing stage without sorting, renumbering or
changing coordinate bits. It does not create triangles or choose the current
provider, descriptor, normalization state, output format or renderer.

## Native descriptor and scan contract

**VERIFIED static bodies:** `0x140100020` copies seven DWORDs from the selected
28-byte descriptor, then resets destination word5, at byte offset `0x14`, to
zero. Its return is at `0x140100081`, ending the 98-byte function. The following
14 INT3 bytes in the earlier 112-byte capture are padding, not part of this
operation. Native source/destination loads and stores are interleaved; no
arbitrary overlapping-memory or memmove behavior is asserted. The Rust input is
a descriptor value, and its result has independent storage.

Complete 208-byte `0x140100090` takes the tessellator, copied descriptor,
destination records and requested count. A zero request returns zero without
advancing the cursor. Otherwise it reads the cursor as an unsigned DWORD and
compares it against the tessellator's 64-bit generated-record count. An already
exhausted cursor returns no records and remains unchanged; it is not clamped.

For each visited vertex, native page addressing selects a 20-byte record from
pages of sixteen records. The zero-extended u16 at record `+0x12` is compared
against the **full u32** at descriptor word0. A match copies all five DWORDs in
order. Both match and miss advance descriptor word5 by one. The scan ends
immediately when it has copied the requested number, or when the source ends.
The returned integer is the copied count, not the number of records scanned.

There is no mesh-sentinel exclusion here: descriptor value `0xffff` matches a
record with mesh value `0xffff`. A descriptor above `0xffff` matches none. The
native generator's earlier rule for unused vertices is a different operation.
The method neither checks nor uses descriptor word6 as a scan bound, and it
does not sort by the copied vertex's local ordinal.

The existing `GeneratedVertex` fields cover the five copied words:

| Word | Retained fields |
| --- | --- |
| 0, 1 | Exact X/Y IEEE32 bits |
| 2 | u32 mesh-local ordinal |
| 3 | Two u16 styles, in their original order |
| 4 | u16 flags followed by u16 mesh ID |

No floating-point arithmetic occurs. Nonfinite values, signed zeros, raw style
bits and flags are copied unchanged. Valid source pages, distinct destination
storage and a stable source during the call are conditions of this transfer.
Changing the generated geometry between calls is not proved safe merely by
retaining a cursor.

## Rust API and bounds

`copy_descriptor([u32; 7])` resets only word5. `from_snapshot([u32; 7])` preserves
every word for explicit continuation, including arbitrary u32 cursor values.
`snapshot()` returns those words without exposing mutable state.
`export_batch(&[GeneratedVertex], requested)` returns records in scan order and
updates only word5. The returned vector length is the native copied count.

The slice represents the concatenation of the native generated pages. Its
positions must already belong to the caller's intended stage. In the native
output route, inverse-outer transformation and normalization process **all**
generated vertices before this selection; selecting first would change extrema.
See [normalization evidence](SCALE9_RENDERING.md) and
[triangle transfer](SCALE9_GEOMETRY.md).

The Rust helper admits at most 65,535 source vertices and 256 requested records
per call. The former is an inspection bound shared with the normalization API;
the latter is the recovered caller's batch size, not an internal limit imposed
by `100090`. Caps are checked even for a zero request. They exclude native
cursor wraparound and invalid-memory behavior. Limit/allocation errors leave
the descriptor unchanged; native failure effects are not emulated. No native
page allocator or ownership/refcount operation is introduced.

## Focused checks and provenance

**VERIFIED implementation checks:** four focused tests pass in a frozen private
composite. They cover reset versus resume, skipped-record progress, partial
batches and final exhaustion, nonfinite/signed-zero bit preservation, nonmonotonic
ordinals, full-width ID comparison, matching `0xffff`, zero requests, an initial
`u32::MAX` cursor and atomic cap rejection. The composite emits one unused-method
warning from unchanged math support excluded from this focused composition.
No full suite or native process was run. Production integration and independent
byte-witness checks are separate from this receipt.

Private evidence root: `evidence/native-primary-export/` beneath the private
project storage. `focused/receipt.json` SHA-256:
`657968d3275fb3d119c1202880e69c29e4231e510cf84f2ab2d058bbe73f4df8`.
Frozen new module SHA-256:
`5a9b1e80de79537bedc42ca0409ca795ab6245d9e0c8d99c13c7a8f2b6d0409d`.

This original Rust composition uses the already registered owned 1.52.0.1700
executable, SHA-256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`,
and prior REA records `ev_66aafda720ebe6405b090d7fdce305935eae9a3dbffedae3e2bc0f8e1dc11d88`
and `ev_9e3ed8adc570762e215d6367e267064d679e2daf46fa54c05993685ca2692c33`.
The corresponding saved results are in `scale9-geometry/native/` and
`scale9-output/`. No outside implementation, native allocation, render-state
choice, shader behavior or pixel parity is supplied by this copy operation.
