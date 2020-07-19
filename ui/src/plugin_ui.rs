use lv2_sys as sys;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::str::Utf8Error;

use lv2_core::prelude::*;
use std::fmt::Debug;
use urid::*;

use crate::port::*;

#[derive(Debug)]
pub enum PluginUIInfoError {
    InvalidBundlePathUtf8(Utf8Error),
}

/// Wrapper for the LV2UI_Write_Function
pub struct PluginPortWriteHandle {
    write_function: sys::LV2UI_Write_Function,
    controller: sys::LV2UI_Controller,
}

impl PluginPortWriteHandle {
    pub fn write_port(&self, port: &impl UIPort) {
        if let Some(write_function) = self.write_function {
            unsafe {
                write_function(
                    self.controller,
                    port.index(),
                    port.size() as u32,
                    port.protocol(),
                    port.data(),
                );
            }
        }
    }
}

/// Information about the Plugin UI
///
/// Holds the URIs of Plugin and UI as well as athe bundle path
pub struct PluginUIInfo<'a> {
    plugin_uri: &'a Uri,
    ui_uri: &'a Uri,
    bundle_path: &'a Path,
}

impl<'a> PluginUIInfo<'a> {
    /// Instanciate a PluginUIInfo from a set of raw pointers
    ///
    unsafe fn from_raw(
        descriptor: *const sys::LV2UI_Descriptor,
        plugin_uri: *const c_char,
        bundle_path: *const c_char,
    ) -> Result<Self, PluginUIInfoError> {
        let bundle_path = Path::new(
            Uri::from_ptr(bundle_path)
                .to_str()
                .map_err(PluginUIInfoError::InvalidBundlePathUtf8)?,
        );
        Ok(Self::new(
            Uri::from_ptr(plugin_uri),
            Uri::from_ptr((*descriptor).URI),
            bundle_path,
        ))
    }

    pub fn new(plugin_uri: &'a Uri, ui_uri: &'a Uri, bundle_path: &'a Path) -> Self {
        Self {
            plugin_uri,
            ui_uri,
            bundle_path,
        }
    }

    /// The URI of the plugin that is being instantiated.
    pub fn plugin_uri(&self) -> &Uri {
        self.plugin_uri
    }

    /// The URI of the UI that is being instantiated.
    pub fn ui_uri(&self) -> &Uri {
        self.ui_uri
    }

    /// The path to the LV2 bundle directory which contains this plugin binary.
    ///
    /// This is useful to get if the plugin needs to store extra resources in its bundle directory,
    /// such as presets, or any other kind of data.
    pub fn bundle_path(&self) -> &Path {
        self.bundle_path
    }
}

/// The central trait to describe the LV2 Plugin UI
///
/// This trait and the structs that implement it are the centre of
/// every plugin UI, it hosts the methods that are called when the
/// hosts wants to pass information to the plugin.
///
/// However, the host will not directly talk to the PluginUI
/// object. Instead, it will create and talk to the PluginUIInstance,
/// which dereferences raw pointers, does safety checks and then calls
/// the corresponding methods in PluginUI.
///
pub trait PluginUI: Sized + 'static {
    /// The type of the port collection
    type UIPorts: UIPortCollection;

    /// The host features used by this plugin UI
    ///
    /// This collection will be created by the framework when the
    /// plugin UI is initialized.
    ///
    /// If a host feature is missing, the plugin UI creation simply
    /// fails and your plugin host will tell you so.
    type InitFeatures: FeatureCollection<'static>;

    /// Create a plugin UI instance
    fn new(
        plugin_ui_info: &PluginUIInfo,
        features: &mut Self::InitFeatures,
        parent_window: *mut std::ffi::c_void,
        write_handle: PluginPortWriteHandle,
    ) -> Option<Self>;

    /// Cleanup the PluguinUI
    fn cleanup(&mut self);

    /// Supposed to return a mutable reference to the UI's port collection
    fn ports(&mut self) -> &mut Self::UIPorts;

    /// Called when some port has been updated.
    ///
    /// The plugin UI then should check all its ports what has changed
    /// and trigger repaint (exposure) events to update the UI
    /// accordingly.
    fn update(&mut self);

    /// Called periodically from the hosts. The UI then can process UI
    /// events and communicate events back to the plugin by updating
    /// its ports.
    fn idle(&mut self) -> i32;

    /// Supposed to return the LV2UI_Widget pointer
    fn widget(&self) -> sys::LV2UI_Widget;

    /// Updates a specific ports, when the host wants to message.
    /// Neither to be called manually nor to be reimplemented
    fn port_event(
        &mut self,
        port_index: u32,
        buffer_size: u32,
        format: u32,
        buffer: *const std::ffi::c_void,
    ) {
        self.ports()
            .port_event(port_index, buffer_size, format, buffer);
        self.update();
    }
}

#[repr(C)]
pub struct PluginUIInstance<T: PluginUI> {
    instance: T,
    widget: sys::LV2UI_Widget,
    features: *const *const sys::LV2_Feature,
}

fn retrieve_parent_window(features: *const *const sys::LV2_Feature) -> *mut std::ffi::c_void {
    let mut fptr = features;

    while !fptr.is_null() {
        unsafe {
            if CStr::from_ptr((**fptr).URI)
                == CStr::from_bytes_with_nul_unchecked(sys::LV2_UI__parent)
            {
                return (**fptr).data;
            }
            fptr = fptr.add(1);
        }
    }
    std::ptr::null_mut()
}

impl<T: PluginUI> PluginUIInstance<T> {
    /// The `instatiate()` function of the LV2UI_Descriptor stcuct
    ///
    /// Instanciates the UI instance and the UI itself
    ///
    /// Only to be called by the host.
    ///
    /// # Safety
    ///
    /// Unsafe because it derefenrences raw pointers reveived from the host.
    pub unsafe extern "C" fn instantiate(
        descriptor: *const sys::LV2UI_Descriptor,
        plugin_uri: *const c_char,
        bundle_path: *const c_char,
        write_function: sys::LV2UI_Write_Function,
        controller: sys::LV2UI_Controller,
        widget: *mut sys::LV2UI_Widget,
        features: *const *const sys::LV2_Feature,
    ) -> sys::LV2UI_Handle {
        let descriptor = match descriptor.as_ref() {
            Some(descriptor) => descriptor,
            None => {
                eprintln!("Failed to initialize plugin UI: Descriptor points to null");
                return std::ptr::null_mut();
            }
        };

        let plugin_ui_info = match PluginUIInfo::from_raw(descriptor, plugin_uri, bundle_path) {
            Ok(info) => info,
            Err(e) => {
                eprintln!(
                    "Failed to initialize plugin: Illegal info from host: {:?}",
                    e
                );
                return std::ptr::null_mut();
            }
        };

        let mut feature_cache = FeatureCache::from_raw(features);

        let parent_widget = retrieve_parent_window(features);

        let mut init_features =
            match T::InitFeatures::from_cache(&mut feature_cache, ThreadingClass::Instantiation) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("extension data {}", e);
                    return std::ptr::null_mut();
                }
            };

        let write_handle = PluginPortWriteHandle {
            write_function,
            controller,
        };

        match T::new(
            &plugin_ui_info,
            &mut init_features,
            parent_widget,
            write_handle,
        ) {
            Some(instance) => {
                *widget = instance.widget();
                let handle = Box::new(Self {
                    instance,
                    widget: *widget,
                    features,
                });
                Box::leak(handle) as *mut Self as sys::LV2UI_Handle
            }
            None => std::ptr::null_mut(),
        }
    }

    /// The `cleanup()` function of the LV2UI_Descriptor stcuct
    ///
    /// Forwards the port event to the UI
    ///
    /// Only to be called by the host.
    ///
    /// # Safety
    ///
    /// Unsafe because it derefenrences a raw pointer to the UI
    /// reveived from the host.
    pub unsafe extern "C" fn cleanup(handle: sys::LV2UI_Handle) {
        let handle = handle as *mut Self;
        (*handle).instance.cleanup();
    }

    /// The `port_event()` function of the LV2UI_Descriptor stcuct
    ///
    /// Forwards the port event to the UI instantce so that the pords
    /// get updated.
    ///
    /// Only to be called by the host.
    ///
    /// # Safety
    ///
    /// Unsafe because it derefenrences a raw pointer to the UI
    /// reveived from the host.
    pub unsafe extern "C" fn port_event(
        handle: sys::LV2UI_Handle,
        port_index: u32,
        buffer_size: u32,
        format: u32,
        buffer: *const std::ffi::c_void,
    ) {
        let handle = handle as *mut Self;
        (*handle)
            .instance
            .port_event(port_index, buffer_size, format, buffer);
    }

    /// The `extension_data()` function of the LV2UI_Descriptor stcuct
    ///
    /// Only to be called by the host
    ///
    /// # Safety
    ///
    /// Unsafe because it derefenrences a raw pointer to the UI
    /// reveived from the host.
    ///
    /// # Todo
    ///
    /// needs to be forwared to the UI so that the UI can hand back
    /// its extensions.
    pub unsafe extern "C" fn extension_data(uri: *const c_char) -> *const std::ffi::c_void {
        if CStr::from_ptr(uri) == CStr::from_bytes_with_nul_unchecked(sys::LV2_UI__idleInterface) {
            let interface = Box::new(sys::LV2UI_Idle_Interface {
                idle: Some(Self::idle),
            });
            Box::leak(interface) as *mut sys::LV2UI_Idle_Interface as *const std::ffi::c_void
        } else {
            std::ptr::null()
        }
    }

    /// The `idle()` function of the LV2UI_Descriptor stcuct
    ///
    /// Forwarded to the UI
    ///
    /// Only to be called by the host
    ///
    /// # Safety
    ///
    /// Unsafe because it derefenrences a raw pointer to the UI
    /// reveived from the host.
    pub unsafe extern "C" fn idle(handle: sys::LV2UI_Handle) -> i32 {
        let handle = handle as *mut Self;
        (*handle).instance.idle()
    }
}

pub unsafe trait PluginUIInstanceDescriptor {
    const DESCRIPTOR: sys::LV2UI_Descriptor;
}
