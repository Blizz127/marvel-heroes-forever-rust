//! Bounded position-element transfers recovered from the owned client.
//!
//! This handles source code 0x132 and exact destination codes 0x132, 0x133,
//! and 0x162. It does not choose a runtime vertex format, construct a GPU
//! declaration, copy other vertex elements, or identify a coordinate space.
//! See docs/SCALE9_PACKED_VERTEX_FORMAT.md for native evidence and limits.

use std::fmt;

/// Engineering admission limit; not a claim about the native allocator's limit.
pub const MAX_POSITION_COUNT: u32 = 65_535;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementLayout {
    pub code: u32,
    pub stride: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionTransfer {
    pub source: ElementLayout,
    pub destination: ElementLayout,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferError {
    UnsupportedSource(u32),
    UnsupportedDestination(u32),
    TooManyPositions,
    InvalidLayout(&'static str),
    NativeSpanOverflow(&'static str),
    BufferTooShort(&'static str),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSource(code) => {
                write!(f, "unsupported native source position code {code:#x}")
            }
            Self::UnsupportedDestination(code) => {
                write!(f, "unsupported native destination position code {code:#x}")
            }
            Self::TooManyPositions => f.write_str("position transfer admission limit exceeded"),
            Self::InvalidLayout(side) => write!(f, "invalid {side} position layout"),
            Self::NativeSpanOverflow(side) => write!(f, "{side} native 32-bit span overflows"),
            Self::BufferTooShort(side) => write!(f, "{side} position buffer is too short"),
        }
    }
}
impl std::error::Error for TransferError {}

fn check_layout(
    layout: ElementLayout,
    width: u32,
    count: u32,
    buffer_len: usize,
    side: &'static str,
) -> Result<(), TransferError> {
    let end = layout
        .offset
        .checked_add(width)
        .ok_or(TransferError::InvalidLayout(side))?;
    if layout.stride == 0 || end > layout.stride {
        return Err(TransferError::InvalidLayout(side));
    }
    let total_span = layout
        .stride
        .checked_mul(count)
        .ok_or(TransferError::NativeSpanOverflow(side))?;
    if count != 0 {
        // The validated layout guarantees this is <= total_span. Only the
        // actual last element must be backed; trailing padding is never read.
        let required = total_span - layout.stride + end;
        if required as usize > buffer_len {
            return Err(TransferError::BufferTooShort(side));
        }
    }
    Ok(())
}

/// Transfer only the admitted position element between disjoint byte slices.
///
/// 0x132 -> 0x132 copies the two words. 0x132 -> 0x133 preserves those words
/// and writes a zero third word. 0x132 -> 0x162 sign-extends each little-endian
/// i16 and writes its exact IEEE binary32 value, with no normalization.
///
/// All checks precede writes, so errors leave the entire destination unchanged.
/// Valid safe Rust borrows enforce the non-overlap profile. Native overlapping
/// transfers and arbitrary flag-bearing format aliases are not emulated.
/// Zero count performs no accesses after validating formats and layouts.
pub fn transfer_positions(
    source: &[u8],
    destination: &mut [u8],
    transfer: PositionTransfer,
) -> Result<(), TransferError> {
    if transfer.source.code != 0x132 {
        return Err(TransferError::UnsupportedSource(transfer.source.code));
    }
    let width = match transfer.destination.code {
        0x132 => 4,
        0x133 => 6,
        0x162 => 8,
        code => return Err(TransferError::UnsupportedDestination(code)),
    };
    if transfer.count > MAX_POSITION_COUNT {
        return Err(TransferError::TooManyPositions);
    }
    check_layout(transfer.source, 4, transfer.count, source.len(), "source")?;
    check_layout(
        transfer.destination,
        width,
        transfer.count,
        destination.len(),
        "destination",
    )?;
    for index in 0..transfer.count {
        let src = (index * transfer.source.stride + transfer.source.offset) as usize;
        let dst = (index * transfer.destination.stride + transfer.destination.offset) as usize;
        if transfer.destination.code == 0x162 {
            let x = i16::from_le_bytes([source[src], source[src + 1]]) as f32;
            let y = i16::from_le_bytes([source[src + 2], source[src + 3]]) as f32;
            destination[dst..dst + 4].copy_from_slice(&x.to_le_bytes());
            destination[dst + 4..dst + 8].copy_from_slice(&y.to_le_bytes());
        } else {
            destination[dst..dst + 4].copy_from_slice(&source[src..src + 4]);
            if transfer.destination.code == 0x133 {
                destination[dst + 4..dst + 6].fill(0);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(destination_code: u32, destination_stride: u32, count: u32) -> PositionTransfer {
        PositionTransfer {
            source: ElementLayout {
                code: 0x132,
                stride: 4,
                offset: 0,
            },
            destination: ElementLayout {
                code: destination_code,
                stride: destination_stride,
                offset: 0,
            },
            count,
        }
    }

    #[test]
    fn signed_extrema_and_unit_words_become_exact_float_bits() {
        let source = [
            0x00, 0x80, 0xff, 0x7f, 0xff, 0xff, 0x00, 0x00, 0x01, 0x00, 0xfe, 0xff,
        ];
        let mut destination = [0xa5; 24];
        transfer_positions(&source, &mut destination, transfer(0x162, 8, 3)).unwrap();
        let expected = [
            0xc700_0000u32,
            0x46ff_fe00,
            0xbf80_0000,
            0,
            0x3f80_0000,
            0xc000_0000,
        ];
        for (bytes, bits) in destination.chunks_exact(4).zip(expected) {
            assert_eq!(bytes, bits.to_le_bytes());
        }
    }

    #[test]
    fn unaligned_short3_transfer_preserves_other_elements_and_padding() {
        let source = [
            0xa5, 0x00, 0x80, 0xff, 0x7f, 0x33, 0x44, 0x55, 0x66, 0xa5, 0xff, 0xff, 0x01, 0x00,
        ];
        let mut destination = [0xcc; 20];
        let plan = PositionTransfer {
            source: ElementLayout {
                code: 0x132,
                stride: 9,
                offset: 1,
            },
            destination: ElementLayout {
                code: 0x133,
                stride: 11,
                offset: 3,
            },
            count: 2,
        };
        transfer_positions(&source, &mut destination, plan).unwrap();
        assert_eq!(
            destination,
            [
                0xcc, 0xcc, 0xcc, 0x00, 0x80, 0xff, 0x7f, 0, 0, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xff,
                0xff, 1, 0, 0, 0
            ]
        );
        assert_eq!(source[5..9], [0x33, 0x44, 0x55, 0x66]);
    }

    #[test]
    fn identical_code_copies_words_without_initializing_extra_bytes() {
        let mut destination = [0x5a; 9];
        let mut plan = transfer(0x132, 9, 1);
        plan.destination.offset = 3;
        transfer_positions(&[0x34, 0x92, 0xcd, 0xab], &mut destination, plan).unwrap();
        assert_eq!(
            destination,
            [0x5a, 0x5a, 0x5a, 0x34, 0x92, 0xcd, 0xab, 0x5a, 0x5a]
        );
    }

    #[test]
    fn invalid_profiles_fail_before_any_destination_write() {
        let base = transfer(0x162, 8, 1);
        let mut cases = vec![base; 8];
        cases[0].source.code = 0x162;
        cases[1].destination.code = 0x1_0162;
        cases[2].source.offset = 1;
        cases[3].destination.stride = 7;
        cases[4].count = MAX_POSITION_COUNT + 1;
        cases[5].source.stride = u32::MAX;
        cases[5].count = 2;
        cases[6].count = 2; // Insufficient source.
        cases[7].destination.offset = 1;
        cases[7].destination.stride = 9; // Insufficient destination.
        for plan in cases {
            let mut destination = [0xa5; 8];
            assert!(transfer_positions(&[0, 0, 0, 0], &mut destination, plan).is_err());
            assert_eq!(destination, [0xa5; 8]);
        }
    }

    #[test]
    fn zero_count_requires_valid_profile_but_no_backing_bytes() {
        let mut plan = transfer(0x162, 16, 0);
        plan.source.stride = 12;
        plan.source.offset = 7;
        plan.destination.offset = 3;
        transfer_positions(&[], &mut [], plan).unwrap();
        plan.destination.code = 0x163;
        assert_eq!(
            transfer_positions(&[], &mut [], plan),
            Err(TransferError::UnsupportedDestination(0x163))
        );
    }
}
