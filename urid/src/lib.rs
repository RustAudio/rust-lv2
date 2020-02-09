//! LV2 specification centered around the Map feature.
//!
//! The URID specification provides a host feature that can be used by plugins to map URIs to integers, so-called URIDs. These URIDs are used by many other specifications to identify other URI bounds and combine the flexibility of URIs with the comparison speed of integers.
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_core as core;
extern crate lv2_sys as sys;

#[cfg(feature = "host")]
pub mod mapper;

mod feature;
mod urid;

pub use lv2_urid_derive::*;

pub use feature::*;
pub use urid::*;

/// Prelude of `lv2_urid` for wildcard usage.
pub mod prelude {
    pub use crate::feature::{Map, Unmap};
    pub use crate::{URIDCache, URID};
    pub use lv2_urid_derive::*;
}
