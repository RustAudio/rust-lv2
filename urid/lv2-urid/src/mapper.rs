use std::ffi::c_void;
use std::os::raw::c_char;
use std::pin::Pin;
use std::ptr::null;
use urid::*;

pub struct HostMap<M: Map + Unmap + Unpin> {
    internal_map: M,
}

impl<M: Map + Unmap + Unpin> From<M> for HostMap<M> {
    fn from(map: M) -> Self {
        HostMap { internal_map: map }
    }
}

impl<M: Map + Unmap + Unpin> HostMap<M> {
    /// Unsafe wrapper of the `map` method, used by the feature interface.
    ///
    /// If the `map` method returns `None`, this method will return `0`.
    ///
    /// # Safety
    ///
    /// This method is unsafe since it has to dereference a raw pointer and since it's part of the C interface.
    pub unsafe extern "C" fn extern_map(
        handle: crate::sys::LV2_URID_Map_Handle,
        uri: *const c_char,
    ) -> crate::sys::LV2_URID {
        match (*(handle as *const Self))
            .internal_map
            .map_uri(Uri::from_ptr(uri))
        {
            Some(urid) => urid.get(),
            _ => 0,
        }
    }

    /// Create a raw map interface.
    pub fn make_map_interface(self: Pin<&mut Self>) -> sys::LV2_URID_Map {
        sys::LV2_URID_Map {
            handle: self.get_mut() as *mut Self as *mut c_void,
            map: Some(Self::extern_map),
        }
    }

    /// Unsafe wrapper of the `unmap` method, used by the feature interface.
    ///
    /// If the given URID is invalid or `unmap` returns `None`, this method returns a null pointer.
    ///
    /// # Safety
    ///
    /// The method is unsafe since it has to dereference raw pointers and it is part of the C interface.
    pub unsafe extern "C" fn extern_unmap(
        handle: crate::sys::LV2_URID_Map_Handle,
        urid: crate::sys::LV2_URID,
    ) -> *const c_char {
        match URID::new(urid) {
            Some(urid) => match (*(handle as *const Self)).internal_map.unmap(urid) {
                Some(uri) => uri.as_ptr(),
                None => null(),
            },
            None => null(),
        }
    }

    /// Create an unmap interface.
    ///
    /// This method clones the mapper and creates a self-contained `UnmapInterface`.
    pub fn make_unmap_interface(self: Pin<&mut Self>) -> sys::LV2_URID_Unmap {
        sys::LV2_URID_Unmap {
            handle: self.get_mut() as *mut Self as *mut c_void,
            unmap: Some(Self::extern_unmap),
        }
    }
}
