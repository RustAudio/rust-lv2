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

        let field_name: Vec<_> = self.fields.iter().map(|f| f.identifier).collect();
        let field_type: Vec<_> = self.fields.iter().map(|f| f.port_type).collect();

        let index_start_name: Vec<_> = self
            .fields
            .iter()
            .map(|f| format_ident!("__INDEX_START_{}", f.identifier))
            .collect();
        let index_end_name: Vec<_> = self
            .fields
            .iter()
            .map(|f| format_ident!("__INDEX_END_{}", f.identifier))
            .collect();

        let index_start_value: Vec<_> = index_start_name
            .iter()
            .enumerate()
            .map(|(i, _)| {
                i.checked_sub(1)
                    .and_then(|prev_i| index_end_name.get(prev_i))
                    .map(|previous_name| quote! { #previous_name + 1 })
                    .unwrap_or_else(|| quote! { 0 })
            })
            .collect();

        (quote! {
        const _: () = {
            impl PortCollection for #struct_name {
                type Connections = #internal_cache_name;

                #[inline]
                unsafe fn from_connections(
                    connections: &<Self as PortCollection>::Connections,
                    sample_count: u32,
                ) -> Option<Self> {
                    Some(Self {
                        #(
                            #field_name: <#field_type as PortCollection>::from_connections(
                                &connections.#field_name,
                                sample_count,
                            )?
                        ),*
                    })
                }
            }

            #[allow(non_snake_case, non_camel_case_types)]
            struct #internal_cache_name {
                #(#field_name: <#field_type as PortCollection>::Connections),*
            }

            impl PortConnections for #internal_cache_name {
                const SIZE: usize = #(<#field_type as PortCollection>::Connections::SIZE)+*;
                fn new() -> Self {
                    Self {
                        #(#field_name: <#field_type as PortCollection>::Connections::new()),*
                    }
                }

                #[allow(non_upper_case_globals)]
                fn set_connection(&mut self, index: u32) -> Option<&mut *mut core::ffi::c_void> {
                    #(
                        const #index_start_name: u32 = #index_start_value;
                        const #index_end_name: u32 = #index_start_name
                            + <#field_type as PortCollection>::Connections::SIZE as u32
                            - 1;
                    )*

                    match index {
                        #(#index_start_name..=#index_end_name => {
                            self.#field_name.set_connection(index - #index_start_name)
                        })*
                        _ => None,
                    }
                }
            }
        };
                })
        .into()
    }
}

/// Implement `PortCollection` for a struct.
#[inline]
pub fn port_collection_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    PortCollectionStruct::from_derive_input(&input).make_derived_contents()
}
