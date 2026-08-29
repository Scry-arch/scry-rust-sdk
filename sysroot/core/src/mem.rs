//! Memory utilities.

use crate::intrinsics;
use crate::marker::Sized;

pub const fn size_of<T>() -> usize {
    <T as SizedTypeProperties>::SIZE
}

pub const fn align_of<T>() -> usize {
    <T as SizedTypeProperties>::ALIGN
}

/// Disposes of a value (drop glue runs as it goes out of scope).
pub fn drop<T>(_x: T) {}

trait SizedTypeProperties: Sized {
    #[lang = "mem_size_const"]
    const SIZE: usize = intrinsics::size_of::<Self>();

    #[lang = "mem_align_const"]
    const ALIGN: usize = intrinsics::align_of::<Self>();
}
impl<T> SizedTypeProperties for T {}

#[lang = "manually_drop"]
#[repr(transparent)]
pub struct ManuallyDrop<T: ?Sized> {
    pub value: T,
}

#[lang = "maybe_uninit"]
#[repr(transparent)]
pub union MaybeUninit<T> {
    pub uninit: (),
    pub value: ManuallyDrop<T>,
}
