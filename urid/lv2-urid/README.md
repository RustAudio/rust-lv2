# Rust-LV2's URID handling library.

URI <-> URID mapping utilities used by [`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust.

LV2 often uses URIs to identify types. However, comparing strings isn't particularly performant. Therefore, every URI can be mapped to a URID (basically a `u32`) by the host, which can be efficiently compared. This concept is implemented by the [`urid` crate](https://crates.io/crates/urid) and this crate adapts it for the use by an LV2 plugin or host.

## Documentation

The original LV2 API (in the `C` programming language) is documented by ["the LV2 book"](https://lv2plug.in/book/). This book is in the process of being translated to Rust along with the development of `rust-lv2` [(link)](https://janonard.github.io/rust-lv2-book/) and describes how to properly use `rust-lv2`.

## Features

Like any other crate of `rust-lv2`, this crate has the optional `host` feature. Some of the types defined by some crates are only useful for testing or LV2 hosts. Since the goal of this framework is to provide an easy way to create plugins, these aren't necessary and therefore gated behind that feature.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.