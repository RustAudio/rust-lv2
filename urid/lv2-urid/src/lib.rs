//! LV2 integration of the URID concept.
//!
//! The URID specification provides a host feature that can be used by plugins to map URIs to integers, so-called URIDs. These URIDs are used by many other specifications to identify other URI bounds and combine the flexibility of URIs with the comparison speed of integers.

#[cfg(feature = "lv2-core")]
extern crate lv2_core as core;
extern crate lv2_sys as sys;

#[cfg(feature = "lv2-core")]
mod feature;
mod mapper;

#[cfg(feature = "lv2-core")]
pub use feature::*;
pub use mapper::*;
