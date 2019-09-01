/// Container for host features.
///
/// At initialization time, a raw LV2 plugin receives a null-terminated array containing all requested host features. Obviously, this is not suited for safe Rust code and therefore, it needs an abstraction layer.
///
/// Internally, this struct contains a hash map which is filled the raw LV2 feature descriptors. Using this map, methods are defined to identify and retrieve features.
pub struct FeatureContainer<'a> {
    internal: HashMap<&'a CStr, *const c_void>,
}

impl<'a> FeatureContainer<'a> {
    /// Construct a container from the raw features array.
    ///
    /// It basically populates a hash map by walking through the array and then creates a `FeatureContainer` with it. However, this method is unsafe since it dereferences a C string to a URI. Also, this method should only be used with the features list supplied by the host since the soundness of the whole module depends on that assumption.
    pub(crate) unsafe fn from_raw(raw: *const *const ::sys::LV2_Feature) -> Self {
        let mut internal_map = HashMap::new();
        let mut feature_ptr = raw;

        if !raw.is_null() {
            while !(*feature_ptr).is_null() {
                let uri = CStr::from_ptr((**feature_ptr).URI);
                let data = (**feature_ptr).data as *const c_void;
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
    pub fn retrieve_feature<T: Feature<'a>>(&mut self) -> Option<&'a T> {
        self.internal
            .remove(T::uri())
            .and_then(|ptr| unsafe { cast_feature_ref(ptr) })
    }
}

use crate::feature::descriptor::cast_feature_ref;
use crate::feature::{Feature, FeatureCollection, FeatureDescriptor};
use std::collections::{hash_map, HashMap};
use std::ffi::{c_void, CStr};
use std::iter::Map;

type HashMapIterator<'a> = hash_map::IntoIter<&'a CStr, *const c_void>;
type DescriptorBuildFn<'a> = fn((&'a CStr, *const c_void)) -> FeatureDescriptor<'a>;

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

impl<'a> FeatureCollection<'a> for FeatureContainer<'a> {
    fn from_container(container: &mut FeatureContainer<'a>) -> Option<Self> {
        Some(FeatureContainer {
            internal: container.internal.clone(),
        })
    }
}