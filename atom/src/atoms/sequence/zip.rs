use crate::atoms::sequence::{SequenceIterator, SequenceUnit};
use crate::UnidentifiedAtom;

pub struct ZipSequence<'a, U: SequenceUnit> {
    first: SequenceIterator<'a, U>,
    second: SequenceIterator<'a, U>,

    last_value_from_first: Option<(U::Value, &'a UnidentifiedAtom)>,
    last_value_from_second: Option<(U::Value, &'a UnidentifiedAtom)>,
}

impl<'a, U: SequenceUnit> ZipSequence<'a, U> {
    #[inline]
    pub(crate) fn new(first: SequenceIterator<'a, U>, second: SequenceIterator<'a, U>) -> Self {
        Self {
            first,
            second,
            last_value_from_first: None,
            last_value_from_second: None,
        }
    }
}

impl<'a, U: SequenceUnit> Iterator for ZipSequence<'a, U> {
    type Item = (U::Value, &'a UnidentifiedAtom);

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_value_from_first.is_none() {
            self.last_value_from_first = self.first.next();
        }

        if self.last_value_from_second.is_none() {
            self.last_value_from_second = self.second.next()
        }

        match (
            self.last_value_from_first.map(|(t, _)| t),
            self.last_value_from_second.map(|(t, _)| t),
        ) {
            (Some(t_first), Some(t_second)) if t_first <= t_second => {
                self.last_value_from_first.take()
            }
            (Some(_), Some(_)) | (None, Some(_)) => self.last_value_from_second.take(),
            (Some(_), None) => self.last_value_from_second.take(),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::atom_prelude::{AtomWriteError, SpaceWriter};
    use crate::atoms::AtomURIDCollection;
    use crate::space::AlignedVec;
    use crate::AtomHeader;
    use lv2_units::units::Frame;
    use urid::{HashURIDMapper, Map, URID};

    fn write_ints(
        ints: &[(i64, i32)],
        buf: &mut AlignedVec<AtomHeader>,
        urids: &AtomURIDCollection,
    ) -> Result<(), AtomWriteError> {
        let mut cursor = buf.write();
        let mut sequence = cursor.write_atom(urids.sequence)?.with_frame_unit()?;

        for (timestamp, value) in ints {
            sequence.new_event(*timestamp, urids.int)?.set(*value)?;
        }

        Ok(())
    }

    #[test]
    pub fn example() {
        let mapper = HashURIDMapper::new();
        let urids = mapper.populate_collection().unwrap();
        let frame: URID<Frame> = mapper.map_type().unwrap();
        let mut first_buf = AlignedVec::<AtomHeader>::new();
        let mut second_buf = AlignedVec::<AtomHeader>::new();

        write_ints(
            &[(1, 10), (2, 20), (5, 50), (6, 60)],
            &mut first_buf,
            &urids,
        )
        .unwrap();
        write_ints(
            &[(3, 30), (4, 40), (5, 55), (8, 80)],
            &mut second_buf,
            &urids,
        )
        .unwrap();

        // SAFETY: we just wrote those atoms
        let first_seq = unsafe { first_buf.as_space().read().next_atom() }
            .unwrap()
            .read(urids.sequence)
            .unwrap()
            .with_unit(frame)
            .unwrap();

        let second_seq = unsafe { second_buf.as_space().read().next_atom() }
            .unwrap()
            .read(urids.sequence)
            .unwrap()
            .with_unit(frame)
            .unwrap();

        let ints: Vec<_> = first_seq
            .zip_sequence(second_seq)
            .map(|(t, val)| (t, *val.read(urids.int).unwrap()))
            .collect();

        assert_eq!(
            ints,
            &[
                (1, 10),
                (2, 20),
                (3, 30),
                (4, 40),
                (5, 50),
                (5, 55),
                (6, 60),
                (8, 80)
            ]
        )
    }
}
