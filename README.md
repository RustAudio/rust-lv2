# Rust-LV2

[![Build Status][travis-badge]][travis-url] [![Current Crates.io Version][crates-badge]][crates-url]

[travis-badge]: https://travis-ci.org/rustaudio/rust-lv2.svg?branch=master
[travis-url]: https://travis-ci.org/rustaudio/rust-lv2
[crates-badge]: https://img.shields.io/crates/v/lv2.svg
[crates-url]: https://crates.io/crates/lv2

A safe, fast, and ergonomic framework to create [LV2 plugins](http://lv2plug.in/) for audio processing, written in Rust.

**This library is a work in progress.**

It provides the following features, through the [LV2 Core specification](http://lv2plug.in/ns/lv2core/lv2core.html):

* Lightweight, real-time non-blocking and allocation-free audio processing.
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
* Asynchronous work processing
* Custom Graphical User Interfaces, both in a toolkit-agnostic and in a platform-agnostic way **(Not yet implemented)**
* Presets handling **(Not yet implemented)**
* ... and more! (Not yet implemented either)

Note that this library will only provide Rust bindings for the official LV2 specifications, however it is compatible with any other arbitrary or custom specification, and other, external crates are able and welcome to provide Rust bindings to any other specification that will integrate with this library.

## Example

This example contains the code of a simple amplification plugin. Please note that this isn't the only thing required to create a plugin, see the documentation below for more details.

```Rust
// Import everything we need.
use lv2::prelude::*;

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
//
// LV2 uses URIs to identify types. This association is expressed via the `UriBound` trait,
// which tells the framework that the type `Amp` is identified by the given URI. The usual
// way to implement this trait is to use the `uri` attribute.
#[uri("urn:rust-lv2-book:eg-amp-rs")]
struct Amp;

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

## About this framework

### Q&A

#### Does my host program support it?

Plugins created with `rust-lv2` are compatible to all LV2 hosts that comply to the specifications. If your application uses [`lilv`](https://drobilla.net/software/lilv), it's a good sign that it will support your plugin. Some prime examples are [Carla](https://kx.studio/Applications:Carla) and [Ardour](https://ardour.org/).

#### Can I host plugins with `rust-lv2`?

Currently, hosting plugins is not supported. This project was initially started to create plugins using safe Rust and therefore, it is very plugin-centric. There are plans for integrated plugin hosting or a spin-off project, but those won't start in the near future.

However, there is a lot of code that can be re-used for a hosting framework. If you want to create such a framework, you should take a look at `lv2-sys`, `urid`, and `lv2-atom`.

A bare hosting framework would require an RDF triple store which can load Turtle files, an internal store for plugin interfaces and their extensions, a centralized URID map store, and a graph based work scheduling system to execute `run` functions in order.

### Documentation

There are multiple valuable sources of documentation:
* ["The Rust-LV2 book"](https://rustaudio.github.io/rust-lv2/) describes how to use Rust-LV2 in general, broad terms. It's the ideal point to get started and is updated with every new version of Rust-LV2.
* [The API documentation](https://docs.rs/lv2).
* [The LV2 specification reference](https://lv2plug.in/ns/).

### Features

Internally, this framework is built of several sub-crates which are re-exported by the `lv2` crate. All dependencies are optional and can be enabled via features. These are:

* `lv2-atom`: General data IO.
* `lv2-core`: Implementation of the core LV2 specification.
* `lv2-midi`: MIDI message extension for `lv2-midi`. Support for the [`wmidi` crate](https://crates.io/crates/wmidi) can be enabled with the `wmidi` feature.
* `lv2-state`: Extension for LV2 plugins to store their state.
* `lv2-time`: Specification to describe position in time and passage of time, in both real and musical terms.
* `lv2-units`: Measuring unit definitions.
* `lv2-urid`: LV2 integration of the URID concept.
* `lv2-worker`: Work scheduling library that allows real-time capable LV2 plugins to execute non-real-time actions.
* `urid`: Idiomatic URID support.

Sub-crates with an `lv2-` prefix implement a certain LV2 specification, which can be looked up in [the reference](https://lv2plug.in/ns/). Enabling a crate only adds new content, it does not remove or break others.

There are also feature sets that account for common scenarios:
* `minimal_plugin`: The bare minimum to create plugins. Includes `lv2-core` and `urid`.
* `plugin`: Usual crates for standard plugins. Includes `lv2-core`, `lv2-atom`, `lv2-midi`, `lv2-urid`, and `urid`. **This is the default.**
* `full`: All sub-crates.

Some build targets are not fully supported yet. Use the `experimental-targets` feature to enable them.

## Supported targets

Rust-LV2 uses pre-generated LV2 API bindings for different targets in order to increase usability and building speed. Rust has a lot of [supported targets](https://forge.rust-lang.org/release/platform-support.html), but our maintaining power is limited and therefore, only certain targets can be supported.

A target is supported by Rust-LV2 if a compatible binding was generated and verified for it. This requires a [maintainer](https://github.com/orgs/RustAudio/teams/lv2-maintainers) who can generate a compatible binding and a tester who can verify binding for that target. Bindings are generated with the [LV2 systool](sys/tool/) and verification is done by building the [example plugins of the book](docs) and testing them with a host of that target.

Some targets have a compatible binding but haven't been verified, these targets are usable but may not work and aren't officially supported.

Some bindings have a maintainer, but haven't been verified yet on any target. These binding are experimental and are gated behind the `experimental-targets` feature.

### Bindings

| Binding | Maintainer | Experimental | Compatibility |
|---------|------------|--------------|---------------|
| `linux/arm.rs`| @Yruama_Lairba | Yes | `arm*-*-linux-*` and `aarch64-*-linux-*` |
| `linux/x86_64.rs` | @Janonard | No | `x86_64-*-linux-*` |
| `linux/x86.rs` | @Janonard | No | `x86-*-linux-*`|
| `windows.rs` | @Janonard | Yes | `*-windows-*` |

### Supported targets

| Target | Tester | Last Verification |
|--------|--------|-------------------|
| `x86_64-unknown-linux-gnu` | @Janonard | 10. of May 2020, using [Carla](https://github.com/falkTX/Carla) v2.1, running on Arch Linux |
| `x86-unknown-linux-gnu` | @Janonard | 16th of May 2020, using [Carla](https://github.com/falkTX/Carla) v2.1, running on Linux Mint 19.3 32-bit |

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
