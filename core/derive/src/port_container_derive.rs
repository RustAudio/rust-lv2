use proc_macro::TokenStream;
use syn::export::Span;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

/// A field in the struct we implement `PortContainer` for.
struct PortContainerField<'a> {
    identifier: &'a Ident,
    port_type: &'a Type,
}

impl<'a> PortContainerField<'a> {
    /// Create a `Self` instance from a field object.
    fn from_input_field(input: &'a Field) -> Self {
        PortContainerField {
            identifier: input.ident.as_ref().unwrap(),
            port_type: &input.ty,
        }
    }

    /// Create the field initialization line for the implementing struct.
    fn make_connection_from_raw(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type;
        quote! {
            #identifier: {
                let connection = <#port_type as ::lv2_core::port::PortHandle>::from_raw(connections.#identifier, sample_count);
                if let Some(connection) = connection {
                    connection
                } else {
                    return None;
                }
            },
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

/// Representation of a struct we implement `PortContainer` for.
///
/// The implementation creates a hidden, mirrored version of the implementing struct that contains  
/// the raw pointers for the port. Then, the ports object is created from the raw version.
struct PortContainerStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<PortContainerField<'a>>,
}

impl<'a> PortContainerStruct<'a> {
    /// Return an `Ident` for the internal module name.
    fn internal_mod_name(&self) -> Ident {
        Ident::new(
            &format!("__lv2_plugin_ports_derive_{}", self.struct_name),
            Span::call_site(),
        )
    }

    /// Construct a `Self` instance from a `DeriveInput`.
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Enum(_) | Data::Union(_) => panic!("Only structs can implement PortContainer"),
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(PortContainerField::from_input_field)
                .collect(),
        };
        PortContainerStruct {
            struct_name,
            fields,
        }
    }

    /// Implement `PortContainer` for the struct.
    fn make_derived_contents(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let internal_mod_name = self.internal_mod_name();

        let connections_from_raw = self
            .fields
            .iter()
            .map(PortContainerField::make_connection_from_raw);
        let raw_field_declarations = self
            .fields
            .iter()
            .map(PortContainerField::make_raw_field_declaration);
        let raw_field_inits = self
            .fields
            .iter()
            .map(PortContainerField::make_raw_field_initialization);
        let connect_matchers = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_connect_matcher(i as u32));

        (quote! {
            impl PortContainer for #struct_name {
                type Cache = #internal_mod_name::DerivedPortPointerCache;

                #[inline]
                unsafe fn from_connections(connections: &<Self as PortContainer>::Cache, sample_count: u32) -> Option<Self> {
                    Some(
                        Self {
                            #(#connections_from_raw)*
                        }
                    )
                }
            }

            #[doc(hidden)]
            #[allow(non_snake_case)]
            mod #internal_mod_name {
                pub struct DerivedPortPointerCache {
                    #(#raw_field_declarations)*
                }

                impl Default for DerivedPortPointerCache {
                    #[inline]
                    fn default() -> Self {
                        Self {
                            #(#raw_field_inits)*
                        }
                    }
                }

                impl ::lv2_core::port::PortPointerCache for DerivedPortPointerCache {
                    fn connect(&mut self, index: u32, pointer: *mut ::std::ffi::c_void) {
                        match index {
                            #(#connect_matchers)*
                            _ => ()
                        }
                    }
                }
            }
        }).into()
    }
}

/// Implement `PortContainer` for a struct.
#[inline]
pub fn port_container_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let list = PortContainerStruct::from_derive_input(&input);
    list.make_derived_contents()
}
