# Generate Rust bindings of the LV2 C API

## Requirements

Since the bindings to the raw C headers are generated with bindgen, you need to
have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't
in your system's standard path, set the environment variable `LIBCLANG_PATH` to
the path of `libClang`.

## Usage

Usage (anywhere is rust-lv2 workspace):
* `cargo run -p systool -- --lv2 <DIR> --out <OUT> [-- <CLANG-ARGS>...]`
* `cargo systool --lv2 <DIR> --out <OUT> [-- <CLANG-ARGS>...]` (alias of the
  first)

Options:
* `-I, --lv2 <DIR>`: The path to the LV2 C API
* `-o, --out <OUT>`: The file to write the bindings to

Args:
* `<CLANG-ARGS>...`:   Arguments passed to clang

## Cross-platform bindings

Cross-platform bindings generation require C headers of the target
platform. The easiest way to obtain them is installing a C cross-compiler for
that target platform. Then, you provide the cross-compiler root path as a clang
arguments. Example with `arm-linux-gnueabi`:

`cargo run -p systool -- --lv2 lv2 --out arm-linux-gnueabi.rs --
--target=arm-linux-gnueabi --sysroot /usr/arm-linux-gnueabi`

Sometimes, clang can't find some headers, you need to manually provide their
path using `-I`. Example with `aarch64-unknown-linux-gnu`:

`cargo run -p systool -- --lv2 lv2 --out aarch64-unknown-linux-gnu.rs --
--target=aarch64-unknown-linux-gnu --sysroot /usr/aarch64-unknown-linux-gnu
-I/usr/aarch64-linux-gnu/include/`
