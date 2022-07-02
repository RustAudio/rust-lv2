//! Types to declare derivable port collections.
//!
//! Every plugin has a type of [`PortCollection`](trait.PortCollection.html) which is used to
//! handle input/output ports. In order to make the creation of these port collection types easier,
//! `PortCollection` can simply be derived. However, the macro that implements `PortCollection`
//! requires the fields of the struct to have specific types. These types are provided in this
//! module.
pub mod inplace;
pub mod not_inplace;

use std::cell::Cell;
use std::ffi::c_void;

/// A readonly cell. Used to give read only access for input port with inplace processing support.
///
/// This cell is used to give read only access to data when a writable alias may exist for the
/// underlying memory location. This is used by inplace input port because it's allow inplace
/// processing while preventing to write data through the current input port.
#[repr(transparent)]
pub struct RCell<T: ?Sized> {
    value: Cell<T>,
}

impl<T: Copy> RCell<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.value.get()
    }
}

impl<T> RCell<[T]> {
    /// Returns a `&[RCell<T>]` from a `&RCell<[T]>`
    pub fn as_slice_of_cells(&self) -> &[RCell<T>] {
        // SAFETY: `RCell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const RCell<[T]> as *const [RCell<T>]) }
    }
}

/// A read/write cell. Used to give read/write access for output port with inplace processing
/// support.
///
/// This cell is used to give read and write access to data when an alias may exist for the
/// underlying memory location. It works by giving interior mutability, like [`std::cell::Cell`].
/// This is used by inplace output because it's allow inplace processing.
// Note: technically, a std::Cell could be used, but custom cell is better to express the specific
// usage.
#[repr(transparent)]
pub struct RwCell<T: ?Sized> {
    value: Cell<T>,
}

impl<T: Copy> RwCell<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.value.get()
    }
}

impl<T> RwCell<T> {
    ///Sets the contained value.
    #[inline]
    pub fn set(&self, val: T) {
        self.value.set(val);
    }
}

impl<T> RwCell<[T]> {
    /// Returns a `&[RwCell<T>]` from a `&RwCell<[T]>`
    pub fn as_slice_of_cells(&self) -> &[RwCell<T>] {
        // SAFETY: `RwCell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const RwCell<[T]> as *const [RwCell<T>]) }
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
    /// Implementing this method requires a de-referentation of a raw pointer and therefore, it is
    /// unsafe.
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self>;
}

impl<T: PortHandle> PortHandle for Option<T> {
    unsafe fn from_raw(pointer: *mut c_void, sample_count: u32) -> Option<Self> {
        Some(T::from_raw(pointer, sample_count))
    }
}

/// Collection of IO ports.
///
/// Plugins do not handle port management on their own. Instead, they define a struct with all of
/// the required ports. Then, the plugin instance will collect the port pointers from the host and
/// create a `PortCollection` instance for every `run` call. Using this instance, plugins have
/// access to all of their required ports.
///
/// # Implementing
///
/// The most convenient way to create a port collections is to define a struct with port types from
/// the [`port`](index.html) module and then simply derive `PortCollection` for it. An example:
/// ```
///     # pub use lv2_core_derive::*;
///     use lv2_core::port::*;
///
///     #[derive(PortCollection)]
///     struct MyPortCollection {
///         audio_input: InputPort<Audio>,
///         audio_output: OutputPort<Audio>,
///         control_input: InputPort<Control>,
///         control_output: OutputPort<Control>,
///         optional_control_input: Option<InputPort<Control>>,
///     }
/// ```
///
/// Please note that port indices are mapped in the order of occurrence; In our example, the
/// implementation will treat `audio_input` as port `0`, `audio_output` as port `1` and so on.
/// Therefore, your plugin definition and your port collection have to match. Otherwise, undefined
/// behaviour will occur.
pub trait PortCollection: Sized {
    /// The type of the port pointer cache.
    ///
    /// The host passes port pointers to the plugin one by one and in an undefined order.
    /// Therefore, the plugin instance can not collect these pointers in the port collection
    /// directly. Instead, the pointers are stored in a cache which is then used to create the
    /// proper port collection.
    type Cache: PortPointerCache;

    /// Try to construct a port collection instance from a port pointer cache.
    ///
    /// If one of the port connection pointers is null, this method will return `None`, because a
    /// `PortCollection` can not be constructed.
    ///
    /// # Safety
    ///
    /// Since the pointer cache is only storing the pointers, implementing this method requires the
    /// de-referencation of raw pointers and therefore, this method is unsafe.
    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self>;
}

impl PortCollection for () {
    type Cache = ();

    unsafe fn from_connections(_cache: &(), _sample_count: u32) -> Option<Self> {
        Some(())
    }
}

/// Cache for port connection pointers.
///
/// The host will pass the port connection pointers one by one and in an undefined order.
/// Therefore, the `PortCollection` struct can not be created instantly. Instead, the pointers will
/// be stored in a cache, which is then used to create a proper port collection for the plugin.
pub trait PortPointerCache: Sized + Default {
    /// Store the connection pointer for the port with index `index`.
    ///
    /// The passed pointer may not be valid yet and therefore, implementors should only store the
    /// pointer, not dereference it.
    fn connect(&mut self, index: u32, pointer: *mut c_void);
}

impl PortPointerCache for () {
    fn connect(&mut self, _index: u32, _pointer: *mut c_void) {}
}
