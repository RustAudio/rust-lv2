pub extern crate lv2_core_sys as sys;

mod extension_data;

pub mod feature;
pub mod plugin;

pub use self::extension_data::*;

use std::ffi::CStr;

pub unsafe trait UriBound {
    const URI: &'static [u8];

    #[inline]
    fn uri() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::URI) }
    }
}
