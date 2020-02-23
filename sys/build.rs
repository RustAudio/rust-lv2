extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Standard settings.
    let mut bindings = bindgen::Builder::default()
        .rustfmt_bindings(true)
        .size_t_is_usize(true);
    
    // Retrieve the root directory of the crate.
    let mut root_dir = PathBuf::new();
    root_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Adding the crate to the include path of clang.
    // Otherwise, included headers can not be found.
    bindings = bindings.clang_arg(format!("-I{}", root_dir.to_str().unwrap()));

    // Iterating through the spec directory.
    root_dir.push("lv2");
    for entry in fs::read_dir(root_dir.clone()).unwrap() {
        if let Ok(entry) = entry {
            let mut spec_dir = root_dir.clone();
            spec_dir.push(entry.path());
            // Iterating through every file of every specification and adding
            // C headers to the bindings.
            for entry in fs::read_dir(spec_dir).unwrap() {
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
        }
    }

    // Generating the bindings.
    let bindings = bindings.generate().expect("Unable to generate bindings");

    // Writing the bindings to a file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
