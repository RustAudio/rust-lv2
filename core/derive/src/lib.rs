//! Procedural macros for `lv2-core`.

use proc_macro::TokenStream;
use quote::quote;

/// Generate external symbols for LV2 plugins.
#[proc_macro]
pub fn lv2_descriptors(_input: TokenStream) -> TokenStream {
    (quote! {
        #[no_mangle]
        fn lv2_descriptor(index: u32) -> *const SysFoo {
            todo!()
        }
    })
    .into()
    // lv2_descriptors::lv2_descriptors_impl(input)
    //(quote! {}).into()
}
