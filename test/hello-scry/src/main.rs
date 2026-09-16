//! A minimal Scry program. Build and run it with `cargo scry run`; scryer prints the value
//! returned from `_start` under "Returned Operands".

#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    add(40, 2)
}

#[inline(never)]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
