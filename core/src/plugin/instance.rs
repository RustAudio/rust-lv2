use crate::extension::ExtensionInterface;
use crate::feature::*;
use crate::plugin::{Plugin, PluginInfo};
use crate::port::PortCollection;
use crate::port::*;
use crate::prelude::LV2_Descriptor;
use std::ffi::c_void;
use std::os::raw::c_char;
use sys::LV2_Handle;
use urid::Uri;

/// Plugin wrapper which translated between the host and the plugin.
///
/// The host interacts with the plugin via a C API, but the plugin is implemented with ideomatic, safe Rust. To bridge this gap, this wrapper is used to translate and abstract the communcation between the host and the plugin.
///
/// This struct is `repr(C)` and has the plugin as it's first field. Therefore, a valid `*mut PluginInstance<T>` is also a valid `*mut T`.
#[repr(C)]
pub struct PluginInstance<T: Plugin> {
    /// The plugin instance.
    instance: T,
    /// A temporary storage for all ports of the plugin.
    connections: <T::Ports as PortCollection>::Cache,
    /// All features that may be used in the initialization threading class.
    init_features: T::InitFeatures,
    /// All features that may be used in the audio threading class.
    audio_features: T::AudioFeatures,
}

impl<T: Plugin> PluginInstance<T> {
    /// Try to create a port collection from the currently collected connections.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it needs to dereference raw pointers, which are only valid if the method is called in the "Audio" threading class.
    pub unsafe fn ports(&self, sample_count: u32) -> Option<T::Ports> {
        <T::Ports as PortCollection>::from_connections(&self.connections, sample_count)
    }

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
        let mut init_features_cache = FeatureCache::from_raw(features);
        let mut audio_features_cache = init_features_cache.clone();

        let mut init_features = match T::InitFeatures::from_cache(
            &mut init_features_cache,
            ThreadingClass::Instantiation,
        ) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}", e);
                return std::ptr::null_mut();
            }
        };
        let audio_features =
            match T::AudioFeatures::from_cache(&mut audio_features_cache, ThreadingClass::Audio) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    return std::ptr::null_mut();
                }
            };

        // Instantiate the plugin.
        match T::new(&plugin_info, &mut init_features) {
            Some(instance) => {
                let instance = Box::new(Self {
                    instance,
                    connections: <<T::Ports as PortCollection>::Cache as Default>::default(),
                    init_features,
                    audio_features,
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
        instance.instance.activate(&mut instance.init_features)
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
        instance.instance.deactivate(&mut instance.init_features)
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
        if let Some(mut ports) = instance.ports(sample_count) {
            instance
                .instance
                .run(&mut ports, &mut instance.audio_features, sample_count);
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

        T::extension_data(uri)
            .map(ExtensionInterface::get_ptr)
            .unwrap_or_else(core::ptr::null)
    }

    /// Retrieve the internal plugin.
    pub fn plugin_handle(&mut self) -> &mut T {
        &mut self.instance
    }

    /// Retrieve the required handles to execute an Initialization class method.
    ///
    /// This method can be used by extensions to call an extension method in the Initialization threading class and provide it the host features for that class.
    pub fn init_class_handle(&mut self) -> (&mut T, &mut T::InitFeatures) {
        (&mut self.instance, &mut self.init_features)
    }

    /// Retrieve the required handles to execute an Audio class method.
    ///
    /// This method can be used by extensions to call an extension method in the Audio threading class and provide it the host features for that class.
    pub fn audio_class_handle(&mut self) -> (&mut T, &mut T::AudioFeatures) {
        (&mut self.instance, &mut self.audio_features)
    }
}

#[doc(hidden)]
pub unsafe trait PluginInstanceDescriptor {
    const DESCRIPTOR: sys::LV2_Descriptor;
}

unsafe impl<T: Plugin> PluginInstanceDescriptor for T {
    const DESCRIPTOR: LV2_Descriptor = LV2_Descriptor {
        URI: T::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
        instantiate: Some(PluginInstance::<T>::instantiate),
        connect_port: Some(PluginInstance::<T>::connect_port),
        activate: Some(PluginInstance::<T>::activate),
        run: Some(PluginInstance::<T>::run),
        deactivate: Some(PluginInstance::<T>::deactivate),
        cleanup: Some(PluginInstance::<T>::cleanup),
        extension_data: Some(PluginInstance::<T>::extension_data),
    };
}
