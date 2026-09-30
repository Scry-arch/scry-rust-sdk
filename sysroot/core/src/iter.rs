//! Iteration: the `Iterator` and `IntoIterator` traits the compiler desugars
//! `for` into, iteration of `a..b` and `a..=b` over the <=32-bit integer
//! types, and the two adapters `rev` and `enumerate`. Slices are iterable
//! through `crate::slice`.

use crate::marker::Sized;
use crate::ops::{Range, RangeInclusive};
use crate::option::Option::{self, None, Some};

#[lang = "iterator"]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub trait Iterator {
    type Item;

    #[lang = "next"]
    fn next(&mut self) -> Option<Self::Item>;

    /// Iterates from the back instead: `for i in (0..n).rev()`.
    #[inline]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedIterator,
    {
        Rev { iter: self }
    }

    /// Pairs every item with its position, counting from 0:
    /// `for (i, x) in slice.iter().enumerate()`.
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate { iter: self, count: 0 }
    }
}

/// An iterator that can also produce items from its back end.
pub trait DoubleEndedIterator: Iterator {
    fn next_back(&mut self) -> Option<Self::Item>;
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

    #[inline]
    fn into_iter(self) -> I {
        self
    }
}

impl<'a, I: Iterator> Iterator for &'a mut I {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        (**self).next()
    }
}

// ---- adapters ----

/// The iterator returned by [`Iterator::rev`].
pub struct Rev<I> {
    iter: I,
}

impl<I: DoubleEndedIterator> Iterator for Rev<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        self.iter.next_back()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for Rev<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}

/// The iterator returned by [`Iterator::enumerate`].
pub struct Enumerate<I> {
    iter: I,
    count: usize,
}

impl<I: Iterator> Iterator for Enumerate<I> {
    type Item = (usize, I::Item);

    #[inline]
    fn next(&mut self) -> Option<(usize, I::Item)> {
        match self.iter.next() {
            Some(item) => {
                let i = self.count;
                self.count = i + 1;
                Some((i, item))
            }
            None => None,
        }
    }
}

// ---- ranges ----

// Real core goes through the `Step` trait here. Ranges are only iterable for
// the integer types that have arithmetic at all, so implement them directly.
macro_rules! range_iter_impls {
    ($($t:ty)*) => {$(
        impl Iterator for Range<$t> {
            type Item = $t;

            #[inline]
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

        impl DoubleEndedIterator for Range<$t> {
            #[inline]
            fn next_back(&mut self) -> Option<$t> {
                if self.start < self.end {
                    self.end = self.end - 1;
                    Some(self.end)
                } else {
                    None
                }
            }
        }

        impl Iterator for RangeInclusive<$t> {
            type Item = $t;

            #[inline]
            fn next(&mut self) -> Option<$t> {
                if self.exhausted || self.start > self.end {
                    None
                } else if self.start < self.end {
                    let n = self.start;
                    self.start = n + 1;
                    Some(n)
                } else {
                    // Produce `end` without stepping past it, which would
                    // overflow for a range ending at the type's maximum.
                    self.exhausted = true;
                    Some(self.start)
                }
            }
        }

        impl DoubleEndedIterator for RangeInclusive<$t> {
            #[inline]
            fn next_back(&mut self) -> Option<$t> {
                if self.exhausted || self.start > self.end {
                    None
                } else if self.start < self.end {
                    let n = self.end;
                    self.end = n - 1;
                    Some(n)
                } else {
                    self.exhausted = true;
                    Some(self.end)
                }
            }
        }
    )*};
}

range_iter_impls! { u8 u16 u32 usize i8 i16 i32 isize }
