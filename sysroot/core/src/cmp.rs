//! Comparison: `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Ordering`, shaped
//! like real core so that all four can be derived, plus `min`, `max` and
//! equality of arrays and slices.

#[allow(unused_imports)]
use crate::marker::{Copy, PhantomData, Sized};
use crate::option::Option::{self, Some};

use self::Ordering::{Equal, Greater, Less};

// ---- equality ----

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    #[inline]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

/// The builtin `#[derive(PartialEq)]` macro.
#[rustc_builtin_macro]
pub macro PartialEq($item:item) {
    /* compiler built-in */
}

/// Equality that is reflexive, as it is for integers.
pub trait Eq: PartialEq<Self> {
    // Both are hooks for `derive(Eq)`, which checks through them that every
    // field is `Eq` as well. Which one it uses depends on the compiler version.
    #[inline]
    fn assert_receiver_is_total_eq(&self) {}
    #[inline]
    fn assert_fields_are_eq(&self) {}
}

/// The builtin `#[derive(Eq)]` macro. Its expansion marks the helper it
/// generates `#[coverage(off)]`, an unstable attribute this allows.
#[rustc_builtin_macro]
#[allow_internal_unstable(coverage_attribute)]
pub macro Eq($item:item) {
    /* compiler built-in */
}

/// Used by the `derive(Eq)` expansion to require that a field type is `Eq`.
pub struct AssertParamIsEq<T: Eq + ?Sized> {
    _field: PhantomData<T>,
}

// ---- ordering ----

/// The result of comparing two values.
#[lang = "Ordering"]
#[repr(i8)]
pub enum Ordering {
    Less = -1,
    Equal = 0,
    Greater = 1,
}

impl Ordering {
    #[inline]
    pub const fn is_eq(self) -> bool {
        self as i8 == 0
    }
    #[inline]
    pub const fn is_ne(self) -> bool {
        self as i8 != 0
    }
    #[inline]
    pub const fn is_lt(self) -> bool {
        (self as i8) < 0
    }
    #[inline]
    pub const fn is_gt(self) -> bool {
        self as i8 > 0
    }
    #[inline]
    pub const fn is_le(self) -> bool {
        self as i8 <= 0
    }
    #[inline]
    pub const fn is_ge(self) -> bool {
        self as i8 >= 0
    }

    /// `Less` becomes `Greater` and the other way around.
    #[inline]
    pub const fn reverse(self) -> Ordering {
        match self {
            Less => Greater,
            Equal => Equal,
            Greater => Less,
        }
    }

    /// `self` unless it is `Equal`, then `other`: chains comparisons of
    /// several keys.
    #[inline]
    pub const fn then(self, other: Ordering) -> Ordering {
        match self {
            Equal => other,
            _ => self,
        }
    }
}

impl Copy for Ordering {}

impl crate::clone::Clone for Ordering {
    #[inline]
    fn clone(&self) -> Ordering {
        *self
    }
}

impl PartialEq for Ordering {
    #[inline]
    fn eq(&self, other: &Ordering) -> bool {
        *self as i8 == *other as i8
    }
}

impl Eq for Ordering {}

#[lang = "partial_ord"]
pub trait PartialOrd<Rhs: ?Sized = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

    #[inline]
    fn lt(&self, other: &Rhs) -> bool {
        match self.partial_cmp(other) {
            Some(Less) => true,
            _ => false,
        }
    }
    #[inline]
    fn le(&self, other: &Rhs) -> bool {
        match self.partial_cmp(other) {
            Some(Less | Equal) => true,
            _ => false,
        }
    }
    #[inline]
    fn gt(&self, other: &Rhs) -> bool {
        match self.partial_cmp(other) {
            Some(Greater) => true,
            _ => false,
        }
    }
    #[inline]
    fn ge(&self, other: &Rhs) -> bool {
        match self.partial_cmp(other) {
            Some(Greater | Equal) => true,
            _ => false,
        }
    }
}

/// The builtin `#[derive(PartialOrd)]` macro.
#[rustc_builtin_macro]
pub macro PartialOrd($item:item) {
    /* compiler built-in */
}

/// A total order, as integers have.
pub trait Ord: Eq + PartialOrd<Self> {
    fn cmp(&self, other: &Self) -> Ordering;

    /// The larger of the two, `other` if they are equal.
    #[inline]
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Greater => self,
            _ => other,
        }
    }

    /// The smaller of the two, `self` if they are equal.
    #[inline]
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Greater => other,
            _ => self,
        }
    }

    /// `self` limited to the range `min..=max`. Panics if `min > max`.
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        if min.cmp(&max).is_gt() {
            crate::panicking::panic("assertion failed: min <= max");
        }
        if self.cmp(&min).is_lt() {
            min
        } else if self.cmp(&max).is_gt() {
            max
        } else {
            self
        }
    }
}

/// The builtin `#[derive(Ord)]` macro.
#[rustc_builtin_macro]
pub macro Ord($item:item) {
    /* compiler built-in */
}

/// The larger of `a` and `b`, `b` if they are equal.
#[inline]
pub fn max<T: Ord>(a: T, b: T) -> T {
    a.max(b)
}

/// The smaller of `a` and `b`, `a` if they are equal.
#[inline]
pub fn min<T: Ord>(a: T, b: T) -> T {
    a.min(b)
}

// ---- primitives ----

macro_rules! cmp_impls {
    ($($t:ty)*) => {$(
        impl PartialEq for $t {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                (*self) == (*other)
            }
            #[inline]
            fn ne(&self, other: &$t) -> bool {
                (*self) != (*other)
            }
        }

        impl Eq for $t {}

        impl PartialOrd for $t {
            #[inline]
            fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                Some(Ord::cmp(self, other))
            }
            #[inline]
            fn lt(&self, other: &$t) -> bool {
                (*self) < (*other)
            }
            #[inline]
            fn le(&self, other: &$t) -> bool {
                (*self) <= (*other)
            }
            #[inline]
            fn gt(&self, other: &$t) -> bool {
                (*self) > (*other)
            }
            #[inline]
            fn ge(&self, other: &$t) -> bool {
                (*self) >= (*other)
            }
        }

        impl Ord for $t {
            #[inline]
            fn cmp(&self, other: &$t) -> Ordering {
                if *self < *other {
                    Less
                } else if *self == *other {
                    Equal
                } else {
                    Greater
                }
            }
        }
    )*};
}

cmp_impls! { bool char u8 u16 u32 usize i8 i16 i32 isize }

impl<T: ?Sized> PartialEq for *const T {
    #[inline]
    fn eq(&self, other: &*const T) -> bool {
        *self == *other
    }
    #[inline]
    fn ne(&self, other: &*const T) -> bool {
        *self != *other
    }
}

impl<T: ?Sized> PartialEq for *mut T {
    #[inline]
    fn eq(&self, other: &*mut T) -> bool {
        *self == *other
    }
    #[inline]
    fn ne(&self, other: &*mut T) -> bool {
        *self != *other
    }
}

// ---- references ----

impl<'a, 'b, A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&'b B> for &'a A {
    #[inline]
    fn eq(&self, other: &&'b B) -> bool {
        PartialEq::eq(*self, *other)
    }
    #[inline]
    fn ne(&self, other: &&'b B) -> bool {
        PartialEq::ne(*self, *other)
    }
}

impl<'a, A: ?Sized + Eq> Eq for &'a A {}

impl<'a, 'b, A: ?Sized + PartialOrd<B>, B: ?Sized> PartialOrd<&'b B> for &'a A {
    #[inline]
    fn partial_cmp(&self, other: &&'b B) -> Option<Ordering> {
        PartialOrd::partial_cmp(*self, *other)
    }
    #[inline]
    fn lt(&self, other: &&'b B) -> bool {
        PartialOrd::lt(*self, *other)
    }
    #[inline]
    fn le(&self, other: &&'b B) -> bool {
        PartialOrd::le(*self, *other)
    }
    #[inline]
    fn gt(&self, other: &&'b B) -> bool {
        PartialOrd::gt(*self, *other)
    }
    #[inline]
    fn ge(&self, other: &&'b B) -> bool {
        PartialOrd::ge(*self, *other)
    }
}

impl<'a, A: ?Sized + Ord> Ord for &'a A {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(*self, *other)
    }
}

// ---- arrays and slices ----
//
// Element-wise equality, including between an array and a slice, so that
// `&buffer[..4] == b"abcd"` works.

impl<A: PartialEq<B>, B> PartialEq<[B]> for [A] {
    #[inline]
    fn eq(&self, other: &[B]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let mut i = 0;
        while i < self.len() {
            if self[i] != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<T: Eq> Eq for [T] {}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<[B; N]> for [A; N] {
    #[inline]
    fn eq(&self, other: &[B; N]) -> bool {
        PartialEq::eq(self as &[A], other as &[B])
    }
}

impl<T: Eq, const N: usize> Eq for [T; N] {}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<[B]> for [A; N] {
    #[inline]
    fn eq(&self, other: &[B]) -> bool {
        PartialEq::eq(self as &[A], other)
    }
}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<[B; N]> for [A] {
    #[inline]
    fn eq(&self, other: &[B; N]) -> bool {
        PartialEq::eq(self, other as &[B])
    }
}

// A slice reference compared with an array, as in `&buffer[..3] == [1, 2, 3]`
// or `slice == [1, 2, 3]`.

impl<A: PartialEq<B>, B, const N: usize> PartialEq<[B; N]> for &[A] {
    #[inline]
    fn eq(&self, other: &[B; N]) -> bool {
        PartialEq::eq(*self, other as &[B])
    }
}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<[B; N]> for &mut [A] {
    #[inline]
    fn eq(&self, other: &[B; N]) -> bool {
        PartialEq::eq(&**self, other as &[B])
    }
}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<&[B]> for [A; N] {
    #[inline]
    fn eq(&self, other: &&[B]) -> bool {
        PartialEq::eq(self as &[A], *other)
    }
}

impl<A: PartialEq<B>, B, const N: usize> PartialEq<&mut [B]> for [A; N] {
    #[inline]
    fn eq(&self, other: &&mut [B]) -> bool {
        PartialEq::eq(self as &[A], &**other)
    }
}
