//@ check-pass
#![no_std]
#![crate_type = "lib"]
#![expect(incomplete_features)]
#![feature(field_projections)]

use core::ops::place::{borrow_place, read_place, write_place};

pub struct Pair {
    pub a: u32,
    pub b: u32,
}

pub unsafe fn exercise(p: &mut Pair) -> u32 {
    #[allow(dead_code)]
    struct ReadPlace;
    #[allow(dead_code)]
    struct WritePlace;
    #[allow(dead_code)]
    struct BorrowPlace;

    unsafe {
        write_place!(p.a = 42);
        let r: &u32 = borrow_place!(@p.a);
        let m = borrow_place!(@<&mut u32> p.b);
        *m = *r + 1;
        read_place!(p.b)
    }
}
