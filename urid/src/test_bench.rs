use crate::feature::*;
use core::feature::Feature;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::sync::Mutex;

type InternalMap = Mutex<HashMap<&'static CStr, u32>>;

unsafe extern "C" fn internal_mapping_fn(handle: *mut c_void, uri: *const i8) -> u32 {
    let mut handle = (*(handle as *mut InternalMap)).lock().unwrap();
    let uri = CStr::from_ptr(uri);
    if !handle.contains_key(uri) {
        let new_urid = handle.len() as u32 + 1;
        handle.insert(uri, new_urid);
    }
    handle[uri]
}

unsafe extern "C" fn internal_unmapping_fn(handle: *mut c_void, urid: u32) -> *const i8 {
    let handle = (*(handle as *mut InternalMap)).lock().unwrap();
    for key in handle.keys() {
        if handle[key] == urid {
            return key.as_ptr();
        }
    }
    std::ptr::null()
}

pub struct TestBench {
    pub internal_map: Box<InternalMap>,
    pub sys_map: sys::LV2_URID_Map,
    pub sys_unmap: sys::LV2_URID_Unmap,
}

impl TestBench {
    pub fn new() -> Self {
        let mut internal_map = Box::new(Mutex::new(HashMap::new()));
        let sys_map = sys::LV2_URID_Map {
            handle: internal_map.as_mut() as *mut InternalMap as *mut c_void,
            map: Some(internal_mapping_fn),
        };
        let sys_unmap = sys::LV2_URID_Unmap {
            handle: internal_map.as_mut() as *mut InternalMap as *mut c_void,
            unmap: Some(internal_unmapping_fn),
        };
        Self {
            internal_map,
            sys_map,
            sys_unmap,
        }
    }

    pub fn make_map<'a>(&'a mut self) -> Map<'a> {
        Map::from_raw_data(Some(&mut self.sys_map)).unwrap()
    }

    pub fn make_unmap<'a>(&'a mut self) -> Unmap<'a> {
        Unmap::from_raw_data(Some(&mut self.sys_unmap)).unwrap()
    }
}
