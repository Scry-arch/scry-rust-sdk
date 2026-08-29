//! `macro_rules!` macros. `#[macro_export]` places them at the crate root, and
//! the compiler's `#[macro_use] extern crate core` injection puts them in
//! scope for every `#![no_std]` crate.

/// Panic. Scry-core supports only the literal forms, there is no formatting
/// machinery, so `panic!("x = {}", x)` is a compile error by design.
#[macro_export]
macro_rules! panic {
    () => {
        $crate::panicking::panic("explicit panic")
    };
    ($msg:literal $(,)?) => {
        $crate::panicking::panic($msg)
    };
    ($($arg:tt)+) => {
        $crate::compile_error!(
            "scry-core has no formatting machinery: panic! accepts only a string literal"
        )
    };
}

#[macro_export]
macro_rules! assert {
    ($cond:expr $(,)?) => {
        if !$cond {
            $crate::panicking::panic($crate::concat!(
                "assertion failed: ",
                $crate::stringify!($cond)
            ))
        }
    };
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            $crate::panicking::panic($msg)
        }
    };
}

#[macro_export]
macro_rules! unreachable {
    () => {
        $crate::panicking::panic("internal error: entered unreachable code")
    };
    ($msg:literal $(,)?) => {
        $crate::panicking::panic($msg)
    };
}
