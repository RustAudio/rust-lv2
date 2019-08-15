//! Thin but safe wrappers for the URID mapping features.
use crate::{URIDCache, URID};
use core::feature::Feature;
use core::UriBound;
use std::ffi::CStr;

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
    pub fn map_uri(&self, uri: &CStr) -> Option<URID> {
        let handle = self.internal.handle;
        let uri = uri.as_ptr();
        URID::new(unsafe { (self.internal.map.unwrap())(handle, uri) })
    }

    pub fn map_type<T: UriBound>(&self) -> Option<URID<T>> {
        let handle = self.internal.handle;
        let uri = T::URI.as_ptr() as *const i8;
        let urid = unsafe { (self.internal.map?)(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }

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
    /// This method capsules the raw mapping method provided by the host. Therefore, it may not be very fast or even capable of running in a real-time environment. Instead of calling this method every time you need a URID, you should call it once and cache it.
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

#[cfg(test)]
mod tests {}
