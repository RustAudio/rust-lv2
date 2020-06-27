//! A safe, fast, and ergonomic Rust framework to create [LV2 plugins](http://lv2plug.in/) for audio processing or synthesis.
//!
//! **This library is a work in progress.**
//!
//! It provides the following features, through the [LV2 Core specification](http://lv2plug.in/ns/lv2core/lv2core.html):
//!
//! * Lightweight, realtime non-blocking and allocation-free audio processing.
//! * Generates all the boilerplate to make a LV2 plugin binary, usable by any LV2 host.
//! * Any number of ports / Any channel mapping, which can be different for input and output.
//!   This obviously includes Mono, Stereo, Surround, etc., anything your CPU can handle.
//! * Can be extended to support any additional features, extensions and port types.
//!   They can be official, unofficial or completely custom.
//!
//! Through the [LV2 official additional specifications](http://lv2plug.in/ns/), this library also provide many
//! additional features, including:
//!
//! * MIDI processing
//! * Serialization of custom data structures, and plugin-plugin or plugin-GUI communication and property manipulation
//! * State management
//! * Asynchronous work processing
//! * Custom Graphical User Interfaces, both in a toolkit-agnostic and in a platform-agnostic way **(Not yet implemented)**
//! * Presets handling **(Not yet implemented)**
//! * ... and more! (Not yet implemented either)
//!
//! Note that this library will only provide Rust bindings for the official LV2 specifications, however it is compatible
//! with any other arbitrary or custom specification, and other, external crates are able and welcome to provide Rust bindings
//! to any other specification that will integrate with this library.
//!
//! # Example
//!
//! A simple amplification plugin would like this:
//!
//! ```
//! // Import everything we need.
//! use lv2::prelude::*;
//!
//! // The input and output ports are defined by a struct which implements the `PortCollection` trait.
//! // In this case, there is an input control port for the gain of the amplification, an input audio
//! // port and an output audio port.
//! #[derive(PortCollection)]
//! struct Ports {
//!     gain: InputPort<Control>,
//!     input: InputPort<Audio>,
//!     output: OutputPort<Audio>,
//! }
//!
//! // The plugin struct. In this case, we don't need any data and therefore, this struct is empty.
//! //
//! // LV2 uses URIs to identify types. This association is expressed via the `UriBound` trait,
//! // which tells the framework that the type `Amp` is identified by the given URI. The usual
//! // way to implement this trait is to use the `uri` attribute.
//! #[uri("urn:rust-lv2-book:eg-amp-rs")]
//! struct Amp;
//!
//! // The implementation of the `Plugin` trait, which turns `Amp` into a plugin.
//! impl Plugin for Amp {
//!     // Tell the framework which ports this plugin has.
//!     type Ports = Ports;
//!
//!     // We don't need any special host features; We can leave them out.
//!     type InitFeatures = ();
//!     type AudioFeatures = ();
//!
//!     // Create a new instance of the plugin; Trivial in this case.
//!     fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
//!         Some(Self)
//!     }
//!
//!     // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
//!     // iterates over.
//!     fn run(&mut self, ports: &mut Ports, _features: &mut ()) {
//!         let coef = if *(ports.gain) > -90.0 {
//!             10.0_f32.powf(*(ports.gain) * 0.05)
//!         } else {
//!             0.0
//!         };
//!
//!         for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
//!             *out_frame = in_frame * coef;
//!         }
//!     }
//! }
//! ```
//!
//! # Using this framework
//!
//! ## Documentation
//!
//! There are multiple valuable sources of documentation:
//! * ["The Rust-LV2 book"](https://rustaudio.github.io/rust-lv2/) describes how to use Rust-LV2 in general, broad terms. It's the ideal point to get started and is updated with every new version of Rust-LV2.
//! * [The API documentation](https://docs.rs/lv2).
//! * [The LV2 specification reference](https://lv2plug.in/ns/).
//!
//! ## Features
//!
//! Internally, this framework is built of several sub-crates which are re-exported by the `lv2` crate. All dependencies are optional and can be enabled via features. These are:
//!
//! * `lv2-atom`: General data IO.
//! * `lv2-core`: Implementation of the core LV2 specification.
//! * `lv2-options`: Implementation of the LV2 Options extension.
//! * `lv2-midi`: MIDI message extension for `lv2-midi`. Support for the [`wmidi` crate](https://crates.io/crates/wmidi) can be enabled with the `wmidi` feature.
//! * `lv2-state`: Extension for LV2 plugins to store their state.
//! * `lv2-time`: Specification to describe position in time and passage of time, in both real and musical terms.
//! * `lv2-units`: Measuring unit definitions.
//! * `lv2-urid`: LV2 integration of the URID concept.
//! * `lv2-worker`: Work scheduling library that allows real-time capable LV2 plugins to execute non-real-time actions.
//! * `urid`: Idiomatic URID support.
//!
//! Sub-crates with an `lv2-` prefix implement a certain LV2 specification, which can be looked up in [the reference](https://lv2plug.in/ns/). Enabling a crate only adds new content, it does not remove or break others.
//!
//! There are also feature sets that account for common scenarios:
//! * `minimal_plugin`: The bare minimum to create plugins. Includes `lv2-core` and `urid`.
//! * `plugin`: Usual crates for standard plugins. Includes `lv2-core`, `lv2-atom`, `lv2-midi` with the `wmidi` feature, `lv2-units`, `lv2-urid`, and `urid`. **This is the default.**
//! * `full`: All sub-crates.
//!
//! # Extending
//!
//! Please note that this re-export crate is only meant to be used by plugin projects. If you want to extend the framework with your own crates, please use the sub-crates as your dependencies instead. This will dramatically boost building durations and backwards compability.

/// The super-prelude that contains the preludes of all enabled crates.
pub mod prelude {
    #[cfg(feature = "lv2-atom")]
    pub use ::lv2_atom::prelude::*;
    #[cfg(feature = "lv2-core")]
    pub use ::lv2_core::prelude::*;
    #[cfg(feature = "lv2-midi")]
    pub use ::lv2_midi::prelude::*;
    #[cfg(feature = "lv2-options")]
    pub use ::lv2_options::*;
    #[cfg(feature = "lv2-state")]
    pub use ::lv2_state::*;
    #[cfg(feature = "lv2-time")]
    pub use ::lv2_time::prelude::*;
    #[cfg(feature = "lv2-units")]
    pub use ::lv2_units::prelude::*;
    #[cfg(feature = "lv2-urid")]
    pub use ::lv2_urid::*;
    #[cfg(feature = "lv2-work")]
    pub use ::lv2_worker::prelude::*;
    #[cfg(feature = "urid")]
    pub use ::urid::*;
}

#[cfg(feature = "lv2-atom")]
pub extern crate lv2_atom;

#[cfg(feature = "lv2-core")]
pub extern crate lv2_core;

#[cfg(feature = "lv2-midi")]
pub extern crate lv2_midi;

#[cfg(feature = "lv2-options")]
pub extern crate lv2_options;

#[cfg(feature = "lv2-state")]
pub extern crate lv2_state;

#[cfg(feature = "lv2-sys")]
pub extern crate lv2_sys;

#[cfg(feature = "lv2-time")]
pub extern crate lv2_time;

#[cfg(feature = "urid")]
pub extern crate urid;

#[cfg(feature = "lv2-units")]
pub extern crate lv2_units;

#[cfg(feature = "lv2-urid")]
pub extern crate lv2_urid;

#[cfg(feature = "lv2-worker")]
pub extern crate lv2_worker;
