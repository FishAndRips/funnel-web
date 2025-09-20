//! Home of the [`Rectangle`] type.

/// Represents a 2D rectangle.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
#[expect(missing_docs)]
pub struct Rectangle {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16
}
impl Rectangle {
    /// Create a rectangle at (0,0) in the top-left corner with the given width and height.
    ///
    /// This is a convenience function for
    /// ```text
    /// Rectangle {
    ///     top: 0,
    ///     left: 0,
    ///     bottom: height,
    ///     right: width
    /// }
    /// ```
    #[inline]
    pub const fn from_width_and_height(width: i16, height: i16) -> Self {
        Rectangle {
            top: 0,
            left: 0,
            bottom: height,
            right: width
        }
    }

    /// Get the width of the rectangle.
    #[inline]
    pub const fn width(self) -> u16 {
        (self.right as i32).wrapping_sub(self.left as i32) as u16
    }

    /// Get the height of the rectangle.
    #[inline]
    pub const fn height(self) -> u16 {
        (self.bottom as i32).wrapping_sub(self.top as i32) as u16
    }

    /// Center a rectangle inside a rectangle.
    #[inline]
    pub const fn centered_inside(self, what: Rectangle) -> Self {
        let left_offset = ((what.width() / 2) as i32) - ((self.width() / 2) as i32);
        let top_offset = ((what.height() / 2) as i32) - ((self.height() / 2) as i32);
        Rectangle {
            top: self.top.wrapping_add(top_offset as i16),
            left: self.left.wrapping_add(left_offset as i16),
            bottom: self.bottom.wrapping_add(top_offset as i16),
            right: self.right.wrapping_add(left_offset as i16),
        }
    }

    /// Get the aspect ratio of the rectangle.
    ///
    /// # Panics
    ///
    /// Panics if height or width are 0.
    #[inline]
    pub const fn get_aspect_ratio(self) -> f32 {
        let width = self.width();
        let height = self.height();
        assert!(width > 0 && height > 0, "self.width and self.height must be at least 0");
        (width as f32) / (height as f32)
    }
}
