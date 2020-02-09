//! An atom containg a series of other atoms.
//!
//! This atom is just like a [sequence](../sequence/index.html), only without time stamps: It contains multiple arbitrary atoms which you can either iterate through or write in sequence.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_urid::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::tuple::{TupleIterator, TupleWriter};
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCache) {
//!     let input: TupleIterator = ports.input.read(urids.tuple, ()).unwrap();
//!     let mut output: TupleWriter = ports.output.init(urids.tuple, ()).unwrap();
//!     for atom in input {
//!         if let Some(integer) = atom.read(urids.int, ()) {
//!             output.init(urids.int, integer * 2).unwrap();
//!         } else {
//!             output.init(urids.int, -1).unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Tuple](http://lv2plug.in/ns/ext/atom/atom.html#Tuple)
use crate::space::*;
use crate::*;
use core::prelude::*;
use urid::prelude::*;

/// An atom  containing a series of other atoms.
///
/// [See also the module documentation.](index.html)
pub struct Tuple;

unsafe impl UriBound for Tuple {
    const URI: &'static [u8] = sys::LV2_ATOM__Tuple;
}

impl<'a, 'b> Atom<'a, 'b> for Tuple
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = TupleIterator<'a>;
    type WriteParameter = ();
    type WriteHandle = TupleWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<TupleIterator<'a>> {
        Some(TupleIterator { space: body })
    }

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<TupleWriter<'a, 'b>> {
        Some(TupleWriter { frame })
    }
}

/// An iterator over all atoms in a tuple.
///
/// The item of this iterator is simply the space a single atom occupies.
pub struct TupleIterator<'a> {
    space: Space<'a>,
}

impl<'a> Iterator for TupleIterator<'a> {
    type Item = UnidentifiedAtom<'a>;

    fn next(&mut self) -> Option<UnidentifiedAtom<'a>> {
        let (atom, space) = self.space.split_atom()?;
        self.space = space;
        Some(UnidentifiedAtom::new(atom))
    }
}

/// The writing handle to add atoms to a tuple.
pub struct TupleWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>,
}

impl<'a, 'b> TupleWriter<'a, 'b> {
    /// Initialize a new tuple element.
    pub fn init<'c, A: Atom<'a, 'c>>(
        &'c mut self,
        child_urid: URID<A>,
        child_parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        let child_frame = (&mut self.frame as &mut dyn MutSpace).create_atom_frame(child_urid)?;
        A::init(child_frame, child_parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use std::mem::size_of;
    use urid::mapper::*;
    use urid::prelude::*;

    #[test]
    fn test_tuple() {
        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.tuple)
                .unwrap();
            let mut writer = Tuple::init(frame, ()).unwrap();
            {
                let mut vector_writer =
                    writer.init::<Vector<Int>>(urids.vector, urids.int).unwrap();
                vector_writer.append(&[17; 9]).unwrap();
            }
            writer.init::<Int>(urids.int, 42).unwrap();
        }

        // verifying
        {
            let (atom, space) = raw_space.split_at(size_of::<sys::LV2_Atom>());
            let atom = unsafe { &*(atom.as_ptr() as *const sys::LV2_Atom) };
            assert_eq!(atom.type_, urids.tuple);
            assert_eq!(
                atom.size as usize,
                size_of::<sys::LV2_Atom_Vector>()
                    + size_of::<i32>() * 9
                    + 4
                    + size_of::<sys::LV2_Atom_Int>()
            );

            let (vector, space) = space.split_at(size_of::<sys::LV2_Atom_Vector>());
            let vector = unsafe { &*(vector.as_ptr() as *const sys::LV2_Atom_Vector) };
            assert_eq!(vector.atom.type_, urids.vector);
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * 9
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int);

            let (vector_items, space) = space.split_at(size_of::<i32>() * 9);
            let vector_items =
                unsafe { std::slice::from_raw_parts(vector_items.as_ptr() as *const i32, 9) };
            assert_eq!(vector_items, &[17; 9]);
            let (_, space) = space.split_at(4);

            let (int, _) = space.split_at(size_of::<sys::LV2_Atom_Int>());
            let int = unsafe { &*(int.as_ptr() as *const sys::LV2_Atom_Int) };
            assert_eq!(int.atom.type_, urids.int);
            assert_eq!(int.atom.size as usize, size_of::<i32>());
            assert_eq!(int.body, 42);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.tuple).unwrap();
            let items: Vec<UnidentifiedAtom> = Tuple::read(body, ()).unwrap().collect();
            assert_eq!(items[0].read(urids.vector, urids.int).unwrap(), [17; 9]);
            assert_eq!(items[1].read(urids.int, ()).unwrap(), 42);
        }
    }
}
