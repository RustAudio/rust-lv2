use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::port_type::PortType;

pub struct InputPort<T: PortType> {
    port: T::InputPortType,
}

impl<T: PortType> Deref for InputPort<T> {
    type Target = T::InputPortType;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

pub struct OutputPort<T: PortType> {
    port: T::OutputPortType,
}

impl<T: PortType> Deref for OutputPort<T> {
    type Target = T::OutputPortType;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<T: PortType> DerefMut for OutputPort<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

pub trait PortHandle: Sized {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self>;
}

impl<T: PortType> PortHandle for InputPort<T> {
    #[inline]
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        if let Some(pointer) = NonNull::new(pointer) {
            Some(Self {
                port: T::input_from_raw(pointer, sample_count),
            })
        } else {
            None
        }
    }
}

impl<T: PortType> PortHandle for OutputPort<T> {
    #[inline]
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        if let Some(pointer) = NonNull::new(pointer) {
            Some(Self {
                port: T::output_from_raw(pointer, sample_count),
            })
        } else {
            None
        }
    }
}

impl<T: PortHandle> PortHandle for Option<T> {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(T::from_raw(pointer, sample_count))
    }
}
