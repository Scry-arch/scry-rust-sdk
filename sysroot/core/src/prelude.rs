//! The prelude. The compiler injects `use core::prelude::rust_20XX::*;` into
//! every `#![no_std]` crate, so these modules MUST exist for user code to
//! compile at all.

pub mod v1 {
    pub use crate::clone::Clone;
    pub use crate::cmp::{PartialEq, PartialOrd};
    pub use crate::iter::{IntoIterator, Iterator};
    pub use crate::marker::{Copy, Send, Sized, Sync, Unpin};
    pub use crate::mem::drop;
    pub use crate::ops::{Drop, Fn, FnMut, FnOnce};
    pub use crate::option::Option;
    pub use crate::option::Option::{None, Some};

    // Builtin macros (the derive macros ride along with the trait re-exports
    // above, sharing their paths in the macro namespace).
    pub use crate::{cfg, compile_error, concat, derive, file, line, stringify};

    // The injected `extern crate core` is NOT `#[macro_use]`, so our
    // macro_rules! macros reach user code through the prelude, not implicitly.
    pub use crate::{assert, panic, unreachable};
}

pub mod rust_2015 {
    pub use super::v1::*;
}

pub mod rust_2018 {
    pub use super::v1::*;
}

pub mod rust_2021 {
    pub use super::v1::*;
}

pub mod rust_2024 {
    pub use super::v1::*;
    pub use crate::mem::{align_of, size_of};
}
