extern crate bindgen;
use lv2_sys_bindgen::*;
use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;

type DynError = Box<dyn Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let mut input: Option<_> = None;
    let mut output: Option<_> = None;
    let mut target: Option<_> = None;
    let mut args = env::args();

    //parse the arguments
    while let Some(arg) = args.next() {
        //lv2 path
        if arg == "-i" {
            if let Some(t) = args.next() {
                input = Some(t);
            }
        }
        //out path
        if arg == "-o" {
            if let Some(arg) = args.next() {
                output = Some(arg)
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
    let lv2_path: PathBuf = if let Some(val) = input {
        val.into()
    } else {
        return Err("No path to the LV2 directory was provided.".into())
    };
    let out_path: PathBuf = if let Some(val) = output {
        val.into()
    } else {
        return Err("No output file was provided".into())
    };

    let target = target.as_deref();

    print!("Generating bindings...");
    io::stdout().flush().unwrap();
    generate_bindings(
        &env::current_dir().unwrap().join(&lv2_path),
        &env::current_dir().unwrap().join(&out_path),
        target,
    );
    println!(" Done");
    Ok(())
}
