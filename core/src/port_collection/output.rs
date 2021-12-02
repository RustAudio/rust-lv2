use crate::port::{PortType, PortTypeHandle};
use crate::port_collection::PortCollection;
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// Handle for output ports.
///
/// Fields of this type can be dereferenced to the output type of the port type.
pub struct OutputPort<'a, T: PortType> {
    port: <T::OutputPortType as PortTypeHandle<'a>>::Handle,
}

impl<'a, T: PortType> Deref for OutputPort<'a, T> {
    type Target = <T::OutputPortType as PortTypeHandle<'a>>::Handle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a, T: PortType> DerefMut for OutputPort<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

impl<'a, T: PortType> PortCollection<'a> for OutputPort<'a, T> {
    type Cache = *mut c_void;

    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self> {
        Some(Self {
            port: T::output_from_raw(NonNull::new(*cache)?, sample_count),
        })
    }
}
