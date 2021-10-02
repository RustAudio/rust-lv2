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
//! use lv2_atom::space::error::AtomError;
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
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) -> Result<(), AtomError> {
//!     // Get the read handle to the sequence.
//!     let input_sequence = ports.input
//!         .read(urids.atom.sequence)?
//!         .with_unit(urids.units.frame)?;
//!
//!     // Get the write handle to the sequence.
//!     let mut output_sequence = ports.output
//!         .write(urids.atom.sequence)?
//!         .with_unit(urids.units.frame)?;
//!
//!     // Iterate through all events in the input sequence.
//!     // An event contains a timestamp and an atom.
//!     for (timestamp, atom) in input_sequence {
//!         // If the read atom is a 32-bit integer...
//!         if let Ok(integer) = atom.read(urids.atom.int) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.new_event(timestamp, urids.atom.int)?.set(*integer * 2)?;
//!         } else {
//!             // Forward the atom to the sequence without a change.
//!             output_sequence.forward(timestamp, atom)?;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Internals
//!
//! Internally, all atoms are powered by the structs in the [`space`](space/index.html) module. They safely abstract the reading and writing process and assure that no memory is improperly accessed or leaked and that alignments are upheld. If you simply want to use the atoms in this crate, you don't need to deal with. They are only interesting if you want to create your own atom types.

#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]

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

mod unidentified;
pub(crate) mod util;
pub use unidentified::UnidentifiedAtom;

/// Prelude of `lv2_atom` for wildcard usage.
pub mod prelude {
    pub use atoms::{
        chunk::Chunk,
        object::{Object, ObjectHeader, PropertyHeader},
        scalar::{AtomURID, Bool, Double, Float, Int, Long},
        sequence::Sequence,
        string::{Literal, LiteralInfo, String},
        tuple::Tuple,
        vector::Vector,
    };
    pub use port::AtomPort;

    use crate::*;
    pub use crate::{atoms::AtomURIDCollection, Atom, UnidentifiedAtom};
}

/// A special prelude re-exporting all utilities to implement custom atom types.
pub mod atom_prelude {
    pub use crate::prelude::*;

    pub use crate::space::{
        error::{AlignmentError, AtomError, AtomReadError, AtomWriteError},
        AlignedSpace, AtomSpace, AtomWriter, SpaceAllocator, SpaceCursor, SpaceWriter, Terminated,
        VecSpace,
    };
    pub use crate::{Atom, AtomHandle, AtomHeader, UnidentifiedAtom};
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
    /// # Errors
    /// This method may return any error if the atom in the given space is somehow malformed, or if
    /// there wasn't enough space to read it properly.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that the given [`AtomSpace`] contains a valid instance of this atom,
    /// or the resulting `ReadHandle` will be completely invalid, triggering Undefined Behavior.
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
    /// # Errors
    ///
    /// This method may return an error if the buffer is out of space, or if any invalid state is
    /// observed. In those cases, the written data may be incomplete and should be discarded.
    ///
    fn write(
        writer: AtomWriter,
    ) -> Result<<Self::WriteHandle as AtomHandle>::Handle, AtomWriteError>;
}

/// An Atom super-trait that allows to get a byte slice from an atom's read handle.
///
/// Some LV2 APIs (such as `Option`) request a data pointer to the value of a given atom type, but
/// in many cases that pointer can be simply retrieved from a reference to a raw value. Most notably,
/// pointers to any scalar value (e.g. `&i32`) can be safely turned into a byte slice (`&[u8]).
///
/// However, not all atoms have this capability, hence the need for a separate trait that is not
/// implemented for all types.
///
/// # Example
///
/// ```
/// use lv2_atom::atoms::scalar::Int;
/// use lv2_atom::AtomAsBytes;
///
/// let value: i32 = 42;
/// let bytes: &[u8] = Int::read_as_bytes(&value);
///
/// assert_eq!(bytes.len(), ::core::mem::size_of::<i32>())
/// ```
pub trait AtomAsBytes: Atom {
    /// Returns the type returned by an Atom's read handle as a byte slice.
    #[allow(clippy::needless_lifetimes)] // Clippy false positive
    fn read_as_bytes<'a>(handle: <Self::ReadHandle as AtomHandle<'a>>::Handle) -> &'a [u8];
}
