extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut bindings = bindgen::Builder::default().size_t_is_usize(true);

    let mut source_dir = PathBuf::new();
    source_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Adding the crate to the include path of clang.
    // Otherwise, included headers can not be found.
    bindings = bindings.clang_arg(format!("-I{}", source_dir.to_str().unwrap()));

    source_dir.push("lv2");

    for entry in fs::read_dir(source_dir).unwrap() {
        let spec_dir = if let Ok(spec_dir) = entry {
            spec_dir.path()
        } else {
            continue;
        };

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
    bindings = bindings
        .whitelist_type("LV2.*")
        .whitelist_function("LV2.*")
        .whitelist_var("LV2.*")
        .bitfield_enum("LV2_State_Flags");

    // Generating the bindings.
    let bindings = bindings.generate().expect("Unable to generate bindings");

    // Writing the bindings to a file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
