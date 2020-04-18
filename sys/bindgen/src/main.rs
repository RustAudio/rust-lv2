extern crate bindgen;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Generate lv2-sys bindings and write them to out.
pub fn generate_bindings(source_dir: &Path, out: &Path) {
    let mut bindings = bindgen::Builder::default()
        .size_t_is_usize(true)
        .whitelist_type("LV2.*")
        .whitelist_function("LV2.*")
        .whitelist_var("LV2.*")
        .layout_tests(false)
        .bitfield_enum("LV2_State_Flags");

    // Adding the headers to the include path of clang.
    // Otherwise, included headers can not be found.
    let mut include_path = PathBuf::from(source_dir);
    include_path.pop();
    bindings = bindings.clang_arg(format!("-I{}", include_path.to_str().unwrap()));

    // Iterate over every folder and header file in the source dir and add them to the bindings.
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

    // Generating the bindings.
    let bindings = bindings.generate().expect("Unable to generate bindings");

    // Writing the bindings to the file.
    bindings
        .write_to_file(out)
        .expect("Couldn't write bindings!");
}

fn main() {
    let matches = clap::App::new("lv2-sys-bindgen")
        .author("© 2020 Amaury 'Yruama_Lairba' Abrail, Jan-Oliver 'Janonard' Opdenhövel")
        .about("Generate Rust bindings of the LV2 C API")
        .version("0.1.0")
        .arg(
            clap::Arg::with_name("LV2")
                .help("The path to the LV2 C API")
                .required(true)
                .short("I")
                .long("lv2")
                .value_name("DIR")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("out")
                .help("The file to write the bindings to")
                .required(true)
                .short("o")
                .long("out")
                .value_name("OUT")
                .takes_value(true),
        )
        .get_matches();

    let headers = PathBuf::from(".").join(matches.value_of("LV2").unwrap());
    let out = PathBuf::from(".").join(matches.value_of("out").unwrap());

    generate_bindings(&headers, &out);
}
