use crate::plugin::Plugin;
use crate::uri::{Uri, UriBound};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;

/// A trait for marking a type as an LV2 Plugin extension.
///
/// # Unsafety
///
/// The `RAW_DATA` field can be set to any static value, it is up to the implementer of the
/// extension to set it correctly:
///
/// * The reference must point to a structure with the exact same fields as the one defined by the
///   LV2 Extension specification;
/// * The struct being pointed to must be `'static`, i.e. available forever;
/// * The struct being pointed to must be correctly initialized, as defined by the LV2 Extension
///   specification;
/// * The struct being pointed to must be `#[repr(C)]`.
/// * The URI associated to this extension must be the exact same as the one defined in the LV2
///   Extension specification, and must also be the one tied to the struct being pointed to by
///   `RAW_DATA`.
///
/// Failing to meet any of these requirements will lead to undefined behavior when the host will
/// try to load the extension for any given plugin.
///
pub unsafe trait Extension<P: Plugin>: UriBound {
    /// The raw data structure defined by the extension, as returned by the plugin's `extension_data()` method.
    const RAW_DATA: &'static (dyn Any + 'static);

    /// The descriptor corresponding to an implementation of the extension.
    ///
    /// This const is automatically defined by the Extension trait, and cannot be changed.
    ///
    /// Most (if not all) users should not be concerned by this field, its only use is to provide
    /// runtime information for the extension, currently only used by this crate's implementation
    /// of the `extension_data()` method.
    ///
    /// See the `ExtensionDescriptor` struct documentation for more information.
    const DESCRIPTOR: ExtensionDescriptor<P> = ExtensionDescriptor {
        uri: Self::URI,
        raw_data: Self::RAW_DATA as *const _ as *mut _,
        _plugin: PhantomData,
    };
}

pub struct ExtensionDescriptor<P: Plugin> {
    uri: &'static [u8],
    raw_data: *mut c_void,
    _plugin: PhantomData<P>,
}

impl<P: Plugin> ExtensionDescriptor<P> {
    pub fn uri(&self) -> &Uri {
        unsafe { Uri::from_bytes_unchecked(self.uri) }
    }

    pub fn raw_data(&self) -> *mut c_void {
        self.raw_data
    }
}

#[macro_export]
macro_rules! lv2_extensions {
    ($($extension:ty),*) => {
        const EXTENSIONS: &'static [lv2_core::extension::ExtensionDescriptor<Self>] = &[
            $(<$extension as lv2_core::extension::Extension<Self>>::DESCRIPTOR,)*
        ];
    };
}
