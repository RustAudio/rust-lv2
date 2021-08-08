//! An atom containg a series of other atoms.
//!
//! This atom is just like a [sequence](../sequence/index.html), only without time stamps: It contains multiple arbitrary atoms which you can either iterate through or write in sequence.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::tuple::{TupleIterator, TupleWriter};
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
use crate::space::*;
use crate::*;
use urid::*;

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
        Some(TupleIterator { space: body })
    }

    fn init(frame: AtomSpaceWriter<'handle, 'space>, _: ()) -> Option<TupleWriter<'handle, 'space>> {
        Some(TupleWriter { frame })
    }
}

/// An iterator over all atoms in a tuple.
///
/// The item of this iterator is simply the space a single atom occupies.
pub struct TupleIterator<'a> {
    space: &'a Space,
}

impl<'a> Iterator for TupleIterator<'a> {
    type Item = UnidentifiedAtom<'a>;

    fn next(&mut self) -> Option<UnidentifiedAtom<'a>> {
        // SAFETY: The validity of the space is guaranteed by this type.
        let (atom, space) = unsafe { self.space.split_atom() }?;
        self.space = space;
        Some(atom)
    }
}

/// The writing handle to add atoms to a tuple.
pub struct TupleWriter<'handle, 'space> {
    frame: AtomSpaceWriter<'handle, 'space>,
}

impl<'handle, 'space> TupleWriter<'handle, 'space> {
    /// Initialize a new tuple element.
    pub fn init<A: Atom<'handle, 'space>>(
        &'handle mut self,
        child_urid: URID<A>,
        child_parameter: A::WriteParameter,
    ) -> Option<A::WriteHandle> {
        space::init_atom(&mut self.frame, child_urid, child_parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_tuple() {
        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = AtomSpace::boxed(256);

        // writing
        {
            let mut space = raw_space.as_bytes_mut();
            let mut writer = crate::space::init_atom(&mut space, urids.tuple, ()).unwrap();
            {
                let mut vector_writer =
                    writer.init::<Vector<Int>>(urids.vector, urids.int).unwrap();
                vector_writer.append(&[17; 9]).unwrap();
            }
            writer.init::<Int>(urids.int, 42).unwrap();
        }

        // verifying
        {
            let (atom, space) = unsafe { raw_space.split_atom() }.unwrap();
            let header = atom.header().unwrap();
            assert_eq!(header.urid(), urids.tuple);
            assert_eq!(
                header.size_of_body(),
                size_of::<sys::LV2_Atom_Vector>()
                    + size_of::<i32>() * 9
                    + 4
                    + size_of::<sys::LV2_Atom_Int>()
            );

            let (vector, space) = unsafe { space.split_for_value_as_unchecked::<sys::LV2_Atom_Vector>() }.unwrap();
            assert_eq!(vector.atom.type_, urids.vector);
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * 9
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int);

            let (vector_items, space) = space.split_at(size_of::<i32>() * 9).unwrap();
            let vector_items = unsafe { std::slice::from_raw_parts(vector_items.as_bytes().as_ptr() as *const i32, 9) };
            assert_eq!(vector_items, &[17; 9]);

            let (int, _) = unsafe { space.split_for_value_as_unchecked::<sys::LV2_Atom_Int>() }.unwrap();
            assert_eq!(int.atom.type_, urids.int);
            assert_eq!(int.atom.size as usize, size_of::<i32>());
            assert_eq!(int.body, 42);
        }

        // reading
        {
            let (body, _) = unsafe { raw_space.split_atom_body(urids.tuple) }.unwrap();
            let items: Vec<UnidentifiedAtom> = unsafe { Tuple::read(body, ()) }.unwrap().collect();
            assert_eq!(items[0].read(urids.vector, urids.int).unwrap(), [17; 9]);
            assert_eq!(items[1].read(urids.int, ()).unwrap(), 42);
        }
    }
}
