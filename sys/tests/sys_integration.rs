extern crate lv2_sys_bindgen;
use quote::quote;
use proc_macro2::Span;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use syn::{Field, Fields, FieldsUnnamed, Ident, Item, ItemStruct, ItemType, Path, Type, TypePath};

use std::env;

// this function allow to ignore some i32/u32 difference
fn i32_to_u32(mut item: Item) -> Item {
    match &mut item {
        Item::Type(ItemType { ty, .. }) => {
            if let Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            }) = ty.as_mut()
            {
                for e in segments.iter_mut() {
                    if format!("{}", e.ident) == "i32" {
                        e.ident = Ident::new("u32", Span::call_site());
                    }
                }
            }
        }
        Item::Struct(ItemStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => {
            for u in unnamed {
                if let Field {
                    ty:
                        Type::Path(TypePath {
                            path: Path { segments, .. },
                            ..
                        }),
                    ..
                } = u
                {
                    for e in segments.iter_mut() {
                        if format!("{}", e.ident) == "i32" {
                            e.ident = Ident::new("u32", Span::call_site());
                        }
                    }
                }
            }
        }
        _ => (),
    }
    item
}

// the idea here is to represent a binding file as a unordonned collection of syn::Item. This to be
// insensitive to formatting or definition order in the file.
#[test]
fn bindings_are_equivalent() {

    let work_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = work_dir.join("lv2");
    let bindings1_dir = work_dir.join("build_data");
    let bindings2_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //println!("{}", bindings2_dir.to_str().unwrap());
    lv2_sys_bindgen::generate_bindings(&source_dir, &bindings2_dir, None);
    
    let f1 = fs::read_to_string(bindings1_dir.join("bindings.rs")).unwrap();
    let f1 = syn::parse_str::<syn::File>(&f1).unwrap();
    let h1: HashSet<_> = f1.items.into_iter().map(i32_to_u32).collect();

    let f2 = fs::read_to_string(bindings2_dir.join("bindings.rs")).unwrap();
    let f2 = syn::parse_str::<syn::File>(&f2).unwrap();
    let h2: HashSet<_> = f2.items.into_iter().map(i32_to_u32).collect();

    if h1 != h2 {
        let diff1: HashSet<_> = h1.difference(&h2).collect();
        let diff2: HashSet<_> = h2.difference(&h1).collect();
        let mut message = String::from("Error, binding aren't equivalent\n");
        if !diff1.is_empty() {
            message.push_str("Item present only in static bindings:\n");
            for e in diff1 {
                message.push_str(&format!("{}\n", quote!(#e)));
            }
        }
        if !diff2.is_empty() {
            message.push_str("Item present only in generated bindings:\n");
            for e in diff2 {
                message.push_str(&format!("{}\n", quote!(#e)));
            }
        }
        panic!(message);
    }
}
