//! Comparison in the SDK's `core`: derived `PartialEq`, `Eq`, `PartialOrd` and `Ord` on structs
//! and enums, comparison through references, and a generic sort over `Ord`.
//!
//! Every check sets one bit of the returned value when it passes, so a wrong result shows which
//! check failed. Values pass through `black_box` so that the checks run on the target.

#![no_std]
#![no_main]

use core::cmp::Ordering;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    Dot,
    Line(u8),
    Box { w: u8, h: u8 },
}

/// Insertion sort over any `Ord` type.
fn sort<T: Ord + Copy>(items: &mut [T]) {
    for i in 1..items.len() {
        let mut j = i;
        while j > 0 && items[j - 1] > items[j] {
            items.swap(j - 1, j);
            j -= 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    let mut c = Checks { passed: 0, next: 0 };
    let p = black_box(Point { x: 1, y: 2 });
    let q = black_box(Point { x: 1, y: 3 });
    let r = black_box(Point { x: 0, y: 9 });

    // 0: derived equality of structs
    c.check(p == p && p != q);
    // 1: derived ordering of structs is lexicographic by field
    c.check(p < q && r < p && q > r && p <= p && p.cmp(&q) == Ordering::Less);
    // 2: min and max of derived types
    c.check(p.max(q) == q && p.min(r) == r);
    // 3: derived equality of enums, with and without fields
    let line = black_box(Shape::Line(4));
    c.check(
        line == Shape::Line(4)
            && line != Shape::Line(5)
            && line != Shape::Dot
            && Shape::Box { w: 1, h: 2 } == black_box(Shape::Box { w: 1, h: 2 }),
    );
    // 4: derived ordering of enums: by variant first, then by fields
    c.check(
        black_box(Shape::Dot) < line
            && line < Shape::Line(5)
            && line < Shape::Box { w: 0, h: 0 }
            && Shape::Box { w: 1, h: 2 } < black_box(Shape::Box { w: 1, h: 3 }),
    );
    // 5: comparison through references
    let (a, b) = (&p, &q);
    c.check(a < b && a == &p && a.cmp(&b) == Ordering::Less);
    // 6: a generic sort over `Ord`
    let mut points = black_box([q, r, p, Point { x: -1, y: 0 }]);
    sort(&mut points);
    let mut numbers = black_box([5i16, -2, 9, 0, -2]);
    sort(&mut numbers);
    c.check(points == [Point { x: -1, y: 0 }, r, p, q] && numbers == [-2, -2, 0, 5, 9]);

    c.passed
}
