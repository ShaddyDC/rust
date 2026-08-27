//@ revisions: old next
//@ [next] compile-flags: -Znext-solver
//@ run-pass
#![expect(incomplete_features)]
#![feature(field_projections)]

use std::field::field_of;
use std::ops::place::{DerefPlace, LocalHandle, MutHandle, ProjectPlace, RefHandle, raw_handle};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Line {
    start: Point,
    end: Point,
}

fn main() {
    let mut line = Line {
        start: Point { x: 10, y: 20 },
        end: Point { x: 30, y: 40 },
    };

    // 1. Direct place handles with compiler-inferred types (no fat arrow annotations!)
    let h_root: LocalHandle<Line> = raw_handle!(line);
    let h_start: LocalHandle<Point> = raw_handle!(line.start);
    let h_start_y: LocalHandle<i32> = raw_handle!(line.start.y);

    unsafe {
        assert_eq!(
            *h_root.as_ptr(),
            Line { start: Point { x: 10, y: 20 }, end: Point { x: 30, y: 40 } }
        );
        assert_eq!(*h_start.as_ptr(), Point { x: 10, y: 20 });
        assert_eq!(*h_start_y.as_ptr(), 20);

        *h_start_y.as_mut_ptr() = 999;
    }
    assert_eq!(line.start.y, 999);

    // 2. Explicit mutability qualifiers
    let h_mut = raw_handle!(mut line.end.x);
    unsafe {
        assert_eq!(*h_mut.as_ptr(), 30);
        *h_mut.as_mut_ptr() = 777;
    }
    assert_eq!(line.end.x, 777);

    let h_const = raw_handle!(const line.end.y);
    assert_eq!(unsafe { *h_const.as_ptr() }, 40);

    // 3. Interoperability with `ProjectPlace` and `field_of!`
    let start_subplace: field_of!(Line, start) = Default::default();
    let projected_start = unsafe { h_root.project_place(start_subplace) };
    let y_subplace: field_of!(Point, y) = Default::default();
    let projected_y = unsafe { projected_start.project_place(y_subplace) };
    assert_eq!(unsafe { *projected_y.as_ptr() }, 999);

    // 4. Interoperability with `DerefPlace` returning `MutHandle`
    let line_ref = &mut line;
    let ref_h = raw_handle!(line_ref);
    let deref_h: MutHandle<'_, Line> = unsafe { DerefPlace::deref_place(ref_h) };
    let end_subplace: field_of!(Line, end) = Default::default();
    let projected_end: MutHandle<'_, Point> = unsafe { deref_h.project_place(end_subplace) };
    let x_subplace: field_of!(Point, x) = Default::default();
    let mut projected_end_x: MutHandle<'_, i32> =
        unsafe { projected_end.project_place(x_subplace) };
    assert_eq!(unsafe { *projected_end_x.as_ptr() }, 777);
    unsafe {
        *projected_end_x.as_mut() = 888;
    }
    assert_eq!(line.end.x, 888);

    // 5. Interoperability with `DerefPlace` returning `RefHandle` (read-only)
    let shared_ref = &line;
    let shared_h = raw_handle!(shared_ref);
    let deref_shared: RefHandle<'_, Line> = unsafe { DerefPlace::deref_place(shared_h) };
    let projected_shared_end: RefHandle<'_, Point> =
        unsafe { deref_shared.project_place(end_subplace) };
    assert_eq!(unsafe { *projected_shared_end.as_ptr() }, Point { x: 888, y: 40 });
    assert_eq!(unsafe { deref_shared.as_ref().start.y }, 999);
}
