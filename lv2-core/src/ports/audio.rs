use crate::port::PortType;
use crate::ports::base::{InputSampledData, OutputSampledData};
use std::ptr::NonNull;

pub struct Audio;

impl PortType for Audio {
    const NAME: &'static str = "Audio";
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__AudioPort;

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
