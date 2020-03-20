use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use regex::Regex;
use syn::{parse, Ident, Item};

/// Get the identity of the item we have to implement `UriBound` for.
///
/// This function also checks that the item has no generics, since this macro isn't smart enough to
/// implement `UriBound` for all arguments of the generic type.
fn get_type_ident(item: TokenStream) -> Ident {
    const PARSING_ERROR: &str = "Only structs, enums, types, and unions may have a URI";

    let (ident, generics) = match parse::<Item>(item).expect(PARSING_ERROR) {
        Item::Enum(definition) => (definition.ident, definition.generics),
        Item::Struct(definition) => (definition.ident, definition.generics),
        Item::Type(definition) => (definition.ident, definition.generics),
        Item::Union(definition) => (definition.ident, definition.generics),
        _ => panic!(PARSING_ERROR),
    };

    if !generics.params.is_empty() {
        panic!("The uri attribute does not support generic types");
    }
    ident
}

/// Parse the attribute argument and create the URI literal from it.
///
/// This includes multiple checks to assure that the literal is formatted correctly.
fn get_uri(attr: TokenStream) -> Literal {
    lazy_static! {
        static ref GET_STRING_RE: Regex = Regex::new(r#"^"(.*)"$"#).unwrap();
    }

    const PARSING_ERROR: &str = "A URI has to be a string literal";

    if parse::<Literal>(attr.clone()).is_err() {
        panic!(PARSING_ERROR);
    }

    let attr = attr.to_string();
    let captures = GET_STRING_RE.captures(attr.as_str()).expect(PARSING_ERROR);
    let uri = captures.get(1).expect(PARSING_ERROR).as_str();
    if !uri.is_ascii() {
        panic!("A URI has to be an ASCII string");
    }

    let mut uri_vec: Vec<u8> = Vec::with_capacity(uri.len() + 1);
    uri_vec.extend(uri.as_bytes());
    uri_vec.push(0);

    Literal::byte_string(uri_vec.as_ref())
}

/// Implement `UriBound` for a given item.
pub fn impl_uri_bound(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let ident = get_type_ident(item.clone());
    let uri = get_uri(attr);

    let implementation: TokenStream = quote! {
        unsafe impl ::urid::UriBound for #ident {
            const URI: &'static [u8] = #uri;
        }
    }
    .into();
    item.extend(implementation);
    item
}
