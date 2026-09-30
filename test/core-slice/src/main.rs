//! Slices, arrays, iteration and strings in the SDK's `core`: length, range indexing, iteration
//! (including reversed, enumerated and inclusive ranges), `swap`, `fill`, `copy_from_slice`,
//! equality, pointer copies and `str`.
//!
//! Every check sets one bit of the returned value when it passes, so a wrong result shows which
//! check failed. Data and indices pass through `black_box` so that the checks run on the target.

#![no_std]
#![no_main]

use core::hint::black_box;
use core::{ptr, slice};

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
    let arr = black_box([1u32, 2, 3, 4, 5]);
    let (one, three) = (black_box(1usize), black_box(3usize));

    // 0: length
    c.check(arr.len() == 5 && !arr.is_empty() && arr[..0].is_empty() && arr[one..three].len() == 2);
    // 1: range indexing, all five kinds
    c.check(
        arr[one..three] == [2, 3]
            && arr[three..] == [4, 5]
            && arr[..one] == [1]
            && arr[..] == [1, 2, 3, 4, 5]
            && arr[one..=three] == [2, 3, 4],
    );
    // 2: get
    c.check(arr.get(4) == Some(&5) && arr.get(black_box(5)).is_none());
    // 3: iteration by reference, as a method and in `for`
    let mut sum = 0;
    for x in arr.iter() {
        sum += *x;
    }
    for x in &arr {
        sum += *x;
    }
    c.check(sum == 30);
    // 4: reversed and enumerated iteration
    let mut weighted = 0;
    for (i, x) in arr.iter().enumerate() {
        weighted += i as u32 * *x;
    }
    let mut digits = 0;
    for x in arr.iter().rev() {
        digits = digits * 10 + *x;
    }
    c.check(weighted == 40 && digits == 54321);
    // 5: iteration over mutable references
    let mut doubled = arr;
    for x in doubled.iter_mut() {
        *x *= 2;
    }
    for x in &mut doubled {
        *x += 1;
    }
    c.check(doubled == [3, 5, 7, 9, 11]);
    // 6: swap, fill and copy_from_slice
    let mut buf = arr;
    buf.swap(0, black_box(4));
    let swapped = buf == [5, 2, 3, 4, 1];
    buf[one..three].fill(0);
    let filled = buf == [5, 0, 0, 4, 1];
    buf[..3].copy_from_slice(&arr[2..]);
    c.check(swapped && filled && buf == [3, 4, 5, 4, 1]);
    // 7: inclusive and reversed ranges; a u8 range up to 255 must stop
    let mut total = 0;
    for i in 1..=black_box(10u32) {
        total += i;
    }
    let mut countdown = 0;
    for i in (0..black_box(5u32)).rev() {
        countdown = countdown * 10 + i;
    }
    let mut steps = 0;
    for _ in black_box(250u8)..=255 {
        steps += 1;
    }
    c.check(total == 55 && countdown == 43210 && steps == 6);
    // 8: equality of slices, arrays and byte strings
    let bytes = black_box(*b"abcdef");
    c.check(
        &bytes[..3] == b"abc"
            && &bytes[3..] != b"dex"
            && bytes[..2] != bytes[1..3]
            && arr[..] == arr
            && [0u8; 0] == [],
    );
    // 9: strings
    let word = black_box("hello");
    c.check(
        word.len() == 5
            && word == "hello"
            && word != "help!"
            && word.as_bytes()[1] == b'e'
            && "".is_empty(),
    );
    // 10: raw parts and pointer copies
    let part = unsafe { slice::from_raw_parts(arr.as_ptr().add(1), 3) };
    let mut shifted = arr;
    // Overlapping copy, as memmove: shift the first four right by one.
    unsafe { ptr::copy(shifted.as_ptr(), shifted.as_mut_ptr().add(1), 4) };
    c.check(part == [2, 3, 4] && shifted == [1, 1, 2, 3, 4]);
    // 11: two-dimensional arrays
    let mut grid = [[0u8; 3]; 2];
    for (r, row) in grid.iter_mut().enumerate() {
        for (col, cell) in row.iter_mut().enumerate() {
            *cell = (r * 3 + col) as u8;
        }
    }
    c.check(grid == [[0, 1, 2], [3, 4, 5]] && grid[1][black_box(2)] == 5);

    c.passed
}
