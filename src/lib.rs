//! A safe, fast, and ergonomic Rust library to create [LV2 plugins](http://lv2plug.in/) for audio processing or synthesis,
//! on any platform.
//!
//! **This library is a work in progress.**
//!
//! It provides the following features, through the [LV2 Core specification](http://lv2plug.in/ns/lv2core/lv2core.html):
//!
//! * Lightweight, realtime non-blocking and allocation-free audio processing.
//! * Generates all the boilerplate to make a LV2 plugin binary, usable by any LV2 host on any platform.
//! * Any number of ports / Any channel mapping, which can be different for input and output.
//!   This obviously includes Mono, Stereo, Surround, etc., anything your CPU can handle.
//! * Can be extended to support any additional features, extensions and port types.
//!   They can be official, unofficial or completely custom.
//!
//! Through the [LV2 official additional specifications](http://lv2plug.in/ns/), this library also provide many
//! additional features, including:
//!
//! * MIDI processing
//! * Custom Graphical User Interfaces, both in a toolkit-agnostic and in a platform-agnostic way **(Not yet implemented)**
//! * Serialization of custom data structures, and plugin-plugin or plugin-GUI communication and property manipulation
//! * Presets handling and State management **(Not yet implemented)**
//! * Asynchronous work processing **(Not yet implemented)**
//! * … and more! (Not yet implemented either)
//!
//! Note that this library will only provide Rust bindings for the official LV2 specifications, however it is compatible
//! with any other arbitrary or custom specification, and other, external crates are able and welcome to provide Rust bindings
//! to any other specification that will integrate with this library.
//!
//! This specific crate actually does nothing but re-export the several sub-crates of this library,
//! each of which correspond to a specific LV2 official specification.
//!
//! This crate is provided only for convenience (like in examples or prototypes), however in
//! production code we recommend relying on each sub-crate you really need, to reduce the number
//! of dependencies to be built, as well as the final binary size.
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.

#![deny(missing_docs)]

pub extern crate lv2_atom as atom;
pub extern crate lv2_core as core;
pub extern crate lv2_midi as midi;
pub extern crate lv2_units as units;
pub extern crate lv2_urid as urid;
