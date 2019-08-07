use core::feature::Feature;
use core::UriBound;
use std::ffi::CStr;

pub use sys::LV2_URID as URID;

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
    pub fn map(&self, uri: &CStr) -> URID {
        let handle = self.internal.handle;
        let uri = uri.as_ptr();
        unsafe { (self.internal.map.unwrap())(handle, uri) }
    }
}

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
    pub fn unmap(&self, urid: URID) -> &CStr {
        let handle = self.internal.handle;
        unsafe {
            let uri = (self.internal.unmap.unwrap())(handle, urid);
            CStr::from_ptr(uri)
        }
    }
}
