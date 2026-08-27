#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]

pub use place_macros_impl::{
    desugar_place, desugar_raw_handle, desugar_type_of, raw_handle, type_of,
};
