/// Atom types to give raw access to MIDI messages.
///
/// This implementation is very low-level; Basically an alias for a chunk. It should only be used by those who don't want additional dependencies or want to modify messages directly.
///
/// If you just want to use MIDI messages in your plugin, you should use the optional `wmidi` feature.
use atom::chunk::ByteWriter;
use atom::prelude::*;
use atom::space::*;
use core::prelude::*;

/// Midi Event.
///
/// This low-level implementation is basically the same as a chunk atom: It reads a slice of bytes and writes with a `ByteWriter`.
pub struct MidiEvent;

unsafe impl UriBound for MidiEvent {
    const URI: &'static [u8] = sys::LV2_MIDI__MidiEvent;
}

impl<'a, 'b> Atom<'a, 'b> for MidiEvent
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = &'a [u8];
    type WriteParameter = ();
    type WriteHandle = ByteWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<&'a [u8]> {
        body.data()
    }

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<ByteWriter<'a, 'b>> {
        Some(ByteWriter::new(frame))
    }
}
