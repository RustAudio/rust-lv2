use crate::uri::UriBound;
use std::ffi::c_void;

/// Represents extension data for a given feature.
///
/// Features have to be `#[repr(C)]`, since they have to have a valid representation in C. Since
/// this requirement can not be checked with super-traits, this trait is `unsafe` to implement.
pub unsafe trait Feature: Sized + Copy + UriBound {
    
    /// Create a raw feature descriptor for the plugin.
    fn create_raw_feature(&mut self) -> ::sys::LV2_Feature {
        ::sys::LV2_Feature {
            URI: <Self as UriBound>::URI.as_ptr() as *const i8,
            data: self as *mut _ as *mut c_void,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HardRTCapable;

unsafe impl UriBound for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

unsafe impl Feature for HardRTCapable {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InPlaceBroken;

unsafe impl UriBound for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

unsafe impl Feature for InPlaceBroken {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct IsLive;

unsafe impl UriBound for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}

unsafe impl Feature for IsLive {}
