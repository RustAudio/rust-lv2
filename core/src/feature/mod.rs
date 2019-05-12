mod descriptor;
mod list;

pub use self::descriptor::*;
pub use self::list::*;

use crate::feature::descriptor::FeatureDescriptor;
use crate::uri::UriBound;

/// Represents extension data for a given feature.
///
/// Features have to be `#[repr(C)]`, since they have to have a valid representation in C. Since
/// this requirement can not be checked with super-traits, this trait is `unsafe` to implement.
pub unsafe trait Feature: Sized + Copy + UriBound {
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
