use crate::port::PortType;
use crate::port_collection::PortCollection;
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::NonNull;

/// Handle for input ports.
///
/// Fields of this type can be dereferenced to the input type of the port type.
pub struct InputPort<T: PortType + ?Sized> {
    port: T::Input,
    // TODO: remove these sometime
    pub(crate) ptr: NonNull<c_void>,
    pub(crate) sample_count: u32,
}

impl<T: PortType> Deref for InputPort<T> {
    type Target = T::Input;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<T: PortType> PortCollection for InputPort<T> {
    type Connections = *mut c_void;

    unsafe fn from_connections(cache: &Self::Connections, sample_count: u32) -> Option<Self> {
        let ptr = NonNull::new(*cache)?;
        Some(Self {
            port: T::input_from_raw(ptr, sample_count),
            ptr,
            sample_count,
        })
    }
}
