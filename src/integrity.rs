//! Verify the exact owned bytes used by a private source/profile parser.
//! Expected identities come from the caller's pinned source ledger.

use sha2::{Digest, Sha256};
use std::io::{self, Read};

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Read one complete finite source, with explicit size and byte budget, and
/// return exactly the buffer whose digest matched. This does not validate the
/// provenance of an expected digest or make claims about a later filesystem read.
/// Reads at most expected_size + 1 bytes; the extra byte detects trailing data.
pub fn read_verified(
    source: impl Read,
    expected_size: u64,
    expected_sha256: [u8; 32],
    max_bytes: usize,
) -> io::Result<Vec<u8>> {
    let length = usize::try_from(expected_size)
        .map_err(|_| invalid("pinned source size does not fit this platform"))?;
    if length > max_bytes {
        return Err(invalid("pinned source exceeds its byte budget"));
    }
    let cap = expected_size
        .checked_add(1)
        .ok_or_else(|| invalid("pinned source size overflow"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| io::Error::other("pinned source allocation failed"))?;
    source.take(cap).read_to_end(&mut bytes)?;
    if bytes.len() != length {
        return Err(invalid("pinned source size differs from expectation"));
    }
    if sha256(&bytes) != expected_sha256 {
        return Err(invalid("pinned source SHA-256 differs from expectation"));
    }
    Ok(bytes)
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn digest_and_size_bind_the_returned_parser_buffer() {
        let source = b"private inventory fixture";
        let digest = sha256(source);
        assert_eq!(
            read_verified(
                Cursor::new(source),
                source.len() as u64,
                digest,
                source.len()
            )
            .unwrap(),
            source
        );
        let mut changed = source.to_vec();
        changed[0] ^= 1;
        assert!(
            read_verified(
                Cursor::new(changed),
                source.len() as u64,
                digest,
                source.len()
            )
            .is_err()
        );
        assert!(
            read_verified(
                Cursor::new(source),
                source.len() as u64 - 1,
                digest,
                source.len()
            )
            .is_err()
        );
        assert!(
            read_verified(
                Cursor::new(source),
                source.len() as u64 + 1,
                digest,
                source.len() + 1
            )
            .is_err()
        );
    }

    #[test]
    fn budget_rejects_before_source_access_and_extra_bytes_are_bounded() {
        struct Forbidden;
        impl Read for Forbidden {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("budget must reject before reading");
            }
        }
        assert!(read_verified(Forbidden, 100, [0; 32], 99).is_err());
        let mut source = Cursor::new([7u8; 4096]);
        assert!(read_verified(&mut source, 8, sha256(&[7; 8]), 8).is_err());
        assert_eq!(source.position(), 9);
        assert_eq!(
            read_verified(Cursor::new([]), 0, sha256(&[]), 0).unwrap(),
            Vec::<u8>::new()
        );
    }
}
