use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

/// Representation of a field in the cache.
struct CacheField<'a> {
    identifier: &'a Ident,
    subcache_type: &'a Type,
}

impl<'a> CacheField<'a> {
    /// Construct a field representation.
    fn from_input_field(input: &'a Field) -> Self {
        Self {
            identifier: input.ident.as_ref().unwrap(),
            subcache_type: &input.ty,
        }
    }

    /// Construct the initialization of the cache field.
    fn make_initialization(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let subcache_type = self.subcache_type;
        quote! {#identifier: map.populate_cache::<#subcache_type>()?,}
    }
}

/// Representation of the cache struct we implement `URIDCache` for.
struct CacheStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<CacheField<'a>>,
}

impl<'a> CacheStruct<'a> {
    /// Construct a cache representation from the derive input.
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => {
                fields.iter().map(CacheField::from_input_field).collect()
            }
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
        let initializations = self.fields.iter().map(|field| field.make_initialization());
        (quote! {
            impl ::lv2_urid::URIDCache for #struct_name {
                fn from_map(map: &::lv2_urid::Map) -> Option<Self> {
                    Some(Self {
                        #(#initializations)*
                    })
                }
            }
        })
        .into()
    }
}

pub fn urid_cache_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let cache_struct = CacheStruct::from_derive_input(&input);
    cache_struct.make_implementation()
}
