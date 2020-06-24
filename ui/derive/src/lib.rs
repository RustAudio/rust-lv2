//! Procedural macros for `lv2-ui`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod lv2ui_descriptors;

use proc_macro::TokenStream;

/// Generate external symbols for LV2 plugin UIs.
#[proc_macro]
pub fn lv2ui_descriptors(input: TokenStream) -> TokenStream {
    lv2ui_descriptors::lv2_ui_descriptors_impl(input)
}
