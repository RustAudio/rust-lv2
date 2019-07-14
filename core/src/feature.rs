use crate::uri::UriBound;
use std::ffi::c_void;

/// Represents extension data for a given feature.
///
/// Features have to be `#[repr(C)]`, since they have to have a valid representation in C. Since
/// this requirement can not be checked with super-traits, this trait is `unsafe` to implement.
pub unsafe trait Feature: UriBound {
    /// The type that is used by the C interface to contain a feature's data.
    type RawDataType;

    /// Create a feature object from raw data.
    unsafe fn from_raw_data(data: *mut Self::RawDataType) -> Self;

    /// Return a pointer to the raw data.
    fn raw_data(&mut self) -> *mut Self::RawDataType;

    /// Create a raw feature descriptor for the plugin.
    fn create_raw_feature(&mut self) -> ::sys::LV2_Feature {
        ::sys::LV2_Feature {
            URI: <Self as UriBound>::URI.as_ptr() as *const i8,
            data: self.raw_data() as *mut c_void,
        }
    }
}

pub struct HardRTCapable;

unsafe impl UriBound for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

unsafe impl Feature for HardRTCapable {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}

pub struct InPlaceBroken;

unsafe impl UriBound for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

unsafe impl Feature for InPlaceBroken {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}

pub struct IsLive;

unsafe impl UriBound for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}

unsafe impl Feature for IsLive {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}
