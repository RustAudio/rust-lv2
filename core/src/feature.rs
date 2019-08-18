//! Additional host functionalities.
use crate::UriBound;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};

/// Trait to generalize the feature detection system.
///
/// A host that only implements the core LV2 specification does not have much functionality. Therefore, host can provide extra functionalities, called "Features", a plugin can use to become more useful.
///
/// A native plugin written in C would discover a host's features by iterating through an array of URIs and pointers. When it finds the URI of the feature it is looking for, it casts the pointer to the type of the feature interface and uses the information from the interface.
///
/// In Rust, most of this behaviour is done internally and instead of simply casting a pointer, a safe feature descriptor, which implements this trait, is constructed using the [`from_raw_data`](#tymethod.from_raw_data) method.
pub trait Feature<'a>: UriBound + Sized {
    /// The type that is used by the C interface to contain a feature's data.
    ///
    /// This should be the struct type defined by the specification, contained in your `sys` crate, if you have one.
    type RawDataType: 'static;

    /// Create a feature object from raw data.
    fn from_raw_data(data: Option<&'a mut Self::RawDataType>) -> Option<Self>;
}

/// Marker feature to signal that the plugin can run in a hard real-time environment.
pub struct HardRTCapable;

unsafe impl UriBound for HardRTCapable {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__hardRTCapable;
}

impl<'a> Feature<'a> for HardRTCapable {
    type RawDataType = c_void;

    fn from_raw_data(_data: Option<&mut c_void>) -> Option<Self> {
        Some(Self {})
    }
}

/// Marker feature to signal the host to avoid in-place operation.
///
/// This feature has to be required by any plugin that may break if ANY input port is connected to the same memory location as ANY output port.
pub struct InPlaceBroken;

unsafe impl UriBound for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__inPlaceBroken;
}

impl<'a> Feature<'a> for InPlaceBroken {
    type RawDataType = c_void;

    fn from_raw_data(_data: Option<&mut c_void>) -> Option<Self> {
        Some(Self {})
    }
}

/// Marker feature to signal the host to only run the plugin in a live environment.
pub struct IsLive;

unsafe impl UriBound for IsLive {
    const URI: &'static [u8] = ::lv2_core_sys::LV2_CORE__isLive;
}

impl<'a> Feature<'a> for IsLive {
    type RawDataType = c_void;

    fn from_raw_data(_data: Option<&mut c_void>) -> Option<Self> {
        Some(Self {})
    }
}

/// Descriptor of a single host feature.
pub struct FeatureDescriptor<'a> {
    uri: &'a CStr,
    data: *mut c_void,
}

impl<'a> FeatureDescriptor<'a> {
    /// Return the URI of the feature.
    pub fn uri(&self) -> &CStr {
        self.uri
    }

    /// Return the data pointer of the feature.
    pub fn data(&self) -> *mut c_void {
        self.data
    }

    /// Evaluate whether this object describes the given feature.
    pub fn is_feature<T: Feature<'a>>(&self) -> bool {
        self.uri == T::uri()
    }

    /// Try to return a reference the data of the feature.
    ///
    /// If this object describes the requested feature, it will be created from the raw data. This operation consumes the descriptor since it would be possible to have multiple features instances otherwise.
    ///
    /// If the feature construction fails, the descriptor will be returned again.
    pub fn into_feature<T: Feature<'a>>(self) -> Result<T, Self> {
        if self.uri == T::uri() {
            if let Some(feature) =
                T::from_raw_data(unsafe { (self.data as *mut T::RawDataType).as_mut() })
            {
                Ok(feature)
            } else {
                Err(self)
            }
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
    internal: HashMap<&'a CStr, *mut c_void>,
}

impl<'a> FeatureContainer<'a> {
    /// Construct a container from the raw features array.
    ///
    /// It basically populates a hash map by walking through the array and then creates a `FeatureContainer` with it. However, this method is unsafe since it dereferences a C string to a URI. Also, this method should only be used with the features list supplied by the host since the soundness of the whole module depends on that assumption.
    pub unsafe fn from_raw(raw: *const *const ::sys::LV2_Feature) -> Self {
        let mut internal_map = HashMap::new();
        let mut feature_ptr = raw;

        if !raw.is_null() {
            while !(*feature_ptr).is_null() {
                let uri = CStr::from_ptr((**feature_ptr).URI);
                let data = (**feature_ptr).data;
                internal_map.insert(uri, data);
                feature_ptr = feature_ptr.add(1);
            }
        }

        Self {
            internal: internal_map,
        }
    }

    /// Evaluate whether this object contains the requested feature.
    pub fn contains<T: Feature<'a>>(&self) -> bool {
        self.internal.contains_key(T::uri())
    }

    /// Try to retrieve a feature.
    ///
    /// If feature is not found, this method will return `None`. Since the resulting feature object may have writing access to the raw data, it will be removed from the container to avoid the existence of two feature objects with writing access.
    pub fn retrieve_feature<T: Feature<'a>>(&mut self) -> Option<T> {
        self.internal
            .remove(T::uri())
            .and_then(|ptr| T::from_raw_data(unsafe { (ptr as *mut T::RawDataType).as_mut() }))
    }
}

use std::collections::hash_map;
use std::iter::Map;
type HashMapIterator<'a> = hash_map::IntoIter<&'a CStr, *mut c_void>;
type DescriptorBuildFn<'a> = fn((&'a CStr, *mut c_void)) -> FeatureDescriptor<'a>;

impl<'a> std::iter::IntoIterator for FeatureContainer<'a> {
    type Item = FeatureDescriptor<'a>;
    type IntoIter = Map<HashMapIterator<'a>, DescriptorBuildFn<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.internal.into_iter().map(|element| {
            let uri = element.0;
            let data = element.1;
            FeatureDescriptor { uri, data }
        })
    }
}

/// Convenience trait for feature collections.
///
/// The feature container is only for temporary use; Once a feature is retrieved, it is removed from the container. Therefore you need a way to properly store features.
///
/// You can simply create a struct with features as it's fields and derive `FeatureCollection` for it. A procedural macro will then create a method that populates the struct from the container, or returns `None` if one of the required features is not in the container.
///
/// An example using the few built-in features:
///
///     use lv2_core::plugin::*;
///     use lv2_core::feature::*;
///
///     #[derive(FeatureCollection)]
///     struct MyCollection {
///         hardrt: HardRTCapable,
///         live: IsLive,
///     }
pub trait FeatureCollection: Sized {
    /// Populate a collection with features from the container.
    fn from_container(container: &mut FeatureContainer) -> Option<Self>;
}

#[cfg(test)]
mod tests {
    use crate::{feature::*, plugin::*, UriBound};
    use std::os::raw::c_char;

    struct FeatureA {
        pub number: i32,
    }

    struct FeatureB {
        number: f32,
    }

    unsafe impl UriBound for FeatureA {
        const URI: &'static [u8] = b"urn:lv2Feature:A\0";
    }

    impl<'a> Feature<'a> for FeatureA {
        type RawDataType = i32;

        fn from_raw_data(data: Option<&mut i32>) -> Option<Self> {
            if let Some(data) = data {
                Some(FeatureA { number: *data })
            } else {
                None
            }
        }
    }

    unsafe impl UriBound for FeatureB {
        const URI: &'static [u8] = b"urn:lv2Fearure:B\0";
    }

    impl<'a> Feature<'a> for FeatureB {
        type RawDataType = f32;

        fn from_raw_data(data: Option<&mut f32>) -> Option<Self> {
            if let Some(data) = data {
                Some(FeatureB { number: *data })
            } else {
                None
            }
        }
    }

    #[derive(FeatureCollection)]
    struct Collection {
        a: FeatureA,
        b: FeatureB,
    }

    struct FeatureTestSetting<'a> {
        pub data_a: Box<i32>,
        pub feature_a_sys: Box<::sys::LV2_Feature>,
        pub data_b: Box<f32>,
        pub feature_b_sys: Box<::sys::LV2_Feature>,
        pub features_container: FeatureContainer<'a>,
    }

    impl<'a> FeatureTestSetting<'a> {
        fn new() -> Self {
            let mut data_a: Box<i32> = Box::new(42);
            let feature_a_sys = Box::new(::sys::LV2_Feature {
                URI: FeatureA::URI.as_ptr() as *const c_char,
                data: data_a.as_mut() as *mut i32 as *mut c_void,
            });

            let mut data_b: Box<f32> = Box::new(17.0);
            let feature_b_sys = Box::new(::sys::LV2_Feature {
                URI: FeatureB::URI.as_ptr() as *const c_char,
                data: data_b.as_mut() as *mut f32 as *mut c_void,
            });

            let features_list: &[*const sys::LV2_Feature] = &[
                feature_a_sys.as_ref(),
                feature_b_sys.as_ref(),
                std::ptr::null(),
            ];

            // Constructing the container.
            let features_container = unsafe { FeatureContainer::from_raw(features_list.as_ptr()) };

            Self {
                data_a,
                feature_a_sys,
                data_b,
                feature_b_sys,
                features_container,
            }
        }
    }

    #[test]
    fn test_feature_container() {
        // Constructing the test case.
        let setting = FeatureTestSetting::new();
        let mut features_container = setting.features_container;

        // Testing the container.
        assert!(features_container.contains::<FeatureA>());
        assert!(features_container.contains::<FeatureB>());

        let retrieved_feature_a = features_container.retrieve_feature::<FeatureA>().unwrap();
        assert!(retrieved_feature_a.number == *(setting.data_a));

        let retrieved_feature_b = features_container.retrieve_feature::<FeatureB>().unwrap();
        assert!(retrieved_feature_b.number == *(setting.data_b));
    }

    #[test]
    fn test_feature_descriptor() {
        // Constructing the test case.
        let setting = FeatureTestSetting::new();
        let features_container = setting.features_container;

        // Collect all items from the feature iterator.
        let feature_descriptors: Vec<FeatureDescriptor> = features_container.into_iter().collect();

        // Test the collected items.
        assert_eq!(feature_descriptors.len(), 2);

        let mut feature_a_found = false;
        let mut feature_b_found = false;
        for descriptor in feature_descriptors {
            if descriptor.is_feature::<FeatureA>() {
                if let Ok(retrieved_feature_a) = descriptor.into_feature::<FeatureA>() {
                    assert!(retrieved_feature_a.number == *(setting.data_a));
                } else {
                    panic!("Feature interpretation failed!");
                }
                feature_a_found = true;
            } else if descriptor.is_feature::<FeatureB>() {
                if let Ok(retrieved_feature_b) = descriptor.into_feature::<FeatureB>() {
                    assert!(retrieved_feature_b.number == *(setting.data_b));
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

    #[test]
    fn test_feature_collection() {
        // Construct the setting.
        let setting = FeatureTestSetting::new();
        let mut features_container = setting.features_container;

        let container = Collection::from_container(&mut features_container).unwrap();
        assert_eq!(container.a.number, *(setting.data_a));
        assert_eq!(container.b.number, *(setting.data_b));
    }
}
