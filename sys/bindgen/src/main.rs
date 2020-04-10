extern crate bindgen;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

type DynError = Box<dyn Error>;

fn main() {
    generate_bindings();
    generate_valid_target();
}

use std::thread;

fn generate_valid_target() {
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
        match res {
            Ok(res) => {
                let pat = "pub type test = ";
                let i1 = res.find(pat).unwrap() + pat.len();
                let i2 = i1 + res.get(i1..).unwrap().find(';').unwrap();
                let repr = res.get(i1..i2).unwrap();
                print!("enum is {}, ", repr);
                if repr.contains("u32") || repr.contains("i32") {
                    println!("valid.");
                    valid_targets.push(target);
                } else {
                    println!("not valid.");
                }
            }
            Err(_) => {
                println!("bindgen error");
            }
        };
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

/// Return the target enum representation or error if bindgen panics
fn get_target_enum(target: &str) -> Result<String, DynError> {
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
        //restore default panic hook because it's not thread local
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
