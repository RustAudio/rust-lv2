use atom::prelude::*;
use atom::space::*;
use core::prelude::*;
use std::convert::TryFrom;
use urid::prelude::*;

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
    type ReadHandle = wmidi::MidiMessage<'a>;
    type WriteParameter = wmidi::MidiMessage<'b>;
    type WriteHandle = ();

    fn read(space: Space<'a>, _: ()) -> Option<wmidi::MidiMessage<'a>> {
        space
            .data()
            .and_then(|bytes| wmidi::MidiMessage::try_from(bytes).ok())
    }

    fn init(mut frame: FramedMutSpace<'a, 'b>, message: wmidi::MidiMessage<'b>) -> Option<()> {
        frame
            .allocate(message.bytes_size(), false)
            .and_then(|(_, space)| message.copy_to_slice(space).ok())
            .map(|_| ())
    }
}
