//! Provides operations for colors.

/// [`ColorRGB`] with alpha (transparency) component
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ColorARGB {
    /// Alpha channel in the range of `[0..1]`
    pub a: f32,

    /// Color values
    pub color: ColorRGB
}
impl AsRef<ColorARGB> for ColorARGB {
    fn as_ref(&self) -> &ColorARGB {
        self
    }
}
impl ColorARGB {
    /// Instantiates a color with all channels set to 0.0 (i.e. transparent [`ColorRGB::BLACK`]).
    #[must_use] 
    pub const fn new() -> Self {
        ColorARGB {
            a: 0.0,
            color: ColorRGB::BLACK
        }
    }

    /// Returns `true` the color is valid (i.e. all channels are between 0 and 1, inclusive).
    #[must_use] 
    pub const fn is_valid(&self) -> bool {
        self.a >= 0.0 && self.a <= 1.0 && self.color.is_valid()
    }

    /// Convert the value to a [`Pixel32`].
    ///
    /// # Panics
    ///
    /// Panics if `!self.is_valid()`.
    #[must_use] 
    pub const fn to_pixel32(&self) -> Pixel32 {
        assert!(self.is_valid());

        let a = (self.a * 255.0) as u32;
        let r = (self.color.r * 255.0) as u32;
        let g = (self.color.g * 255.0) as u32;
        let b = (self.color.b * 255.0) as u32;
        Pixel32((a << 24) | (r << 16) | (g << 8) | b)
    }

    /// Clamps all values to between 0 and 1 (inclusive).
    #[must_use] 
    pub const fn clamped(self) -> ColorARGB {
        ColorARGB {
            a: self.a.clamp(0.0, 1.0),
            color: self.color.clamped()
        }
    }
}

/// Three channel color represented in three floats.
///
/// Although it is stored as 32 bits per color, it does not have the same depth as 32-bit ints, as
/// its actual representation as a float is limited to values between 0 and 1 (inclusive).
///
/// This means two bits of the float are completely unused (the sign bit and the uppermost bit of
/// the exponent, representing negative numbers and values ≥ 2, respectively). Additionally, the
/// other seven bits of the exponent cannot all be set at the same time unless the mantissa is zero,
/// as this would make the value greater than 1.
///
/// This essentially puts it as slightly less than a 30-bit value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ColorRGB {
    /// Red channel in the range of `[0..1]`
    pub r: f32,

    /// Green channel in the range of `[0..1]`
    pub g: f32,

    /// Blue channel in the range of `[0..1]`
    pub b: f32
}

impl ColorRGB {
    /// The color white (rgb = 1.0).
    pub const WHITE: ColorRGB = ColorRGB { r: 1.0, g: 1.0, b: 1.0 };

    /// The color black (rgb = 0.0).
    pub const BLACK: ColorRGB = ColorRGB { r: 0.0, g: 0.0, b: 0.0 };

    /// Instantiate a color with all channels set to 0 (i.e. [`ColorRGB::BLACK`]).
    #[must_use] 
    pub const fn new() -> Self {
        Self::BLACK
    }

    /// Returns `true` if the color is valid.
    #[must_use] 
    pub const fn is_valid(&self) -> bool {
        self.r >= 0.0 && self.r <= 1.0
            && self.g >= 0.0 && self.g <= 1.0
            && self.b >= 0.0 && self.b <= 1.0
    }

    /// Converts the color to a [`ColorARGB`] with alpha set to 1 (fully opaque).
    #[must_use] 
    pub const fn as_colorargb(self) -> ColorARGB {
        ColorARGB { a: 1.0, color: self }
    }

    /// Clamps all values to between 0 and 1 (inclusive).
    #[must_use] 
    pub const fn clamped(self) -> ColorRGB {
        ColorRGB { r: self.r.clamp(0.0, 1.0), g: self.g.clamp(0.0, 1.0), b: self.b.clamp(0.0, 1.0) }
    }
}

impl From<ColorRGB> for ColorARGB {
    fn from(value: ColorRGB) -> Self {
        value.as_colorargb()
    }
}

/// Represents a single 32BPP A8R8G8B8 color value.
///
/// Can be represented in binary form as `0xAARRGGBB`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(transparent)]
pub struct Pixel32(pub u32);
