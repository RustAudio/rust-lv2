//! Procedural macros for `lv2-core`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod lv2_descriptors;
mod lv2_ports_derive;

use proc_macro::TokenStream;

/// Generate external symbols for LV2 plugins.
#[proc_macro]
pub fn lv2_descriptors(input: TokenStream) -> TokenStream {
    lv2_descriptors::lv2_descriptors_impl(input)
}

/// Implement the `Lv2Ports` trait for a port struct.
#[proc_macro_derive(Lv2Ports)]
pub fn lv2_ports_derive(input: TokenStream) -> TokenStream {
    lv2_ports_derive::lv2_ports_derive_impl(input)
}
