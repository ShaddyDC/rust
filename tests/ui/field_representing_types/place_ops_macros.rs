//@ run-pass
#![expect(incomplete_features)]
#![feature(field_projections)]

use std::cell::Cell;
use std::fmt::Debug;
use std::ops::place::{borrow_place, read_place, write_place};
use std::ptr::NonNull;

struct Point {
    x: i32,
    y: i32,
}

struct Line {
    start: Point,
    end: Point,
}

struct PtrHolder<'a> {
    r: &'a mut i32,
    p: *mut i32,
}

trait PtrAlias {
    type Ptr;
}

struct Alias;

impl PtrAlias for Alias {
    type Ptr = *const i32;
}

static STATIC_POINT: Point = Point { x: 7, y: 8 };
static mut STATIC_MUT_VAL: i32 = 10;

macro_rules! place_mac {
    ($p:expr) => {
        $p
    };
}

macro_rules! forward_write {
    ($p:expr, $v:expr) => {
        write_place!($p = $v)
    };
}

macro_rules! forward_assign {
    ($e:expr) => {
        write_place!($e)
    };
}

macro_rules! forward_borrow {
    ($t:ty, $p:expr) => {
        borrow_place!(@<$t> $p)
    };
}

#[deny(unused_parens, unused_braces)]
fn test_hygiene_shadowing() {
    #[allow(dead_code, non_camel_case_types)]
    mod core {}
    #[allow(dead_code, non_camel_case_types)]
    mod std {}
    #[allow(dead_code, non_camel_case_types)]
    mod ops {}
    #[allow(dead_code, non_camel_case_types)]
    mod place {}
    #[allow(dead_code)]
    struct ReadPlace;
    #[allow(dead_code)]
    struct WritePlace;
    #[allow(dead_code)]
    struct BorrowPlace;
    #[allow(unused_macros)]
    macro_rules! raw_handle {
        ($($tt:tt)*) => {
            compile_error!("shadowed raw_handle should not be called")
        };
    }

    let mut x = 1;
    unsafe {
        write_place!(x = 2);
        assert_eq!(read_place!(x), 2);
        let r: &i32 = borrow_place!(@x);
        assert_eq!(*r, 2);
    }
}

fn main() {
    test_hygiene_shadowing();

    let mut line = Line { start: Point { x: 10, y: 20 }, end: Point { x: 30, y: 40 } };
    let mut val1 = 50;
    let mut val2 = 60;
    let holder = PtrHolder { r: &mut val1, p: &raw mut val2 };
    let mut tup = (100, 200);

    unsafe {
        // `read_place!` on locals, nested fields, tuples, parens, refs, raw pointers, and statics.
        assert_eq!(read_place!(line.start.x), 10);
        assert_eq!(read_place!((line.start).y), 20);
        assert_eq!(read_place!(tup.1), 200);
        assert_eq!(read_place!(place_mac!(line.end.x)), 30);
        assert_eq!(read_place!(STATIC_POINT.x), 7);
        assert_eq!(read_place!(STATIC_MUT_VAL), 10);

        let shared = &line;
        assert_eq!(read_place!(shared.end.y), 40);
        assert_eq!(read_place!((*shared).end.x), 30);

        assert_eq!(read_place!(*holder.r), 50);
        assert_eq!(read_place!(*holder.p), 60);

        // `write_place!` on locals, nested fields, tuples, refs, raw pointers, statics, and macros.
        write_place!(line.start.y = 999);
        assert_eq!(read_place!(line.start.y), 999);

        write_place! { line.start.x = 12 }
        let () = write_place!(line.start.x = 13);
        assert_eq!(line.start.x, 13);

        write_place!(STATIC_MUT_VAL = 25);
        assert_eq!(read_place!(STATIC_MUT_VAL), 25);

        write_place!(tup.0 = {
            let a = 120;
            a + 3
        });
        assert_eq!(tup.0, 123);

        write_place!(place_mac!(line.end.x) = 777);
        assert_eq!(line.end.x, 777);

        forward_write!(line.end.y, 888);
        assert_eq!(line.end.y, 888);

        forward_assign!(line.start.x = 222);
        assert_eq!(line.start.x, 222);

        let line_ref = &mut line;
        write_place!(line_ref.start.x = 111);
        assert_eq!(read_place!(line_ref.start.x), 111);

        write_place!(*holder.r = 555);
        write_place!(*holder.p = 666);

        // Evaluation order in `write_place!`: place handle MUST be evaluated before value.
        let mut a = 1;
        let mut b = 2;
        let mut ptr: *mut i32 = &raw mut a;
        write_place!(*ptr = {
            ptr = &raw mut b;
            99
        });
        assert_eq!(a, 99);
        assert_eq!(b, 2);
        assert_eq!(ptr, &raw mut b);

        // `borrow_place!` with inferred type (`@place`), `@<_>`, and explicit `@<Type>`.
        let r_inferred: &i32 = borrow_place!(@line.start.x);
        assert_eq!(*r_inferred, 111);

        let r_underscore: &i32 = borrow_place!(@<_> line.start.y);
        assert_eq!(*r_underscore, 999);

        let r_forwarded = forward_borrow!(&i32, line.start.x);
        assert_eq!(*r_forwarded, 111);

        let r_place_mac: &i32 = borrow_place!(@place_mac!(line.end.y));
        assert_eq!(*r_place_mac, 888);

        let r_static = borrow_place!(@<&i32> STATIC_POINT.y);
        assert_eq!(*r_static, 8);

        let m_inferred: &mut i32 = borrow_place!(@line.end.x);
        *m_inferred = 321;
        assert_eq!(line.end.x, 321);

        let r_explicit = borrow_place!(@<&i32> line.end.y);
        assert_eq!(*r_explicit, 888);

        // Nested generic closing `>>` and `>>>` in `@<...>`.
        let mut nn = borrow_place!(@<NonNull<i32>> line.start.x);
        assert_eq!(*nn.as_ref(), 111);
        let nnn = borrow_place!(@<NonNull<NonNull<i32>>> nn);
        assert_eq!(*nnn.as_ref().as_ref(), 111);

        // Qualified associated type starting with `<<` in `@<<Alias as PtrAlias>::Ptr>`.
        let qptr = borrow_place!(@<<Alias as PtrAlias>::Ptr> line.start.y);
        assert_eq!(*qptr, 999);

        // Borrowing through `&T`, `&mut T`, and raw pointers.
        let shared = &line;
        let shared_r: &i32 = borrow_place!(@shared.end.x);
        assert_eq!(*shared_r, 321);
        let shared_partial = borrow_place!(@<&_> shared.end.x);
        assert_eq!(*shared_partial, 321);
        let shared_p = borrow_place!(@<*const i32> (*shared).end.y);
        assert_eq!(*shared_p, 888);

        let line_mut = &mut line;
        let mut_r = borrow_place!(@<&mut i32> line_mut.start.y);
        *mut_r = 444;
        let mut_shared = borrow_place!(@<&i32> line_mut.start.y);
        assert_eq!(*mut_shared, 444);
        let mut_const_ptr = borrow_place!(@<*const i32> line_mut.start.y);
        assert_eq!(*mut_const_ptr, 444);
        let mut_ptr = borrow_place!(@<*mut i32> line_mut.start.y);
        assert_eq!(*mut_ptr, 444);
        let mut_partial = borrow_place!(@<&mut _> line_mut.start.y);
        assert_eq!(*mut_partial, 444);
        let mut_nn = borrow_place!(@<NonNull<i32>> (*line_mut).start.y);
        assert_eq!(*mut_nn.as_ref(), 444);

        let local_mut_ptr = borrow_place!(@<*mut i32> line.end.x);
        assert_eq!(*local_mut_ptr, 321);

        let raw_r = borrow_place!(@<&i32> *holder.p);
        assert_eq!(*raw_r, 666);

        // Non-dropping `write_place!` overwrite and `!Copy` `read_place!`/`write_place!`.
        struct DropCount<'a>(&'a Cell<usize>);
        impl Drop for DropCount<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Cell::new(0);
        {
            let mut slot = DropCount(&drops);
            write_place!(slot = DropCount(&drops));
            assert_eq!(drops.get(), 0);
        }
        assert_eq!(drops.get(), 1);

        let next_local = String::from("world");
        let next_mut = String::from("done");
        let mut local_s = String::from("hello");
        let moved_local = read_place!(local_s);
        write_place!(local_s = next_local);
        assert_eq!(moved_local, "hello");
        assert_eq!(local_s, "world");

        let s_ref = &mut local_s;
        let moved_mut = read_place!(*s_ref);
        write_place!(*s_ref = next_mut);
        assert_eq!(moved_mut, "world");
        assert_eq!(local_s, "done");

        // Unsized targets behind `&[i32]`, `&mut [i32]`, `*mut [i32]`, and `&dyn Debug`.
        let slice: &[i32] = &[1, 2, 3];
        let borrowed_slice = borrow_place!(@<&[i32]> *slice);
        assert_eq!(borrowed_slice, &[1, 2, 3]);

        let mut arr = [4, 5, 6];
        let mut_slice: &mut [i32] = &mut arr;
        let borrowed_mut_slice = borrow_place!(@<&mut [i32]> *mut_slice);
        borrowed_mut_slice[1] = 50;
        assert_eq!(arr, [4, 50, 6]);

        let raw_slice: *mut [i32] = &raw mut arr;
        let borrowed_raw_slice = borrow_place!(@<&[i32]> *raw_slice);
        assert_eq!(borrowed_raw_slice, &[4, 50, 6]);

        let dbg_ref: &(dyn Debug + Sync) = &line.start.x;
        let borrowed_dbg = borrow_place!(@<&(dyn Debug + Sync)> *dbg_ref);
        assert_eq!(format!("{borrowed_dbg:?}"), "111");
    }

    assert_eq!(line.start.x, 111);
    assert_eq!(line.start.y, 444);
    assert_eq!(line.end.x, 321);
    assert_eq!(line.end.y, 888);
    assert_eq!(val1, 555);
    assert_eq!(val2, 666);
}
