//! Compiler intrinsics.

use crate::marker::{Copy, Sized};

#[rustc_intrinsic]
pub fn abort() -> !;
#[rustc_intrinsic]
pub const fn size_of<T>() -> usize;
#[rustc_intrinsic]
pub unsafe fn size_of_val<T: ?Sized>(val: *const T) -> usize;
#[rustc_intrinsic]
pub const fn align_of<T>() -> usize;
#[rustc_intrinsic]
pub unsafe fn align_of_val<T: ?Sized>(val: *const T) -> usize;
#[rustc_intrinsic]
pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize);
#[rustc_intrinsic]
pub const unsafe fn transmute<T, U>(e: T) -> U;
#[rustc_intrinsic]
pub unsafe fn ctlz_nonzero<T>(x: T) -> u32;
#[rustc_intrinsic]
pub const fn needs_drop<T: ?Sized>() -> bool;
#[rustc_intrinsic]
pub fn bitreverse<T>(x: T) -> T;
#[rustc_intrinsic]
pub const fn bswap<T>(x: T) -> T;
#[rustc_intrinsic]
pub unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize);
#[rustc_intrinsic]
pub unsafe fn unreachable() -> !;

// Integer arithmetic. Bit counting (`ctlz`, `cttz`, `ctpop`) is deliberately
// absent: the Scry backend cannot lower it, so `num` implements it in Rust.
#[rustc_intrinsic]
pub const fn wrapping_add<T: Copy>(a: T, b: T) -> T;
#[rustc_intrinsic]
pub const fn wrapping_sub<T: Copy>(a: T, b: T) -> T;
#[rustc_intrinsic]
pub const fn wrapping_mul<T: Copy>(a: T, b: T) -> T;
// The rotations have fallback bodies, as in real core: cg_clif lowers them
// itself, but LLVM expects the fallback, which lets the SDK's core also be
// built for another target to test it.
#[rustc_intrinsic]
pub const fn rotate_left<T: Copy + [const] RotateFallback>(x: T, shift: u32) -> T {
    T::rotate_left_fallback(x, shift)
}
#[rustc_intrinsic]
pub const fn rotate_right<T: Copy + [const] RotateFallback>(x: T, shift: u32) -> T {
    T::rotate_right_fallback(x, shift)
}

/// Rotations built from shifts, for backends that do not lower the rotation
/// intrinsics themselves. Implemented for the unsigned types, which are the
/// only ones `num` rotates.
pub const trait RotateFallback: Copy {
    fn rotate_left_fallback(x: Self, shift: u32) -> Self;
    fn rotate_right_fallback(x: Self, shift: u32) -> Self;
}

macro_rules! rotate_fallback_impls {
    ($($t:ty: $bits:literal)*) => {$(
        const impl RotateFallback for $t {
            #[inline]
            fn rotate_left_fallback(x: $t, shift: u32) -> $t {
                let s = shift % $bits;
                (x << s) | (x >> (($bits - s) % $bits))
            }
            #[inline]
            fn rotate_right_fallback(x: $t, shift: u32) -> $t {
                let s = shift % $bits;
                (x >> s) | (x << (($bits - s) % $bits))
            }
        }
    )*};
}

rotate_fallback_impls! { u8: 8 u16: 16 u32: 32 usize: 32 }

// Pointers and memory.
#[rustc_intrinsic]
pub const unsafe fn offset<Ptr, Delta>(dst: Ptr, offset: Delta) -> Ptr;
#[rustc_intrinsic]
pub const fn ptr_metadata<P: crate::ptr::Pointee<Metadata = M> + crate::marker::PointeeSized, M>(ptr: *const P) -> M;
#[rustc_intrinsic]
pub const fn aggregate_raw_ptr<P, D, M>(data: D, meta: M) -> P;
#[rustc_intrinsic]
pub const unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);
#[rustc_intrinsic]
pub const unsafe fn read_via_copy<T>(ptr: *const T) -> T;
#[rustc_intrinsic]
pub const unsafe fn write_via_move<T>(ptr: *mut T, value: T);
#[rustc_intrinsic]
pub const fn discriminant_value<T>(v: &T) -> <T as crate::marker::DiscriminantKind>::Discriminant;
/// A load the compiler must perform exactly as written: never removed, merged
/// with another, or reordered. Use through `ptr::read_volatile`.
#[rustc_intrinsic]
#[rustc_nounwind]
pub unsafe fn volatile_load<T>(src: *const T) -> T;
/// A store the compiler must perform exactly as written. Use through
/// `ptr::write_volatile`.
#[rustc_intrinsic]
#[rustc_nounwind]
pub unsafe fn volatile_store<T>(dst: *mut T, val: T);
