//! Means to extend the interface of a plugin.
//!
//! This module is relatively thin: It only contains a trait and a macro. Instead, most of the extension handling is a convention:
//!
//! An extension is a trait a plugin can implement and every extension has a descriptor: This is a marker struct that implements the [`ExtensionDescriptor`](trait.ExtensionDescriptor.html) trait for every plugin that implements the extension. This descriptor is then used by the [`match_extensions`](../macro.match_extensions.html) macro to generate the body of a plugin's `extension_data` method.
//!
//! # Example
//!
//! This is a complete example on how to create an extension and implement it for a plugin:
//!
//! ```
//! use lv2_core::extension::ExtensionDescriptor;
//! use lv2_core::prelude::*;
//! use std::any::Any;
//! use std::ffi::c_void;
//! use std::marker::PhantomData;
//! use std::path::Path;
//!
//! // ######################
//! // Defining the extension
//! // ######################
//!
//! /// The trait that actually extends the plugin.
//! pub trait MyExtension: Plugin {
//!     fn add_number(&mut self, number: u32);
//! }
//!
//! /// A descriptor for the plugin. This is just a marker type to associate constants and methods with.
//! pub struct MyExtensionDescriptor<P: MyExtension> {
//!     plugin: PhantomData<P>,
//! }
//!
//! #[repr(C)]
//! /// This struct would be part of a sys crate.
//! pub struct MyExtensionInterface {
//!     add_number: unsafe extern "C" fn(*mut c_void, number: u32),
//! }
//!
//! unsafe impl<P: MyExtension> UriBound for MyExtensionDescriptor<P> {
//!     const URI: &'static [u8] = b"urn:my-project:my-extension\0";
//! }
//!
//! impl<P: MyExtension> MyExtensionDescriptor<P> {
//!     /// The extern, unsafe version of the extending method.
//!     ///
//!     /// This is actually called by the host.
//!     unsafe extern "C" fn extern_add_number(handle: *mut c_void, number: u32) {
//!         let plugin = (handle as *mut P).as_mut().unwrap();
//!         plugin.add_number(number);
//!     }
//! }
//!
//! // Implementing the trait that contains the interface.
//! impl<P: MyExtension> ExtensionDescriptor for MyExtensionDescriptor<P> {
//!     type ExtensionInterface = MyExtensionInterface;
//!
//!     const INTERFACE: &'static MyExtensionInterface = &MyExtensionInterface {
//!         add_number: Self::extern_add_number,
//!     };
//! }
//!
//! // ##########
//! // The plugin
//! // ##########
//!
//! /// This plugin actually isn't a plugin, it only has a counter.
//! pub struct MyPlugin {
//!     internal: u32,
//! }
//!
//! unsafe impl UriBound for MyPlugin {
//!     const URI: &'static [u8] = b"urn:my-project:my-plugin\0";
//! }
//!
//! impl Plugin for MyPlugin {
//!     type Ports = ();
//!     type Features = ();
//!
//!     fn new(_: &PluginInfo, _: ()) -> Option<Self> {
//!         Some(Self { internal: 0 })
//!     }
//!
//!     fn run(&mut self, _: &mut ()) {
//!         self.internal += 1;
//!     }
//!
//!     fn extension_data(uri: &Uri) -> Option<&'static dyn Any> {
//!         // This macro use matches the given URI with the URIs of the given extension descriptors.
//!         // If one of them matches, it's interface is returned.
//!         //
//!         // Note that you have to add the type parameter. Otherwise, bad things may happen!
//!         match_extensions![uri, MyExtensionDescriptor<Self>]
//!     }
//! }
//!
//! // Actually implementing the extension.
//! impl MyExtension for MyPlugin {
//!     fn add_number(&mut self, number: u32) {
//!         self.internal += number;
//!     }
//! }
//!
//! // ########
//! // The host
//! // ########
//!
//! let plugin_uri: &Uri = MyPlugin::uri();
//! let bundle_path = Path::new("");
//! let sample_rate = 44100.0;
//! let plugin_info = PluginInfo::new(plugin_uri, bundle_path, sample_rate);
//!
//! let mut plugin = MyPlugin::new(&plugin_info, ()).unwrap();
//!
//! let extension = MyPlugin::extension_data(MyExtensionDescriptor::<MyPlugin>::uri())
//!     .and_then(|interface| interface.downcast_ref::<MyExtensionInterface>())
//!     .unwrap();
//!
//! unsafe { (extension.add_number)(&mut plugin as *mut _ as *mut c_void, 42) };
//!
//! assert_eq!(42, plugin.internal);
//! ```
use crate::UriBound;
use std::any::Any;

/// A descriptor for a plugin extension.
///
/// This trait is very minimal: It only contains a constant, static reference to the extension interface.
///
/// [For a usage example, see the module documentation.](index.html)
pub trait ExtensionDescriptor: UriBound {
    type ExtensionInterface: 'static + Any;

    const INTERFACE: &'static Self::ExtensionInterface;
}

/// Generate the body of a plugin's `extension_data` function.
///
/// This macro takes a URI as it's first argument, followed by a list of extension descriptors. This will
/// create a match expression that matches the given URI with the URIs of the extension descriptors. If one of the extension URIs matches, the statement returns the interface of the descriptor.
///
/// The generated statement returns a value of `Option<&'static dyn std::any::Any>`.
///
/// See the documentation of the `extension` module for more information on how to use this macro.
#[macro_export]
macro_rules! match_extensions {
    ($uri:expr, $($descriptor:ty),*) => {
        match ($uri).to_bytes_with_nul() {
            $(
                <$descriptor as ::lv2_core::UriBound>::URI => Some(<$descriptor as ::lv2_core::extension::ExtensionDescriptor>::INTERFACE as &'static dyn std::any::Any),
            )*
            _ => None,
        }
    };
}

pub use crate::match_extensions;
