use lv2_core::feature::{Feature, ThreadingClass};
use lv2_core::port::index::PortIndex;
use lv2_core::port::{PortCollection, PortHandle};
use std::ffi::c_void;
use urid::UriBound;

pub struct Touch<'a> {
    inner: &'a lv2_sys::LV2UI_Touch,
}

impl<'a> Touch<'a> {
    #[inline]
    pub fn set_grabbed<P: PortHandle, C: PortCollection>(
        &mut self,
        port_index: PortIndex<P, C>,
        grabbed: bool,
    ) {
        unsafe { self.inner.touch.unwrap()(self.inner.handle, port_index.get(), grabbed) }
    }
}

unsafe impl<'a> UriBound for Touch<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_UI__touch;
}

unsafe impl<'a> Feature for Touch<'a> {
    #[inline]
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        feature
            .cast::<lv2_sys::LV2UI_Touch>()
            .as_ref()
            .map(|inner| Self { inner })
    }
}
