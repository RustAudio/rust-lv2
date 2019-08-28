use crate::atomspace::*;
use crate::frame::WritingFrame;
use crate::AtomURIDCache;
use core::UriBound;
use std::mem::size_of;
use std::os::raw::*;
use urid::{URIDBound, URID};

pub trait ScalarAtom: URIDBound {
    type InternalValue: Sized + Copy + 'static;

    fn get(space: &mut AtomSpace, urids: &Self::CacheType) -> Option<Self::InternalValue> {
        let atom = unsafe { space.retrieve_type::<sys::LV2_Atom>() }?;

        if atom.size as usize != size_of::<Self::InternalValue>()
            || atom.type_ != Self::urid(urids).get()
        {
            return None;
        }

        let value = unsafe { space.retrieve_type::<Self::InternalValue>() }?;
        Some(*value)
    }

    fn write<'a>(
        space: &mut dyn WritingFrame<'a>,
        value: Self::InternalValue,
        urids: &Self::CacheType,
    ) -> Option<&'a mut Self::InternalValue> {
        let mut frame = space.create_atom_frame(Self::urid(urids).into_general())?;
        (&mut frame as &mut dyn WritingFrame).write(&value)
    }
}

macro_rules! make_scalar_atom {
    ($atom:ty, $internal:ty, $uri:expr, $urid:expr) => {
        unsafe impl UriBound for $atom {
            const URI: &'static [u8] = $uri;
        }

        impl URIDBound for $atom {
            type CacheType = AtomURIDCache;

            fn urid(cache: &AtomURIDCache) -> URID<$atom> {
                #[allow(clippy::redundant_closure_call)]
                ($urid)(cache)
            }
        }

        impl ScalarAtom for $atom {
            type InternalValue = $internal;
        }
    };
}

pub struct Double;

make_scalar_atom!(
    Double,
    c_double,
    sys::LV2_ATOM__Double,
    |urids: &AtomURIDCache| urids.double
);

pub struct Float;

make_scalar_atom!(
    Float,
    c_float,
    sys::LV2_ATOM__Float,
    |urids: &AtomURIDCache| urids.float
);

pub struct Int;

make_scalar_atom!(Int, c_int, sys::LV2_ATOM__Int, |urids: &AtomURIDCache| {
    urids.int
});

pub struct Long;

make_scalar_atom!(
    Long,
    c_long,
    sys::LV2_ATOM__Long,
    |urids: &AtomURIDCache| urids.long
);

pub struct AtomURID;

make_scalar_atom!(
    AtomURID,
    URID,
    sys::LV2_ATOM__URID,
    |urids: &AtomURIDCache| urids.urid
);

#[cfg(test)]
mod tests {
    use crate::atomspace::AtomSpace;
    use crate::frame::*;
    use crate::scalar::*;
    use std::mem::{size_of, size_of_val};
    use sys::*;
    use urid::URIDCache;

    #[test]
    fn test_scalar_retrieval() {
        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = crate::AtomURIDCache::from_map(&map).unwrap();

        macro_rules! test_atom {
            ($orig:ident, $raw:ty, $atom:ty, $value:expr) => {
                let original_atom = $orig {
                    atom: sys::LV2_Atom {
                        type_: <$atom>::urid(&urids).get(),
                        size: size_of::<$raw>() as u32,
                    },
                    body: $value,
                };
                let data_slice = unsafe {
                    std::slice::from_raw_parts(
                        &original_atom as *const _ as *const u8,
                        size_of_val(&original_atom),
                    )
                };
                let mut space = AtomSpace::new(data_slice);
                let value = <$atom>::get(&mut space, &urids).unwrap();
                assert_eq!($value, value);
            };
        }

        test_atom!(LV2_Atom_Double, c_double, Double, 42.0);
        test_atom!(LV2_Atom_Float, c_float, Float, 42.0);
        test_atom!(LV2_Atom_Long, c_long, Long, 42);
        test_atom!(LV2_Atom_Int, c_int, Int, 42);
        test_atom!(LV2_Atom_URID, URID, AtomURID, urids.urid.get());
    }

    #[test]
    fn test_scalar_writing() {
        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let urids = crate::AtomURIDCache::from_map(&map_interface.map()).unwrap();

        let mut memory: [u64; 256] = [0; 256];
        let raw_memory: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(memory.as_mut_ptr() as *mut u8, 256 * size_of::<u64>())
        };

        macro_rules! test_atom {
            ($orig:ident, $raw:ty, $atom:ty, $value:expr) => {{
                let mut root_frame = RootWritingFrame::new(raw_memory);
                <$atom>::write(&mut root_frame, $value, &urids).unwrap();
            }
            let raw_atom = unsafe { &*(raw_memory.as_ptr() as *const $orig) };
            assert_eq!(raw_atom.atom.size as usize, size_of::<$raw>());
            assert_eq!(raw_atom.atom.type_, <$atom>::urid(&urids).get());
            assert_eq!(raw_atom.body, $value);};
        }

        test_atom!(LV2_Atom_Double, c_double, Double, 42.0);
        test_atom!(LV2_Atom_Float, c_float, Float, 42.0);
        test_atom!(LV2_Atom_Long, c_long, Long, 42);
        test_atom!(LV2_Atom_Int, c_int, Int, 42);
        test_atom!(LV2_Atom_URID, URID, AtomURID, urids.urid.into_general());
    }
}
