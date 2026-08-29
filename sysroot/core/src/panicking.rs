//! Panic entry points. On Scry every panic is an immediate trap, there is no
//! unwinding, no `#[panic_handler]` indirection, and no message formatting.

use crate::intrinsics;
use crate::stringify;

#[lang = "panic"]
#[track_caller]
pub fn panic(_msg: &'static str) -> ! {
    intrinsics::abort();
}

macro_rules! panic_const {
    ($($lang:ident = $message:expr,)+) => {
        pub mod panic_const {
            use super::*;

            $(
                #[track_caller]
                #[lang = stringify!($lang)]
                pub fn $lang() -> ! {
                    panic($message);
                }
            )+
        }
    }
}

panic_const! {
    panic_const_add_overflow = "attempt to add with overflow",
    panic_const_sub_overflow = "attempt to subtract with overflow",
    panic_const_mul_overflow = "attempt to multiply with overflow",
    panic_const_div_overflow = "attempt to divide with overflow",
    panic_const_rem_overflow = "attempt to calculate the remainder with overflow",
    panic_const_neg_overflow = "attempt to negate with overflow",
    panic_const_shr_overflow = "attempt to shift right with overflow",
    panic_const_shl_overflow = "attempt to shift left with overflow",
    panic_const_div_by_zero = "attempt to divide by zero",
    panic_const_rem_by_zero = "attempt to calculate the remainder with a divisor of zero",
}

#[lang = "panic_bounds_check"]
#[track_caller]
fn panic_bounds_check(_index: usize, _len: usize) -> ! {
    intrinsics::abort();
}

#[lang = "panic_cannot_unwind"]
#[track_caller]
fn panic_cannot_unwind() -> ! {
    intrinsics::abort();
}

#[lang = "panic_in_cleanup"]
fn panic_in_cleanup() -> ! {
    loop {}
}

// Never called with panic=abort; present so nothing ever demands it.
#[lang = "eh_personality"]
fn eh_personality(
    _version: i32,
    _actions: i32,
    _exception_class: u32,
    _exception_object: *mut (),
    _context: *mut (),
) -> i32 {
    loop {}
}
