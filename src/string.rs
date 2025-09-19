use alloc::string::String;
use core::ffi::{c_char, CStr};
use core::fmt::{Debug, Display, Formatter};

/// Null-terminated multi character ASCII string.
///
/// This string is guaranteed to be entirely composed of ASCII characters without any control
/// characters. Also, its length is always less than `LEN`, with the last byte in the buffer being
/// `0x00` (as well as all bytes between the end of the string and the last byte in the buffer).
#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[repr(transparent)]
pub struct ASCIIString<const LEN: usize>([u8; LEN]);

impl<const LEN: usize> ASCIIString<LEN> {
    /// Instantiate an empty ASCIIString.
    #[inline(always)]
    pub const fn new() -> Self {
        assert!(LEN > 0, "LEN must be nonzero");

        Self([0u8; LEN])
    }

    /// Instantiate an ASCIIString from bytes.
    ///
    /// Returns `None` if the bytes contain control or non-ASCII characters or the string is not
    /// null terminated.
    ///
    /// # Panics
    ///
    /// Panics if `LEN == 0`
    pub const fn from_bytes(mut bytes: [u8; LEN]) -> Option<Self> {
        assert!(LEN > 0, "LEN must be nonzero");

        // Must be null terminated
        if bytes[LEN - 1] != 0 {
            return None
        }

        let mut q = 0usize;
        while q < LEN {
            let byte = bytes[q];

            // found the null terminator; zero out the rest
            if byte == 0 {
                q += 1;
                while q < LEN {
                    bytes[q] = 0;
                    q += 1;
                }
                break
            }

            // control characters are banned
            if !byte.is_ascii() || byte.is_ascii_control() {
                return None
            }

            q += 1;
        }

        Some(Self(bytes))
    }

    /// Instantiate an empty ASCIIString.
    ///
    /// Returns `None` if the string is longer than the maximum length or is non-ASCII.
    ///
    /// # Panics
    ///
    /// Panics if `LEN == 0`
    pub const fn from_str(str: &str) -> Option<Self> {
        assert!(LEN > 0, "LEN must be nonzero");

        let str_bytes = str.as_bytes();
        let len = str_bytes.len();
        if len >= LEN {
            return None
        }

        let mut bytes = [0u8; LEN];
        let mut q = 0;
        while q < len {
            bytes[q] = str_bytes[q];
            q += 1;
        }

        Self::from_bytes(bytes)
    }

    /// Get the full bytes buffer.
    #[inline(always)]
    pub const fn bytes(&self) -> &[u8; LEN] {
        &self.0
    }

    /// Get the length of the string.
    pub const fn string_len(&self) -> usize {
        self.as_cstr().to_bytes().len()
    }

    /// Get the string data as a string.
    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        let str_bytes = self.as_cstr().to_bytes();

        // SAFETY: All constructors also ensure this is ASCII, thus it should also be UTF-8.
        unsafe {
            core::str::from_utf8_unchecked(str_bytes)
        }
    }

    /// Get the string data as a CStr.
    #[inline(always)]
    pub const fn as_cstr(&self) -> &CStr {
        // SAFETY: All constructors ensure that this is a null-terminated C string.
        unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
        }
    }
}

impl<const LEN: usize> Default for ASCIIString<LEN> {
    fn default() -> Self {
        Self::new()
    }
}

/// Common string type used in tag data.
///
/// This is often used for identifying reflexives, but it has other purposes, too, such as being
/// used for the scenario name and build string in cache file headers.
pub type String32 = ASCIIString<32>;

impl<const LEN: usize> Display for ASCIIString<LEN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const LEN: usize> Debug for ASCIIString<LEN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl<const LEN: usize> AsRef<str> for ASCIIString<LEN> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const LEN: usize> AsRef<CStr> for ASCIIString<LEN> {
    fn as_ref(&self) -> &CStr {
        self.as_cstr()
    }
}

impl<const LEN: usize> PartialEq<str> for ASCIIString<LEN> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const LEN: usize> PartialEq<ASCIIString<LEN>> for str {
    fn eq(&self, other: &ASCIIString<LEN>) -> bool {
        self == other.as_str()
    }
}

impl<const LEN: usize> PartialEq<&str> for ASCIIString<LEN> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const LEN: usize> PartialEq<ASCIIString<LEN>> for &str {
    fn eq(&self, other: &ASCIIString<LEN>) -> bool {
        *self == other.as_str()
    }
}

impl<const LEN: usize> PartialEq<String> for ASCIIString<LEN> {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl<const LEN: usize> PartialEq<ASCIIString<LEN>> for String {
    fn eq(&self, other: &ASCIIString<LEN>) -> bool {
        self == other.as_str()
    }
}

#[cfg(test)]
mod test {
    use crate::string::String32;

    #[test]
    fn string32_test() {
        assert_eq!(String32::new(), "");
        assert_eq!(String32::from_str("this is a string").unwrap(), "this is a string");
        assert_eq!(String32::from_str("this is a string").unwrap(), "this is a string");
    }
}
