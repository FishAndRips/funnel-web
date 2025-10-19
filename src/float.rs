//! Floating point operations.
//! 
//! See [FloatOps]'s documentation.

use core::cmp::Ordering;

/// Adds basic floating point operations.
/// 
/// All methods are prefixed with `fw_` to avoid conflicting with the Rust standard library.
/// 
/// These are guaranteed to be accurate to the way Halo calculates its floats, at least on SSE.
pub trait FloatOps: Copy + Copy {
    /// Approximately equal to 2.0 times pi.
    const FW_2PI: Self;

    /// Approximately equal to pi.
    const FW_PI: Self;

    /// Approximately equal to 0.5 times pi.
    const FW_HALF_PI: Self;

    /// Approximately equal to 0.25 times pi.
    const FW_QUARTER_PI: Self;

    /// Approximately equal to 1 degree in radians.
    ///
    /// Used for converting degrees to radians with the same bitwise accuracy as tool.exe
    const FW_RADIANS_PER_DEGREE: Self;

    /// Approximately equal to 1 radian in degrees.
    ///
    /// Used for converting radians to degrees with the same bitwise accuracy as tool.exe
    const FW_DEGREES_PER_RADIAN: Self;

    /// Calculate the square root of a float.
    #[must_use]
    fn fw_sqrt(self) -> Self;

    /// Calculate the inverse square root of a float.
    ///
    /// That is, 1.0 / x.sqrt()
    #[must_use]
    fn fw_inverse_sqrt(self) -> Self;

    /// Calculate the float to the integer power.
    #[must_use]
    fn fw_powi(self, exponent: i32) -> Self;

    /// Calculate the float to the float power.
    #[must_use]
    fn fw_powf(self, exponent: Self) -> Self;

    /// Calculate the absolute value of the float.
    #[must_use]
    fn fw_fabs(self) -> Self;

    /// Calculate the sine of the float.
    ///
    /// The float is treated as being in radians.
    #[must_use]
    fn fw_sin(self) -> Self;

    /// Calculate the inverse sine (arcsine) of the float.
    ///
    /// The float is treated as being in radians.
    fn fw_asin(self) -> Self;

    /// Calculate the cosine of the float.
    ///
    /// The float is treated as being in radians.
    #[must_use]
    fn fw_cos(self) -> Self;

    /// Calculate the inverse cosine (arccosine) of the float.
    ///
    /// The float is treated as being in radians.
    #[must_use]
    fn fw_acos(self) -> Self;

    /// Calculate the tangent of the float.
    ///
    /// The float is treated as being in radians.
    #[must_use]
    fn fw_tan(self) -> Self;

    /// Calculate the inverse tangent (arctangent) of the float.
    ///
    /// The float is treated as being in radians.
    #[must_use]
    fn fw_atan(self) -> Self;

    /// Calculate the inverse tangent (arctangent) of `self/x`.
    #[must_use]
    fn fw_atan2(self, x: f32) -> Self;

    /// Round the float.
    ///
    /// If the float is evenly between two values (i.e. X.5), then round to the nearest even number.
    #[must_use]
    fn fw_round_ties_even_to_int(self) -> i32;

    /// Round the float towards zero.
    ///
    /// This is like using floor() for a positive value and ceil() for a negative number.
    #[must_use]
    fn fw_round_towards_zero_to_int(self) -> i32;

    /// Floor the float.
    #[must_use]
    fn fw_floor_to_int(self) -> i32;

    /// Return true if the given value is close to another value.
    #[must_use]
    fn fw_is_close_to(self, to: Self) -> bool;

    /// Return true if the given value is non-NaN and non-infinite.
    #[must_use]
    fn fw_is_finite(self) -> bool;

    /// Return true if the given value is close to 0.0.
    ///
    /// This is a convenience function for `self.fw_is_close_to(0.0)`
    #[must_use]
    fn fw_is_close_to_zero(self) -> bool;

    /// Return true if the given value is close to 0.0 or less.
    #[must_use]
    fn fw_is_close_to_zero_or_less(self) -> bool;
}

impl FloatOps for f32 {
    const FW_2PI: Self = core::f32::consts::TAU;
    const FW_PI: Self = core::f32::consts::PI;
    const FW_HALF_PI: Self = core::f32::consts::FRAC_PI_2;
    const FW_QUARTER_PI: Self = core::f32::consts::FRAC_PI_4;
    const FW_RADIANS_PER_DEGREE: Self = 0.01745329251994;
    const FW_DEGREES_PER_RADIAN: Self = 57.29577951308;

    #[inline]
    fn fw_sqrt(self) -> Self {
        libm::sqrtf(self)
    }
    #[inline]
    fn fw_inverse_sqrt(self) -> Self {
        1.0 / self.fw_sqrt()
    }
    #[inline]
    fn fw_powi(self, exponent: i32) -> Self {
        self.fw_powf(exponent as f32)
    }
    #[inline]
    fn fw_powf(self, exponent: Self) -> Self {
        libm::powf(self, exponent)
    }
    #[inline]
    fn fw_fabs(self) -> Self {
        libm::fabsf(self)
    }
    #[inline]
    fn fw_sin(self) -> Self { libm::sinf(self) }
    #[inline]
    fn fw_asin(self) -> Self { libm::asinf(self) }
    #[inline]
    fn fw_cos(self) -> Self { libm::cosf(self) }
    #[inline]
    fn fw_acos(self) -> Self { libm::acosf(self) }
    #[inline]
    fn fw_tan(self) -> Self { libm::tanf(self) }
    #[inline]
    fn fw_atan(self) -> Self { libm::atanf(self) }
    #[inline]
    fn fw_atan2(self, x: f32) -> Self { libm::atan2f(self, x) }

    fn fw_round_ties_even_to_int(self) -> i32 {
        let a = self.fw_floor_to_int();
        let b = a.saturating_add(1);
        let low = self - (a as f32);
        let high = (b as f32) - self;

        match low.total_cmp(&high) {
            Ordering::Less => a,
            Ordering::Greater => b,

            // Round to the nearest even number
            Ordering::Equal => if (a & 1) != 0 { b } else { a }
        }
    }
    #[inline]
    fn fw_round_towards_zero_to_int(self) -> i32 {
        self as i32
    }
    fn fw_floor_to_int(self) -> i32 {
        let rounded = self.fw_round_towards_zero_to_int();
        if self == (rounded as f32) {
            rounded
        }
        else {
            match self.total_cmp(&0.0) {
                Ordering::Equal => 0,
                Ordering::Greater => rounded,
                Ordering::Less => rounded - 1
            }
        }
    }
    #[inline]
    fn fw_is_close_to(self, to: Self) -> bool {
        (self - to).abs() < 0.0001
    }
    #[inline]
    fn fw_is_finite(self) -> bool {
        self.is_finite()
    }
    #[inline]
    fn fw_is_close_to_zero(self) -> bool {
        self.abs() < 0.0001
    }
    #[inline]
    fn fw_is_close_to_zero_or_less(self) -> bool {
        self < 0.0001
    }
}

#[cfg(test)]
mod test {
    use crate::util::assert_similar;
    use crate::float::FloatOps;

    #[test]
    fn powf() {
        assert_eq!(2.0f32.fw_powf(3.0), 8.0);
        assert_eq!(2.0f32.fw_powf(-1.0), 0.5);
        assert_eq!(2.0f32.fw_powf(0.0), 1.0);
        assert_eq!(0.5f32.fw_powf(3.0), 0.125);
        assert_eq!(0.5f32.fw_powf(-1.0), 2.0);
        assert_eq!(0.5f32.fw_powf(0.0), 1.0);
    }

    #[test]
    fn powi() {
        assert_eq!(2.0f32.fw_powi(3), 8.0);
        assert_eq!(2.0f32.fw_powi(-1), 0.5);
        assert_eq!(2.0f32.fw_powi(0), 1.0);
        assert_eq!(0.5f32.fw_powi(3), 0.125);
        assert_eq!(0.5f32.fw_powi(-1), 2.0);
        assert_eq!(0.5f32.fw_powi(0), 1.0);
    }

    #[test]
    fn sqrt() {
        assert_eq!(4.0f32.fw_sqrt(), 2.0);
        assert_eq!(0.25f32.fw_sqrt(), 0.5);
    }

    #[test]
    fn inverse_sqrt() {
        assert_eq!(4.0f32.fw_inverse_sqrt(), 0.5);
        assert_eq!(0.25f32.fw_inverse_sqrt(), 2.0);
    }

    #[test]
    fn sin_cos_tan() {
        assert_similar!(f32::FW_PI.fw_sin(), 0.0, 0.001);
        assert_similar!(f32::FW_HALF_PI.fw_sin(), 1.0, 0.001);
        assert_similar!(f32::FW_QUARTER_PI.fw_sin(), core::f64::consts::FRAC_1_SQRT_2, 0.001);
        assert_similar!(0.0.fw_sin(), 0.0, 0.001);

        assert_similar!(f32::FW_PI.fw_cos(), -1.0, 0.001);
        assert_similar!(f32::FW_HALF_PI.fw_cos(), 0.0, 0.001);
        assert_similar!(f32::FW_QUARTER_PI.fw_cos(), core::f64::consts::FRAC_1_SQRT_2, 0.001);
        assert_similar!(0.0.fw_cos(), 1.0, 0.001);

        assert_similar!(f32::FW_PI.fw_tan(), 0.0, 0.001);
        assert_similar!(f32::FW_QUARTER_PI.fw_tan(), 1.0, 0.001);
        assert_similar!(0.0.fw_tan(), 0.0, 0.001);
        // tan(180.0 degrees) = inf, so we aren't going to test that
    }

    #[test]
    fn round_ties_even_to_int() {
        assert_eq!(0.0f32.fw_round_ties_even_to_int(), 0);
        assert_eq!(1.0f32.fw_round_ties_even_to_int(), 1);
        assert_eq!(2.0f32.fw_round_ties_even_to_int(), 2);
        assert_eq!(3.0f32.fw_round_ties_even_to_int(), 3);
        assert_eq!(-1.0f32.fw_round_ties_even_to_int(), -1);
        assert_eq!(-2.0f32.fw_round_ties_even_to_int(), -2);
        assert_eq!(-3.0f32.fw_round_ties_even_to_int(), -3);

        assert_eq!(0.5f32.fw_round_ties_even_to_int(), 0);
        assert_eq!(-0.5f32.fw_round_ties_even_to_int(), 0);
        assert_eq!(1.5f32.fw_round_ties_even_to_int(), 2);
        assert_eq!(-1.5f32.fw_round_ties_even_to_int(), -2);
        assert_eq!(2.5f32.fw_round_ties_even_to_int(), 2);
        assert_eq!(-2.5f32.fw_round_ties_even_to_int(), -2);
        assert_eq!(3.5f32.fw_round_ties_even_to_int(), 4);
        assert_eq!(-3.5f32.fw_round_ties_even_to_int(), -4);
        assert_eq!(4.5f32.fw_round_ties_even_to_int(), 4);
        assert_eq!(-4.5f32.fw_round_ties_even_to_int(), -4);

        assert_eq!(0.6f32.fw_round_ties_even_to_int(), 1);
        assert_eq!(-0.6f32.fw_round_ties_even_to_int(), -1);
        assert_eq!(1.6f32.fw_round_ties_even_to_int(), 2);
        assert_eq!(-1.6f32.fw_round_ties_even_to_int(), -2);
        assert_eq!(2.6f32.fw_round_ties_even_to_int(), 3);
        assert_eq!(-2.6f32.fw_round_ties_even_to_int(), -3);
        assert_eq!(3.6f32.fw_round_ties_even_to_int(), 4);
        assert_eq!(-3.6f32.fw_round_ties_even_to_int(), -4);
        assert_eq!(4.6f32.fw_round_ties_even_to_int(), 5);
        assert_eq!(-4.6f32.fw_round_ties_even_to_int(), -5);

        assert_eq!(0.4f32.fw_round_ties_even_to_int(), 0);
        assert_eq!(-0.4f32.fw_round_ties_even_to_int(), 0);
        assert_eq!(1.4f32.fw_round_ties_even_to_int(), 1);
        assert_eq!(-1.4f32.fw_round_ties_even_to_int(), -1);
        assert_eq!(2.4f32.fw_round_ties_even_to_int(), 2);
        assert_eq!(-2.4f32.fw_round_ties_even_to_int(), -2);
        assert_eq!(3.4f32.fw_round_ties_even_to_int(), 3);
        assert_eq!(-3.4f32.fw_round_ties_even_to_int(), -3);
        assert_eq!(4.4f32.fw_round_ties_even_to_int(), 4);
        assert_eq!(-4.4f32.fw_round_ties_even_to_int(), -4);
    }

    #[test]
    fn round_towards_zero_to_int() {
        assert_eq!(0.0f32.fw_round_towards_zero_to_int(), 0);
        assert_eq!(1.0f32.fw_round_towards_zero_to_int(), 1);
        assert_eq!(2.0f32.fw_round_towards_zero_to_int(), 2);
        assert_eq!(3.0f32.fw_round_towards_zero_to_int(), 3);
        assert_eq!((-1.0f32).fw_round_towards_zero_to_int(), -1);
        assert_eq!((-2.0f32).fw_round_towards_zero_to_int(), -2);
        assert_eq!((-3.0f32).fw_round_towards_zero_to_int(), -3);

        assert_eq!(0.5f32.fw_round_towards_zero_to_int(), 0);
        assert_eq!((-0.5f32).fw_round_towards_zero_to_int(), 0);
        assert_eq!(1.5f32.fw_round_towards_zero_to_int(), 1);
        assert_eq!((-1.5f32).fw_round_towards_zero_to_int(), -1);
        assert_eq!(2.5f32.fw_round_towards_zero_to_int(), 2);
        assert_eq!((-2.5f32).fw_round_towards_zero_to_int(), -2);
        assert_eq!(3.5f32.fw_round_towards_zero_to_int(), 3);
        assert_eq!((-3.5f32).fw_round_towards_zero_to_int(), -3);
        assert_eq!(4.5f32.fw_round_towards_zero_to_int(), 4);
        assert_eq!((-4.5f32).fw_round_towards_zero_to_int(), -4);

        assert_eq!(0.6f32.fw_round_towards_zero_to_int(), 0);
        assert_eq!((-0.6f32).fw_round_towards_zero_to_int(), 0);
        assert_eq!(1.6f32.fw_round_towards_zero_to_int(), 1);
        assert_eq!((-1.6f32).fw_round_towards_zero_to_int(), -1);
        assert_eq!(2.6f32.fw_round_towards_zero_to_int(), 2);
        assert_eq!((-2.6f32).fw_round_towards_zero_to_int(), -2);
        assert_eq!(3.6f32.fw_round_towards_zero_to_int(), 3);
        assert_eq!((-3.6f32).fw_round_towards_zero_to_int(), -3);
        assert_eq!(4.6f32.fw_round_towards_zero_to_int(), 4);
        assert_eq!((-4.6f32).fw_round_towards_zero_to_int(), -4);

        assert_eq!(0.4f32.fw_round_towards_zero_to_int(), 0);
        assert_eq!((-0.4f32).fw_round_towards_zero_to_int(), 0);
        assert_eq!(1.4f32.fw_round_towards_zero_to_int(), 1);
        assert_eq!((-1.4f32).fw_round_towards_zero_to_int(), -1);
        assert_eq!(2.4f32.fw_round_towards_zero_to_int(), 2);
        assert_eq!((-2.4f32).fw_round_towards_zero_to_int(), -2);
        assert_eq!(3.4f32.fw_round_towards_zero_to_int(), 3);
        assert_eq!((-3.4f32).fw_round_towards_zero_to_int(), -3);
        assert_eq!(4.4f32.fw_round_towards_zero_to_int(), 4);
        assert_eq!((-4.4f32).fw_round_towards_zero_to_int(), -4);
    }

    #[test]
    fn fw_floor_to_int() {
        assert_eq!(0.0f32.fw_floor_to_int(), 0);
        assert_eq!(1.0f32.fw_floor_to_int(), 1);
        assert_eq!(2.0f32.fw_floor_to_int(), 2);
        assert_eq!(3.0f32.fw_floor_to_int(), 3);
        assert_eq!((-1.0f32).fw_floor_to_int(), -1);
        assert_eq!((-2.0f32).fw_floor_to_int(), -2);
        assert_eq!((-3.0f32).fw_floor_to_int(), -3);

        assert_eq!(0.5f32.fw_floor_to_int(), 0);
        assert_eq!((-0.5f32).fw_floor_to_int(), -1);
        assert_eq!(1.5f32.fw_floor_to_int(), 1);
        assert_eq!((-1.5f32).fw_floor_to_int(), -2);
        assert_eq!(2.5f32.fw_floor_to_int(), 2);
        assert_eq!((-2.5f32).fw_floor_to_int(), -3);
        assert_eq!(3.5f32.fw_floor_to_int(), 3);
        assert_eq!((-3.5f32).fw_floor_to_int(), -4);
        assert_eq!(4.5f32.fw_floor_to_int(), 4);
        assert_eq!((-4.5f32).fw_floor_to_int(), -5);

        assert_eq!(0.6f32.fw_floor_to_int(), 0);
        assert_eq!((-0.6f32).fw_floor_to_int(), -1);
        assert_eq!(1.6f32.fw_floor_to_int(), 1);
        assert_eq!((-1.6f32).fw_floor_to_int(), -2);
        assert_eq!(2.6f32.fw_floor_to_int(), 2);
        assert_eq!((-2.6f32).fw_floor_to_int(), -3);
        assert_eq!(3.6f32.fw_floor_to_int(), 3);
        assert_eq!((-3.6f32).fw_floor_to_int(), -4);
        assert_eq!(4.6f32.fw_floor_to_int(), 4);
        assert_eq!((-4.6f32).fw_floor_to_int(), -5);

        assert_eq!(0.4f32.fw_floor_to_int(), 0);
        assert_eq!((-0.4f32).fw_floor_to_int(), -1);
        assert_eq!(1.4f32.fw_floor_to_int(), 1);
        assert_eq!((-1.4f32).fw_floor_to_int(), -2);
        assert_eq!(2.4f32.fw_floor_to_int(), 2);
        assert_eq!((-2.4f32).fw_floor_to_int(), -3);
        assert_eq!(3.4f32.fw_floor_to_int(), 3);
        assert_eq!((-3.4f32).fw_floor_to_int(), -4);
        assert_eq!(4.4f32.fw_floor_to_int(), 4);
        assert_eq!((-4.4f32).fw_floor_to_int(), -5);
    }
}
