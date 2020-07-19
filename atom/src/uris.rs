//! Commonly used URIs from the https://lv2plug.in/ns/ext/atom domain

use urid::*;

pub struct AtomTransfer;

unsafe impl UriBound for AtomTransfer {
    const URI: &'static [u8] = sys::LV2_ATOM__atomTransfer;
}

pub struct BeatTime;

unsafe impl UriBound for BeatTime {
    const URI: &'static [u8] = sys::LV2_ATOM__beatTime;
}

pub struct BufferType;

unsafe impl UriBound for BufferType {
    const URI: &'static [u8] = sys::LV2_ATOM__bufferType;
}

pub struct ChildType;

unsafe impl UriBound for ChildType {
    const URI: &'static [u8] = sys::LV2_ATOM__childType;
}

pub struct EventTransfer;

unsafe impl UriBound for EventTransfer {
    const URI: &'static [u8] = sys::LV2_ATOM__eventTransfer;
}

pub struct FrameTime;

unsafe impl UriBound for FrameTime {
    const URI: &'static [u8] = sys::LV2_ATOM__frameTime;
}

pub struct Supports;

unsafe impl UriBound for Supports {
    const URI: &'static [u8] = sys::LV2_ATOM__supports;
}

pub struct TimeUnit;

unsafe impl UriBound for TimeUnit {
    const URI: &'static [u8] = sys::LV2_ATOM__timeUnit;
}
