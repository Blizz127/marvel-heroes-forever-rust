//! Bounded composition of native primary normalization, export and record stores.
//!
//! This owns one immutable generated-record namespace. It does not choose a
//! live format, resolve native colors/cache state, invoke an output receiver,
//! generate triangles, or enable a rendering path.

use super::native_primary_export::{ExportError, VertexExportCursor};
use super::native_primary_packing::{
    PackingError, PackingProfile, ResolvedColorWord, write_primary_records,
};
use super::native_scale9::{
    MAX_PRIMARY_OUTPUT_VERTICES, MathError, Matrix2x4, PrimaryNormalizedVertices,
    normalize_primary_vertices_finite_ieee_nearest,
};
use super::native_scale9_geometry::{GeometryError, export_triangle_indices};
use super::native_scale9_tessellation::GeneratedVertex;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputError {
    AllocationFailed,
    Math(MathError),
    Export(ExportError),
    Packing(PackingError),
    Geometry(GeometryError),
}
impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailed => {
                f.write_str("cannot reserve primary output composition storage")
            }
            Self::Math(error) => error.fmt(f),
            Self::Export(error) => error.fmt(f),
            Self::Packing(error) => error.fmt(f),
            Self::Geometry(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for OutputError {}

/// ALL generated records after inverse-outer and normalization, retaining the
/// same record order, local ordinals, styles, flags and mesh IDs as the input.
/// Private immutable fields prevent changing positions or supplying a second
/// ordinal array between descriptor batches and triangle transfer.
#[derive(Debug)]
pub struct PreparedOutput {
    vertices: Vec<GeneratedVertex>,
    ordinals: Vec<u32>,
    normalization: PrimaryNormalizedVertices,
}
impl PreparedOutput {
    /// Normalize every supplied record BEFORE any descriptor selection. Unused
    /// mesh-0xffff records still contribute extrema. This cannot establish that
    /// the caller supplied the complete native array, nor replay enclosing
    /// native mesh/count guards. The finite-nearest arithmetic profile and
    /// engineering vertex cap are inherited from the arithmetic primitive.
    /// Borrowed input remains unchanged on success and on rejection.
    pub fn prepare(
        all_generated: &[GeneratedVertex],
        outer: Matrix2x4,
    ) -> Result<Self, OutputError> {
        if all_generated.len() > MAX_PRIMARY_OUTPUT_VERTICES {
            return Err(OutputError::Math(MathError::TooManyPrimaryVertices));
        }
        let mut points = Vec::new();
        points
            .try_reserve_exact(all_generated.len())
            .map_err(|_| OutputError::AllocationFailed)?;
        points.extend(all_generated.iter().map(|vertex| vertex.position));
        let normalization = normalize_primary_vertices_finite_ieee_nearest(&points, outer)
            .map_err(OutputError::Math)?;
        let mut vertices = Vec::new();
        vertices
            .try_reserve_exact(all_generated.len())
            .map_err(|_| OutputError::AllocationFailed)?;
        let mut ordinals = Vec::new();
        ordinals
            .try_reserve_exact(all_generated.len())
            .map_err(|_| OutputError::AllocationFailed)?;
        for (&vertex, &position) in all_generated.iter().zip(&normalization.positions) {
            vertices.push(GeneratedVertex { position, ..vertex });
            ordinals.push(vertex.local_ordinal);
        }
        Ok(Self {
            vertices,
            ordinals,
            normalization,
        })
    }

    pub fn vertices(&self) -> &[GeneratedVertex] {
        &self.vertices
    }

    pub fn normalization(&self) -> &PrimaryNormalizedVertices {
        &self.normalization
    }

    /// Copy the explicit seven-word descriptor and reset only scan word5.
    /// Selection/provenance of that descriptor remains a caller input.
    pub fn begin_descriptor(&self, words: [u32; 7]) -> DescriptorOutput<'_> {
        DescriptorOutput {
            prepared: self,
            cursor: VertexExportCursor::copy_descriptor(words),
        }
    }

    /// Attach an explicit snapshot to this source, preserving all seven words.
    /// The caller must establish that the snapshot belongs to this invocation.
    pub fn resume_descriptor(&self, words: [u32; 7]) -> DescriptorOutput<'_> {
        DescriptorOutput {
            prepared: self,
            cursor: VertexExportCursor::from_snapshot(words),
        }
    }

    /// Transfer triangle corners through local ordinals from this same source.
    /// Triangle inputs use global generated-record indices. Mesh membership,
    /// descriptor counts, connectivity and invocation order are not inferred.
    pub fn triangle_indices(
        &self,
        generated_triangles: &[[u32; 3]],
        first: usize,
        count: usize,
    ) -> Result<Vec<u16>, OutputError> {
        export_triangle_indices(&self.ordinals, generated_triangles, first, count)
            .map_err(OutputError::Geometry)
    }
}

/// One descriptor cursor borrowing a stable prepared source. Explicit color
/// words must include native cache resolution across chunks AND descriptors;
/// this object neither creates nor resets that invocation-wide native cache.
#[derive(Debug)]
pub struct DescriptorOutput<'a> {
    prepared: &'a PreparedOutput,
    cursor: VertexExportCursor,
}
impl<'a> DescriptorOutput<'a> {
    pub fn prepared(&self) -> &'a PreparedOutput {
        self.prepared
    }

    pub fn snapshot(&self) -> [u32; 7] {
        self.cursor.snapshot()
    }

    /// Inspect the next selected records without advancing this cursor. This
    /// permits explicit color resolution before a write; no callback runs here.
    pub fn preview(&self, requested: usize) -> Result<Vec<GeneratedVertex>, OutputError> {
        let mut cursor = self.cursor;
        cursor
            .export_batch(&self.prepared.vertices, requested)
            .map_err(OutputError::Export)
    }

    /// Export at most `requested` records and store them with explicit resolved
    /// colors/profile. Commit scan progress only after the entire store succeeds.
    /// Returns the copied record count; successful no-match scans still advance.
    /// Padding and destination tail are retained by the record writer.
    ///
    /// Every error retains BOTH destination and cursor. This is Rust admission
    /// policy, not native rollback behavior. In particular, native begin can
    /// write inverse-N before failing; no native receiver is invoked here.
    pub fn write_batch(
        &mut self,
        requested: usize,
        colors: &[ResolvedColorWord],
        profile: PackingProfile,
        destination: &mut [u8],
    ) -> Result<usize, OutputError> {
        let mut next = self.cursor;
        let vertices = next
            .export_batch(&self.prepared.vertices, requested)
            .map_err(OutputError::Export)?;
        write_primary_records(&vertices, colors, profile, destination)
            .map_err(OutputError::Packing)?;
        self.cursor = next;
        Ok(vertices.len())
    }
}

#[cfg(test)]
mod tests {
    use super::super::native_primary_packing::EXACT_PRIMARY_FORMAT;
    use super::*;

    fn vertex(position: [f32; 2], mesh: u16, local_ordinal: u32) -> GeneratedVertex {
        GeneratedVertex {
            position,
            mesh,
            local_ordinal,
            styles: [1, 2],
            flags: 0x0a,
        }
    }
    fn profile() -> PackingProfile {
        PackingProfile {
            format: EXACT_PRIMARY_FORMAT,
            forced_style: 0,
            morph_owner_present: false,
            morph_factor: 0.0,
        }
    }
    fn color() -> ResolvedColorWord {
        ResolvedColorWord {
            style_id: 1,
            word: 0x12345678,
        }
    }

    #[test]
    fn normalizes_unused_extrema_before_selecting_and_keeps_one_namespace() {
        let input = [
            vertex([-1.0, -1.0], 7, 2),
            vertex([1.0, 1.0], 7, 0),
            vertex([-4.0, -4.0], 0xffff, u32::MAX),
            vertex([4.0, 4.0], 0xffff, u32::MAX),
        ];
        let original = input;
        let prepared = PreparedOutput::prepare(&input, Matrix2x4::IDENTITY).unwrap();
        assert_eq!(
            prepared.normalization().pre_normalization_extrema,
            [-4.0, -4.0, 4.0, 4.0]
        );
        let descriptor = prepared.begin_descriptor([7, 11, 12, 13, 14, 99, 15]);
        assert_eq!(descriptor.snapshot(), [7, 11, 12, 13, 14, 0, 15]);
        assert!(std::ptr::eq(descriptor.prepared(), &prepared));
        let selected = descriptor.preview(2).unwrap();
        assert_eq!(selected[0], vertex([-8191.0, -8191.0], 7, 2));
        assert_eq!(selected[1], vertex([8191.0, 8191.0], 7, 0));
        assert_eq!(descriptor.snapshot()[5], 0);
        assert_eq!(
            prepared.triangle_indices(&[[1, 0, 2]], 0, 1).unwrap(),
            [0, 2, 0xffff]
        );
        assert_eq!(input, original);
    }

    #[test]
    fn failed_packing_retains_cursor_and_every_byte_then_resume_uses_same_source() {
        let input = [
            vertex([0.0, 0.0], 3, 10),
            vertex([1.0, 1.0], 7, 1),
            vertex([2.0, 2.0], 3, 11),
            vertex([3.0, 3.0], 7, 0),
        ];
        let prepared = PreparedOutput::prepare(&input, Matrix2x4::IDENTITY).unwrap();
        let mut descriptor = prepared.begin_descriptor([7, 11, 12, 13, 14, 99, 15]);
        let before = descriptor.snapshot();
        let mut destination = [0xa5; 27];
        let bad = ResolvedColorWord {
            style_id: 2,
            word: 0x12345678,
        };
        assert_eq!(
            descriptor.write_batch(2, &[color(), bad], profile(), &mut destination),
            Err(OutputError::Packing(PackingError::ColorStyleMismatch {
                vertex: 1
            }))
        );
        assert_eq!(descriptor.snapshot(), before);
        assert_eq!(destination, [0xa5; 27]);
        assert_eq!(
            descriptor
                .write_batch(1, &[color()], profile(), &mut destination)
                .unwrap(),
            1
        );
        assert_eq!(descriptor.snapshot()[5], 2);
        assert_eq!(&destination[10..], &[0xa5; 17]);
        let mut resumed = prepared.resume_descriptor(descriptor.snapshot());
        assert_eq!(resumed.preview(1).unwrap()[0].local_ordinal, 0);
        assert_eq!(
            resumed
                .write_batch(1, &[color()], profile(), &mut destination)
                .unwrap(),
            1
        );
        assert_eq!(resumed.snapshot(), [7, 11, 12, 13, 14, 4, 15]);
        assert_eq!(descriptor.snapshot()[5], 2);
        let saved = destination;
        assert_eq!(
            resumed
                .write_batch(1, &[], profile(), &mut destination)
                .unwrap(),
            0
        );
        assert_eq!(destination, saved);
        let mut no_match = prepared.begin_descriptor([0x10007, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            no_match
                .write_batch(1, &[], profile(), &mut destination)
                .unwrap(),
            0
        );
        assert_eq!(no_match.snapshot()[5], 4);
    }

    #[test]
    fn bounded_admission_and_empty_source_preserve_explicit_cursor() {
        let prepared = PreparedOutput::prepare(&[], Matrix2x4::IDENTITY).unwrap();
        assert!(!prepared.normalization().normalization_applied);
        let words = [0xffff, 1, 2, 3, 4, u32::MAX, 6];
        let mut descriptor = prepared.resume_descriptor(words);
        let mut output = [0x39; 5];
        assert_eq!(
            descriptor.write_batch(257, &[], profile(), &mut output),
            Err(OutputError::Export(ExportError::BatchLimit))
        );
        assert_eq!(descriptor.snapshot(), words);
        assert_eq!(output, [0x39; 5]);
        assert_eq!(
            descriptor
                .write_batch(0, &[], profile(), &mut output)
                .unwrap(),
            0
        );
        assert_eq!(descriptor.snapshot(), words);
        let oversized = vec![vertex([f32::NAN, 0.0], 1, 0); MAX_PRIMARY_OUTPUT_VERTICES + 1];
        assert!(matches!(
            PreparedOutput::prepare(&oversized, Matrix2x4::IDENTITY),
            Err(OutputError::Math(MathError::TooManyPrimaryVertices))
        ));
        assert_eq!(
            prepared.triangle_indices(&[[0, 0, 0]], 0, 1),
            Err(OutputError::Geometry(GeometryError::VertexIndex))
        );
    }
}
