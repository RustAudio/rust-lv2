//! Additional host functionalities.
use crate::{Uri, UriBound};

mod container;
mod core_features;
mod descriptor;

pub use container::FeatureContainer;
pub use core_features::*;
pub use descriptor::FeatureDescriptor;

/// Trait to generalize the feature detection system.
///
/// A host that only implements the core LV2 specification does not have much functionality. Therefore, host can provide extra functionalities, called "Features", a plugin can use to become more useful.
///
/// A native plugin written in C would discover a host's features by iterating through an array of URIs and pointers. When it finds the URI of the feature it is looking for, it casts the pointer to the type of the feature interface and uses the information from the interface.
///
/// In Rust, most of this behaviour is done internally and instead of simply casting a pointer, a safe feature descriptor, which implements this trait, is constructed using the [`from_raw_data`](#tymethod.from_raw_data) method.
pub unsafe trait Feature<'a>: UriBound + Sized + 'a {
    /// Returns a feature descriptor for this feature instance, to be passed down to plugins.
    #[cfg(feature = "host")]
    fn descriptor(&self) -> FeatureDescriptor<'a> {
        FeatureDescriptor {
            uri: &<Self as UriBound>::uri(),
            data: self as *const Self as *const std::ffi::c_void,
        }
    }
}

/// An error created during feature resolution when a required feature is missing.
#[derive(Copy, Clone, Debug)]
pub struct MissingFeatureError {
    pub(crate) uri: &'static Uri,
}

impl std::fmt::Display for MissingFeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let uri = self.uri.to_str().unwrap_or("[error while reading URI]");
        write!(
            f,
            "Unable to instantiate plugin: missing required feature: {}",
            uri
        )
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
///     use lv2_core::plugin::FeatureCollection;
///     use lv2_core::feature::{IsLive, HardRTCapable};
///
///     #[derive(FeatureCollection)]
///     struct MyCollection<'a> {
///         live: &'a IsLive,
///         hardrt: Option<&'a HardRTCapable>,
///     }
pub trait FeatureCollection<'a>: Sized + 'a {
    /// Populate a collection with features from the container.
    fn from_container(container: &mut FeatureContainer<'a>) -> Result<Self, MissingFeatureError>;
}

impl<'a> FeatureCollection<'a> for () {
    #[inline]
    fn from_container(_container: &mut FeatureContainer) -> Result<Self, MissingFeatureError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate as lv2_core;
    use crate::feature::FeatureContainer;
    use crate::{feature::*, plugin::*, UriBound};
    use std::ffi::c_void;
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

    unsafe impl<'a> Feature<'a> for FeatureA {}

    unsafe impl UriBound for FeatureB {
        const URI: &'static [u8] = b"urn:lv2Feature:B\0";
    }

    unsafe impl<'a> Feature<'a> for FeatureB {}

    #[derive(FeatureCollection)]
    struct Collection<'a> {
        a: &'a FeatureA,
        b: &'a FeatureB,
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

        let retrieved_feature_a: &FeatureA = features_container.retrieve_feature().unwrap();
        assert_eq!(retrieved_feature_a.number, *(setting.data_a));

        let retrieved_feature_b: &FeatureB = features_container.retrieve_feature().unwrap();
        assert!(retrieved_feature_b.number - *(setting.data_b) < std::f32::EPSILON);
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
                    assert_eq!(retrieved_feature_b.number, *(setting.data_b));
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
