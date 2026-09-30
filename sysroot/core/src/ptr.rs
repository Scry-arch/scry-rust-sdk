//! Pointer types and metadata.

use crate::marker::{Copy, Freeze, PhantomData, PointeeSized, Sized, Sync, Unpin};
use crate::ops::{CoerceUnsized, DispatchFromDyn};
use crate::pattern_type;

#[lang = "pointee_trait"]
pub trait Pointee: PointeeSized {
    #[lang = "metadata_type"]
    // Needed so that layout_of returns `TooGeneric` instead of `Unknown` when
    // asked for the layout of `*const T`, which matters for transmutes between
    // raw pointers (especially pattern types of raw pointers).
    type Metadata: Copy + Sync + Unpin + Freeze;
}

#[lang = "dyn_metadata"]
pub struct DynMetadata<Dyn: PointeeSized> {
    _vtable_ptr: NonNull<VTable>,
    _phantom: PhantomData<Dyn>,
}

unsafe extern "C" {
    /// Opaque type for accessing vtables. There is conceptually no Abstract
    /// Machine memory behind this pointer.
    type VTable;
}

#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
pub struct NonNull<T: PointeeSized>(pub pattern_type!(*const T is !null));

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<NonNull<U>> for NonNull<T> where T: crate::marker::Unsize<U> {}
impl<T: PointeeSized, U: PointeeSized> DispatchFromDyn<NonNull<U>> for NonNull<T> where T: crate::marker::Unsize<U> {}

pub struct Unique<T: PointeeSized> {
    pub pointer: NonNull<T>,
    pub _marker: PhantomData<T>,
}

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<Unique<U>> for Unique<T> where T: crate::marker::Unsize<U> {}
impl<T: PointeeSized, U: PointeeSized> DispatchFromDyn<Unique<U>> for Unique<T> where T: crate::marker::Unsize<U> {}

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<pattern_type!(*const U is !null)>
    for pattern_type!(*const T is !null)
where
    T: crate::marker::Unsize<U>,
{
}

impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is !null)> for pattern_type!(T is !null) {}

#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: ?Sized>(_to_drop: &mut T) {
    // Body does not matter, replaced by real drop glue by the compiler.
}

// ---- reading, writing and copying ----

/// Reads the value at `src` without moving it.
///
/// # Safety
///
/// `src` must be valid for reads and properly aligned.
#[inline]
pub unsafe fn read<T>(src: *const T) -> T {
    unsafe { crate::intrinsics::read_via_copy(src) }
}

/// Overwrites the value at `dst` with `src`, without reading or dropping the
/// old value.
///
/// # Safety
///
/// `dst` must be valid for writes and properly aligned.
#[inline]
pub unsafe fn write<T>(dst: *mut T, src: T) {
    unsafe { crate::intrinsics::write_via_move(dst, src) }
}

/// Copies `count` values from `src` to `dst`. The two regions may overlap,
/// like C's `memmove`.
///
/// # Safety
///
/// Both regions must be valid and properly aligned.
#[inline]
pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize) {
    unsafe { crate::intrinsics::copy(src, dst, count) }
}

/// Copies `count` values from `src` to `dst`, like C's `memcpy`.
///
/// # Safety
///
/// Both regions must be valid and properly aligned, and must not overlap.
#[inline]
pub unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
    unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
}

/// Swaps the values at `a` and `b`, which may be the same location.
///
/// # Safety
///
/// Both must be valid for reads and writes and properly aligned, and must not
/// partially overlap.
#[inline]
pub unsafe fn swap<T>(a: *mut T, b: *mut T) {
    unsafe {
        let tmp = read(a);
        copy(b, a, 1);
        write(b, tmp);
    }
}

/// Forms a raw slice from a pointer to its first element and a length.
#[inline]
pub const fn slice_from_raw_parts<T>(data: *const T, len: usize) -> *const [T] {
    crate::intrinsics::aggregate_raw_ptr(data, len)
}

/// Forms a mutable raw slice from a pointer to its first element and a length.
#[inline]
pub const fn slice_from_raw_parts_mut<T>(data: *mut T, len: usize) -> *mut [T] {
    crate::intrinsics::aggregate_raw_ptr(data, len)
}

// ---- volatile access ----
//
// Memory-mapped I/O registers change on their own and react to being read or
// written, so every access in the source must happen, once, in order. Ordinary
// loads and stores may be removed or merged by the compiler; these may not.

/// Reads the value at `src` with a load that is never optimised away, without
/// moving it.
///
/// # Safety
///
/// `src` must be valid for reads and properly aligned, as for an ordinary read.
/// Like real core, this does not drop or otherwise touch the value at `src`.
#[inline]
pub unsafe fn read_volatile<T>(src: *const T) -> T {
    unsafe { crate::intrinsics::volatile_load(src) }
}

/// Writes `src` to `dst` with a store that is never optimised away, without
/// reading or dropping the old value.
///
/// # Safety
///
/// `dst` must be valid for writes and properly aligned, as for an ordinary
/// write.
#[inline]
pub unsafe fn write_volatile<T>(dst: *mut T, src: T) {
    unsafe { crate::intrinsics::volatile_store(dst, src) }
}

// The method forms, `ptr.read_volatile()` and `ptr.write_volatile(value)`.
// Inherent impls on primitive types are only allowed in core, which is what
// the crate's `rustc_coherence_is_core` attribute declares.

impl<T: PointeeSized> *const T {
    /// Offsets the pointer by `count` values of `T`.
    ///
    /// # Safety
    ///
    /// The result must stay within, or one past the end of, the same object.
    #[inline]
    pub const unsafe fn add(self, count: usize) -> Self
    where
        T: Sized,
    {
        unsafe { crate::intrinsics::offset(self, count) }
    }

    /// See [`read_volatile`].
    #[inline]
    pub unsafe fn read_volatile(self) -> T
    where
        T: Sized,
    {
        unsafe { read_volatile(self) }
    }
}

impl<T: PointeeSized> *mut T {
    /// Offsets the pointer by `count` values of `T`.
    ///
    /// # Safety
    ///
    /// The result must stay within, or one past the end of, the same object.
    #[inline]
    pub const unsafe fn add(self, count: usize) -> Self
    where
        T: Sized,
    {
        unsafe { crate::intrinsics::offset(self, count) }
    }

    /// See [`read_volatile`].
    #[inline]
    pub unsafe fn read_volatile(self) -> T
    where
        T: Sized,
    {
        unsafe { read_volatile(self) }
    }

    /// See [`write_volatile`].
    #[inline]
    pub unsafe fn write_volatile(self, val: T)
    where
        T: Sized,
    {
        unsafe { write_volatile(self, val) }
    }
}
