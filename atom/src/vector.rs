use crate::scalar::ScalarAtom;
use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::marker::PhantomData;
use std::mem::size_of;
use urid::{URIDBound, URID};

/// An atom containing a slice of scalar atoms.
///
/// This atom is specified [here](http://lv2plug.in/ns/ext/atom/atom.html#Vector).
pub struct Vector;

unsafe impl UriBound for Vector {
    const URI: &'static [u8] = sys::LV2_ATOM__Vector;
}

impl URIDBound for Vector {
    type CacheType = AtomURIDCache;

    fn urid(urids: &AtomURIDCache) -> URID<Self> {
        urids.vector
    }
}

impl Vector {
    /// Try to read the content of a vector containing `C` atoms.
    ///
    /// If successful, the method returns the content of the atom, a slice of `C::InternalType`, and the space behind the atom.
    ///
    /// If the space is not big enough or does not contain a vector with the given content type, the method returns `None`.
    pub fn read<'a, C: ScalarAtom>(
        space: Space<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<(&'a [C::InternalType], Space<'a>)> {
        let (body, space) = space.split_atom_body(urids.vector)?;
        let (header, body) = body.split_type::<sys::LV2_Atom_Vector_Body>()?;

        if header.child_type != C::urid(child_urids)
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
        Some((children, space))
    }

    /// Initialize a vector containing `C` atoms.
    ///
    /// This method initializes an empty vector and returns a writer to add `C` to the vector. If the space is not big enough, the method returns `None`.
    pub fn write<'a, 'b, C: ScalarAtom>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<VectorWriter<'a, 'b, C::InternalType>> {
        let mut frame = space.create_atom_frame(urids.vector)?;

        let body = sys::LV2_Atom_Vector_Body {
            child_type: C::urid(child_urids).get(),
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
pub struct VectorWriter<'a, 'b, T>
where
    T: Unpin + Copy + Send + Sync + Sized + 'static,
{
    frame: FramedMutSpace<'a, 'b>,
    type_: PhantomData<T>,
}

impl<'a, 'b, T> VectorWriter<'a, 'b, T>
where
    T: Unpin + Copy + Send + Sync + Sized + 'static,
{
    /// Push a single value to the vector.
    pub fn push(&mut self, child: T) -> Option<&mut T> {
        (&mut self.frame as &mut dyn MutSpace).write(&child, false)
    }

    /// Append a slice of undefined memory to the vector.
    ///
    /// Using this method, you don't need to have the elements in memory before you can write them.
    pub fn allocate(&mut self, size: usize) -> Option<&mut [T]> {
        self.frame
            .allocate(size_of::<T>() * size, false)
            .map(|(_, data)| unsafe {
                std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut T, size)
            })
    }

    /// Append multiple elements to the vector.
    pub fn append(&mut self, data: &[T]) -> Option<&mut [T]> {
        let raw_data = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        self.frame
            .allocate(raw_data.len(), false)
            .map(|(_, space)| unsafe {
                space.copy_from_slice(raw_data);
                std::slice::from_raw_parts_mut(space.as_mut_ptr() as *mut T, data.len())
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::scalar::Int;
    use crate::space::*;
    use crate::vector::*;
    use std::mem::size_of;
    use urid::URIDCache;

    #[test]
    fn test_vector() {
        const CHILD_COUNT: usize = 17;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let mut writer = Vector::write::<Int>(&mut space, &urids, &urids).unwrap();
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
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            let children: &[i32] = Vector::read::<Int>(space, &urids, &urids).unwrap().0;

            assert_eq!(children.len(), CHILD_COUNT);
            for i in 0..children.len() - 1 {
                assert_eq!(children[i], 42);
            }
            assert_eq!(children[children.len() - 1], 1);
        }
    }
}
