mod input;
mod io;
mod output;
mod pointer_cache;

pub use input::InputPort;
pub use io::InputOutputPort;
pub use output::OutputPort;
pub use pointer_cache::PortConnections;

/// Collection of IO ports.
///
/// Plugins do not handle port management on their own. Instead, they define a struct with all of the required ports. Then, the plugin instance will collect the port pointers from the host and create a `PortCollection` instance for every `run` call. Using this instance, plugins have access to all of their required ports.
///
/// # Implementing
///
/// The most convenient way to create a port collections is to define a struct with port types from the [`port`](index.html) module and then simply derive `PortCollection` for it. An example:
///
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
///
/// Please note that port indices are mapped in the order of occurrence; In our example, the implementation will treat `audio_input` as port `0`, `audio_output` as port `1` and so on. Therefore, your plugin definition and your port collection have to match. Otherwise, undefined behaviour will occur.
pub trait PortCollection: Sized {
    /// The type of the port pointer cache.
    ///
    /// The host passes port pointers to the plugin one by one and in an undefined order. Therefore, the plugin instance can not collect these pointers in the port collection directly. Instead, the pointers are stored in a cache which is then used to create the proper port collection.
    type Connections: PortConnections;

    /// Try to construct a port collection instance from a port pointer cache.
    ///
    /// If one of the port connection pointers is null, this method will return `None`, because a `PortCollection` can not be constructed.
    ///
    /// # Safety
    ///
    /// Since the pointer cache is only storing the pointers, implementing this method requires the de-referencation of raw pointers and therefore, this method is unsafe.
    unsafe fn from_connections(cache: &Self::Connections, sample_count: u32) -> Option<Self>;
}

impl PortCollection for () {
    type Connections = ();

    unsafe fn from_connections(_cache: &(), _sample_count: u32) -> Option<Self> {
        Some(())
    }
}

impl<T: PortCollection> PortCollection for Option<T> {
    type Connections = T::Connections;

    #[inline]
    unsafe fn from_connections(cache: &Self::Connections, sample_count: u32) -> Option<Self> {
        Some(T::from_connections(cache, sample_count))
    }
}
