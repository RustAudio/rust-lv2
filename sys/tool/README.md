# Generate Rust bindings of the LV2 C API

## Requirements

Since the bindings to the raw C headers are generated with bindgen, you need to
have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't
in your system's standard path, set the environment variable `LIBCLANG_PATH` to
the path of `libClang`.

## Usage

Usage (anywhere is rust-lv2 workspace):
* `cargo run -p lv2-sys-bindgen -- [OPTIONS]`
* `cargo make-bindings [OPTIONS]` (alias of the first)

Options:
* `-I, --lv2 <DIR>`: The path to the LV2 C API
* `-o, --out <OUT>`: The file to write the bindings to