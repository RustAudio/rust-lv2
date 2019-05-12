use crate::uri::{Uri, UriError};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::str::Utf8Error;

#[derive(Debug)]
#[doc(hidden)]
pub(crate) enum PluginInfoError {
    InvalidPluginUri(UriError),
    InvalidBundlePathUtf8(Utf8Error),
}

/// Holds various data that is passed from the host at plugin instantiation time.
///
/// See the methods' documentation to know which data is available.
pub struct PluginInfo<'a> {
    plugin_uri: &'a Uri,
    bundle_path: &'a Path,
    sample_rate: f64,
}

impl<'a> PluginInfo<'a> {
    #[doc(hidden)]
    pub(crate) unsafe fn from_raw(
        plugin_descriptor: *const crate::sys::_LV2_Descriptor,
        bundle_path: *const c_char,
        sample_rate: f64,
    ) -> Result<Self, PluginInfoError> {
        Self::new(
            CStr::from_ptr((*plugin_descriptor).URI),
            CStr::from_ptr(bundle_path),
            sample_rate,
        )
    }

    #[doc(hidden)]
    pub(crate) fn new(
        plugin_uri: &'a CStr,
        bundle_path: &'a CStr,
        sample_rate: f64,
    ) -> Result<Self, PluginInfoError> {
        let plugin_uri = Uri::from_cstr(plugin_uri).map_err(PluginInfoError::InvalidPluginUri)?;

        let bundle_path = Path::new(
            bundle_path
                .to_str()
                .map_err(PluginInfoError::InvalidBundlePathUtf8)?,
        );

        Ok(Self {
            sample_rate,
            plugin_uri,
            bundle_path,
        })
    }

    /// The URI of the plugin that is being instantiated.
    #[inline]
    pub fn plugin_uri(&self) -> &Uri {
        self.plugin_uri
    }

    /// The path to the LV2 bundle directory which contains this plugin binary.
    ///
    /// This is useful to get if the plugin needs to store extra resources in its bundle directory,
    /// such as presets, or any other kind of data.
    #[inline]
    pub fn bundle_path(&self) -> &Path {
        self.bundle_path
    }

    /// The sample rate, in Hz, that is being used by the host.
    /// The host will always send audio data to the plugin at this sample rate.
    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}
