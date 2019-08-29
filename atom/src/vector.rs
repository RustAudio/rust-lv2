use crate::space::*;
use crate::AtomURIDCache;
use crate::ScalarAtom;
use core::UriBound;
use std::mem::size_of;
use urid::{URIDBound, URID};

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
    pub fn get_children<'a, C: ScalarAtom>(
        space: Space<'a, Self>,
        urids: &C::CacheType,
    ) -> Option<&'a [C::InternalType]> {
        let (body, space) = unsafe { space.split_type::<sys::LV2_Atom_Vector_Body>() }?;

        if body.child_type != C::urid(urids)
            || body.child_size as usize != size_of::<C::InternalType>()
        {
            return None;
        }

        let data = space.data()?;

        assert_eq!(data.len() % size_of::<C::InternalType>(), 0);
        let children_count = data.len() / size_of::<C::InternalType>();

        let children = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const C::InternalType, children_count)
        };
        Some(children)
    }

    pub fn new_vector<'a, 'b, C: ScalarAtom>(
        space: &'b mut dyn MutSpace<'a>,
        urids: &AtomURIDCache,
        child_urids: &C::CacheType,
    ) -> Option<VectorWriter<'a, 'b, C>> {
        let mut frame: FramedMutSpace<Self> = unsafe { space.create_atom_frame(urids)? };

        let body = sys::LV2_Atom_Vector_Body {
            child_type: C::urid(child_urids).get(),
            child_size: size_of::<C::InternalType>() as u32,
        };
        unsafe { (&mut frame as &mut dyn MutSpace).write(&body, true)? };

        let children = unsafe { frame.allocate(0, false) }?;
        Some(VectorWriter {
            children: unsafe {
                std::slice::from_raw_parts_mut(
                    children.as_mut_ptr() as *mut C::InternalType,
                    children.len(),
                )
            },
            space: frame,
        })
    }
}

pub struct VectorWriter<'a, 'b, C: ScalarAtom> {
    children: &'a mut [C::InternalType],
    space: FramedMutSpace<'a, 'b, Vector>,
}

impl<'a, 'b, C: ScalarAtom> VectorWriter<'a, 'b, C> {
    pub fn add_child(&mut self, child: C::InternalType) -> Option<()> {
        self.add_children(1).map(|slice| slice[0] = child)
    }

    pub fn add_children(&mut self, count: usize) -> Option<&mut [C::InternalType]> {
        let new_children = unsafe {
            self.space
                .allocate(size_of::<C::InternalType>() * count, false)?
        };
        let new_children = unsafe {
            std::slice::from_raw_parts_mut(new_children.as_mut_ptr() as *mut C::InternalType, count)
        };
        self.children = unsafe {
            std::slice::from_raw_parts_mut(
                self.children.as_mut_ptr() as *mut C::InternalType,
                self.children.len() + new_children.len(),
            )
        };
        Some(new_children)
    }

    pub fn get_children(self) -> &'a mut [C::InternalType] {
        self.children
    }
}

#[cfg(test)]
mod tests {
    use crate::space::*;
    use crate::Int;
    use crate::Vector;
    use std::mem::{size_of, size_of_val};
    use urid::URIDCache;

    #[test]
    fn test_vector_retrieval() {
        const CHILD_COUNT: usize = 17;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);
        {
            let (vector, children) = raw_space.split_at_mut(size_of::<sys::LV2_Atom_Vector>());
            let vector = unsafe { &mut *(vector.as_mut_ptr() as *mut sys::LV2_Atom_Vector) };
            let children = unsafe {
                std::slice::from_raw_parts_mut(children.as_mut_ptr() as *mut i32, CHILD_COUNT)
            };

            vector.atom = sys::LV2_Atom {
                type_: urids.vector.get(),
                size: (size_of_val(children) + size_of::<sys::LV2_Atom_Vector_Body>()) as u32,
            };
            vector.body = sys::LV2_Atom_Vector_Body {
                child_size: size_of::<i32>() as u32,
                child_type: urids.int.get(),
            };

            for i in 0..children.len() {
                children[i] = i as i32;
            }
        }

        let space: Space<Vector> =
            unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom), &urids) }
                .unwrap();
        let children: &[i32] = Vector::get_children::<Int>(space, &urids).unwrap();

        assert_eq!(children.len(), CHILD_COUNT);
        for i in 0..children.len() {
            assert_eq!(children[i], i as i32);
        }
    }

    #[test]
    fn test_vector_writing() {
        const CHILD_COUNT: usize = 17;

        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());

            let mut writer = Vector::new_vector::<Int>(&mut space, &urids, &urids).unwrap();
            {
                let children = writer.add_children(CHILD_COUNT).unwrap();
                for i in 0..children.len() {
                    children[i] = i as i32;
                }
            }
            // Checking that the written slice and the general slice are the same.
            for (i, child) in writer.get_children().into_iter().enumerate() {
                assert_eq!(i, *child as usize);
            }
        }

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
        for i in 0..CHILD_COUNT {
            assert_eq!(children[i], i as i32);
        }
    }
}
