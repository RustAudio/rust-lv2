//! Commonly used URIs from the lv2plug.in domain associated with lv2-core
//!
use urid::*;

pub struct SampleRate;

unsafe impl UriBound for SampleRate {
    const URI: &'static [u8] = sys::LV2_CORE__sampleRate;
}

pub struct BoundedBlockLength;

unsafe impl UriBound for BoundedBlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__boundedBlockLength;
}

pub struct FixedBlockLength;

unsafe impl UriBound for FixedBlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__fixedBlockLength;
}

pub struct MaxBlockLength;

unsafe impl UriBound for MaxBlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__maxBlockLength;
}

pub struct MinBlockLength;

unsafe impl UriBound for MinBlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__minBlockLength;
}

pub struct NominalBlockLength;

unsafe impl UriBound for NominalBlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__nominalBlockLength;
}

pub struct PowerOf2BlockLength;

unsafe impl UriBound for PowerOf2BlockLength {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__powerOf2BlockLength;
}

pub struct SequenceSize;

unsafe impl UriBound for SequenceSize {
    const URI: &'static [u8] = sys::LV2_BUF_SIZE__maxBlockLength;
}
