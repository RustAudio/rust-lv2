use crate::atomspace::*;
use crate::frame::{AtomWritingFrame, WritingFrame};
use crate::AtomBody;
use crate::AtomURIDCache;
use core::UriBound;
use std::ops::Deref;
use std::os::raw::*;
use urid::URID;

macro_rules! make_scalar_atom {
    ($atom:ty, $internal:ty, $uri:expr, $urid:expr) => {
        impl $atom {
            pub fn new(value: $internal) -> Self {
                Self(value)
            }
        }

        impl Deref for $atom {
            type Target = $internal;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        unsafe impl UriBound for $atom {
            const URI: &'static [u8] = $uri;
        }

        impl AtomBody for $atom {
            type InitializationParameter = $internal;

            #[allow(clippy::redundant_closure_call)]
            fn urid(urids: &AtomURIDCache) -> URID<Self> {
                ($urid)(urids)
            }

            fn retrieve(bytes: AtomSpace) -> Option<Self> {
                Some(Self(*unsafe { bytes.retrieve_type::<$internal>() }?.0))
            }

            fn initialize_frame(frame: &mut AtomWritingFrame<Self>, param: &$internal) -> bool {
                (frame as &mut dyn WritingFrame).write(param).is_some()
            }
        }
    };
}

#[repr(transparent)]
pub struct Double(c_double);

make_scalar_atom!(
    Double,
    c_double,
    sys::LV2_ATOM__Double,
    |urids: &AtomURIDCache| urids.double
);

#[repr(transparent)]
pub struct Float(c_float);

make_scalar_atom!(
    Float,
    c_float,
    sys::LV2_ATOM__Float,
    |urids: &AtomURIDCache| urids.float
);

#[repr(transparent)]
pub struct Int(c_int);

make_scalar_atom!(Int, c_int, sys::LV2_ATOM__Int, |urids: &AtomURIDCache| {
    urids.int
});

#[repr(transparent)]
pub struct Long(c_long);

make_scalar_atom!(
    Long,
    c_long,
    sys::LV2_ATOM__Long,
    |urids: &AtomURIDCache| urids.long
);

#[repr(transparent)]
pub struct AtomURID(URID);

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
                let space = AtomSpace::new(data_slice);
                let value = space.retrieve_atom::<$atom>(&urids).unwrap().0;
                assert_eq!($value, *value);
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
                let mut atom_frame = (&mut root_frame as &mut dyn WritingFrame)
                    .create_atom_frame(&urids)
                    .unwrap();

                <$atom>::initialize_frame(&mut atom_frame, &$value);
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
