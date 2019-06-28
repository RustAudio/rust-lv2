use crate::port::{
    base::{InputSampledData, OutputSampledData},
    PortType,
};
use crate::uri::UriBound;
use std::ptr::NonNull;

pub struct Audio;

unsafe impl UriBound for Audio {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__AudioPort;
}

impl PortType for Audio {
    const NAME: &'static str = "Audio";

    type InputPortType = InputSampledData<f32>;
    type OutputPortType = OutputSampledData<f32>;

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::InputPortType {
        InputSampledData::new(pointer, sample_count)
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::OutputPortType {
        OutputSampledData::new(pointer, sample_count)
    }
}
