//! Ports version not supporting inplace processing.
//!
//! These ports are easier to use thant their inplace counterpart, but they **must not** be used
//! when a host can provide same data location for input and output. Most of the time, this require
//! `inPlaceBroken` LV2 feature for your plugin.  Using `inPlaceBroken` feature is discouraged
//! because many host doesn't support it.
//!
//! TODO: example
use crate::port::PortHandle;
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr::*;

/// Audio input port. Gives a read only access to a buffer of audio samples.
///
/// Audio samples are float data normalized between -1.0 and 1.0, though there is no requirement
/// for samples to be strictly within this range.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#AudioPort) for more information.
pub struct AudioInput {
    ptr: *const [f32],
}

impl PortHandle for AudioInput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts(pointer as *const f32, sample_count as usize),
        })
    }
}

impl Deref for AudioInput {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
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
    ptr: *mut [f32],
}

impl PortHandle for AudioOutput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts_mut(pointer as *mut f32, sample_count as usize),
        })
    }
}

impl Deref for AudioOutput {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for AudioOutput {
    fn deref_mut(&mut self) -> &mut [f32] {
        unsafe { &mut *self.ptr }
    }
}

/// Control input port. Gives a read only access to a single float.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
pub struct ControlInput {
    ptr: *const f32,
}

impl PortHandle for ControlInput {
    unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: pointer as *const f32,
        })
    }
}

impl Deref for ControlInput {
    type Target = f32;
    fn deref(&self) -> &f32 {
        unsafe { &*self.ptr }
    }
}

/// Control output port. Gives a read/write access to a single float.
///
/// See the [LV2 reference](https://lv2plug.in/ns/lv2core#ControlPort) for more information.
pub struct ControlOutput {
    ptr: *mut f32,
}

impl PortHandle for ControlOutput {
    unsafe fn from_raw(pointer: *mut c_void, _sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: pointer as *mut f32,
        })
    }
}

impl Deref for ControlOutput {
    type Target = f32;
    fn deref(&self) -> &f32 {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for ControlOutput {
    fn deref_mut(&mut self) -> &mut f32 {
        unsafe { &mut *self.ptr }
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
    ptr: *const [f32],
}

impl PortHandle for CVInput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts(pointer as *const f32, sample_count as usize),
        })
    }
}

impl Deref for CVInput {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
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
    ptr: *mut [f32],
}

impl PortHandle for CVOutput {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(Self {
            ptr: slice_from_raw_parts_mut(pointer as *mut f32, sample_count as usize),
        })
    }
}

impl Deref for CVOutput {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for CVOutput {
    fn deref_mut(&mut self) -> &mut [f32] {
        unsafe { &mut *self.ptr }
    }
}
