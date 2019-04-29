use proc_macro::TokenStream;
use syn::export::Span;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

struct Lv2PortField<'a> {
    identifier: &'a Ident,
    port_type: &'a Type,
}

impl<'a> Lv2PortField<'a> {
    fn from_input_field(input: &'a Field) -> Self {
        Self {
            identifier: input.ident.as_ref().unwrap(),
            port_type: &input.ty,
        }
    }

    fn make_connection_from_raw(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type; // Todo: check types implements PortHandle
        quote! {
            #identifier: <#port_type as ::lv2_core::plugin::PortHandle>::from_raw(connections.#identifier, sample_count),
        }
    }

    fn make_raw_field_declaration(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            pub #identifier: *mut (),
        }
    }

    fn make_raw_field_initialization(&self) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            #identifier: ::std::ptr::null_mut(),
        }
    }

    fn make_connect_matcher(&self, index: u32) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        quote! {
            #index => self.#identifier = pointer,
        }
    }
}

struct Lv2PortsFields<'a> {
    struct_name: &'a Ident,
    fields: Vec<Lv2PortField<'a>>,
}

impl<'a> Lv2PortsFields<'a> {
    fn internal_mod_name(&self) -> Ident {
        Ident::new(
            &format!("__lv2_plugin_ports_derive_{}", self.struct_name),
            Span::call_site(),
        )
    }

    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
            Data::Struct(DataStruct { fields, .. }) => {
                fields.iter().map(Lv2PortField::from_input_field).collect()
            }
        };
        Self {
            struct_name,
            fields,
        }
    }

    fn make_derived_contents(&self) -> TokenStream {
        let struct_name = self.struct_name;
        let internal_mod_name = self.internal_mod_name();

        let connections_from_raw = self
            .fields
            .iter()
            .map(Lv2PortField::make_connection_from_raw);
        let raw_field_declarations = self
            .fields
            .iter()
            .map(Lv2PortField::make_raw_field_declaration);
        let raw_field_inits = self
            .fields
            .iter()
            .map(Lv2PortField::make_raw_field_initialization);
        let connect_matchers = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_connect_matcher(i as u32));

        (quote! {
            impl Lv2Ports for #struct_name {
                type Connections = #internal_mod_name::DerivedPortsConnections;

                #[inline]
                fn from_connections(connections: &<Self as Lv2Ports>::Connections, sample_count: u32) -> Self {
                    unsafe {
                        Self {
                            #(#connections_from_raw)*
                        }
                    }
                }
            }

            #[doc(hidden)]
            #[allow(non_snake_case)]
            mod #internal_mod_name {
                pub struct DerivedPortsConnections {
                    #(#raw_field_declarations)*
                }

                impl Default for DerivedPortsConnections {
                    #[inline]
                    fn default() -> Self {
                        Self {
                            #(#raw_field_inits)*
                        }
                    }
                }

                impl ::lv2_core::plugin::PortsConnections for DerivedPortsConnections {
                    unsafe fn connect(&mut self, index: u32, pointer: *mut ()) {
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

#[inline]
pub fn lv2_ports_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let list = Lv2PortsFields::from_derive_input(&input);
    list.make_derived_contents()
}
