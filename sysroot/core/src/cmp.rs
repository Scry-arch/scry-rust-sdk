//! Comparison traits. Note: no `Ordering`/`Eq`/`Ord` yet, so `partial_cmp` is
//! absent and `derive(PartialOrd)` is unsupported; the four operator methods
//! are required methods instead.

#[allow(unused_imports)]
use crate::marker::Sized;

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

/// The builtin `#[derive(PartialEq)]` macro.
#[rustc_builtin_macro]
pub macro PartialEq($item:item) {
    /* compiler built-in */
}

#[lang = "partial_ord"]
pub trait PartialOrd<Rhs: ?Sized = Self>: PartialEq<Rhs> {
    fn lt(&self, other: &Rhs) -> bool;
    fn le(&self, other: &Rhs) -> bool;
    fn gt(&self, other: &Rhs) -> bool;
    fn ge(&self, other: &Rhs) -> bool;
}

macro_rules! cmp_impls {
    ($($t:ty)*) => {$(
        impl PartialEq for $t {
            fn eq(&self, other: &$t) -> bool {
                (*self) == (*other)
            }
            fn ne(&self, other: &$t) -> bool {
                (*self) != (*other)
            }
        }
        impl PartialOrd for $t {
            fn lt(&self, other: &$t) -> bool {
                (*self) < (*other)
            }
            fn le(&self, other: &$t) -> bool {
                (*self) <= (*other)
            }
            fn gt(&self, other: &$t) -> bool {
                (*self) > (*other)
            }
            fn ge(&self, other: &$t) -> bool {
                (*self) >= (*other)
            }
        }
    )*};
}

cmp_impls! { bool char u8 u16 u32 usize i8 i16 i32 isize }

impl<T: ?Sized> PartialEq for *const T {
    fn eq(&self, other: &*const T) -> bool {
        *self == *other
    }
    fn ne(&self, other: &*const T) -> bool {
        *self != *other
    }
}

impl<T: ?Sized> PartialEq for *mut T {
    fn eq(&self, other: &*mut T) -> bool {
        *self == *other
    }
    fn ne(&self, other: &*mut T) -> bool {
        *self != *other
    }
}

impl<'a, 'b, A: ?Sized + PartialEq<B>, B: ?Sized> PartialEq<&'b B> for &'a A {
    fn eq(&self, other: &&'b B) -> bool {
        PartialEq::eq(*self, *other)
    }
    fn ne(&self, other: &&'b B) -> bool {
        PartialEq::ne(*self, *other)
    }
}
