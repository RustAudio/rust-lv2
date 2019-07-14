//! Types to create plugins.
pub(crate) mod info;
pub mod port;

pub use info::PluginInfo;
pub use lv2_core_derive::*;

use crate::feature::Feature;
use crate::uri::Uri;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use sys::LV2_Handle;

/// Container for port handling.
///
/// Plugins do not handle port management on their own. Instead, they define a struct with all of the required ports. Then, the plugin instance will collect the port pointers from the host and create a `PortContainer` instance for every `run` call. Using this instance, plugins have access to all of their required ports.
///
/// # Implementing
///
/// The most convenient way to create port containers is to define a struct with port types from the [`port`](port/index.html) module and then simply derive `PortContainer` for it. An example:
///
///     use lv2_core::plugin::PortContainer;
///     use lv2_core::plugin::port::*;
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
    /// # unsafety
    ///
    /// Since the pointer cache is only storing the pointers, implementing this method requires the de-referencation of raw pointers and therefore, this method is unsafe.
    unsafe fn from_connections(cache: &Self::Cache, sample_count: u32) -> Option<Self>;
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

/// The central trait to describe LV2 plugins.
///
/// This trait and the structs that implement it are the centre of every plugin project, since it hosts the `run` method. This method is called by the host for every processing cycle.
///
/// However, the host will not directly talk to the plugin. Instead, it will create and talk to the [`PluginInstance`](struct.PluginInstance.html), which dereferences raw pointers, does safety checks and then calls the corresponding plugin methods.
pub trait Plugin: Sized + Send + Sync {
    /// The type of the port container.
    type Ports: PortContainer;

    /// Create a new plugin instance.
    ///
    /// This method only creates an instance of the plugin, it does not reset or set up it's internal state. This is done by the `activate` method.
    fn new(plugin_info: &PluginInfo, features: FeatureContainer) -> Self;

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
}

/// Descriptor of a single host feature.
pub struct FeatureDescriptor<'a> {
    uri: &'a Uri,
    data: *mut c_void,
}

impl<'a> FeatureDescriptor<'a> {
    /// Return the URI of the feature.
    pub fn uri(&self) -> &Uri {
        self.uri
    }

    /// Return the data pointer of the feature.
    pub fn data(&self) -> *mut c_void {
        self.data
    }

    /// Evaluate whether this object describes the given feature.
    pub fn is_feature<T: Feature>(&self) -> bool {
        self.uri == T::uri()
    }

    /// Try to return a reference the data of the feature.
    ///
    /// If this object describes the requested feature, it will be created from the raw data. This operation consumes the descriptor since it would be possible to have multiple features instances otherwise.
    /// 
    /// If the feature construction fails, the descriptor will be returned again.
    pub fn as_feature<T: Feature>(self) -> Result<T, Self> {
        if self.uri == T::uri() {
            Ok(unsafe { T::from_raw_data(self.data as *mut _) })
        } else {
            Err(self)
        }
    }
}

/// Container for host features.
///
/// At initialization time, a raw LV2 plugin receives a null-terminated array containing all requested host features. Obviously, this is not suited for safe Rust code and therefore, it needs an abstraction layer.
///
/// Internally, this struct contains a hash map which is filled the raw LV2 feature descriptors. Using this map, methods are defined to identify and retrieve features.
pub struct FeatureContainer<'a> {
    internal: HashMap<&'a Uri, *mut c_void>,
}

impl<'a> FeatureContainer<'a> {
    /// Construct a container from the raw features array.
    ///
    /// It basically populates a hash map by walking through the array and then creates a `FeatureContainer` with it. However, this method is unsafe since it dereferences a C string to a URI. Also, this method should only be used with the features list supplied by the host since the soundness of the whole module depends on that assumption.
    unsafe fn from_raw(raw: *const *const ::sys::LV2_Feature) -> Self {
        let mut internal_map = HashMap::new();
        let mut feature_ptr = raw;

        while !(*feature_ptr).is_null() {
            let uri = Uri::from_cstr_unchecked(CStr::from_ptr((**feature_ptr).URI));
            let data = (**feature_ptr).data;
            internal_map.insert(uri, data);
            feature_ptr = feature_ptr.add(1);
        }

        Self {
            internal: internal_map,
        }
    }

    /// Evaluate whether this object contains the requested feature.
    pub fn contains<T: Feature>(&self) -> bool {
        self.internal.contains_key(T::uri())
    }

    /// Try to retrieve a feature.
    ///
    /// If feature is not found, this method will return `None`. Since the resulting feature object may have writing access to the raw data, it will be removed from the container to avoid the existence of two feature objects with writing access.
    pub fn retrieve_feature<T: Feature>(&mut self) -> Option<T> {
        self.internal
            .remove(T::uri())
            .map(|ptr| unsafe { T::from_raw_data(ptr as *mut T::RawDataType) })
    }

    /// Iterate over all contained features.
    pub fn into_iter(self) -> impl std::iter::Iterator<Item = FeatureDescriptor<'a>> {
        self.internal.into_iter().map(|element| {
            let uri = element.0;
            let data = element.1;
            FeatureDescriptor { uri, data }
        })
    }
}

/// Plugin wrapper which translated between the host and the plugin.
///
/// The host interacts with the plugin via a C API, but the plugin is implemented with ideomatic, safe Rust. To bridge this gap, this wrapper is used to translate and abstract the communcation between the host and the plugin.
pub struct PluginInstance<T: Plugin> {
    instance: T,
    connections: <T::Ports as PortContainer>::Cache,
}

impl<T: Plugin> PluginInstance<T> {
    /// Instantiate the plugin.
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
        let features = FeatureContainer::from_raw(features);

        // Instantiate the plugin.
        let instance = Box::new(Self {
            instance: T::new(&plugin_info, features),
            connections: <<T::Ports as PortContainer>::Cache as Default>::default(),
        });
        Box::leak(instance) as *mut Self as LV2_Handle
    }

    /// Clean the plugin.
    pub unsafe extern "C" fn cleanup(instance: *mut c_void) {
        let instance = instance as *mut Self;
        Box::from_raw(instance);
    }

    /// Call `activate`.
    pub unsafe extern "C" fn activate(instance: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).instance.activate()
    }

    /// Call `deactivate`
    pub unsafe extern "C" fn deactivate(instance: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).instance.deactivate()
    }

    /// Update a port pointer.
    pub unsafe extern "C" fn connect_port(instance: *mut c_void, port: u32, data: *mut c_void) {
        let instance = instance as *mut Self;
        (*instance).connections.connect(port, data)
    }

    /// Construct a port container and call the `run` method.
    pub unsafe extern "C" fn run(instance: *mut c_void, sample_count: u32) {
        let instance = instance as *mut Self;
        let ports =
            <T::Ports as PortContainer>::from_connections(&(*instance).connections, sample_count);
        if let Some(mut ports) = ports {
            (*instance).instance.run(&mut ports);
        }
    }

    /// Return extension data.
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

#[cfg(test)]
pub unsafe fn create_feature_container<'a>(
    raw: *const *const ::sys::LV2_Feature,
) -> FeatureContainer<'a> {
    FeatureContainer::from_raw(raw)
}

#[cfg(test)]
mod tests {
    use crate::feature::*;
    use crate::plugin::*;
    use crate::uri::*;

    #[derive(Clone, Copy)]
    #[repr(C)]
    struct FeatureA {
        pub number: i32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    struct FeatureB {
        number: f32,
    }

    unsafe impl UriBound for FeatureA {
        const URI: &'static [u8] = b"urn:lv2Feature:A\0";
    }

    unsafe impl Feature for FeatureA {
        type RawDataType = i32;

        unsafe fn from_raw_data(data: *mut i32) -> Self {
            FeatureA { number: *data }
        }

        fn raw_data(&mut self) -> *mut i32 {
            &mut self.number
        }
    }

    unsafe impl UriBound for FeatureB {
        const URI: &'static [u8] = b"urn:lv2Fearure:B\0";
    }

    unsafe impl Feature for FeatureB {
        type RawDataType = f32;

        unsafe fn from_raw_data(data: *mut f32) -> Self {
            FeatureB { number: *data }
        }

        fn raw_data(&mut self) -> *mut f32 {
            &mut self.number
        }
    }

    #[test]
    fn test_feature_container() {
        // Constructing the test case.
        let mut feature_a = FeatureA { number: 42 };
        let feature_a_sys = feature_a.create_raw_feature();

        let mut feature_b = FeatureB { number: 17.0 };
        let feature_b_sys = feature_b.create_raw_feature();

        let features_list: &[*const ::sys::LV2_Feature] =
            &[&feature_a_sys, &feature_b_sys, std::ptr::null()];

        // Constructing the container.
        let mut features_container =
            unsafe { crate::plugin::create_feature_container(features_list.as_ptr()) };

        // Testing the container.
        assert!(features_container.contains::<FeatureA>());
        assert!(features_container.contains::<FeatureB>());

        let retrieved_feature_a = features_container.retrieve_feature::<FeatureA>().unwrap();
        assert!(retrieved_feature_a.number == feature_a.number);

        let retrieved_feature_b = features_container.retrieve_feature::<FeatureB>().unwrap();
        assert!(retrieved_feature_b.number == feature_b.number);
    }

    #[test]
    fn test_feature_descriptor() {
        // Constructing the test case.
        let mut feature_a = FeatureA { number: 42 };
        let feature_a_sys = feature_a.create_raw_feature();

        let mut feature_b = FeatureB { number: 17.0 };
        let feature_b_sys = feature_b.create_raw_feature();

        let features_list: &[*const ::sys::LV2_Feature] =
            &[&feature_a_sys, &feature_b_sys, std::ptr::null()];

        // Constructing the container.
        let features_container =
            unsafe { crate::plugin::create_feature_container(features_list.as_ptr()) };

        // Collect all items from the feature iterator.
        let feature_descriptors: Vec<FeatureDescriptor> = features_container.into_iter().collect();

        // Test the collected items.
        assert_eq!(feature_descriptors.len(), 2);

        let mut feature_a_found = false;
        let mut feature_b_found = false;
        for descriptor in feature_descriptors {
            if descriptor.is_feature::<FeatureA>() {
                if let Ok(retrieved_feature_a) = descriptor.as_feature::<FeatureA>() {
                    assert!(retrieved_feature_a.number == feature_a.number);
                } else {
                    panic!("Feature interpretation failed!");
                }
                feature_a_found = true;
            } else if descriptor.is_feature::<FeatureB>() {
                if let Ok(retrieved_feature_b) = descriptor.as_feature::<FeatureB>() {
                    assert!(retrieved_feature_b.number == feature_b.number);
                } else {
                    panic!("Feature interpretation failed!");
                }
                feature_b_found = true;
            } else {
                panic!("Invalid feature in feature iterator!");
            }
        }
        assert!(feature_a_found && feature_b_found);
    }
}
