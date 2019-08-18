# rust-lv2

[![Build Status](https://travis-ci.org/rust-dsp/rust-lv2.svg?branch=master)](https://travis-ci.org/rust-dsp/rust-lv2)

A safe, fast, and ergonomic Rust library to create [LV2 plugins](http://lv2plug.in/) for audio processing or synthesis,
on any platform.

**This library is a work in progress.**

It provides the following features, through the [LV2 Core specification](http://lv2plug.in/ns/lv2core/lv2core.html):

* Lightweight, realtime non-blocking and allocation-free audio processing.
* Generates all the boilerplate to make a LV2 plugin binary, usable by any LV2 host on any platform.
* Any number of ports / Any channel mapping, which can be different for input and output.  
  This obviously includes Mono, Stereo, Surround, etc., any configuration your CPU can handle.
* Can be extended to support any additional features, extensions and port types.  
  They can be official, unofficial or completely custom.

Through the [LV2 official additional specifications](http://lv2plug.in/ns/), this library also provide many
additional features, including:

* MIDI processing **(Not yet implemented)**
* Custom Graphical User Interfaces, both in a toolkit-agnostic and in a platform-agnostic way **(Not yet implemented)**
* Serialization of custom data structures, and plugin-plugin or plugin-GUI communication and property manipulation **(Not yet implemented)**
* Presets handling and State management **(Not yet implemented)**
* Asynchronous work processing **(Not yet implemented)**
* … and more! (Not yet implemented either)

Note that this library will only provide Rust bindings for the official LV2 specifications, however it is compatible
with any other arbitrary or custom specification, and other, external crates are able and welcome to provide Rust bindings
to any other specification that will integrate with this library.

## Building

Since the `sys` crates provided by this workspace use `bindgen` to create the C API bindings at compile time, you need to have clang installed on your machine in order to build them.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
