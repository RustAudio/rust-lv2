//! Ports version supporting inplace processing.
//!
//! These ports are safe to use when host provide same data location for input and output. Because
//! of that:
//!  - All ports of this module use [`RCell`] or [`RwCell`] to reflect potential aliasing of data
//!  location.
//!  - All output ports of this module are writted through the interior mutablility provided by
//!  [`RwCell`].
//!  - Care must be taken to read input datas before they are altered by writing to output.
//!
//!  TODO: Example and bad example

use crate::port::{PortHandle, RCell, RwCell};
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::*;

/// Audio input port. Gives a read only access to a buffer of audio samples.
///
/// Audio samples are float data normalized between -1.0 and 1.0, though there is no requirement
/// for samples to be strictly within this range.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#AudioPort) for more information.
pub struct AudioInput {
    ptr: *const [RCell<f32>],
}

impl PortHandle for AudioInput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts(pointer as *const RCell<f32>, sample_count as usize),
        })
    }
}

impl Deref for AudioInput {
    type Target = [RCell<f32>];
    fn deref(&self) -> &[RCell<f32>] {
        unsafe { &*self.ptr }
    }
}

/// Audio output port. Gives a read/write access to a buffer of audio samples.
///
/// Audio samples are float data normalized between -1.0 and 1.0, though there is no requirement
/// for samples to be strictly within this range.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#AudioPort) for more information.
pub struct AudioOutput {
    ptr: *mut [RwCell<f32>],
}

impl PortHandle for AudioOutput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts_mut(pointer as *mut RwCell<f32>, sample_count as usize),
        })
    }
}

impl Deref for AudioOutput {
    type Target = [RwCell<f32>];
    fn deref(&self) -> &[RwCell<f32>] {
        unsafe { &*self.ptr }
    }
}

/// Control input port. Gives a read only access to a single float.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
pub struct ControlInput {
    ptr: *const RCell<f32>,
}

impl PortHandle for ControlInput {
    unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: pointer as *const RCell<f32>,
        })
    }
}

impl Deref for ControlInput {
    type Target = RCell<f32>;
    fn deref(&self) -> &RCell<f32> {
        unsafe { &*self.ptr }
    }
}

/// Control output port. Gives a read/write access to a single float.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
pub struct ControlOutput {
    ptr: *mut RwCell<f32>,
}

impl PortHandle for ControlOutput {
    unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: pointer as *mut RwCell<f32>,
        })
    }
}

impl Deref for ControlOutput {
    type Target = RwCell<f32>;
    fn deref(&self) -> &RwCell<f32> {
        unsafe { &*self.ptr }
    }
}

/// CV input port. Gives a read only acces to a buffer of audio rate control values.
///
/// Ports of this type have the same buffer format as [`AudioInput`] ports, except the buffer
/// represents audio-rate control data rather than audio. It is generally safe to connect an audio
/// output to a CV input, but not vice-versa.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#CVPort) for more information.
pub struct CVInput {
    ptr: *const [RCell<f32>],
}

impl PortHandle for CVInput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts(pointer as *const RCell<f32>, sample_count as usize),
        })
    }
}

impl Deref for CVInput {
    type Target = [RCell<f32>];
    fn deref(&self) -> &[RCell<f32>] {
        unsafe { &*self.ptr }
    }
}

/// CV output port. Gives a read/write acces to a buffer of audio rate control values.
///
/// Ports of this type have the same buffer format as [`AudioInput`] ports, except the buffer
/// represents audio-rate control data rather than audio. It is generally safe to connect an audio
/// output to a CV input, but not vice-versa.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#CVPort) for more information.
pub struct CVOutput {
    ptr: *mut [RwCell<f32>],
}

impl PortHandle for CVOutput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts_mut(pointer as *mut RwCell<f32>, sample_count as usize),
        })
    }
}

impl Deref for CVOutput {
    type Target = [RwCell<f32>];
    fn deref(&self) -> &[RwCell<f32>] {
        unsafe { &*self.ptr }
    }
}
