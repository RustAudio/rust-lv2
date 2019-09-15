use crate::space::*;
use crate::*;
use core::UriBound;
use std::os::raw::*;
use sys::LV2_Atom_Event_Timestamp as RawTimeStamp;
use units::units::{BeatPerMinute, Frame};
use urid::{URIDBound, URID};

pub struct Sequence;

unsafe impl UriBound for Sequence {
    const URI: &'static [u8] = sys::LV2_ATOM__Sequence;
}

impl URIDBound for Sequence {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.sequence
    }
}

impl<'a, 'b> Atom<'a, 'b> for Sequence
where
    'a: 'b,
{
    type ReadParameter = URID<BeatPerMinute>;
    type ReadHandle = SequenceIterator<'a>;
    type WriteParameter = (TimeStampUnit, URID<BeatPerMinute>, URID<Frame>);
    type WriteHandle = SequenceWriter<'a, 'b>;

    fn read(body: Space, bpm_urid: URID<BeatPerMinute>) -> Option<SequenceIterator> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Sequence_Body>()?;
        let unit = if header.unit == bpm_urid {
            TimeStampUnit::BeatsPerMinute
        } else {
            TimeStampUnit::Frames
        };
        Some(SequenceIterator { space: body, unit })
    }

    fn write(
        mut frame: FramedMutSpace<'a, 'b>,
        parameter: Self::WriteParameter,
    ) -> Option<SequenceWriter<'a, 'b>> {
        let (unit, bpm_urid, frame_urid) = parameter;
        {
            let frame = &mut frame as &mut dyn MutSpace;
            let header = sys::LV2_Atom_Sequence_Body {
                unit: match unit {
                    TimeStampUnit::BeatsPerMinute => bpm_urid.get(),
                    TimeStampUnit::Frames => frame_urid.get(),
                },
                pad: 0,
            };
            frame.write(&header, true)?;
        }
        Some(SequenceWriter { frame, unit })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeStampUnit {
    Frames,
    BeatsPerMinute,
}

#[derive(Clone, Copy, Debug)]
pub enum TimeStamp {
    Frames(i64),
    BeatsPerMinute(c_double),
}

impl TimeStamp {
    pub fn unwrap_frames(self) -> i64 {
        match self {
            Self::Frames(frame) => frame,
            _ => panic!("unwrap_frames called on {:?}", self),
        }
    }

    pub fn unwrap_bpm(self) -> c_double {
        match self {
            Self::BeatsPerMinute(bpm) => bpm,
            _ => panic!("unwrap_bpm called on {:?}", self),
        }
    }
}

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

pub struct SequenceWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
    unit: TimeStampUnit,
}

impl<'a, 'b> SequenceWriter<'a, 'b> {
    pub fn write<'c, A: Atom<'a, 'c>>(
        &'c mut self,
        stamp: TimeStamp,
        urid: URID<A>,
        parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        let raw_stamp = match stamp {
            TimeStamp::Frames(frames) => {
                if self.unit == TimeStampUnit::Frames {
                    RawTimeStamp { frames }
                } else {
                    return None;
                }
            }
            TimeStamp::BeatsPerMinute(beats) => {
                if self.unit == TimeStampUnit::BeatsPerMinute {
                    RawTimeStamp { beats }
                } else {
                    return None;
                }
            }
        };
        let frame = &mut self.frame as &mut dyn MutSpace;
        frame.write(&raw_stamp, true)?;
        let child_frame = frame.create_atom_frame(urid)?;
        A::write(child_frame, parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::scalar::*;
    use crate::sequence::*;
    use std::mem::size_of;
    use units::UnitURIDCache;
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    #[derive(URIDCache)]
    struct TestURIDCache {
        atom: AtomURIDCache,
        units: UnitURIDCache,
    }

    #[test]
    fn test_sequence() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = TestURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.atom.sequence)
                .unwrap();
            let mut writer = Sequence::write(
                frame,
                (TimeStampUnit::Frames, urids.units.bpm, urids.units.frame),
            )
            .unwrap();

            writer
                .write::<Int>(TimeStamp::Frames(0), urids.atom.int, 42)
                .unwrap();
            writer
                .write::<Long>(TimeStamp::Frames(1), urids.atom.long, 17)
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
                    + size_of::<sys::LV2_Atom_Event_Timestamp>()
                    + size_of::<sys::LV2_Atom_Int>()
                    + 4
                    + size_of::<sys::LV2_Atom_Event_Timestamp>()
                    + size_of::<sys::LV2_Atom_Long>()
            );
            assert_eq!(sequence.body.unit, urids.units.frame);

            let (stamp, space) = space.split_at(size_of::<sys::LV2_Atom_Event_Timestamp>());
            let stamp = unsafe { *(stamp.as_ptr() as *const sys::LV2_Atom_Event_Timestamp) };
            assert_eq!(unsafe { stamp.frames }, 0);

            let (int, space) = space.split_at(size_of::<sys::LV2_Atom_Int>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Int) };
            assert_eq!(int.atom.type_, urids.atom.int);
            assert_eq!(int.atom.size as usize, size_of::<c_int>());
            assert_eq!(int.body, 42);
            let (_, space) = space.split_at(4);

            let (stamp, space) = space.split_at(size_of::<sys::LV2_Atom_Event_Timestamp>());
            let stamp = unsafe { *(stamp.as_ptr() as *const sys::LV2_Atom_Event_Timestamp) };
            assert_eq!(unsafe { stamp.frames }, 1);

            let (int, _) = space.split_at(size_of::<sys::LV2_Atom_Long>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Long) };
            assert_eq!(int.atom.type_, urids.atom.long);
            assert_eq!(int.atom.size as usize, size_of::<c_long>());
            assert_eq!(int.body, 17);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.atom.sequence).unwrap();
            let mut reader = Sequence::read(body, urids.units.bpm).unwrap();

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
