#![expect(incomplete_features)]
#![feature(field_projections)]

use std::ops::place::{borrow_place, read_place, write_place};
use std::ptr::NonNull;

struct NonCopy(i32);

struct Point {
    x: i32,
    nc: NonCopy,
}

fn dummy_ptr<T: ?Sized>() -> *mut i32 {
    std::ptr::null_mut()
}

fn infer_borrow(pt: &Point) {
    // Unconstrained `borrow_place!(@place)` requires type inference context.
    unsafe {
        let _ = borrow_place!(@pt.x);
        //~^ ERROR: type annotations needed
    }
}

fn main() {
    let pt = Point { x: 10, nc: NonCopy(1) };

    // `RefHandle` does not allow writing, mutable borrowing, or reading `!Copy` values.
    let shared = &pt;
    unsafe {
        write_place!(shared.x = 30);
        //~^ ERROR: the trait bound `RefHandle<'_, i32>: WritePlace` is not satisfied
        let _ = borrow_place!(@<&mut i32> shared.x);
        //~^ ERROR: the trait bound `RefHandle<'_, i32>: BorrowPlace<&mut i32>` is not satisfied
        let _ = borrow_place!(@<*mut i32> shared.x);
        //~^ ERROR: the trait bound `RefHandle<'_, i32>: BorrowPlace<*mut i32>` is not satisfied
        let _ = borrow_place!(@<NonNull<i32>> shared.x);
        //~^ ERROR: the trait bound `RefHandle<'_, i32>: BorrowPlace<NonNull<i32>>` is not satisfied
        let _ = read_place!(shared.nc);
        //~^ ERROR: the trait bound `NonCopy: Copy` is not satisfied
    }

    // Unsized targets cannot be read via `ReadPlace`.
    let slice: &[i32] = &[1, 2, 3];
    unsafe {
        let _ = read_place!(*slice);
        //~^ ERROR: the trait bound `[i32]: Copy` is not satisfied
        //~| ERROR: the size for values of type `[i32]` cannot be known at compilation time
    }

    // Temporaries (including LHS with `=` inside `<...>`) are rejected cleanly as non-places.
    unsafe {
        let _: i32 = read_place!(42);
        //~^ ERROR: cannot take address of a temporary
        write_place!(42 = 1);
        //~^ ERROR: cannot take address of a temporary
        write_place!(*dummy_ptr::<dyn Iterator<Item = i32>>() = 1);
        //~^ ERROR: cannot take address of a temporary
        let _ = borrow_place!(@<&i32> 42);
        //~^ ERROR: cannot take address of a temporary
    }
}
