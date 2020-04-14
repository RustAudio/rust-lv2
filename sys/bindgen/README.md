# Script to pre-generate bindings for lv2-sys

Script that pre-generate bindings for lv2-sys crate.

## Requirement

Since the bindings to the raw C headers are generated with bindgen, you need to
have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't
in your system's standard path, set the environment variable `LIBCLANG_PATH` to
the path of `libClang`.

C enums should map to `u32` or `i32` on the host. Don't worry, it's the case on
most computers.

## Usage

Just run `cargo run -p lv2-sys-bindgen` or `cargo run lv2-sys-bindgen` anywhere
inside the rust-lv2 workspace. It require to be launch through cargo since it
use some environment variables defined by it.

## Output files

The script create `bindings.rs` and `valid_targets.txt`inside
`rust-lv2/sys/build_data`:
* `bindings.rs`: contains bindings to lv2 headers.
* `valid_targets.txt`: contains a list of target-triple with wich bindings can
  work.

## Reports

During the generation of the target-triple compatible with `bindings.rs`, the
script display a tab of tested target-triple:
* target triple: the tested target triple.
* enum repr.: rust type used to represent a common enum. `??` indicate it's
  couldn't be determined.
* status: indicate the result of the test:
  * `Ok`: bindings will work with this target-triple
  * `Not Ok`: bindings will not work with this target-triple
  * `Error`: bindgen panicked during the test
