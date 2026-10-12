//@ revisions: current next
//@ ignore-compare-mode-next-solver (explicit revisions)
//@[next] compile-flags: -Znext-solver
//@ aux-build: anti_fundamental_negative_reasoning.rs

// The auxiliary crate's `Which` impls are only disjoint because downstream crates cannot implement
// `AntiFundamentalTrait` for its fundamental type. Check that every impl which would make them
// overlap is rejected here, so that `Which` cannot end up with two impls for the same type.

extern crate anti_fundamental_negative_reasoning;

use anti_fundamental_negative_reasoning::{AntiFundamentalTrait, FundamentalWrapper};

struct LocalType;
struct LocalGeneric<T>(T);

impl AntiFundamentalTrait for FundamentalWrapper<LocalType> {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

impl AntiFundamentalTrait for &FundamentalWrapper<LocalType> {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

impl AntiFundamentalTrait for &mut FundamentalWrapper<LocalType> {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

impl<T> AntiFundamentalTrait for FundamentalWrapper<LocalGeneric<T>> {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

impl AntiFundamentalTrait for FundamentalWrapper<Box<LocalType>> {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

trait Helper {
    type Assoc;
}
impl Helper for LocalType {
    type Assoc = &'static FundamentalWrapper<LocalGeneric<u8>>;
}
impl AntiFundamentalTrait for <LocalType as Helper>::Assoc {}
//~^ ERROR cannot implement `AntiFundamentalTrait` for the fundamental type

fn main() {}
