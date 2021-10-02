//! Procedural macros for `options`.
#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Ident};

#[proc_macro_derive(OptionsCollection)]
pub fn options_collection_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let struct_name = input.ident;
    let serializer_name = Ident::new(&format!("__{}_Serializer", struct_name), struct_name.span());

    let fields = match input.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Only structs can implement `OptionsCollection`"),
    };

    let field_inits = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .map(|ident| quote! {#ident: map.populate_collection()?,});

    let serializer_fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        quote! {
            #ident: <#ty as __lv2_options::collection::OptionsCollection>::Serializer,
        }
    });

    let fields_deserialize_new = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote! {
            #ident: self.#ident.deserialize_new(options)?,
        }
    });

    let fields_deserialize_to = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote! {
            self.#ident.deserialize_to(&mut destination.#ident, options)?;
        }
    });

    let fields_respond_to_request = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote! {
            match self.#ident.respond_to_request(&options.#ident, request) {
                Err(__lv2_options::OptionsError::BadKey) => {}
                r => return r,
            };
        }
    });

    let implementation = quote! {
        const _: () = {
            extern crate urid as __urid;
            extern crate lv2_options as __lv2_options;

            #[allow(non_camel_case_types)]
            pub struct #serializer_name {
                #(#serializer_fields)*
            }

            impl lv2_options::collection::OptionsSerializationContext<#struct_name>
                for #serializer_name
            {
                fn deserialize_new(
                    &self,
                    options: &__lv2_options::list::OptionsList,
                ) -> Result<#struct_name, __lv2_options::OptionsError> {
                    Ok(#struct_name {
                        #(#fields_deserialize_new)*
                    })
                }

                fn deserialize_to(
                    &self,
                    destination: &mut #struct_name,
                    options: &__lv2_options::list::OptionsList,
                ) -> Result<(), __lv2_options::OptionsError> {
                    #(#fields_deserialize_to)*
                    Ok(())
                }

                fn respond_to_request<'a>(
                    &self,
                    options: &'a #struct_name,
                    request: &mut __lv2_options::prelude::OptionRequest<'a>,
                ) -> Result<(), __lv2_options::OptionsError> {
                    #(#fields_respond_to_request)*

                    Err(__lv2_options::OptionsError::BadKey)
                }
            }

            impl __lv2_options::collection::OptionsCollection for #struct_name {
                type Serializer = #serializer_name;
            }

            impl __urid::URIDCollection for #serializer_name  {
                fn from_map<M: __urid::Map + ?Sized>(map: &M) -> Option<Self> {
                    Some(Self {
                        #(#field_inits)*
                    })
                }
            }
        };
    };

    implementation.into()
}
