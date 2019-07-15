pub extern crate lv2_core_sys as sys;

pub mod extension;
pub mod feature;
pub mod plugin;

use std::ffi::CStr;

pub unsafe trait UriBound {
    const URI: &'static [u8];

    #[inline]
    fn uri() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::URI) }
    }
}
