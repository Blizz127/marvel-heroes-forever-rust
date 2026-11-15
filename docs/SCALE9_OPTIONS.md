# Native primary options and mesh flags

**VERIFIED static dataflow:** the ordinary primary adapter samples a 72-byte
option block from its supplied render object and selects antialias width using
the mesh's supplied flags. The recovered constructor writes nonzero tolerance
and width. The zero-tolerance, non-AA rectangle profile in
[SCALE9_TESSELLATION.md](SCALE9_TESSELLATION.md) therefore cannot stand in for
these constructor values. Neither constructor state nor an observed source
shape proves current renderer settings.

This increment adds evidence only. No rendering path, preset, default option,
native execution or runtime state is changed.

## Sampled block and constructor checkpoint

Let M be the mesh context and R its `+0x50` pointer. Complete `0x1400c4dd0`
copies `R+0xcd8..0xd1f` into a 72-byte stack block before its later calls.
The block uses four unaligned 16-byte loads followed by one eight-byte load;
it is a snapshot, not a retained pointer. Its primary workspace is `R+0x58`
and the primary tessellator is workspace `+0xa0`.

Complete constructor `0x140071960` writes all eighteen block words in order
at `0x140071a25..0x140071acf`. At the checkpoint immediately after those
stores, these selected values are established:

| Block offset | R offset | Bits | Stored binary32 value | Selected use |
| --- | --- | --- | --- | --- |
| `+0` | `+0xcd8` | `3727c5ac` | `0.000009999999747378752` | Relative tolerance |
| `+0x0c` | `+0xce4` | `3a83126f` | `0.0010000000474974513` | Copied to primary `+0x3b4`; other consumers remain separate |
| `+0x38` | `+0xd10` | `3f733333` | `0.949999988079071` | Width used by the antialias selection below |

The private receipt retains the exact eighteen-word inventory. This document
does not assign unproved SDK names to the other words. Later constructor calls,
setters, aliases, actual object identity and preservation until sampling remain
outside this checkpoint claim. The primary context's `R` identity is supplied;
matching an offset alone does not prove that a live object used this constructor.

At `0x1400c5192` the adapter sets primary mode to 1. Its following `fed20`
call stores block word 0 in primary `+0x1c`, sets primary byte `+0x20` iff
that value compares greater than zero, and copies block word `+0x0c` into
primary `+0x3b4`. Both complete return branches are retained. Floating-point
exception behavior for signaling NaNs and the runtime floating-point control
state are not reconstructed here.

## Width selection and large-source retry

For the original incoming flag word F, the ordinary adapter selects:

| Condition | Value passed to `0x1400fed60` |
| --- | --- |
| `(F & 3) == 1` | binary32 `block[+0x38] * 0.5` |
| Otherwise | positive zero |

The multiplier's actual constant bits are `3f000000`. With the constructor
width and ordinary round-to-nearest arithmetic, the product is `3ef33333`,
or `0.4749999940395355`. This is a conditional arithmetic result, not a live
width observation. `fed60` stores the supplied value in primary `+0x3ac`
and sets byte `+0x3b0` iff it compares greater than zero.

After the first `c3d30` source submission, if original F bit0 is set and
the dword at workspace `+0x118` is at least `0x8000`, the adapter invokes
`bdbd0`, writes a zero AA width, resubmits the source and clears flag bit0.
The guard tests original bit0 even when bit1 had already selected zero width.
This is a second submission with reset effects; a future adapter must not
merely change a final render flag. The full reset behavior and large-source
geometry are separate from this control-flow result.

## Origin of the flag word

The ordinary receiver route in
[SCALE9_OUTPUT_RECEIVER.md](SCALE9_OUTPUT_RECEIVER.md) reads dword `M+0x98`
and passes it as provider argument R9D. Concrete provider `c5d50` forwards it
unchanged to the ordinary adapter's sixth argument. Cache-key flags and this
mesh flag word are separate values.

Complete base constructor `03bda0` writes its seventh argument directly to
`M+0x98`, after retaining the supplied provider through virtual `+8`.
It stores supplied R at `M+0x50`, provider at `+0x58`, eight matrix words
at `+0x70`, the supplied float at `+0x90` and sixth argument at `+0x94`.
Ordinary constructor `03bf20` forwards those arguments without changing the
flag word, then initializes its ordinary receiver/output fields.

In the complete selected caller `0b0f10`, its original sixth argument is saved
before cache work and passed as the new context's seventh argument. Separately,
that caller maps the same input's low two bits to cache-key bits6/7 and ORs
them with `afef0`'s result. The combined cache key does not replace the original
flag word passed to `03bf20`. On successful allocation the returned context
is stored into the selected cache entry `+0x18`; that cache entry is subsequently
stored at the caller's source-object `+0xb8`.

The proof covers the new ordinary allocation branch. Existing-cache reuse,
the alternate `03d450` object, arbitrary provider retain callbacks, actual
caller flag selection and subsequent mutation are not inferred from it.
The constructor `+0x98` write occurs after its retain callback, but that alone
does not prove all supplied objects remain valid across the callback.

## Evidence and provenance

Private root: `evidence/scale9-options/`. `verification.json` SHA-256:
`feeae355f0beb2b2c21a47353fd0e334b7a01f80294121dccd9457445ec41808`.

The original validator checks selected canonical instruction/data arrays
against the pinned PE and independently decodes the eighteen constructor
immediates. Its totals are 1,196 instruction rows / 5,111 instruction bytes and
200 data bytes, including a duplicated 231-byte constructor capture. That new
capture agrees with a previously retained record and adds no body coverage.
The primary-input selection includes padding and adjacent leaf routines;
unselected neighboring bodies are not claimed complete.

| Source | Licence / ownership | Use |
| --- | --- | --- |
| Owner's `1.52.0.1700` executable, SHA-256 `8b7212d6376c9f86b0180efb34448935b7c6a3e8dee090a8ab24385c23392fc1` | Proprietary private owner installation | Constructor, sampled options, flag forwarding and constant evidence |
| Retained `scale9-output`, `scale9-geometry`, `scale9-rendering` and `scale9-output-receiver` canonical records, individually hashed in the receipt and linked receiver ledger | Original investigation of that owner artifact | Reuse complete constructors, adapters and exact selected leaves |
| REA 6.9.0 / Rizin 0.9.1 and previously recorded skill/guides | MIT / LGPL-3.0 tooling | Two bounded target-free reads, export and reset |
| Python standard library | PSF tooling | Original byte/hash/PE and constructor-immediate validator |

New records `ev_c46f625c9cd65ab7e1e5a561c19e5f230075dffee402080337706af7d1b73058`
and `ev_e6d290e4ab6beebf884e834d9ea8bca6fb22546d565684da5efcb554dcf8de10`
are preserved in `final.bundle.json`; reset retains no target or records.
Both completed with exit zero, untruncated output and unchanged artifact hash.
Bounded raw-byte candidate scans are retained as discovery aids, not proof of
instruction alignment or absence of other callers/setters. No external renderer
implementation, binary or extracted native instruction listing enters git.
