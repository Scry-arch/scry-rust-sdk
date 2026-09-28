//! Volatile accesses in an optimised build.
//!
//! Each block below is a pattern that the optimiser rewrites for ordinary loads and stores: it
//! merges repeated reads, forwards a stored value to a later read, and removes overwritten,
//! repeated or unused accesses. A merged read returns the same value as the reads it replaced, so
//! the returned operands cannot show the difference. `release-metrics.txt` therefore pins the
//! number of memory accesses the simulator counts in a release build.

#![no_std]
#![no_main]

use core::ptr;

static mut REGISTER: u32 = 1;
static mut FIFO: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    let register = &raw mut REGISTER;
    let fifo = &raw mut FIFO;

    unsafe {
        // Two writes of different values: the first is not dead.
        register.write_volatile(7);
        ptr::write_volatile(register, 13);

        // The same value twice: the second write is not redundant.
        fifo.write_volatile(3);
        fifo.write_volatile(3);

        // Three reads of one register: they are not merged into one, and the first is not
        // replaced by the value written above.
        let sum = register.read_volatile()
            + (register as *const u32).read_volatile()
            + ptr::read_volatile(register);

        // A read whose value is unused is still made, as reading can have an effect.
        let _ = fifo.read_volatile();
        let byte = fifo.read_volatile();

        // 3 * 13 + 3 = 42
        sum + byte as u32
    }
}
