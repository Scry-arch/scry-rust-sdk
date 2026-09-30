//! The integer methods of the SDK's `core`: wrapping arithmetic, shifts, rotations, byte order,
//! bit counting, bounds, and `min`/`max`/`clamp`.
//!
//! Every check sets one bit of the returned value when it passes, so a wrong result shows which
//! check failed. Inputs pass through `black_box` so that the checks run on the target instead of
//! being folded at compile time.

#![no_std]
#![no_main]

use core::cmp::{self, Ordering};
use core::hint::black_box;

struct Checks {
    passed: u32,
    next: u32,
}

impl Checks {
    fn check(&mut self, ok: bool) {
        if ok {
            self.passed |= 1 << self.next;
        }
        self.next += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    let mut c = Checks { passed: 0, next: 0 };

    // 0: wrapping arithmetic
    c.check(
        black_box(0xFFFF_FFFFu32).wrapping_add(2) == 1
            && black_box(1u8).wrapping_sub(2) == 255
            && black_box(0x1234_5678u32).wrapping_mul(0x9E37_79B9) == 0x8879_34B8
            && black_box(i8::MIN).wrapping_neg() == i8::MIN
            && black_box(5i32).wrapping_neg() == -5,
    );
    // 1: shifts with the amount taken modulo the width
    c.check(black_box(1u32).wrapping_shl(33) == 2 && black_box(0x80u8).wrapping_shr(9) == 0x40);
    // 2: shifts by an amount of another integer type
    c.check(black_box(0x81u8) << black_box(1u32) == 0x02 && black_box(0x8000u16) >> black_box(15i32) == 1);
    // 3: rotations
    c.check(
        black_box(0x8000_0001u32).rotate_left(4) == 0x18
            && black_box(0x1234_5678u32).rotate_right(8) == 0x7812_3456
            && black_box(0x81u8).rotate_left(1) == 0x03
            && black_box(-2i16).rotate_right(1) == 0x7FFF,
    );
    // 4: byte swapping
    c.check(
        black_box(0x1234_5678u32).swap_bytes() == 0x7856_3412
            && black_box(0x1234u16).swap_bytes() == 0x3412
            && black_box(0xABu8).swap_bytes() == 0xAB,
    );
    // 5: byte order conversions
    c.check(
        black_box(0x1234_5678u32).to_be_bytes() == [0x12, 0x34, 0x56, 0x78]
            && black_box(0x1234_5678u32).to_le_bytes() == [0x78, 0x56, 0x34, 0x12]
            && u32::from_be_bytes(black_box([0xDE, 0xAD, 0xBE, 0xEF])) == 0xDEAD_BEEF
            && u16::from_le_bytes(black_box([0x34, 0x12])) == 0x1234
            && u32::from_be(black_box(0x1234_5678u32).to_be()) == 0x1234_5678,
    );
    // 6: leading zeros
    c.check(
        black_box(1u32).leading_zeros() == 31
            && black_box(0u32).leading_zeros() == 32
            && black_box(0x80u8).leading_zeros() == 0
            && black_box(1u16).leading_zeros() == 15
            && black_box(-1i32).leading_zeros() == 0
            && black_box(0x0001_0000u32).leading_zeros() == 15,
    );
    // 7: trailing zeros
    c.check(
        black_box(8u32).trailing_zeros() == 3
            && black_box(0u8).trailing_zeros() == 8
            && black_box(0x8000_0000u32).trailing_zeros() == 31
            && black_box(-4i16).trailing_zeros() == 2,
    );
    // 8: counting bits
    c.check(
        black_box(0xF0F0_0001u32).count_ones() == 9
            && black_box(-1i8).count_ones() == 8
            && black_box(0u16).count_zeros() == 16
            && black_box(64u32).is_power_of_two()
            && !black_box(96u32).is_power_of_two(),
    );
    // 9: bounds
    c.check(
        i8::MIN == -128
            && i8::MAX == 127
            && i16::MIN == -32768
            && i32::MAX == 2_147_483_647
            && u16::MAX == 65535
            && u8::MIN == 0
            && usize::BITS == 32
            && black_box(u32::MAX) == 0xFFFF_FFFF,
    );
    // 10: abs and signum
    c.check(
        black_box(-7i32).abs() == 7
            && black_box(7i32).abs() == 7
            && black_box(i32::MIN).wrapping_abs() == i32::MIN
            && black_box(-3i8).signum() == -1
            && black_box(0i16).signum() == 0
            && black_box(9isize).signum() == 1,
    );
    // 11: min, max and clamp
    c.check(
        black_box(3u32).max(9) == 9
            && black_box(-3i32).min(2) == -3
            && black_box(20i32).clamp(-5, 10) == 10
            && black_box(-20i32).clamp(-5, 10) == -5
            && black_box(4i32).clamp(-5, 10) == 4
            && cmp::max(black_box(1u8), 2) == 2
            && cmp::min(black_box(1u8), 2) == 1,
    );
    // 12: three-way comparison
    let (a, b) = (black_box(3i32), black_box(5i32));
    c.check(
        a.cmp(&b) == Ordering::Less
            && b.cmp(&a) == Ordering::Greater
            && a.cmp(&a) == Ordering::Equal
            && a.cmp(&b).is_lt()
            && a.cmp(&b).reverse() == Ordering::Greater
            && a.cmp(&a).then(Ordering::Less) == Ordering::Less,
    );

    c.passed
}
