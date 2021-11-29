//! MIDI message extension for `lv2_atom`.
//!
//! This crate adds a new atom type for the `lv2_atom` crate: The `MidiEvent`, a message conformant to the [MIDI specification](https://www.midi.org/specifications-old/item/the-midi-1-0-specification). Due to the one-crate-per-spec policy of the `rust-lv2` project, this relatively small crate isn't integrated into the main atom crate.
//!
//! If compiled with the optional `wmidi` dependency, the crate also has an additional module containing the `WMidiEvent`. This atom uses the `MidiMessage` type defined in by `wmidi` instead of byte slices.
//!
//! # Example
//!
//! This example showcases a MIDI event processor that modulates every played note up a forth, using the `wmidi` optional dependency.
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_midi::prelude::*;
//! use lv2_units::prelude::*;
//! use wmidi::*;
//! use urid::*;
//!
//! #[derive(URIDCollection)]
//! struct MyURIDs {
//!     atom: AtomURIDCollection,
//!     midi: MidiURIDCollection,
//!     units: UnitURIDCollection,
//! }
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: MyURIDs) {
//!     // Retrieving the input and output sequence.
//!     let input_sequence = ports.input
//!         .read(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame)
//!         .unwrap();
//!
//!     let mut output_sequence = ports.output
//!         .write(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame)
//!         .unwrap();
//!
//!     for (timestamp, atom) in input_sequence {
//!         // If the atom encodes a message...
//!         if let Ok(message) = atom.read(urids.midi.wmidi) {
//!             // Calculate the message to send.
//!             let message_to_send = match message {
//!                 MidiMessage::NoteOn(channel, note, velocity) => {
//!                     let note = note.step(5).unwrap(); // modulate up five half-steps.
//!                     MidiMessage::NoteOn(channel, note, velocity)
//!                 }
//!                 MidiMessage::NoteOff(channel, note, velocity) => {
//!                     let note = note.step(5).unwrap(); // modulate up five half-steps.
//!                     MidiMessage::NoteOff(channel, note, velocity)
//!                 }
//!                 _ => message,
//!             };
//!             // Write the modulated message or forward it.
//!             output_sequence.new_event(timestamp, urids.midi.wmidi).unwrap().set(message_to_send).unwrap();
//!         } else {
//!             // Forward the other, uninterpreted message.
//!             output_sequence.forward(timestamp, atom);
//!         }
//!     }
//! }
//! ```
extern crate lv2_atom as atom;
extern crate lv2_sys as sys;

use urid::*;

pub mod raw;

#[cfg(feature = "wmidi")]
pub mod wmidi_binding;

/// Collection with the URIDs of all `UriBound`s in this crate.
pub struct MidiURIDCollection {
    pub raw: URID<raw::MidiEvent>,
    #[cfg(feature = "wmidi")]
    pub wmidi: URID<wmidi_binding::WMidiEvent>,
    #[cfg(feature = "wmidi")]
    pub sysex_wmidi: URID<wmidi_binding::SystemExclusiveWMidiEvent>,
}

impl URIDCollection for MidiURIDCollection {
    fn from_map<M: Map + ?Sized>(map: &M) -> Option<Self> {
        Some(Self {
            raw: map.map_type()?,
            #[cfg(feature = "wmidi")]
            wmidi: map.map_type()?,
            #[cfg(feature = "wmidi")]
            sysex_wmidi: map.map_type()?,
        })
    }
}

/// Prelude for wildcard use, containing many important types.
pub mod prelude {
    pub use crate::raw::MidiEvent;
    #[cfg(feature = "wmidi")]
    pub use crate::wmidi_binding::SystemExclusiveWMidiEvent;
    #[cfg(feature = "wmidi")]
    pub use crate::wmidi_binding::WMidiEvent;
    pub use crate::MidiURIDCollection;
}
