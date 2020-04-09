extern crate bindgen;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    generate_bindings();
    test_compatible_target();
}

fn generate_bindings() {
    let mut bindings = bindgen::Builder::default().size_t_is_usize(true);

    let mut work_dir = PathBuf::new();
    work_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    work_dir.pop();

    let source_dir = work_dir.join("lv2");
    let out_path = work_dir.join("build_data");

    // Adding the crate to the include path of clang.
    // Otherwise, included headers can not be found.
    bindings = bindings.clang_arg(format!("-I{}", work_dir.to_str().unwrap()));

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
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

use std::io::Write;
use std::thread;

fn test_compatible_target() {
    let mut test_h = PathBuf::new();
    test_h.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    test_h.push("enum_test.h");
    let test_h = test_h.to_str().unwrap();
    let output = Command::new("rustc")
        .args(&["--print", "target-list"])
        .output()
        .expect("failed to execute rustc --print target-list");

    if !output.status.success() {
        panic!("'rustc --print target-list' returned an error");
    }
    let targets = std::str::from_utf8(&output.stdout)
        .unwrap()
        .split_whitespace();
    let mut valid_targets = Vec::new();
    for target in targets {
        let target2 = String::from(target);
        let test_h = String::from(test_h);
        print!("{}: ", target);
        //the thread spawning avoid to exit when bindgen panics
        let handle = thread::spawn(move || {
            let builder = bindgen::Builder::default()
                .size_t_is_usize(true)
                .clang_arg(format!("--target={}", target2))
                .header(test_h);
            // make silent panic
            std::panic::set_hook(Box::new(|_| ()));
            let bindings = builder
                .generate()
                .expect("failed to generate a test binding");
            //restore default panic hook because it's not thread local
            let _ = std::panic::take_hook();
            bindings.to_string()
        });
        let res = handle.join();
        let is_ok = if let Ok(res) = res {
            let pat = "pub type test = ";
            if let Some(i) = res.find(pat) {
                print!("{}, ", &res[i + pat.len()..i + pat.len() + 3]);
            }
            res.contains("pub type test = u32") || res.contains("pub type test = i32")
        } else {
            false
        };
        if is_ok {
            valid_targets.push(target);
        }
        println!("{}", is_ok);
    }
    //write valid target to a file
    let mut out_path = PathBuf::new();
    out_path.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_path.pop();
    out_path.push("build_data");
    out_path.push("valid_targets.txt");
    let mut f = fs::File::create(out_path).unwrap();
    for target in valid_targets {
        writeln!(f, "{}", target).unwrap();
    }
}
