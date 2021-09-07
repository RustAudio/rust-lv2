//! An atom containg a series of other atoms.
//!
//! This atom is just like a [sequence](../sequence/index.html), only without time stamps: It contains multiple arbitrary atoms which you can either iterate through or write in sequence.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::atoms::tuple::{TupleIterator, TupleWriter};
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
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
use crate::space::reader::SpaceReader;
use crate::*;

/// An atom  containing a series of other atoms.
///
/// [See also the module documentation.](index.html)
pub struct Tuple;

unsafe impl UriBound for Tuple {
    const URI: &'static [u8] = sys::LV2_ATOM__Tuple;
}

impl<'handle, 'space: 'handle> Atom<'handle, 'space> for Tuple {
    type ReadParameter = ();
    type ReadHandle = TupleIterator<'handle>;
    type WriteParameter = ();
    type WriteHandle = TupleWriter<'handle, 'space>;

    unsafe fn read(body: &'space Space, _: ()) -> Option<TupleIterator<'space>> {
        Some(TupleIterator {
            reader: body.read(),
        })
    }

    fn init(
        frame: AtomSpaceWriter<'handle, 'space>,
        _: (),
    ) -> Option<TupleWriter<'handle, 'space>> {
        Some(TupleWriter { frame })
    }
}

/// An iterator over all atoms in a tuple.
///
/// The item of this iterator is simply the space a single atom occupies.
pub struct TupleIterator<'a> {
    reader: SpaceReader<'a>,
}

impl<'a> Iterator for TupleIterator<'a> {
    type Item = &'a UnidentifiedAtom;

    fn next(&mut self) -> Option<&'a UnidentifiedAtom> {
        // SAFETY: the validity of the given space is guaranteed by this type.
        unsafe { self.reader.next_atom() }
    }
}

/// The writing handle to add atoms to a tuple.
pub struct TupleWriter<'handle, 'space> {
    frame: AtomSpaceWriter<'handle, 'space>,
}

impl<'handle, 'space> TupleWriter<'handle, 'space> {
    /// Initialize a new tuple element.
    pub fn init<'a, A: Atom<'a, 'space>>(
        &'a mut self,
        child_urid: URID<A>,
        child_parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        self.frame.init_atom(child_urid, child_parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_tuple() {
        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut cursor = raw_space.write();
            let mut writer = cursor.init_atom(urids.tuple, ()).unwrap();
            {
                let mut vector_writer = writer.init(urids.vector, urids.int).unwrap();
                vector_writer.append(&[17; 9]).unwrap();
            }
            writer.init(urids.int, 42).unwrap();
        }

        // verifying
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();
            let header = atom.header();
            assert_eq!(header.urid(), urids.tuple);
            assert_eq!(
                header.size_of_body(),
                size_of::<sys::LV2_Atom_Vector>()
                    + size_of::<i32>() * 9
                    + 4
                    + size_of::<sys::LV2_Atom_Int>()
            );

            let mut reader = atom.body().read();
            let vector: &sys::LV2_Atom_Vector = unsafe { reader.next_value().unwrap() };

            assert_eq!(vector.atom.type_, urids.vector);
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * 9
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int);

            let vector_items = unsafe { reader.next_slice::<i32>(9) }.unwrap();
            assert_eq!(vector_items, &[17; 9]);

            let int: &sys::LV2_Atom_Int = unsafe { reader.next_value() }.unwrap();
            assert_eq!(int.atom.type_, urids.int);
            assert_eq!(int.atom.size as usize, size_of::<i64>());
            assert_eq!(int.body, 42);
        }

        // reading
        {
            let body = unsafe { raw_space.read().next_atom().unwrap().body() };
            let items: Vec<&UnidentifiedAtom> = unsafe { Tuple::read(body, ()) }.unwrap().collect();
            assert_eq!(items[0].read(urids.vector, urids.int).unwrap(), [17; 9]);
            assert_eq!(items[1].read(urids.int, ()).unwrap(), 42);
        }
    }
}
