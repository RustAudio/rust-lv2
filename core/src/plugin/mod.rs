//! Types to create plugins.
pub(crate) mod info;
mod instance;
pub use instance::*;

pub use info::PluginInfo;
pub use lv2_core_derive::*;

use crate::extension::ExtensionInterface;
use crate::feature::*;
use crate::port::*;
use urid::{Uri, UriBound};

/// The central trait to describe LV2 plugins.
///
/// This trait and the structs that implement it are the centre of every plugin project, since it hosts the `run` method. This method is called by the host for every processing cycle.
///
/// However, the host will not directly talk to the plugin. Instead, it will create and talk to the [`PluginInstance`](struct.PluginInstance.html), which dereferences raw pointers, does safety checks and then calls the corresponding plugin methods. However, it guarantees that a valid `sys::LV2_Handle` is always a valid `*mut MyPlugin`, where `MyPlugin` is your plugin's name.
pub trait Plugin: UriBound + Sized + Send + Sync + 'static {
    /// The type of the port collection.
    type Ports: PortCollection;

    /// The host features used by this plugin in the "Initialization" thread class.
    ///
    /// This collection will be created by the framework when the plugin is initialized and every
    /// method in the "Initialization" threading class has access to it via a mutable reference.
    ///
    /// If a host feature is missing, the plugin creation simply fails and your plugin host will tell you so. However, this collection may only contain features that are usable in the "Initialization" thread class. Otherwise, the backend may panic during initialization. Please consult each feature's documentation.
    type InitFeatures: FeatureCollection<'static>;

    /// The host features used by this plugin in the "Audio" thread class.
    ///
    /// This collection will be created by the framework when the plugin is initialized and every
    /// method in the "Audio" threading class has access to it via a mutable reference.
    ///
    /// If a host feature is missing, the plugin creation simply fails and your plugin host will tell you so. However, this collection may only contain features that are usable in the "Audio" thread class. Otherwise, the backend may panic during initialization. Please consult each feature's documentation.
    type AudioFeatures: FeatureCollection<'static>;

    /// Create a new plugin instance.
    ///
    /// This method only creates an instance of the plugin, it does not reset or set up it's internal state. This is done by the `activate` method.
    fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self>;

    /// Run a processing step.
    ///
    /// The host will always call this method after `active` has been called and before `deactivate` has been called.
    ///
    /// The sample count is the number of frames covered by this `run` call. Audio and CV ports will contain exactly `sample_count` frames. Please note that `sample_count` may be differ between calls.
    fn run(
        &mut self,
        ports: &mut Self::Ports,
        features: &mut Self::AudioFeatures,
        sample_count: u32,
    );

    /// Reset and initialize the complete internal state of the plugin.
    ///
    /// This method will be called if the plugin has just been created of if the plugin has been deactivated. Also, a host's `activate` call will be as close as possible to the first `run` call.
    fn activate(&mut self, _features: &mut Self::InitFeatures) {}

    /// Deactivate the plugin.
    ///
    /// The host will always call this method when it wants to shut the plugin down. After `deactivate` has been called, `run` will not be called until `activate` has been called again.
    fn deactivate(&mut self, _features: &mut Self::InitFeatures) {}

    /// Return additional, extension-specific data.
    ///
    /// Sometimes, the methods from the `Plugin` trait aren't enough to support additional LV2 specifications. For these cases, extension exist. In most cases and for Rust users, an extension is simply a trait that can be implemented for a plugin.
    ///
    /// However, these implemented methods must be passed to the host. This is where this method comes into play: The host will call it with a URI for an extension. Then, it is the plugin's responsibilty to return the extension data to the host.
    ///
    /// In most cases, you can simply use the [`match_extensions`](../macro.match_extensions.html) macro to generate an appropiate method body.
    fn extension_data(_uri: &Uri) -> Option<ExtensionInterface> {
        None
    }
}
