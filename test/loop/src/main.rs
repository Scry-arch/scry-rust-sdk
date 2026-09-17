//! A minimal Scry program using loops

#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    fn_loop(40, 2)
}

#[inline(never)]
fn fn_loop(a: u32, b: u32) -> u32 {
	let mut acc = 0;
	let mut i = 0;
    loop {
		if i<b {
			acc += a;
			i += 1;
		} else {
			break;
		}
	}
	acc
}
