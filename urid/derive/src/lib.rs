//! Procedural macros for `lv2-urid`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod urid_cache_derive;

use proc_macro::TokenStream;

#[proc_macro_derive(URIDCache)]
pub fn urid_cache_derive(input: TokenStream) -> TokenStream {
    urid_cache_derive::urid_cache_derive_impl(input)
}
