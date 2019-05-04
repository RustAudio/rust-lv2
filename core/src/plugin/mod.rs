mod descriptor;
mod features;
mod ports;

pub use descriptor::*;
pub use features::*;
pub use ports::*;

pub use lv2_core_derive::*;

use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use crate::sys::LV2_Handle;
use crate::uri::Uri;
use crate::{FeatureList, RawFeatureDescriptor};

pub trait Plugin: Sized {
    type Ports: Lv2Ports;
    type Features: Lv2Features;

    fn new(
        plugin_uri: &Uri,
        sample_rate: f64,
        bundle_path: &CStr,
        features: Self::Features,
    ) -> Self;
    fn run(&mut self, ports: &Self::Ports);

    #[inline]
    fn activate(&mut self) {}
    #[inline]
    fn deactivate(&mut self) {}
}

pub trait Lv2Ports: Sized {
    type Connections: PortsConnections;

    fn from_connections(connections: &Self::Connections, sample_count: u32) -> Self;
}

pub trait PortsConnections: Sized + Default {
    unsafe fn connect(&mut self, index: u32, pointer: *mut ());
}

pub struct PluginInstance<T: Plugin> {
    instance: T,
    connections: <T::Ports as Lv2Ports>::Connections,
}

impl<T: Plugin> PluginInstance<T> {
    pub unsafe extern "C" fn instantiate(
        descriptor: *const PluginDescriptor<Self>,
        sample_rate: f64,
        bundle_path: *const c_char,
        features: *const *const RawFeatureDescriptor,
    ) -> LV2_Handle {
        let descriptor = match descriptor.as_ref() {
            Some(descriptor) => descriptor,
            None => {
                eprintln!("Failed to initialize plugin: Descriptor points to null");
                return std::ptr::null_mut();
            }
        };
        let plugin_uri = Uri::from_cstr_unchecked(CStr::from_ptr(descriptor.URI));

        let bundle_path = CStr::from_ptr(bundle_path);

        let feature_list = FeatureList::from_raw(features);
        let features = match <T::Features as Lv2Features>::from_feature_list(feature_list) {
            Ok(features) => features,
            Err(error) => {
                eprintln!("Failed to initialize plugin: {:?}", error);
                return std::ptr::null_mut();
            }
        };

        let instance = Box::new(Self {
            instance: T::new(plugin_uri, sample_rate, bundle_path, features),
            connections: <<T::Ports as Lv2Ports>::Connections as Default>::default(),
        });
        Box::leak(instance) as *mut Self as LV2_Handle
    }

    pub unsafe extern "C" fn cleanup(instance: *mut Self) {
        Box::from_raw(instance);
    }

    pub unsafe extern "C" fn activate(instance: *mut Self) {
        (*instance).instance.activate()
    }

    pub unsafe extern "C" fn deactivate(instance: *mut Self) {
        (*instance).instance.deactivate()
    }

    pub unsafe extern "C" fn connect_port(instance: *mut Self, port: u32, data: *mut c_void) {
        (*instance).connections.connect(port, data as *mut ())
    }

    pub unsafe extern "C" fn run(instance: *mut Self, sample_count: u32) {
        let instance = &mut *instance;
        let ports = <T::Ports as Lv2Ports>::from_connections(&instance.connections, sample_count);
        instance.instance.run(&ports);
    }

    pub unsafe extern "C" fn extension_data(_uri: *const c_char) -> *const c_void {
        // TODO
        ::std::ptr::null()
    }
}

pub unsafe trait PluginInstanceDescriptor: Plugin {
    const URI: &'static [u8];
    const DESCRIPTOR: PluginDescriptor<PluginInstance<Self>>;
}
