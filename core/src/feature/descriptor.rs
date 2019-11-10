//! This module is for internal organization only and is not meant to be exposed.

use crate::feature::Feature;
use std::ffi::{c_void, CStr};

/// A dummy pointer to be used to represent zero-sized features.
///
/// See the `cast_feature_ref` function below for details.
static DUMMY_FEATURE_POINTER: () = ();

/// An helper functions that casts a raw feature pointer to a Rust reference to the given feature
/// type.
/// If the given pointer is null, this function returns `None`.
///
/// This function does an additional check to handle the case where the reference is null but the
/// Feature type is zero-sized (examples include `HardRTCapable` or `InPlaceBroken`).
/// If that is the case, a reference to a dummy value is returned instead, which is just as invalid
/// to read but is non-zero at least.
///
/// C LV2 hosts legitimately do this because they have literally nowhere to point to, but in Rust
/// creating a null reference is instant Undefined Behavior even if the reference itself is never
/// read and/or zero-sized (e.g. because of `Option`).
/// 
/// # Safety
/// 
/// This function may create an reference to a value that isn't of type `T`. Therefore, you should be cautious when you use it.
/// 
/// The validity of this method currently under debate and may be replaced soon.
#[inline]
pub unsafe fn cast_feature_ref<'a, T: Feature<'a>>(feature: *const c_void) -> Option<&'a T> {
    if ::std::mem::size_of::<T>() == 0 && feature.is_null() {
        Some(&*(&DUMMY_FEATURE_POINTER as *const () as *const T))
    } else {
        (feature as *const T).as_ref()
    }
}

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
    pub fn is_feature<T: Feature<'a>>(&self) -> bool {
        self.uri == T::uri()
    }

    /// Try to return a reference the data of the feature.
    ///
    /// If this object describes the requested feature, it will be created from the raw data. This operation consumes the descriptor since it would be possible to have multiple features instances otherwise.
    ///
    /// If the feature construction fails, or if the pointer is `null`, the descriptor will be returned again.
    pub fn into_feature<T: Feature<'a>>(self) -> Result<&'a T, Self> {
        if self.uri == T::uri() {
            unsafe { cast_feature_ref(self.data) }.ok_or(self)
        } else {
            Err(self)
        }
    }
}
