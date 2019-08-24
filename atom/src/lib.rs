//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod atomspace;
pub mod frame;
pub mod scalar;

use crate::atomspace::*;
use crate::frame::AtomWritingFrame;
use core::UriBound;
use urid::{URIDCache, URID};

#[derive(URIDCache)]
pub struct AtomURIDCache {
    double: URID<scalar::Double>,
    float: URID<scalar::Float>,
    int: URID<scalar::Int>,
    long: URID<scalar::Long>,
    urid: URID<scalar::AtomURID>,
}

pub trait AtomBody: UriBound + Sized {
    type InitializationParameter: ?Sized;

    fn urid(urids: &AtomURIDCache) -> URID<Self>;

    fn retrieve(bytes: AtomSpace) -> Option<Self>;

    fn initialize_frame(
        frame: &mut AtomWritingFrame<Self>,
        param: &Self::InitializationParameter,
    ) -> bool;
}
