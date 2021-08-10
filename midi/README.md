## Rust-LV2's MIDI processing library.

The MIDI processing library of [`rust-lv2`](https://crates.io/crates/lv2), a safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust.

This is a addon to [`lv2-atom`](https://crates.io/crates/lv2-atom) that adds the `MidiEvent` atom type. There is also an optional dependency to [`wmidi`](https://crates.io/crates/wmidi), which introduces the `WMidiEvent` atom type, which allows you to directly read and write the events defined by `wmidi`.

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