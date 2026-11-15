# Explicit native index-word transfer

This module implements the recovered rebasing result for explicitly supplied,
disjoint little-endian source and destination byte slices. Each destination
WORD is `(sourceWord + suppliedCumulativeVertexOffset) mod 65536`, in source
order. It supplies no upload collection, buffer allocation, current cumulative
count, renderer, device operation or native dispatch state.

The exact native scalar and SSE bodies support that word operation under their
valid memory/alignment profiles. The Rust implementation always uses safe byte
accesses. It does not emulate the native global selector, physical-address
dispatch, streaming stores, memory ordering, overlap effects or faults.

## Native arguments and caller

**VERIFIED retained static instructions:** the selected dispatcher is
`0x140033050..0x140033082`, the scalar body
`0x140032ee0..0x140032f9d`, and the SSE body
`0x140032fa0..0x140033047`. End addresses here are exclusive. Neighboring INT3
padding in the captures is excluded. These leaf bodies have no `.pdata` entry;
their complete retained instruction paths end in the stated return or tail
branch. The engine caller `0x141255030..0x141255280` is a complete 592-byte
unwind-bounded function.

The transfer arguments are:

| Register | Native role |
| --- | --- |
| RCX | Destination byte address |
| RDX | Source byte address |
| R8D | Unsigned 32-bit **word count**, not byte count |
| R9W | Low 16 bits of the cumulative vertex offset |

At **0x1412551fe..0x141255212**, the caller supplies source
`owner_staging_base + M.indexByteOffset`, loads the DWORD count from `M+0x48`,
zero-extends cumulative `SI` into R9D, and calls `033050`. The caller starts
its cumulative ESI count at zero and adds each mesh's DWORD vertex count
`M+0x44` with 32-bit arithmetic. At **0x14125522b..0x141255237**, the destination
index pointer advances by twice the zero-extended DWORD index count using
64-bit address arithmetic. There is no inverse-normalization matrix argument
to this transfer. Actual collection identity, readiness, allocation and current
cumulative count remain caller obligations.

## Scalar, vector and tail behavior

The scalar body consumes `count & 3` initial words, followed by groups of four.
Every element uses a WORD load, `ADD AX,R9W`, and WORD store. Addition wraps;
there is no saturation, ordinal sorting, mesh lookup, triangle reconstruction
or change in index order. After its remainder prefix, the full-group endpoint
uses the zero-extended `(count & ~3)` multiplied by two in 64-bit address
arithmetic. Zero count performs no source or destination access in this body.

The SSE body first compares the source and destination low four address bits.
Different residues tail-call the scalar body. Equal residues compute:

```text
vector_start = align_up(destination, 16)
vector_end   = align_down(destination + 2*zext(count), 16)
source_end   = source + 2*zext(count)
```

When a vector interval exists, initial WORD operations move toward
`vector_start`; eight-lane `PADDW` blocks use `MOVDQU` source loads and
`MOVNTDQ` destination stores, then remaining WORD operations stop at
`source_end`. Sign extension while broadcasting R9W does not change the repeated
16-bit offset. All valid scalar/vector lanes therefore have the same modulo
65,536 result.

**Alignment boundary:** equal **odd** source/destination residues do not
establish the vector store's alignment. A two-byte prefix step from an odd
address can pass the aligned boundary without landing on it. If that profile
enters the vector region, the captured `MOVNTDQ` cannot be treated as an
ordinary valid unaligned store. The safe Rust helper supports odd-address byte
slices using the scalar word contract; tests of those slices do not claim
native SSE odd-address or fault parity. Actual address validity and overflow
are also outside the native static proof.

The dispatcher reads DWORD `0x142c9476c`. When its bit 0 is clear, it sets that
bit and writes 1 to selector byte `0x142c94768`, then selects SSE. Otherwise,
a zero selector byte chooses scalar and a nonzero byte chooses SSE. The
captured dispatcher contains no CPU-query instruction. Its global writes can
occur even for count zero. The Rust helper neither reads nor changes these
globals; no current native selector value is inferred.

## Bounded Rust contract

`write_rebased_indices(source, destination, index_count, cumulative_vertex_offset)`
accepts explicit byte slices, a u32 word count and a u16 offset. It returns
`Result<(), IndexTransferError>`; no native return-count convention is invented.

`MAX_INDEX_WORDS = 196_605` is an explicit engineering admission cap. It is
**not** derived from a native triangle-count limit, the 256-triangle staging
batch size, or the width of the native count. A caller may supply a longer
source or destination slice; only the first `2 * index_count` bytes participate.
Odd slice addresses, odd unused trailing byte counts and zero-length transfers
are supported by byte access, without a native SIMD-selection claim.

Admission checks run before any store, in this order: count/conversion limit,
source length, destination length. On rejection, all destination and source
bytes are unchanged. A successful transfer changes exactly the specified
destination prefix; its tail is untouched and the source remains unchanged.
After admission, each operation is infallible bounded WORD arithmetic, so no
staging allocation or partial-write rollback is needed. The function allocates
nothing and uses no unsafe code.

Atomic rejection is a Rust engineering guarantee. The native routines have no
equivalent slice/capacity checks and progressively load/store their data.
Arbitrary overlapping buffers are not a memmove profile; safe disjoint borrows
are the supported ownership condition. Cache policy, streaming-store ordering,
dispatch side effects and actual device upload remain separate work.

## Evidence and validation

The owned proprietary retail 1.52.0.1700 executable has SHA256
`8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1`.
The retained conversion record is
`scale9-output-receiver/native/rea-upload-conversions.json`,
`ev_28249ed9f242e9dc8242a9d45cb8097af9b5c97256ad6fb469ce246e4b85d6db`.
Its source file SHA256 is
`c7e917cb82c237d8594cbba795d8c9eb23bedaeb82cf9bfd051c0dce84ee4203`.
The caller is retained in
`scale9-output-receiver/native/rea-caller-and-matrix-candidate.json`,
`ev_a1cdc27adbcb11046e48df5d166eac3c0f53c40d716f2f931243532bed55d3ab`.
These paths are beneath the private project evidence directory.

The independent pre-source oracle, focused module checks and exact byte
comparison are retained under `evidence/native-index-transfer/`. The sealed
receipt records source hashes, selected native byte extents, validation results
and source/alias/alignment limits. No outside implementation or new dependency
was used. This is original project code from owned compatibility evidence,
not native runtime, rendering or complete-upload parity.

The oracle was frozen before the new Rust source was written, SHA256
`f0963cd2d9e5e2ac8fe9359f8f931424de6d347da07e9dc1ab6f7d314ab4d9e3`.
Its independent receipt is
`443802559c7da32583ced0ccc16919110ded077b51478fa498beffc14a2de500`.
It verifies 264 selected instructions / 998 bytes and excludes 12 padding
bytes. Its 33 synthetic cases contain 28 successful byte transfers and five
explicit Rust-policy rejections, covering scalar remainders, qualified vector
boundaries/prefixes/tails, odd slices, wrapping extremes, zero, the full
engineering cap, rejected counts/lengths, source preservation and destination
canaries. Expected results were not generated by the new implementation.

Three focused standalone module tests pass: literal wrapping/order/tail bytes,
actual odd slice starts with an unused odd byte, and zero/count/length rejection
atomicity. These checks introduce no module hook or product build; the integrator
owns that separate validation.

The optimized private comparison passes **100 checks across all 33 frozen
cases**, including the complete 196,605-word transfer, every source/destination
and canary byte, and all five atomic rejections. Actual odd source/destination
residues were exercised as safe scalar semantics, without a physical native
SSE-parity claim. Compile and run exit zero. The implementation source remains
SHA256 `0fa2b466e808186158312fc67e113a63eff9de17afa5d428ffd534bc047fb585`,
and the frozen oracle remains unchanged. These comparisons execute original
Rust only; they do not execute the retail client or an upload/device path.

A final coverage audit found that the immutable oracle's prose listed a
successful zero-offset case which its actual JSON did not contain; its positive
transfers also all included unused tails. The comparison retains that metadata
correction without changing the oracle. Two separate focused supplementary
checks pass for positive-count exact-length slices, one with offset zero and
one with wrapping offset `0x8001`. These are explicitly post-source checks,
not additional pre-source oracle cases. Their private receipt is
`b73436a50f490704be6aa7bb9ecb144322c218a0dff09f1fe4973c4b3d946d4e`.
