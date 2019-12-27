//! Types to declare derivable port containers.
//!
//! Every plugin has a type of [`PortContainer`](trait.PortContainer.html) which is used to handle input/output ports. In order to make the creation of these port container types easier, `PortContainer` can simply be derived. However, the macro that implements `PortContainer` requires the fields of the struct to have specific types. These types are provided in this module.
use crate::UriBound;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub use lv2_core_derive::*;

/// Generalization of port types.
///
/// A port can read input or create a pointer to the output, but the exact type of input/output (pointer) depends on the type of port. This trait generalizes these types and behaviour.
pub trait PortType {
    /// The type of input read by the port.
    type InputPortType: Sized;
    /// The type of output reference created by the port.
    type OutputPortType: Sized;

    /// Read data from the pointer or create a reference to the input.
    ///
    /// If the resulting data is a slice, `sample_count` is the length of the slice.
    ///
    /// # Safety
    ///
    /// This method is unsafe because one needs to de-reference a raw pointer to implement this method.
    unsafe fn input_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::InputPortType;

    /// Create a reference to the data where output should be written to.
    ///
    /// If the data is a slice, `sample_count` is the length of the slice.
    ///
    /// # Safety
    ///
    /// This method is unsafe because one needs to de-reference a raw pointer to implement this method.
    unsafe fn output_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::OutputPortType;
}

/// Audio port type.
///
/// Audio ports are the most common type of input/output ports: Their input is a slice of audio samples, as well as their output.
pub struct Audio;

unsafe impl UriBound for Audio {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__AudioPort;
}

impl PortType for Audio {
    type InputPortType = &'static [f32];
    type OutputPortType = &'static mut [f32];

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::InputPortType {
        std::slice::from_raw_parts(pointer.as_ptr() as *const f32, sample_count as usize)
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::OutputPortType {
        std::slice::from_raw_parts_mut(pointer.as_ptr() as *mut f32, sample_count as usize)
    }
}

/// Control value port type.
///
/// Control ports in general are used to control the behaviour of the plugin. These control value ports only have one value per `run` call and therefore don't have a fixed sampling rate.
///
/// Therefore, their input is a floating-point number and their output is a mutable reference to a floating-point number.
pub struct Control;

unsafe impl UriBound for Control {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__ControlPort;
}

impl PortType for Control {
    type InputPortType = f32;
    type OutputPortType = &'static mut f32;

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> f32 {
        *(pointer.cast().as_ref())
    }

    unsafe fn output_from_raw(pointer: NonNull<c_void>, _sample_count: u32) -> &'static mut f32 {
        (pointer.as_ptr() as *mut f32).as_mut().unwrap()
    }
}

/// CV port type.
///
/// Control ports in general are used to control the behaviour of the plugin. CV ports are sampled just like [audio data](struct.Audio.html). This means that audio data is often valid CV data, but CV data generally is not audio data, because it may not be within the audio bounds of -1.0 to 1.0.
pub struct CV;

unsafe impl UriBound for CV {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__CVPort;
}

impl PortType for CV {
    type InputPortType = &'static [f32];
    type OutputPortType = &'static mut [f32];

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::InputPortType {
        std::slice::from_raw_parts(pointer.as_ptr() as *const f32, sample_count as usize)
    }

    #[inline]
    unsafe fn output_from_raw(pointer: NonNull<c_void>, sample_count: u32) -> Self::OutputPortType {
        std::slice::from_raw_parts_mut(pointer.as_ptr() as *mut f32, sample_count as usize)
    }
}

/// Abstraction of safe port handles.
pub trait PortHandle: Sized {
    /// Try to create a port handle from a port connection pointer and the sample count.
    ///
    /// If the pointer is null, this method will return `None`.
    ///
    /// # Safety
    ///
    /// Implementing this method requires a de-referentation of a raw pointer and therefore, it is unsafe.
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self>;
}

/// Handle for input ports.
///
/// Fields of this type can be dereferenced to the input type of the port type.
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

/// Handle for output ports.
///
/// Fields of this type can be dereferenced to the output type of the port type.
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

/// Container for port handling.
///
/// Plugins do not handle port management on their own. Instead, they define a struct with all of the required ports. Then, the plugin instance will collect the port pointers from the host and create a `PortContainer` instance for every `run` call. Using this instance, plugins have access to all of their required ports.
///
/// # Implementing
///
/// The most convenient way to create port containers is to define a struct with port types from the [`port`](index.html) module and then simply derive `PortContainer` for it. An example:
///
///     use lv2_core::port::*;
///
///     #[derive(PortContainer)]
///     struct MyPortContainer {
///         audio_input: InputPort<Audio>,
///         audio_output: OutputPort<Audio>,
///         control_input: InputPort<Control>,
///         control_output: OutputPort<Control>,
///         optional_control_input: Option<InputPort<Control>>,
///     }
///
/// Please note that port indices are mapped in the order of occurence; In our example, the implementation will treat `audio_input` as port `0`, `audio_output` as port `1` and so on. Therefore, your plugin definition and your port container have to match. Otherwise, undefined behaviour will occur.
pub trait PortContainer: Sized {
    /// The type of the port pointer cache.
    ///
    /// The host passes port pointers to the plugin one by one and in an undefined order. Therefore, the plugin instance can not collect these pointers in the port container directly. Instead, the pointers are stored in a cache which is then used to create the proper port container.
    type Cache: PortPointerCache;

    /// Try to construct a port container instance from a port pointer cache.
    ///
    /// If one of the port connection pointers is null, this method will return `None`, because a `PortContainer` can not be constructed.
    ///
    /// # Safety
    ///
    /// Since the pointer cache is only storing the pointers, implementing this method requires the de-referencation of raw pointers and therefore, this method is unsafe.
    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self>;
}

impl PortContainer for () {
    type Cache = ();

    unsafe fn from_connections(_cache: &(), _sample_count: u32) -> Option<Self> {
        Some(())
    }
}

/// Cache for port connection pointers.
///
/// The host will pass the port connection pointers one by one and in an undefined order. Therefore, the `PortContainer` struct can not be created instantly. Instead, the pointers will be stored in a cache, which is then used to create a proper port container for the plugin.
pub trait PortPointerCache: Sized + Default {
    /// Store the connection pointer for the port with index `index`.
    ///
    /// The passed pointer may not be valid yet and therefore, implementors should only store the pointer, not dereference it.
    fn connect(&mut self, index: u32, pointer: *mut c_void);
}

impl PortPointerCache for () {
    fn connect(&mut self, _index: u32, _pointer: *mut c_void) {}
}
