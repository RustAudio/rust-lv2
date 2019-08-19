use crate::AtomBody;
use crate::AtomURIDCache;
use core::UriBound;
use std::mem::size_of;
use std::ops::Deref;
use std::os::raw::*;
use urid::URID;

macro_rules! make_scalar_atom {
    ($atom:ident, $internal:ty, $uri:expr, $urid:expr) => {
        #[repr(transparent)]
        pub struct $atom($internal);

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
            fn urid(urids: &AtomURIDCache) -> URID<Self> {
                ($urid)(urids)
            }

            unsafe fn create_ref(bytes: &[u8]) -> Option<&Self> {
                if bytes.len() == size_of::<Self>() {
                    (bytes.as_ptr() as *const Self).as_ref()
                } else {
                    None
                }
            }
        }
    };
}

make_scalar_atom!(
    AtomDouble,
    c_double,
    sys::LV2_ATOM__Double,
    |urids: &AtomURIDCache| urids.double
);
make_scalar_atom!(
    AtomFloat,
    c_float,
    sys::LV2_ATOM__Float,
    |urids: &AtomURIDCache| urids.float
);
make_scalar_atom!(
    AtomInt,
    c_int,
    sys::LV2_ATOM__Int,
    |urids: &AtomURIDCache| urids.int
);
make_scalar_atom!(
    AtomLong,
    c_long,
    sys::LV2_ATOM__Long,
    |urids: &AtomURIDCache| urids.long
);
make_scalar_atom!(
    AtomURID,
    URID,
    sys::LV2_ATOM__URID,
    |urids: &AtomURIDCache| urids.urid
);

#[cfg(test)]
mod tests {
    use crate::scalar::*;
    use crate::UnidentifiedAtom;
    use std::mem::{size_of, size_of_val};
    use urid::URIDCache;
    use sys::*;

    #[test]
    fn test_scalars() {
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
                    body: $value
                };
                let data_slice = unsafe {
                    std::slice::from_raw_parts(
                        &original_atom as *const _ as *const u8,
                        size_of_val(&original_atom),
                    )
                };
                let atom = unsafe { UnidentifiedAtom::from_slice(data_slice) }.unwrap();
                let value = atom.identify::<$atom>(&urids).unwrap();
                assert_eq!($value, **value);
            };
        }

        test_atom!(LV2_Atom_Double, c_double, AtomDouble, 42.0);
        test_atom!(LV2_Atom_Float, c_float, AtomFloat, 42.0);
        test_atom!(LV2_Atom_Long, c_long, AtomLong, 42);
        test_atom!(LV2_Atom_Int, c_int, AtomInt, 42);
    }
}
