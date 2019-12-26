//! MIDI message extension for `lv2_atom`.
//!
//! This crate adds a new atom type for the `lv2_atom` crate: The `MidiEvent`, a message conformant to the [MIDI specification](https://www.midi.org/specifications-old/item/the-midi-1-0-specification). Due to the one-crate-per-spec policy of the `rust-lv2` project, this relatively small crate isn't integrated into the main atom crate.
//!
//! If compiled with the optional `wmidi` dependency, the crate also has an additional module containing the `WMidiEvent`. This atom uses the `MidiMessage` type defined in by `wmidi` instead of byte slices.
extern crate lv2_atom as atom;
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_urid as urid;

use urid::prelude::*;

pub mod raw;

#[cfg(feature = "wmidi")]
pub mod wmidi_binding;

/// Container with the URIDs of all `UriBound`s in this crate.
#[derive(URIDCache)]
pub struct MidiURIDCache {
    pub raw: URID<raw::MidiEvent>,
    #[cfg(feature = "wmidi")]
    pub wmidi: URID<wmidi_binding::WMidiEvent>,
    #[cfg(feature = "wmidi")]
    pub sysex_wmidi: URID<wmidi_binding::SystemExclusiveWMidiEvent>,
}

pub mod prelude {
    pub use crate::raw::MidiEvent;
    #[cfg(feature = "wmidi")]
    pub use crate::wmidi_binding::SystemExclusiveWMidiEvent;
    #[cfg(feature = "wmidi")]
    pub use crate::wmidi_binding::WMidiEvent;
    pub use crate::MidiURIDCache;
}
