//! Bounded Stage geometry recovered from the owner's 1.52.0.1700 client.
//!
//! See `docs/STAGE_RUNTIME.md` for instruction addresses, evidence and limits.
//! Inputs are explicit movie state, never inferred from a window, DPI or preset.
//! This module does not instantiate AS2 objects, coerce VM Values, or run events.
//! Arithmetic assumes the ordinary round-to-nearest, ties-to-even f32 profile.

use std::fmt;

const TWIPS_PER_PIXEL: f32 = 20.0;
const PIXELS_PER_TWIP: f32 = f32::from_bits(0x3d4c_cccd);

/// Opaque caller-assigned identity of the movie retained by a Stage object.
/// The caller owns uniqueness and lifetime; these are not native addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct MovieIdentity {
    pub runtime: u64,
    pub movie: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwipsRect {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

/// Only the native descriptor fields read by the effective-rectangle math.
/// `factor_cc` and `factor_d0` deliberately retain their native offset names:
/// their association with surface size, pixel density or DPI is not established.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportGeometry {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub factor_cc: f32,
    pub factor_d0: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageInputs {
    pub viewport: ViewportGeometry,
    /// Definition endpoints, before subtracting the minimum from the maximum.
    pub definition: Option<TwipsRect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ScaleMode {
    NoScale = 0,
    ShowAll = 1,
    ExactFit = 2,
    NoBorder = 3,
}

impl TryFrom<u32> for ScaleMode {
    type Error = StageError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoScale),
            1 => Ok(Self::ShowAll),
            2 => Ok(Self::ExactFit),
            3 => Ok(Self::NoBorder),
            _ => Err(StageError::UnsupportedScaleMode(value)),
        }
    }
}

impl ScaleMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoScale => "noScale",
            Self::ShowAll => "showAll",
            Self::ExactFit => "exactFit",
            Self::NoBorder => "noBorder",
        }
    }

    /// An actual ASCII string after VM conversion. No trimming or coercion.
    /// Native unknown strings select ShowAll, including the empty string.
    pub fn from_ascii(value: &str) -> Result<Self, StageError> {
        admit_ascii(value, "scaleMode")?;
        Ok(if value.eq_ignore_ascii_case("noScale") {
            Self::NoScale
        } else if value.eq_ignore_ascii_case("exactFit") {
            Self::ExactFit
        } else if value.eq_ignore_ascii_case("noBorder") {
            Self::NoBorder
        } else {
            Self::ShowAll
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Alignment {
    Center = 0,
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4,
    TopLeft = 5,
    TopRight = 6,
    BottomLeft = 7,
    BottomRight = 8,
}

impl TryFrom<u32> for Alignment {
    type Error = StageError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Center),
            1 => Ok(Self::Top),
            2 => Ok(Self::Bottom),
            3 => Ok(Self::Left),
            4 => Ok(Self::Right),
            5 => Ok(Self::TopLeft),
            6 => Ok(Self::TopRight),
            7 => Ok(Self::BottomLeft),
            8 => Ok(Self::BottomRight),
            _ => Err(StageError::UnsupportedAlignment(value)),
        }
    }
}

impl Alignment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Center => "",
            Self::Top => "T",
            Self::Bottom => "B",
            Self::Left => "L",
            Self::Right => "R",
            Self::TopLeft => "LT",
            Self::TopRight => "TR",
            Self::BottomLeft => "LB",
            Self::BottomRight => "RB",
        }
    }

    /// Native alignment examines the first two uppercase bytes only. Trailing
    /// ASCII bytes are ignored; unknown first bytes (including C) mean center.
    pub fn from_ascii(value: &str) -> Result<Self, StageError> {
        admit_ascii(value, "align")?;
        let mut bytes = value.bytes().map(|b| b.to_ascii_uppercase());
        Ok(match (bytes.next(), bytes.next()) {
            (Some(b'T'), Some(b'L')) | (Some(b'L'), Some(b'T')) => Self::TopLeft,
            (Some(b'T'), Some(b'R')) | (Some(b'R'), Some(b'T')) => Self::TopRight,
            (Some(b'B'), Some(b'L')) | (Some(b'L'), Some(b'B')) => Self::BottomLeft,
            (Some(b'B'), Some(b'R')) | (Some(b'R'), Some(b'B')) => Self::BottomRight,
            (Some(b'T'), _) => Self::Top,
            (Some(b'B'), _) => Self::Bottom,
            (Some(b'L'), _) => Self::Left,
            (Some(b'R'), _) => Self::Right,
            _ => Self::Center,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StageError {
    MissingDefinition,
    UnsupportedScaleMode(u32),
    UnsupportedAlignment(u32),
    UnsupportedString { property: &'static str },
    UnsupportedFloat { field: &'static str },
    NonPositive { field: &'static str },
    IntegerOverflow { field: &'static str },
    CenterOutOfRange { axis: &'static str },
}

impl fmt::Display for StageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDefinition => f.write_str("Stage definition is unavailable"),
            Self::UnsupportedScaleMode(v) => write!(f, "unsupported native Stage scale mode {v}"),
            Self::UnsupportedAlignment(v) => write!(f, "unsupported native Stage alignment {v}"),
            Self::UnsupportedString { property } => {
                write!(f, "Stage {property} requires ASCII without embedded NUL")
            }
            Self::UnsupportedFloat { field } => {
                write!(f, "Stage {field} is nonfinite or subnormal")
            }
            Self::NonPositive { field } => write!(f, "Stage {field} must be positive"),
            Self::IntegerOverflow { field } => {
                write!(f, "Stage {field} exceeds admitted i32 arithmetic")
            }
            Self::CenterOutOfRange { axis } => {
                write!(f, "Stage {axis} center exceeds admitted conversion")
            }
        }
    }
}

impl std::error::Error for StageError {}

/// A requested direct resize path from a changed scaleMode becoming NoScale.
/// This is not an event queue entry, callback execution, or extension argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoScaleResizeRequest {
    pub retained_movie: MovieIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageChange {
    pub retained_movie: MovieIdentity,
    pub previous: TwipsRect,
    pub effective: TwipsRect,
    /// Endpoint equality only; native viewport event conditions are broader.
    pub geometry_changed: bool,
    pub no_scale_resize: Option<NoScaleResizeRequest>,
}

/// State belonging to one retained movie. No native or loaded-script defaults
/// are supplied. All fallible changes validate before changing this state.
#[derive(Clone, Debug, PartialEq)]
pub struct StageState {
    retained_movie: MovieIdentity,
    inputs: StageInputs,
    mode: ScaleMode,
    alignment: Alignment,
    effective: TwipsRect,
}

impl StageState {
    pub fn from_inputs(
        retained_movie: MovieIdentity,
        inputs: StageInputs,
        mode: ScaleMode,
        alignment: Alignment,
    ) -> Result<Self, StageError> {
        let effective = compute_effective_rect(inputs, mode, alignment)?;
        Ok(Self {
            retained_movie,
            inputs,
            mode,
            alignment,
            effective,
        })
    }

    /// Retain an explicitly supplied effective rectangle without recomputing
    /// it. Snapshot values need not equal a new calculation from the descriptor.
    /// Later setters/recompute validate their own complete arithmetic paths.
    pub fn from_snapshot(
        retained_movie: MovieIdentity,
        inputs: StageInputs,
        mode: ScaleMode,
        alignment: Alignment,
        effective: TwipsRect,
    ) -> Result<Self, StageError> {
        admit_inputs(inputs)?;
        admit_effective(effective)?;
        Ok(Self {
            retained_movie,
            inputs,
            mode,
            alignment,
            effective,
        })
    }

    pub fn retained_movie(&self) -> MovieIdentity {
        self.retained_movie
    }
    pub fn inputs(&self) -> StageInputs {
        self.inputs
    }
    pub fn scale_mode(&self) -> ScaleMode {
        self.mode
    }
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
    pub fn effective_rect(&self) -> TwipsRect {
        self.effective
    }

    /// The native getter reads the retained movie's effective endpoints.
    pub fn width(&self) -> i32 {
        let twips = self.effective.x_max - self.effective.x_min;
        cvttss2si(twips * PIXELS_PER_TWIP)
    }

    pub fn height(&self) -> i32 {
        let twips = self.effective.y_max - self.effective.y_min;
        cvttss2si(twips * PIXELS_PER_TWIP)
    }

    /// Recompute from explicit descriptor/definition state. This reports only
    /// endpoint changes, not the native viewport setter's entire event policy.
    pub fn recompute(&mut self, inputs: StageInputs) -> Result<StageChange, StageError> {
        let effective = compute_effective_rect(inputs, self.mode, self.alignment)?;
        let change = self.change(effective, false);
        self.inputs = inputs;
        self.effective = effective;
        Ok(change)
    }

    pub fn set_scale_mode_ascii(&mut self, value: &str) -> Result<StageChange, StageError> {
        let mode = ScaleMode::from_ascii(value)?;
        // Native setter recomputes even when the resulting enum is unchanged.
        let effective = compute_effective_rect(self.inputs, mode, self.alignment)?;
        let request = self.mode != mode && mode == ScaleMode::NoScale;
        let change = self.change(effective, request);
        self.mode = mode;
        self.effective = effective;
        Ok(change)
    }

    pub fn set_alignment_ascii(&mut self, value: &str) -> Result<StageChange, StageError> {
        let alignment = Alignment::from_ascii(value)?;
        let effective = compute_effective_rect(self.inputs, self.mode, alignment)?;
        let change = self.change(effective, false);
        self.alignment = alignment;
        self.effective = effective;
        Ok(change)
    }

    fn change(&self, effective: TwipsRect, request: bool) -> StageChange {
        StageChange {
            retained_movie: self.retained_movie,
            previous: self.effective,
            effective,
            geometry_changed: self.effective != effective,
            no_scale_resize: request.then_some(NoScaleResizeRequest {
                retained_movie: self.retained_movie,
            }),
        }
    }
}

/// Numeric result of x86 CVTTSS2SI to signed 32 bits, including its indefinite
/// MIN result for NaN/infinity/out-of-range. Does not model exception flags or
/// traps. A plain Rust float-to-int cast would instead saturate or map NaN to 0.
pub fn cvttss2si(value: f32) -> i32 {
    if !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        i32::MIN
    } else {
        value as i32
    }
}

/// Effective endpoints from native 0x14005ae60, preserving each scalar f32
/// operation. Definition minima contribute only through extent subtraction.
/// This excludes transform fields and native matrix recomputation.
pub fn compute_effective_rect(
    inputs: StageInputs,
    mode: ScaleMode,
    alignment: Alignment,
) -> Result<TwipsRect, StageError> {
    let (dw, dh, vx, vy) = admit_inputs(inputs)?;
    let v = inputs.viewport;
    let (x_min, y_min, width, height) = match mode {
        ScaleMode::ExactFit => (0.0, 0.0, dw, dh),
        ScaleMode::NoScale => {
            let scaled_x = number(vx * v.factor_d0, "NoScale intermediate width")?;
            let width = positive(scaled_x * v.factor_cc, "NoScale width")?;
            let height = positive(v.factor_cc * vy, "NoScale height")?;
            let x_min = match alignment {
                Alignment::Center | Alignment::Top | Alignment::Bottom => center(dw, width, "x")?,
                Alignment::Left | Alignment::TopLeft | Alignment::BottomLeft => 0.0,
                Alignment::Right | Alignment::TopRight | Alignment::BottomRight => {
                    number(dw - width, "right offset")?
                }
            };
            let y_min = match alignment {
                Alignment::Center | Alignment::Left | Alignment::Right => center(dh, height, "y")?,
                Alignment::Top | Alignment::TopLeft | Alignment::TopRight => 0.0,
                Alignment::Bottom | Alignment::BottomLeft | Alignment::BottomRight => {
                    number(dh - height, "bottom offset")?
                }
            };
            (x_min, y_min, width, height)
        }
        ScaleMode::ShowAll | ScaleMode::NoBorder => {
            let adjusted_x = positive(vx * v.factor_d0, "adjusted viewport width")?;
            let ratio_x = positive(adjusted_x / dw, "horizontal ratio")?;
            let ratio_y = positive(vy / dh, "vertical ratio")?;
            let width_branch = if mode == ScaleMode::ShowAll {
                ratio_y > ratio_x
            } else {
                ratio_x > ratio_y
            };
            if width_branch {
                let half_definition = number(dh * 0.5, "half definition height")?;
                let product = number(dw * vy, "height numerator")?;
                let height = positive(product / adjusted_x, "effective height")?;
                let half_effective = number(height * 0.5, "half effective height")?;
                let y_min = number(half_definition - half_effective, "vertical offset")?;
                (0.0, y_min, dw, height)
            } else {
                let half_definition = number(dw * 0.5, "half definition width")?;
                let product = number(adjusted_x * dh, "width numerator")?;
                let width = positive(product / vy, "effective width")?;
                let half_effective = number(width * 0.5, "half effective width")?;
                let x_min = number(half_definition - half_effective, "horizontal offset")?;
                (x_min, 0.0, width, dh)
            }
        }
    };
    let effective = TwipsRect {
        x_min,
        y_min,
        x_max: number(width + x_min, "effective x maximum")?,
        y_max: number(height + y_min, "effective y maximum")?,
    };
    admit_effective(effective)?;
    Ok(effective)
}

fn admit_ascii(value: &str, property: &'static str) -> Result<(), StageError> {
    if value.is_ascii() && !value.as_bytes().contains(&0) {
        Ok(())
    } else {
        Err(StageError::UnsupportedString { property })
    }
}

fn number(value: f32, field: &'static str) -> Result<f32, StageError> {
    if value.is_finite() && !value.is_subnormal() {
        Ok(value)
    } else {
        Err(StageError::UnsupportedFloat { field })
    }
}

fn positive(value: f32, field: &'static str) -> Result<f32, StageError> {
    number(value, field)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(StageError::NonPositive { field })
    }
}

fn rect_extents(rect: TwipsRect) -> Result<(f32, f32), StageError> {
    number(rect.x_min, "rectangle x minimum")?;
    number(rect.y_min, "rectangle y minimum")?;
    number(rect.x_max, "rectangle x maximum")?;
    number(rect.y_max, "rectangle y maximum")?;
    Ok((
        positive(rect.x_max - rect.x_min, "rectangle width")?,
        positive(rect.y_max - rect.y_min, "rectangle height")?,
    ))
}

fn admit_effective(rect: TwipsRect) -> Result<(), StageError> {
    let (width, height) = rect_extents(rect)?;
    number(width * PIXELS_PER_TWIP, "getter pixel width")?;
    number(height * PIXELS_PER_TWIP, "getter pixel height")?;
    Ok(())
}

fn admit_inputs(inputs: StageInputs) -> Result<(f32, f32, f32, f32), StageError> {
    let definition = inputs.definition.ok_or(StageError::MissingDefinition)?;
    let (dw, dh) = rect_extents(definition)?;
    let v = inputs.viewport;
    positive(v.factor_cc, "descriptor factor_cc")?;
    positive(v.factor_d0, "descriptor factor_d0")?;
    let vx = viewport_extent(v.left, v.width, "viewport horizontal extent")?;
    let vy = viewport_extent(v.top, v.height, "viewport vertical extent")?;
    Ok((dw, dh, vx, vy))
}

fn viewport_extent(origin: i32, size: i32, field: &'static str) -> Result<f32, StageError> {
    if size <= 0 {
        return Err(StageError::NonPositive { field });
    }
    let end = origin
        .checked_add(size)
        .ok_or(StageError::IntegerOverflow { field })?;
    // Conversion before multiplication/subtraction is observable at large origins.
    let end_twips = number((end as f32) * TWIPS_PER_PIXEL, field)?;
    let start_twips = number((origin as f32) * TWIPS_PER_PIXEL, field)?;
    positive(end_twips - start_twips, field)
}

fn center(definition: f32, effective: f32, axis: &'static str) -> Result<f32, StageError> {
    let half_definition = number(definition * 0.5, "center half definition")?;
    let half_effective = number(effective * 0.5, "center half effective")?;
    let delta = number(half_definition - half_effective, "center delta")?;
    let pixels = number(delta * PIXELS_PER_TWIP, "center pixels")?;
    if !(-2_147_483_648.0..2_147_483_648.0).contains(&pixels) {
        return Err(StageError::CenterOutOfRange { axis });
    }
    let twips = cvttss2si(pixels)
        .checked_mul(20)
        .ok_or(StageError::CenterOutOfRange { axis })?;
    number(twips as f32, "center twips")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVIE: MovieIdentity = MovieIdentity {
        runtime: 7,
        movie: 3,
    };
    fn rect(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> TwipsRect {
        TwipsRect {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }
    fn inputs(width: i32, height: i32) -> StageInputs {
        StageInputs {
            viewport: ViewportGeometry {
                left: 0,
                top: 0,
                width,
                height,
                factor_cc: 1.0,
                factor_d0: 1.0,
            },
            definition: Some(rect(0.0, 0.0, 2000.0, 1600.0)),
        }
    }

    #[test]
    fn modes_and_definition_minima() {
        let mut i = inputs(100, 100);
        i.definition = Some(rect(40.0, 60.0, 2040.0, 1660.0));
        for a in 0..=8 {
            let a = Alignment::try_from(a).unwrap();
            assert_eq!(
                compute_effective_rect(i, ScaleMode::ExactFit, a).unwrap(),
                rect(0.0, 0.0, 2000.0, 1600.0)
            );
            assert_eq!(
                compute_effective_rect(i, ScaleMode::ShowAll, a).unwrap(),
                rect(0.0, -200.0, 2000.0, 1800.0)
            );
            assert_eq!(
                compute_effective_rect(i, ScaleMode::NoBorder, a).unwrap(),
                rect(200.0, 0.0, 1800.0, 1600.0)
            );
        }
    }

    #[test]
    fn no_scale_center_truncates_toward_zero_in_pixels() {
        let mut i = inputs(60, 50);
        i.definition = Some(rect(0.0, 0.0, 2030.0, 1610.0));
        let small =
            StageState::from_inputs(MOVIE, i, ScaleMode::NoScale, Alignment::Center).unwrap();
        assert_eq!(small.effective_rect(), rect(400.0, 300.0, 1600.0, 1300.0));
        assert_eq!((small.width(), small.height()), (60, 50));
        i.viewport.width = 120;
        i.viewport.height = 100;
        assert_eq!(
            compute_effective_rect(i, ScaleMode::NoScale, Alignment::Center).unwrap(),
            rect(-180.0, -180.0, 2220.0, 1820.0)
        );
    }

    #[test]
    fn no_scale_alignments_and_descriptor_factors() {
        let expected = [
            (400.0, 300.0),
            (400.0, 0.0),
            (400.0, 610.0),
            (0.0, 300.0),
            (830.0, 300.0),
            (0.0, 0.0),
            (830.0, 0.0),
            (0.0, 610.0),
            (830.0, 610.0),
        ];
        let mut i = inputs(60, 50);
        i.definition = Some(rect(0.0, 0.0, 2030.0, 1610.0));
        for (a, (x, y)) in expected.into_iter().enumerate() {
            assert_eq!(
                compute_effective_rect(
                    i,
                    ScaleMode::NoScale,
                    Alignment::try_from(a as u32).unwrap()
                )
                .unwrap(),
                rect(x, y, 1200.0 + x, 1000.0 + y)
            );
        }
        let mut i = inputs(100, 100);
        i.viewport.factor_cc = 0.75;
        i.viewport.factor_d0 = 1.25;
        let s = StageState::from_inputs(MOVIE, i, ScaleMode::NoScale, Alignment::TopLeft).unwrap();
        assert_eq!(s.effective_rect(), rect(0.0, 0.0, 1875.0, 1500.0));
        assert_eq!((s.width(), s.height()), (93, 75));
    }

    #[test]
    fn string_mapping_preserves_unknown_behavior() {
        for (name, mode) in [
            ("nOsCaLe", ScaleMode::NoScale),
            ("EXACTFIT", ScaleMode::ExactFit),
            ("noborder", ScaleMode::NoBorder),
            ("showAll", ScaleMode::ShowAll),
            ("noScale ", ScaleMode::ShowAll),
            ("", ScaleMode::ShowAll),
        ] {
            assert_eq!(ScaleMode::from_ascii(name).unwrap(), mode);
        }
        for (name, a) in [
            ("C", Alignment::Center),
            ("", Alignment::Center),
            ("unknown", Alignment::Center),
            ("lTignored", Alignment::TopLeft),
            ("tb", Alignment::Top),
            ("rb", Alignment::BottomRight),
            ("bl", Alignment::BottomLeft),
            ("rt", Alignment::TopRight),
        ] {
            assert_eq!(Alignment::from_ascii(name).unwrap(), a);
        }
        for a in 0..=8 {
            let a = Alignment::try_from(a).unwrap();
            assert_eq!(Alignment::from_ascii(a.as_str()).unwrap(), a);
        }
        assert!(Alignment::from_ascii("T\0L").is_err());
        assert!(ScaleMode::from_ascii("nóScale").is_err());
        assert!(ScaleMode::try_from(4).is_err());
        assert!(Alignment::try_from(9).is_err());
    }

    #[test]
    fn snapshot_getters_retain_movie_and_recompute_even_unchanged_mode() {
        let effective = rect(100.0, -50.0, 119.0, 29.0);
        let mut s = StageState::from_snapshot(
            MOVIE,
            inputs(100, 100),
            ScaleMode::NoScale,
            Alignment::TopLeft,
            effective,
        )
        .unwrap();
        assert_eq!(s.effective_rect(), effective);
        assert_eq!((s.width(), s.height()), (0, 3));
        let change = s.set_scale_mode_ascii("noScale").unwrap();
        assert!(change.geometry_changed);
        assert_eq!(change.previous, effective);
        assert_eq!(change.retained_movie, MOVIE);
        assert_eq!(s.retained_movie(), MOVIE);
        assert!(change.no_scale_resize.is_none());
        assert_eq!((s.width(), s.height()), (100, 100));
    }

    #[test]
    fn resize_request_requires_changed_mode_becoming_no_scale() {
        let mut s = StageState::from_inputs(
            MOVIE,
            inputs(100, 80),
            ScaleMode::ExactFit,
            Alignment::Center,
        )
        .unwrap();
        let change = s.set_scale_mode_ascii("noScale").unwrap();
        assert!(!change.geometry_changed);
        assert_eq!(
            change.no_scale_resize,
            Some(NoScaleResizeRequest {
                retained_movie: MOVIE
            })
        );
        assert!(
            s.set_scale_mode_ascii("NOSCALE")
                .unwrap()
                .no_scale_resize
                .is_none()
        );
        assert!(
            s.set_alignment_ascii("TL")
                .unwrap()
                .no_scale_resize
                .is_none()
        );
        assert!(
            s.recompute(inputs(120, 90))
                .unwrap()
                .no_scale_resize
                .is_none()
        );
        assert!(
            s.set_scale_mode_ascii("unknown")
                .unwrap()
                .no_scale_resize
                .is_none()
        );
        assert_eq!(s.scale_mode(), ScaleMode::ShowAll);
    }

    #[test]
    fn invalid_inputs_are_explicit_and_changes_are_atomic() {
        let mut s = StageState::from_inputs(
            MOVIE,
            inputs(100, 100),
            ScaleMode::ShowAll,
            Alignment::Center,
        )
        .unwrap();
        let before = s.clone();
        let mut i = inputs(100, 100);
        i.viewport.left = i32::MAX;
        assert!(matches!(
            s.recompute(i),
            Err(StageError::IntegerOverflow { .. })
        ));
        assert_eq!(s, before);
        assert!(s.set_alignment_ascii("é").is_err());
        assert_eq!(s, before);
        i = inputs(100, 100);
        i.definition = None;
        assert_eq!(s.recompute(i), Err(StageError::MissingDefinition));
        for factor in [0.0, -1.0, f32::INFINITY, f32::NAN, f32::from_bits(1)] {
            i = inputs(100, 100);
            i.viewport.factor_cc = factor;
            assert!(s.recompute(i).is_err());
            assert_eq!(s, before);
        }
        i = inputs(1, 1);
        i.viewport.left = 1 << 30; // Converted endpoints collapse to one f32.
        assert!(s.recompute(i).is_err());
    }

    #[test]
    fn center_overflow_is_rejected_without_saturating() {
        let mut i = inputs(100, 100);
        i.definition = Some(rect(0.0, 0.0, 1.0e11, 1.0e11));
        // TopLeft does not evaluate unused center conversion.
        let mut s =
            StageState::from_inputs(MOVIE, i, ScaleMode::NoScale, Alignment::TopLeft).unwrap();
        let before = s.clone();
        assert!(matches!(
            s.set_alignment_ascii("C"),
            Err(StageError::CenterOutOfRange { .. })
        ));
        assert_eq!(s, before);
        i.definition = Some(rect(0.0, 0.0, 5.0e9, 5.0e9));
        assert!(matches!(
            compute_effective_rect(i, ScaleMode::NoScale, Alignment::Center),
            Err(StageError::CenterOutOfRange { .. })
        ));
    }

    #[test]
    fn cvtt_numeric_edges_and_large_getter() {
        for v in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            2_147_483_648.0,
            -2_147_483_904.0,
            -2_147_483_648.0,
        ] {
            assert_eq!(cvttss2si(v), i32::MIN);
        }
        assert_eq!(cvttss2si(f32::from_bits(0x4eff_ffff)), 2_147_483_520);
        assert_eq!(cvttss2si(f32::from_bits(0xceff_ffff)), -2_147_483_520);
        for v in [0.0, -0.0, f32::from_bits(1), -f32::from_bits(1)] {
            assert_eq!(cvttss2si(v), 0);
        }
        assert_eq!(cvttss2si(-19.9), -19);
        let s = StageState::from_snapshot(
            MOVIE,
            inputs(100, 100),
            ScaleMode::ShowAll,
            Alignment::Center,
            rect(0.0, 0.0, 1.0e11, 1.0e11),
        )
        .unwrap();
        assert_eq!((s.width(), s.height()), (i32::MIN, i32::MIN));
    }
}
