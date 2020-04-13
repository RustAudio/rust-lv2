# Rust-LV2's library to implement LV2 Worker extension.

Work scheduling library that allows real-time capable LV2 plugins to execute
non-real-time actions. This is a part of
[`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic
framework to create [LV2 plugins](http://lv2plug.in/) for audio processing,
written in Rust.

## Documentation

The original LV2 API (in the `C` programming language) is documented by 
["the LV2 book"](https://lv2plug.in/book/). This book is in the process of
being translated to Rust along with the development of `rust-lv2`
[(link)](https://janonard.github.io/rust-lv2-book/) and describes how to
properly use `rust-lv2`.

## Building

Since the bindings to the raw C headers are generated with bindgen, you need to
have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't
in your system's standard path, set the environment variable `LIBCLANG_PATH` to
the path of `libClang`.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
