//! General data IO for LV2 plugins.
//!
//! This crate empowers LV2 plugins to communicate using a common type system, which is defined in the [LV2 Atom Specification](http://lv2plug.in/ns/ext/atom/atom.html). Many plugin standards only provide audio and MIDI IO, but LV2 plugins can read and write anything from simple integers over vectors and strings to event sequences using this specification.
//!
//! # How to use atoms
//!
//! The foundation of this crate is the [`Atom`](trait.Atom.html) trait. This trait provides type definitions and methods to read and write atoms. However, you will never handle these types directly, only via handles and generics.
//!
//! Your entry point to the atom system are the [`PortReader`](port/struct.PortReader.html) and [`PortWriter`](port/struct.PortWriter.html) structs provided by your plugin's ports. If you provide them the URID of the desired atom type as a atom-specific parameter, they will try to retrieve a handle that either lets you access the contents of an atom or write additional data to it. This is a general pattern in this crate; you will encounter it several times. From there, you use the handles as documented.
//!
//! # Example
//!
//! ```
//! use lv2_atom::prelude::*;
//! use lv2_core::prelude::*;
//! use lv2_units::prelude::*;
//! use urid::*;
//!
//! #[derive(PortCollection)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCollection)]
//! struct MyURIDs {
//!     atom: AtomURIDCollection,
//!     units: UnitURIDCollection,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) {
//!     // Get the read handle to the sequence.
//!     let input_sequence = ports.input
//!         .read(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame)
//!         .unwrap();
//!
//!     // Get the write handle to the sequence.
//!     let mut output_sequence = ports.output.init(urids.atom.sequence)
//!         .unwrap()
//!         .with_unit(urids.units.frame)
//!         .unwrap();
//!
//!     // Iterate through all events in the input sequence.
//!     // An event contains a timestamp and an atom.
//!     for (timestamp, atom) in input_sequence {
//!         // If the read atom is a 32-bit integer...
//!         if let Ok(integer) = atom.read(urids.atom.int) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.new_event(timestamp, urids.atom.int).unwrap().set(*integer * 2).unwrap();
//!         } else {
//!             // Forward the atom to the sequence without a change.
//!             output_sequence.forward(timestamp, atom).unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! # Internals
//!
//! Internally, all atoms are powered by the structs in the [`space`](space/index.html) module. They safely abstract the reading and writing process and assure that no memory is improperly accessed or leaked and that alignments are upheld. If you simply want to use the atoms in this crate, you don't need to deal with. They are only interesting if you want to create your own atom types.
extern crate lv2_sys as sys;
extern crate lv2_units as units;

use crate::space::error::{AtomReadError, AtomWriteError};
pub use header::AtomHeader;
use space::*;
use urid::*;

pub mod atoms;
mod header;
#[cfg(feature = "lv2-core")]
pub mod port;
pub mod space;

pub(crate) mod util;

/// Prelude of `lv2_atom` for wildcard usage.
pub mod prelude {
    pub use atoms::chunk::Chunk;
    pub use atoms::object::{Object, ObjectHeader, PropertyHeader};
    pub use atoms::scalar::{AtomURID, Bool, Double, Float, Int, Long};
    pub use atoms::sequence::Sequence;
    pub use atoms::string::{Literal, LiteralInfo, String};
    pub use atoms::tuple::Tuple;
    pub use atoms::vector::Vector;
    pub use port::AtomPort;
    pub use space::{AlignedSpace, AtomSpace, AtomSpaceWriter, SpaceAllocator};

    use crate::*;
    pub use crate::{atoms::AtomURIDCollection, Atom, UnidentifiedAtom};
}

pub trait AtomHandle<'a> {
    type Handle: 'a;
}

/// Atom type.
///
/// This is the foundation of this crate: Types that implement `Atom` define the reading and writing functions for an atom type. However, these types will never be constructed; They are only names to be used for generic type arguments.
///
/// This trait has two lifetime parameters: The first one is the lifetime of the atom in memory. In practice, this will often be `'static`, but it's good to keep it generic for testing purposes. The second parameter is the lifetime of the `MutSpace` borrowed by the `FramedMutSpace` parameter in the `write` method. Since the `WriteParameter` may contain this `FramedMutSpace`, it has to be assured that it lives long enough. Since the referenced `MutSpace` also has to borrow the atom, it may not live longer than the atom.
pub trait Atom: UriBound {
    /// The return value of the `read` function.
    ///
    /// It may contain a reference to the atom and therefore may not outlive it.
    type ReadHandle: for<'a> AtomHandle<'a>;

    /// The return value of the `write` function.
    ///
    /// It may contain a reference to a `MutSpace` and therefore may not outlive it.
    type WriteHandle: for<'a> AtomHandle<'a>;

    /// Reads the body of the atom.
    ///
    /// The passed space exactly covers the body of the atom, excluding the header.
    ///
    /// If the atom is malformed, this method returns `None`.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that the given [`Space`] contains a valid instance of this atom,
    /// or the resulting `ReadHandle` will be completely invalid, and Undefined Behavior will happen.
    unsafe fn read(
        body: &AtomSpace,
    ) -> Result<<Self::ReadHandle as AtomHandle>::Handle, AtomReadError>;

    /// Initialize the body of the atom.
    ///
    /// In this method, the atom is prepared for the writing handle. Usually, the atom will not be
    /// valid when initialized; Users have to use the write handle to make it valid.
    ///
    /// The frame of the atom was already initialized, containing the URID.
    ///
    /// If space is insufficient, you may not panic and return `None` instead. The written results are assumed to be malformed.
    fn init(
        writer: AtomSpaceWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError>;
}

/// An atom of yet unknown type.
///
/// This is used by reading handles that have to return a reference to an atom, but can not check it's type. This struct contains a `Space` containing the header and the body of the atom and can identify/read the atom from it.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct UnidentifiedAtom {
    header: AtomHeader,
}

impl UnidentifiedAtom {
    /// Construct a new unidentified atom.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the given space actually contains both a valid atom header, and a valid corresponding atom body.
    #[inline]
    pub unsafe fn from_space(space: &AtomSpace) -> Result<&Self, AtomReadError> {
        Ok(Self::from_header(space.read().next_value()?))
    }

    /// Construct a new unidentified atom.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the given space actually contains both a valid atom header, and a valid corresponding atom body.
    #[inline]
    pub unsafe fn from_space_mut(space: &mut AtomSpace) -> Result<&mut Self, AtomWriteError> {
        let available = space.len();

        Ok(Self::from_header_mut(
            space
                .assume_init_slice_mut()
                .get_mut(0)
                .ok_or(AtomWriteError::WritingOutOfBounds {
                    available,
                    requested: ::core::mem::size_of::<AtomHeader>(),
                })?,
        ))
    }

    #[inline]
    pub(crate) unsafe fn from_header(header: &AtomHeader) -> &Self {
        // SAFETY: UnidentifiedAtom is repr(C) and has AtomHeader as its only field, so transmuting between the two is safe.
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub(crate) unsafe fn from_header_mut(header: &mut AtomHeader) -> &mut Self {
        // SAFETY: UnidentifiedAtom is repr(C) and has AtomHeader as its only field, so transmuting between the two is safe.
        &mut *(header as *mut _ as *mut _)
    }

    /// Try to read the atom.
    ///
    /// To identify the atom, its URID and an atom-specific parameter is needed. If the atom was identified, a reading handle is returned.
    pub fn read<A: Atom>(
        &self,
        urid: URID<A>,
    ) -> Result<<A::ReadHandle as AtomHandle>::Handle, AtomReadError> {
        self.header.check_urid(urid)?;

        // SAFETY: the fact that this contains a valid instance of A is checked above.
        unsafe { A::read(self.body()) }
    }

    #[inline]
    pub fn header(&self) -> &AtomHeader {
        &self.header
    }

    #[inline]
    fn body_bytes(&self) -> &[u8] {
        if self.header.size_of_body() == 0 {
            &[]
        } else {
            // SAFETY: This type's constructor ensures the atom's body is valid
            // The edge case of an empty body is also checked above.
            let ptr = unsafe { (self as *const UnidentifiedAtom).add(1) };

            // SAFETY: This type's constructor ensures the atom's body is valid
            unsafe { ::core::slice::from_raw_parts(ptr.cast(), self.header.size_of_body()) }
        }
    }

    #[inline]
    fn body_bytes_mut(&mut self) -> &mut [u8] {
        if self.header.size_of_body() == 0 {
            &mut []
        } else {
            // SAFETY: This type's constructor ensures the atom's body is valid
            // The edge case of an empty body is also checked above.
            let ptr = unsafe { (self as *mut UnidentifiedAtom).add(1) };

            // SAFETY: This type's constructor ensures the atom's body is valid
            unsafe { ::core::slice::from_raw_parts_mut(ptr.cast(), self.header.size_of_body()) }
        }
    }

    #[inline]
    pub fn atom_space(&self) -> &AtomSpace {
        let ptr = self as *const UnidentifiedAtom as *const u8;
        let bytes = unsafe { ::core::slice::from_raw_parts(ptr, self.header.size_of_atom()) };

        // SAFETY: the bytes are necessarily aligned, since they point to the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_unchecked(bytes) }
    }

    #[inline]
    pub fn atom_space_mut(&mut self) -> &mut AtomSpace {
        let ptr = self as *mut UnidentifiedAtom as *mut u8;
        let bytes = unsafe { ::core::slice::from_raw_parts_mut(ptr, self.header.size_of_atom()) };

        // SAFETY: the bytes are necessarily aligned, since they point to the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_mut_unchecked(bytes) }
    }

    #[inline]
    pub fn body(&self) -> &AtomSpace {
        // SAFETY: the bytes are necessarily aligned, since they are right after the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_unchecked(self.body_bytes()) }
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut AtomSpace {
        // SAFETY: the bytes are necessarily aligned, since they are right after the aligned AtomHeader
        unsafe { AtomSpace::from_bytes_mut_unchecked(self.body_bytes_mut()) }
    }
}
