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
//!     let input: &[i32] = ports.input.read(urids.vector).unwrap().of_type(urids.int).unwrap();
//!     let mut output: VectorWriter<Int> = ports.output.write(urids.vector).unwrap().of_type(urids.int).unwrap();
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
use crate::space::SpaceReader;
use crate::*;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};

/// An atom containing an homogenous array of scalar atom bodies.
///
/// [See also the module documentation.](index.html)
pub struct Vector;

unsafe impl UriBound for Vector {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

pub struct VectorReadHandle;

impl<'a> AtomHandle<'a> for VectorReadHandle {
    type Handle = VectorReader<'a>;
}

pub struct VectorWriteHandle;

impl<'a> AtomHandle<'a> for VectorWriteHandle {
    type Handle = VectorTypeWriter<'a>;
}

/// A type-state for the Vector Reader, that reads the header of a vector to figure out its internal
/// type.
pub struct VectorReader<'a> {
    reader: SpaceReader<'a>,
    header: &'a sys::LV2_Atom_Vector_Body,
}

impl<'a> VectorReader<'a> {
    /// Attempts to read the vector as containing a given atom type.
    ///
    /// # Errors
    ///
    /// This method will return an error if the type or size of the atoms contained do not match the
    /// vector being currently read.
    pub fn of_type<C: ScalarAtom>(
        self,
        atom_type: URID<C>,
    ) -> Result<&'a [C::InternalType], AtomReadError> {
        if self.header.child_type != atom_type {
            let found_urid =
                URID::new(self.header.child_type).ok_or(AtomReadError::InvalidAtomValue {
                    reading_type_uri: Vector::uri(),
                    error_message: "Invalid child type URID (0)",
                })?;

            return Err(AtomReadError::AtomUridMismatch {
                found_urid,
                expected_urid: atom_type.into_general(),
                expected_uri: C::uri(),
            });
        }

        if self.header.child_size as usize != size_of::<C::InternalType>() {
            return Err(AtomReadError::InvalidAtomValue {
                reading_type_uri: Vector::uri(),
                error_message: "child_size value does not match actual size of type",
            });
        }

        // SAFETY: The data type has just been checked above, and we can assume this data was
        // properly initialized by the host.
        Ok(unsafe { self.reader.as_slice() }?)
    }

    /// Returns the length, i.e. number of elements in the vector, without knowing their type.
    ///
    /// This can be figured out thanks to the `child_size` attribute in a vector atom header.
    ///
    /// This will always return zero if the elements are zero-sized.
    #[inline]
    pub fn len(&self) -> usize {
        self.reader
            .remaining_bytes()
            .len()
            .checked_div(self.header.child_size as usize)
            .unwrap_or(0)
    }

    /// Returns if the vector is empty, i.e. its `len` is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct VectorTypeWriter<'a> {
    writer: AtomWriter<'a>,
}

impl<'a> VectorTypeWriter<'a> {
    /// Initializes the vector with the given child type URID.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    pub fn of_type<C: ScalarAtom>(
        mut self,
        atom_type: URID<C>,
    ) -> Result<VectorWriter<'a, C>, AtomWriteError> {
        let body = sys::LV2_Atom_Vector_Body {
            child_type: atom_type.get(),
            child_size: size_of::<C::InternalType>() as u32,
        };

        self.writer.write_value(body)?;

        Ok(VectorWriter {
            writer: self.writer,
            type_: PhantomData,
        })
    }
}

impl Atom for Vector {
    type ReadHandle = VectorReadHandle;
    type WriteHandle = VectorWriteHandle;

    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        let mut reader = body.read();
        let header: &sys::LV2_Atom_Vector_Body = reader.next_value()?;

        Ok(VectorReader { reader, header })
    }

    fn write(
        writer: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError> {
        Ok(VectorTypeWriter { writer })
    }
}

/// Handle to append elements to a vector.
///
/// This works by allocating a slice of memory behind the vector and then writing your data to it.
pub struct VectorWriter<'a, A: ScalarAtom> {
    writer: AtomWriter<'a>,
    type_: PhantomData<A>,
}

impl<'a, A: ScalarAtom> VectorWriter<'a, A> {
    /// Push a single value to the vector.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    #[inline]
    pub fn push(&mut self, child: A::InternalType) -> Result<&mut A::InternalType, AtomWriteError> {
        self.writer.write_value(child)
    }

    /// Allocates a slice of initialized memory from the vector.
    ///
    /// This is useful if you need deferred initialization of the vector's contents.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    #[inline]
    pub fn allocate_uninit(
        &mut self,
        count: usize,
    ) -> Result<&mut [MaybeUninit<A::InternalType>], AtomWriteError> {
        self.writer.allocate_values(count)
    }

    /// Append multiple elements to the vector.
    ///
    /// # Errors
    ///
    /// This method will return an error if there is not enough space in the underlying buffer,
    /// or if any other write error occurs.
    #[inline]
    pub fn append(
        &mut self,
        data: &[A::InternalType],
    ) -> Result<&mut [A::InternalType], AtomWriteError> {
        self.writer.write_values(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::AtomURIDCollection;
    use crate::space::*;
    use crate::AtomHeader;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let map = HashURIDMapper::new();
        let urids: AtomURIDCollection = AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = AlignedVec::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            let mut writer = space
                .write_atom(urids.vector)
                .unwrap()
                .of_type(urids.int)
                .unwrap();

            writer.append(&[42; CHILD_COUNT - 1]).unwrap();
            writer.push(1).unwrap();
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

            let children = unsafe { reader.next_values::<i32>(CHILD_COUNT) }.unwrap();
            assert_eq!(children.len(), CHILD_COUNT);
            for value in &children[..CHILD_COUNT - 1] {
                assert_eq!(*value, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }

        // reading
        {
            let atom = unsafe { raw_space.read().next_atom() }.unwrap();
            let children: &[i32] = atom.read(urids.vector).unwrap().of_type(urids.int).unwrap();

            assert_eq!(children.len(), CHILD_COUNT);
            for i in &children[..CHILD_COUNT - 1] {
                assert_eq!(*i, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }
}
