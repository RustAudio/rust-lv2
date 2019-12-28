use crate::Uri;
use std::os::raw::c_char;
use std::path::Path;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum PluginInfoError {
    InvalidBundlePathUtf8(Utf8Error),
}

/// Holds various data that is passed from the host at plugin instantiation time.
pub struct PluginInfo<'a> {
    plugin_uri: &'a Uri,
    bundle_path: &'a Path,
    sample_rate: f64,
}

impl<'a> PluginInfo<'a> {
    /// Create a new plugin info instance from raw information.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it dereferences raw pointers. It panics when one of the pointers is null,
    /// but does not check the pointers for other validity.
    pub unsafe fn from_raw(
        plugin_descriptor: *const crate::sys::LV2_Descriptor,
        bundle_path: *const c_char,
        sample_rate: f64,
    ) -> Result<Self, PluginInfoError> {
        let bundle_path = Path::new(
            Uri::from_ptr(bundle_path)
                .to_str()
                .map_err(PluginInfoError::InvalidBundlePathUtf8)?,
        );
        Ok(Self::new(
            Uri::from_ptr((*plugin_descriptor).URI),
            bundle_path,
            sample_rate,
        ))
    }

    /// Create a new plugin info instance.
    pub fn new(plugin_uri: &'a Uri, bundle_path: &'a Path, sample_rate: f64) -> Self {
        Self {
            sample_rate,
            plugin_uri,
            bundle_path,
        }
    }

    /// The URI of the plugin that is being instantiated.
    pub fn plugin_uri(&self) -> &Uri {
        self.plugin_uri
    }

    /// The path to the LV2 bundle directory which contains this plugin binary.
    ///
    /// This is useful to get if the plugin needs to store extra resources in its bundle directory,
    /// such as presets, or any other kind of data.
    pub fn bundle_path(&self) -> &Path {
        self.bundle_path
    }

    /// The sample rate, in Hz, that is being used by the host.
    /// The host will always send audio data to the plugin at this sample rate.
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}
