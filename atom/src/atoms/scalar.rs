//! Scalar, single-value atoms.
//!
//! These atoms are the simplest of them all: They are simply represented by an internal type and their values can simply be copied. Due to this common behaviour, there is another trait called [`ScalarAtom`](trait.ScalarAtom.html) which provides this behaviour. Every type that implements `ScalarAtom` also implements `Atom`.
//!
//! Unlike other atoms, scalars do not need to be written after the initialization. However, you still can modify the scalar after it was initialized.
//!
//! # Example
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_atom::prelude::*;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &AtomURIDCollection) {
//!     // Scalar atoms don't need a reading parameter.
//!     let read_value: f32 = ports.input.read(urids.float, ()).unwrap();
//!
//!     // Writing is done with the value of the atom.
//!     ports.output.init(urids.float, 17.0).unwrap();
//! }
//! ```
//!
//! # Specification
//!
//! [http://lv2plug.in/ns/ext/atom/atom.html#Number](http://lv2plug.in/ns/ext/atom/atom.html#Number)
use crate::*;
use std::marker::Unpin;
use urid::UriBound;
use urid::URID;

/// An atom that only contains a single, scalar value.
///
/// Since scalar values are so simple, the reading and writing methods are exactly the same.
pub trait ScalarAtom: UriBound {
    /// The internal representation of the atom.
    ///
    /// For example, the `Int` atom has the internal type of `i32`, which is `i32` on most platforms.
    type InternalType: Unpin + Copy + Send + Sync + Sized + 'static;

    /// Try to read the atom from a space.
    ///
    /// If the space does not contain the atom or is not big enough, return `None`. The second return value is the space behind the atom.
    #[inline]
    unsafe fn read_scalar(body: &Space) -> Option<Self::InternalType> {
        body.read().next_value().copied()
    }

    /// Try to write the atom into a space.
    ///
    /// Write an atom with the value of `value` into the space and return a mutable reference to the written value. If the space is not big enough, return `None`.
    #[inline]
    fn write_scalar<'handle, 'space: 'handle>(
        mut frame: AtomSpaceWriter<'handle, 'space>,
        value: Self::InternalType,
    ) -> Option<()> {
        frame.write_value(value)?;
        // Scalars have extra padding due to repr(C)
        frame.allocate_padding_for::<AtomHeader>();
        Some(())
    }
}

impl<'handle, 'space: 'handle, A: ScalarAtom> Atom<'handle, 'space> for A {
    type ReadParameter = ();
    type ReadHandle = A::InternalType;
    type WriteParameter = A::InternalType;
    type WriteHandle = ();

    unsafe fn read(body: &'space Space, _: ()) -> Option<A::InternalType> {
        <A as ScalarAtom>::read_scalar(body)
    }

    fn init(frame: AtomSpaceWriter<'handle, 'space>, value: A::InternalType) -> Option<()> {
        <A as ScalarAtom>::write_scalar(frame, value)
    }
}

/// Macro to atomate the definition of scalar atoms.
macro_rules! make_scalar_atom {
    ($atom:ty, $internal:ty, $uri:expr, $urid:expr) => {
        unsafe impl UriBound for $atom {
            const URI: &'static [u8] = $uri;
        }

        impl ScalarAtom for $atom {
            type InternalType = $internal;
        }
    };
}

/// A scalar atom containing a `f64` (`f64` on most platforms).
pub struct Double;

make_scalar_atom!(
    Double,
    f64,
    sys::LV2_ATOM__Double,
    |urids: &AtomURIDCollection| urids.double
);

/// A scalar atom containing a `f32` (`f32` on most platforms).
pub struct Float;

make_scalar_atom!(
    Float,
    f32,
    sys::LV2_ATOM__Float,
    |urids: &AtomURIDCollection| { urids.float }
);

/// A scalar atom containing a `i64` (`i64` on most platforms).
pub struct Long;

make_scalar_atom!(
    Long,
    i64,
    sys::LV2_ATOM__Long,
    |urids: &AtomURIDCollection| { urids.long }
);

/// A scalar atom containing a `i32` (`i32` on most platforms).
pub struct Int;

make_scalar_atom!(
    Int,
    i32,
    sys::LV2_ATOM__Int,
    |urids: &AtomURIDCollection| { urids.int }
);

/// A scalar atom representing a boolean.
///
/// Internally, this atom is represented by a `i32`, which is `==0` for `false` and `>= 1` for `true`
pub struct Bool;

make_scalar_atom!(
    Bool,
    i32,
    sys::LV2_ATOM__Bool,
    |urids: &AtomURIDCollection| { urids.bool }
);

/// A scalar atom containing a URID.
pub struct AtomURID;

make_scalar_atom!(
    AtomURID,
    URID,
    sys::LV2_ATOM__URID,
    |urids: &AtomURIDCollection| urids.urid
);

#[cfg(test)]
mod tests {
    use crate::atoms::scalar::ScalarAtom;
    use crate::prelude::*;
    use crate::space::*;
    use crate::AtomHeader;
    use std::convert::TryFrom;
    use urid::*;

    fn test_scalar<A: ScalarAtom>(value: A::InternalType)
    where
        A::InternalType: PartialEq<A::InternalType>,
        A::InternalType: std::fmt::Debug,
    {
        let map = HashURIDMapper::new();
        let urid: URID<A> = map.map_type().unwrap();

        let mut raw_space = VecSpace::<AtomHeader>::new_with_capacity(64);
        let raw_space = raw_space.as_space_mut();

        // writing
        {
            let mut space = SpaceCursor::new(raw_space.as_bytes_mut());
            space.init_atom(urid, value).unwrap();
        }

        // verifying
        {
            /// Generic version of the scalar atom structs.
            #[repr(C, align(8))]
            struct Scalar<B: Sized> {
                atom: sys::LV2_Atom,
                body: B,
            }

            let scalar: &Scalar<A::InternalType> =
                unsafe { raw_space.read().next_value().unwrap() };

            assert_eq!(scalar.atom.type_, urid);
            assert_eq!(scalar.atom.size as usize, 8); // All are always aligned and padded
            assert_eq!(scalar.body, value);
        }

        // reading
        {
            let read_value = unsafe { raw_space.read().next_atom() }
                .unwrap()
                .read(urid, ())
                .unwrap();

            assert_eq!(read_value, value);
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
