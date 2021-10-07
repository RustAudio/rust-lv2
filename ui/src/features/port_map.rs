use lv2_core::feature::{Feature, ThreadingClass};
use lv2_core::port::index::PortIndex;
use std::ffi::c_void;
use urid::{Uri, UriBound};

pub struct PortMap<'a> {
    inner: &'a lv2_sys::LV2UI_Port_Map,
}

unsafe impl<'a> UriBound for PortMap<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_UI__portMap;
}

unsafe impl<'a> Feature for PortMap<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        feature
            .cast::<lv2_sys::LV2UI_Port_Map>()
            .as_ref()
            .map(|inner| Self { inner })
    }
}

impl<'a> PortMap<'a> {
    #[inline]
    pub fn get_port_index(&self, port_uri: &Uri) -> Option<PortIndex<()>> {
        let port = unsafe { self.inner.port_index.unwrap()(self.inner.handle, port_uri.as_ptr()) };

        if port == u32::MAX {
            None
        } else {
            Some(PortIndex::new(port))
        }
    }
}
