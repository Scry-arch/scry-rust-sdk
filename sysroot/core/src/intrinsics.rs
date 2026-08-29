//! Compiler intrinsics.

use crate::marker::Sized;

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
pub unsafe fn transmute<T, U>(e: T) -> U;
#[rustc_intrinsic]
pub unsafe fn ctlz_nonzero<T>(x: T) -> u32;
#[rustc_intrinsic]
pub const fn needs_drop<T: ?Sized>() -> bool;
#[rustc_intrinsic]
pub fn bitreverse<T>(x: T) -> T;
#[rustc_intrinsic]
pub fn bswap<T>(x: T) -> T;
#[rustc_intrinsic]
pub unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize);
#[rustc_intrinsic]
pub unsafe fn unreachable() -> !;
