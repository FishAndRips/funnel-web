//! Module for [`UTF8Replacer`]
//!
//! Used for improved terminal compatibility.

use core::fmt::{Display, Formatter};
use spin::rwlock::RwLock;

static CURRENT_REPLACER_MODE: spin::RwLock<UTF8ReplacerMode> = RwLock::new(UTF8ReplacerMode::UTF8);

/// Represents `˚` as `deg` in ASCII
pub const UTF8_DEGREES: UTF8Replacer = UTF8Replacer::make("˚", "deg");

/// Represents `≥` as `>=` in ASCII
pub const UTF8_GREATER_THAN_OR_EQUAL_TO: UTF8Replacer = UTF8Replacer::make("≥", ">=");

/// Represents `≥` as `<=` in ASCII
pub const UTF8_LESS_THAN_OR_EQUAL_TO: UTF8Replacer = UTF8Replacer::make("≤", "<=");

/// Determines what mode to display [`UTF8Replacer`] in.
///
/// Use [`UTF8Replacer::set_mode`] to change the mode.
#[derive(Copy, Clone, Debug, Default)]
pub enum UTF8ReplacerMode {
    /// Display in UTF-8 mode.
    ///
    /// This is the default mode.
    #[default]
    UTF8,

    /// Display in ASCII mode.
    ASCII,
}

/// UTF-8 replacer
#[derive(Copy, Clone)]
pub struct UTF8Replacer {
    ascii: &'static str,
    utf8: &'static str
}

impl UTF8Replacer {
    /// Set the current replacement mode.
    ///
    /// Note: This is thread-safe, but it is not immune to race conditions.
    pub fn set_mode(mode: UTF8ReplacerMode) {
        *CURRENT_REPLACER_MODE.write() = mode
    }

    /// Get the current replacement mode.
    ///
    /// Note: This is thread-safe, but it is not immune to race conditions.
    pub fn get_mode() -> UTF8ReplacerMode {
        *CURRENT_REPLACER_MODE.read()
    }

    pub(crate) const fn make(utf8: &'static str, ascii: &'static str) -> Self {
        Self { ascii, utf8 }
    }
}

impl Display for UTF8Replacer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match Self::get_mode() {
            UTF8ReplacerMode::UTF8 => f.write_str(self.utf8),
            UTF8ReplacerMode::ASCII => f.write_str(self.ascii)
        }
    }
}
