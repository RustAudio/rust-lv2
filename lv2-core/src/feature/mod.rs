mod descriptor;
mod list;

pub use self::descriptor::*;
pub use self::list::*;

use crate::feature::descriptor::FeatureDescriptor;
use crate::uri::UriBound;

/// Represents extension data for a given feature.
/// # Unsafety
/// Since extension data is passed to plugin as a raw pointer,
/// structs implementing this trait must be `#[repr(C)]`.
pub unsafe trait Feature: Sized + Copy {
    const URI: &'static [u8];

    #[inline]
    fn descriptor(&self) -> FeatureDescriptor {
        FeatureDescriptor::from_feature(self)
    }
}

unsafe impl<F: Feature> UriBound for F {
    const URI: &'static [u8] = <F as Feature>::URI;
}

#[repr(transparent)]
pub struct RawFeatureDescriptor {
    pub(crate) inner: ::lv2_core_sys::LV2_Feature,
}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HardRTCapable;

unsafe impl Feature for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InPlaceBroken;

unsafe impl Feature for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct IsLive;

unsafe impl Feature for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}
