//! Atom types to make direct usage of the `wmidi` crate.
//!
//! This is the high-level and recommended module for MIDI message handling. The contained atom type can convert the raw MIDI message to rustic enumerations and back.
//!
//! If you want to have raw, low-level access to the messages, you should use the [raw module](../raw/index.html).
use atom::prelude::*;
use std::convert::TryFrom;
use urid::*;

/// Midi event.
///
/// This atom implementation makes direct use of the `wmidi::MidiMessage` type as it's reading handle as well as it's writing parameter.
///
/// This means that the message has to be fully constructed before it can be written as an atom. For most messages this isn't a problem since they have a static size, but system exclusive messages have a dynamic size.
///
/// If you want to write system exclusive messages, you should use the [`SystemExclusiveWMidiEvent`](struct.SystemExclusiveWMidiEvent.html). It can read any message, but specializes on writing system exclusive messages with a custom writer.
pub struct WMidiEvent;

unsafe impl UriBound for WMidiEvent {
    const URI: &'static [u8] = sys::LV2_MIDI__MidiEvent;
}

impl<'a, 'b> Atom<'a, 'b> for WMidiEvent
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = wmidi::MidiMessage<'a>;
    type WriteParameter = wmidi::MidiMessage<'static>;
    type WriteHandle = ();

    fn read(space: Space<'a>, _: ()) -> Option<wmidi::MidiMessage<'a>> {
        space
            .as_bytes()
            .and_then(|bytes| wmidi::MidiMessage::try_from(bytes).ok())
    }

    fn init(mut frame: FramedMutSpace<'a, 'b>, message: wmidi::MidiMessage<'b>) -> Option<()> {
        frame
            .allocate(message.bytes_size(), false)
            .and_then(|(_, space)| message.copy_to_slice(space).ok())
            .map(|_| ())
    }
}

/// System exclusive MIDI event.
///
/// This atom is an alternative to [`WMidiEvent`](struct.WMidiEvent.html): It can only write system exclusive messages, but doesn't require the message to be already constructed in memory. Instead, it uses a writing handle similar to a chunk's writing handle.
///
/// Although it can only write system exclusive message, it can read any other message too.
pub struct SystemExclusiveWMidiEvent;

unsafe impl UriBound for SystemExclusiveWMidiEvent {
    const URI: &'static [u8] = sys::LV2_MIDI__MidiEvent;
}

impl<'a, 'b> Atom<'a, 'b> for SystemExclusiveWMidiEvent
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = wmidi::MidiMessage<'a>;
    type WriteParameter = ();
    type WriteHandle = Writer<'a, 'b>;

    fn read(space: Space<'a>, _: ()) -> Option<wmidi::MidiMessage<'a>> {
        WMidiEvent::read(space, ())
    }

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<Writer<'a, 'b>> {
        let mut writer = Writer { frame };
        writer.write::<u8>(&0xf0);
        Some(writer)
    }
}

/// System exclusive message writer.
///
/// This writing handle is similar to a chunk's `ByteWriter`: You can allocate space, write raw bytes and generic values.
///
/// The "start of system exclusive" status byte is written by [`SystemExclusiveWMidiEvent::init`](struct.SystemExclusiveWMidiEvent.html#method.init) method and the "end of system exclusive" status byte is written when the writer is dropped.
pub struct Writer<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> Writer<'a, 'b> {
    pub fn allocate(&mut self, size: usize) -> Option<&'a mut [u8]> {
        self.frame.allocate(size, false).map(|(_, slice)| slice)
    }

    pub fn write_raw(&mut self, data: &[u8]) -> Option<&'a mut [u8]> {
        self.frame.write_raw(data, false)
    }

    pub fn write<T>(&mut self, instance: &T) -> Option<&'a mut T>
    where
        T: Unpin + Copy + Send + Sync + Sized + 'static,
    {
        (&mut self.frame as &mut dyn MutSpace).write(instance, false)
    }
}

impl<'a, 'b> Drop for Writer<'a, 'b> {
    fn drop(&mut self) {
        self.write::<u8>(&0xf7);
    }
}

#[cfg(test)]
mod tests {
    use crate::wmidi_binding::*;
    use atom::space::RootMutSpace;
    use std::convert::TryFrom;
    use std::mem::size_of;
    use wmidi::*;

    #[test]
    fn test_midi_event() {
        let map = HashURIDMapper::new();
        let urid = map.map_type::<WMidiEvent>().unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);
        let reference_message =
            MidiMessage::NoteOn(Channel::Ch1, Note::A0, Velocity::try_from(125).unwrap());

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            (&mut space as &mut dyn MutSpace)
                .init(urid, reference_message.clone())
                .unwrap();
        }

        // verifying
        {
            let (header, raw_space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let header = unsafe { &*(header.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(header.type_, urid);
            assert_eq!(header.size as usize, 3);

            let (message, _) = raw_space.split_at(3);
            let message = MidiMessage::try_from(message).unwrap();
            assert_eq!(message, reference_message);
        }

        // reading
        {
            let space = Space::from_ref(raw_space.as_ref());

            let message = WMidiEvent::read(space.split_atom_body(urid).unwrap().0, ()).unwrap();
            assert_eq!(message, reference_message);
        }
    }

    #[test]
    fn test_sysex_event() {
        let map = HashURIDMapper::new();
        let urid = map.map_type::<SystemExclusiveWMidiEvent>().unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = (&mut space as &mut dyn MutSpace).init(urid, ()).unwrap();
            writer.write_raw(&[1, 2, 3, 4]);
        }

        // verifying
        {
            let (header, raw_space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let header = unsafe { &*(header.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(header.type_, urid);
            assert_eq!(header.size as usize, 6);

            let (message, _) = raw_space.split_at(6);
            assert_eq!(message, &[0xf0, 1, 2, 3, 4, 0xf7]);
        }

        // reading
        {
            let space = Space::from_ref(raw_space.as_ref());

            let message = WMidiEvent::read(space.split_atom_body(urid).unwrap().0, ()).unwrap();
            assert_eq!(
                message,
                MidiMessage::SysEx(unsafe { U7::from_bytes_unchecked(&[1, 2, 3, 4]) })
            );
        }
    }
}
