extern crate bindgen;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

pub mod compare;

type DynError = Box<dyn Error>;

/// Generate lv2-sys bindings
pub fn generate_bindings(lv2_dir: &Path, out_file: &Path, target: Option<&str>) {
    let mut bindings = bindgen::Builder::default().size_t_is_usize(true);

    // Adding the crate to the include path of clang.
    // Otherwise, included headers can not be found.
    let mut include_path = PathBuf::from(lv2_dir);
    include_path.pop();
    bindings = bindings.clang_arg(format!("-I{}", include_path.to_str().unwrap()));
    if let Some(target) = target {
        bindings = bindings.clang_arg(format!("--target={}", target));
    }

    for entry in fs::read_dir(lv2_dir).unwrap() {
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
        .write_to_file(out_file)
        .expect("Couldn't write bindings!");
}

/// Check and display target compatitibily and saves the good one into a file.
///
/// work_dir is the path to the lv2-sys crate
pub fn generate_valid_target(out_dir: &Path) {
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
    println!("Target compatibility:");
    println!(
        "{:-^40}|{:-^12}|{:-^8}|",
        "target triple", "enum repr.", "status"
    );
    for target in targets {
        print!("{:<40}|", target);
        io::stdout().flush().unwrap();
        match get_target_enum(target) {
            Ok(res) => {
                print!("{:^12}|", res);
                if res.contains("32") {
                    println!("{:^8}|", "Ok");
                    valid_targets.push(target);
                } else {
                    println!("{:^8}|", "Not Ok");
                }
            }
            Err(_) => {
                println!("{:^12}|{:^8}|", "??", "Error");
            }
        };
    }
    //write valid target to a file
    let mut out_path = PathBuf::from(out_dir);
    out_path.push("valid_targets.txt");
    let mut f = fs::File::create(out_path).unwrap();
    for target in valid_targets {
        writeln!(f, "{}", target).unwrap();
    }
    println!("Valid targets saved!");
}

/// Return the target enum representation or error if bindgen panics
pub fn get_target_enum(target: &str) -> Result<String, DynError> {
    let mut test_h = PathBuf::new();
    test_h.push(env::var("CARGO_MANIFEST_DIR")?);
    test_h.push("enum_test.h");
    let test_h = test_h.to_str().unwrap();
    let test_h = String::from(test_h);
    let target = String::from(target);
    //the thread spawning avoid to exit when bindgen panics
    let handle = thread::spawn(move || {
        let builder = bindgen::Builder::default()
            .size_t_is_usize(true)
            .clang_arg(format!("--target={}", target))
            .header(test_h);
        // make silent panic
        std::panic::set_hook(Box::new(|_| ()));
        let bindings = builder.generate().unwrap();
        //restore default panic hook
        let _ = std::panic::take_hook();
        bindings.to_string()
    });
    let res = handle.join();
    match res {
        Ok(res) => {
            let pat = "pub type test = ";
            let i1 = res.find(pat).unwrap() + pat.len();
            let i2 = i1 + res.get(i1..).unwrap().find(';').unwrap();
            let repr = res.get(i1..i2).unwrap();
            Ok(String::from(repr))
        }
        Err(_) => Err("bindgen panicked".into()),
    }
}
