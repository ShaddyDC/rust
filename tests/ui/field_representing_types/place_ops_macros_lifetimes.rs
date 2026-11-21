#![expect(incomplete_features)]
#![feature(field_projections)]

use std::ops::place::borrow_place;

struct Point {
    x: i32,
}

// Note: `mut p` avoids `thir/cx/expr.rs`'s `PtrToPtr` cast for immutable bindings in #164627,
// which currently erases `'a` on `&raw const p as *mut _`.
fn extend_ref<'a>(mut p: &'a Point) -> &'static i32 {
    unsafe { borrow_place!(@<&'static i32> p.x) }
    //~^ ERROR: borrowed data escapes outside of function
}

fn extend_mut<'a>(mut p: &'a mut Point) -> &'static mut i32 {
    unsafe { borrow_place!(@<&'static mut i32> p.x) }
    //~^ ERROR: borrowed data escapes outside of function
}

fn main() {}
