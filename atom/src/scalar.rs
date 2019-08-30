use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::os::raw::*;
use urid::{URIDBound, URID};

pub trait ScalarAtom: URIDBound {
    type InternalType: Copy + Sized + 'static;

    fn space_as_body(space: Space<Self>) -> Option<Self::InternalType> {
        unsafe { space.split_type::<Self::InternalType>() }.map(|(value, _)| *value)
    }

    fn write_body<'a>(
        space: &mut dyn MutSpace<'a>,
        value: Self::InternalType,
        urids: &Self::CacheType,
    ) -> Option<&'a mut Self::InternalType> {
        unsafe {
            space
                .create_atom_frame::<Self>(urids)
                .and_then(|mut frame| (&mut frame as &mut dyn MutSpace).write(&value, true))
        }
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
            type InternalType = $internal;
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

pub struct Bool;

make_scalar_atom!(Bool, c_int, sys::LV2_ATOM__Bool, |urids: &AtomURIDCache| {
    urids.bool
});

pub struct AtomURID;

make_scalar_atom!(
    AtomURID,
    URID,
    sys::LV2_ATOM__URID,
    |urids: &AtomURIDCache| urids.urid
);

#[cfg(test)]
mod tests {
    use crate::scalar::*;
    use std::convert::TryFrom;
    use std::mem::size_of;
    use urid::URIDCache;

    fn test_scalar<A: ScalarAtom>(value: A::InternalType)
    where
        A::InternalType: PartialEq<A::InternalType>,
        A::InternalType: std::fmt::Debug,
    {
        let mut map_interface = urid::mapper::URIDMap::new().make_map_interface();
        let map = map_interface.map();
        let urids = A::CacheType::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            A::write_body(&mut space, value, &urids).unwrap();
        }

        // verifying
        {
            /// Generic version of the scalar atom structs.
            #[repr(C)]
            struct Scalar<B: Sized> {
                atom: sys::LV2_Atom,
                body: B,
            }

            let (scalar, _) = raw_space.split_at(size_of::<sys::LV2_Atom>());

            let scalar = unsafe { &*(scalar.as_ptr() as *const Scalar<A::InternalType>) };
            assert_eq!(scalar.atom.type_, A::urid(&urids));
            assert_eq!(scalar.atom.size as usize, size_of::<A::InternalType>());
            assert_eq!(scalar.body, value);
        }

        // reading
        {
            let space: Space<A> = unsafe {
                Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom), &urids).unwrap()
            };
            assert_eq!(A::space_as_body(space).unwrap(), value);
        }
    }

    #[test]
    fn test_scalars() {
        test_scalar::<Double>(42.0);
        test_scalar::<Float>(42.0);
        test_scalar::<Long>(42);
        test_scalar::<Int>(42);
        test_scalar::<Bool>(1);
        test_scalar::<AtomURID>(URID::try_from(1).unwrap());
    }
}
