//! Additional host functionalities.
use crate::UriBound;
use std::ffi::c_void;

/// Trait to generalize the feature detection system.
///
/// A host that only implements the core LV2 specification does not have much functionality. Therefore, host can provide extra functionalities, called "Features", a plugin can use to become more useful.
///
/// A native plugin written in C would discover a host's features by iterating through an array of URIs and pointers. When it finds the URI of the feature it is looking for, it casts the pointer to the type of the feature interface and uses the information from the interface.
///
/// In Rust, most of this behaviour is done internally and instead of simply casting a pointer, a safe feature descriptor, which implements this trait, is constructed using the [`from_raw_data`](#method.from_raw_data) method.
pub trait Feature: UriBound {
    /// The type that is used by the C interface to contain a feature's data.
    ///
    /// This should be the struct type defined by the specification, contained in your `sys` crate, if you have one.
    type RawDataType;

    /// Create a feature object from raw data.
    ///
    /// Since this will most likely involve dereferencing the data pointer, this method is marked as `unsafe´.
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

/// Marker feature to signal that the plugin can run in a hard real-time environment.
pub struct HardRTCapable;

unsafe impl UriBound for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

impl Feature for HardRTCapable {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}

/// Marker feature to signal the host to avoid in-place operation.
///
/// This feature has to be required by any plugin that may break if ANY input port is connected to the same memory location as ANY output port.
pub struct InPlaceBroken;

unsafe impl UriBound for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

impl Feature for InPlaceBroken {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}

/// Marker feature to signal the host to only run the plugin in a live environment.
pub struct IsLive;

unsafe impl UriBound for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}

impl Feature for IsLive {
    type RawDataType = c_void;

    unsafe fn from_raw_data(_data: *mut c_void) -> Self {
        Self {}
    }

    fn raw_data(&mut self) -> *mut c_void {
        std::ptr::null_mut()
    }
}
