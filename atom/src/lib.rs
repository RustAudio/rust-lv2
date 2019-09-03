//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod scalar;
pub mod vector;
pub mod chunk;
pub mod literal;

pub mod space;

use urid::{URIDCache, URID};

#[derive(URIDCache)]
/// Container for the URIDs of all `UriBound`s in this crate.
pub struct AtomURIDCache {
    pub double: URID<scalar::Double>,
    pub float: URID<scalar::Float>,
    pub int: URID<scalar::Int>,
    pub long: URID<scalar::Long>,
    pub urid: URID<scalar::AtomURID>,
    pub bool: URID<scalar::Bool>,
    pub vector: URID<vector::Vector>,
    pub chunk: URID<chunk::Chunk>,
    pub literal: URID<literal::Literal>,
}
