//! This module is for internal organization only and is not meant to be exposed.

use crate::feature::Feature;
use std::ffi::{c_void, CStr};

/// Descriptor of a single host feature.
///
/// This struct is slightly different from the raw `LV2_Feature` struct, as the length of the
/// contained URI is precomputed for faster comparison.
pub struct FeatureDescriptor<'a> {
    pub(crate) uri: &'a CStr,
    pub(crate) data: *const c_void,
}

impl<'a> FeatureDescriptor<'a> {
    /// Return the URI of the feature.
    pub fn uri(&self) -> &CStr {
        self.uri
    }

    /// Returns the data pointer of the feature descriptor.
    pub fn data(&self) -> *const c_void {
        self.data
    }

    /// Evaluate whether this object describes the given feature.
    pub fn is_feature<T: Feature>(&self) -> bool {
        self.uri == T::uri()
    }

    /// Try to return a feature struct instance from the internal data.
    ///
    /// If this object describes the requested feature, it will be created from the raw data. This operation consumes the descriptor since it would be possible to have multiple features instances otherwise.
    ///
    /// If the feature construction fails, the descriptor will be returned again.
    pub fn into_feature<T: Feature>(self) -> Result<T, Self> {
        unsafe { T::from_feature_ptr(self.data) }.ok_or(self)
    }
}
