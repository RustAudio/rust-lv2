use crate::port::PortType;
use std::ptr::NonNull;
use crate::ports::base::{InputSampledData, OutputSampledData};

pub struct CV;

impl PortType for CV {
    const NAME: &'static str = "CV";
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__CVPort;

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