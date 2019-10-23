use crate::lib_name;
use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::{parse_macro_input, Data, DataStruct};

pub fn urid_cache_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let crate_name = lib_name();

    let struct_name = input.ident;

    let fields = match input.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Only structs can implement `URIDCache`"),
    };

    let field_inits = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .map(|ident| quote! {#ident: map.populate_cache()?,});

    let implementation = quote! {
        impl #crate_name::URIDCache for #struct_name {
            fn from_map(map: &#crate_name::Map) -> Option<Self> {
                Some(Self {
                    #(#field_inits)*
                })
            }
        }
    };

    implementation.into()
}
