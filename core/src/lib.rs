//! Implementation of the core LV2 specification
//!
//! This crate forms the foundation of the LV2 experience for Rust: It contains the plugin trait and ports, as well as means to retrieve features from the host and to extend the interface of the plugin.
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_sys as sys;

pub mod extension;
pub mod feature;
pub mod plugin;
pub mod port;
pub mod prelude;
