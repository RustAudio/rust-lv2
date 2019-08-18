//! Thin but safe wrappers for the URID mapping features.
use crate::{URIDCache, URID};
use core::feature::Feature;
use core::UriBound;
use std::ffi::CStr;
use std::os::raw::c_char;

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
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///
    ///     use lv2_urid::mapper::URIDMap;
    ///     use lv2_urid::URID;
    ///     use std::ffi::CStr;
    ///
    ///     // Creating a mapping feature.
    ///     // This is normally done by the host.
    ///     let mut raw_interface = URIDMap::new().make_map_interface();
    ///     let map = raw_interface.map();
    ///
    ///     // Creating the URI and mapping it to it's URID.
    ///     let uri: &CStr = CStr::from_bytes_with_nul(b"http://lv2plug.in\0").unwrap();
    ///     let urid: URID = map.map_uri(uri).unwrap();
    ///     assert_eq!(1, urid);
    pub fn map_uri(&self, uri: &CStr) -> Option<URID> {
        let handle = self.internal.handle;
        let uri = uri.as_ptr();
        let urid = unsafe { (self.internal.map.unwrap())(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }

    /// Return the URID of the given URI bound.
    ///
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///
    ///     use lv2_core::UriBound;
    ///     use lv2_urid::mapper::URIDMap;
    ///     use lv2_urid::URID;
    ///     use std::ffi::CStr;
    ///
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Creating a mapping feature.
    ///     // This is normally done by the host.
    ///     let mut raw_interface = URIDMap::new().make_map_interface();
    ///     let map = raw_interface.map();
    ///
    ///     // Mapping the type to it's URID.
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     assert_eq!(1, urid);
    pub fn map_type<T: UriBound>(&self) -> Option<URID<T>> {
        let handle = self.internal.handle;
        let uri = T::URI.as_ptr() as *const c_char;
        let urid = unsafe { (self.internal.map?)(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }

    /// Populate a URID cache.
    ///
    /// This is basically an alias for `T::from_map(self)` that makes the derive macro for `URIDCache` easier.
    pub fn populate_cache<T: URIDCache>(&self) -> Option<T> {
        T::from_map(self)
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
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it using a [`URIDCache`](trait.URIDCache.html).
    ///
    /// # Usage example:
    ///
    ///     use lv2_core::UriBound;
    ///     use lv2_urid::mapper::URIDMap;
    ///     use lv2_urid::URID;
    ///     use std::ffi::CStr;
    ///
    ///     struct MyUriBound;
    ///
    ///     unsafe impl UriBound for MyUriBound {
    ///         const URI: &'static [u8] = b"http://lv2plug.in\0";
    ///     }
    ///
    ///     // Creating a mapping feature.
    ///     // This is normally done by the host.
    ///     let host_map = URIDMap::new();
    ///
    ///     let mut raw_map_interface = host_map.make_map_interface();
    ///     let map = raw_map_interface.map();
    ///
    ///     let mut raw_unmap_interface = host_map.make_unmap_interface();
    ///     let unmap = raw_unmap_interface.unmap();
    ///
    ///     // Mapping the type to it's URID, and then back to it's URI.
    ///     let urid: URID<MyUriBound> = map.map_type::<MyUriBound>().unwrap();
    ///     let uri: &CStr = unmap.unmap(urid).unwrap();
    ///     assert_eq!(MyUriBound::uri(), uri);
    pub fn unmap<T>(&self, urid: URID<T>) -> Option<&CStr> {
        let handle = self.internal.handle;
        let uri_ptr = unsafe { (self.internal.unmap.unwrap())(handle, urid.get()) };
        if uri_ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(uri_ptr) })
        }
    }
}
