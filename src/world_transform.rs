//! Explicit finite world-transform primitives; effective saved-object defaults
//! and actor/component relationships must be established by the caller.
//!
//! The scaled matrix follows the bounded native helper documented in
//! `docs/WORLD_TRANSFORMS.md`. It does not infer a parent from an export outer,
//! choose missing scale/rotation values, or evaluate the live frontend scene.

use crate::camera::{CameraError, CameraMath, Matrix4, Pose, RotatorWords};
use crate::world_scene::{SavedSceneObject, SceneObjectKind};
use std::{error::Error, fmt};

/// Every value is explicit. `scale` is the effective three-component input to
/// the native matrix helper, not a guessed combination of saved scalar/vector
/// properties. Zero and negative scale components are permitted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplicitScaledTransform {
    pub translation: [f32; 3],
    pub rotation: RotatorWords,
    pub scale: [f32; 3],
}

/// Raw explicit inputs to native actor helper 0x1408598c0. `pivot_input` is the
/// vector read at native actor+0xa8; its algebraic effect is a pre-pivot, while
/// association with the saved PrePivot property remains independently qualified.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplicitActorTransform {
    pub position: [f32; 3],
    pub rotation: RotatorWords,
    pub uniform_scale: f32,
    pub scale3: [f32; 3],
    pub pivot_input: [f32; 3],
}

/// The caller must supply all three choices. This increment accepts only all
/// false; it does not infer that omitted saved flags are false.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbsoluteFlags {
    pub translation: bool,
    pub rotation: bool,
    pub scale: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplicitComponentTransform {
    pub translation: [f32; 3],
    pub rotation: RotatorWords,
    pub uniform_scale: f32,
    pub scale3: [f32; 3],
    pub absolute: AbsoluteFlags,
}

/// A required saved-placement slot. Absence never means zero or unit scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementField {
    Location,
    Translation,
    Rotation,
    DrawScale,
    DrawScale3D,
    Scale,
    Scale3D,
}

/// Relationships whose effective semantics cannot be selected from an export's
/// outer/name or absent tagged properties. This remains independent of whether
/// all four placement properties happen to be explicitly saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationObligation {
    ActorBasePivotAndUpdatePath,
    ComponentOwnerAbsoluteFlagsAndUpdatePath,
}

/// An audit of direct saved inputs, not a default resolver or effective matrix.
/// The borrowed source retains every field's property index, tag/payload spans,
/// exact bits, object/outer/archetype references and opaque native payload.
#[derive(Debug)]
pub struct SavedTransformAudit<'a> {
    pub source: &'a SavedSceneObject,
    pub missing_fields: Vec<PlacementField>,
    pub relation_obligation: RelationObligation,
}

/// Reports which placement values need evidence-backed resolution. There is no
/// automatic conversion from this audit into `ExplicitScaledTransform`: even
/// an empty missing list does not establish the owner/attachment policy, scalar
/// scale combination or final world placement.
pub fn audit_saved_transform(source: &SavedSceneObject) -> SavedTransformAudit<'_> {
    let placement = &source.placement;
    let mut missing_fields = Vec::with_capacity(4);
    let relation_obligation = match source.kind {
        SceneObjectKind::Actor => {
            for (field, absent) in [
                (PlacementField::Location, placement.location.is_none()),
                (PlacementField::Rotation, placement.rotation.is_none()),
                (PlacementField::DrawScale, placement.draw_scale.is_none()),
                (
                    PlacementField::DrawScale3D,
                    placement.draw_scale3d.is_none(),
                ),
            ] {
                if absent {
                    missing_fields.push(field);
                }
            }
            RelationObligation::ActorBasePivotAndUpdatePath
        }
        SceneObjectKind::Component => {
            for (field, absent) in [
                (PlacementField::Translation, placement.translation.is_none()),
                (PlacementField::Rotation, placement.rotation.is_none()),
                (PlacementField::Scale, placement.scale.is_none()),
                (PlacementField::Scale3D, placement.scale3d.is_none()),
            ] {
                if absent {
                    missing_fields.push(field);
                }
            }
            RelationObligation::ComponentOwnerAbsoluteFlagsAndUpdatePath
        }
    };
    SavedTransformAudit {
        source,
        missing_fields,
        relation_obligation,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformError {
    Camera(CameraError),
    NonFiniteScale,
    NonFinitePivot,
    NonFiniteParent,
    NonFiniteMatrix,
    UnsupportedAbsoluteFlags,
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Camera(error) => write!(f, "{error}"),
            Self::NonFiniteScale => f.write_str("world transform scale must be finite"),
            Self::NonFinitePivot => f.write_str("world transform pivot input must be finite"),
            Self::NonFiniteParent => f.write_str("world transform parent matrix must be finite"),
            Self::NonFiniteMatrix => {
                f.write_str("world transform arithmetic produced a non-finite matrix")
            }
            Self::UnsupportedAbsoluteFlags => {
                f.write_str("absolute component transform flags are not supported by this profile")
            }
        }
    }
}

impl Error for TransformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Camera(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CameraError> for TransformError {
    fn from(error: CameraError) -> Self {
        Self::Camera(error)
    }
}

/// Native 0x14092d230: evaluate the grouped rotation terms, then multiply each
/// first-three row's first-three values by that row's supplied scale component.
/// Translation is copied unchanged and the homogeneous column is written as
/// +0,+0,+0,1 rather than scaling those zero terms. No default values are used.
pub fn scaled_matrix(
    math: &CameraMath,
    transform: ExplicitScaledTransform,
) -> Result<Matrix4, TransformError> {
    if !transform.scale.into_iter().all(f32::is_finite) {
        return Err(TransformError::NonFiniteScale);
    }
    let mut result = math.rotation_translation(Pose {
        position: transform.translation,
        rotation: transform.rotation,
    })?;
    for (row, scale) in result.rows[..3].iter_mut().zip(transform.scale) {
        for value in &mut row[..3] {
            *value *= scale;
        }
    }
    if result.rows.iter().flatten().all(|value| value.is_finite()) {
        Ok(result)
    } else {
        Err(TransformError::NonFiniteMatrix)
    }
}

/// Actor-specific grouped scalar arithmetic. Its real-number interpretation is
/// pivot translation followed by scale/rotation and final position; its float
/// words must not be replaced by the generic scaled matrix or matrix products.
pub fn actor_matrix(
    math: &CameraMath,
    transform: ExplicitActorTransform,
) -> Result<Matrix4, TransformError> {
    if !transform.position.into_iter().all(f32::is_finite) {
        return Err(TransformError::Camera(CameraError::NonFinitePosition));
    }
    if !transform.uniform_scale.is_finite() || !transform.scale3.into_iter().all(f32::is_finite) {
        return Err(TransformError::NonFiniteScale);
    }
    if !transform.pivot_input.into_iter().all(f32::is_finite) {
        return Err(TransformError::NonFinitePivot);
    }
    let [p, y, r] = transform.rotation.0;
    let (sp, cp) = math.sin_cos(p);
    let (sy, cy) = math.sin_cos(y);
    let (sr, cr) = math.sin_cos(r);
    let sx = transform.uniform_scale * transform.scale3[0];
    let sy_scale = transform.uniform_scale * transform.scale3[1];
    let sz = transform.uniform_scale * transform.scale3[2];
    let [px, py, pz] = transform.pivot_input;
    let [lx, ly, lz] = transform.position;
    let flip = |value: f32| f32::from_bits(value.to_bits() ^ 0x8000_0000);

    let cy_cr = cy * cr;
    let sx_cp = sx * cp;
    let sy_cp = sy_scale * cp;
    let m00 = (cy * cp) * sx;
    let m01 = sx_cp * sy;
    let m02 = sx * sp;
    let m10 = ((cy * sp) * sr - cr * sy) * sy_scale;
    let m11 = ((sp * sr) * sy + cy_cr) * sy_scale;
    let m12 = flip(sy_cp * sr);
    let m20 = flip((cy_cr * sp + sy * sr) * sz);
    let m21 = (cy * sr - (cr * sp) * sy) * sz;
    let m22 = (cp * cr) * sz;

    let tx = lx - m00 * px;
    let tx = tx + ((cy_cr * sz) * pz) * sp;
    let tx = tx - (((sy_scale * cy) * py) * sp) * sr;
    let tx = tx + ((sy_scale * cr) * py) * sy;
    let tx = tx + ((sz * pz) * sr) * sy;
    let ty_sum = ((sz * cy) * pz) * sr + (cy_cr * sy_scale) * py;
    let ty_sum = ty_sum + (sx_cp * px) * sy;
    let ty_sum = ty_sum - (((sz * cr) * pz) * sp) * sy;
    let ty_sum = ty_sum + (((sy_scale * py) * sp) * sr) * sy;
    let ty = ly - ty_sum;
    let tz_sum = (sx * px) * sp + m22 * pz;
    let tz_sum = tz_sum - (sy_cp * py) * sr;
    let tz = lz - tz_sum;
    let rows = [
        [m00, m01, m02, 0.0],
        [m10, m11, m12, 0.0],
        [m20, m21, m22, 0.0],
        [tx, ty, tz, 1.0],
    ];
    if rows.iter().flatten().all(|value| value.is_finite()) {
        Ok(Matrix4 { rows })
    } else {
        Err(TransformError::NonFiniteMatrix)
    }
}

/// Supported native component path: scalar*vector scale, local SRT, then
/// local*the explicitly supplied parent. Owner/collection membership, defaults
/// and selection of this parent must be resolved separately by the caller.
pub fn component_matrix(
    math: &CameraMath,
    transform: ExplicitComponentTransform,
    parent: Matrix4,
) -> Result<Matrix4, TransformError> {
    if transform.absolute.translation || transform.absolute.rotation || transform.absolute.scale {
        return Err(TransformError::UnsupportedAbsoluteFlags);
    }
    if !parent.rows.iter().flatten().all(|value| value.is_finite()) {
        return Err(TransformError::NonFiniteParent);
    }
    if !transform.uniform_scale.is_finite() || !transform.scale3.into_iter().all(f32::is_finite) {
        return Err(TransformError::NonFiniteScale);
    }
    let scale = transform
        .scale3
        .map(|value| transform.uniform_scale * value);
    if !scale.into_iter().all(f32::is_finite) {
        return Err(TransformError::NonFiniteMatrix);
    }
    let local = scaled_matrix(
        math,
        ExplicitScaledTransform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale,
        },
    )?;
    let mut rows = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            rows[i][j] = ((local.rows[i][0] * parent.rows[0][j]
                + local.rows[i][1] * parent.rows[1][j])
                + local.rows[i][2] * parent.rows[2][j])
                + local.rows[i][3] * parent.rows[3][j];
        }
    }
    if rows.iter().flatten().all(|value| value.is_finite()) {
        Ok(Matrix4 { rows })
    } else {
        Err(TransformError::NonFiniteMatrix)
    }
}
