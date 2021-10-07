use lv2_core::feature::{Feature, ThreadingClass};
use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use urid::UriBound;

pub struct Resize<'a> {
    inner: &'a lv2_sys::LV2UI_Resize,
}

impl<'a> Resize<'a> {
    pub fn resize(&self, width: i32, height: i32) -> Result<(), ResizeError> {
        let res = unsafe { self.inner.ui_resize.unwrap()(self.inner.handle, width, height) };

        if res == 0 {
            Ok(())
        } else {
            Err(ResizeError)
        }
    }
}

unsafe impl<'a> UriBound for Resize<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_UI__resize;
}

unsafe impl<'a> Feature for Resize<'a> {
    #[inline]
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        feature
            .cast::<lv2_sys::LV2UI_Resize>()
            .as_ref()
            .map(|inner| Self { inner })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ResizeError;

impl Display for ResizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LV2 UI Resize failed")
    }
}

impl Error for ResizeError {}
