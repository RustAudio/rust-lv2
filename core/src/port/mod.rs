mod audio;
mod control;
mod cv;

use crate::uri::{Uri, UriBound};
use std::ptr::NonNull;

pub mod base;

pub use self::audio::*;
pub use self::control::*;
pub use self::cv::*;

pub trait PortType: 'static + Sized + UriBound {
    const NAME: &'static str;

    type InputPortType: Sized;
    type OutputPortType: Sized;

    unsafe fn input_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::InputPortType;
    unsafe fn output_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::OutputPortType;

    #[inline]
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_with_nul_unchecked(Self::URI) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}
