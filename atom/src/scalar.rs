use crate::space::*;
use crate::AtomURIDCache;
use core::UriBound;
use std::marker::Unpin;
use std::os::raw::*;
use urid::{URIDBound, URID};

/// An atom that only contains a single, scalar value.
///
/// Since scalar values are so simple, the reading and writing methods are exactly the same.
pub trait ScalarAtom: URIDBound {
    /// The internal representation of the atom.
    ///
    /// For example, the `Int` atom has the internal type of `c_int`, which is `i32` on most platforms.
    type InternalType: Unpin + Copy + Send + Sync + Sized + 'static;

    /// Try to read the atom from a space.
    ///
    /// If the space does not contain the atom or is not big enough, return `None`. The second return value is the space behind the atom.
    fn read<'a>(
        space: Space<'a>,
        urids: &Self::CacheType,
    ) -> Option<(Self::InternalType, Space<'a>)> {
        let (body, space) = space.split_atom_body(Self::urid(urids))?;
        Some((*body.split_type::<Self::InternalType>()?.0, space))
    }

    /// Try to write the atom into a space.
    ///
    /// Write an atom with the value of `value` into the space and return a mutable reference to the written value. If the space is not big enough, return `None`.
    fn write<'a>(
        space: &mut dyn MutSpace<'a>,
        value: Self::InternalType,
        urids: &Self::CacheType,
    ) -> Option<&'a mut Self::InternalType> {
        space
            .create_atom_frame(Self::urid(urids))
            .and_then(|mut frame| (&mut frame as &mut dyn MutSpace).write(&value, true))
    }
}

/// Macro to atomate the definition of scalar atoms.
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

/// A scalar atom containing a `c_double` (`f64` on most platforms).
pub struct Double;

make_scalar_atom!(
    Double,
    c_double,
    sys::LV2_ATOM__Double,
    |urids: &AtomURIDCache| urids.double
);

/// A scalar atom containing a `c_float` (`f32` on most platforms).
pub struct Float;

make_scalar_atom!(
    Float,
    c_float,
    sys::LV2_ATOM__Float,
    |urids: &AtomURIDCache| urids.float
);

/// A scalar atom containing a `c_long` (`i64` on most platforms).
pub struct Long;

make_scalar_atom!(
    Long,
    c_long,
    sys::LV2_ATOM__Long,
    |urids: &AtomURIDCache| urids.long
);

/// A scalar atom containing a `c_int` (`i32` on most platforms).
pub struct Int;

make_scalar_atom!(Int, c_int, sys::LV2_ATOM__Int, |urids: &AtomURIDCache| {
    urids.int
});

/// A scalar atom representing a boolean.
///
/// Internally, this atom is represented by a `c_int`, which is `==0` for `false` and `>= 1` for `true`
pub struct Bool;

make_scalar_atom!(Bool, c_int, sys::LV2_ATOM__Bool, |urids: &AtomURIDCache| {
    urids.bool
});

/// A scalar atom containing a URID.
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
            A::write(&mut space, value, &urids).unwrap();
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
            let space = unsafe { Space::from_atom(&*(raw_space.as_ptr() as *const sys::LV2_Atom)) };
            assert_eq!(A::read(space, &urids).unwrap().0, value);
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
