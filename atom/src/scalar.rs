use crate::space::*;
use crate::*;
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
    fn read_scalar(body: Space) -> Option<Self::InternalType> {
        body.split_type::<Self::InternalType>()
            .map(|(value, _)| *value)
    }

    /// Try to write the atom into a space.
    ///
    /// Write an atom with the value of `value` into the space and return a mutable reference to the written value. If the space is not big enough, return `None`.
    fn write_scalar<'a, 'b>(
        mut frame: FramedMutSpace<'a, 'b>,
        value: Self::InternalType,
    ) -> Option<&'a mut Self::InternalType> {
        (&mut frame as &mut dyn MutSpace).write(&value, true)
    }
}

impl<'a, 'b, A: ScalarAtom> Atom<'a, 'b> for A
where
    'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = A::InternalType;
    type WriteParameter = A::InternalType;
    type WriteHandle = &'a mut A::InternalType;

    fn read(body: Space<'a>, _: ()) -> Option<A::InternalType> {
        <A as ScalarAtom>::read_scalar(body)
    }

    fn write(
        frame: FramedMutSpace<'a, 'b>,
        value: A::InternalType,
    ) -> Option<&'a mut A::InternalType> {
        <A as ScalarAtom>::write_scalar(frame, value)
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
#[cfg(feature = "host")]
mod tests {
    use crate::scalar::*;
    use std::convert::TryFrom;
    use std::mem::size_of;
    use urid::feature::Map;
    use urid::mapper::HashURIDMapper;
    use urid::URIDCache;

    fn test_scalar<A: ScalarAtom>(value: A::InternalType)
    where
        A::InternalType: PartialEq<A::InternalType>,
        A::InternalType: std::fmt::Debug,
    {
        let mapper = HashURIDMapper::new();
        let map = Map::new(&mapper);
        let urids = A::CacheType::from_map(&map).unwrap();

        let mut raw_space: Box<[u8]> = Box::new([0; 256]);

        // writing
        {
            let mut space = RootMutSpace::new(raw_space.as_mut());
            let frame = (&mut space as &mut dyn MutSpace)
                .create_atom_frame(A::urid(&urids))
                .unwrap();
            A::write(frame, value).unwrap();
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
            let space = Space::from_slice(raw_space.as_ref());
            let (body, _) = space.split_atom_body(A::urid(&urids)).unwrap();
            assert_eq!(A::read(body, ()).unwrap(), value);
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
