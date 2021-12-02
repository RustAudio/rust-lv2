use crate::port::{AtomicPortType, PortType, PortTypeHandle};
use crate::port_collection::PortCollection;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

// No generics here, for simplicity of this example, assume it is always just basic audio (`f32`).
pub struct InputOutputPort<'a, T: PortType> {
    input: NonNull<c_void>,
    output: NonNull<c_void>,
    sample_count: u32,
    _port: PhantomData<&'a mut T>,
}

impl<'a, T: PortType> InputOutputPort<'a, T> {
    pub fn input(&self) -> <T::InputPortType as PortTypeHandle>::Handle {
        // SAFETY: Pointer validity is upheld by from_connections, and is the only way to construct this type.
        unsafe { T::input_from_raw(self.input, self.sample_count) }
    }

    pub fn output(&mut self) -> <T::OutputPortType as PortTypeHandle>::Handle {
        // SAFETY: Pointer validity is upheld by from_connections, and is the only way to construct this type.
        unsafe { T::output_from_raw(self.input, self.sample_count) }
    }
}

impl<'a, T: AtomicPortType> InputOutputPort<'a, T> {
    #[inline]
    pub fn input_output(&mut self) -> <T::InputOutputPortType as PortTypeHandle>::Handle {
        // SAFETY: Pointer validity is upheld by from_connections, and is the only way to construct this type.
        unsafe { T::input_output_from_raw(self.input, self.output, self.sample_count) }
    }
}

impl<
        'a,
        I: Sized + 'a,
        IO: PortTypeHandle<'a, Handle = (&'a [I], &'a [I])>,
        T: AtomicPortType<InputOutputPortType = IO>,
    > InputOutputPort<'a, T>
{
    #[inline]
    pub fn zip<'b: 'a>(&'b mut self) -> impl Iterator<Item = (&'b I, &'b I)> + 'b
    where
        'a: 'b,
    {
        let (input, output): (&'b [I], &'b [I]) = self.input_output();
        input.iter().zip(output)
    }
}

impl<'a, T: PortType> PortCollection<'a> for InputOutputPort<'a, T> {
    type Cache = [*mut c_void; 2];

    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self> {
        Some(Self {
            input: NonNull::new(cache[0])?,
            output: NonNull::new(cache[1])?,
            sample_count,
            _port: PhantomData,
        })
    }
}
