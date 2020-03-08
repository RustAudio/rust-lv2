use crate::feature::*;
use lv2_urid::prelude::*;
use std::ffi::c_void;
use std::os::raw::c_char;

/// Host feature to map URIs to integers
#[repr(transparent)]
pub struct Map<'a> {
    internal: &'a sys::LV2_URID_Map,
}

unsafe impl<'a> UriBound for Map<'a> {
    const URI: &'static [u8] = sys::LV2_URID_MAP_URI;
}

unsafe impl<'a> Feature for Map<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        match class {
            ThreadingClass::Audio => {
                panic!("The URID mapping feature isn't allowed in the audio threading class")
            }
            _ => (feature as *const sys::LV2_URID_Map)
                .as_ref()
                .map(|internal| Self { internal }),
        }
    }
}

impl<'a> Map<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Map) -> Self {
        Self { internal }
    }
}

impl<'a> URIDMap for Map<'a> {
    fn map_uri(&self, uri: &Uri) -> Option<URID> {
        let uri = uri.as_ptr();
        let urid = unsafe { (self.internal.map.unwrap())(self.internal.handle, uri) };
        URID::new(urid)
    }

    fn map_type<T: UriBound + ?Sized>(&self) -> Option<URID<T>> {
        let handle = self.internal.handle;
        let uri = T::URI.as_ptr() as *const c_char;
        let urid = unsafe { (self.internal.map?)(handle, uri) };
        if urid == 0 {
            None
        } else {
            Some(unsafe { URID::new_unchecked(urid) })
        }
    }
}

/// Host feature to revert the URI -> URID mapping.
#[repr(transparent)]
pub struct Unmap<'a> {
    internal: &'a sys::LV2_URID_Unmap,
}

unsafe impl<'a> UriBound for Unmap<'a> {
    const URI: &'static [u8] = sys::LV2_URID_UNMAP_URI;
}

unsafe impl<'a> Feature for Unmap<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, class: ThreadingClass) -> Option<Self> {
        match class {
            ThreadingClass::Audio => {
                panic!("The URID unmapping feature isn't allowed in the audio threading class")
            }
            _ => (feature as *const sys::LV2_URID_Unmap)
                .as_ref()
                .map(|internal| Self { internal }),
        }
    }
}

impl<'a> Unmap<'a> {
    pub fn new(internal: &'a sys::LV2_URID_Unmap) -> Self {
        Self { internal }
    }
}

impl<'a> URIDUnmap for Unmap<'a> {
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let uri_ptr = unsafe { (self.internal.unmap.unwrap())(self.internal.handle, urid.get()) };
        if uri_ptr.is_null() {
            None
        } else {
            Some(unsafe { Uri::from_ptr(uri_ptr) })
        }
    }
}
