//! An atom containg an array of scalar atom bodies.
//!
//! This atom is able to handle arrays (aka slices) of the internal types of scalar atoms.
//!
//! Reading a vector requires the URID fo the scalar that's been used and the reading process fails if the vector does not contain the requested scalar atom. The return value of the reading process is a slice of the internal type.
//!
//! Writing a vector is done with a writer that appends slices to the atom.
//!
//! # Example
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::atoms::vector::VectorWriter;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     let input: &[i32] = ports.input.read(urids.vector(), urids.int).unwrap();
//!     let mut output: VectorWriter<Int> = ports.output.init(urids.vector(), urids.int).unwrap();
//!     output.append(input).unwrap();
//! }
//! ```
//!
//! You may note that, unlike other atoms, the vector's URID is retrieved by calling the `vector` method. This is because two vectors with a different item type are considered two different types, and therefore would have the different URIDs. In reality, however, all vectors have the same URID and the `vector` method returns it with the fitting type.
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Vector](http://lv2plug.in/ns/ext/atom/atom.html#Vector)
use crate::atoms::scalar::ScalarAtom;
use crate::space::reader::SpaceReader;
use crate::*;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};

/// An atom containg an array of scalar atom bodies.
///
/// [See also the module documentation.](index.html)
pub struct Vector;

unsafe impl UriBound for Vector {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

struct VectorReadHandle;

impl<'a> AtomHandle<'a> for VectorReadHandle {
    type Handle = VectorReader<'a>;
}

struct VectorWriteHandle;

impl<'a> AtomHandle<'a> for VectorWriteHandle {
    type Handle = VectorTypeWriter<'a>;
}

pub struct VectorReader<'a> {
    reader: SpaceReader<'a>,
    header: &'a sys::LV2_Atom_Vector_Body,
}

impl<'a> VectorReader<'a> {
    pub fn of_type<C: ScalarAtom>(self, atom_type: URID<C>) -> Option<&'a [C::InternalType]> {
        if self.header.child_type != atom_type
            || self.header.child_size as usize != size_of::<C::InternalType>()
        {
            return None;
        }

        // SAFETY: The data type has just been checked above, and we can assume this data was
        // properly initialized by the host.
        unsafe { self.reader.as_slice() }
    }
}

pub struct VectorTypeWriter<'a> {
    header: &'a mut MaybeUninit<sys::LV2_Atom_Vector_Body>,
    writer: AtomSpaceWriter<'a>,
}

impl<'a> VectorTypeWriter<'a> {
    pub fn of_type<C: ScalarAtom>(mut self, atom_type: URID<C>) -> VectorWriter<'a, C> {
        let body = sys::LV2_Atom_Vector_Body {
            child_type: atom_type.get(),
            child_size: size_of::<C::InternalType>() as u32,
        };

        crate::util::write_uninit(&mut self.header, body);

        VectorWriter {
            writer: self.writer,
            type_: PhantomData,
        }
    }
}

impl Atom for Vector {
    type ReadHandle = VectorReadHandle;
    type WriteHandle = VectorWriteHandle;

    unsafe fn read<'handle, 'space: 'handle>(
        body: &'space AtomSpace,
    ) -> Option<<Self::ReadHandle as AtomHandle<'handle>>::Handle> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Vector_Body = reader.next_value()?;

        Some(VectorReader { reader, header })
    }

    fn init(mut writer: AtomSpaceWriter) -> Option<<Self::WriteHandle as AtomHandle>::Handle> {
        let header = writer.write_value(MaybeUninit::uninit())?;

        Some(VectorTypeWriter { writer, header })
    }
}

/// Handle to append elements to a vector.
///
/// This works by allocating a slice of memory behind the vector and then writing your data to it.
pub struct VectorWriter<'handle, A: ScalarAtom> {
    writer: AtomSpaceWriter<'handle>,
    type_: PhantomData<A>,
}

impl<'handle, A: ScalarAtom> VectorWriter<'handle, A> {
    /// Push a single value to the vector.
    #[inline]
    pub fn push(&mut self, child: A::InternalType) -> Option<&mut A::InternalType> {
        self.writer.write_value(child)
    }

    /// Append a slice of undefined memory to the vector.
    ///
    /// Using this method, you don't need to have the elements in memory before you can write them.
    #[inline]
    pub fn allocate_uninit(&mut self, count: usize) -> Option<&mut [MaybeUninit<A::InternalType>]> {
        self.writer.allocate_values(count)
    }

    /// Append multiple elements to the vector.
    #[inline]
    pub fn append(&mut self, data: &[A::InternalType]) -> Option<&mut [A::InternalType]> {
        self.writer.write_values(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let map = HashURIDMapper::new();
        let urids = crate::atoms::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space.init_atom(urids.vector(), urids.int).unwrap();
            writer.append(&[42; CHILD_COUNT - 1]);
            writer.push(1);
        }

        // verifying
        {
            let mut reader = raw_space.read();
            let vector: &sys::LV2_Atom_Vector = unsafe { reader.next_value() }.unwrap();
            assert_eq!(vector.atom.type_, urids.vector.get());
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * CHILD_COUNT
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int.get());

            let children = unsafe { reader.next_slice::<i32>(CHILD_COUNT) }.unwrap();
            for value in &children[0..children.len() - 1] {
                assert_eq!(*value, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }

        // reading
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();
            let children: &[i32] = atom.read(urids.vector).unwrap();

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() - 1 {
                assert_eq!(children[i], 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }
}
