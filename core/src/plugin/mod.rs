//! Types to create plugins.
pub(crate) mod info;

pub use info::PluginInfo;
pub use lv2_core_derive::*;

use crate::feature::*;
use crate::port::*;
use crate::{Uri, UriBound};
use std::any::Any;
use std::ffi::c_void;
use std::os::raw::c_char;
use sys::LV2_Handle;

/// The central trait to describe LV2 plugins.
///
/// This trait and the structs that implement it are the centre of every plugin project, since it hosts the `run` method. This method is called by the host for every processing cycle.
///
/// However, the host will not directly talk to the plugin. Instead, it will create and talk to the [`PluginInstance`](struct.PluginInstance.html), which dereferences raw pointers, does safety checks and then calls the corresponding plugin methods. However, it guarantees that a valid `sys::LV2_Handle` is always a valid `*mut MyPlugin`, where `MyPlugin` is your plugin's name.
pub trait Plugin: UriBound + Sized + Send + Sync + 'static {
    /// The type of the port collection.
    type Ports: PortCollection;

    /// The host features used by this plugin.
    type Features: FeatureCollection<'static>;

    /// Create a new plugin instance.
    ///
    /// This method only creates an instance of the plugin, it does not reset or set up it's internal state. This is done by the `activate` method.
    fn new(plugin_info: &PluginInfo, features: Self::Features) -> Option<Self>;

    /// Run a processing step.
    ///
    /// The host will always call this method after `active` has been called and before `deactivate` has been called.
    fn run(&mut self, ports: &mut Self::Ports);

    /// Reset and initialize the complete internal state of the plugin.
    ///
    /// This method will be called if the plugin has just been created of if the plugin has been deactivated. Also, a host's `activate` call will be as close as possible to the first `run` call.
    fn activate(&mut self) {}

    /// Deactivate the plugin.
    ///
    /// The host will always call this method when it wants to shut the plugin down. After `deactivate` has been called, `run` will not be called until `activate` has been called again.
    fn deactivate(&mut self) {}

    /// Return additional, extension-specific data.
    ///
    /// Sometimes, the methods from the `Plugin` trait aren't enough to support additional LV2 specifications. For these cases, extension exist. In most cases and for Rust users, an extension is simply a trait that can be implemented for a plugin.
    ///
    /// However, these implemented methods must be passed to the host. This is where this method comes into play: The host will call it with a URI for an extension. Then, it is the plugin's responsibilty to return the extension data to the host.
    ///
    /// In most cases, you can simply use the [`match_extensions`](../macro.match_extensions.html) macro to generate an appropiate method body.
    fn extension_data(_uri: &Uri) -> Option<&'static dyn Any> {
        None
    }
}

/// Plugin wrapper which translated between the host and the plugin.
///
/// The host interacts with the plugin via a C API, but the plugin is implemented with ideomatic, safe Rust. To bridge this gap, this wrapper is used to translate and abstract the communcation between the host and the plugin.
///
/// This struct is `repr(C)` and has the plugin as it's first field. Therefore, a valid `*mut PluginInstance<T>` is also a valid `*mut T`.
#[repr(C)]
pub struct PluginInstance<T: Plugin> {
    instance: T,
    connections: <T::Ports as PortCollection>::Cache,
}

impl<T: Plugin> PluginInstance<T> {
    /// Instantiate the plugin.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn instantiate(
        descriptor: *const sys::LV2_Descriptor,
        sample_rate: f64,
        bundle_path: *const c_char,
        features: *const *const sys::LV2_Feature,
    ) -> LV2_Handle {
        // Dereference the descriptor.
        let descriptor = match descriptor.as_ref() {
            Some(descriptor) => descriptor,
            None => {
                eprintln!("Failed to initialize plugin: Descriptor points to null");
                return std::ptr::null_mut();
            }
        };

        // Dereference the plugin info.
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

        // Collect the supported features.
        let mut features = FeatureCache::from_raw(features);

        let features = match <T::Features as FeatureCollection>::from_cache(&mut features) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}", e);
                return std::ptr::null_mut();
            }
        };

        // Instantiate the plugin.
        match T::new(&plugin_info, features) {
            Some(instance) => {
                let instance = Box::new(Self {
                    instance,
                    connections: <<T::Ports as PortCollection>::Cache as Default>::default(),
                });
                Box::leak(instance) as *mut Self as LV2_Handle
            }
            None => std::ptr::null_mut(),
        }
    }

    /// Clean the plugin.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn cleanup(instance: *mut c_void) {
        let instance = instance as *mut Self;
        Box::from_raw(instance);
    }

    /// Call `activate`.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn activate(instance: *mut c_void) {
        let instance = &mut *(instance as *mut Self);
        instance.instance.activate()
    }

    /// Call `deactivate`.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn deactivate(instance: *mut c_void) {
        let instance = &mut *(instance as *mut Self);
        instance.instance.deactivate()
    }

    /// Update a port pointer.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn connect_port(instance: *mut c_void, port: u32, data: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).connections.connect(port, data)
    }

    /// Construct a port collection and call the `run` method.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn run(instance: *mut c_void, sample_count: u32) {
        let instance = &mut *(instance as *mut Self);
        let ports =
            <T::Ports as PortCollection>::from_connections(&instance.connections, sample_count);
        if let Some(mut ports) = ports {
            instance.instance.run(&mut ports);
        }
    }

    /// Dereference the URI, call the `extension_data` function and return the pointer.
    ///
    /// This method provides a required method for the C interface of a plugin and is used by the `lv2_descriptors` macro.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it derefences multiple raw pointers and is part of the C interface.
    pub unsafe extern "C" fn extension_data(uri: *const c_char) -> *const c_void {
        let uri = Uri::from_ptr(uri);
        if let Some(data) = T::extension_data(uri) {
            data as *const _ as *const c_void
        } else {
            std::ptr::null()
        }
    }
}

#[doc(hidden)]
pub unsafe trait PluginInstanceDescriptor: Plugin {
    const DESCRIPTOR: sys::LV2_Descriptor;
}
