//! String slices: their length, their bytes and equality. There is no UTF-8
//! processing: a Scry program handles text as bytes.

use crate::cmp::{Eq, PartialEq};
use crate::intrinsics;

impl str {
    /// The length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The UTF-8 bytes of the string.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // `str` has the same layout as `[u8]`.
        unsafe { intrinsics::transmute(self) }
    }
}

impl PartialEq for str {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for str {}
