//! An atom containing a sequence of time-stamped events.
//!
//! These events are atoms again. Atoms passed in a sequence can be handled with frame-perfect timing and therefore is the prefered way to transmit events like MIDI messages. However, MIDI messages are implemented in separate crate.
//!
//! # Example
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_urid::prelude::*;
//! use lv2_units::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::sequence::*;
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCache)]
//! struct MyURIDs {
//!     atom: AtomURIDCache,
//!     units: UnitURIDCache,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) {
//!     // Get the read handle to the sequence.
//!     // The reading method needs the URID of the BPM unit to tell if the time stamp
//!     // is measured in beats or in frames. If the atom doesn't says that it's measured
//!     // in beats, it is assumed that it is measured in frames.
//!     let input_sequence: SequenceIterator = ports.input.read(
//!         urids.atom.sequence,
//!         urids.units.beat
//!     ).unwrap();
//!
//!     // Get the write handle to the sequence.
//!     // You have to provide the unit of the time stamps.
//!     let mut output_sequence: SequenceWriter = ports.output.init(
//!         urids.atom.sequence,
//!         TimeStampURID::Frames(urids.units.frame)
//!     ).unwrap();
//!
//!     // Iterate through all events in the input sequence.
//!     //
//!     // The specifications don't require the time stamps to be monotonic, your algorithms should
//!     // be able to handle older events written after younger events.
//!     //
//!     // The sequence writer, however, assures that the written time stamps are monotonic.
//!     for event in input_sequence {
//!         // An event contains a timestamp and an atom.
//!         let (timestamp, atom): (TimeStamp, UnidentifiedAtom) = event;
//!         // If the read atom is a 32-bit integer...
//!         if let Some(integer) = atom.read(urids.atom.int, ()) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.init(timestamp, urids.atom.int, integer * 2).unwrap();
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
use crate::space::*;
use crate::*;
use core::prelude::*;
use sys::LV2_Atom_Event__bindgen_ty_1 as RawTimeStamp;
use units::prelude::*;
use urid::prelude::*;

/// An atom containing a sequence of time-stamped events.
///
/// [See also the module documentation.](index.html)
pub struct Sequence;

unsafe impl UriBound for Sequence {
    const URI: &'static [u8] = sys::LV2_ATOM__Sequence;
}

impl<'a, 'b> Atom<'a, 'b> for Sequence
where
    'a: 'b,
{
    type ReadParameter = URID<Beat>;
    type ReadHandle = SequenceIterator<'a>;
    type WriteParameter = TimeStampURID;
    type WriteHandle = SequenceWriter<'a, 'b>;

    fn read(body: Space, bpm_urid: URID<Beat>) -> Option<SequenceIterator> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Sequence_Body>()?;
        let unit = if header.unit == bpm_urid {
            TimeStampUnit::BeatsPerMinute
        } else {
            TimeStampUnit::Frames
        };
        Some(SequenceIterator { space: body, unit })
    }

    fn init(
        mut frame: FramedMutSpace<'a, 'b>,
        unit: TimeStampURID,
    ) -> Option<SequenceWriter<'a, 'b>> {
        {
            let frame = &mut frame as &mut dyn MutSpace;
            let header = sys::LV2_Atom_Sequence_Body {
                unit: match unit {
                    TimeStampURID::BeatsPerMinute(urid) => urid.get(),
                    TimeStampURID::Frames(urid) => urid.get(),
                },
                pad: 0,
            };
            frame.write(&header, true)?;
        }
        Some(SequenceWriter {
            frame,
            unit: unit.into(),
            last_stamp: None,
        })
    }
}

/// The measuring units of time stamps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeStampUnit {
    Frames,
    BeatsPerMinute,
}

/// An event time stamp.
#[derive(Clone, Copy, Debug)]
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
    space: Space<'a>,
    unit: TimeStampUnit,
}

impl<'a> SequenceIterator<'a> {
    pub fn unit(&self) -> TimeStampUnit {
        self.unit
    }
}

impl<'a> Iterator for SequenceIterator<'a> {
    type Item = (TimeStamp, UnidentifiedAtom<'a>);

    fn next(&mut self) -> Option<(TimeStamp, UnidentifiedAtom<'a>)> {
        let (raw_stamp, space) = self.space.split_type::<RawTimeStamp>()?;
        let stamp = match self.unit {
            TimeStampUnit::Frames => unsafe { TimeStamp::Frames(raw_stamp.frames) },
            TimeStampUnit::BeatsPerMinute => unsafe { TimeStamp::BeatsPerMinute(raw_stamp.beats) },
        };
        let (atom, space) = space.split_atom()?;
        self.space = space;
        Some((stamp, UnidentifiedAtom::new(atom)))
    }
}

/// The writing handle for sequences.
pub struct SequenceWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
    unit: TimeStampUnit,
    last_stamp: Option<TimeStamp>,
}

impl<'a, 'b> SequenceWriter<'a, 'b> {
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
        (&mut self.frame as &mut dyn MutSpace)
            .write(&raw_stamp, true)
            .map(|_| ())
    }

    /// Initialize an event.
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn init<'c, A: Atom<'a, 'c>>(
        &'c mut self,
        stamp: TimeStamp,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        self.write_time_stamp(stamp)?;
        let child_frame = (&mut self.frame as &mut dyn MutSpace).create_atom_frame(urid)?;
        A::init(child_frame, parameter)
    }

    /// Forward an unidentified atom to the sequence.
    ///
    /// If your cannot identify the type of the atom but have to write it, you can simply forward it.
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn forward(&mut self, stamp: TimeStamp, atom: UnidentifiedAtom) -> Option<()> {
        let data = atom.space.data()?;
        self.write_time_stamp(stamp)?;
        self.frame.write_raw(data, true).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::sequence::*;
    use std::mem::size_of;
    use sys::LV2_Atom_Event__bindgen_ty_1 as RawTimeStamp;
    use urid::mapper::*;

    #[derive(URIDCache)]
    struct TestURIDCache {
        atom: AtomURIDCache,
        units: UnitURIDCache,
    }

    #[test]
    fn test_sequence() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = TestURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.atom.sequence)
                .unwrap();
            let mut writer =
                Sequence::init(frame, TimeStampURID::Frames(urids.units.frame)).unwrap();

            writer
                .init::<Int>(TimeStamp::Frames(0), urids.atom.int, 42)
                .unwrap();
            writer
                .init::<Long>(TimeStamp::Frames(1), urids.atom.long, 17)
                .unwrap();
        }

        // verifying
        {
            let (sequence, space) = raw_space.split_at(size_of::<sys::LV2_Atom_Sequence>());
            let sequence = unsafe { &*(sequence.as_ptr() as *const sys::LV2_Atom_Sequence) };
            assert_eq!(sequence.atom.type_, urids.atom.sequence);
            assert_eq!(
                sequence.atom.size as usize,
                size_of::<sys::LV2_Atom_Sequence_Body>()
                    + size_of::<RawTimeStamp>()
                    + size_of::<sys::LV2_Atom_Int>()
                    + 4
                    + size_of::<RawTimeStamp>()
                    + size_of::<sys::LV2_Atom_Long>()
            );
            assert_eq!(sequence.body.unit, urids.units.frame);

            let (stamp, space) = space.split_at(size_of::<RawTimeStamp>());
            let stamp = unsafe { *(stamp.as_ptr() as *const RawTimeStamp) };
            assert_eq!(unsafe { stamp.frames }, 0);

            let (int, space) = space.split_at(size_of::<sys::LV2_Atom_Int>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Int) };
            assert_eq!(int.atom.type_, urids.atom.int);
            assert_eq!(int.atom.size as usize, size_of::<i32>());
            assert_eq!(int.body, 42);
            let (_, space) = space.split_at(4);

            let (stamp, space) = space.split_at(size_of::<RawTimeStamp>());
            let stamp = unsafe { *(stamp.as_ptr() as *const RawTimeStamp) };
            assert_eq!(unsafe { stamp.frames }, 1);

            let (int, _) = space.split_at(size_of::<sys::LV2_Atom_Long>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Long) };
            assert_eq!(int.atom.type_, urids.atom.long);
            assert_eq!(int.atom.size as usize, size_of::<i64>());
            assert_eq!(int.body, 17);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.atom.sequence).unwrap();
            let mut reader = Sequence::read(body, urids.units.beat).unwrap();

            assert_eq!(reader.unit(), TimeStampUnit::Frames);

            let (stamp, atom) = reader.next().unwrap();
            match stamp {
                TimeStamp::Frames(frames) => assert_eq!(frames, 0),
                _ => panic!("Invalid time stamp!"),
            }
            assert_eq!(atom.read::<Int>(urids.atom.int, ()).unwrap(), 42);

            let (stamp, atom) = reader.next().unwrap();
            match stamp {
                TimeStamp::Frames(frames) => assert_eq!(frames, 1),
                _ => panic!("Invalid time stamp!"),
            }
            assert_eq!(atom.read::<Long>(urids.atom.long, ()).unwrap(), 17);

            assert!(reader.next().is_none());
        }
    }
}
