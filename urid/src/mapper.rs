//! Implementation of the mapping feature for testing purposes.
use crate::URID;
use crate::{Map, Unmap};
use core::feature::Feature;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::pin::Pin;

/// A working URI → URID mapper.
///
/// This mapper is able to map URIs (technically even every string) to URIDs. Since it's map store is hidden behind a mutex and an `Arc`, it can be cloned and accessed from any thread at any time.
#[derive(Clone)]
pub struct URIDMap(Arc<Mutex<HashMap<&'static CStr, URID>>>);

impl URIDMap {
    /// Create a new URID map store.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    /// Map a URI to a URID.
    ///
    /// If the URI has not been mapped before, a new URID will be assigned. Please note that this method may block the thread since it tries to lock an internal mutex. You should therefore never call this method in a performance or real-time-critical context.
    pub fn map(&self, uri: &'static CStr) -> URID {
        let mut map = self.0.lock().unwrap();
        let next_urid = URID::new(map.len() as u32 + 1).unwrap();
        *map.entry(uri).or_insert(next_urid)
    }

    /// Unsafe interface version of `map`.
    unsafe extern "C" fn extern_map(handle: *mut c_void, uri: *const i8) -> u32 {
        let handle = if let Some(handle) = (handle as *mut Self).as_ref() {
            handle
        } else {
            return 0;
        };

        if uri.is_null() {
            return 0;
        }
        let uri = CStr::from_ptr(uri);

        handle.map(uri).get()
    }

    /// Try to find the URI which is mapped to the given URID.
    ///
    /// In this implementation, this is failable: If the given URID has not been assigned to URI, this method will return `None`. Please note that this method may block the thread since it tries to lock an internal mutex. You should therefore never call this method in a performance or real-time-critical context.
    pub fn unmap(&self, urid: URID) -> Option<&'static CStr> {
        let map = self.0.lock().unwrap();
        for (uri, contained_urid) in map.iter() {
            if *contained_urid == urid {
                return Some(uri);
            }
        }
        None
    }

    /// Unsafe interface version of `unmap`.
    unsafe extern "C" fn extern_unmap(handle: *mut c_void, urid: u32) -> *const i8 {
        let handle = if let Some(handle) = (handle as *mut Self).as_ref() {
            handle
        } else {
            return null();
        };
        let urid = if let Some(urid) = URID::new(urid) {
            urid
        } else {
            return null();
        };

        handle.unmap(urid).map(|uri| uri.as_ptr()).unwrap_or(null())
    }

    /// Create an interface for the `map` feature.
    ///
    /// This is accomplished by cloning the smart pointer to the URID store, storing the copy in a `Box` and creating a raw interface struct pointing to the copied smart pointer.
    pub fn make_map_interface(&self) -> MapInterface {
        let mut map = Box::pin(self.clone());
        let raw_map = sys::LV2_URID_Map {
            handle: map.as_mut().get_mut() as *mut _ as *mut c_void,
            map: Some(Self::extern_map),
        };
        MapInterface { _map: map, raw_map }
    }

    /// Create an interface for the `unmap` feature.
    ///
    /// This is accomplished by cloning the smart pointer to the URID store, storing the copy in a `Box` and creating a raw interface struct pointing to the copied smart pointer.
    pub fn make_unmap_interface(&self) -> UnmapInterface {
        let mut map = Box::pin(self.clone());
        let raw_unmap = sys::LV2_URID_Unmap {
            handle: map.as_mut().get_mut() as *mut _ as *mut c_void,
            unmap: Some(Self::extern_unmap),
        };
        UnmapInterface {
            _map: map,
            raw_unmap,
        }
    }
}

impl Default for URIDMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Copy of a `URIDMap` to ensure the validity of a `sys::LV2_URID_Map`.
pub struct MapInterface {
    _map: Pin<Box<URIDMap>>,
    raw_map: sys::LV2_URID_Map,
}

impl MapInterface {
    /// Return a mutable reference to the raw mapping feature.
    pub fn raw_map(&mut self) -> &mut sys::LV2_URID_Map {
        &mut self.raw_map
    }

    /// Return a safe mapping feature instance
    pub fn map(&mut self) -> Map {
        Map::from_raw_data(Some(self.raw_map())).unwrap()
    }
}

/// Copy of a `URIDMap` to ensure the validity of a `sys::LV2_URID_Unmap`.
pub struct UnmapInterface {
    _map: Pin<Box<URIDMap>>,
    raw_unmap: sys::LV2_URID_Unmap,
}

impl UnmapInterface {
    /// Return a mutable reference to the raw unmapping feature.
    pub fn raw_unmap(&mut self) -> &mut sys::LV2_URID_Unmap {
        &mut self.raw_unmap
    }

    /// Return a safe unmapping feature instance
    pub fn unmap(&mut self) -> Unmap {
        Unmap::from_raw_data(Some(self.raw_unmap())).unwrap()
    }
}
