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
