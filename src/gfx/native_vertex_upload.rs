//! Conditional whole-stride upload of explicit packed primary vertex records.
//!
//! Models the selected 032c90 scratch loop, not a device upload or live format
//! choice. Initial scratch bytes are caller inputs because the native wrapper
//! does not initialize padding before copying complete destination strides.

use super::native_primary_packing::{EXACT_PRIMARY_FORMAT, PackedFormatProfile};
use std::fmt;

pub const SCRATCH_BYTES: usize = 8192;
/// Engineering admission bounds, not native format/allocation limits.
pub const MAX_UPLOAD_VERTICES: u32 = 65_535;
pub const MAX_DESTINATION_ELEMENTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexUpload<'a> {
    pub source: PackedFormatProfile,
    pub destination_stride: u32,
    /// Ordered (offset, code) descriptors, including a code-zero terminator.
    /// Entries following the first terminator are not inspected.
    pub destination_elements: &'a [(u32, u32)],
    pub count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadError {
    UnsupportedSourceFormat,
    InvalidDestinationStride,
    TooManyVertices,
    TooManyElements,
    UnterminatedElements,
    UnsupportedElement { index: usize, code: u32 },
    ElementOutsideStride { index: usize },
    SourceTooShort,
    DestinationTooShort,
}
impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSourceFormat => {
                f.write_str("upload source is not exact primary format")
            }
            Self::InvalidDestinationStride => {
                f.write_str("upload destination stride must be 1..8192")
            }
            Self::TooManyVertices => f.write_str("upload vertex admission limit exceeded"),
            Self::TooManyElements => f.write_str("upload destination element limit exceeded"),
            Self::UnterminatedElements => {
                f.write_str("upload destination has no code-zero terminator")
            }
            Self::UnsupportedElement { index, code } => {
                write!(f, "unsupported upload element {index} with code {code:#x}")
            }
            Self::ElementOutsideStride { index } => {
                write!(f, "upload element {index} exceeds its destination stride")
            }
            Self::SourceTooShort => f.write_str("upload source does not hold all packed records"),
            Self::DestinationTooShort => {
                f.write_str("upload destination does not hold all strides")
            }
        }
    }
}
impl std::error::Error for UploadError {}

#[derive(Clone, Copy)]
enum Operation {
    Skip,
    Copy {
        source: usize,
        destination: usize,
        width: usize,
    },
    Short3 {
        destination: usize,
    },
    Float2 {
        destination: usize,
    },
}

fn plan(
    transfer: VertexUpload<'_>,
) -> Result<([Operation; MAX_DESTINATION_ELEMENTS], usize), UploadError> {
    let mut operations = [Operation::Skip; MAX_DESTINATION_ELEMENTS];
    for (index, &(offset, code)) in transfer.destination_elements.iter().enumerate() {
        if code == 0 {
            return Ok((operations, index));
        }
        if index == MAX_DESTINATION_ELEMENTS {
            return Err(UploadError::TooManyElements);
        }
        // Native branches around source lookup and all stores for these scalar
        // kinds. Their offsets are not dereferenced, including out-of-stride ones.
        if matches!(code & 0xf0, 0x70 | 0x80) {
            continue;
        }
        let destination = offset as usize;
        let (operation, width) = match code {
            0x132 => (
                Operation::Copy {
                    source: 0,
                    destination,
                    width: 4,
                },
                4,
            ),
            0x133 => (Operation::Short3 { destination }, 6),
            0x162 => (Operation::Float2 { destination }, 8),
            0x251 => (
                Operation::Copy {
                    source: 4,
                    destination,
                    width: 4,
                },
                4,
            ),
            0x1211 => (
                Operation::Copy {
                    source: 8,
                    destination,
                    width: 1,
                },
                1,
            ),
            0x2211 => (
                Operation::Copy {
                    source: 9,
                    destination,
                    width: 1,
                },
                1,
            ),
            _ => return Err(UploadError::UnsupportedElement { index, code }),
        };
        if offset
            .checked_add(width)
            .is_none_or(|end| end > transfer.destination_stride)
        {
            return Err(UploadError::ElementOutsideStride { index });
        }
        operations[index] = operation;
    }
    Err(UploadError::UnterminatedElements)
}

/// Convert admitted elements in descriptor order into one reused scratch buffer,
/// then copy every byte of each destination stride. Overlapping destination
/// elements retain that ordering; absent/skipped fields retain scratch bytes.
/// The destination tail is untouched. Source records are immutable.
///
/// The caller supplies initial scratch contents; this function returns their
/// exact final state through the mutable borrow. No zero-padding, source-padding
/// copy, prior-destination-padding preservation or fresh scratch per chunk is
/// inferred. The stride determines native chunk capacity: floor(8192 / stride).
///
/// All admission checks precede either output mutation, including for count zero.
/// This failure-atomic policy is a Rust engineering choice, not native failure
/// behavior. Exact source/profile association, nonoverlapping live storage and
/// ordinary successful memory operations remain explicit premises. Callback
/// auxiliary state is unused by the admitted position/raw-copy/skip routes.
pub fn transfer_primary_vertices(
    source: &[u8],
    destination: &mut [u8],
    scratch: &mut [u8; SCRATCH_BYTES],
    transfer: VertexUpload<'_>,
) -> Result<(), UploadError> {
    if transfer.source != EXACT_PRIMARY_FORMAT {
        return Err(UploadError::UnsupportedSourceFormat);
    }
    if transfer.destination_stride == 0 || transfer.destination_stride > SCRATCH_BYTES as u32 {
        return Err(UploadError::InvalidDestinationStride);
    }
    if transfer.count > MAX_UPLOAD_VERTICES {
        return Err(UploadError::TooManyVertices);
    }
    let (operations, operation_count) = plan(transfer)?;
    let count = transfer.count as usize;
    let stride = transfer.destination_stride as usize;
    // The admitted count/stride bounds keep both complete spans below 2^32;
    // every per-chunk native u32 stride product therefore also remains exact.
    if source.len() < count * 12 {
        return Err(UploadError::SourceTooShort);
    }
    if destination.len() < count * stride {
        return Err(UploadError::DestinationTooShort);
    }
    let capacity = SCRATCH_BYTES / stride;
    let mut completed = 0;
    while completed < count {
        let chunk = capacity.min(count - completed);
        for &operation in &operations[..operation_count] {
            if matches!(operation, Operation::Skip) {
                continue;
            }
            for vertex in 0..chunk {
                let input = (completed + vertex) * 12;
                let output = vertex * stride;
                match operation {
                    Operation::Skip => {}
                    Operation::Copy {
                        source: offset,
                        destination,
                        width,
                    } => {
                        scratch[output + destination..output + destination + width]
                            .copy_from_slice(&source[input + offset..input + offset + width]);
                    }
                    Operation::Short3 { destination } => {
                        scratch[output + destination..output + destination + 4]
                            .copy_from_slice(&source[input..input + 4]);
                        scratch[output + destination + 4..output + destination + 6].fill(0);
                    }
                    Operation::Float2 { destination } => {
                        let x = i16::from_le_bytes([source[input], source[input + 1]]) as f32;
                        let y = i16::from_le_bytes([source[input + 2], source[input + 3]]) as f32;
                        scratch[output + destination..output + destination + 4]
                            .copy_from_slice(&x.to_le_bytes());
                        scratch[output + destination + 4..output + destination + 8]
                            .copy_from_slice(&y.to_le_bytes());
                    }
                }
            }
        }
        let bytes = chunk * stride;
        destination[completed * stride..completed * stride + bytes]
            .copy_from_slice(&scratch[..bytes]);
        completed += chunk;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(elements: &[(u32, u32)], stride: u32, count: u32) -> VertexUpload<'_> {
        VertexUpload {
            source: EXACT_PRIMARY_FORMAT,
            destination_stride: stride,
            destination_elements: elements,
            count,
        }
    }

    #[test]
    fn whole_stride_uses_explicit_scratch_padding_and_retains_tail() {
        let source = [0, 0x80, 0xff, 0x7f, 1, 2, 3, 4, 0x7f, 0xff, 0x11, 0x22];
        let mut scratch = [0x5a; SCRATCH_BYTES];
        let mut output = [0xcc; 19];
        let elements = [(0, 0x162), (8, 0x251), (12, 0x1211), (13, 0x2211), (0, 0)];
        transfer_primary_vertices(
            &source,
            &mut output,
            &mut scratch,
            request(&elements, 16, 1),
        )
        .unwrap();
        assert_eq!(
            output,
            [
                0, 0, 0, 0xc7, 0, 0xfe, 0xff, 0x46, 1, 2, 3, 4, 0x7f, 0xff, 0x5a, 0x5a, 0xcc, 0xcc,
                0xcc
            ]
        );
        assert_eq!(&scratch[..16], &output[..16]);
        assert!(scratch[16..].iter().all(|&byte| byte == 0x5a));
    }

    #[test]
    fn descriptor_order_overlap_skip_and_first_terminator_are_preserved() {
        let source = [0x12, 0x34, 0x56, 0x78, 1, 2, 3, 4, 0xab, 0xcd, 0, 0];
        let mut scratch = [0x5a; SCRATCH_BYTES];
        let mut output = [0xcc; 8];
        let elements = [
            (0, 0x133),
            (2, 0x251),
            (3, 0x1211),
            (u32::MAX, 0x170),
            (0, 0),
            (0, 0xffffffff),
        ];
        transfer_primary_vertices(&source, &mut output, &mut scratch, request(&elements, 8, 1))
            .unwrap();
        assert_eq!(output, [0x12, 0x34, 1, 0xab, 3, 4, 0x5a, 0x5a]);
    }

    #[test]
    fn late_admission_failure_and_zero_count_leave_both_outputs_unchanged() {
        let mut scratch = [0x5a; SCRATCH_BYTES];
        let mut output = [0xcc; 16];
        let bad = [(0, 0x132), (8, 0x251), (15, 0x162), (0, 0)];
        assert_eq!(
            transfer_primary_vertices(&[0; 12], &mut output, &mut scratch, request(&bad, 16, 1)),
            Err(UploadError::ElementOutsideStride { index: 2 })
        );
        assert_eq!(output, [0xcc; 16]);
        assert!(scratch.iter().all(|&byte| byte == 0x5a));
        let elements = [(0, 0x132), (0, 0)];
        assert_eq!(
            transfer_primary_vertices(&[], &mut output, &mut scratch, request(&elements, 4, 1)),
            Err(UploadError::SourceTooShort)
        );
        transfer_primary_vertices(&[], &mut output, &mut scratch, request(&elements, 4, 0))
            .unwrap();
        assert_eq!(output, [0xcc; 16]);
        assert!(scratch.iter().all(|&byte| byte == 0x5a));
    }
}
