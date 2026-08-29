//! Operator traits. Impls exist ONLY for <=32-bit integer types,  this is the
//! SDK guardrail: arithmetic on u64/i64/u128/f32/... fails typeck (E0369)
//! instead of reaching the backend.

use crate::marker::{MetaSized, PointeeSized, Sized, Tuple, Unsize};

#[lang = "add"]
pub trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

#[lang = "sub"]
pub trait Sub<Rhs = Self> {
    type Output;
    fn sub(self, rhs: Rhs) -> Self::Output;
}

#[lang = "mul"]
pub trait Mul<Rhs = Self> {
    type Output;
    fn mul(self, rhs: Rhs) -> Self::Output;
}

#[lang = "div"]
pub trait Div<Rhs = Self> {
    type Output;
    fn div(self, rhs: Rhs) -> Self::Output;
}

#[lang = "rem"]
pub trait Rem<Rhs = Self> {
    type Output;
    fn rem(self, rhs: Rhs) -> Self::Output;
}

#[lang = "bitand"]
pub trait BitAnd<Rhs = Self> {
    type Output;
    fn bitand(self, rhs: Rhs) -> Self::Output;
}

#[lang = "bitor"]
pub trait BitOr<Rhs = Self> {
    type Output;
    fn bitor(self, rhs: Rhs) -> Self::Output;
}

#[lang = "bitxor"]
pub trait BitXor<Rhs = Self> {
    type Output;
    fn bitxor(self, rhs: Rhs) -> Self::Output;
}

#[lang = "shl"]
pub trait Shl<Rhs = Self> {
    type Output;
    fn shl(self, rhs: Rhs) -> Self::Output;
}

#[lang = "shr"]
pub trait Shr<Rhs = Self> {
    type Output;
    fn shr(self, rhs: Rhs) -> Self::Output;
}

#[lang = "not"]
pub trait Not {
    type Output;
    fn not(self) -> Self::Output;
}

#[lang = "neg"]
pub trait Neg {
    type Output;
    fn neg(self) -> Self::Output;
}

#[lang = "add_assign"]
pub trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, rhs: Rhs);
}

#[lang = "sub_assign"]
pub trait SubAssign<Rhs = Self> {
    fn sub_assign(&mut self, rhs: Rhs);
}

#[lang = "mul_assign"]
pub trait MulAssign<Rhs = Self> {
    fn mul_assign(&mut self, rhs: Rhs);
}

#[lang = "div_assign"]
pub trait DivAssign<Rhs = Self> {
    fn div_assign(&mut self, rhs: Rhs);
}

#[lang = "rem_assign"]
pub trait RemAssign<Rhs = Self> {
    fn rem_assign(&mut self, rhs: Rhs);
}

#[lang = "bitand_assign"]
pub trait BitAndAssign<Rhs = Self> {
    fn bitand_assign(&mut self, rhs: Rhs);
}

#[lang = "bitor_assign"]
pub trait BitOrAssign<Rhs = Self> {
    fn bitor_assign(&mut self, rhs: Rhs);
}

#[lang = "bitxor_assign"]
pub trait BitXorAssign<Rhs = Self> {
    fn bitxor_assign(&mut self, rhs: Rhs);
}

#[lang = "shl_assign"]
pub trait ShlAssign<Rhs = Self> {
    fn shl_assign(&mut self, rhs: Rhs);
}

#[lang = "shr_assign"]
pub trait ShrAssign<Rhs = Self> {
    fn shr_assign(&mut self, rhs: Rhs);
}

macro_rules! int_ops_impls {
    ($($t:ty)*) => {$(
        impl Add for $t {
            type Output = $t;
            fn add(self, rhs: $t) -> $t { self + rhs }
        }
        impl Sub for $t {
            type Output = $t;
            fn sub(self, rhs: $t) -> $t { self - rhs }
        }
        impl Mul for $t {
            type Output = $t;
            fn mul(self, rhs: $t) -> $t { self * rhs }
        }
        impl Div for $t {
            type Output = $t;
            fn div(self, rhs: $t) -> $t { self / rhs }
        }
        impl Rem for $t {
            type Output = $t;
            fn rem(self, rhs: $t) -> $t { self % rhs }
        }
        impl BitAnd for $t {
            type Output = $t;
            fn bitand(self, rhs: $t) -> $t { self & rhs }
        }
        impl BitOr for $t {
            type Output = $t;
            fn bitor(self, rhs: $t) -> $t { self | rhs }
        }
        impl BitXor for $t {
            type Output = $t;
            fn bitxor(self, rhs: $t) -> $t { self ^ rhs }
        }
        impl Shl for $t {
            type Output = $t;
            fn shl(self, rhs: $t) -> $t { self << rhs }
        }
        impl Shr for $t {
            type Output = $t;
            fn shr(self, rhs: $t) -> $t { self >> rhs }
        }
        impl Not for $t {
            type Output = $t;
            fn not(self) -> $t { !self }
        }
        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: $t) { *self = *self + rhs; }
        }
        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: $t) { *self = *self - rhs; }
        }
        impl MulAssign for $t {
            fn mul_assign(&mut self, rhs: $t) { *self = *self * rhs; }
        }
        impl DivAssign for $t {
            fn div_assign(&mut self, rhs: $t) { *self = *self / rhs; }
        }
        impl RemAssign for $t {
            fn rem_assign(&mut self, rhs: $t) { *self = *self % rhs; }
        }
        impl BitAndAssign for $t {
            fn bitand_assign(&mut self, rhs: $t) { *self = *self & rhs; }
        }
        impl BitOrAssign for $t {
            fn bitor_assign(&mut self, rhs: $t) { *self = *self | rhs; }
        }
        impl BitXorAssign for $t {
            fn bitxor_assign(&mut self, rhs: $t) { *self = *self ^ rhs; }
        }
        impl ShlAssign for $t {
            fn shl_assign(&mut self, rhs: $t) { *self = *self << rhs; }
        }
        impl ShrAssign for $t {
            fn shr_assign(&mut self, rhs: $t) { *self = *self >> rhs; }
        }
    )*};
}

int_ops_impls! { u8 u16 u32 usize i8 i16 i32 isize }

macro_rules! neg_impls {
    ($($t:ty)*) => {$(
        impl Neg for $t {
            type Output = $t;
            fn neg(self) -> $t { -self }
        }
    )*};
}

neg_impls! { i8 i16 i32 isize }

impl Not for bool {
    type Output = bool;
    fn not(self) -> bool {
        !self
    }
}

impl BitAnd for bool {
    type Output = bool;
    fn bitand(self, rhs: bool) -> bool {
        self & rhs
    }
}

impl BitOr for bool {
    type Output = bool;
    fn bitor(self, rhs: bool) -> bool {
        self | rhs
    }
}

impl BitXor for bool {
    type Output = bool;
    fn bitxor(self, rhs: bool) -> bool {
        self ^ rhs
    }
}

impl<'a> BitOr<bool> for &'a bool {
    type Output = bool;
    fn bitor(self, rhs: bool) -> bool {
        *self | rhs
    }
}

// ---- deref ----

#[lang = "deref"]
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

#[lang = "deref_mut"]
pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}

impl<T: MetaSized> Deref for &T {
    type Target = T;
    fn deref(&self) -> &T {
        *self
    }
}

impl<T: MetaSized> Deref for &mut T {
    type Target = T;
    fn deref(&self) -> &T {
        *self
    }
}

impl<T: MetaSized> DerefMut for &mut T {
    fn deref_mut(&mut self) -> &mut T {
        *self
    }
}

// ---- indexing ----

#[lang = "index"]
pub trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

#[lang = "index_mut"]
pub trait IndexMut<Idx: ?Sized>: Index<Idx> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output;
}

impl<T> Index<usize> for [T] {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self[index]
    }
}

impl<T> IndexMut<usize> for [T] {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self[index]
    }
}

impl<T, const N: usize> Index<usize> for [T; N] {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for [T; N] {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self[index]
    }
}

// ---- callables ----

#[lang = "fn_once"]
#[rustc_paren_sugar]
pub trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"]
    type Output;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

#[lang = "fn_mut"]
#[rustc_paren_sugar]
pub trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

#[lang = "fn"]
#[rustc_paren_sugar]
pub trait Fn<Args: Tuple>: FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

// ---- drop ----

#[lang = "drop"]
pub trait Drop {
    fn drop(&mut self);
}

// ---- unsizing coercions ----

#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T> {}

impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a mut U> for &'a mut T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*const U> for *const T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*mut U> for *mut T {}

#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T> {}

impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a mut U> for &'a mut T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<*const U> for *const T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<*mut U> for *mut T {}

#[lang = "legacy_receiver"]
pub trait LegacyReceiver {}

impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
