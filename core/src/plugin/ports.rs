use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::ffi::c_void;

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
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Self;
}

impl<T: PortType> PortHandle for InputPort<T> {
    #[inline]
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Self {
        Self {
            port: T::input_from_raw(NonNull::new_unchecked(pointer), sample_count),
        }
    }
}

impl<T: PortType> PortHandle for OutputPort<T> {
    #[inline]
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Self {
        Self {
            port: T::output_from_raw(NonNull::new_unchecked(pointer), sample_count),
        }
    }
}

impl<T: PortHandle> PortHandle for Option<T> {
    #[inline]
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Self {
        NonNull::new(pointer).map(|ptr| T::from_raw(ptr.as_ptr(), sample_count))
    }
}
