#![expect(incomplete_features)]
#![feature(field_projections)]

use std::ops::place::{borrow_place, read_place, write_place};

struct Point {
    x: i32,
}

fn main() {
    let mut val = 10;
    let mut pt = Point { x: 10 };

    // Bare locals require `unsafe` for the place operation itself.
    let _ = read_place!(val);
    //~^ ERROR: call to unsafe function `read_place`
    write_place!(val = 20);
    //~^ ERROR: call to unsafe function `write_place`
    let _ = borrow_place!(@<&i32> val);
    //~^ ERROR: call to unsafe function `borrow_place`

    // Field projections additionally require `unsafe` for `project_place`.
    let _ = read_place!(pt.x);
    //~^ ERROR: call to unsafe function `read_place`
    //~| ERROR: call to unsafe function `project_place`
    write_place!(pt.x = 20);
    //~^ ERROR: call to unsafe function `write_place`
    //~| ERROR: call to unsafe function `project_place`
    let _ = borrow_place!(@<&i32> pt.x);
    //~^ ERROR: call to unsafe function `borrow_place`
    //~| ERROR: call to unsafe function `project_place`
}
