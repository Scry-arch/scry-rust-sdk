//! Marker traits: the sized hierarchy, Copy, auto traits, PhantomData.

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

#[lang = "unsize"]
pub trait Unsize<T: PointeeSized>: PointeeSized {}

#[lang = "destruct"]
pub trait Destruct {}

#[lang = "tuple_trait"]
pub trait Tuple {}

#[lang = "bikeshed_guaranteed_no_drop"]
pub trait BikeshedGuaranteedNoDrop {}

#[lang = "structural_peq"]
pub trait StructuralPartialEq {}

/// The type of an enum's discriminant, implemented by the compiler. The derived
/// comparisons of enums read discriminants through
/// `intrinsics::discriminant_value`.
#[lang = "discriminant_kind"]
pub trait DiscriminantKind {
    #[lang = "discriminant_type"]
    type Discriminant: Copy;
}

#[lang = "copy"]
pub trait Copy {}

/// The builtin `#[derive(Copy)]` macro (rides on the same path as the trait).
#[rustc_builtin_macro]
pub macro Copy($item:item) {
    /* compiler built-in */
}

impl Copy for bool {}
impl Copy for char {}
impl Copy for u8 {}
impl Copy for u16 {}
impl Copy for u32 {}
impl Copy for usize {}
impl Copy for i8 {}
impl Copy for i16 {}
impl Copy for i32 {}
impl Copy for isize {}
impl<'a, T: PointeeSized> Copy for &'a T {}
impl<T: PointeeSized> Copy for *const T {}
impl<T: PointeeSized> Copy for *mut T {}

#[lang = "sync"]
pub unsafe auto trait Sync {}

pub unsafe auto trait Send {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

unsafe impl<T: PointeeSized> Freeze for PhantomData<T> {}
unsafe impl<T: PointeeSized> Freeze for *const T {}
unsafe impl<T: PointeeSized> Freeze for *mut T {}
unsafe impl<T: PointeeSized> Freeze for &T {}
unsafe impl<T: PointeeSized> Freeze for &mut T {}

#[lang = "unpin"]
pub auto trait Unpin {}

/// Consulted by optimised builds to decide whether a `&mut` argument may be
/// marked as not aliasing anything else. An auto trait, so every type without
/// an opt-out has it.
#[lang = "unsafe_unpin"]
pub unsafe auto trait UnsafeUnpin {}

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

// Arrays of Copy elements are Copy (real core provides this blanket impl;
// without it `*dst = *src` on a borrowed array is a move-out error, E0508).
impl<T: Copy, const N: usize> Copy for [T; N] {}
