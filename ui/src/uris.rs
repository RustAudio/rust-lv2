use lv2_sys as sys;
use urid::*;

pub struct ScaleFactor;

unsafe impl UriBound for ScaleFactor {
    const URI: &'static [u8] = sys::LV2_UI__scaleFactor;
}

pub struct UpdateRate;

unsafe impl UriBound for UpdateRate {
    const URI: &'static [u8] = sys::LV2_UI__updateRate;
}
