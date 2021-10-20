//! Procedural macros for `lv2-core`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

mod feature_collection_derive;
mod lv2_descriptors;
mod port_collection_derive;

use proc_macro::TokenStream;

/// Generate external symbols for LV2 plugins.
#[proc_macro]
pub fn lv2_descriptors(input: TokenStream) -> TokenStream {
    lv2_descriptors::lv2_descriptors_impl(input)
}

/// Implement the `PortCollection` trait for a port struct.
#[proc_macro_derive(PortCollection)]
pub fn port_collection_derive(input: TokenStream) -> TokenStream {
    port_collection_derive::port_collection_derive_impl(input)
}

#[proc_macro_derive(FeatureCollection)]
pub fn feature_collection_derive(input: TokenStream) -> TokenStream {
    feature_collection_derive::feature_collection_derive_impl(input)
}
