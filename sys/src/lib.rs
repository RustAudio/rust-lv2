//! Raw bindings of all LV2 specification headers.
//!
//! Bindings to the official [LV2](https://lv2plug.in/) API headers, used by [`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust. The crate uses the version 1.18.0 of the specification, as pulled from the [project's website](https://lv2plug.in/lv2-1-18-0.html).
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(
    all(target_os = "windows", feature = "experimental-targets"),
    path = "windows.rs"
)]
mod unsupported;
pub use unsupported::*;

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
