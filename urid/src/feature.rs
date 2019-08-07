//! Thin but safe wrappers for the URID mapping features.
use core::feature::Feature;
use core::UriBound;
use std::ffi::CStr;

pub use sys::LV2_URID as URID;

/// Host feature to map URIs to integers
pub struct Map<'a> {
    internal: &'a sys::LV2_URID_Map,
}

unsafe impl<'a> UriBound for Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

impl<'a> Feature<'a> for Map<'a> {
    type RawDataType = sys::LV2_URID_Map;

    fn from_raw_data(data: Option<&'a mut sys::LV2_URID_Map>) -> Option<Self> {
        if let Some(internal) = data {
            Some(Self { internal })
        } else {
            None
        }
    }
}

impl<'a> Map<'a> {
    /// Return the URID of the given URI.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it.
    pub fn map(&self, uri: &CStr) -> URID {
        let handle = self.internal.handle;
        let uri = uri.as_ptr();
        unsafe { (self.internal.map.unwrap())(handle, uri) }
    }
}

/// Host feature to revert the URI -> URID mapping.
pub struct Unmap<'a> {
    internal: &'a sys::LV2_URID_Unmap,
}

unsafe impl<'a> UriBound for Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

impl<'a> Feature<'a> for Unmap<'a> {
    type RawDataType = sys::LV2_URID_Unmap;

    fn from_raw_data(data: Option<&'a mut sys::LV2_URID_Unmap>) -> Option<Self> {
        if let Some(internal) = data {
            Some(Self { internal })
        } else {
            None
        }
    }
}

impl<'a> Unmap<'a> {
    /// Return the URI of the given URID.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it.
    pub fn unmap(&self, urid: URID) -> &CStr {
        let handle = self.internal.handle;
        unsafe {
            let uri = (self.internal.unmap.unwrap())(handle, urid);
            CStr::from_ptr(uri)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::feature::*;
    use std::collections::HashMap;
    use std::ffi::{c_void, CStr};

    unsafe extern "C" fn internal_mapping_fn(handle: *mut c_void, uri: *const i8) -> URID {
        let handle = (handle as *mut HashMap<&CStr, URID>).as_mut().unwrap();
        let uri = CStr::from_ptr(uri);
        if !handle.contains_key(uri) {
            handle.insert(uri, handle.len() as u32);
        }
        handle[uri]
    }

    unsafe extern "C" fn internal_unmapping_fn(handle: *mut c_void, urid: URID) -> *const i8 {
        let handle = (handle as *mut HashMap<&CStr, URID>).as_mut().unwrap();
        for key in handle.keys() {
            if handle[key] == urid {
                return key.as_ptr();
            }
        }
        std::ptr::null()
    }

    #[test]
    fn test_map_unmap() {
        let mut internal_map: HashMap<&CStr, URID> = HashMap::new();
        let mut sys_map = sys::LV2_URID_Map {
            handle: (&mut internal_map) as *mut _ as *mut c_void,
            map: Some(internal_mapping_fn),
        };
        let mut sys_unmap = sys::LV2_URID_Unmap {
            handle: (&mut internal_map) as *mut _ as *mut c_void,
            unmap: Some(internal_unmapping_fn),
        };

        let map = Map::from_raw_data(Some(&mut sys_map)).unwrap();
        let unmap = Unmap::from_raw_data(Some(&mut sys_unmap)).unwrap();

        let uri_a = CStr::from_bytes_with_nul(b"urn:my-uri-a\0").unwrap();
        let uri_b = CStr::from_bytes_with_nul(b"urn:my_uri-b\0").unwrap();

        assert_eq!(0, map.map(uri_a));
        assert_eq!(1, map.map(uri_b));
        assert_eq!(0, map.map(uri_a));

        assert_eq!(uri_a, unmap.unmap(0));
        assert_eq!(uri_b, unmap.unmap(1));
    }
}
