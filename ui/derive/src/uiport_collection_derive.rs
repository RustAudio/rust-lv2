use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Field;
use syn::{parse_macro_input, Data, DataStruct, Ident, Type};

/// A field in the struct we implement `UIPortCollection` for.
struct UIPortCollectionField<'a> {
    identifier: &'a Ident,
    port_type: &'a Type,
}

impl<'a> UIPortCollectionField<'a> {
    /// Create a `Self` instance from a field object.
    fn from_input_field(input: &'a Field) -> Self {
        UIPortCollectionField {
            identifier: input.ident.as_ref().unwrap(),
            port_type: &input.ty,
        }
    }

    fn make_field_initialization(&self, index: u32) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type;
        match port_type {
            Type::Path(typepath)
                if typepath.path.segments.last().unwrap().ident == "UIControlPort" =>
            {
                quote! {
                    #identifier: #port_type::new(#index),
                }
            }
            Type::Path(typepath)
                if typepath.path.segments.last().unwrap().ident == "UIAtomPort" =>
            {
                quote! {
                    #identifier: #port_type::new(urid, #index),
                }
            }
            _ => {
                quote! {}
            }
        }
    }

    fn make_control_ports(&self, index: u32) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type;
        match port_type {
            Type::Path(typepath)
                if typepath.path.segments.last().unwrap().ident == "UIControlPort" =>
            {
                quote! {
                    #index => Some(&mut self.#identifier),
                }
            }
            _ => {
                quote! {}
            }
        }
    }

    fn make_atom_ports(&self, index: u32) -> impl ::quote::ToTokens {
        let identifier = self.identifier;
        let port_type = self.port_type;
        match port_type {
            Type::Path(typepath)
                if typepath.path.segments.last().unwrap().ident == "UIAtomPort" =>
            {
                quote! {
                    #index => Some(&mut self.#identifier),
                }
            }
            _ => {
                quote! {}
            }
        }
    }
}

/// Representation of a struct we implement `UIPortCollection` for.
///
/// The implementation creates a hidden, mirrored version of the implementing struct that contains
/// the raw pointers for the port. Then, the ports object is created from the raw version.
struct UIPortCollectionStruct<'a> {
    struct_name: &'a Ident,
    fields: Vec<UIPortCollectionField<'a>>,
}

impl<'a> UIPortCollectionStruct<'a> {
    /// Construct a `Self` instance from a `DeriveInput`.
    fn from_derive_input(input: &'a DeriveInput) -> Self {
        let struct_name = &input.ident;
        let fields = match &input.data {
            Data::Enum(_) | Data::Union(_) => panic!("Only structs can implement UIPortCollection"),
            Data::Struct(DataStruct { fields, .. }) => fields
                .iter()
                .map(UIPortCollectionField::from_input_field)
                .collect(),
        };
        UIPortCollectionStruct {
            struct_name,
            fields,
        }
    }

    /// Implement `UIPortCollection` for the struct.
    fn make_derived_contents(&self) -> TokenStream {
        let struct_name = self.struct_name;

        let uiports_initializers = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_field_initialization(i as u32));

        let ui_control_ports = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_control_ports(i as u32));

        let ui_atom_ports = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.make_atom_ports(i as u32));

        (quote! {
            impl #struct_name {
                fn new(urid: URID<EventTransfer>) -> Self {
                    Self {
                        #(#uiports_initializers)*
                    }
                }
            }

            impl UIPortCollection for #struct_name {
                fn map_control_port(&mut self, port_index: u32) -> Option<&mut UIControlPort> {
                    match port_index {
                        #(#ui_control_ports)*
                        _ => None
                    }
                }
                fn map_atom_port(&mut self, port_index: u32) -> Option<&mut UIAtomPort> {
                    match port_index {
                        #(#ui_atom_ports)*
                        _ => None
                    }
                }
            }
        })
        .into()
    }
}

/// Implement `UIPortCollection` for a struct.
#[inline]
pub fn uiport_collection_derive_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let list = UIPortCollectionStruct::from_derive_input(&input);
    list.make_derived_contents()
}
