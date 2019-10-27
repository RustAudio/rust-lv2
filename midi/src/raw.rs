use atom::chunk::ByteWriter;
use atom::prelude::*;
use atom::space::*;
use core::prelude::*;
use urid::prelude::*;

/// Midi Event.
/// 
/// This low-level implementation is basically the same as a chunk atom: It reads a slice of bytes and writes with a `ByteWriter`.
pub struct MidiEvent;

unsafe impl UriBound for MidiEvent {
    const URI: &'static [u8] = sys::LV2_MIDI__MidiEvent;
}

impl URIDBound for MidiEvent {
    type CacheType = URID<MidiEvent>;

    fn urid(cache: &URID<MidiEvent>) -> URID<MidiEvent> {
        *cache
    }
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
