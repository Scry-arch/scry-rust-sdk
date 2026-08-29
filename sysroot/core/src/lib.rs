//! The Scry `core`: a minimal core implementation for Scry targets.
//!
//! Deliberate divergences from real core:
//! - Operator impls exist ONLY for <=32-bit types; `u64 + u64` etc. fail typeck (E0369).
//! - Panics trap immediately; no `#[panic_handler]`, no unwinding, no formatting.
//!   `panic!` accepts only a string literal.
//! - Not yet provided: floats, Iterator/for-loops, ranges, Ordering/Eq/Ord,
//!   derive(PartialOrd), str comparison, UnsafeCell, fmt.
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
    freeze_impls
)]
#![no_core]
#![allow(dead_code, internal_features, ambiguous_wide_pointer_comparisons)]

pub mod arch;
pub mod clone;
pub mod cmp;
pub mod ffi;
pub mod intrinsics;
pub mod marker;
pub mod mem;
pub mod ops;
pub mod option;
pub mod panic;
pub mod panicking;
pub mod prelude;
pub mod ptr;

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
