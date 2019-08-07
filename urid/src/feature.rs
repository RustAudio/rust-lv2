use core::feature::Feature;
use core::UriBound;
use std::ffi::CStr;

pub use sys::LV2_URID as URID;

pub struct Map<'a> {
    internal: &'a mut sys::LV2_URID_Map,
}

unsafe impl<'a> UriBound for Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

impl<'a> Feature for Map<'a> {
    type RawDataType = sys::LV2_URID_Map;

    unsafe fn from_raw_data(data: *mut sys::LV2_URID_Map) -> Self {
        Self {
            internal: data.as_mut().unwrap(),
        }
    }

    fn raw_data(&mut self) -> *mut sys::LV2_URID_Map {
        self.internal
    }
}

impl<'a> Map<'a> {
    pub fn map(&self, uri: &CStr) -> URID {
        let handle = self.internal.handle;
        let uri = uri.as_ptr();
        unsafe { (self.internal.map.unwrap())(handle, uri) }
    }
}

pub struct Unmap<'a> {
    internal: &'a mut sys::LV2_URID_Unmap,
}

unsafe impl<'a> UriBound for Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

impl<'a> Feature for Unmap<'a> {
    type RawDataType = sys::LV2_URID_Unmap;

    unsafe fn from_raw_data(data: *mut sys::LV2_URID_Unmap) -> Self {
        Self {
            internal: data.as_mut().unwrap(),
        }
    }

    fn raw_data(&mut self) -> *mut sys::LV2_URID_Unmap {
        self.internal
    }
}

impl<'a> Unmap<'a> {
    pub fn unmap(&self, urid: URID) -> &CStr {
        let handle = self.internal.handle;
        unsafe {
            let uri = (self.internal.unmap.unwrap())(handle, urid);
            CStr::from_ptr(uri)
        }
    }
}
