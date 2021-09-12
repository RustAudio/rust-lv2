//! Atom types to give raw access to MIDI messages.
//!
//! This implementation is very low-level; Basically an alias for a chunk. It should only be used by those who don't want additional dependencies or want to modify messages directly.
//!
//! If you just want to use MIDI messages in your plugin, you should use the optional `wmidi` feature.
use atom::prelude::*;
use atom::space::AtomError;
use atom::AtomHandle;
use urid::*;

/// Midi Event.
///
/// This low-level implementation is basically the same as a chunk atom: It reads a slice of bytes and writes with a `ByteWriter`.
pub struct MidiEvent;

unsafe impl UriBound for MidiEvent {
    const URI: &'static [u8] = sys::LV2_MIDI__MidiEvent;
}

pub struct MidiEventReadHandle;

impl<'a> AtomHandle<'a> for MidiEventReadHandle {
    type Handle = &'a [u8];
}

pub struct MidiEventWriteHandle;

impl<'a> AtomHandle<'a> for MidiEventWriteHandle {
    type Handle = AtomSpaceWriter<'a>;
}

impl Atom for MidiEvent {
    type ReadHandle = MidiEventReadHandle;
    type WriteHandle = MidiEventWriteHandle;

    unsafe fn read(body: &AtomSpace) -> Result<&[u8], AtomError> {
        Ok(body.as_bytes())
    }

    fn init(frame: AtomSpaceWriter) -> Result<AtomSpaceWriter, AtomError> {
        Ok(frame)
    }
}
