//! Raw bindings of all LV2 specification headers.
//!
//! Bindings to the official [LV2](https://lv2plug.in/) API headers, used by [`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust. The crate uses the version 1.18.0 of the specification, as pulled from the [project's website](https://lv2plug.in/lv2-1-18-0.html).
//!
//! ## Building
//! 
//! Since the bindings to the raw C headers are generated with bindgen, you need to have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't in your system's standard path, set the environment variable `LIBCLANG_PATH` to the path of `libClang`.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]
#![allow(improper_ctypes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(windows)]
impl From<u32> for LV2_State_Flags {
    fn from(flags: u32) -> Self {
        Self(flags as i32)
    }
}

#[cfg(not(windows))]
impl From<u32> for LV2_State_Flags {
    fn from(flags: u32) -> Self {
        Self(flags)
    }
}

#[cfg(windows)]
impl From<LV2_State_Flags> for u32 {
    fn from(flags: LV2_State_Flags) -> u32 {
        flags.0 as u32
    }
}

#[cfg(not(windows))]
impl From<LV2_State_Flags> for u32 {
    fn from(flags: LV2_State_Flags) -> u32 {
        flags.0
    }
}
