//! Atom types to make direct usage of the `wmidi` crate.
//!
//! This is the high-level and recommended module for MIDI message handling. The contained atom type can convert the raw MIDI message to rustic enumerations and back.
//!
//! If you want to have raw, low-level access to the messages, you should use the [raw module](../raw/index.html).
use atom::prelude::*;
use atom::space::{AtomError, Terminated};
use atom::AtomHandle;
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

pub struct WMidiEventReadHandle;

impl<'a> AtomHandle<'a> for WMidiEventReadHandle {
    type Handle = wmidi::MidiMessage<'a>;
}

pub struct WMidiEventWriteHandle;

impl<'a> AtomHandle<'a> for WMidiEventWriteHandle {
    type Handle = WMidiEventWriter<'a>;
}

pub struct WMidiEventWriter<'a> {
    writer: AtomSpaceWriter<'a>,
}

impl<'a> WMidiEventWriter<'a> {
    #[inline]
    pub fn set(mut self, message: wmidi::MidiMessage) -> Result<(), AtomError> {
        let space = self.writer.allocate(message.bytes_size())?;

        // The error shouldn't be happening, as we allocated just as many bytes as needed
        message
            .copy_to_slice(space)
            .map_err(|_| AtomError::Unknown)?;
        Ok(())
    }
}

impl Atom for WMidiEvent {
    type ReadHandle = WMidiEventReadHandle;
    type WriteHandle = WMidiEventWriteHandle;

    #[inline]
    unsafe fn read(space: &AtomSpace) -> Result<wmidi::MidiMessage, AtomError> {
        wmidi::MidiMessage::try_from(space.as_bytes()).map_err(|_| AtomError::InvalidAtomValue {
            reading_type_uri: Self::uri(),
        })
    }

    #[inline]
    fn init(writer: AtomSpaceWriter) -> Result<WMidiEventWriter, AtomError> {
        Ok(WMidiEventWriter { writer })
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

pub struct SystemExclusiveWMidiEventWriteHandle;

impl<'a> AtomHandle<'a> for SystemExclusiveWMidiEventWriteHandle {
    type Handle = Writer<'a>;
}

impl Atom for SystemExclusiveWMidiEvent {
    type ReadHandle = WMidiEventReadHandle;
    type WriteHandle = SystemExclusiveWMidiEventWriteHandle;

    #[inline]
    unsafe fn read(space: &AtomSpace) -> Result<wmidi::MidiMessage, AtomError> {
        WMidiEvent::read(space)
    }

    fn init(mut frame: AtomSpaceWriter) -> Result<Writer, AtomError> {
        frame.write_value(0xf0u8)?;

        Ok(Writer {
            frame: frame.terminated(0xf7),
        })
    }
}

/// System exclusive message writer.
///
/// This writing handle is similar to a chunk's `ByteWriter`: You can allocate space, write raw bytes and generic values.
///
/// The "start of system exclusive" status byte is written by [`SystemExclusiveWMidiEvent::init`](struct.SystemExclusiveWMidiEvent.html#method.init) method and the "end of system exclusive" status byte is written when the writer is dropped.
pub struct Writer<'a> {
    frame: Terminated<AtomSpaceWriter<'a>>,
}

impl<'a> Writer<'a> {
    #[inline]
    pub fn write_raw(&mut self, data: &[u8]) -> Result<&mut [u8], AtomError> {
        self.frame.write_bytes(data)
    }

    #[inline]
    pub fn write<T>(&mut self, instance: T) -> Result<&mut T, AtomError>
    where
        T: Copy + Sized + 'static,
    {
        self.frame.write_value(instance)
    }
}

#[cfg(test)]
mod tests {
    use crate::wmidi_binding::*;
    use lv2_atom::space::{SpaceCursor, VecSpace};
    use lv2_atom::AtomHeader;
    use std::convert::TryFrom;
    use wmidi::*;

    #[test]
    fn test_midi_event() {
        let map = HashURIDMapper::new();
        let urid = map.map_type::<WMidiEvent>().unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();
        let reference_message =
            MidiMessage::NoteOn(Channel::Ch1, Note::A0, Velocity::try_from(125).unwrap());

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            space
                .init_atom(urid)
                .unwrap()
                .set(reference_message.clone())
                .unwrap();
        }

        // verifying
        {
            let atom = unsafe { UnidentifiedAtom::from_space(&raw_space) }.unwrap();
            assert_eq!(atom.header().urid(), urid);
            assert_eq!(atom.header().size_of_body(), 3);

            let message = &atom.body().as_bytes()[..3];
            let message = MidiMessage::try_from(message).unwrap();
            assert_eq!(message, reference_message);
        }

        // reading
        {
            let space = unsafe { UnidentifiedAtom::from_space(&raw_space) }
                .unwrap()
                .body();

            let message = unsafe { WMidiEvent::read(space) }.unwrap();
            assert_eq!(message, reference_message);
        }
    }

    #[test]
    fn test_sysex_event() {
        let map = HashURIDMapper::new();
        let urid = map.map_type::<SystemExclusiveWMidiEvent>().unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space.init_atom(urid).unwrap();
            writer.write_raw(&[1, 2, 3, 4]).unwrap();
        }

        // verifying
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();

            assert_eq!(atom.header().urid(), urid);
            assert_eq!(atom.header().size_of_body(), 6);
            assert_eq!(&atom.body().as_bytes()[..6], &[0xf0, 1, 2, 3, 4, 0xf7]);
        }

        // reading
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();
            let message = atom.read(urid).unwrap();
            assert_eq!(
                message,
                MidiMessage::SysEx(unsafe { U7::from_bytes_unchecked(&[1, 2, 3, 4]) })
            );
        }
    }
}
