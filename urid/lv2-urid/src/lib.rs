//! LV2 integration of the URID concept.
//!
//! The URID specification provides a host feature that can be used by plugins to map URIs to integers, so-called URIDs. These URIDs are used by many other specifications to identify other URI bounds and combine the flexibility of URIs with the comparison speed of integers.
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;

mod feature;
mod mapper;

pub use feature::*;
pub use mapper::*;
