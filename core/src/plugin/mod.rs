mod features;
pub(crate) mod info;
mod ports;

pub use features::*;
pub use info::PluginInfo;
pub use ports::*;

pub use lv2_core_derive::*;

use crate::FeatureList;
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
}

pub trait PortContainer: Sized {
    type Connections: PortsConnections;

    fn from_connections(connections: &Self::Connections, sample_count: u32) -> Self;
}

pub trait PortsConnections: Sized + Default {
    unsafe fn connect(&mut self, index: u32, pointer: *mut ());
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

    pub unsafe extern "C" fn extension_data(_uri: *const c_char) -> *const c_void {
        // TODO
        ::std::ptr::null()
    }
}

pub unsafe trait PluginInstanceDescriptor: Plugin {
    const URI: &'static [u8];
    const DESCRIPTOR: sys::LV2_Descriptor;
}
