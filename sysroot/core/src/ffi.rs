//! FFI support types.

unsafe extern "C" {
    type VaListImpl;
}

#[lang = "va_list"]
#[repr(transparent)]
pub struct VaList<'a>(&'a mut VaListImpl);

/// Equivalent to C's `void` in pointer contexts (`*mut c_void`).
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum c_void {
    #[doc(hidden)]
    __Variant1,
    #[doc(hidden)]
    __Variant2,
}
