//! Since this crate depends on `-sys` crates that use `bindgen` to create the C API bindings,
//! you need to have clang installed on your machine.
pub extern crate lv2_atom_sys as sys;
extern crate lv2_core as core;
extern crate lv2_urid as urid;

pub mod scalar;

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

    unsafe fn create_ref(bytes: &[u8]) -> Option<&Self>;
}

pub struct UnidentifiedAtom<'a> {
    type_urid: URID,
    data: &'a [u8],
}

impl<'a> UnidentifiedAtom<'a> {
    pub fn new(type_urid: URID, data: &'a [u8]) -> Self {
        Self { type_urid, data }
    }

    pub fn identify<T: AtomBody>(&self, urids: &AtomURIDCache) -> Option<&T> {
        if self.type_urid == T::urid(urids) {
            unsafe { T::create_ref(self.data) }
        } else {
            None
        }
    }
}
