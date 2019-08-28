//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod space;
pub mod scalar;

use urid::{URIDCache, URID};

#[derive(URIDCache)]
pub struct AtomURIDCache {
    double: URID<scalar::Double>,
    float: URID<scalar::Float>,
    int: URID<scalar::Int>,
    long: URID<scalar::Long>,
    urid: URID<scalar::AtomURID>,
}
