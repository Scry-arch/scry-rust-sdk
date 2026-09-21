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

// ---- volatile access ----
//
// Memory-mapped I/O registers change on their own and react to being read or
// written, so every access in the source must happen, once, in order. Ordinary
// loads and stores may be removed or merged by the compiler; these may not.
//
// WORKAROUND: the two functions below are `#[inline(never)]` on purpose. The
// backend lowers the volatile intrinsics to ordinary loads and stores, and
// Cranelift's alias analysis then merges repeated loads of one address in
// optimised builds (three `read_volatile`s of a register became one load). It
// has no per-load opt-out. Alias analysis works within one function, so keeping
// each access in its own never-inlined function guarantees it happens, at the
// price of a call per access. Once the backend keeps volatile accesses intact
// itself (for example by disabling `enable_alias_analysis`), these can go back
// to `#[inline]`; the `volatile` run test does not detect a regression, so
// check the simulator's `DataReads` count for a release build when changing it.

/// Reads the value at `src` with a load that is never optimised away, without
/// moving it.
///
/// # Safety
///
/// `src` must be valid for reads and properly aligned, as for an ordinary read.
/// Like real core, this does not drop or otherwise touch the value at `src`.
#[inline(never)]
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
#[inline(never)]
pub unsafe fn write_volatile<T>(dst: *mut T, src: T) {
    unsafe { crate::intrinsics::volatile_store(dst, src) }
}

// The method forms, `ptr.read_volatile()` and `ptr.write_volatile(value)`.
// Inherent impls on primitive types are only allowed in core, which is what
// the crate's `rustc_coherence_is_core` attribute declares.

impl<T: PointeeSized> *const T {
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
