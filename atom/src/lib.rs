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
//! use lv2_urid::prelude::*;
//! use lv2_units::prelude::*;
//!
//! #[derive(PortContainer)]
//! struct MyPorts {
//!     input: InputPort<AtomPort>,
//!     output: OutputPort<AtomPort>,
//! }
//!
//! #[derive(URIDCache)]
//! struct MyURIDs {
//!     atom: AtomURIDCache,
//!     units: UnitURIDCache,
//! }
//!
//! /// Something like a plugin's run method.
//! fn run(ports: &mut MyPorts, urids: &MyURIDs) {
//!     // Get the read handle to the sequence.
//!     let input_sequence = ports.input.read(
//!         urids.atom.sequence,
//!         urids.units.beat
//!     ).unwrap();
//!
//!     // Get the write handle to the sequence.
//!     let mut output_sequence = ports.output.init(
//!         urids.atom.sequence,
//!         TimeStampURID::Frames(urids.units.frame)
//!     ).unwrap();
//!
//!     // Iterate through all events in the input sequence.
//!     for event in input_sequence {
//!         // An event contains a timestamp and an atom.
//!         let (timestamp, atom) = event;
//!         // If the read atom is a 32-bit integer...
//!         if let Some(integer) = atom.read(urids.atom.int, ()) {
//!             // Multiply it by two and write it to the sequence.
//!             output_sequence.init(timestamp, urids.atom.int, integer * 2).unwrap();
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
//!
//! # Non-Rust requirements
//!
//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
extern crate lv2_core as core;
extern crate lv2_sys as sys;
extern crate lv2_units as units;
extern crate lv2_urid as urid;

pub mod chunk;
pub mod object;
pub mod port;
pub mod scalar;
pub mod sequence;
pub mod space;
pub mod string;
pub mod tuple;
pub mod vector;

/// Prelude of `lv2_atom` for wildcard usage.
pub mod prelude {
    use crate::*;

    pub use crate::{Atom, AtomURIDCache, UnidentifiedAtom};
    pub use chunk::Chunk;
    pub use object::{Object, ObjectHeader, PropertyHeader};
    pub use port::AtomPort;
    pub use scalar::{AtomURID, Bool, Double, Float, Int, Long};
    pub use sequence::{Sequence, TimeStamp, TimeStampURID};
    pub use string::{Literal, LiteralInfo, String};
    pub use tuple::Tuple;
    pub use vector::Vector;
}

use core::UriBound;
use space::*;
use urid::{URIDCache, URID};

#[derive(Clone, URIDCache)]
/// Container with the URIDs of all `UriBound`s in this crate.
pub struct AtomURIDCache {
    pub double: URID<scalar::Double>,
    pub float: URID<scalar::Float>,
    pub int: URID<scalar::Int>,
    pub long: URID<scalar::Long>,
    pub urid: URID<scalar::AtomURID>,
    pub bool: URID<scalar::Bool>,
    pub vector: URID<vector::Vector<scalar::Int>>,
    pub chunk: URID<chunk::Chunk>,
    pub literal: URID<string::Literal>,
    pub object: URID<object::Object>,
    pub property: URID<object::Property>,
    pub string: URID<string::String>,
    pub tuple: URID<tuple::Tuple>,
    pub sequence: URID<sequence::Sequence>,
}

/// Atom type.
///
/// This is the foundation of this crate: Types that implement `Atom` define the reading and writing functions for an atom type. However, these types will never be constructed; They are only names to be used for generic type arguments.
///
/// This trait has two lifetime parameters: The first one is the lifetime of the atom in memory. In practice, this will often be `'static`, but it's good to keep it generic for testing purposes. The second parameter is the lifetime of the `MutSpace` borrowed by the `FramedMutSpace` parameter in the `write` method. Since the `WriteParameter` may contain this `FramedMutSpace`, it has to be assured that it lives long enough. Since the referenced `MutSpace` also has to borrow the atom, it may not live longer than the atom.
pub trait Atom<'a, 'b>: UriBound
where
    'a: 'b,
{
    /// The atom-specific parameter of the `read` function.
    ///
    /// If your atom does not need a reading parameter, you may set it to `()`.
    type ReadParameter;

    /// The return value of the `read` function.
    ///
    /// It may contain a reference to the atom and therefore may not outlive it.
    type ReadHandle: 'a;

    /// The atom-specific parameter of the `write` function.
    ///
    /// If your atom does not need a writing parameter, you may set it to `()`.
    type WriteParameter;

    /// The return value of the `write` function.
    ///
    /// It may contain a reference to a `MutSpace` and therefore may not outlive it.
    type WriteHandle: 'b;

    /// Read the body of the atom.
    ///
    /// The passed space exactly covers the body of the atom, excluding the header. You may assume that the body is actually of your atom type, since the URID of the atom was checked beforehand.
    ///
    /// If the atom is malformed, you may not panic and return `None` instead.
    fn read(body: Space<'a>, parameter: Self::ReadParameter) -> Option<Self::ReadHandle>;

    /// Initialize the body of the atom.
    ///
    /// In this method, the atom is prepared for the writing handle. Usually, the atom will not be
    /// valid when initializied; Users have to use the write handle to make it valid.
    ///
    /// The frame of the atom was already initialized, containing the URID.
    ///
    /// If space is insufficient, you may not panic and return `None` instead. The written results are assumed to be malformed.
    fn init(
        frame: FramedMutSpace<'a, 'b>,
        parameter: Self::WriteParameter,
    ) -> Option<Self::WriteHandle>;
}

/// An atom of yet unknown type.
///
/// This is used by reading handles that have to return a reference to an atom, but can not check it's type. This struct contains a `Space` containing the header and the body of the atom and can identify/read the atom from it.
#[derive(Clone, Copy)]
pub struct UnidentifiedAtom<'a> {
    space: Space<'a>,
}

impl<'a> UnidentifiedAtom<'a> {
    /// Construct a new unidentified atom.
    ///
    /// The space actually has to contain an atom. If it doesn't, crazy (but not undefined) things can happen.
    pub fn new(space: Space<'a>) -> Self {
        Self { space }
    }

    /// Try to read the atom.
    ///
    /// To identify the atom, it's URID and an atom-specific parameter is needed. If the atom was identified, a reading handle is returned.
    pub fn read<'b, A: Atom<'a, 'b>>(
        self,
        urid: URID<A>,
        parameter: A::ReadParameter,
    ) -> Option<A::ReadHandle> {
        self.space
            .split_atom_body(urid)
            .map(|(body, _)| body)
            .and_then(|body| A::read(body, parameter))
    }
}
