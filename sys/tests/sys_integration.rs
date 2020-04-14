#[test]
fn bindings_are_equivalent() {
    extern crate lv2_sys_bindgen;
    use std::env;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::iter::Iterator;
    use std::path::PathBuf;

    let work_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = work_dir.join("lv2");
    let bindings1_dir = work_dir.join("build_data");
    let bindings2_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("{}", bindings2_dir.to_str().unwrap());
    lv2_sys_bindgen::generate_bindings(&source_dir, &bindings2_dir);

    let f1 = File::open(bindings1_dir.join("bindings.rs")).unwrap();
    let mut f1 = BufReader::new(f1).lines();

    let f2 = File::open(bindings2_dir.join("bindings.rs")).unwrap();
    let mut f2 = BufReader::new(f2).lines();

    let mut line_count = 1_usize;

    loop {
        let l1 = f1.next();
        let l2 = f2.next();
        match (l1, l2) {
            (Some(l1), Some(l2)) => {
                let l1 = l1.unwrap().replace("i32", "u32");
                let l2 = l2.unwrap().replace("i32", "u32");
                if l1 != l2 {
                    panic!(
                        "line {}: bindings aren't equivalent\npre-bindings: {}\ngenerated   : {}",
                        line_count, l1, l2
                    );
                }
            }
            (Some(_), None) => panic!("pre-bindings contains more lines than generated bindings"),
            (None, Some(_)) => panic!("generated bindings contains more lines than pre-bindings"),
            (None, None) => break,
        }
        //counting line
        line_count += 1;
    }
}
