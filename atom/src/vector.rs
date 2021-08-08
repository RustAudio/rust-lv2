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
//! use lv2_atom::vector::VectorWriter;
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
use crate::scalar::ScalarAtom;
use crate::space::*;
use crate::*;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use urid::*;

/// An atom containg an array of scalar atom bodies.
///
/// [See also the module documentation.](index.html)
pub struct Vector<C: ScalarAtom> {
    child: PhantomData<C>,
}

unsafe impl<C: ScalarAtom> UriBound for Vector<C> {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

impl<'handle, 'space: 'handle, C: ScalarAtom> Atom<'handle, 'space> for Vector<C> where C: 'space, {
    type ReadParameter = URID<C>;
    type ReadHandle = &'space [C::InternalType];
    type WriteParameter = URID<C>;
    type WriteHandle = VectorWriter<'handle, 'space, C>;

    unsafe fn read(body: &'space Space, child_urid: URID<C>) -> Option<&'space [C::InternalType]> {
        let (header, body) = body.split_for_value_as_unchecked::<sys::LV2_Atom_Vector_Body>()?;

        if header.child_type != child_urid || header.child_size as usize != size_of::<C::InternalType>() {
            return None;
        }

        // SAFETY: We can assume this data was properly initialized by the host.
        Some(body.aligned()?.assume_init())
    }

    fn init(mut frame: AtomSpaceWriter<'handle, 'space>, child_urid: URID<C>) -> Option<VectorWriter<'handle, 'space, C>> {
        let body = sys::LV2_Atom_Vector_Body {
            child_type: child_urid.get(),
            child_size: size_of::<C::InternalType>() as u32,
        };
        space::write_value(&mut frame, body)?;

        Some(VectorWriter {
            frame,
            type_: PhantomData,
        })
    }
}

/// Handle to append elements to a vector.
///
/// This works by allocating a slice of memory behind the vector and then writing your data to it.
pub struct VectorWriter<'handle, 'space, A: ScalarAtom> {
    frame: AtomSpaceWriter<'handle, 'space>,
    type_: PhantomData<A>,
}

impl<'handle, 'space, A: ScalarAtom> VectorWriter<'handle, 'space, A> {
    /// Push a single value to the vector.
    #[inline]
    pub fn push(&'handle mut self, child: A::InternalType) -> Option<&'handle mut A::InternalType> {
        space::write_value(&mut self.frame, child)
    }

    /// Append a slice of undefined memory to the vector.
    ///
    /// Using this method, you don't need to have the elements in memory before you can write them.
    #[inline]
    pub fn allocate_uninit(&'handle mut self, count: usize) -> Option<&'handle mut [MaybeUninit<A::InternalType>]> {
        space::allocate_values(&mut self.frame, count)
    }

    /// Append multiple elements to the vector.
    #[inline]
    pub fn append(&'handle mut self, data: &[A::InternalType]) -> Option<&'handle mut [A::InternalType]> {
        space::write_values(&mut self.frame, data)
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use std::mem::size_of;
    use urid::*;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let map = HashURIDMapper::new();
        let urids = crate::AtomURIDCollection::from_map(&map).unwrap();

        let mut raw_space = AtomSpace::boxed(256);

        // writing
        {
            let mut space = raw_space.as_bytes_mut();
            let mut writer = crate::space::init_atom(&mut space, urids.vector(), urids.int).unwrap();
            writer.append(&[42; CHILD_COUNT - 1]);
            writer.push(1);
        }

        // verifying
        {
            let (vector, children) = unsafe { raw_space.split_for_value_as_unchecked::<sys::LV2_Atom_Vector>() }.unwrap();
            assert_eq!(vector.atom.type_, urids.vector.get());
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * CHILD_COUNT
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int.get());

            let children =
                unsafe { std::slice::from_raw_parts(children.as_bytes().as_ptr() as *const i32, CHILD_COUNT) };
            for value in &children[0..children.len() - 1] {
                assert_eq!(*value, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }

        // reading
        {
            let atom = unsafe { raw_space.to_atom() }.unwrap();
            let children: &[i32] = atom.read(urids.vector, urids.int).unwrap();

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() - 1 {
                assert_eq!(children[i], 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }
}
