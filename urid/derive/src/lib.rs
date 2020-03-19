//! Procedural macros for `urid`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod uri_bound;
mod urid_collection_derive;

use proc_macro::TokenStream;

#[proc_macro_derive(URIDCollection)]
pub fn urid_collection_derive(input: TokenStream) -> TokenStream {
    urid_collection_derive::urid_collection_derive_impl(input)
}

#[proc_macro_attribute]
pub fn uri_bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    uri_bound::impl_uri_bound(attr, item)
}
