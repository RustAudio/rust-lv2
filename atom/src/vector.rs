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
//! use lv2_urid::prelude::*;
//! use lv2_atom::prelude::*;
//! use lv2_atom::vector::VectorWriter;
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCache) {
//!     let input: &[i32] = ports.input.read(urids.vector, urids.int).unwrap();
//!     let mut output: VectorWriter<Int> = ports.output.init(urids.vector, urids.int).unwrap();
//!     output.append(input).unwrap();
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Vector](http://lv2plug.in/ns/ext/atom/atom.html#Vector)
use crate::scalar::ScalarAtom;
use crate::space::*;
use crate::*;
use core::prelude::*;
use std::marker::PhantomData;
use std::mem::size_of;
use urid::prelude::*;

/// An atom containg an array of scalar atom bodies.
///
/// [See also the module documentation.](index.html)
pub struct Vector<C: ScalarAtom> {
    child: PhantomData<C>,
}

unsafe impl<C: ScalarAtom> UriBound for Vector<C> {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

impl<'a, 'b, C: ScalarAtom> Atom<'a, 'b> for Vector<C>
where
    'a: 'b,
    C: 'b,
{
    type ReadParameter = URID<C>;
    type ReadHandle = &'a [C::InternalType];
    type WriteParameter = URID<C>;
    type WriteHandle = VectorWriter<'a, 'b, C>;

    fn read(body: Space<'a>, child_urid: URID<C>) -> Option<&'a [C::InternalType]> {
        let (header, body) = body.split_type::<sys::LV2_Atom_Vector_Body>()?;

        if header.child_type != child_urid
            || header.child_size as usize != size_of::<C::InternalType>()
        {
            return None;
        }

        let data = body.data()?;

        assert_eq!(data.len() % size_of::<C::InternalType>(), 0);
        let children_count = data.len() / size_of::<C::InternalType>();

        let children = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const C::InternalType, children_count)
        };
        Some(children)
    }

    fn init(
        mut frame: FramedMutSpace<'a, 'b>,
        child_urid: URID<C>,
    ) -> Option<VectorWriter<'a, 'b, C>> {
        let body = sys::LV2_Atom_Vector_Body {
            child_type: child_urid.get(),
            child_size: size_of::<C::InternalType>() as u32,
        };
        (&mut frame as &mut dyn MutSpace).write(&body, false)?;

        Some(VectorWriter {
            frame,
            type_: PhantomData,
        })
    }
}

/// Handle to append elements to a vector.
///
/// This works by allocating a slice of memory behind the vector and then writing your data to it.
pub struct VectorWriter<'a, 'b, A: ScalarAtom> {
    frame: FramedMutSpace<'a, 'b>,
    type_: PhantomData<A>,
}

impl<'a, 'b, A: ScalarAtom> VectorWriter<'a, 'b, A> {
    /// Push a single value to the vector.
    pub fn push(&mut self, child: A::InternalType) -> Option<&mut A::InternalType> {
        (&mut self.frame as &mut dyn MutSpace).write(&child, false)
    }

    /// Append a slice of undefined memory to the vector.
    ///
    /// Using this method, you don't need to have the elements in memory before you can write them.
    pub fn allocate(&mut self, size: usize) -> Option<&mut [A::InternalType]> {
        self.frame
            .allocate(size_of::<A::InternalType>() * size, false)
            .map(|(_, data)| unsafe {
                std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut A::InternalType, size)
            })
    }

    /// Append multiple elements to the vector.
    pub fn append(&mut self, data: &[A::InternalType]) -> Option<&mut [A::InternalType]> {
        let raw_data = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        self.frame
            .allocate(raw_data.len(), false)
            .map(|(_, space)| unsafe {
                space.copy_from_slice(raw_data);
                std::slice::from_raw_parts_mut(
                    space.as_mut_ptr() as *mut A::InternalType,
                    data.len(),
                )
            })
    }
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use crate::prelude::*;
    use crate::space::*;
    use std::mem::size_of;
    use urid::mapper::*;
    use urid::prelude::*;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let mut mapper = Box::pin(HashURIDMapper::new());
        let interface = mapper.as_mut().make_map_interface();
        let map = Map::new(&interface);
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(urids.vector)
                .unwrap();
            let mut writer = Vector::<Int>::init(frame, urids.int).unwrap();
            writer.append(&[42; CHILD_COUNT - 1]);
            writer.push(1);
        }

        // verifying
        {
            let (vector, children) = raw_space.split_at(size_of::<sys::LV2_Atom_Vector>());

            let vector = unsafe { &*(vector.as_ptr() as *const sys::LV2_Atom_Vector) };
            assert_eq!(vector.atom.type_, urids.vector.get());
            assert_eq!(
                vector.atom.size as usize,
                size_of::<sys::LV2_Atom_Vector_Body>() + size_of::<i32>() * CHILD_COUNT
            );
            assert_eq!(vector.body.child_size as usize, size_of::<i32>());
            assert_eq!(vector.body.child_type, urids.int.get());

            let children =
                unsafe { std::slice::from_raw_parts(children.as_ptr() as *const i32, CHILD_COUNT) };
            for value in &children[0..children.len() - 1] {
                assert_eq!(*value, 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }

        // reading
        {
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(urids.vector).unwrap();
            let children: &[i32] = Vector::<Int>::read(body, urids.int).unwrap();

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() - 1 {
                assert_eq!(children[i], 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }
}
