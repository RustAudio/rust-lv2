//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_core as core;
pub extern crate lv2_urid_sys as sys;

pub mod feature;
mod urid;

pub use urid::*;
