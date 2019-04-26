use std::os::raw::c_char;

use std::ffi::c_void;
use crate::feature::RawFeatureDescriptor;

#[allow(non_snake_case)]
#[repr(C)]
pub struct PluginDescriptor<T> {
    pub URI: *const c_char,
    pub instantiate: Option<unsafe extern "C" fn(descriptor: *const PluginDescriptor<T>, sample_rate: f64, bundle_path: *const c_char, features: *const *const RawFeatureDescriptor) -> *mut T>,
    pub connect_port: Option<unsafe extern "C" fn(instance: *mut T, port: u32, data_location: *mut c_void)>,
    pub activate: Option<unsafe extern "C" fn(instance: *mut T)>,
    pub run: Option<unsafe extern "C" fn(instance: *mut T, sample_count: u32)>,
    pub deactivate: Option<unsafe extern "C" fn(instance: *mut T)>,
    pub cleanup: Option<unsafe extern "C" fn(instance: *mut T)>,
    pub extension_data: Option<unsafe extern "C" fn(uri: *const c_char) -> *const c_void>
}

impl<T> PluginDescriptor<T> {
    #[inline]
    pub fn get_raw(&self) -> *const RawFeatureDescriptor {
        self as *const Self as *const RawFeatureDescriptor
    }
}

unsafe impl<T> Send for PluginDescriptor<T> {}
unsafe impl<T> Sync for PluginDescriptor<T> {}

#[repr(transparent)]
pub struct PluginHandle {
    _private: c_void
}
