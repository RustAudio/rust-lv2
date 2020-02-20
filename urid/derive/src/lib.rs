//! Procedural macros for `lv2-urid`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod urid_collection_derive;

use proc_macro::TokenStream;

#[proc_macro_derive(URIDCollection)]
pub fn urid_collection_derive(input: TokenStream) -> TokenStream {
    urid_collection_derive::urid_collection_derive_impl(input)
}
