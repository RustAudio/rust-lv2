//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod atomspace;
pub mod frame;
pub mod scalar;

use crate::atomspace::*;
use core::UriBound;
use urid::{URIDCache, URID};

#[derive(URIDCache)]
pub struct AtomURIDCache {
    double: URID<scalar::AtomDouble>,
    float: URID<scalar::AtomFloat>,
    int: URID<scalar::AtomInt>,
    long: URID<scalar::AtomLong>,
    urid: URID<scalar::AtomURID>,
}

pub trait AtomBody: UriBound {
    fn urid(urids: &AtomURIDCache) -> URID<Self>;

    fn create_ref(bytes: AtomSpace) -> Option<&Self>;
}
