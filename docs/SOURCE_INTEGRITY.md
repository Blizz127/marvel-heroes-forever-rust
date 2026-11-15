# Private source integrity

`integrity::sha256` hashes the exact byte slice supplied to a parser verifier.
`read_verified` checks an explicit size and byte budget, reads at most the expected
size plus one trailing-data probe byte, and returns only the buffer whose size
and SHA-256 matched. The budget is a content limit, not an allocator/RSS claim.
The caller supplies a finite source and its trusted expected identity; later
filesystem reads are not covered by this check.

Native-name inventories remain private data. Their parser's verifier callback
can use this digest function on the precise buffer it receives, avoiding a
disconnected claimed digest for some other read. This does not establish native
semantics or trust an arbitrary caller-supplied expected identity.

The linked primitive is RustCrypto `sha2 = 0.10.9`, MIT OR Apache-2.0. Project
buffer admission and verification glue is original code; no hashing implementation
was copied into project sources. Sources consulted: the cached crate's manifest,
README and both licence files, and its
[primary versioned API documentation](https://docs.rs/sha2/0.10.9/sha2/).
Their private hashes are in `evidence/source-integrity/source-ledger.json`.
Cargo.lock pins the dependency graph; private build notices must include it.
