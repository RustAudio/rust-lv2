//! A safe, fast, and ergonomic Rust library to create [LV2 plugins](http://lv2plug.in/) for audio processing or synthesis.
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

#[cfg(feature = "lv2-atom")]
pub mod atom {
    pub use lv2_atom::*;
}

#[cfg(feature = "lv2-core")]
pub mod core {
    pub use lv2_core::*;
}

#[cfg(feature = "lv2-midi")]
pub mod midi {
    pub use lv2_midi::*;
}

#[cfg(feature = "lv2-state")]
pub mod state {
    pub use lv2_state::*;
}

#[cfg(feature = "lv2-time")]
pub mod time {
    pub use lv2_time::*;
}

#[cfg(feature = "lv2-units")]
pub mod units {
    pub use lv2_units::*;
}

#[cfg(any(feature = "lv2-urid", feature = "urid"))]
pub mod urid {
    #[cfg(feature = "urid")]
    pub use urid::*;
    #[cfg(feature = "lv2-urid")]
    pub use lv2_urid::*;
}

#[cfg(feature = "lv2-worker")]
pub mod worker {
    pub use lv2_worker::*;
}

pub mod prelude {
    #[cfg(feature = "lv2-atom")]
    pub use ::lv2_atom::prelude::*;
    #[cfg(feature = "lv2-core")]
    pub use ::lv2_core::prelude::*;
    #[cfg(feature = "lv2-midi")]
    pub use ::lv2_midi::prelude::*;
    #[cfg(feature = "lv2-state")]
    pub use ::lv2_state::*;
    #[cfg(feature = "lv2-time")]
    pub use ::lv2_time::prelude::*;
    #[cfg(feature = "lv2-units")]
    pub use ::lv2_units::prelude::*;
    #[cfg(feature = "lv2-urid")]
    pub use ::lv2_urid::*;
    #[cfg(feature = "urid")]
    pub use ::urid::*;
    #[cfg(feature = "lv2-work")]
    pub use ::lv2_worker::prelude::*;
}
