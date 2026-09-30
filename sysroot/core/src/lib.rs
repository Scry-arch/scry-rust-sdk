//! The Scry `core`: a minimal core implementation for Scry targets.
//!
//! Deliberate divergences from real core:
//! - Operator impls exist ONLY for <=32-bit types; `u64 + u64` etc. fail typeck (E0369).
//! - Panics trap immediately; no `#[panic_handler]`, no unwinding, no formatting.
//!   `panic!` accepts only a string literal.
//! - Integers have wrapping arithmetic, rotations, byte order conversion, bit
//!   counting, `abs`, `MIN`/`MAX`/`BITS`, and `min`/`max`/`clamp` through
//!   `Ord`. No checked, overflowing or saturating arithmetic, no `pow`.
//! - `for` loops work over `a..b` and `a..=b` ranges of the integer types and
//!   over slices and arrays by reference. `Iterator` has only the adapters
//!   `rev` and `enumerate`; there is no `Step`, and arrays cannot be iterated
//!   by value.
//! - Slices and arrays have `len`, `get`, `iter`, `swap`, `fill`,
//!   `copy_from_slice`, range indexing and element-wise equality, but no
//!   ordering, searching or sorting.
//! - Raw pointers have `add`, and `ptr` has `read`, `write`, `copy`,
//!   `copy_nonoverlapping` and `swap`, plus the volatile accessors
//!   (`read_volatile`, `write_volatile`, also as methods) for memory-mapped I/O.
//! - `str` has only `len`, `as_bytes` and equality.
//! - `hint::black_box` goes through a volatile store and load, because the
//!   backend ignores the intrinsic.
//! - Not yet provided: floats, 64-bit integers, UnsafeCell, fmt.
#![feature(
    no_core,
    lang_items,
    intrinsics,
    unboxed_closures,
    extern_types,
    decl_macro,
    rustc_attrs,
    transparent_unions,
    pattern_types,
    auto_traits,
    freeze_impls,
    allow_internal_unstable,
    const_trait_impl
)]
#![no_core]
// Lets this crate, like real core, add inherent methods to primitive types
// such as raw pointers.
#![rustc_coherence_is_core]
#![allow(dead_code, internal_features, ambiguous_wide_pointer_comparisons)]

pub mod arch;
pub mod clone;
pub mod cmp;
pub mod ffi;
pub mod hint;
pub mod intrinsics;
pub mod iter;
pub mod marker;
pub mod mem;
pub mod num;
pub mod ops;
pub mod option;
pub mod panic;
pub mod panicking;
pub mod prelude;
pub mod ptr;
pub mod slice;
pub mod str;

mod macros;

// ---- builtin macros living at the crate root, like real core ----

/// The `#[derive(...)]` attribute itself is a builtin macro that must be
/// declared and re-exported through the prelude, like real core does.
#[rustc_builtin_macro]
pub macro derive($item:item) {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro stringify($($t:tt)*) {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro concat($($t:tt)*) {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro compile_error($($t:tt)*) {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro file() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro line() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semiopaque"]
pub macro cfg() {
    /* compiler built-in */
}

#[rustc_builtin_macro(pattern_type)]
#[macro_export]
macro_rules! pattern_type {
    ($($arg:tt)*) => {
        /* compiler built-in */
    };
}
