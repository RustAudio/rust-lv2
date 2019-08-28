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

impl<'a> Space<'a, Vector> {
    pub fn as_children_slice<A: ScalarAtom>(
        self,
        urids: &A::CacheType,
    ) -> Option<&'a [A::InternalType]> {
        let (body, space) = unsafe { self.split_type::<sys::LV2_Atom_Vector_Body>() }?;

        if body.child_type != A::urid(urids)
            || body.child_size as usize != size_of::<A::InternalType>()
        {
            return None;
        }

        let data = space.data()?;

        assert_eq!(data.len() % size_of::<A::InternalType>(), 0);
        let children_count = data.len() / size_of::<A::InternalType>();

        let children = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const A::InternalType, children_count)
        };
        Some(children)
    }
}

impl<'a, 'b> FramedMutSpace<'a, 'b, Vector> {
    pub fn write_body<A: ScalarAtom>(
        &mut self,
        urids: &A::CacheType,
        children_count: usize,
    ) -> Option<&'a mut [A::InternalType]> {
        let body = sys::LV2_Atom_Vector_Body {
            child_type: A::urid(urids).get(),
            child_size: size_of::<A::InternalType>() as u32,
        };
        (self as &mut dyn MutSpace).write(&body)?;

        self.allocate(children_count * size_of::<A::InternalType>())
            .map(|space| unsafe {
                std::slice::from_raw_parts_mut(
                    space.as_mut_ptr() as *mut A::InternalType,
                    children_count,
                )
            })
    }
}
