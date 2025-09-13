/// ID primitive
///
/// `SALT` allows the type of ID to be compared (besides null IDs). This is typically the first two
/// ASCII letters of a table's name read in little endian.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct ID<const SALT: u16>(u32);

pub const NULL_ID: u32 = 0xFFFFFFFF;

impl<const SALT: u16> Default for ID<SALT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SALT: u16> ID<SALT> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(NULL_ID)
    }

    #[inline(always)]
    pub const fn from_index(index: Index) -> Self {
        match index.0 {
            u16::MAX => Self(NULL_ID),
            index => {
                let salt = (SALT ^ index) | 0x8000;
                Self((salt as u32) << 16 | (index as u32))
            }
        }
    }

    #[inline(always)]
    pub const fn from_usize(index: usize) -> Option<Self> {
        match Index::from_usize(index) {
            Some(index) => Some(Self::from_index(index)),
            None => None
        }
    }

    #[inline(always)]
    pub const fn from_u32(id: u32) -> Option<Self> {
        match id {
            // Zeroed out has a special meaning for this toolset: it's treated as null
            0 => Some(Self(NULL_ID)),

            // These are all invalid
            ..=0x7FFFFFFF => None,

            // Actually valid
            0x80000000.. => Some(Self(id))
        }
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.0 == NULL_ID
    }

    #[inline(always)]
    pub const fn index(self) -> Option<usize> {
        match self {
            Self(NULL_ID) => None,
            Self(n) => Some((n & (u16::MAX as u32)) as usize)
        }
    }

    #[inline(always)]
    pub const fn full_id(self) -> u32 {
        self.0
    }
}

pub type TagID = ID<0x6174>;
pub type ScriptNodeID = ID<0x6373>;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct Index(pub u16);

impl Index {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0xFFFF)
    }

    #[inline(always)]
    pub const fn from_usize(index: usize) -> Option<Self> {
        if index >= u16::MAX as usize {
            return None
        }
        Some(Self(index as u16))
    }

    #[inline(always)]
    pub const fn index(self) -> Option<usize> {
        match self.0 {
            u16::MAX => None,
            n => Some(n as usize)
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
