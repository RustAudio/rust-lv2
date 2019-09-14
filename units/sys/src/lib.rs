//! Binding of the C API for the [units specification of LV2](http://lv2plug.in/ns/extensions/units/units.html).
//!
//! Since this crate usese `bindgen` to create the C API bindings, you need to have clang installed on your machine.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::all)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;
