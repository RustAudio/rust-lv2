//! Thin but safe wrappers for the URID mapping features.
use core::feature::Feature;
use core::prelude::*;
use std::ffi::c_void;
use urid::*;

/// Host feature to map URIs to integers
#[repr(transparent)]
pub struct LV2Map<'a> {
    internal: &'a sys::LV2_URID_Map,
}

unsafe impl<'a> UriBound for LV2Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

unsafe impl<'a> Feature for LV2Map<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        if class != ThreadingClass::Audio {
            (feature as *const sys::LV2_URID_Map)
                .as_ref()
                .map(|internal| Self { internal })
        } else {
            panic!("The URID mapping feature isn't allowed in the audio threading class");
        }
    }
}

impl<'a> LV2Map<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Map) -> Self {
        Self { internal }
    }
}

impl<'a> Map for LV2Map<'a> {
    fn map(&self, uri: &Uri) -> Option<URID> {
        let uri = uri.as_ptr();
        let urid = unsafe { (self.internal.map.unwrap())(self.internal.handle, uri) };
        URID::new(urid)
    }
}

/// Host feature to revert the URI -> URID mapping.
#[repr(transparent)]
pub struct LV2Unmap<'a> {
    internal: &'a sys::LV2_URID_Unmap,
}

unsafe impl<'a> UriBound for LV2Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

unsafe impl<'a> Feature for LV2Unmap<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        if class != ThreadingClass::Audio {
            (feature as *const sys::LV2_URID_Unmap)
                .as_ref()
                .map(|internal| Self { internal })
        } else {
            panic!("The URID unmapping feature isn't allowed in the audio threading class");
        }
    }
}

impl<'a> LV2Unmap<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Unmap) -> Self {
        Self { internal }
    }
}

impl<'a> Unmap for LV2Unmap<'a> {
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let uri_ptr = unsafe { (self.internal.unmap.unwrap())(self.internal.handle, urid.get()) };
        if uri_ptr.is_null() {
            None
        } else {
            Some(unsafe { Uri::from_ptr(uri_ptr) })
        }
    }
}
