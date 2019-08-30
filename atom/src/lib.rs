//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

mod scalar;
mod vector;

pub mod space;
pub use scalar::*;
pub use vector::*;

use urid::{URIDCache, URID};

#[derive(URIDCache)]
pub struct AtomURIDCache {
    pub double: URID<Double>,
    pub float: URID<Float>,
    pub int: URID<Int>,
    pub long: URID<Long>,
    pub urid: URID<AtomURID>,
    pub vector: URID<Vector>,
    pub bool: URID<Bool>,
}
