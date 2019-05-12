mod descriptor;
mod list;

pub use self::descriptor::*;
pub use self::list::*;

use crate::feature::descriptor::FeatureDescriptor;
use crate::uri::UriBound;

/// Represents extension data for a given feature.
pub trait Feature: Sized + Copy + UriBound {
    #[inline]
    fn descriptor(&self) -> FeatureDescriptor {
        FeatureDescriptor::from_feature(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HardRTCapable;

unsafe impl UriBound for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

impl Feature for HardRTCapable {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InPlaceBroken;

unsafe impl UriBound for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

impl Feature for InPlaceBroken {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct IsLive;

unsafe impl UriBound for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}

impl Feature for IsLive {}
