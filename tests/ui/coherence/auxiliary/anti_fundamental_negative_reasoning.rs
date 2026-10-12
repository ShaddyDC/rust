// Auxiliary crate whose coherence relies on downstream crates being unable to implement an
// anti-fundamental trait for a fundamental type defined here.

#![feature(fundamental)]
#![feature(rustc_attrs)]

#[fundamental]
pub struct FundamentalWrapper<T>(pub T);

#[rustc_anti_fundamental]
pub trait AntiFundamentalTrait {}

pub trait Which {
    const WHICH: u8;
}

// These only avoid overlapping because no downstream crate can implement
// `AntiFundamentalTrait` for `FundamentalWrapper<_>`, `&FundamentalWrapper<_>`
// or `&mut FundamentalWrapper<_>`.
impl<T: AntiFundamentalTrait> Which for T {
    const WHICH: u8 = 0;
}
impl<T> Which for FundamentalWrapper<T> {
    const WHICH: u8 = 1;
}
impl<'a, T> Which for &'a FundamentalWrapper<T> {
    const WHICH: u8 = 2;
}
impl<'a, T> Which for &'a mut FundamentalWrapper<T> {
    const WHICH: u8 = 3;
}
