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
    type DATA: 'static;

    const URI: &'static [u8];

    fn get_data(&'static self) -> Option<&'static Self::DATA>;

    #[inline]
    fn descriptor(&self) -> FeatureDescriptor {
        FeatureDescriptor::from_feature(self)
    }
}

unsafe impl<F: Feature> UriBound for F {
    const URI: &'static [u8] = <F as Feature>::URI;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HardRTCapable;

unsafe impl Feature for HardRTCapable {
    type DATA = ();

    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;

    fn get_data(&'static self) -> Option<&'static ()> {
        None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InPlaceBroken;

unsafe impl Feature for InPlaceBroken {
    type DATA = ();

    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;

    fn get_data(&'static self) -> Option<&'static ()> {
        None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct IsLive;

unsafe impl Feature for IsLive {
    type DATA = ();

    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;

    fn get_data(&'static self) -> Option<&'static ()> {
        None
    }
}
