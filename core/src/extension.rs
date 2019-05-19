use crate::plugin::Plugin;
use crate::uri::Uri;
use std::marker::PhantomData;
use std::os::raw::c_void;

pub trait Extension<P: Plugin> {
    const URI: &'static [u8];
    const RAW_DATA: *mut c_void;

    const DESCRIPTOR: ExtensionDescriptor<P> = ExtensionDescriptor {
        uri: Self::URI,
        raw_data: Self::RAW_DATA,
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
