use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

struct FeatureCollectionField<'a> {
    identifier: &'a Ident,
    feature_type: &'a Type,
}

impl<'a> FeatureCollectionField<'a> {
    fn from_input_field(input: &'a Field) -> Self {
        FeatureCollectionField {
            identifier: input.ident.as_ref().unwrap(),
            feature_type: &input.ty,
        }
    }

    fn make_retrieval(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let feature_type = self.feature_type;
        quote! {#identifier: container.retrieve_feature::<#feature_type>()?,}
    }
}

struct FeatureCollectionStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<FeatureCollectionField<'a>>,
}

impl<'a> FeatureCollectionStruct<'a> {
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(FeatureCollectionField::from_input_field)
                .collect(),
            _ => panic!("Only structs can implement `FeatureCollection`"),
        };
        FeatureCollectionStruct {
            struct_name,
            fields,
        }
    }

    fn make_implementation(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let retrievals = self
            .fields
            .iter()
            .map(|field| field.make_retrieval());
        (quote! {
            impl FeatureCollection for #struct_name {
                fn from_container(container: &mut FeatureContainer) -> Option<Self> {
                    Some(Self {
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
