//! Explicit, finite-input camera math recovered from the owner's native client.
//!
//! See `docs/CAMERA_MATH.md` for evidence, operation order and remaining gaps.
//! No pose, FOV, viewport, depth, active camera or tangent implementation is
//! selected here. Arithmetic assumes IEEE754 round-to-nearest/ties-to-even;
//! the caller's floating-point environment and native CRT parity are unverified.

use std::{error::Error, fmt};

const TABLE_LEN: usize = 16_384;
const Q_EQUAL: f32 = f32::from_bits(0x3f7f_be77);

/// Raw packed words at native offsets 0, 4 and 8. Field names remain qualified
/// by the evidence; the mathematical effects are elevation, azimuth and roll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotatorWords(pub [i32; 3]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub rotation: RotatorWords,
}

/// Row-major memory and row-vector multiplication convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 {
    pub rows: [[f32; 4]; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FovAxis {
    X,
    Y,
    Major,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProjectionProfile {
    Unconstrained {
        axis: FovAxis,
    },
    /// Retained explicitly, but rejected: native viewport cropping consumes
    /// an additional scalar whose meaning and runtime value remain unresolved.
    ConstrainedAspect {
        aspect: f32,
    },
}

/// Equal positive near/far is supported and selects the native Q=0.999... branch.
/// Otherwise far must exceed near. No depth value is supplied by this module.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthRange {
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionInput {
    pub fov_degrees: f32,
    pub viewport: [u32; 2],
    pub profile: ProjectionProfile,
    pub depth: DepthRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraError {
    NonFinitePosition,
    InvalidFov,
    EmptyViewport,
    InvalidDepth,
    InvalidAspect,
    UnresolvedConstrainedViewport,
    InvalidTangent,
    NonFiniteMatrix,
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinitePosition => "camera position must be finite",
            Self::InvalidFov => "camera FOV must be finite and strictly between 0 and 180 degrees",
            Self::EmptyViewport => "camera viewport dimensions must be positive",
            Self::InvalidDepth => "camera near/far must be finite, positive and far >= near",
            Self::InvalidAspect => "camera aspect must be finite and positive",
            Self::UnresolvedConstrainedViewport => "constrained viewport scalar is not established",
            Self::InvalidTangent => "caller-supplied tangent must return a finite positive value",
            Self::NonFiniteMatrix => "camera arithmetic produced a non-finite matrix",
        })
    }
}

impl Error for CameraError {}

/// Owns an originally generated table; no extracted retail table is embedded.
pub struct CameraMath {
    sine: Box<[f32]>,
}

impl CameraMath {
    pub fn new() -> Self {
        let mut sine = Vec::with_capacity(TABLE_LEN);
        let pi = f64::from_bits(0x4009_21fb_5444_2d18);
        for n in 0..TABLE_LEN {
            let doubled = (2 * n) as f32;
            let angle = ((f64::from(doubled) * pi) * (1.0 / 16_384.0)) as f32;
            sine.push(table_sine(angle));
        }
        Self {
            sine: sine.into_boxed_slice(),
        }
    }

    /// Quantizes by discarding the low two bits and wraps every 65,536 units.
    pub fn sin_cos(&self, word: i32) -> (f32, f32) {
        let index = |a: u32| ((a >> 2) & 0x3fff) as usize;
        let a = word as u32;
        (
            self.sine[index(a)],
            self.sine[index(a.wrapping_add(0x4000))],
        )
    }

    /// Native grouped scalar rotation constructor with the supplied translation.
    pub fn rotation_translation(&self, pose: Pose) -> Result<Matrix4, CameraError> {
        validate_position(pose.position)?;
        let [p, y, r] = pose.rotation.0;
        let (sp, cp) = self.sin_cos(p);
        let (sy, cy) = self.sin_cos(y);
        let (sr, cr) = self.sin_cos(r);
        let sp_sr = sp * sr;
        let cr_sp = cr * sp;
        checked([
            [cy * cp, cp * sy, sp, 0.0],
            [
                sp_sr * cy - cr * sy,
                sp_sr * sy + cy * cr,
                sign_flip(cp * sr),
                0.0,
            ],
            [
                sign_flip(cr_sp * cy + sy * sr),
                cy * sr - cr_sp * sy,
                cp * cr,
                0.0,
            ],
            [pose.position[0], pose.position[1], pose.position[2], 1.0],
        ])
    }

    /// Two ordered native SIMD products. This intentionally does not transpose
    /// the separately rounded forward rotation matrix.
    pub fn inverse_rotation(&self, rotation: RotatorWords) -> Result<Matrix4, CameraError> {
        let [p, y, r] = rotation.0;
        let (sp, cp) = self.sin_cos(p);
        let (sy, cy) = self.sin_cos(y);
        let (sr, cr) = self.sin_cos(r);
        let yaw = [
            [cy, sign_flip(sy), 0.0, 0.0],
            [sy, cy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let pitch = [
            [cp, 0.0, sign_flip(sp), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [sp, 0.0, cp, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let roll = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cr, sr, 0.0],
            [0.0, sign_flip(sr), cr, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        checked(inverse_product(inverse_product(yaw, pitch), roll))
    }

    /// T(-position) * inverse_rotation * basis, using native product ordering.
    /// The basis changes (forward, right, up) into (right, up, forward).
    pub fn view(&self, pose: Pose) -> Result<Matrix4, CameraError> {
        validate_position(pose.position)?;
        let t = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                sign_flip(pose.position[0]),
                sign_flip(pose.position[1]),
                sign_flip(pose.position[2]),
                1.0,
            ],
        ];
        let b = [
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let inverse = self.inverse_rotation(pose.rotation)?.rows;
        checked(view_product(view_product(t, inverse), b))
    }

    /// Caller selects the tangent implementation explicitly. Passing `f32::tan`
    /// is a platform-math choice, not evidence of native MSVCR120 parity.
    /// Calls tangent for Y, then X, even when the two input angles are equal.
    pub fn projection(
        &self,
        input: ProjectionInput,
        mut tangent: impl FnMut(f32) -> f32,
    ) -> Result<Matrix4, CameraError> {
        let h = half_fov_radians(input.fov_degrees)?;
        let [width, height] = input.viewport;
        if width == 0 || height == 0 {
            return Err(CameraError::EmptyViewport);
        }
        let DepthRange { near, far } = input.depth;
        if !near.is_finite() || !far.is_finite() || near <= 0.0 || far < near {
            return Err(CameraError::InvalidDepth);
        }
        let axis = match input.profile {
            ProjectionProfile::Unconstrained { axis } => axis,
            ProjectionProfile::ConstrainedAspect { aspect } => {
                if !aspect.is_finite() || aspect <= 0.0 {
                    return Err(CameraError::InvalidAspect);
                }
                return Err(CameraError::UnresolvedConstrainedViewport);
            }
        };
        let (xm, ym) = if axis == FovAxis::X || (axis == FovAxis::Major && width > height) {
            (1.0, width as f32 / height as f32)
        } else {
            (height as f32 / width as f32, 1.0)
        };
        // Native computes the depth quotient independently for its two entries.
        let q_for_translation = if near == far {
            Q_EQUAL
        } else {
            far / (far - near)
        };
        let q = if near == far {
            Q_EQUAL
        } else {
            far / (far - near)
        };
        let depth_translation = sign_flip(q_for_translation * near);
        let ty = tangent(h);
        if !ty.is_finite() || ty <= 0.0 {
            return Err(CameraError::InvalidTangent);
        }
        let yy = ym / ty;
        let tx = tangent(h);
        if !tx.is_finite() || tx <= 0.0 {
            return Err(CameraError::InvalidTangent);
        }
        let xx = xm / tx;
        checked([
            [xx, 0.0, 0.0, 0.0],
            [0.0, yy, 0.0, 0.0],
            [0.0, 0.0, q, 1.0],
            [0.0, 0.0, depth_translation, 0.0],
        ])
    }
}

impl Default for CameraMath {
    fn default() -> Self {
        Self::new()
    }
}

/// Native caller's two separate binary32 multiplies, with no selected FOV.
pub fn half_fov_radians(fov_degrees: f32) -> Result<f32, CameraError> {
    if !fov_degrees.is_finite() || fov_degrees <= 0.0 || fov_degrees >= 180.0 {
        return Err(CameraError::InvalidFov);
    }
    Ok((fov_degrees * f32::from_bits(0x4049_0fdb)) * f32::from_bits(0x3b36_0b61))
}

fn sign_flip(x: f32) -> f32 {
    f32::from_bits(x.to_bits() ^ 0x8000_0000)
}

fn validate_position(position: [f32; 3]) -> Result<(), CameraError> {
    if position.into_iter().all(f32::is_finite) {
        Ok(())
    } else {
        Err(CameraError::NonFinitePosition)
    }
}

fn checked(rows: [[f32; 4]; 4]) -> Result<Matrix4, CameraError> {
    if rows.iter().flatten().all(|x| x.is_finite()) {
        Ok(Matrix4 { rows })
    } else {
        Err(CameraError::NonFiniteMatrix)
    }
}

fn inverse_product(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] =
                ((a[i][1] * b[1][j] + a[i][0] * b[0][j]) + a[i][2] * b[2][j]) + a[i][3] * b[3][j];
        }
    }
    out
}

fn view_product(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let first = (a[i][0] * b[0][j] + a[i][1] * b[1][j]) + a[i][2] * b[2][j];
            let last = a[i][3] * b[3][j];
            out[i][j] = if i == 3 { last + first } else { first + last };
        }
    }
    out
}

fn table_sine(a: f32) -> f32 {
    let k = (a * f32::from_bits(0x3ea2_f983)).round_ties_even();
    let b = ((a - k * f32::from_bits(0x4049_0000)) - k * f32::from_bits(0x3a7d_a000))
        - k * f32::from_bits(0x3422_2000);
    let t = b - k * f32::from_bits(0x2cb4_611a);
    let z = b * b;
    let q = f32::from_bits(0x362f_0519) * z;
    let q = (q + f32::from_bits(0xb94f_baf1)) * z;
    let q = (q + f32::from_bits(0x3c08_8773)) * z;
    let q = (q + f32::from_bits(0xbe2a_aaa5)) * z;
    let value = q * t + t;
    f32::from_bits(value.to_bits() ^ (((k as u32) & 1) << 31))
}
