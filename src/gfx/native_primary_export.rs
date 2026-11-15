//! Native primary descriptor copy/reset and generated-vertex export.
//!
//! The selected 100020/100090 bodies copy integer words without geometry or
//! floating-point arithmetic. Source positions must already be in the caller's
//! desired stage. See docs/SCALE9_PRIMARY_VERTEX_EXPORT.md for the exact scope.

use super::native_scale9::MAX_PRIMARY_OUTPUT_VERTICES;
use super::native_scale9_tessellation::GeneratedVertex;
use std::fmt;

/// Maximum requested by the recovered output caller, not a limit in 100090.
pub const MAX_VERTEX_BATCH: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportError {
    VertexLimit,
    BatchLimit,
    AllocationFailed,
}
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::VertexLimit => "primary export source vertex limit exceeded",
            Self::BatchLimit => "primary vertex export batch exceeds 256",
            Self::AllocationFailed => "cannot reserve primary vertex export storage",
        })
    }
}
impl std::error::Error for ExportError {}

/// Seven native descriptor words. Word0 is the full u32 mesh comparison value;
/// word5 (+0x14) is the global generated-vertex scan cursor, not an output count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexExportCursor {
    words: [u32; 7],
}
impl VertexExportCursor {
    /// The already-selected descriptor's 100020 copy, resetting only word5.
    /// Selection from the native descriptor pages is the caller's responsibility.
    pub fn copy_descriptor(mut descriptor: [u32; 7]) -> Self {
        descriptor[5] = 0;
        Self { words: descriptor }
    }

    /// Resume an explicit snapshot without resetting or clamping its cursor.
    /// Out-of-range cursor values are valid exhausted snapshots for this method.
    pub fn from_snapshot(words: [u32; 7]) -> Self {
        Self { words }
    }

    pub fn snapshot(&self) -> [u32; 7] {
        self.words
    }

    /// Copy matching generated records in scan order, up to `requested` records.
    /// Every visited record advances word5, including nonmatching records. The
    /// descriptor ID is not narrowed: values above u16::MAX match no vertex.
    /// A descriptor value of 0xffff can match that exact vertex mesh value.
    ///
    /// This models the concatenation of native 16-record pages. It does not
    /// assign ordinals, sort, normalize, pack coordinates or interpret flags.
    /// All five record words survive unchanged, including nonfinite float bits.
    /// Source stability, descriptor provenance and invocation order are caller
    /// inputs, not established by this transfer.
    ///
    /// Inspection caps and allocation failures reject without cursor mutation.
    /// These are Rust admission rules, not native invalid-memory/failure behavior.
    pub fn export_batch(
        &mut self,
        all_vertices: &[GeneratedVertex],
        requested: usize,
    ) -> Result<Vec<GeneratedVertex>, ExportError> {
        if all_vertices.len() > MAX_PRIMARY_OUTPUT_VERTICES {
            return Err(ExportError::VertexLimit);
        }
        if requested > MAX_VERTEX_BATCH {
            return Err(ExportError::BatchLimit);
        }
        let mut next = self.words[5] as usize;
        let mut output = Vec::new();
        output
            .try_reserve_exact(requested.min(all_vertices.len().saturating_sub(next)))
            .map_err(|_| ExportError::AllocationFailed)?;
        while output.len() < requested && next < all_vertices.len() {
            let vertex = all_vertices[next];
            next += 1;
            if u32::from(vertex.mesh) == self.words[0] {
                output.push(vertex);
            }
        }
        // An initial cursor is u32 and any scanned position is bounded above by
        // the source cap. No cursor increment can wrap within this admission.
        self.words[5] = next as u32;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(mesh: u16, ordinal: u32) -> GeneratedVertex {
        GeneratedVertex {
            position: [f32::from_bits(0x7fc1_2345), f32::from_bits(0x8000_0000)],
            local_ordinal: ordinal,
            styles: [0xabcd, 0x1234],
            flags: 0x89ab,
            mesh,
        }
    }
    fn words(v: GeneratedVertex) -> [u32; 5] {
        [
            v.position[0].to_bits(),
            v.position[1].to_bits(),
            v.local_ordinal,
            u32::from(v.styles[0]) | (u32::from(v.styles[1]) << 16),
            u32::from(v.flags) | (u32::from(v.mesh) << 16),
        ]
    }

    #[test]
    fn descriptor_copy_resets_only_scan_cursor() {
        let source = [17, 0x7654_3210, 2, 3, 4, u32::MAX, 999];
        let mut copied = source;
        copied[5] = 0;
        assert_eq!(
            VertexExportCursor::copy_descriptor(source).snapshot(),
            copied
        );
        assert_eq!(VertexExportCursor::from_snapshot(source).snapshot(), source);
    }

    #[test]
    fn partial_batches_preserve_bits_order_and_skipped_record_progress() {
        let source = [
            vertex(2, 0),
            vertex(7, 99),
            vertex(2, 0),
            vertex(7, 3),
            vertex(2, 0),
        ];
        let mut cursor = VertexExportCursor::copy_descriptor([7, 31, 32, 33, 34, 0, 35]);
        let first = cursor.export_batch(&source, 1).unwrap();
        assert_eq!(
            first.iter().copied().map(words).collect::<Vec<_>>(),
            [words(source[1])]
        );
        assert_eq!(cursor.snapshot(), [7, 31, 32, 33, 34, 2, 35]);
        let second = cursor.export_batch(&source, 1).unwrap();
        assert_eq!(
            second.iter().copied().map(words).collect::<Vec<_>>(),
            [words(source[3])]
        );
        assert_eq!(cursor.snapshot()[5], 4);
        assert!(cursor.export_batch(&source, 1).unwrap().is_empty());
        assert_eq!(cursor.snapshot()[5], 5);
        assert!(cursor.export_batch(&source, 1).unwrap().is_empty());
        assert_eq!(cursor.snapshot()[5], 5);
    }

    #[test]
    fn zero_requests_and_wide_identifiers_keep_native_boundary_semantics() {
        let source = [vertex(0xffff, 14), vertex(0, 20)];
        let initial = [0xffff, 1, 2, 3, 4, 0, 6];
        let mut cursor = VertexExportCursor::from_snapshot(initial);
        assert!(cursor.export_batch(&source, 0).unwrap().is_empty());
        assert_eq!(cursor.snapshot(), initial);
        let result = cursor.export_batch(&source, 2).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(words(result[0]), words(source[0]));
        assert_eq!(cursor.snapshot()[5], 2);
        let mut wide = VertexExportCursor::copy_descriptor([0x1_ffff, 0, 0, 0, 0, 0, 2]);
        assert!(wide.export_batch(&source, 1).unwrap().is_empty());
        assert_eq!(wide.snapshot()[5], 2);
        let snapshot = [0, 1, 2, 3, 4, u32::MAX, 6];
        let mut exhausted = VertexExportCursor::from_snapshot(snapshot);
        assert!(exhausted.export_batch(&source, 256).unwrap().is_empty());
        assert_eq!(exhausted.snapshot(), snapshot);
    }

    #[test]
    fn exact_batch_limit_and_failed_admission_do_not_change_cursor() {
        let source = vec![vertex(1, 0); 257];
        let mut cursor = VertexExportCursor::copy_descriptor([1, 0, 0, 0, 0, 0, 0]);
        let before = cursor.snapshot();
        assert_eq!(
            cursor.export_batch(&source, 257),
            Err(ExportError::BatchLimit)
        );
        assert_eq!(cursor.snapshot(), before);
        assert_eq!(cursor.export_batch(&source, 256).unwrap().len(), 256);
        assert_eq!(cursor.snapshot()[5], 256);
        assert_eq!(cursor.export_batch(&source, 256).unwrap().len(), 1);
        assert_eq!(cursor.snapshot()[5], 257);
        let source = vec![vertex(1, 0); MAX_PRIMARY_OUTPUT_VERTICES + 1];
        let before = cursor.snapshot();
        assert_eq!(
            cursor.export_batch(&source, 1),
            Err(ExportError::VertexLimit)
        );
        assert_eq!(cursor.snapshot(), before);
    }
}
