//! LV2 specification for measuring unit definitions.
//!
//! The original [specification](http://lv2plug.in/ns/extensions/units/units.html) contains means to describe units for LV2 values in RDF files. This implementation is focused on the stock units defined by the specification by binding them to marker types.
mod plugin;

extern crate lv2_sys as sys;

use lv2_core::feature::Feature;
use lv2_core::prelude::*;
use std::ffi::c_void;

pub struct InstanceAccess {
    plugin_handle: *const c_void,
}
/*
unsafe impl UriBound for InstanceAccess {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__hardRTCapable;
}*/
/*
unsafe impl Feature for InstanceAccess {
    unsafe fn from_feature_ptr(plugin_handle: *const c_void, _: ThreadingClass) -> Option<Self> {
        Some(Self { plugin_handle })
    }
}
*/
