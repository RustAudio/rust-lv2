use crate::port::PortType;
use crate::port_collection::PortCollection;
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// Handle for output ports.
///
/// Fields of this type can be dereferenced to the output type of the port type.
pub struct OutputPort<T: PortType + ?Sized> {
    port: T::Output,
    // TODO: remove these sometime
    pub(crate) ptr: NonNull<c_void>,
    pub(crate) sample_count: u32,
}

impl<T: PortType> Deref for OutputPort<T> {
    type Target = T::Output;

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

impl<T: PortType> PortCollection for OutputPort<T> {
    type Connections = *mut c_void;

    unsafe fn from_connections(cache: &Self::Connections, sample_count: u32) -> Option<Self> {
        let ptr = NonNull::new(*cache)?;
        Some(Self {
            port: T::output_from_raw(ptr, sample_count),
            ptr,
            sample_count,
        })
    }
}
