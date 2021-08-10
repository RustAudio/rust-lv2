# Rust-LV2's wrapper of LV2's unit types.

This crate binds some of the unit-related URIs from [the `sys`-crate](https://crates.io/crates/lv2-sys) to types and is a part of [`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust.

## Documentation

The original LV2 API (in the `C` programming language) is documented by ["the LV2 book"](https://lv2plug.in/book/). This book is in the process of being translated to Rust along with the development of `rust-lv2` [(link)](https://janonard.github.io/rust-lv2-book/) and describes how to properly use `rust-lv2`.

## Features

There are two optional features:
* `host`:  Some of the types defined by some crates are only useful for testing or LV2 hosts. Since the goal of this framework is to provide an easy way to create plugins, these aren't necessary and therefore gated behind that feature.
* `wmidi`: Add [`wmidi`](https://crates.io/crates/wmidi) as an optional dependency to `lv2-midi`, which enables a shortcut to read and write MIDI events directly with the types defined by this crate.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.