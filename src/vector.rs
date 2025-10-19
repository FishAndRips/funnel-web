//! Defines vectors and vector math.

use core::cmp::Ordering;
use core::fmt::{Debug, Display, Formatter};
use core::mem::transmute;
use core::ops::{Add, Mul, MulAssign, Neg, Sub};
use crate::float::FloatOps;

/// A matrix with just the forward and up components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Matrix2x3 {
    pub forward: Vector3D,
    pub up: Vector3D
}

impl Matrix2x3 {
    /// Identity matrix.
    pub const IDENTITY: Matrix2x3 = Matrix2x3 { forward: Vector3D { x: 1.0, y: 0.0, z: 0.0 }, up: Vector3D { x: 0.0, y: 0.0, z: 1.0 } };
}

/// A full 3x3 matrix.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Matrix3x3 {
    pub forward: Vector3D,
    pub left: Vector3D,
    pub up: Vector3D
}

impl Matrix3x3 {
    /// Identity matrix.
    pub const IDENTITY: Matrix3x3 = Matrix3x3 {
        forward: Vector3D { x: 1.0, y: 0.0, z: 0.0 },
        left: Vector3D { x: 0.0, y: 1.0, z: 0.0 },
        up: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
    };

    /// Multiply two matrices.
    #[must_use]
    pub const fn multiply(&self, by: &Self) -> Self {
        Matrix3x3 {
            forward: Vector3D {
                x: by.forward.x * self.forward.x + by.forward.y * self.left.x + by.forward.z * self.up.x,
                y: by.forward.x * self.forward.y + by.forward.y * self.left.y + by.forward.z * self.up.y,
                z: by.forward.x * self.forward.z + by.forward.y * self.left.z + by.forward.z * self.up.z
            },
            left: Vector3D {
                x: by.left.x * self.forward.x + by.left.y * self.left.x + by.left.z * self.up.x,
                y: by.left.x * self.forward.y + by.left.y * self.left.y + by.left.z * self.up.y,
                z: by.left.x * self.forward.z + by.left.y * self.left.z + by.left.z * self.up.z
            },
            up: Vector3D {
                x: by.up.x * self.forward.x + by.up.y * self.left.x + by.up.z * self.up.x,
                y: by.up.x * self.forward.y + by.up.y * self.left.y + by.up.z * self.up.y,
                z: by.up.x * self.forward.z + by.up.y * self.left.z + by.up.z * self.up.z
            }
        }
    }

    /// Interpolate this matrix by another one by `by` amount.
    #[must_use]
    pub fn interpolated(self, with: Matrix3x3, by: f32) -> Matrix3x3 {
        self.as_quaternion().interpolated(with.as_quaternion(), by).into()
    }

    /// Convert the matrix to a quaternion.
    #[must_use]
    pub fn as_quaternion(&self) -> Quaternion {
        // http://www.euclideanspace.com/maths/geometry/rotations/conversions/matrixToQuaternion/index.htm
        let tr = self.forward.x + self.left.y + self.up.z;
        if tr > 0.0 {
            let s = (tr + 1.0).fw_sqrt() * 2.0; // S=4*qw
            Quaternion {
                w: 0.25 * s,
                x: (self.up.y - self.left.z) / s,
                y: (self.forward.z - self.up.x) / s,
                z: (self.left.x - self.forward.y) / s,
            }
        }
        else if (self.forward.x > self.left.y) & (self.forward.x > self.up.z) {
            let s = (1.0 + self.forward.x - self.left.y - self.up.z).fw_sqrt() * 2.0; // S=4*qx
            Quaternion {
                w: (self.up.y - self.left.z) / s,
                x: 0.25 * s,
                y: (self.forward.y + self.left.x) / s,
                z: (self.forward.z + self.up.x) / s,
            }
        }
        else if self.left.y > self.up.z  {
            let s = (1.0 + self.left.y - self.forward.x - self.up.z).fw_sqrt() * 2.0; // S=4*qy
            Quaternion {
                w: (self.forward.z - self.up.x) / s,
                x: (self.forward.y + self.left.x) / s,
                y: 0.25 * s,
                z: (self.left.z + self.up.y) / s,
            }
        }
        else {
            let s = (1.0 + self.up.z - self.forward.x - self.left.y).fw_sqrt() * 2.0; // S=4*qz
            Quaternion {
                w: (self.left.x - self.forward.y) / s,
                x: (self.forward.z + self.up.x) / s,
                y: (self.left.z + self.up.y) / s,
                z: 0.25 * s,
            }
        }
    }

    /// Transform the vector.
    #[must_use]
    pub const fn transform_vector(&self, normal: &Vector3D) -> Vector3D {
        Vector3D {
            x: normal.x * self.forward.x + normal.y * self.left.x + normal.z * self.up.x,
            y: normal.x * self.forward.y + normal.y * self.left.y + normal.z * self.up.y,
            z: normal.x * self.forward.z + normal.y * self.left.z + normal.z * self.up.z,
        }
    }

    /// Return the matrix inverted.
    #[must_use]
    pub const fn inverted(self) -> Matrix3x3 {
        let determinant =
            self.forward.x * self.left.y * self.up.z +
            self.forward.y * self.left.z * self.up.x +
            self.forward.z * self.left.x * self.up.y -
            self.forward.x * self.left.z * self.up.y -
            self.forward.y * self.left.x * self.up.z -
            self.forward.z * self.left.y * self.up.x;

        let determinant_inverse = 1.0 / determinant;

        // SAFETY: Can be safely represented as this
        let array: [[f32; 3]; 3] = unsafe { transmute(self) };
        let mut inverse = [[0.0f32; 3]; 3];

        // Do this funky loop because const does not allow for loops yet
        let mut i = 0;
        while i < 3 {
            let mut j = 0;
            while j < 3 {
                let ip = if i < 2 { i + 1 } else { 0 };
                let im = if i > 0 { i - 1 } else { 2 };
                let jp = if j < 2 { j + 1 } else { 0 };
                let jm = if j > 0 { j - 1 } else { 2 };
                inverse[j][i] = determinant_inverse * (array[ip][jp] * array[im][jm] - array[ip][jm] * array[im][jp]);
                j += 1;
            }
            i += 1;
        }

        unsafe { transmute(inverse) }
    }
}

impl Mul<Matrix3x3> for Matrix3x3 {
    type Output = Self;

    fn mul(self, rhs: Matrix3x3) -> Self::Output {
        self.multiply(&rhs)
    }
}

/// Represents a 3D angle using four real numbers.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    /// Identity quaternion.
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    /// Square length of the quaternion.
    #[must_use]
    pub const fn square_length(self) -> f32 {
        self.dot(self)
    }

    /// Convert the quaternion to a matrix.
    #[must_use]
    pub const fn as_matrix(self) -> Matrix3x3 {
        let square_length = self.square_length();
        if square_length.is_nan() || square_length == 0.0 {
            return Matrix3x3::IDENTITY;
        }

        let doubled_inverse_square_length = 2.0 / square_length;

        let inv_x = self.x * doubled_inverse_square_length;
        let inv_y = self.y * doubled_inverse_square_length;
        let inv_z = self.z * doubled_inverse_square_length;

        let wx = self.w * inv_x;
        let wy = self.w * inv_y;
        let wz = self.w * inv_z;
        let xx = self.x * inv_x;
        let xy = self.x * inv_y;
        let xz = self.x * inv_z;
        let yy = self.y * inv_y;
        let yz = self.y * inv_z;
        let zz = self.z * inv_z;

        Matrix3x3 {
            forward: Vector3D {
                x: 1.0 - (yy + zz),
                y: xy - wz,
                z: xz + wy
            },
            left: Vector3D {
                x: xy + wz,
                y: 1.0 - (xx + zz),
                z: yz - wx
            },
            up: Vector3D {
                x: xz - wy,
                y: yz + wx,
                z: 1.0 - (xx + yy)
            }
        }
    }

    /// Normalize the quaternion.
    #[must_use]
    pub fn normalized(self) -> Quaternion {
        let square_length = self.square_length();
        if square_length <= 0.0 {
            return Self::IDENTITY
        }

        let inv = square_length.fw_inverse_sqrt();
        Self {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
            w: self.w * inv
        }
    }

    /// Interpolate this quaternion with another one by `by` amount, returning a normalized vector.
    ///
    /// This function is more accurate than [linear_interpolated_unnormalized](Self::linear_interpolated_unnormalized),
    /// but it is less performant.
    #[must_use]
    pub fn interpolated(self, b: Quaternion, by: f32) -> Quaternion {
        // special thanks to MosesOfEgypt for the rotation interpolation stuff here
        let a = self.normalized();
        let b = b.normalized();
        let mut cos_half_theta = a.dot(b);

        let mut with_n = b;
        if cos_half_theta < 0.0 {
            with_n = -with_n;
            cos_half_theta = -cos_half_theta;
        }

        if cos_half_theta.fw_fabs() < 0.01 {
            return a.linear_interpolated(b, by)
        }

        let half_theta = cos_half_theta.min(1.0).fw_acos();
        let m = 1.0 - cos_half_theta*cos_half_theta;
        let sin_half_theta = m.max(0.0);

        let mut r0 = 1.0 - by;
        let mut r1 = by;

        if sin_half_theta > 0.00001 {
            r0 = (r0 * half_theta).fw_sin() / sin_half_theta;
            r1 = (r1 * half_theta).fw_sin() / sin_half_theta;
        }

        (with_n * r1 + a * r0).normalized()
    }

    /// Linear interpolate this quaternion with another one by `by` amount, returning a normalized
    /// vector.
    ///
    /// This function is faster than [interpolate](Self::interpolated) but less accurate.
    ///
    /// This function returns a normalized vector. If one isn't necessary, use
    /// [linear_interpolated_unnormalized](Self::linear_interpolated_unnormalized).
    #[must_use]
    pub fn linear_interpolated(self, with: Quaternion, by: f32) -> Quaternion {
        self.linear_interpolated_unnormalized(with, by).normalized()
    }

    /// Linear interpolate this quaternion with another one by `by` amount, returning an
    /// unnormalized vector.
    ///
    /// This function is faster than [interpolated](Self::interpolated) but less accurate.
    ///
    /// This function returns a (most likely) unnormalized vector. If one is necessary, use
    /// [linear_interpolated](Self::linear_interpolated).
    #[must_use]
    pub fn linear_interpolated_unnormalized(self, with: Quaternion, by: f32) -> Quaternion {
        // linear interpolate; this is not very good, but this is how Halo originally does it
        let dot = self.dot(with);

        let this_amt = 1.0 - by;
        let with_amt = if dot < 0.0 {
            -by
        }
        else {
            by
        };

        self * this_amt + with * with_amt
    }

    const fn dot(self, with: Quaternion) -> f32 {
        let xx = self.x * with.x;
        let yy = self.y * with.y;
        let zz = self.z * with.z;
        let ww = self.w * with.w;
        xx + yy + zz + ww
    }

    const fn multiplied_by(self, by: f32) -> Quaternion {
        Quaternion {
            x: self.x * by,
            y: self.y * by,
            z: self.z * by,
            w: self.w * by,
        }
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: f32) -> Self::Output {
        self.multiplied_by(rhs)
    }
}

impl MulAssign<f32> for Quaternion {
    fn mul_assign(&mut self, value: f32) {
        *self = *self * value;
    }
}

impl Add<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn add(self, rhs: Quaternion) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Sub<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn sub(self, rhs: Quaternion) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl From<Matrix3x3> for Quaternion {
    fn from(value: Matrix3x3) -> Self {
        value.as_quaternion()
    }
}

impl From<Quaternion> for Matrix3x3 {
    fn from(value: Quaternion) -> Self {
        value.as_matrix()
    }
}

/// Represents a vector with two components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32
}

impl Vector2D {
    /// Vector with all components set to 0.
    pub const ZEROED: Self = Vector2D::from_scalar(0.0);

    /// Return a Vector2D with x and y set to `scalar`.
    #[inline]
    #[must_use]
    pub const fn from_scalar(scalar: f32) -> Self {
        Self { x: scalar, y: scalar }
    }

    /// Return `true` if all components of the vector are valid.
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        !self.x.is_nan() && !self.y.is_nan()
    }

    /// Return the dot product with another vector.
    #[inline]
    #[must_use]
    pub const fn dot(self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Get the magnitude squared.
    ///
    /// This is cheaper than calling [`magnitude`](Self::magnitude).
    #[inline]
    #[must_use]
    pub const fn magnitude_squared(self) -> f32 {
        self.dot(&self)
    }

    /// Get the magnitude.
    ///
    /// This is more expensive than calling [`magnitude_squared`](Self::magnitude_squared) due to
    /// having to square root the result.
    #[inline]
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.dot(&self).fw_sqrt()
    }

    /// Multiply all components with `amount`.
    #[inline]
    #[must_use]
    pub const fn scaled(self, amount: f32) -> Self {
        Self {
            x: self.x * amount,
            y: self.y * amount
        }
    }

    /// Negate the signs of all components of this vector.
    #[inline]
    #[must_use]
    pub const fn negated(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y
        }
    }

    /// Convert the vector to a unit vector, if possible.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Option<Self> {
        let magnitude = self.magnitude();
        if magnitude.fw_is_close_to(0.0) {
            None
        }
        else {
            // Bad for floating point precision, but needed to be accurate to the original...
            Some(self.scaled(1.0 / magnitude))
        }
    }

    /// Calculate the cross product with another vector (as 3D vectors) and return the Z coordinate.
    #[inline]
    #[must_use]
    pub const fn cross_product(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Offset the point `offset` world units in `direction`.
    #[inline]
    #[must_use]
    pub fn apply_offset(self, direction: Vector2D, offset: f32) -> Vector2D {
        self + direction * offset
    }
}

impl Add<Vector2D> for Vector2D {
    type Output = Self;
    fn add(self, value: Vector2D) -> Self {
        Self {
            x: self.x + value.x,
            y: self.y + value.y
        }
    }
}

impl Mul<f32> for Vector2D {
    type Output = Self;
    fn mul(self, value: f32) -> Self {
        self.scaled(value)
    }
}

impl MulAssign<f32> for Vector2D {
    fn mul_assign(&mut self, value: f32) {
        *self = *self * value;
    }
}

/// Represents a cuboid.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Cube3D {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub front: f32,
    pub back: f32
}

/// Represents a vector with four components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Vector4D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

/// Represents a projection matrix.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct ProjectionMatrix {
    pub x: Vector4D,
    pub y: Vector4D,
    pub z: Vector4D,
    pub w: Vector4D
}

/// Represents a vector with three components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3D {
    /// Vector with all components set to 0.
    pub const ZEROED: Self = Vector3D::from_scalar(0.0);

    /// Return a Vector3D with x and y set to `scalar`.
    #[inline]
    #[must_use]
    pub const fn from_scalar(scalar: f32) -> Self {
        Self { x: scalar, y: scalar, z: scalar }
    }

    /// Return `true` if all components of the vector are valid.
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        !self.x.is_nan() && !self.y.is_nan() && !self.z.is_nan()
    }

    /// Return the dot product with another vector.
    #[must_use]
    pub const fn dot(self, other: &Vector3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Multiply all components with `amount`.
    #[must_use]
    pub const fn scaled(self, by: f32) -> Self {
        Self {
            x: self.x * by,
            y: self.y * by,
            z: self.z * by
        }
    }

    /// Get the magnitude squared.
    ///
    /// This is cheaper than calling [`magnitude`](Self::magnitude).
    #[must_use]
    pub const fn magnitude_squared(self) -> f32 {
        self.dot(&self)
    }

    /// Get the magnitude.
    ///
    /// This is more expensive than calling [`magnitude_squared`](Self::magnitude_squared) due to
    /// having to square root the result.
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.magnitude_squared().fw_sqrt()
    }

    /// Interpolate this vector with another one by `by` amount.
    #[must_use]
    pub fn linear_interpolated(self, with: Vector3D, by: f32) -> Vector3D {
        let by = by.clamp(0.0, 1.0);
        let a = by;
        let b = 1.0 - by;
        self * b + with * a
    }

    /// Convert the vector to a unit vector, if possible.
    #[must_use]
    pub fn normalized(self) -> Option<Self> {
        let magnitude = self.magnitude();
        if magnitude.fw_is_close_to(0.0) {
            None
        }
        else {
            // Bad for floating point precision, but needed to be accurate to the original...
            Some(self.scaled(1.0 / magnitude))
        }
    }

    /// Negate the signs of all components of this vector.
    #[inline]
    #[must_use]
    pub const fn negated(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }

    /// Calculate the cross product with another vector (as 3D vectors) and return the Z coordinate.
    #[inline]
    #[must_use]
    pub const fn cross_product(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    /// Return the value represented as a [`Euler2D`].
    #[inline]
    #[must_use]
    pub fn as_euler_angles(self) -> Euler2D {
        Euler2D {
            yaw: self.y.fw_atan2(self.x),
            pitch: self.z.fw_atan2(Vector2D { x: self.x, y: self.y }.magnitude())
        }
    }

    /// Offset the point `offset` world units in `direction`.
    #[inline]
    #[must_use]
    pub fn apply_offset(self, direction: Vector3D, offset: f32) -> Vector3D {
        self + direction * offset
    }
}

impl Default for Vector3D {
    fn default() -> Self {
        Self::ZEROED
    }
}

impl Neg for Vector3D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negated()
    }
}

impl Add<Vector3D> for Vector3D {
    type Output = Self;

    fn add(self, value: Vector3D) -> Self::Output {
        Self {
            x: self.x + value.x,
            y: self.y + value.y,
            z: self.z + value.z,
        }
    }
}

impl Sub<Vector3D> for Vector3D {
    type Output = Self;

    fn sub(self, value: Vector3D) -> Self::Output {
        Self {
            x: self.x - value.x,
            y: self.y - value.y,
            z: self.z - value.z,
        }
    }
}

impl Mul<f32> for Vector3D {
    type Output = Self;

    fn mul(self, value: f32) -> Self::Output {
        self.scaled(value)
    }
}

impl MulAssign<f32> for Vector3D {
    fn mul_assign(&mut self, value: f32) {
        *self = *self * value;
    }
}

impl Display for Vector3D {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("({x},{y},{z})", x=self.x, y=self.y, z=self.z))
    }
}

/// Represents a two-component vector using 16-bit ints (i.e. an X and Y coordinate in pixels).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Vector2DInt {
    pub x: i16,
    pub y: i16
}

/// Represents a rotation using an Euler angle, besides roll.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Euler2D {
    pub yaw: f32,
    pub pitch: f32
}

impl Euler2D {
    /// Convert the Euler2D into a 3D vector.
    #[inline]
    #[must_use]
    pub fn as_vector(self) -> Vector3D {
        let sine_pitch = self.pitch.fw_sin();
        let cosine_pitch = self.pitch.fw_cos();
        let sine_yaw = self.yaw.fw_sin();
        let cosine_yaw = self.yaw.fw_cos();

        Vector3D {
            x: cosine_yaw * cosine_pitch,
            y: sine_yaw * cosine_pitch,
            z: sine_pitch
        }
    }
}

impl From<Euler2D> for Vector3D {
    #[inline]
    fn from(value: Euler2D) -> Self {
        value.as_vector()
    }
}

impl From<Vector3D> for Euler2D {
    #[inline]
    fn from(value: Vector3D) -> Self {
        value.as_euler_angles()
    }
}

/// Represents a rotation using an Euler angle.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Euler3D {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32
}

/// Represents a 2D plane.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Plane2D {
    pub offset: f32,
    pub vector: Vector2D
}

/// Represents a 3D plane.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Plane3D {
    pub vector: Vector3D,
    pub offset: f32,
}
impl Plane3D {
    /// Get the distance `point` is from this plane.
    #[must_use]
    pub const fn distance_to_point(self, point: Vector3D) -> f32 {
        point.dot(&self.vector) - self.offset
    }
}

/// Angle value.
///
/// Internally represents a value in radians.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Default)]
#[repr(transparent)]
pub struct Angle(pub f32);

impl Angle {
    /// The default horizontal FoV in degrees (70 degrees) for the game.
    pub const DEFAULT_HORIZONTAL_FOV: Angle = Angle::from_degrees(70.0);

    /// The default vertical FoV in degrees (~55.41 degrees) for the game.
    pub const DEFAULT_VERTICAL_FOV: Angle = Angle::from_radians(0.96713803047123473857584761442933284839190937900591636936069359052097036749);

    /// 0 degrees.
    pub const _0_DEG: Angle = Angle::from_radians(0.0);

    /// 45 degrees.
    pub const _45_DEG: Angle = Angle::from_radians(f32::FW_QUARTER_PI);

    /// 90 degrees.
    pub const _90_DEG: Angle = Angle::from_radians(f32::FW_HALF_PI);

    /// 180 degrees.
    pub const _180_DEG: Angle = Angle::from_radians(f32::FW_PI);

    /// 360 degrees.
    pub const _360_DEG: Angle = Angle::from_radians(f32::FW_2PI);

    /// Calculate a vertical FoV from a horizontal FoV.
    #[must_use]
    pub fn calculate_vertical_fov(self, aspect_ratio: f32) -> Angle {
        Self::from_radians(2.0 * ((self.radians() / 2.0).fw_tan() / aspect_ratio).fw_atan())
    }

    /// Calculate a horizontal FoV from a vertical FoV.
    #[must_use]
    pub fn calculate_horizontal_fov(self, aspect_ratio: f32) -> Angle {
        Self::from_radians(2.0 * ((self.radians() / 2.0).fw_tan() * aspect_ratio).fw_atan())
    }

    /// Calculate a horizontal FoV from one aspect ratio to another.
    ///
    /// The resulting FoV will have the same vertical FoV.
    #[must_use]
    pub fn convert_horizontal_fov(self, from_aspect_ratio: f32, to_aspect_ratio: f32) -> Angle {
        self.calculate_vertical_fov(from_aspect_ratio).calculate_horizontal_fov(to_aspect_ratio)
    }

    /// Compute an angle from the given degrees.
    #[must_use]
    pub const fn from_degrees(deg: f32) -> Self {
        Self::from_radians(deg * f32::FW_RADIANS_PER_DEGREE)
    }

    /// Compute an angle from the given radians.
    ///
    /// This is provided for completion, as this is simply the same thing as using `Self(rad)`.
    #[must_use]
    pub const fn from_radians(rad: f32) -> Self {
        Self(rad)
    }

    /// Get the value as degrees.
    #[must_use]
    pub const fn degrees(self) -> f32 {
        // Use a constant for multiplying to ensure accuracy/precision with tool.exe
        self.0 * f32::FW_DEGREES_PER_RADIAN
    }

    /// Get the value as radians.
    ///
    /// This is provided for completion, as this is simply the same thing as using `self.0`.
    #[must_use]
    pub const fn radians(self) -> f32 {
        self.0
    }
}
impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}°", self.degrees()))
    }
}
impl Debug for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Neg for Angle {
    type Output = Angle;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Represents a compressed 16-bit float.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(transparent)]
pub struct CompressedFloat(pub u16);

/// Represents a [`Vector2D`] compressed into 32 bits.
///
/// Internally it is `Y8.X8`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(transparent)]
pub struct CompressedVector2D(pub u32);

/// Represents a [`Vector3D`] compressed into 32 bits.
///
/// Internally it is `Z10.Y11.X11`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(transparent)]
pub struct CompressedVector3D(pub u32);

/// Matrix3x3 with position and scale component.
///
/// Represents a basic 3D transformation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Matrix4x3 {
    pub scale: f32,
    pub rotation: Matrix3x3,
    pub position: Vector3D
}

impl Matrix4x3 {
    /// Instantiate using a [`Matrix3x3`], setting `scale` to 1.0 and `position` to [`Vector3D::ZEROED`]
    #[must_use]
    pub const fn from_matrix3x3(matrix3x3: Matrix3x3) -> Self {
        Self {
            scale: 1.0,
            rotation: matrix3x3,
            position: Vector3D::ZEROED
        }
    }

    /// Multiply with another transformation.
    #[must_use]
    pub const fn multiply(&self, by: &Self) -> Self {
        Self {
            scale: self.scale * by.scale,
            position: Vector3D {
                x: (by.position.x * self.rotation.forward.x + by.position.y * self.rotation.left.x + by.position.z * self.rotation.up.x) * self.scale + self.position.x,
                y: (by.position.x * self.rotation.forward.y + by.position.y * self.rotation.left.y + by.position.z * self.rotation.up.y) * self.scale + self.position.y,
                z: (by.position.x * self.rotation.forward.z + by.position.y * self.rotation.left.z + by.position.z * self.rotation.up.z) * self.scale + self.position.z
            },
            rotation: self.rotation.multiply(&by.rotation)
        }
    }

    /// Transform a normal using rotation.
    #[must_use]
    pub const fn transform_normal(&self, normal: &Vector3D) -> Vector3D {
        self.rotation.transform_vector(normal)
    }

    /// Transform a plane applying rotation, scale, and position.
    #[must_use]
    pub const fn transform_plane(&self, plane: &Plane3D) -> Plane3D {
        let vector = self.transform_normal(&plane.vector);
        Plane3D {
            vector,
            offset: self.scale * plane.offset + self.position.dot(&vector)
        }
    }

    /// Transform the vector, applying scale and rotation.
    #[must_use]
    pub fn transform_vector(&self, vector: &Vector3D) -> Vector3D {
        let point_scaled = *vector * self.scale;
        self.rotation.transform_vector(&point_scaled)
    }

    /// Transform the point, applying scale, rotation, and position.
    #[must_use]
    pub fn transform_point(&self, point: &Vector3D) -> Vector3D {
        self.transform_vector(point) + self.position
    }
    /// Instantiate a matrix from a point and rotation.
    #[must_use]
    pub const fn from_point_and_quaternion(point: Vector3D, quaternion: Quaternion) -> Self {
        Self {
            position: point,
            ..Self::from_matrix3x3(quaternion.as_matrix())
        }
    }
    /// Interpolate this matrix by another one by `by` amount.
    #[must_use]
    pub fn interpolated(&self, with: &Matrix4x3, by: f32) -> Matrix4x3 {
        let by = by.clamp(0.0, 1.0);
        Self {
            scale: (1.0 - by) * self.scale + by * with.scale,
            position: self.position.linear_interpolated(with.position, by),
            rotation: self.rotation.interpolated(with.rotation, by)
        }
    }
}

impl Mul<Matrix4x3> for Matrix4x3 {
    type Output = Self;

    fn mul(self, rhs: Matrix4x3) -> Self::Output {
        self.multiply(&rhs)
    }
}

impl From<Matrix3x3> for Matrix4x3 {
    fn from(value: Matrix3x3) -> Self {
        Self::from_matrix3x3(value)
    }
}

const _: () = assert!(size_of::<Matrix4x3>() == 0x34);

#[cfg(test)]
mod test {
    use crate::vector::Angle;

    #[test]
    fn check_angle_constants() {
        assert_eq!(Angle::_0_DEG, Angle::from_degrees(0.0), "0 degrees did not match?");
        assert_eq!(Angle::_45_DEG, Angle::from_degrees(45.0), "45 degrees did not match?");
        assert_eq!(Angle::_90_DEG, Angle::from_degrees(90.0), "90 degrees did not match?");
        assert_eq!(Angle::_180_DEG, Angle::from_degrees(180.0), "180 degrees did not match?");
        assert_eq!(Angle::_360_DEG, Angle::from_degrees(360.0), "360 degrees did not match?");
    }
}
