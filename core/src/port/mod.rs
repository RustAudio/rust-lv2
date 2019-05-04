mod audio;
mod control;
mod cv;

use crate::uri::Uri;
use std::ptr::NonNull;

pub mod base;

pub use self::audio::*;
pub use self::control::*;
pub use self::cv::*;

pub trait PortType: 'static + Sized + Send {
    const NAME: &'static str;
    const URI: &'static [u8];

    type InputPortType: Sized + Send;
    type OutputPortType: Sized + Send;

    unsafe fn input_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::InputPortType;
    unsafe fn output_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::OutputPortType;

    #[inline]
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_unchecked(Self::URI) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}
