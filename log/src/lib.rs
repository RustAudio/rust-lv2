use lv2_core::feature::*;
use std::os::raw::*; //get all common c_type
use urid::*;

/// The Log feature
#[repr(transparent)]
pub struct Log<'a> {
    internal: &'a lv2_sys::LV2_Log_Log,
}

unsafe impl<'a> UriBound for Log<'a> {
    const URI: &'static [u8] = lv2_sys::LV2_WORKER__schedule;
}

unsafe impl<'a> Feature for Log<'a> {
    // Note: this feature can be used in any threading class and doesn't seems to have thready
    // unsafty
    unsafe fn from_feature_ptr(feature: *const c_void, _class: ThreadingClass) -> Option<Self> {
        (feature as *const lv2_sys::LV2_Log_Log)
            .as_ref()
            .map(|internal| Self { internal })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
