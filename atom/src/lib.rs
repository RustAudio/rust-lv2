//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod chunk;
pub mod object;
pub mod scalar;
pub mod space;
pub mod string;
pub mod tuple;
pub mod vector;

use space::*;
use urid::{URIDBound, URIDCache, URID};

#[derive(Clone, URIDCache)]
/// Container for the URIDs of all `UriBound`s in this crate.
pub struct AtomURIDCache {
    pub double: URID<scalar::Double>,
    pub float: URID<scalar::Float>,
    pub int: URID<scalar::Int>,
    pub long: URID<scalar::Long>,
    pub urid: URID<scalar::AtomURID>,
    pub bool: URID<scalar::Bool>,
    pub vector: URID<vector::Vector<scalar::Int>>,
    pub chunk: URID<chunk::Chunk>,
    pub string_literal: URID<string::StringLiteral>,
    pub data_literal: URID<string::DataLiteral>,
    pub object: URID<object::Object>,
    pub property: URID<object::Property>,
    pub string: URID<string::String>,
    pub tuple: URID<tuple::Tuple>,
}

pub trait Atom<'a, 'b>: URIDBound {
    type ReadParameter;
    type ReadHandle: 'a;
    type WriteParameter;
    type WriteHandle: 'b;

    /// Read the body of the atom.
    ///
    /// The passed space exactly covers the body of the atom.
    fn read(body: Space<'a>, parameter: Self::ReadParameter) -> Option<Self::ReadHandle>;

    /// Write the body of the atom.
    ///
    /// The passed frame contains the header of the atom.
    fn write(
        frame: FramedMutSpace<'a, 'b>,
        parameter: Self::WriteParameter,
    ) -> Option<Self::WriteHandle>;
}

#[derive(Clone, Copy)]
pub struct UnidentifiedAtom<'a> {
    space: Space<'a>,
}

impl<'a> UnidentifiedAtom<'a> {
    pub fn new(space: Space<'a>) -> Self {
        Self { space }
    }

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
