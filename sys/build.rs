use std::error::Error;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

#[cfg(not(feature = "bindgen"))]
fn try_main() -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let mut data_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    data_path.push("build_data");

    let target = env::var("TARGET").unwrap();
    let valid_targets = fs::read_to_string(data_path.join("valid_targets.txt"))
        .expect("can't find \"valid_targets.txt\"");
    if !valid_targets.contains(&target) {
        let s = format!("No valid prebinding for {}. ", target)
            + "Add \"lv2_sys\" with \"bindgen\" feature in your dependencies.";
        return Err(s.into()) ;
    }

    let mut out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_path.push("bindings.rs");
    fs::copy(data_path.join("bindings.rs"), out_path).unwrap();
    Ok(())
}


#[cfg(feature = "bindgen")]
fn try_main() -> Result<(), Box<dyn Error>> {
    extern crate lv2_sys_bindgen;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let mut source_dir = PathBuf::new();
    source_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    source_dir.push("lv2");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    lv2_sys_bindgen::generate_bindings(&source_dir, &out_dir);

    Ok(())
}
