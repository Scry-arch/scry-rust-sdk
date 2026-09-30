//! Slices: the common methods of `[T]`, iteration over slices and arrays, and
//! indexing with ranges (`s[a..b]`, `s[a..]`, `s[..b]`, `s[..]`, `s[a..=b]`).
//! Arrays get all of it through their coercion to slices.

use crate::clone::Clone;
use crate::intrinsics;
use crate::iter::{DoubleEndedIterator, IntoIterator, Iterator};
use crate::marker::{Copy, PhantomData};
use crate::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo};
use crate::option::Option::{self, None, Some};
use crate::panicking::panic;
use crate::ptr;

impl<T> [T] {
    /// The number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        intrinsics::ptr_metadata(self as *const [T])
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self as *const [T] as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut [T] as *mut T
    }

    /// The element at `index`, or `None` if it is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() { Some(&self[index]) } else { None }
    }

    /// The element at `index`, or `None` if it is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() { Some(&mut self[index]) } else { None }
    }

    /// Iterates over references to the elements.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { ptr: self.as_ptr(), remaining: self.len(), _marker: PhantomData }
    }

    /// Iterates over mutable references to the elements.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let remaining = self.len();
        IterMut { ptr: self.as_mut_ptr(), remaining, _marker: PhantomData }
    }

    /// Swaps two elements. Panics if either index is out of bounds.
    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        let len = self.len();
        if a >= len || b >= len {
            panic("swap index out of bounds");
        }
        let base = self.as_mut_ptr();
        unsafe { ptr::swap(base.add(a), base.add(b)) }
    }

    /// Sets every element to a clone of `value`.
    #[inline]
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for element in self.iter_mut() {
            *element = value.clone();
        }
    }

    /// Copies all elements of `src` into `self`, like `memcpy`. Panics if the
    /// two have different lengths.
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        if self.len() != src.len() {
            panic("destination and source slices have different lengths");
        }
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), self.len()) }
    }
}

/// Forms a slice from a pointer to its first element and a length.
///
/// # Safety
///
/// `data` must point to `len` initialised, properly aligned values of `T` that
/// stay valid and unmodified for the lifetime `'a`.
#[inline]
pub unsafe fn from_raw_parts<'a, T>(data: *const T, len: usize) -> &'a [T] {
    unsafe { &*ptr::slice_from_raw_parts(data, len) }
}

/// Forms a mutable slice from a pointer to its first element and a length.
///
/// # Safety
///
/// As for [`from_raw_parts`], and nothing else may access the elements for
/// the lifetime `'a`.
#[inline]
pub unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    unsafe { &mut *ptr::slice_from_raw_parts_mut(data, len) }
}

// ---- iteration ----

/// The iterator returned by `[T]::iter` and by `for x in &slice`.
pub struct Iter<'a, T> {
    ptr: *const T,
    remaining: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe {
                let item = &*self.ptr;
                self.ptr = self.ptr.add(1);
                Some(item)
            }
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe { Some(&*self.ptr.add(self.remaining)) }
        }
    }
}

/// The iterator returned by `[T]::iter_mut` and by `for x in &mut slice`.
pub struct IterMut<'a, T> {
    ptr: *mut T,
    remaining: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe {
                let item = &mut *self.ptr;
                self.ptr = self.ptr.add(1);
                Some(item)
            }
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe { Some(&mut *self.ptr.add(self.remaining)) }
        }
    }
}

impl<'a, T> IntoIterator for &'a [T] {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut [T] {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a [T; N] {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut [T; N] {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

// ---- indexing with ranges ----

/// Checks `start..end` against a slice of length `len` and returns the
/// length of the subslice. Panics like real core, but without the numbers:
/// there is no formatting.
#[inline]
fn subslice_len(start: usize, end: usize, len: usize) -> usize {
    if start > end {
        panic("slice index starts after it ends");
    }
    if end > len {
        panic("range end index out of range for slice");
    }
    end - start
}

#[inline]
fn inclusive_end(end: usize) -> usize {
    if end == usize::MAX {
        panic("attempted to index slice up to maximum usize");
    }
    end + 1
}

impl<T> [T] {
    #[inline]
    fn subslice(&self, start: usize, end: usize) -> &[T] {
        let len = subslice_len(start, end, self.len());
        unsafe { from_raw_parts(self.as_ptr().add(start), len) }
    }

    #[inline]
    fn subslice_mut(&mut self, start: usize, end: usize) -> &mut [T] {
        let len = subslice_len(start, end, self.len());
        unsafe { from_raw_parts_mut(self.as_mut_ptr().add(start), len) }
    }
}

macro_rules! range_index_impls {
    ($($range:ty => |$s:ident, $r:ident| ($start:expr, $end:expr);)*) => {$(
        impl<T> Index<$range> for [T] {
            type Output = [T];

            #[inline]
            #[allow(unused_variables)]
            fn index(&self, $r: $range) -> &[T] {
                let $s = &*self;
                self.subslice($start, $end)
            }
        }

        impl<T> IndexMut<$range> for [T] {
            #[inline]
            #[allow(unused_variables)]
            fn index_mut(&mut self, $r: $range) -> &mut [T] {
                let $s = &*self;
                let (start, end) = ($start, $end);
                self.subslice_mut(start, end)
            }
        }
    )*};
}

range_index_impls! {
    Range<usize> => |s, r| (r.start, r.end);
    RangeFrom<usize> => |s, r| (r.start, s.len());
    RangeTo<usize> => |s, r| (0, r.end);
    RangeFull => |s, r| (0, s.len());
    RangeInclusive<usize> => |s, r| (r.start, inclusive_end(r.end));
}
