use crate::feature::FeatureList;
use crate::uri::Uri;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

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

pub trait Lv2Features: Sized {
    fn from_feature_list(
        feature_list: FeatureList<'static>,
    ) -> Result<Self, FeatureResolutionError>;
}

impl Lv2Features for () {
    #[inline(always)]
    fn from_feature_list(
        _feature_list: FeatureList<'static>,
    ) -> Result<Self, FeatureResolutionError> {
        Ok(())
    }
}
