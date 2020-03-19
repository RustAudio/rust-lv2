use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use regex::Regex;
use syn::{Ident, ItemStruct, ItemUnion, ItemEnum, ItemType, parse};

fn get_type_name(item: &TokenStream) -> Ident {
    if let Ok(struct_definition) = parse::<ItemStruct>(item.clone()) {
        struct_definition.ident
    } else if let Ok(enum_definition) = parse::<ItemEnum>(item.clone()) {
        enum_definition.ident
    } else if let Ok(type_definition) = parse::<ItemType>(item.clone()) {
        type_definition.ident
    } else if let Ok(union_definition) = parse::<ItemUnion>(item.clone()) {
        union_definition.ident
    } else {
        panic!();
    }
}

fn get_uri(attr: TokenStream) -> Vec<u8> {
    lazy_static! {
        static ref GET_STRING_RE: Regex = Regex::new(r#""(.*)""#).unwrap();
    }

    let uri = attr.to_string();
    let mut uri = GET_STRING_RE
        .captures(uri.as_str())
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .as_bytes()
        .to_vec();
    uri.push(0);
    uri
}

pub fn impl_uri_bound(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let type_name = get_type_name(&item);
    let uri = Literal::byte_string(get_uri(attr).as_ref());
    let implementation: TokenStream = quote! {
        unsafe impl ::urid::UriBound for #type_name {
            const URI: &'static [u8] = #uri;
        }
    }
    .into();
    item.extend(implementation);
    item
}
