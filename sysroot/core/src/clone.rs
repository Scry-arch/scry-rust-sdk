//! The `Clone` trait, its primitive impls, and the builtin derive.

use crate::marker::{PhantomData, PointeeSized, Sized};

#[lang = "clone"]
pub trait Clone: Sized {
    fn clone(&self) -> Self;
}

/// The builtin `#[derive(Clone)]` macro.
#[rustc_builtin_macro]
pub macro Clone($item:item) {
    /* compiler built-in */
}

/// Marker used by the builtin `derive(Clone)` expansion for types whose clone
/// is a bitwise copy.
pub unsafe trait TrivialClone: Clone {}

// Bound-assertion helpers referenced by the builtin derive expansions.
pub struct AssertParamIsClone<T: Clone + PointeeSized> {
    _field: PhantomData<T>,
}
pub struct AssertParamIsCopy<T: crate::marker::Copy + PointeeSized> {
    _field: PhantomData<T>,
}

macro_rules! clone_impls {
    ($($t:ty)*) => {$(
        impl Clone for $t {
            fn clone(&self) -> $t {
                *self
            }
        }
    )*};
}

clone_impls! { bool char u8 u16 u32 usize i8 i16 i32 isize }

impl<'a, T: PointeeSized> Clone for &'a T {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: PointeeSized> Clone for *const T {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: PointeeSized> Clone for *mut T {
    fn clone(&self) -> Self {
        *self
    }
}

// Arrays of Copy elements clone bitwise. (Real core clones element-wise for
// any T: Clone; the Copy-bounded form covers everything scry-core supports.)
impl<T: crate::marker::Copy, const N: usize> Clone for [T; N] {
    fn clone(&self) -> Self {
        *self
    }
}
