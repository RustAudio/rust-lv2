use proc_macro::TokenStream;
use syn::export::Span;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident};
use syn::{DeriveInput, Generics, Lifetime};

struct FeatureCollectionField<'a> {
    identifier: &'a Ident,
}

impl<'a> FeatureCollectionField<'a> {
    fn from_input_field(input: &'a Field) -> Self {
        FeatureCollectionField {
            identifier: input.ident.as_ref().unwrap(),
        }
    }

    fn make_retrieval(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {#identifier: container.retrieve_feature()?,}
    }
}

struct FeatureCollectionStruct<'a> {
    struct_name: &'a Ident,
    generics: &'a Generics,
    fields: Vec<FeatureCollectionField<'a>>,
}

impl<'a> FeatureCollectionStruct<'a> {
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(FeatureCollectionField::from_input_field)
                .collect(),
            _ => panic!("Only structs can implement `FeatureCollection`"),
        };
        FeatureCollectionStruct {
            struct_name: &input.ident,
            fields,
            generics: &input.generics,
        }
    }

    fn make_implementation(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let generics = self.generics;
        let retrievals = self.fields.iter().map(|field| field.make_retrieval());
        // retrieve the first lifetime of the struct, or set it to `'static` if there is none.
        let lifetime = self
            .generics
            .lifetimes()
            .next()
            .map(|l| l.lifetime.clone())
            .unwrap_or_else(|| Lifetime::new("'static", Span::call_site()));

        (quote! {
            impl#generics FeatureCollection<#lifetime> for #struct_name#generics {
                fn from_container(
                    container: &mut FeatureContainer<#lifetime>
                ) -> Result<Self, MissingFeatureError> {
                    Ok(Self {
                        #(#retrievals)*
                    })
                }
            }
        })
        .into()
    }
}

pub fn feature_collection_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let list = FeatureCollectionStruct::from_derive_input(&input);
    list.make_implementation()
}
