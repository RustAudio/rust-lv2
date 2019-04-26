use std::os::raw::c_char;
use std::ffi::c_void;

use crate::{PluginDescriptor, RawFeatureDescriptor, FeatureList};
use crate::plugin::features::Lv2Features;

pub trait Plugin: Sized {
    type Ports: Lv2Ports;
    type Features: Lv2Features;

    fn new(features: Self::Features) -> Self;
    fn run(&mut self, ports: &Self::Ports);

    #[inline] fn activate(&mut self) {}
    #[inline] fn deactivate(&mut self) {}
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
    connections: <T::Ports as Lv2Ports>::Connections
}

// TODO: add panic protection
impl<T: Plugin> PluginInstance<T> {
    pub unsafe extern "C" fn instanciate(_descriptor: *const PluginDescriptor<Self>,
                       _sample_rate: f64,
                       _bundle_path: *const c_char,
                       features: *const *const RawFeatureDescriptor) -> *mut Self {
        let feature_list = FeatureList::from_raw(features);

        let features = match <T::Features as Lv2Features>::from_feature_list(feature_list) {
            Ok(features) => features,
            Err(error) => {
                eprintln!("Failed to initialize plugin: {:?}", error); // TODO: better error management
                return ::std::ptr::null_mut()
            }
        };

        let instance = Self {
            instance: T::new(features),
            connections: <<T::Ports as Lv2Ports>::Connections as Default>::default()
        };
        Box::into_raw(Box::new(instance))
    }

    pub unsafe extern "C" fn cleanup(instance: *mut Self) {
        Box::from_raw(instance);
    }

    pub unsafe extern "C" fn activate(instance: *mut Self) {
        (&mut *instance).instance.activate()
    }

    pub unsafe extern "C" fn deactivate(instance: *mut Self) {
        (&mut *instance).instance.deactivate()
    }

    pub unsafe extern "C" fn connect_port(instance: *mut Self, port: u32, data: *mut c_void) {
        (&mut *instance).connections.connect(port, data as *mut ())
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
