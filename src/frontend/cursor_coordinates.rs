//! Pure SetMouse coordinate preparation from the owned native evidence.
//!
//! This module performs no window queries, backend writes or cursor operations.
//! Both scaled paths require an explicit nearest-even, masked-exception profile.
//! The complete masked path returns native integer-indefinite values; the finite
//! preflight can instead decline them. Floating status flags and exception
//! delivery are not reproduced or preserved by this numerical API. Actual window
//! identity, API results and sampled inputs are supplied, never defaulted here.
//! See `docs/CURSOR_COORDINATES.md` for the native ordering and excluded cases.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// The width and height values returned by the two separate native getters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportSamples {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetMouseScaling {
    Unscaled,
    /// Selects the supplied arithmetic profile. Preparation determines whether
    /// invalid conversions are returned as native integer-indefinite or declined.
    /// No function here inspects or writes the MXCSR control environment.
    NearestEvenMasked {
        window_rect: WindowRect,
        viewport: ViewportSamples,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateError {
    ZeroViewportDimension(Axis),
    /// The native conversion can produce integer-indefinite with exceptions
    /// masked. That exceptional path is outside the finite preflight's profile.
    OutsideFiniteIntegerConversion(Axis),
}

/// A value to store into the backend point before requesting ClientToScreen.
/// Creating this value does not perform the store or any other host effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparedSetMouse {
    point_to_store: CursorPoint,
}

/// An explicitly supplied, non-owning identity. It is never dereferenced,
/// resolved to a window or given ownership/destruction behavior here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowToken(pub u64);

impl PreparedSetMouse {
    pub fn backend_point_to_store(self) -> CursorPoint {
        self.point_to_store
    }

    /// Supply the handle/token sampled for this call. It need not be the same
    /// handle that supplied the earlier GetWindowRect sample.
    pub fn client_to_screen_request(self, window: WindowToken) -> ClientToScreenRequest {
        ClientToScreenRequest {
            window,
            point: self.point_to_store,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientToScreenRequest {
    pub window: WindowToken,
    pub point: CursorPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientToScreenObservation {
    /// The caller supplies the actual post-call Y-then-X backend reads. No
    /// unchanged-point assumption is made when the imported function fails.
    pub post_call_point: CursorPoint,
    pub reported_success: bool,
}

impl ClientToScreenRequest {
    /// The native path ignores ClientToScreen's status and uses the subsequently
    /// read backend coordinates. This returns a request, never an OS operation.
    pub fn after_observed_return(
        self,
        observation: ClientToScreenObservation,
    ) -> SetCursorPosRequest {
        SetCursorPosRequest {
            point: observation.post_call_point,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetCursorPosRequest {
    pub point: CursorPoint,
}

/// Return the complete masked integer result for every supplied i32/u32 input.
///
/// Under `NearestEvenMasked`, zero denominators, nonfinite products and finite
/// conversion overflow produce `i32::MIN`, as the native masked CVTTSS2SI does.
/// Exactly -2^31 has the same integer value but is a valid conversion. Both axes
/// are calculated even when the first conversion is invalid. This function
/// models numerical values only: it neither sets up MXCSR nor promises native
/// floating status flags, getter timing, partial effects or actual host stores.
pub fn prepare_set_mouse_masked(
    requested: CursorPoint,
    scaling: SetMouseScaling,
) -> PreparedSetMouse {
    let point_to_store = match scaling {
        SetMouseScaling::Unscaled => requested,
        SetMouseScaling::NearestEvenMasked {
            window_rect,
            viewport,
        } => {
            let width = window_rect.right.wrapping_sub(window_rect.left);
            let height = window_rect.bottom.wrapping_sub(window_rect.top);
            CursorPoint {
                x: masked_axis(requested.x, viewport.width, width),
                y: masked_axis(requested.y, viewport.height, height),
            }
        }
    };
    PreparedSetMouse { point_to_store }
}

/// Prepare a finite admitted coordinate pair. No effect request is returned for
/// an unsupported arithmetic case; native behavior outside this profile is not
/// replaced with Rust's saturating float-to-integer conversion.
///
/// This is a preflight, not an emulation of native partial execution. In the
/// native masked path an invalid X conversion still proceeds to the height
/// getter, Y conversion and both backend stores; this preflight can return early.
pub fn prepare_set_mouse(
    requested: CursorPoint,
    scaling: SetMouseScaling,
) -> Result<PreparedSetMouse, CoordinateError> {
    let point_to_store = match scaling {
        SetMouseScaling::Unscaled => requested,
        SetMouseScaling::NearestEvenMasked {
            window_rect,
            viewport,
        } => {
            // Native SUB operates on 32-bit words before signed conversion.
            let width = window_rect.right.wrapping_sub(window_rect.left);
            let height = window_rect.bottom.wrapping_sub(window_rect.top);
            CursorPoint {
                x: scale_axis(requested.x, viewport.width, width, Axis::X)?,
                y: scale_axis(requested.y, viewport.height, height, Axis::Y)?,
            }
        }
    };
    Ok(PreparedSetMouse { point_to_store })
}

fn axis_product(value: i32, dimension: u32, rect_extent: i32) -> f32 {
    // Keep the native single-precision division before multiplication. A
    // multiply-first or double-precision rewrite changes supported results.
    let input = value as f32;
    let denominator = dimension as f32;
    let extent = rect_extent as f32;
    let quotient = input / denominator;
    quotient * extent
}

fn scale_axis(
    value: i32,
    dimension: u32,
    rect_extent: i32,
    axis: Axis,
) -> Result<i32, CoordinateError> {
    if dimension == 0 {
        return Err(CoordinateError::ZeroViewportDimension(axis));
    }
    let product = axis_product(value, dimension, rect_extent);
    finite_integer_conversion(product).ok_or(CoordinateError::OutsideFiniteIntegerConversion(axis))
}

fn masked_axis(value: i32, dimension: u32, rect_extent: i32) -> i32 {
    finite_integer_conversion(axis_product(value, dimension, rect_extent)).unwrap_or(i32::MIN)
}

fn finite_integer_conversion(product: f32) -> Option<i32> {
    // INT_MAX is not representable as f32; its cast rounds to +2^31. The
    // upper bound must therefore be exclusive, while exactly -2^31 is valid.
    if !product.is_finite() || !(-2_147_483_648.0..2_147_483_648.0).contains(&product) {
        return None;
    }
    Some(product as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unscaled_preserves_integer_extremes_without_float_conversion() {
        let requested = CursorPoint {
            x: i32::MAX,
            y: i32::MIN,
        };
        assert_eq!(
            prepare_set_mouse(requested, SetMouseScaling::Unscaled)
                .unwrap()
                .backend_point_to_store(),
            requested
        );
        assert_eq!(
            prepare_set_mouse_masked(requested, SetMouseScaling::Unscaled).backend_point_to_store(),
            requested
        );
    }

    #[test]
    fn finite_products_match_independent_native_instruction_witnesses() {
        // Frozen outputs from the original standalone SSE oracle, not values
        // calculated by this implementation or by an original client run.
        let cases: &[(i32, u32, i32, u32, i32)] = &[
            (867, 1280, 1920, 0x44a2_9000, 1300),
            (-867, 1280, 1920, 0xc4a2_9000, -1300),
            (50, 100, 400, 0x4348_0000, 200),
            (25, 100, -200, 0xc248_0000, -50),
            (1, 1, -1, 0xbf80_0000, -1),
            (16_777_217, 3, 3, 0x4b80_0000, 16_777_216),
            (-16_777_217, 3, 3, 0xcb80_0000, -16_777_216),
            (i32::MAX, u32::MAX, i32::MAX, 0x4e80_0000, 1_073_741_824),
            (2_147_483_520, 1, 1, 0x4eff_ffff, 2_147_483_520),
            (i32::MIN, 1, 1, 0xcf00_0000, i32::MIN),
            (i32::MAX, 3, 0, 0, 0),
            (-1, 3, 0, 0x8000_0000, 0),
            (1, u32::MAX, 1, 0x2f80_0000, 0),
            (123_456_789, 1_234_567, 7_654_321, 0x4e36_7e4e, 765_432_704),
            (
                2_000_000_001,
                2_000_000_000,
                2_000_000_001,
                0x4eee_6b28,
                2_000_000_000,
            ),
            (16_777_215, 7, 3, 0x4adb_6db6, 7_190_235),
            (16_777_215, 16_777_217, 16_777_215, 0x4b7f_fffe, 16_777_214),
        ];
        for &(input, dimension, extent, expected_bits, expected_integer) in cases {
            assert_eq!(
                axis_product(input, dimension, extent).to_bits(),
                expected_bits
            );
            assert_eq!(
                scale_axis(input, dimension, extent, Axis::X),
                Ok(expected_integer)
            );
            assert_eq!(masked_axis(input, dimension, extent), expected_integer);
        }
    }

    #[test]
    fn complete_masked_conversion_matches_exception_witnesses() {
        // All six invalid cases from the retained controlled SSE oracle. Its
        // integer result is MIN in each case; NaN payload bits are not promised.
        let cases = [
            (i32::MAX, 1, 1),
            (i32::MIN, 1, 2),
            (0, 0, 1),
            (1, 0, 1),
            (-1, 0, 1),
            (1, 0, 0),
        ];
        for (value, dimension, extent) in cases {
            assert_eq!(masked_axis(value, dimension, extent), i32::MIN);
        }
        assert_eq!(finite_integer_conversion(-2_147_483_648.0), Some(i32::MIN));
        assert_eq!(finite_integer_conversion(2_147_483_648.0), None);
    }

    #[test]
    fn masked_preparation_continues_after_invalid_x_or_y() {
        let point = prepare_set_mouse_masked(
            CursorPoint { x: 1, y: 867 },
            SetMouseScaling::NearestEvenMasked {
                window_rect: WindowRect {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1920,
                },
                viewport: ViewportSamples {
                    width: 0,
                    height: 1280,
                },
            },
        )
        .backend_point_to_store();
        assert_eq!(
            point,
            CursorPoint {
                x: i32::MIN,
                y: 1300
            }
        );

        let point = prepare_set_mouse_masked(
            CursorPoint {
                x: 867,
                y: i32::MIN,
            },
            SetMouseScaling::NearestEvenMasked {
                window_rect: WindowRect {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 2,
                },
                viewport: ViewportSamples {
                    width: 1280,
                    height: 1,
                },
            },
        )
        .backend_point_to_store();
        assert_eq!(
            point,
            CursorPoint {
                x: 1300,
                y: i32::MIN
            }
        );
    }

    #[test]
    fn rectangle_deltas_wrap_and_do_not_add_window_origin() {
        let prepared = prepare_set_mouse(
            CursorPoint { x: 50, y: 1 },
            SetMouseScaling::NearestEvenMasked {
                window_rect: WindowRect {
                    left: -300,
                    right: 100,
                    top: i32::MIN,
                    bottom: i32::MAX,
                },
                viewport: ViewportSamples {
                    width: 100,
                    height: 1,
                },
            },
        )
        .unwrap();
        assert_eq!(
            prepared.backend_point_to_store(),
            CursorPoint { x: 200, y: -1 }
        );
    }

    #[test]
    fn finite_preflight_declines_exceptional_conversion_paths() {
        for input in [0, 1, -1] {
            assert_eq!(
                scale_axis(input, 0, 1, Axis::X),
                Err(CoordinateError::ZeroViewportDimension(Axis::X))
            );
        }
        for (input, extent) in [(i32::MAX, 1), (i32::MIN, 2)] {
            assert_eq!(
                scale_axis(input, 1, extent, Axis::Y),
                Err(CoordinateError::OutsideFiniteIntegerConversion(Axis::Y))
            );
        }
        assert_eq!(scale_axis(i32::MIN, 1, 1, Axis::Y), Ok(i32::MIN));
        assert_eq!(
            prepare_set_mouse(
                CursorPoint { x: 1, y: i32::MAX },
                SetMouseScaling::NearestEvenMasked {
                    window_rect: WindowRect {
                        left: 0,
                        top: 0,
                        right: 1,
                        bottom: 1,
                    },
                    viewport: ViewportSamples {
                        width: 1,
                        height: 1
                    },
                },
            ),
            Err(CoordinateError::OutsideFiniteIntegerConversion(Axis::Y))
        );
    }

    #[test]
    fn imported_status_does_not_replace_the_supplied_post_call_point() {
        let initial = CursorPoint { x: 123, y: -456 };
        let prepared = prepare_set_mouse(initial, SetMouseScaling::Unscaled).unwrap();
        let actual_after_call = CursorPoint { x: -88, y: 999 };
        for reported_success in [false, true] {
            let request = prepared.client_to_screen_request(WindowToken(0x1234));
            assert_eq!(request.point, initial);
            let next = request.after_observed_return(ClientToScreenObservation {
                post_call_point: actual_after_call,
                reported_success,
            });
            assert_eq!(next.point, actual_after_call);
        }
    }
}
