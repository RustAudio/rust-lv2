mod features;
pub(crate) mod info;
mod ports;

pub use features::*;
pub use info::PluginInfo;
pub use ports::*;

pub use lv2_core_derive::*;

use crate::uri::Uri;
use crate::FeatureList;
use std::any::Any;
use std::ffi::c_void;
use std::os::raw::c_char;
use sys::LV2_Handle;

pub trait Plugin: Sized + Send + Sync {
    type Ports: PortContainer;
    type Features: Lv2Features;

    fn new(plugin_info: &PluginInfo, features: Self::Features) -> Self;
    fn run(&mut self, ports: &Self::Ports);

    #[inline]
    fn activate(&mut self) {}
    #[inline]
    fn deactivate(&mut self) {}

    /// Return extension-specific data.
    ///
    /// There are some specifications for LV2 that require additional callback functions from the
    /// plugin. These callbacks are usually implemented as traits other plugins can implement.
    /// However, these additional functions have to be exported and this function is the usual way
    /// to return them.
    ///
    /// The host calls this function for every extension interface it wants to have, with `uri` set
    /// to the URI of the interface. Then, the plugin has to return the data required by the
    /// specification, or `None` if it doesn't support the interface.
    ///
    /// Usually, you don't have to worry about implementing this on your own, because most
    /// extension interfaces can be exported with the [`export_extension_interfaces`](../
    /// macro.export_extension_interface.html) macro.
    fn extension_data(_uri: &Uri) -> Option<&'static Any> {
        None
    }
}

pub trait PortContainer: Sized {
    type Connections: PortsConnections;

    fn from_connections(connections: &Self::Connections, sample_count: u32) -> Self;
}

impl PortContainer for () {
    type Connections = ();

    fn from_connections(_connections: &Self::Connections, _sample_count: u32) -> Self {}
}

pub trait PortsConnections: Sized + Default {
    unsafe fn connect(&mut self, index: u32, pointer: *mut ());
}

impl PortsConnections for () {
    unsafe fn connect(&mut self, _index: u32, _pointer: *mut ()) {}
}

pub struct PluginInstance<T: Plugin> {
    instance: T,
    connections: <T::Ports as PortContainer>::Connections,
}

impl<T: Plugin> PluginInstance<T> {
    pub unsafe extern "C" fn instantiate(
        descriptor: *const sys::LV2_Descriptor,
        sample_rate: f64,
        bundle_path: *const c_char,
        features: *const *const sys::LV2_Feature,
    ) -> LV2_Handle {
        let descriptor = match descriptor.as_ref() {
            Some(descriptor) => descriptor,
            None => {
                eprintln!("Failed to initialize plugin: Descriptor points to null");
                return std::ptr::null_mut();
            }
        };

        let plugin_info = match PluginInfo::from_raw(descriptor, bundle_path, sample_rate) {
            Ok(info) => info,
            Err(e) => {
                eprintln!(
                    "Failed to initialize plugin: Illegal info from host: {:?}",
                    e
                );
                return std::ptr::null_mut();
            }
        };

        let feature_list = FeatureList::from_raw(features);
        let features = match <T::Features as Lv2Features>::from_feature_list(feature_list) {
            Ok(features) => features,
            Err(error) => {
                eprintln!("Failed to initialize plugin: {:?}", error);
                return std::ptr::null_mut();
            }
        };

        let instance = Box::new(Self {
            instance: T::new(&plugin_info, features),
            connections: <<T::Ports as PortContainer>::Connections as Default>::default(),
        });
        Box::leak(instance) as *mut Self as LV2_Handle
    }

    pub unsafe extern "C" fn cleanup(instance: *mut c_void) {
        let instance = instance as *mut Self;
        Box::from_raw(instance);
    }

    pub unsafe extern "C" fn activate(instance: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).instance.activate()
    }

    pub unsafe extern "C" fn deactivate(instance: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).instance.deactivate()
    }

    pub unsafe extern "C" fn connect_port(instance: *mut c_void, port: u32, data: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).connections.connect(port, data as *mut ())
    }

    pub unsafe extern "C" fn run(instance: *mut c_void, sample_count: u32) {
        let instance = instance as *mut Self;
        let ports =
            <T::Ports as PortContainer>::from_connections(&(*instance).connections, sample_count);
        (*instance).instance.run(&ports);
    }

    pub unsafe extern "C" fn extension_data(uri: *const c_char) -> *const c_void {
        let uri: &Uri = Uri::from_ptr(uri);
        match T::extension_data(uri) {
            Some(data) => data as *const _ as *const c_void,
            None => ::std::ptr::null(),
        }
    }
}

pub unsafe trait PluginInstanceDescriptor: Plugin {
    const URI: &'static [u8];
    const DESCRIPTOR: sys::LV2_Descriptor;
}
