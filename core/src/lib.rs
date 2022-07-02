//! Implementation of the core LV2 specification
//!
//! This crate forms the foundation of the LV2 experience for Rust: It contains the plugin trait and ports, as well as means to retrieve features from the host and to extend the interface of the plugin.
//!
//! # Example
//!
//! ```
//! // Import everything we need.
//! use lv2_core::prelude::*;
//! use port::inplace::*; //imports ports supporting inplace processing
//! use urid::*;
//!
//! // The input and output ports are defined by a struct which implements the `PortCollection` trait.
//! // In this case, there is an input control port for the gain of the amplification, an input audio
//! // port and an output audio port.
//! #[derive(PortCollection)]
//! struct Ports {
//!     gain: ControlInput,
//!     input: AudioInput,
//!     output: AudioOutput,
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
//!     fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
//!         let gain = ports.gain.get();
//!         let coef = if gain > -90.0 {
//!             10.0_f32.powf(gain * 0.05)
//!         } else {
//!             0.0
//!         };
//!
//!         for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter()) {
//!             out_frame.set(in_frame.get() *coef);
//!         }
//!     }
//! }
//! ```
extern crate lv2_sys as sys;

pub mod extension;
pub mod feature;
pub mod plugin;
pub mod port;
pub mod prelude;
