//! MIDI message extension for `lv2_atom`.
//! 
//! This crate adds a new atom type for the `lv2_atom` crate: The `MidiEvent`, a message conformant to the [MIDI specification](https://www.midi.org/specifications-old/item/the-midi-1-0-specification). Due to the one-crate-per-spec policy of the `rust-lv2` project, this relatively small crate isn't integrated into the main atom crate.
//! 
//! If compiled with the optional `wmidi` dependency, `MidiEvent` will use the `MidiMessage` type defined in by `wmidi` instead of byte slices.
extern crate lv2_atom as atom;
extern crate lv2_core as core;
pub extern crate lv2_midi_sys as sys;
extern crate lv2_urid as urid;

#[cfg(not(feature = "wmidi"))]
mod raw;
#[cfg(not(feature = "wmidi"))]
pub use raw::*;

#[cfg(feature = "wmidi")]
mod wmidi_;
#[cfg(feature = "wmidi")]
pub use wmidi_::*;
