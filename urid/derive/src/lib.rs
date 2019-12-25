//! Procedural macros for `lv2-urid`.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod urid_cache_derive;

use proc_macro::TokenStream;

use syn::export::Span;
use syn::{Ident, Path};

pub(crate) fn lib_name() -> Path {
    match proc_macro_crate::crate_name("lv2") {
        Ok(name) => {
            // The `lv2` crate is used. The core crate is found in `lv2::core`.
            let mut p: Path = Ident::new(&name, Span::call_site()).into();
            p.segments
                .push(Ident::new("urid", Span::call_site()).into());
            p
        }
        Err(_) => match proc_macro_crate::crate_name("lv2_urid") {
            // The `lv2_core` crate is used directly.
            Ok(name) => Ident::new(&name, Span::call_site()).into(),
            // None of the crates is used. Therefore, we need to fall back to `lv2_urid`.
            // If the macro is called from `lv2_urid` itself, `crate` will be aliased to `lv2_urid`.
            Err(_) => Ident::new("lv2_urid", Span::call_site()).into(),
        },
    }
}

#[proc_macro_derive(URIDCache)]
pub fn urid_cache_derive(input: TokenStream) -> TokenStream {
    urid_cache_derive::urid_cache_derive_impl(input)
}
