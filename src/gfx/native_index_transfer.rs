//! Bounded little-endian index-word rebasing from the native upload contract.
//!
//! This implements scalar byte semantics for disjoint slices. It does not model
//! native global dispatch, physical alignment, streaming stores or fault behavior.

use std::fmt;

/// Explicit engineering limit, not a native triangle/count restriction.
pub const MAX_INDEX_WORDS: usize = 196_605;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexTransferError {
    IndexLimit,
    SourceTooShort { required: usize, available: usize },
    DestinationTooShort { required: usize, available: usize },
}

impl fmt::Display for IndexTransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexLimit => write!(f, "index transfer exceeds {MAX_INDEX_WORDS} words"),
            Self::SourceTooShort {
                required,
                available,
            } => write!(
                f,
                "index source needs {required} bytes but contains {available}"
            ),
            Self::DestinationTooShort {
                required,
                available,
            } => write!(
                f,
                "index destination needs {required} bytes but contains {available}"
            ),
        }
    }
}

impl std::error::Error for IndexTransferError {}

/// Add the supplied offset to each source WORD, wrapping modulo 65,536.
///
/// `index_count` counts words. Exactly its first `2 * index_count` destination
/// bytes are written; source and destination tails may have any length. Odd
/// byte addresses are supported by scalar byte access, without claiming native
/// SSE alignment or dispatch equivalence. No allocation occurs.
///
/// Count, source length and destination length are checked in that order before
/// any write. Every error therefore preserves the destination. Safe disjoint
/// borrows are the supported aliasing profile; native overlapping-buffer effects
/// and progressive memory faults are not emulated.
pub fn write_rebased_indices(
    source: &[u8],
    destination: &mut [u8],
    index_count: u32,
    cumulative_vertex_offset: u16,
) -> Result<(), IndexTransferError> {
    let count = usize::try_from(index_count).map_err(|_| IndexTransferError::IndexLimit)?;
    if count > MAX_INDEX_WORDS {
        return Err(IndexTransferError::IndexLimit);
    }
    let required = count.checked_mul(2).ok_or(IndexTransferError::IndexLimit)?;
    if source.len() < required {
        return Err(IndexTransferError::SourceTooShort {
            required,
            available: source.len(),
        });
    }
    if destination.len() < required {
        return Err(IndexTransferError::DestinationTooShort {
            required,
            available: destination.len(),
        });
    }

    for (input, output) in source[..required]
        .chunks_exact(2)
        .zip(destination[..required].chunks_exact_mut(2))
    {
        let value = u16::from_le_bytes([input[0], input[1]]);
        output.copy_from_slice(&value.wrapping_add(cumulative_vertex_offset).to_le_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_words_keep_order_source_and_destination_tail() {
        let source = [
            0x00, 0x00, 0x01, 0x00, 0xff, 0x7f, 0x00, 0x80, 0xff, 0xff, 0x34, 0x12, 0xcd, 0xab,
            0xe7,
        ];
        let before = source;
        let mut destination = [0x5a; 17];
        write_rebased_indices(&source, &mut destination, 7, 1).unwrap();
        assert_eq!(
            destination,
            [
                0x01, 0x00, 0x02, 0x00, 0x00, 0x80, 0x01, 0x80, 0x00, 0x00, 0x35, 0x12, 0xce, 0xab,
                0x5a, 0x5a, 0x5a
            ]
        );
        assert_eq!(source, before);
    }

    #[test]
    fn odd_slice_starts_and_unused_odd_byte_are_supported() {
        // Select actual odd addresses without assuming the enclosing stack
        // arrays' alignment. This tests safe byte semantics, not native SIMD.
        let mut source_storage = [0xee; 16];
        let source_start = usize::from(source_storage.as_ptr() as usize & 1 == 0);
        source_storage[source_start..source_start + 7]
            .copy_from_slice(&[0x00, 0x00, 0xff, 0xff, 0x34, 0x12, 0xd7]);
        let source_before = source_storage;
        let mut destination_storage = [0xa5; 18];
        let destination_start = usize::from(destination_storage.as_ptr() as usize & 1 == 0);
        assert_eq!((source_storage.as_ptr() as usize + source_start) & 1, 1);
        assert_eq!(
            (destination_storage.as_ptr() as usize + destination_start) & 1,
            1
        );

        write_rebased_indices(
            &source_storage[source_start..source_start + 7],
            &mut destination_storage[destination_start..destination_start + 9],
            3,
            0xffff,
        )
        .unwrap();
        let mut expected = [0xa5; 18];
        expected[destination_start..destination_start + 6]
            .copy_from_slice(&[0xff, 0xff, 0xfe, 0xff, 0x33, 0x12]);
        assert_eq!(destination_storage, expected);
        assert_eq!(source_storage, source_before);
    }

    #[test]
    fn zero_and_all_rejections_preserve_destination() {
        let mut destination = [0x36; 9];
        let before = destination;
        assert_eq!(
            write_rebased_indices(&[], &mut destination, 0, 0xffff),
            Ok(())
        );
        assert_eq!(destination, before);
        assert_eq!(
            write_rebased_indices(&[], &mut destination, u32::MAX, 1),
            Err(IndexTransferError::IndexLimit)
        );
        assert_eq!(
            write_rebased_indices(&[], &mut destination, MAX_INDEX_WORDS as u32 + 1, 1),
            Err(IndexTransferError::IndexLimit)
        );
        assert_eq!(
            write_rebased_indices(&[1], &mut destination, 5, 0),
            Err(IndexTransferError::SourceTooShort {
                required: 10,
                available: 1
            })
        );
        assert_eq!(
            write_rebased_indices(&[0x12; 10], &mut destination, 5, 0),
            Err(IndexTransferError::DestinationTooShort {
                required: 10,
                available: 9
            })
        );
        assert_eq!(destination, before);
    }
}
