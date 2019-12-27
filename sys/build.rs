extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut source_dir = PathBuf::new();
    source_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    source_dir.push("src");
    let mut bindings = bindgen::Builder::default().rustfmt_bindings(true);

    for entry in fs::read_dir(source_dir).unwrap() {
        let entry = match entry {
            Ok(entry) => entry.path(),
            _ => continue,
        };

        let extension = match entry.extension() {
            Some(extension) => extension,
            None => continue,
        };

        if extension == "h" {
            bindings = bindings.header(entry.to_str().unwrap());
        }
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
