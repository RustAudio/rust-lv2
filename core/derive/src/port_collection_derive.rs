use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

/// A field in the struct we implement `PortCollection` for.
struct PortCollectionField<'a> {
    identifier: &'a Ident,
    port_type: &'a Type,
}

impl<'a> PortCollectionField<'a> {
    /// Create a `Self` instance from a field object.
    fn from_input_field(input: &'a Field) -> Self {
        PortCollectionField {
            identifier: input.ident.as_ref().unwrap(),
            port_type: &input.ty,
        }
    }

    /// Create the field initialization line for the implementing struct.
    fn make_connection_from_raw(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type;
        quote! {
            #identifier: <#port_type as PortHandle>::from_raw(connections.#identifier, sample_count)?,
        }
    }

    /// Create the corresponding field declaration line for the raw pointer struct.
    fn make_raw_field_declaration(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            pub #identifier: *mut ::std::ffi::c_void,
        }
    }

    /// Create the corresponding field initialization line for the raw pointer struct.
    fn make_raw_field_initialization(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            #identifier: ::std::ptr::null_mut(),
        }
    }

    /// Create the connection matching arm for the raw pointer struct.
    fn make_connect_matcher(&self, index: u32) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            #index => self.#identifier = pointer,
        }
    }
}

/// Representation of a struct we implement `PortCollection` for.
///
/// The implementation creates a hidden, mirrored version of the implementing struct that contains  
/// the raw pointers for the port. Then, the ports object is created from the raw version.
struct PortCollectionStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<PortCollectionField<'a>>,
}

impl<'a> PortCollectionStruct<'a> {
    /// Return an `Ident` for the internal module name.
    fn internal_cache_name(&self) -> Ident {
        Ident::new(
            &format!(
                "__lv2_plugin_ports_derive_{}_PortPointerCache",
                self.struct_name
            ),
            Span::call_site(),
        )
    }

    /// Construct a `Self` instance from a `DeriveInput`.
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Enum(_) | Data::Union(_) => panic!("Only structs can implement PortCollection"),
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(PortCollectionField::from_input_field)
                .collect(),
        };
        PortCollectionStruct {
            struct_name,
            fields,
        }
    }

    /// Implement `PortCollection` for the struct.
    fn make_derived_contents(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let internal_cache_name = self.internal_cache_name();

        let connections_from_raw = self
            .fields
            .iter()
            .map(PortCollectionField::make_connection_from_raw);
        let raw_field_declarations = self
            .fields
            .iter()
            .map(PortCollectionField::make_raw_field_declaration);
        let raw_field_inits = self
            .fields
            .iter()
            .map(PortCollectionField::make_raw_field_initialization);
        let connect_matchers = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_connect_matcher(i as u32));

        (quote! {
            impl PortCollection for #struct_name {
                type Cache = #internal_cache_name;

                #[inline]
                unsafe fn from_connections(connections: &<Self as PortCollection>::Cache, sample_count: u32) -> Option<Self> {
                    Some(
                        Self {
                            #(#connections_from_raw)*
                        }
                    )
                }
            }

            #[doc(hidden)]
            #[allow(non_snake_case, non_camel_case_types)]
            pub struct #internal_cache_name {
                #(#raw_field_declarations)*
            }

            impl Default for #internal_cache_name {
                #[inline]
                fn default() -> Self {
                    Self {
                        #(#raw_field_inits)*
                    }
                }
            }

            impl PortPointerCache for #internal_cache_name {
                fn connect(&mut self, index: u32, pointer: *mut ::std::ffi::c_void) {
                    match index {
                        #(#connect_matchers)*
                        _ => ()
                    }
                }
            }
        }).into()
    }
}

/// Implement `PortCollection` for a struct.
#[inline]
pub fn port_collection_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    PortCollectionStruct::from_derive_input(&input).make_derived_contents()
}
