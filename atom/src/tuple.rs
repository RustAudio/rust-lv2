use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use urid::{URIDBound, URID};

/// An atom of variable length, containing a series of other atoms.
///
/// The body of this atom is simply a series of complete atoms, as [specified](http://lv2plug.in/ns/ext/atom/atom.html#Tuple).
pub struct Tuple;

unsafe impl UriBound for Tuple {
    const URI: &'static [u8] = sys::LV2_ATOM__Tuple;
}

impl URIDBound for Tuple {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.tuple
    }
}

impl Tuple {
    /// Create an iterator over all atoms in this tuple.
    ///
    /// The item of the returned iterator simply is a space that contains a complete atom. The other return value is the space behind the tuple atom.
    ///
    /// The method returns `None` if the passed space does not contain a tuple or the space does not have the right size.
    pub fn read<'a>(
        space: Space<'a>,
        urids: &AtomURIDCache,
    ) -> Option<(TupleIterator<'a>, Space<'a>)> {
        space
            .split_atom_body(urids.tuple)
            .map(|(body, space)| (TupleIterator { space: body }, space))
    }

    /// Create a frame for contained atoms.
    ///
    /// The way you write atoms to the tuple is quite simply: Create the frame and write your desired atoms to it, one by another.
    pub fn write<'a, 'b>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
    ) -> Option<FramedMutSpace<'a, 'b>> {
        space.create_atom_frame(urids.tuple)
    }
}

/// An iterator over all atoms in a tuple.
///
/// The item of this iterator is simply the space a single atom occupies.
pub struct TupleIterator<'a> {
    space: Space<'a>,
}

impl<'a> Iterator for TupleIterator<'a> {
    type Item = Space<'a>;

    fn next(&mut self) -> Option<Space<'a>> {
        let (atom, space) = self.space.split_atom()?;
        self.space = space;
        Some(atom)
    }
}

#[cfg(test)]
mod tests {
    use crate::scalar::*;
    use crate::tuple::*;
    use crate::vector::Vector;
    use std::mem::size_of;
    use std::os::raw::*;
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    #[test]
    fn test_tuple() {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut tuple_frame = Tuple::write(&mut space, &urids).unwrap();
            {
                let mut vector_writer =
                    Vector::write::<Int>(&mut tuple_frame, &urids, &urids).unwrap();
                vector_writer.append(&[17; 9]).unwrap();
            }
            Int::write(&mut tuple_frame, 42, &urids).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, urids.tuple);
            assert_eq!(
                atom.size as usize,
                size_of::<sys::LV2_Atom_Vector>()
                    + size_of::<c_int>() * 9
                    + 4
                    + size_of::<sys::LV2_Atom_Int>()
            );

            let (vector, space) = space.split_at(size_of::<sys::LV2_Atom_Vector>());
            let vector = unsafe { &*(vector.as_ptr() as *const sys::LV2_Atom_Vector) };
            assert_eq!(vector.atom.type_, urids.vector);
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<c_int>() * 9
            );
            assert_eq!(vector.body.child_size as usize, size_of::<c_int>());
            assert_eq!(vector.body.child_type, urids.int);

            let (vector_items, space) = space.split_at(size_of::<c_int>() * 9);
            let vector_items =
                unsafe { std::slice::from_raw_parts(vector_items.as_ptr() as *const c_int, 9) };
            assert_eq!(vector_items, &[17; 9]);
            let (_, space) = space.split_at(4);

            let (int, _) = space.split_at(size_of::<sys::LV2_Atom_Int>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Int) };
            assert_eq!(int.atom.type_, urids.int);
            assert_eq!(int.atom.size as usize, size_of::<c_int>());
            assert_eq!(int.body, 42);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let items: Vec<Space> = Tuple::read(space, &urids).unwrap().0.collect();
            assert_eq!(
                Vector::read::<Int>(items[0], &urids, &urids).unwrap().0,
                [17; 9]
            );
            assert_eq!(Int::read(items[1], &urids).unwrap().0, 42);
        }
    }
}
