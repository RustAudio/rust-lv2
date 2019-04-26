#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod lv2_descriptors;
mod lv2_ports_derive;

use proc_macro::TokenStream;

#[proc_macro]
pub fn lv2_descriptors(input: TokenStream) -> TokenStream {
    lv2_descriptors::lv2_descriptors_impl(input)
}

#[proc_macro_derive(Lv2Ports)]
pub fn lv2_ports_derive(input: TokenStream) -> TokenStream {
    lv2_ports_derive::lv2_ports_derive_impl(input)
}