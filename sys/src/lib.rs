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
#![allow(improper_ctypes)]


#[cfg_attr(target_os = "linux", path = "linux.rs")]
mod bindings;

pub use bindings::*;

impl From<u32> for LV2_State_Flags {
    fn from(flags: u32) -> Self {
        Self(flags as _)
    }
}

impl From<LV2_State_Flags> for u32 {
    fn from(flags: LV2_State_Flags) -> u32 {
        flags.0 as u32
    }
}
