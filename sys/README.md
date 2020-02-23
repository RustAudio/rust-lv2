# rust-lv2 header bindings

Bindings to the official [LV2](https://lv2plug.in/) API headers, used by `rust-lv2` audio plugin framework, a safe, fast, and ergonomic framework to create LV2 plugins for audio processing on any platform, written in Rust.

## Building

Since the bindings to the raw C headers are generated with bindgen, you need to have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't in your system's standard path, set the environment variable `LIBCLANG_PATH` to the path of `libClang`.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.