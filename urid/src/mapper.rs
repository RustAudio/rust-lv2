//! Implementation of the mapping feature for testing purposes.
use crate::feature::{URIDMap, URIDUnmap};
use crate::{Uri, URID};
use std::os::raw::*;
use std::pin::Pin;
use std::ptr::null;
use std::sync::RwLock;

/// A simple URI → URID mapper, backed by a standard `HashMap` and a `Mutex` for multi-thread
/// access.
#[derive(Default)]
pub struct HostURIDMapper {
    map: RwLock<Vec<Pin<Box<Uri>>>>,
}

impl URIDMap for HostURIDMapper {
    fn map_uri(&self, uri: &Uri) -> Option<URID<()>> {
        for (stored_urid, stored_uri) in self.map.read().ok()?.iter().enumerate() {
            if uri == stored_uri.as_ref().get_ref() {
                return URID::new((stored_urid + 1) as u32);
            }
        }

        let mut map = self.map.write().ok()?;
        map.push(Pin::new(Box::from(uri)));
        URID::new(map.len() as u32)
    }
}

impl URIDUnmap for HostURIDMapper {
    fn unmap<T: ?Sized>(&self, urid: URID<T>) -> Option<&Uri> {
        let map = self.map.read().ok()?;
        let uri = map.get((urid.get() - 1) as usize)?.as_ref().get_ref();

        // Faking the lifetime of the returned URI.
        // This is sound since we don't allow the deallocation of URIDs (which would introduce
        // many other problems). Therefore, none of the stored URIs are deallocated as long as the
        // mapper lives, which corresponds to the lifetime of the returned reference.
        // A URI is also never altered. Therefore, we may return an immutable reference.
        let ptr = uri.as_ptr();
        let uri = unsafe { Uri::from_ptr(ptr) };

        Some(uri)
    }
}

impl HostURIDMapper {
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
        match (*(handle as *const Self)).map_uri(Uri::from_ptr(uri)) {
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
            Some(urid) => match (*(handle as *mut Self)).unmap(urid) {
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

impl HostURIDMapper {
    /// Create a new URID map store.
    pub fn new() -> Self {
        Default::default()
    }
}
