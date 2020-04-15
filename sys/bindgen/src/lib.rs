use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Generate lv2-sys bindings
pub fn generate_bindings(source_dir: &Path, out_dir: &Path, target:Option<&str>) {
    let mut bindings = bindgen::Builder::default().size_t_is_usize(true);

    // Adding the crate to the include path of clang.
    // Otherwise, included headers can not be found.
    let mut include_path = PathBuf::from(source_dir);
    include_path.pop();
    bindings = bindings.clang_arg(format!("-I{}", include_path.to_str().unwrap()));
    if let Some(target) = target{
        bindings = bindings.clang_arg(format!("--target={}", target));
    }

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
        .layout_tests(false)
        .bitfield_enum("LV2_State_Flags");

    // Generating the bindings.
    let bindings = bindings.generate().expect("Unable to generate bindings");

    // Writing the bindings to a file.
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
