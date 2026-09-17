//! Iteration. Enough for `for` loops over ranges: the `Iterator` and
//! `IntoIterator` traits the compiler desugars `for` into, and `Iterator` for
//! `Range` of the <=32-bit integer types. No adapters (`map`, `rev`, ...) yet.

use crate::ops::Range;
use crate::option::Option::{self, None, Some};

#[lang = "iterator"]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub trait Iterator {
    type Item;

    #[lang = "next"]
    fn next(&mut self) -> Option<Self::Item>;
}

pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    #[lang = "into_iter"]
    fn into_iter(self) -> Self::IntoIter;
}

impl<I: Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;

    fn into_iter(self) -> I {
        self
    }
}

impl<'a, I: Iterator> Iterator for &'a mut I {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        (**self).next()
    }
}

// Real core goes through the `Step` trait here. Ranges are only iterable for
// the integer types that have arithmetic at all, so implement them directly.
macro_rules! range_iter_impls {
    ($($t:ty)*) => {$(
        impl Iterator for Range<$t> {
            type Item = $t;

            fn next(&mut self) -> Option<$t> {
                if self.start < self.end {
                    let n = self.start;
                    self.start = n + 1;
                    Some(n)
                } else {
                    None
                }
            }
        }
    )*};
}

range_iter_impls! { u8 u16 u32 usize i8 i16 i32 isize }
