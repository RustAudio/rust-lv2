//! An atom containing a sequence of time-stamped events.
//!
//! These events are atoms again. Atoms passed in a sequence can be handled with frame-perfect timing and therefore is the prefered way to transmit events like MIDI messages. However, MIDI messages are implemented in separate crate.
//!
//! # Example
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_units::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::atoms::sequence::*;
//! use urid::*;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCollection)]
//! struct MyURIDs {
//!     atom: AtomURIDCollection,
//!     units: UnitURIDCollection,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) {
//!     // Get the read handle to the sequence.
//!     // The reading method needs the URID of the BPM unit to tell if the time stamp
//!     // is measured in beats or in frames. If the atom doesn't says that it's measured
//!     // in beats, it is assumed that it is measured in frames.
//!     let input_sequence: SequenceIterator = ports.input
//!         .read(urids.atom.sequence)
//!         .unwrap().read(urids.units.beat);
//!
//!     // Get the write handle to the sequence.
//!     // You have to provide the unit of the time stamps.
//!     let mut output_sequence: SequenceWriter = ports.output.init(
//!         urids.atom.sequence,
//!     ).unwrap().with_unit(urids.units.beat);
//!
//!     // Iterate through all events in the input sequence.
//!     //
//!     // The specifications don't require the time stamps to be monotonic, your algorithms should
//!     // be able to handle older events written after younger events.
//!     //
//!     // The sequence writer, however, assures that the written time stamps are monotonic.
//!     for event in input_sequence {
//!         // An event contains a timestamp and an atom.
//!         let (timestamp, atom): (TimeStamp, &UnidentifiedAtom) = event;
//!         // If the read atom is a 32-bit integer...
//!         if let Some(integer) = atom.read(urids.atom.int, ()) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.init(timestamp, urids.atom.int).unwrap().set(integer * 2);
//!         } else {
//!             // Forward the atom to the sequence without a change.
//!             output_sequence.forward(timestamp, atom).unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Sequence](http://lv2plug.in/ns/ext/atom/atom.html#Sequence)
use crate::space::reader::SpaceReader;
use crate::*;
use std::mem::MaybeUninit;
use sys::LV2_Atom_Event__bindgen_ty_1 as RawTimeStamp;
use units::prelude::*;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct SequenceBody(sys::LV2_Atom_Sequence_Body);

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct TimestampBody(RawTimeStamp);

/// An atom containing a sequence of time-stamped events.
///
/// [See also the module documentation.](index.html)
pub struct Sequence;

unsafe impl UriBound for Sequence {
    const URI: &'static [u8] = sys::LV2_ATOM__Sequence;
}

struct SequenceReadHandle;
impl<'handle> AtomHandle<'handle> for SequenceReadHandle {
    type Handle = SequenceHeaderReader<'handle>;
}

struct SequenceWriteHandle;
impl<'handle> AtomHandle<'handle> for SequenceWriteHandle {
    type Handle = SequenceHeaderWriter<'handle>;
}

pub struct SequenceHeaderReader<'handle> {
    header: &'handle sys::LV2_Atom_Sequence_Body,
    reader: SpaceReader<'handle>,
}

impl<'handle> SequenceHeaderReader<'handle> {
    pub fn read(self, bpm_urid: URID<Beat>) -> SequenceIterator<'handle> {
        let unit = if self.header.unit == bpm_urid {
            TimeStampUnit::BeatsPerMinute
        } else {
            TimeStampUnit::Frames
        };

        SequenceIterator {
            reader: self.reader,
            unit,
        }
    }
}

pub struct SequenceHeaderWriter<'handle> {
    header: &'handle mut MaybeUninit<SequenceBody>,
    frame: AtomSpaceWriter<'handle>,
}

impl<'a> SequenceHeaderWriter<'a> {
    pub fn with_unit(mut self, unit: TimeStampURID) -> SequenceWriter<'a> {
        let header = SequenceBody(sys::LV2_Atom_Sequence_Body {
            unit: match unit {
                TimeStampURID::BeatsPerMinute(urid) => urid.get(),
                TimeStampURID::Frames(urid) => urid.get(),
            },
            pad: 0,
        });

        crate::util::write_uninit(&mut self.header, header);

        SequenceWriter {
            frame: self.frame,
            unit: unit.into(),
            last_stamp: None,
        }
    }
}

impl Atom for Sequence {
    type ReadHandle = SequenceReadHandle;
    type WriteHandle = SequenceWriteHandle;

    unsafe fn read<'handle, 'space: 'handle>(
        body: &'space AtomSpace,
    ) -> Option<<Self::ReadHandle as AtomHandle<'handle>>::Handle> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Sequence_Body = reader.next_value()?;

        Some(SequenceHeaderReader { reader, header })
    }

    fn init(mut frame: AtomSpaceWriter) -> Option<<Self::WriteHandle as AtomHandle>::Handle> {
        let header = frame.write_value(MaybeUninit::uninit())?;

        Some(SequenceHeaderWriter { header, frame })
    }
}

/// The measuring units of time stamps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeStampUnit {
    Frames,
    BeatsPerMinute,
}

/// An event time stamp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeStamp {
    Frames(i64),
    BeatsPerMinute(f64),
}

/// The measuring units of time stamps, with their URIDs.
#[derive(Clone, Copy)]
pub enum TimeStampURID {
    Frames(URID<Frame>),
    BeatsPerMinute(URID<Beat>),
}

impl From<TimeStampURID> for TimeStampUnit {
    fn from(urid: TimeStampURID) -> TimeStampUnit {
        match urid {
            TimeStampURID::Frames(_) => TimeStampUnit::Frames,
            TimeStampURID::BeatsPerMinute(_) => TimeStampUnit::BeatsPerMinute,
        }
    }
}

impl TimeStamp {
    pub fn as_frames(self) -> Option<i64> {
        match self {
            Self::Frames(frame) => Some(frame),
            _ => None,
        }
    }

    pub fn as_bpm(self) -> Option<f64> {
        match self {
            Self::BeatsPerMinute(bpm) => Some(bpm),
            _ => None,
        }
    }
}

/// An iterator over all events in a sequence.
pub struct SequenceIterator<'a> {
    reader: SpaceReader<'a>,
    unit: TimeStampUnit,
}

impl<'a> SequenceIterator<'a> {
    pub fn unit(&self) -> TimeStampUnit {
        self.unit
    }
}

impl<'a> Iterator for SequenceIterator<'a> {
    type Item = (TimeStamp, &'a UnidentifiedAtom);

    fn next(&mut self) -> Option<(TimeStamp, &'a UnidentifiedAtom)> {
        let unit = self.unit;

        self.reader.try_read(|reader| {
            // SAFETY: The validity of the space's contents is guaranteed by this type.
            let raw_stamp: &RawTimeStamp = unsafe { reader.next_value()? };

            let stamp = match unit {
                TimeStampUnit::Frames => unsafe { TimeStamp::Frames(raw_stamp.frames) },
                TimeStampUnit::BeatsPerMinute => unsafe {
                    TimeStamp::BeatsPerMinute(raw_stamp.beats)
                },
            };

            // SAFETY: The validity of the space's contents is guaranteed by this type.
            let atom = unsafe { reader.next_atom()? };

            Some((stamp, atom))
        })
    }
}

/// The writing handle for sequences.
pub struct SequenceWriter<'handle> {
    frame: AtomSpaceWriter<'handle>,
    unit: TimeStampUnit,
    last_stamp: Option<TimeStamp>,
}

impl<'handle> SequenceWriter<'handle> {
    /// Write out the time stamp and update `last_stamp`.
    ///
    /// This method returns `Ǹone` if:
    /// * The time stamp is not measured in our unit.
    /// * The last time stamp is younger than the time stamp.
    /// * Space is insufficient.
    fn write_time_stamp(&mut self, stamp: TimeStamp) -> Option<()> {
        let raw_stamp = match self.unit {
            TimeStampUnit::Frames => {
                let frames = stamp.as_frames()?;
                if let Some(last_stamp) = self.last_stamp {
                    if last_stamp.as_frames().unwrap() > frames {
                        return None;
                    }
                }
                RawTimeStamp { frames }
            }
            TimeStampUnit::BeatsPerMinute => {
                let beats = stamp.as_bpm()?;
                if let Some(last_stamp) = self.last_stamp {
                    if last_stamp.as_bpm().unwrap() > beats {
                        return None;
                    }
                }
                RawTimeStamp { beats }
            }
        };
        self.last_stamp = Some(stamp);
        self.frame.write_value(TimestampBody(raw_stamp))?;

        Some(())
    }

    /// Initialize an event.       
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn init<A: Atom>(
        &mut self,
        stamp: TimeStamp,
        urid: URID<A>,
    ) -> Option<<A::WriteHandle as AtomHandle>::Handle> {
        self.write_time_stamp(stamp)?;
        self.frame.init_atom(urid)
    }

    /// Forward an unidentified atom to the sequence.
    ///
    /// If your cannot identify the type of the atom but have to write it, you can simply forward it.
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn forward(&mut self, stamp: TimeStamp, atom: &UnidentifiedAtom) -> Option<()> {
        self.write_time_stamp(stamp)?;

        self.frame.forward_atom(atom)?;

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::sequence::*;
    use crate::prelude::*;
    use std::mem::size_of;

    #[derive(URIDCollection)]
    struct TestURIDCollection {
        atom: AtomURIDCollection,
        units: UnitURIDCollection,
    }

    #[test]
    fn test_sequence() {
        let map = HashURIDMapper::new();
        let urids = TestURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space
                .init_atom(urids.atom.sequence)
                .unwrap()
                .with_unit(TimeStampURID::Frames(urids.units.frame));

            writer
                .init::<Int>(TimeStamp::Frames(0), urids.atom.int)
                .unwrap()
                .set(42);

            writer
                .init::<Long>(TimeStamp::Frames(1), urids.atom.long)
                .unwrap()
                .set(17);
        }

        // verifying

        {
            let mut reader = raw_space.read();
            let sequence: &sys::LV2_Atom_Sequence = unsafe { reader.next_value() }.unwrap();
            assert_eq!(sequence.atom.type_, urids.atom.sequence);
            assert_eq!(
                sequence.atom.size as usize,
                size_of::<sys::LV2_Atom_Sequence_Body>()
                    + size_of::<RawTimeStamp>()
                    + size_of::<sys::LV2_Atom_Int>() // Int struct Includes padding
                    + size_of::<RawTimeStamp>()
                    + size_of::<sys::LV2_Atom_Long>()
            );
            assert_eq!(sequence.body.unit, urids.units.frame);

            let stamp: &RawTimeStamp = unsafe { reader.next_value() }.unwrap();
            assert_eq!(unsafe { stamp.frames }, 0);

            let int: &sys::LV2_Atom_Int = unsafe { reader.next_value() }.unwrap();
            assert_eq!(int.atom.type_, urids.atom.int);
            assert_eq!(int.atom.size as usize, 2 * size_of::<i32>());
            assert_eq!(int.body, 42);

            let stamp: &RawTimeStamp = unsafe { reader.next_value() }.unwrap();
            assert_eq!(unsafe { stamp.frames }, 1);

            let int: &sys::LV2_Atom_Int = unsafe { reader.next_value() }.unwrap();
            assert_eq!(int.atom.type_, urids.atom.long);
            assert_eq!(int.atom.size as usize, size_of::<i64>());
            assert_eq!(int.body, 17);
        }

        // reading
        {
            let mut reader = unsafe { raw_space.read().next_atom() }
                .unwrap()
                .read(urids.atom.sequence)
                .unwrap()
                .read(urids.units.beat);

            assert_eq!(reader.unit(), TimeStampUnit::Frames);

            let (stamp, atom) = reader.next().unwrap();
            assert_eq!(stamp, TimeStamp::Frames(0));
            assert_eq!(atom.read::<Int>(urids.atom.int).unwrap(), 42);

            let (stamp, atom) = reader.next().unwrap();
            assert_eq!(stamp, TimeStamp::Frames(1));
            assert_eq!(atom.read::<Long>(urids.atom.long).unwrap(), 17);

            assert!(reader.next().is_none());
        }
    }
}
