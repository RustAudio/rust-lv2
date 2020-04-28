extern crate bindgen;
use lv2_sys_bindgen::compare::*;
use lv2_sys_bindgen::*;
use std::env;
use std::env::Args;
use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;

const USAGE: &str = "
Usage: lv2-sys-bindgen generate -i LV2-DIR -o OUTPUT-FILE    Generate bindings
   or: lv2-sys-bindgen compare FILE1 FILE2                   Compare bindings
";

type DynError = Box<dyn Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        println!("{}", USAGE);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let mut args = env::args();
    let subcommand = args.nth(1);
    if let Some(subcommand) = subcommand {
        match subcommand.as_ref() {
            "generate" => cmd_generate(args)?,
            "compare" => cmd_compare(args)?,
            _ => return Err(format!("Unknown subcommand: `{}`", subcommand).into()),
        }
    } else {
        return Err("Subcommand expected".into());
    }
    Ok(())
}

fn cmd_generate(mut args: Args) -> Result<(), DynError> {
    let mut input: Option<_> = None;
    let mut output: Option<_> = None;
    let mut target: Option<_> = None;

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
        return Err("No path to the LV2 directory was provided.".into());
    };
    let out_path: PathBuf = if let Some(val) = output {
        val.into()
    } else {
        return Err("No output file was provided".into());
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

fn cmd_compare(mut args: Args) -> Result<(), DynError> {
    let file1 = if let Some(arg) = args.next() {
        PathBuf::from(arg)
    } else {
        return Err("missing argument".into());
    };
    let file2 = if let Some(arg) = args.next() {
        PathBuf::from(arg)
    } else {
        return Err("missing argument".into());
    };
    match compare(
        &env::current_dir()?.join(&file1),
        &env::current_dir()?.join(&file2),
    )? {
        CmpResult::Equivalent => println!("Bindings files are equivalent."),
        CmpResult::Different(diff) => {
            println!("Bindings files aren't equivalent:");
            if !diff.file1.is_empty() {
                println!("Item only present in '{}':", file1.to_string_lossy());
                for item in diff.file1 {
                    println!("{}", item);
                }
            }
            if !diff.file2.is_empty() {
                println!("Item only present in '{}':", file2.to_string_lossy());
                for item in diff.file2 {
                    println!("{}", item);
                }
            }
        }
    }
    Ok(())
}
