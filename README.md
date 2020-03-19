# rust-lv2

[![Build Status][travis-badge]][travis-url] [![Current Crates.io Version][crates-badge]][crates-url]

[travis-badge]: https://travis-ci.org/rustaudio/rust-lv2.svg?branch=master
[travis-url]: https://travis-ci.org/rustaudio/rust-lv2
[crates-badge]: https://img.shields.io/crates/v/lv2.svg
[crates-url]: https://crates.io/crates/lv2

A safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust.

**This library is a work in progress.**

It provides the following features, through the [LV2 Core specification](http://lv2plug.in/ns/lv2core/lv2core.html):

* Lightweight, realtime non-blocking and allocation-free audio processing.
* Generates all the boilerplate to make a LV2 plugin binary, usable by any LV2 host.
* Any number of ports / Any channel mapping, which can be different for input and output.  
  This obviously includes Mono, Stereo, Surround, etc., any configuration your CPU can handle.
* Can be extended to support any additional features, extensions and port types.  
  They can be official, unofficial or completely custom.

Through the [LV2 official additional specifications](http://lv2plug.in/ns/), this library also provides many
additional features, including:

* MIDI processing
* Serialization of custom data structures, and plugin-plugin or plugin-GUI communication and property manipulation
* State management
* Custom Graphical User Interfaces, both in a toolkit-agnostic and in a platform-agnostic way **(Not yet implemented)**
* Presets handling **(Not yet implemented)**
* Asynchronous work processing **(Not yet implemented)**
* ... and more! (Not yet implemented either)

Note that this library will only provide Rust bindings for the official LV2 specifications, however it is compatible with any other arbitrary or custom specification, and other, external crates are able and welcome to provide Rust bindings to any other specification that will integrate with this library.

## Example

This example contains the code of a simple amplification plugin. Please note that this isn't the only thing required to create a plugin, see the documentation below for more details.

```Rust
// Import everything we need.
use lv2::core::prelude::*;
use urid::*;

// The input and output ports are defined by a struct which implements the `PortCollection` trait.
// In this case, there is an input control port for the gain of the amplification, an input audio
// port and an output audio port.
#[derive(PortCollection)]
struct Ports {
    gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

// The plugin struct. In this case, we don't need any data and therefore, this struct is empty.
struct Amp;

// LV2 uses URIs to identify types. This association is expressed via the `UriBound` trait, which
// tells the framework that the type `Amp` is identified by the given URI.
//
// This trait is unsafe to implement since you **need** to include the \0 character at the end of
// the string.
unsafe impl UriBound for Amp {
    const URI: &'static [u8] = b"urn:rust-lv2-book:eg-amp-rs\0";
}

// The implementation of the `Plugin` trait, which turns `Amp` into a plugin.
impl Plugin for Amp {
    // Tell the framework which ports this plugin has.
    type Ports = Ports;
    // We don't need any special host features; We can leave them out.
    type InitFeatures = ();
    type AudioFeatures = ();

    // Create a new instance of the plugin; Trivial in this case.
    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self)
    }

    // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
    // iterates over.
    fn run(&mut self, ports: &mut Ports, _features: &mut ()) {
        let coef = if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
            *out_frame = in_frame * coef;
        }
    }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(Amp);
```

## Documentation

The original LV2 API (in the `C` programming language) is documented by ["the LV2 book"](https://lv2plug.in/book/). This book is in the process of being translated to Rust along with the development of `rust-lv2` [(link)](https://janonard.github.io/rust-lv2-book/) and describes how to properly use `rust-lv2`.

## Building

Since the bindings to the raw C headers are generated with bindgen, you need to have [Clang](https://clang.llvm.org/) installed on your system and, if it isn't in your system's standard path, set the environment variable `LIBCLANG_PATH` to the path of `libClang`.

## Q&A

### Does my host program support it?

Plugins created with `rust-lv2` are compatible to all LV2 hosts that comply to the specifications. If your application uses [`lilv`](https://drobilla.net/software/lilv), it's a good sign that it will support your plugin. Some prime examples are [Carla](https://kx.studio/Applications:Carla) and [Ardour](https://ardour.org/).

### What targets are supported?

We currently support stable and beta Rust running on macOS and Linux. Windows will probably work too, but the Windows build environment of Travis CI is currently broken and we therefore can not support it.

We would like to also support Windows as well as ARM-based embedded devices like Raspberry Pis. If you can help us with these targets, please do so!

### Can I host plugins with `rust-lv2`?

Currently, hosting plugins is not supported. This project was initialy started to create plugins using safe Rust and therefore, it is very plugin-centric. There are plans for integrated plugin hosting or a spin-off project, but those won't start in the near future.

However, there is a lot of code that can be re-used for a hosting framework. If you want to create such a framework, you should take a look at `lv2-sys`, `urid`, and `lv2-atom`.

A bare hosting framework would require an RDF triple store which can load Turtle files, an internal store for plugin interfaces and their extensions, a centralized URID map store, and a graph based work scheduling system to execute `run` functions in order.

### Why `bindgen`?

`lv2-sys` uses `bindgen` to generate the Rust representation of the LV2 C API. Rust can not handle verbatim C code, but is able to define type and function definitions that exactly match those from the C headers. However, since serveral importants details in C aren't properly defined, these bindings need to be different for every platform. One example: While Rust's `u32` is always an unsigned, 32-bit wide integer, C's `int` may be 16 to 64 bits wide and may be signed or unsigned; It depends on the platform.

One solution would be to generate bindings for every supported target, but if we would only support stable, beta and nightly Rust on [tier 1 platforms](https://forge.rust-lang.org/release/platform-support.html#tier-1), we would still have to maintain 21 different versions of the same crate. If we would add [tier 2 platforms](https://forge.rust-lang.org/release/platform-support.html#tier-2) too (which would include e.g. the Raspberry Pis), there would be 216(!) different versions.

I guess it's obvious that this isn't a maintainable situation. Therefore, the bindings need to be generated every time they are build, which requires the build dependency to `bindgen`.

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
