//! A minimal Scry program using for-loops

#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    fn_loop(2, 40)
}

#[inline(never)]
fn fn_loop(a: u32, b: u32) -> u32 {
	let mut acc = 0;
	for _ in 0..b {
		acc += a;
	}
	acc
}
