//! ID and index primitives.

/// ID primitive
///
/// Can address up to 65536 items.
///
/// `SALT` allows the type of ID to be compared (besides null IDs). This is typically the first two
/// ASCII letters of a table's name read in little endian.
///
/// `u32::MAX` ([`NULL_ID`]) is a null ID for any salt type.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct ID<const SALT: u16>(u32);

/// Represents a null ID for any salt type.
pub const NULL_ID: u32 = 0xFFFFFFFF;

impl<const SALT: u16> Default for ID<SALT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SALT: u16> ID<SALT> {
    /// Instantiate a null ID.
    #[inline(always)]
    pub const fn new() -> Self {
        Self(NULL_ID)
    }

    /// Create an ID from an [`Index`].
    ///
    /// A null index will result in a null ID.
    #[inline(always)]
    pub const fn from_index(index: Index) -> Self {
        match index.index() {
            None => Self(NULL_ID),
            Some(index) => Self::id_from_index_value(index as u16)
        }
    }

    /// Create an ID from a [`usize`].
    ///
    /// Returns `None` if `index` is out-of-bounds for an id.
    #[inline(always)]
    pub const fn from_usize(index: usize) -> Option<Self> {
        if index > u16::MAX as usize {
            return None
        }
        Some(Self::id_from_index_value(index as u16))
    }

    /// Create an ID from a [`u32`].
    ///
    /// Returns `None` if `id` has the wrong salt.
    #[inline(always)]
    pub const fn from_u32(id: u32) -> Option<Self> {
        // If null or zeroed out, it's null.
        if id == 0 || id == NULL_ID {
            return Some(Self(NULL_ID))
        }

        // If the salt is wrong, no.
        let index = Self::id_from_index_value(id as u16);
        if index.as_u32() == id {
            Some(index)
        }
        else {
            None
        }
    }

    /// Returns the binary representation of the ID.
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns true if null.
    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.0 == NULL_ID
    }

    /// Returns the ID as a [`usize`] index.
    #[inline(always)]
    pub const fn index(self) -> Option<usize> {
        match self {
            Self(NULL_ID) => None,
            Self(n) => Some((n & (u16::MAX as u32)) as usize)
        }
    }

    #[inline(always)]
    const fn id_from_index_value(value: u16) -> Self {
        let salt = (SALT ^ value) | 0x8000;
        Self((salt as u32) << 16 | (value as u32))
    }
}

/// Represents a tag ID.
pub type TagID = ID<0x6174>;

/// Represents a script node ID.
pub type ScriptNodeID = ID<0x6373>;

/// Index type.
///
/// Can address up to 65535 elements.
///
/// [`u16::MAX`] is treated as null.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct Index(pub u16);

impl Index {
    /// Create a null index.
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0xFFFF)
    }

    /// Instantiate an index from a [`usize`] index.
    ///
    /// Returns `None` if `index` is greater than or equal to [`u16::MAX`]. This is because 65535 is
    /// treated as null, and anything higher is too high to be addressable with a 16-bit integer.
    #[inline(always)]
    pub const fn from_usize(index: usize) -> Option<Self> {
        if index >= u16::MAX as usize {
            return None
        }
        Some(Self(index as u16))
    }

    /// Returns true if null.
    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.0 == u16::MAX
    }

    /// Returns the value as a [`usize`] index.
    #[inline(always)]
    pub const fn index(self) -> Option<usize> {
        if self.is_null() {
            None
        }
        else {
            Some(self.0 as usize)
        }
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::id::{Index, TagID};

    #[test]
    fn expected_tag_id_matches() {
        assert_eq!(TagID::from_usize(0).unwrap().as_u32(), 0xE1740000);
        assert_eq!(TagID::from_usize(1).unwrap().as_u32(), 0xE1750001);
        assert_eq!(TagID::from_usize(0x8000).unwrap().as_u32(), 0xE1748000);
        assert_eq!(TagID::from_usize(0x8001).unwrap().as_u32(), 0xE1758001);
        assert_eq!(TagID::from_u32(0xE1750001).unwrap().as_u32(), 0xE1750001);
        assert_eq!(TagID::new().as_u32(), 0xFFFFFFFF);
        assert_eq!(TagID::from_index(Index::from_usize(1).unwrap()).as_u32(), 0xE1750001);

        assert!(TagID::from_u32(0xFFFFFFFF).unwrap().is_null());
        assert!(TagID::from_index(Index::new()).is_null());
    }
}
