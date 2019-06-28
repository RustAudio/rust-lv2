mod features;
pub(crate) mod info;
pub mod port;

pub use self::features::Lv2Features;
pub use info::PluginInfo;
pub use lv2_core_derive::*;

use std::ffi::c_void;
use std::os::raw::c_char;
use sys::LV2_Handle;

use crate::feature::FeatureList;

/// Container for port handling.
///
/// Plugins do not have to port management on their own. Instead, they define a struct with all of the required ports, derive `PortContainer` for them and add them as their `Ports` type.
///
/// Then, the plugin instance will collect the port pointers from the host and create a `PortContainer` instance for every `run` call. Using this instance, plugins have access to all of their required ports.
pub trait PortContainer: Sized {
    /// The type of the port pointer cache.
    type Cache: PortPointerCache;

    /// Try to construct a port container instance from a port pointer cache.
    ///
    /// If one of the port connection pointers is null, this method will return `None`, because a `PortContainer` can not be constructed.
    ///
    /// # unsafety
    ///
    /// Implementing this method requires the de-referencation of raw pointers and therefore, this method is unsafe.
    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self>;
}

/// Cache for port connection pointers.
///
/// The host will pass the port connection pointers in an undefined order. Therefore, the `PortContainer` struct can not be created instantly. Instead, the pointers will be stored in a cache, which is then used to create a proper port container for the plugin.
pub trait PortPointerCache: Sized + Default {
    /// Store the connection pointer for the port with index `index`.
    fn connect(&mut self, index: u32, pointer: *mut c_void);
}

/// The main trait to implement to create an LV2 plugin instance.
pub trait Plugin: Sized + Send + Sync {
    /// See the docs for [PortContainer](lv2_core::PortContainer)
    type Ports: PortContainer;
    type Features: Lv2Features;

    fn new(plugin_info: &PluginInfo, features: Self::Features) -> Self;
    fn run(&mut self, ports: &mut Self::Ports);

    #[inline]
    fn activate(&mut self) {}
    #[inline]
    fn deactivate(&mut self) {}
}

pub struct PluginInstance<T: Plugin> {
    instance: T,
    connections: <T::Ports as PortContainer>::Cache,
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
            connections: <<T::Ports as PortContainer>::Cache as Default>::default(),
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
        (*instance).connections.connect(port, data)
    }

    pub unsafe extern "C" fn run(instance: *mut c_void, sample_count: u32) {
        let instance = instance as *mut Self;
        let ports =
            <T::Ports as PortContainer>::from_connections(&(*instance).connections, sample_count);
        if let Some(mut ports) = ports {
            (*instance).instance.run(&mut ports);
        }
    }

    pub unsafe extern "C" fn extension_data(_uri: *const c_char) -> *const c_void {
        // TODO
        ::std::ptr::null()
    }
}

#[doc(hidden)]
pub unsafe trait PluginInstanceDescriptor: Plugin {
    const URI: &'static [u8];
    const DESCRIPTOR: sys::LV2_Descriptor;
}
