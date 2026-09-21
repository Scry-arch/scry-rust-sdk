//! Volatile reads and writes, as used for memory-mapped I/O.
//!
//! The simulator has no device registers, so a writable static stands in for one. The test covers
//! both the function forms in `core::ptr` and the method forms on raw pointers, for two widths.

#![no_std]
#![no_main]

use core::ptr;

static mut REGISTER: u32 = 1;
static mut STATUS: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    let register = &raw mut REGISTER;
    let status = &raw mut STATUS;

    unsafe {
        // Method forms.
        register.write_volatile(40);
        let first = register.read_volatile();

        // Function forms.
        ptr::write_volatile(register, first + 1);
        let second = ptr::read_volatile(register as *const u32);

        // A byte-wide register, the width the board's MMIO block uses.
        status.write_volatile(3);
        let flags = (status as *const u8).read_volatile();

        // 40 + 41 + 3 - 42 = 42
        first + second + flags as u32 - 42
    }
}
