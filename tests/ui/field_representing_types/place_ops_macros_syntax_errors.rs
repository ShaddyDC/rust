//@ edition:2021
#![expect(incomplete_features)]
#![feature(field_projections)]

use std::ops::place::{borrow_place, read_place, write_place};

fn main() {
    let mut x = 1;
    unsafe {
        let _: i32 = read_place!();
        //~^ ERROR: unexpected end of macro invocation
        let _: i32 = read_place!(x y);
        //~^ ERROR: no rules expected `y`
        let _: i32 = read_place!(/** */);
        //~^ ERROR: expected expression, found end of macro arguments

        write_place!();
        //~^ ERROR: expected `place = value` in `write_place!`
        write_place!(x);
        //~^ ERROR: expected `place = value` in `write_place!`
        write_place!(x += 1);
        //~^ ERROR: expected `place = value` in `write_place!`
        write_place!(= 1);
        //~^ ERROR: expected a place expression in `write_place!`
        write_place!(x =);
        //~^ ERROR: expected expression, found end of macro arguments
        write_place!(x = 1 2);
        //~^ ERROR: expected one of
        write_place!(x y = 1);
        //~^ ERROR: expected one of
        write_place!(/** */ = 1);
        //~^ ERROR: expected expression, found `=`
        write_place!(x = /** */);
        //~^ ERROR: expected expression, found end of macro arguments
        write_place!((x = 1));
        //~^ ERROR: expected `place = value` in `write_place!`
        write_place!(x === 1);
        //~^ ERROR: invalid comparison operator `===`
        write_place!(x = 1 === 2);
        //~^ ERROR: invalid comparison operator `===`
        write_place!(let x = 1);
        //~^ ERROR: expected expression, found `let` statement
        write_place!(x = #[cfg(false)] 1);
        //~^ ERROR: attributes on expressions are experimental
        //~| ERROR: `cfg` is not supported in this position
        write_place!(x = #[allow(unused)] 1);
        //~^ ERROR: attributes on expressions are experimental

        let _: &i32 = borrow_place!();
        //~^ ERROR: expected `@` before place expression in `borrow_place!`
        let _: &i32 = borrow_place!(x);
        //~^ ERROR: expected `@` before place expression in `borrow_place!`
        let _: &i32 = borrow_place!(@);
        //~^ ERROR: expected a place expression in `borrow_place!`
        let _: &i32 = borrow_place!(@<&i32>);
        //~^ ERROR: expected a place expression in `borrow_place!`
        let _: &i32 = borrow_place!(@<&i32 x);
        //~^ ERROR: expected one of
        let _: &i32 = borrow_place!(@<...> x);
        //~^ ERROR: unexpected `...`
        let _: &i32 = borrow_place!(@<'a> x);
        //~^ ERROR: expected type, found lifetime
        let _: &i32 = borrow_place!(@<i32> x === 1);
        //~^ ERROR: invalid comparison operator `===`
        let _: &i32 = borrow_place!(@<i32> let y = 1);
        //~^ ERROR: expected expression, found `let` statement
        let _: &i32 = borrow_place!(@x y);
        //~^ ERROR: expected one of
        let _: &i32 = borrow_place!(@<i32>> x);
        //~^ ERROR: expected expression, found `>`
        let _: &i32 = borrow_place!(@<i32>= x);
        //~^ ERROR: expected expression, found `=`
        let _: &i32 = borrow_place!(@/** */);
        //~^ ERROR: expected expression, found end of macro arguments
    }
}
