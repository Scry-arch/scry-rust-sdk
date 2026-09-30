//! Methods and constants of the <=32-bit integer types: wrapping arithmetic,
//! rotations, byte order, bit counting and `abs`.
//!
//! The Scry backend has no instructions for counting bits, so
//! `leading_zeros`, `trailing_zeros` and `count_ones` are computed with
//! shifts and masks here instead of through the `ctlz`/`cttz`/`ctpop`
//! intrinsics.

// The byte conversions transmute between an integer and its bytes, which is
// what the lint would have them call instead.
#![allow(unnecessary_transmutes)]

use crate::intrinsics;

// ---- 32-bit helpers every width reduces to ----

/// Leading zeros of `x`, by binary search.
#[inline]
const fn leading_zeros_u32(x: u32) -> u32 {
    if x == 0 {
        return 32;
    }
    let mut x = x;
    let mut n = 0;
    if x & 0xFFFF_0000 == 0 {
        n += 16;
        x <<= 16;
    }
    if x & 0xFF00_0000 == 0 {
        n += 8;
        x <<= 8;
    }
    if x & 0xF000_0000 == 0 {
        n += 4;
        x <<= 4;
    }
    if x & 0xC000_0000 == 0 {
        n += 2;
        x <<= 2;
    }
    if x & 0x8000_0000 == 0 {
        n += 1;
    }
    n
}

/// Trailing zeros of `x`: the position of its lowest set bit.
#[inline]
const fn trailing_zeros_u32(x: u32) -> u32 {
    if x == 0 {
        return 32;
    }
    // `x & -x` keeps only the lowest set bit.
    31 - leading_zeros_u32(x & intrinsics::wrapping_sub(0, x))
}

/// Set bits of `x`, counted in parallel in ever wider fields.
#[inline]
const fn count_ones_u32(x: u32) -> u32 {
    let x = x - ((x >> 1) & 0x5555_5555);
    let x = (x & 0x3333_3333) + ((x >> 2) & 0x3333_3333);
    let x = (x + (x >> 4)) & 0x0F0F_0F0F;
    intrinsics::wrapping_mul(x, 0x0101_0101) >> 24
}

// ---- methods common to all widths ----

macro_rules! int_impl {
    ($($t:ident: $u:ident, $bits:literal;)*) => {$(
        impl $t {
            /// The number of bits.
            pub const BITS: u32 = $bits;

            /// `self + rhs`, wrapping around at the type's bounds.
            #[inline]
            pub const fn wrapping_add(self, rhs: $t) -> $t {
                intrinsics::wrapping_add(self, rhs)
            }

            /// `self - rhs`, wrapping around at the type's bounds.
            #[inline]
            pub const fn wrapping_sub(self, rhs: $t) -> $t {
                intrinsics::wrapping_sub(self, rhs)
            }

            /// `self * rhs`, wrapping around at the type's bounds.
            #[inline]
            pub const fn wrapping_mul(self, rhs: $t) -> $t {
                intrinsics::wrapping_mul(self, rhs)
            }

            /// `-self`, wrapping around at the type's bounds.
            #[inline]
            pub const fn wrapping_neg(self) -> $t {
                intrinsics::wrapping_sub(0, self)
            }

            /// `self << rhs`, with the shift amount taken modulo `BITS`.
            #[inline]
            pub const fn wrapping_shl(self, rhs: u32) -> $t {
                self << (rhs & ($bits - 1))
            }

            /// `self >> rhs`, with the shift amount taken modulo `BITS`.
            #[inline]
            pub const fn wrapping_shr(self, rhs: u32) -> $t {
                self >> (rhs & ($bits - 1))
            }

            /// Rotates the bits left by `n`; bits shifted out re-enter on the right.
            #[inline]
            pub const fn rotate_left(self, n: u32) -> $t {
                intrinsics::rotate_left(self as $u, n) as $t
            }

            /// Rotates the bits right by `n`; bits shifted out re-enter on the left.
            #[inline]
            pub const fn rotate_right(self, n: u32) -> $t {
                intrinsics::rotate_right(self as $u, n) as $t
            }

            /// Reverses the byte order.
            #[inline]
            pub const fn swap_bytes(self) -> $t {
                intrinsics::bswap(self as $u) as $t
            }

            /// Converts from native (little-endian) to big-endian byte order.
            #[inline]
            pub const fn to_be(self) -> $t {
                self.swap_bytes()
            }

            /// Converts from native to little-endian byte order: a no-op.
            #[inline]
            pub const fn to_le(self) -> $t {
                self
            }

            /// Converts from big-endian to native (little-endian) byte order.
            #[inline]
            pub const fn from_be(x: $t) -> $t {
                x.swap_bytes()
            }

            /// Converts from little-endian to native byte order: a no-op.
            #[inline]
            pub const fn from_le(x: $t) -> $t {
                x
            }

            /// The bytes of `self`, most significant first.
            #[inline]
            pub const fn to_be_bytes(self) -> [u8; $bits / 8] {
                unsafe { intrinsics::transmute(self.to_be()) }
            }

            /// The bytes of `self`, least significant first.
            #[inline]
            pub const fn to_le_bytes(self) -> [u8; $bits / 8] {
                unsafe { intrinsics::transmute(self.to_le()) }
            }

            /// The value of the given bytes, most significant first.
            #[inline]
            pub const fn from_be_bytes(bytes: [u8; $bits / 8]) -> $t {
                <$t>::from_be(unsafe { intrinsics::transmute(bytes) })
            }

            /// The value of the given bytes, least significant first.
            #[inline]
            pub const fn from_le_bytes(bytes: [u8; $bits / 8]) -> $t {
                <$t>::from_le(unsafe { intrinsics::transmute(bytes) })
            }

            /// The number of zero bits above the highest set bit.
            #[inline]
            pub const fn leading_zeros(self) -> u32 {
                leading_zeros_u32(self as $u as u32) - (32 - $bits)
            }

            /// The number of zero bits below the lowest set bit; `BITS` for zero.
            #[inline]
            pub const fn trailing_zeros(self) -> u32 {
                if self == 0 { $bits } else { trailing_zeros_u32(self as $u as u32) }
            }

            /// The number of set bits.
            #[inline]
            pub const fn count_ones(self) -> u32 {
                count_ones_u32(self as $u as u32)
            }

            /// The number of zero bits.
            #[inline]
            pub const fn count_zeros(self) -> u32 {
                $bits - self.count_ones()
            }
        }
    )*};
}

int_impl! {
    u8: u8, 8;
    u16: u16, 16;
    u32: u32, 32;
    usize: usize, 32;
    i8: u8, 8;
    i16: u16, 16;
    i32: u32, 32;
    isize: usize, 32;
}

// ---- bounds ----

macro_rules! unsigned_impl {
    ($($t:ident)*) => {$(
        impl $t {
            pub const MIN: $t = 0;
            pub const MAX: $t = !0;

            /// Whether exactly one bit is set.
            #[inline]
            pub const fn is_power_of_two(self) -> bool {
                self.count_ones() == 1
            }
        }
    )*};
}

unsigned_impl! { u8 u16 u32 usize }

macro_rules! signed_impl {
    ($($t:ident: $u:ident)*) => {$(
        impl $t {
            pub const MIN: $t = !<$t>::MAX;
            pub const MAX: $t = (<$u>::MAX >> 1) as $t;

            /// The absolute value. Like real core, `MIN.abs()` overflows:
            /// it panics in debug builds and returns `MIN` otherwise.
            #[inline]
            pub const fn abs(self) -> $t {
                if self < 0 { -self } else { self }
            }

            /// The absolute value, wrapping `MIN` around to itself.
            #[inline]
            pub const fn wrapping_abs(self) -> $t {
                if self < 0 { self.wrapping_neg() } else { self }
            }

            /// `-1`, `0` or `1` for a negative, zero or positive value.
            #[inline]
            pub const fn signum(self) -> $t {
                if self < 0 { -1 } else if self == 0 { 0 } else { 1 }
            }
        }
    )*};
}

signed_impl! { i8: u8 i16: u16 i32: u32 isize: usize }
