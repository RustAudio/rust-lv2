extern crate bindgen;
use lv2_sys_bindgen::*;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

type DynError = Box<dyn Error>;

fn main() {
    let mut input: Option<_> = None;
    let mut output: Option<_> = None;
    let mut target: Option<_> = None;
    let mut args = env::args();

    //parse the arguments
    while let Some(arg) = args.next() {
        //out path
        if arg == "-o" {
            if let Some(arg) = args.next() {
                output = Some(arg)
            }
        }
        //lv2 path
        if arg == "-i" {
            if let Some(t) = args.next() {
                input = Some(t);
            }
        }
        //target
        if arg == "--target" {
            if let Some(t) = args.next() {
                target = Some(t);
            }
        } else if arg.starts_with("--target=") {
            if let Some(t) = arg.get("--target=".len()..arg.len()) {
                target = Some(String::from(t))
            }
        }
    }

    //check and get the required argument
    let out_path: PathBuf = if let Some(val) = output {
        val.into()
    } else {
        panic!("No output file was provided")
    };
    let lv2_path: PathBuf = if let Some(val) = input {
        val.into()
    } else {
        panic!("No path to the LV2 directory was provided.")
    };

    let target = target.as_deref();

    let mut work_dir = PathBuf::new();
    work_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    work_dir.pop();

    let out_dir = work_dir.join("build_data");

    if get_target_enum("").unwrap().contains("32") {
        print!("Generating bindings...");
        io::stdout().flush().unwrap();
        generate_bindings(
            &env::current_dir().unwrap().join(&lv2_path),
            &env::current_dir().unwrap().join(&out_path),
            target,
        );
        println!(" Done");
        generate_valid_target(&out_dir);
    } else {
        eprintln!("host enum layout must be u32 or i32");
        std::process::exit(-1);
    }
}

use std::thread;

/// Check and display target compatitibily and saves the good one into a file.
///
/// work_dir is the path to the lv2-sys crate
fn generate_valid_target(out_dir: &Path) {
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
