use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::{parse_macro_input, Data, DataStruct, Ident};

/// Representation of the cache struct we implement `URIDCache` for.
struct CacheStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<&'a Ident>,
}

impl<'a> CacheStruct<'a> {
    /// Construct a cache representation from the derive input.
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect(),
            _ => panic!("Only structs can implement `FeatureCollection`"),
        };
        Self {
            struct_name,
            fields,
        }
    }

    /// Construct the implementation fo `URIDCache` for our cache.
    fn make_implementation(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let initializations = self
            .fields
            .iter()
            .map(|ident| quote! {#ident: map.populate_cache()?,});
        let implementation = quote! {
            impl ::lv2_urid::URIDCache for #struct_name {
                fn from_map(map: &::lv2_urid::Map) -> Option<Self> {
                    Some(Self {
                        #(#initializations)*
                    })
                }
            }
        };
        implementation.into()
    }
}

pub fn urid_cache_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let cache_struct = CacheStruct::from_derive_input(&input);
    cache_struct.make_implementation()
}
