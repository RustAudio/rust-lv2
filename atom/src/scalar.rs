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
