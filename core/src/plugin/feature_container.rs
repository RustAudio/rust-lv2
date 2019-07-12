use crate::feature::Feature;
use crate::uri::Uri;
use std::error::Error;
use std::ffi::{c_void, CStr};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Copy, Clone)]
pub struct FeatureDescriptor {
    uri: &'static Uri,
    data: *mut c_void,
}

impl FeatureDescriptor {
    pub unsafe fn from_raw(raw: *const sys::LV2_Feature) -> FeatureDescriptor {
        FeatureDescriptor {
            data: (*raw).data,
            uri: Uri::from_cstr_unchecked(CStr::from_ptr((*raw).URI)),
        }
    }

    pub fn into_raw(&self) -> ::sys::LV2_Feature {
        ::sys::LV2_Feature {
            URI: self.uri.as_ptr(),
            data: self.data,
        }
    }

    pub fn uri(&self) -> &Uri {
        self.uri
    }

    pub fn data(&self) -> *mut c_void {
        self.data
    }

    pub unsafe fn try_into_feature<T: Feature>(self) -> Option<&'static mut T> {
        if self.uri == T::uri() {
            (self.data as *mut T).as_mut()
        } else {
            None
        }
    }
}

impl fmt::Debug for FeatureDescriptor {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Feature")?;
        fmt::Debug::fmt(self.uri(), f)
    }
}

pub enum FeatureResolutionError {
    MissingRequiredFeature { uri: &'static Uri },
}

impl Debug for FeatureResolutionError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            FeatureResolutionError::MissingRequiredFeature { uri } => {
                write!(f, "Missing required feature: {}", uri)
            }
        }
    }
}

impl Display for FeatureResolutionError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        <FeatureResolutionError as Debug>::fmt(self, f)
    }
}

impl Error for FeatureResolutionError {}

pub trait FeatureContainer: Sized {
    fn from_feature_list(
        feature_list: &[FeatureDescriptor],
    ) -> Result<Self, FeatureResolutionError>;
}

impl FeatureContainer for () {
    #[inline(always)]
    fn from_feature_list(
        _feature_list: &[FeatureDescriptor],
    ) -> Result<Self, FeatureResolutionError> {
        Ok(())
    }
}
