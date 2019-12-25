//! Procedural macros for `lv2-core`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod feature_collection_derive;
mod lv2_descriptors;
mod port_container_derive;

use proc_macro::TokenStream;

use syn::export::Span;
use syn::{Ident, Path};

pub(crate) fn lib_name() -> Path {
    match proc_macro_crate::crate_name("lv2") {
        Ok(name) => {
            // The `lv2` crate is used. The core crate is found in `lv2::core`.
            let mut p: Path = Ident::new(&name, Span::call_site()).into();
            p.segments
                .push(Ident::new("core", Span::call_site()).into());
            p
        }
        Err(_) => match proc_macro_crate::crate_name("lv2_core") {
            // The `lv2_core` crate is used directly.
            Ok(name) => Ident::new(&name, Span::call_site()).into(),
            // None of the crates is used. Therefore, we need to fall back to `lv2_core`.
            // If the macro is called from `lv2_core` itself, `crate` will be aliased to `lv2_core`.
            Err(_) => Ident::new("lv2_core", Span::call_site()).into(),
        },
    }
}

/// Generate external symbols for LV2 plugins.
#[proc_macro]
pub fn lv2_descriptors(input: TokenStream) -> TokenStream {
    lv2_descriptors::lv2_descriptors_impl(input)
}

/// Implement the `PortContainer` trait for a port struct.
#[proc_macro_derive(PortContainer)]
pub fn port_container_derive(input: TokenStream) -> TokenStream {
    port_container_derive::port_container_derive_impl(input)
}

#[proc_macro_derive(FeatureCollection)]
pub fn feature_collection_derive(input: TokenStream) -> TokenStream {
    feature_collection_derive::feature_collection_derive_impl(input)
}
