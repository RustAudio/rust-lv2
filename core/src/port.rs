//! Types to declare derivable port collections.
//!
//! Every plugin has a type of [`PortCollection`](trait.PortCollection.html) which is used to handle input/output ports. In order to make the creation of these port collection types easier, `PortCollection` can simply be derived. However, the macro that implements `PortCollection` requires the fields of the struct to have specific types. These types are provided in this module.
use core::ffi::c_void;
use core::ptr::NonNull;

pub use audio::*;
pub use control::*;
pub use cv::*;
#[cfg(feature = "lv2-core-derive")]
pub use lv2_core_derive::*;

pub use crate::port_collection::*;

mod audio;
mod control;
mod cv;

pub trait PortTypeHandle<'a> {
    type Handle: 'a + Sized;
}

/// Generalization of port types.
///
/// A port can read input or create a pointer to the output, but the exact type of input/output (pointer) depends on the type of port. This trait generalizes these types and behaviour.
pub trait PortType {
    /// The type of input read by the port.
    type InputPortType: for<'a> PortTypeHandle<'a>;
    /// The type of output reference created by the port.
    type OutputPortType: for<'a> PortTypeHandle<'a>;

    /// Read data from the pointer or create a reference to the input.
    ///
    /// If the resulting data is a slice, `sample_count` is the length of the slice.
    ///
    /// # Safety
    ///
    /// This method is unsafe because one needs to de-reference a raw pointer to implement this method.
    unsafe fn input_from_raw<'a>(
        pointer: NonNull<c_void>,
        sample_count: u32,
    ) -> <Self::InputPortType as PortTypeHandle<'a>>::Handle;

    /// Create a reference to the data where output should be written to.
    ///
    /// If the data is a slice, `sample_count` is the length of the slice.
    ///
    /// # Safety
    ///
    /// This method is unsafe because one needs to de-reference a raw pointer to implement this method.
    unsafe fn output_from_raw<'a>(
        pointer: NonNull<c_void>,
        sample_count: u32,
    ) -> <Self::OutputPortType as PortTypeHandle<'a>>::Handle;
}

pub trait AtomicPortType: PortType {
    type InputOutputPortType: for<'a> PortTypeHandle<'a>;

    unsafe fn input_output_from_raw<'a>(
        input: NonNull<c_void>,
        output: NonNull<c_void>,
        sample_count: u32,
    ) -> <Self::InputOutputPortType as PortTypeHandle<'a>>::Handle;
}
