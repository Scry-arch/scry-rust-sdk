//! Hints to the compiler.

use crate::mem::MaybeUninit;
use crate::ptr;

/// Returns `dummy` unchanged, but in a way the optimiser cannot see through: it
/// must assume the value is used, and that the returned value could be
/// anything. Benchmarks use it to keep inputs from being constant-folded and
/// results from being optimised away.
///
/// The backend ignores the `black_box` intrinsic, so this passes the value
/// through a volatile store and load of a local instead. That costs one store
/// and one load, which is also what the C benchmarks' `volatile` variables
/// cost.
#[inline]
pub fn black_box<T>(dummy: T) -> T {
    let mut slot = MaybeUninit::<T> { uninit: () };
    let slot = &raw mut slot as *mut T;
    unsafe {
        ptr::write_volatile(slot, dummy);
        ptr::read_volatile(slot)
    }
}
