//! Raw bindings of all LV2 specification headers.
//!
//! This crate contains all C headers of the LV2 specification. Please note that utility headers are not included. If you want to use utilities, you should use the "nice" LV2 crates or create your own.
//!
//! The bindings are generated at build time using [bindgen](https://crates.io/crates/bindgen), which requires clang to be installed. The installation process is described [here](https://rust-lang.github.io/rust-bindgen/requirements.html).
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
