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
//!     let input_sequence: SequenceIterator<Frame> = ports.input
//!         .read(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame).unwrap();
//!
//!     // Get the write handle to the sequence.
//!     // You have to provide the unit of the time stamps.
//!     let mut output_sequence: SequenceWriter<Frame> = ports.output.init(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame)
//!         .unwrap();
//!
//!     // Iterate through all events in the input sequence.
//!     //
//!     // The specifications don't require the time stamps to be monotonic, your algorithms should
//!     // be able to handle older events written after younger events.
//!     //
//!     // The sequence writer, however, assures that the written time stamps are monotonic.
//!     for event in input_sequence {
//!         // An event contains a timestamp and an atom.
//!         let (timestamp, atom): (i64, &UnidentifiedAtom) = event;
//!         // If the read atom is a 32-bit integer...
//!         if let Ok(integer) = atom.read(urids.atom.int) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.new_event(timestamp, urids.atom.int).unwrap().set(*integer * 2).unwrap();
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
mod unit;

use crate::space::SpaceReader;
use crate::*;
use std::marker::PhantomData;
use sys::LV2_Atom_Event__bindgen_ty_1 as RawTimeStamp;
pub use unit::*;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct SequenceBody(sys::LV2_Atom_Sequence_Body);

/// An atom containing a sequence of time-stamped events.
///
/// [See also the module documentation.](index.html)
pub struct Sequence;

unsafe impl UriBound for Sequence {
    const URI: &'static [u8] = sys::LV2_ATOM__Sequence;
}

pub struct SequenceReadHandle;
impl<'a> AtomHandle<'a> for SequenceReadHandle {
    type Handle = SequenceHeaderReader<'a>;
}

pub struct SequenceWriteHandle;
impl<'a> AtomHandle<'a> for SequenceWriteHandle {
    type Handle = SequenceHeaderWriter<'a>;
}

pub struct SequenceHeaderReader<'a> {
    header: &'a sys::LV2_Atom_Sequence_Body,
    reader: SpaceReader<'a>,
}

impl<'a> SequenceHeaderReader<'a> {
    pub fn with_unit<U: SequenceUnit>(
        self,
        unit_urid: URID<U>,
    ) -> Result<SequenceIterator<'a, U>, AtomReadError> {
        if (self.header.unit == 0 && U::TYPE == SequenceUnitType::Frame)
            || (self.header.unit == unit_urid)
        {
            Ok(SequenceIterator {
                reader: self.reader,
                unit_type: PhantomData,
            })
        } else {
            Err(AtomReadError::InvalidUrid {
                expected_uri: U::uri(),
                expected_urid: unit_urid.into_general(),
                found_urid: self.header.unit,
            })
        }
    }
}

pub struct SequenceHeaderWriter<'a> {
    writer: AtomSpaceWriter<'a>,
}

impl<'a> SequenceHeaderWriter<'a> {
    pub fn with_unit<U: SequenceUnit>(
        mut self,
        unit_urid: URID<U>,
    ) -> Result<SequenceWriter<'a, U>, AtomWriteError> {
        let header = SequenceBody(sys::LV2_Atom_Sequence_Body {
            unit: unit_urid.get(),
            pad: 0,
        });

        self.writer.write_value(header)?;

        Ok(SequenceWriter {
            writer: self.writer,
            last_stamp: None,
        })
    }
}

impl Atom for Sequence {
    type ReadHandle = SequenceReadHandle;
    type WriteHandle = SequenceWriteHandle;

    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Sequence_Body = reader.next_value()?;

        Ok(SequenceHeaderReader { reader, header })
    }

    fn init(
        frame: AtomSpaceWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Ok(SequenceHeaderWriter { writer: frame })
    }
}

/// An iterator over all events in a sequence.
pub struct SequenceIterator<'a, U: SequenceUnit> {
    reader: SpaceReader<'a>,
    unit_type: PhantomData<U>,
}

impl<'a, U: SequenceUnit> Iterator for SequenceIterator<'a, U> {
    type Item = (U::Value, &'a UnidentifiedAtom);

    fn next(&mut self) -> Option<(U::Value, &'a UnidentifiedAtom)> {
        self.reader
            .try_read(|reader| {
                // SAFETY: The validity of the space's contents is guaranteed by this type.
                let raw_stamp: &RawTimeStamp = unsafe { reader.next_value()? };

                // SAFETY: The validity of the unit type is guaranteed by this type.
                let stamp = unsafe { U::convert_from_raw(*raw_stamp) };

                // SAFETY: The validity of the space's contents is guaranteed by this type.
                let atom = unsafe { reader.next_atom()? };

                Ok((stamp, atom))
            })
            .ok()
    }
}

/// The writing handle for sequences.
pub struct SequenceWriter<'a, U: SequenceUnit> {
    writer: AtomSpaceWriter<'a>,
    last_stamp: Option<U::Value>,
}

impl<'a, U: SequenceUnit> SequenceWriter<'a, U> {
    /// Write out the time stamp and update `last_stamp`.
    ///
    /// # Errors
    ///
    /// This method returns an error if either:
    /// * The last time stamp is younger than the time stamp.
    /// * Space is insufficient.
    fn write_time_stamp(&mut self, time_stamp: U::Value) -> Result<(), AtomWriteError> {
        if let Some(last_stamp) = self.last_stamp {
            if last_stamp > time_stamp {
                return Err(AtomWriteError::WritingIllegalState {
                    writing_type_uri: Sequence::uri(),
                });
            }
        }

        self.last_stamp = Some(time_stamp);
        self.writer.write_value(U::convert_into_raw(time_stamp))?;

        Ok(())
    }

    /// Initialize an event.       
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn new_event<A: Atom>(
        &mut self,
        time_stamp: U::Value,
        urid: URID<A>,
    ) -> Result<<A::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        self.write_time_stamp(time_stamp)?;
        self.writer.init_atom(urid)
    }

    /// Forward an unidentified atom to the sequence.
    ///
    /// If your cannot identify the type of the atom but have to write it, you can simply forward it.
    ///
    /// The time stamp has to be measured in the unit of the sequence. If the time stamp is measured in the wrong unit, is younger than the last written time stamp or space is insufficient, this method returns `None`.
    pub fn forward(
        &mut self,
        time_stamp: U::Value,
        atom: &UnidentifiedAtom,
    ) -> Result<(), AtomWriteError> {
        self.write_time_stamp(time_stamp)?;

        self.writer.forward_atom(atom)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::sequence::*;
    use crate::prelude::*;
    use std::mem::size_of;
    use units::UnitURIDCollection;

    #[derive(URIDCollection)]
    struct TestURIDCollection {
        atom: AtomURIDCollection,
        units: UnitURIDCollection,
    }

    #[test]
    fn test_sequence() {
        let map = HashURIDMapper::new();
        let urids: TestURIDCollection = TestURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space
                .init_atom(urids.atom.sequence)
                .unwrap()
                .with_unit(urids.units.frame)
                .unwrap();

            writer
                .new_event(0, urids.atom.int)
                .unwrap()
                .set(42)
                .unwrap();

            writer
                .new_event(1, urids.atom.long)
                .unwrap()
                .set(17)
                .unwrap();
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
                .with_unit(urids.units.frame)
                .unwrap();

            let (stamp, atom) = reader.next().unwrap();
            assert_eq!(stamp, 0);
            assert_eq!(*atom.read::<Int>(urids.atom.int).unwrap(), 42);

            let (stamp, atom) = reader.next().unwrap();
            assert_eq!(stamp, 1);
            assert_eq!(*atom.read::<Long>(urids.atom.long).unwrap(), 17);

            assert!(reader.next().is_none());
        }
    }
}
