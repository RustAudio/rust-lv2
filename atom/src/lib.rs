//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod atom;

use urid::{URIDCache, URID};

#[derive(URIDCache)]
pub struct AtomURIDs {
    pub atom: URID<atom::Atom>,
}
