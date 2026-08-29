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
