use crate::plugin::Plugin;
use crate::uri::{Uri, UriBound};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;

pub unsafe trait Extension<P: Plugin>: UriBound {
    const RAW_DATA: &'static Any;

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
